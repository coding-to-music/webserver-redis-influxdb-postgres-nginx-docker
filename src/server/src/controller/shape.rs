use crate::app::{AppError, AppResult, ParamsError};
use chrono::Utc;
use database::{Database, InsertionResult, Shape as DbShape, ShapeTag as DbShapeTag};
use model::{shape::geojson::*, shape::*, *};
use redis::{
    mobc_redis::{
        mobc::Connection,
        redis::{
            geo::{RadiusOptions, RadiusSearchResult, Unit},
            AsyncCommands,
        },
        RedisConnectionManager,
    },
    RedisPool,
};
use std::{collections::HashSet, convert::TryFrom, sync::Arc};
use uuid::Uuid;

const GEO_KEY: &str = "Shape:Geo";

pub struct ShapeController {
    redis: Arc<RedisPool>,
    shape_db: Arc<Database<DbShape>>,
}

impl ShapeController {
    pub fn new(redis: Arc<RedisPool>, shape_db: Arc<Database<DbShape>>) -> Self {
        Self { redis, shape_db }
    }

    pub async fn add_shape(&self, request: JsonRpcRequest) -> AppResult<add_shape::MethodResult> {
        use add_shape::{MethodResult, Params};
        let params = Params::try_from(request)?;
        let shape = params.shape;
        let id = shape.id.to_string();

        let (db_shape, db_shape_tags) = make_db_entities_for_insertion(shape.clone());
        let result = self
            .shape_db
            .insert_shape(&db_shape, &db_shape_tags.iter().collect::<Vec<_>>())
            .await?;

        match result {
            InsertionResult::Inserted => {
                let mut conn = self.redis.get_connection().await?;
                self.add_points_to_redis(&mut conn, &[shape]).await?;
                Ok(MethodResult::success(id))
            }
            InsertionResult::AlreadyExists => Ok(MethodResult::failure()),
        }
    }

