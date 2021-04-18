use crate::app::{AppError, AppResult, ParamsError};
use chrono::Utc;
use std::{convert::TryFrom, sync::Arc};
use uuid::Uuid;
use webserver_contracts::{shape::geojson::*, shape::*, *};
use webserver_database::{Database, InsertionResult, Shape as DbShape, ShapeTag as DbShapeTag};

pub struct ShapeController {
    _redis: redis::Client,
    shape_db: Arc<Database<DbShape>>,
    shape_tag_db: Arc<Database<DbShapeTag>>,
}

impl ShapeController {
    pub fn new(
        addr: String,
        shape_db: Arc<Database<DbShape>>,
        shape_tag_db: Arc<Database<DbShapeTag>>,
    ) -> Self {
        let _redis = redis::Client::open(addr).unwrap();
        Self {
            _redis,
            shape_db,
            shape_tag_db,
        }
    }

    pub async fn add_shape(&self, request: JsonRpcRequest) -> AppResult<AddShapeResult> {
        let params = AddShapeParams::try_from(request)?;
        let created_s = Utc::now().timestamp();

        let shape = params.shape;
        let id = shape.id.to_string();

        let result = self.shape_db.insert_shape(
            &id,
            shape.name.as_ref().map(|s| s.as_str()),
            &serde_json::to_string(&shape.geo).unwrap(),
            created_s,
        )?;

        match result {
            InsertionResult::Inserted => Ok(AddShapeResult::success(id)),
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

    pub async fn add_shape_tag(&self, request: JsonRpcRequest) -> AppResult<AddShapeTagResult> {
        let params = AddShapeTagParams::try_from(request)?;
        let created_s = Utc::now().timestamp();

        let id = uuid::Uuid::new_v4().to_string();

        let result = self.shape_tag_db.insert_shape_tag(
            &id,
            &params.shape_id.to_string(),
            &params.name,
            &params.value,
            created_s,
        )?;

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

        let success = self.shape_tag_db.delete_tag(&params.id.to_string())?;

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

    fn get_tags_for_shape(&self, shape_id: &str) -> AppResult<Vec<DbShapeTag>> {
        let shapes = self.shape_tag_db.get_tags_for_shape(shape_id)?;
        Ok(shapes)
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

impl ParamsError for AddShapeParamsInvalid {}
impl ParamsError for GetShapeParamsInvalid {}
impl ParamsError for AddShapeTagParamsInvalid {}
impl ParamsError for SearchShapesByTagsParamsInvalid {}
impl ParamsError for DeleteShapeParamsInvalid {}
impl ParamsError for DeleteShapeTagParamsInvalid {}
