use crate::{
    app::{AppError, AppResult, ParamsError},
    RedisPool,
};
use chrono::Utc;
use mobc_redis::{
    mobc::Connection,
    redis::{
        geo::{RadiusOptions, RadiusSearchResult, Unit},
        AsyncCommands,
    },
    RedisConnectionManager,
};
use std::{collections::HashSet, convert::TryFrom, sync::Arc};
use uuid::Uuid;
use webserver_contracts::{shape::geojson::*, shape::*, *};
use webserver_database::{Database, InsertionResult, Shape as DbShape, ShapeTag as DbShapeTag};

const GEO_KEY: &str = "Shape:Geo";

pub struct ShapeController {
    pool: Arc<RedisPool>,
    shape_db: Arc<Database<DbShape>>,
}

impl ShapeController {
    pub fn new(pool: Arc<RedisPool>, shape_db: Arc<Database<DbShape>>) -> Self {
        Self { pool, shape_db }
    }

    pub async fn add_shape(&self, request: JsonRpcRequest) -> AppResult<AddShapeResult> {
        let params = AddShapeParams::try_from(request)?;
        let shape = params.shape;
        let id = shape.id.to_string();

        let (db_shape, db_shape_tags) = make_db_entities(shape.clone());
        let result = self
            .shape_db
            .insert_shape(&db_shape, &db_shape_tags.iter().collect::<Vec<_>>())?;

        match result {
            InsertionResult::Inserted => {
                self.add_points_to_redis(&shape).await?;
                Ok(AddShapeResult::success(id))
            }
            InsertionResult::AlreadyExists => Ok(AddShapeResult::failure()),
        }
    }

    pub async fn delete_shape(&self, request: JsonRpcRequest) -> AppResult<DeleteShapeResult> {
        let params = DeleteShapeParams::try_from(request)?;

        let success = self.shape_db.delete_shape(&params.id.to_string())?;

        Ok(DeleteShapeResult::new(success))
    }

    pub async fn get_shape(&self, request: JsonRpcRequest) -> AppResult<GetShapeResult> {
        let params = GetShapeParams::try_from(request)?;

        let shape = self.shape_db.get_shape(&params.id.to_string())?;

        let shape = match shape {
            Some(db_shape) => db_shape,
            None => return Ok(GetShapeResult::new(None)),
        };

        let tags = self.get_tags_for_shape(&shape.id)?;

        let shape_result = ShapeWrapper::try_from((shape, tags))?;
        Ok(GetShapeResult::new(Some(shape_result.0)))
    }

    pub async fn get_nearby_shapes(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<GetNearbyShapesResult> {
        let params = GetNearbyShapesParams::try_from(request)?;

        let mut conn = self.get_connection().await?;

        let opts = RadiusOptions::default().limit(params.count);
        let results: Vec<RadiusSearchResult> = conn
            .geo_radius(
                GEO_KEY,
                params.lon,
                params.lat,
                params.distance_m as f64,
                Unit::Meters,
                opts,
            )
            .await?;

        let ids: HashSet<_> = results
            .into_iter()
            .filter_map(|r| {
                let parts: Vec<_> = r.name.split("_").collect();
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

        let with_tags = self.get_shapes_with_tags(&ids)?;

        let out: Vec<_> = with_tags
            .into_iter()
            .map(|(shape, tags)| ShapeWrapper::try_from((shape, tags)).map(|w| w.0))
            .collect::<Result<_, _>>()?;

        Ok(GetNearbyShapesResult::new(out))
    }

    pub async fn add_shape_tag(&self, request: JsonRpcRequest) -> AppResult<AddShapeTagResult> {
        let params = AddShapeTagParams::try_from(request)?;
        let created_s = Utc::now().timestamp();

        let id = uuid::Uuid::new_v4().to_string();

        let result = self.shape_db.insert_shape_tag(&DbShapeTag::new(
            id.clone(),
            params.shape_id.to_string(),
            params.name,
            params.value,
            created_s,
        ))?;

        match result {
            InsertionResult::Inserted => Ok(AddShapeTagResult::success(id)),
            InsertionResult::AlreadyExists => Ok(AddShapeTagResult::failure()),
        }
    }

    pub async fn delete_shape_tag(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<DeleteShapeTagResult> {
        let params = DeleteShapeTagParams::try_from(request)?;

        let success = self.shape_db.delete_tag(&params.id.to_string())?;

        Ok(DeleteShapeTagResult::new(success))
    }

    pub async fn search_shapes_by_tags(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<SearchShapesByTagsResult> {
        let params = SearchShapesByTagsParams::try_from(request)?;

        let mut shapes = Vec::new();
        for tags_query in params.or {
            let mut tags = self.shape_db.get_shapes_with_tags(&tags_query)?;
            shapes.append(&mut tags);
        }

        let mut pairs = Vec::new();
        for shape in shapes {
            if let Ok(tags) = self.get_tags_for_shape(&shape.id) {
                pairs.push((shape, tags));
            } else {
                error!("could not retrieve tags for shape: '{}'", shape.id);
            }
        }

        let shapes_out: Vec<_> = pairs
            .into_iter()
            .map(|(shape, tags)| ShapeWrapper::try_from((shape, tags)).map(|w| w.0))
            .collect::<Result<_, _>>()?;

        Ok(SearchShapesByTagsResult::new(shapes_out))
    }

    fn get_shapes_with_tags(
        &self,
        shape_ids: &HashSet<Uuid>,
    ) -> AppResult<Vec<(DbShape, Vec<DbShapeTag>)>> {
        let ids: Vec<String> = shape_ids.iter().map(|s| s.to_string()).collect();
        let ids: Vec<_> = ids.iter().map(|id| id.as_str()).collect();
        let db_shapes = self.shape_db.get_shapes_by_ids(&ids)?;

        let mut with_tags = Vec::new();
        for db_shape in db_shapes {
            let tags = self.get_tags_for_shape(&db_shape.id)?;
            with_tags.push((db_shape, tags));
        }

        Ok(with_tags)
    }

    fn get_tags_for_shape(&self, shape_id: &str) -> AppResult<Vec<DbShapeTag>> {
        let shapes = self.shape_db.get_tags_for_shape(shape_id)?;
        Ok(shapes)
    }

    async fn add_points_to_redis(&self, shape: &Shape) -> AppResult<()> {
        let mut conn = self.get_connection().await?;

        let members: Vec<_> = shape
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
            .collect();

        conn.geo_add(GEO_KEY, members).await?;

        Ok(())
    }

    async fn get_connection(&self) -> Result<Connection<RedisConnectionManager>, AppError> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => Err(AppError::from(e)),
        }
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

fn make_db_entities(shape: Shape) -> (DbShape, Vec<DbShapeTag>) {
    let created_s = Utc::now().timestamp();
    let shape_id = shape.id.to_string();
    let db_shape = DbShape::new(
        shape.id.to_string(),
        shape.name,
        serde_json::to_string(&shape.geo).unwrap(),
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

impl ParamsError for AddShapeParamsInvalid {}
impl ParamsError for AddShapesParamsInvalid {}
impl ParamsError for GetShapeParamsInvalid {}
impl ParamsError for GetNearbyShapesParamsInvalid {}
impl ParamsError for AddShapeTagParamsInvalid {}
impl ParamsError for SearchShapesByTagsParamsInvalid {}
impl ParamsError for DeleteShapeParamsInvalid {}
impl ParamsError for DeleteShapeTagParamsInvalid {}