    pub async fn delete_shape(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<delete_shape::MethodResult> {
        use delete_shape::{MethodResult, Params};
        let params = Params::try_from(request)?;
        let shape_id = params.id.to_string();

        let shape = self.shape_db.get_shape(&shape_id).await?;

        if let Some(db_shape) = shape {
            let success = self.shape_db.delete_shape(&shape_id).await?;
            if success {
                // delete points from redis
                let point_ids = geo_point_ids(&db_shape)?;
                self.delete_points_from_redis(&point_ids).await?;
                return Ok(MethodResult::new(success));
            }
        }

        Ok(MethodResult::new(false))
    }

    pub async fn get_shape(&self, request: JsonRpcRequest) -> AppResult<get_shape::MethodResult> {
        use get_shape::{MethodResult, Params};
        let params = Params::try_from(request)?;
        let as_geojson = params.geojson.unwrap_or(false);

        let shape = self.shape_db.get_shape(&params.id.to_string()).await?;

        let shape = match shape {
            Some(db_shape) if db_shape.deleted_at_s.is_none() => db_shape,
            _ => {
                // the shape has either been deleted or it does not exist
                if as_geojson {
                    return Ok(MethodResult::geojson(None));
                } else {
                    return Ok(MethodResult::shape(None));
                }
            }
        };

        let tags = self.get_tags_for_shape(&shape.id).await?;

        let shape_result = ShapeWrapper::try_from((shape, tags))?.0;

        if as_geojson {
            Ok(MethodResult::geojson(Some(Feature::from(shape_result))))
        } else {
            Ok(MethodResult::shape(Some(shape_result)))
        }
    }

    pub async fn get_nearby_shapes(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<get_nearby_shapes::MethodResult> {
        use get_nearby_shapes::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let mut conn = self.redis.get_connection().await?;

        let results: Vec<RadiusSearchResult> = conn
            .geo_radius(
                GEO_KEY,
                params.lon,
                params.lat,
                params.distance_m as f64,
                Unit::Meters,
                RadiusOptions::default().limit(params.count),
            )
            .await?;

        let ids: HashSet<_> = results
            .into_iter()
            .filter_map(|r| {
                let parts: Vec<_> = r.name.split('_').collect();
                if parts.len() != 2 {
                    error!(
                        "failed to retrieve shape id from geo set member: '{}'",
                        r.name
                    );
                    None
                } else {
                    match Uuid::parse_str(parts[0]) {
                        Ok(id) => Some(id),
                        Err(_) => {
                            error!("failed to parse '{}' as uuid", parts[0]);
                            None
                        }
                    }
                }
            })
            .collect();

        let with_tags = self.get_shapes_with_tags(&ids).await?;

        let out: Vec<_> = with_tags
            .into_iter()
            .map(|(shape, tags)| ShapeWrapper::try_from((shape, tags)).map(|w| w.0))
            .collect::<Result<_, _>>()?;

        Ok(MethodResult::new(out))
    }

    pub async fn add_shape_tag(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<add_shape_tag::MethodResult> {
        let params = add_shape_tag::Params::try_from(request)?;
        let created_s = Utc::now().timestamp();

        let id = uuid::Uuid::new_v4().to_string();

        let result = self
            .shape_db
            .insert_shape_tag(&DbShapeTag::new(
                id.clone(),
                params.shape_id.to_string(),
                params.name,
                params.value,
                created_s,
            ))
            .await?;

        match result {
            InsertionResult::Inserted => Ok(add_shape_tag::MethodResult::success(id)),
            InsertionResult::AlreadyExists => Ok(add_shape_tag::MethodResult::failure()),
        }
    }

    pub async fn delete_shape_tag(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<delete_shape_tag::MethodResult> {
        use delete_shape_tag::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let success = self.shape_db.delete_tag(&params.id.to_string()).await?;

        Ok(MethodResult::new(success))
    }

    pub async fn search_shapes_by_tags(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<search_shapes_by_tags::MethodResult> {
        use search_shapes_by_tags::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let mut shapes = Vec::new();
        for tags_query in params.or {
            let mut tags = self.shape_db.get_shapes_with_tags(&tags_query).await?;
            shapes.append(&mut tags);
        }

        let mut pairs = Vec::new();
        for shape in shapes {
            if let Ok(tags) = self.get_tags_for_shape(&shape.id).await {
                pairs.push((shape, tags));
            } else {
                error!("could not retrieve tags for shape: '{}'", shape.id);
            }
        }

        let shapes_out: Vec<_> = pairs
            .into_iter()
            .map(|(shape, tags)| ShapeWrapper::try_from((shape, tags)).map(|w| w.0))
            .collect::<Result<_, _>>()?;

        Ok(MethodResult::new(shapes_out))
    }

    pub async fn refresh_geo_points_in_cache(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<refresh_geo_points_in_cache::MethodResult> {
        use refresh_geo_points_in_cache::{MethodResult, Params};
        let params = Params::try_from(request)?;
        info!("'{}': refreshing geo points", params.source);

        let shape_rows = self.shape_db.get_all_shapes().await?;

        let mut conn = self.redis.get_connection().await?;

        conn.del(GEO_KEY).await?;

        let mut shapes_to_add: Vec<_> = Vec::new();
        for db_shape in shape_rows {
            if db_shape.deleted_at_s.is_none() {
                let shape = ShapeWrapper::try_from((db_shape, vec![])).map(|w| w.0)?;
                shapes_to_add.push(shape);
            }
        }

        let count = self.add_points_to_redis(&mut conn, &shapes_to_add).await?;

        Ok(MethodResult::new(count))
    }

    async fn get_shapes_with_tags(
        &self,
        shape_ids: &HashSet<Uuid>,
    ) -> AppResult<Vec<(DbShape, Vec<DbShapeTag>)>> {
        let ids: Vec<String> = shape_ids.iter().map(|s| s.to_string()).collect();
        let ids: Vec<_> = ids.iter().map(|id| id.as_str()).collect();
        let db_shapes = self.shape_db.get_shapes_by_ids(&ids).await?;

        let mut with_tags = Vec::new();
        for db_shape in db_shapes {
            let tags = self.get_tags_for_shape(&db_shape.id).await?;
            with_tags.push((db_shape, tags));
        }

        Ok(with_tags)
    }

    async fn get_tags_for_shape(&self, shape_id: &str) -> AppResult<Vec<DbShapeTag>> {
        let shapes = self.shape_db.get_tags_for_shape(shape_id).await?;
        Ok(shapes)
    }

    /// Adds the geo points for the given shapes to cache.
    ///
    /// ## Returns
    /// The number of points that were added.
    async fn add_points_to_redis(
        &self,
        conn: &mut Connection<RedisConnectionManager>,
        shapes: &[Shape],
    ) -> AppResult<usize> {
        let mut members: Vec<_> = Vec::new();
        for shape in shapes {
            members.append(&mut Self::get_geo_members_from_shape(shape));
        }

        let count = members.len();

        conn.geo_add(GEO_KEY, members).await?;

        Ok(count)
    }

    async fn delete_points_from_redis(&self, members: &[String]) -> AppResult<usize> {
        let mut conn = self.redis.get_connection().await?;

        let result: usize = conn.zrem(GEO_KEY, members).await?;

        Ok(result)
    }

    fn get_geo_members_from_shape(shape: &Shape) -> Vec<(String, String, String)> {
        shape
            .coordinates()
            .iter()
            .enumerate()
            .map(|(idx, coord)| {
                (
                    coord.lon.to_string(),
                    coord.lat.to_string(),
                    format!("{}_{}", shape.id, idx),
                )
            })
            .collect()
    }
}

struct ShapeWrapper(Shape);

impl TryFrom<(DbShape, Vec<DbShapeTag>)> for ShapeWrapper {
    type Error = AppError;

    fn try_from((shape, tags): (DbShape, Vec<DbShapeTag>)) -> Result<Self, Self::Error> {
        let id: Uuid = shape
            .id
            .parse()
            .map_err(|e| AppError::internal_error().with_context(&e))?;
        let name = shape.name;
        let geometry: Geometry = serde_json::from_str(&shape.geo)
            .map_err(|e| AppError::internal_error().with_context(&e))?;

        let tags = tags
            .into_iter()
            .map(|db_shape_tag| (db_shape_tag.tag_name, db_shape_tag.tag_value))
            .collect();

        let shape = Shape::new(id, name, geometry, tags);
        Ok(Self(shape))
    }
}

fn make_db_entities_for_insertion(shape: Shape) -> (DbShape, Vec<DbShapeTag>) {
    let created_s = Utc::now().timestamp();
    let shape_id = shape.id.to_string();
    let db_shape = DbShape::new(
        shape.id.to_string(),
        shape.name,
        serde_json::to_string(&shape.geo).unwrap(),
        None,
        created_s,
    );
    let db_shape_tags = shape
        .tags
        .into_iter()
        .map(|(name, value)| {
            DbShapeTag::new(
                Uuid::new_v4().to_string(),
                shape_id.clone(),
                name,
                value,
                created_s,
            )
        })
        .collect();
    (db_shape, db_shape_tags)
}

fn geo_point_ids(db_shape: &DbShape) -> AppResult<Vec<String>> {
    let geo: Geometry = serde_json::from_str(&db_shape.geo)
        .map_err(|e| AppError::internal_error().with_context(&e))?;
    Ok(model::shape::coordinates_in_geo(&geo)
        .iter()
        .enumerate()
        .map(|(idx, _)| format!("{}_{}", db_shape.id, idx))
        .collect())
}

impl ParamsError for add_shape::InvalidParams {}
impl ParamsError for get_shape::InvalidParams {}
impl ParamsError for get_nearby_shapes::InvalidParams {}
impl ParamsError for add_shape_tag::InvalidParams {}
impl ParamsError for search_shapes_by_tags::InvalidParams {}
impl ParamsError for delete_shape::InvalidParams {}
impl ParamsError for delete_shape_tag::InvalidParams {}
impl ParamsError for refresh_geo_points_in_cache::InvalidParams {}
