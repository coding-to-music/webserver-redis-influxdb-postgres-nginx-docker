use crate::app::AppError;
use chrono::Utc;
use std::{convert::TryFrom, sync::Arc};
use uuid::Uuid;
use webserver_contracts::shape::{
    geojson::Geometry, GetShapeParams, GetShapeParamsInvalid, GetShapeResult, Shape,
};
use webserver_contracts::{
    shape::{AddShapeParams, AddShapeParamsInvalid, AddShapeResult},
    JsonRpcError, JsonRpcRequest,
};
use webserver_database::{Database, Shape as DbShape};

pub struct ShapeController {
    _redis: redis::Client,
    shape_db: Arc<Database<DbShape>>,
}

impl ShapeController {
    pub fn new(addr: String, shape_db: Arc<Database<DbShape>>) -> Self {
        let _redis = redis::Client::open(addr).unwrap();
        Self { _redis, shape_db }
    }

    pub async fn add_shape(&self, request: JsonRpcRequest) -> Result<AddShapeResult, AppError> {
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

        if result {
            Ok(AddShapeResult::success(id))
        } else {
            Ok(AddShapeResult::failure())
        }
    }

    pub async fn get_shape(&self, request: JsonRpcRequest) -> Result<GetShapeResult, AppError> {
        let params = GetShapeParams::try_from(request)?;

        let shape = self.shape_db.get_shape(&params.id.to_string())?;

        match shape {
            Some(db_shape) => {
                let shape = ShapeWrapper::try_from(db_shape)?.0;
                Ok(GetShapeResult::new(Some(shape)))
            }
            None => Ok(GetShapeResult::new(None)),
        }
    }
}

impl From<AddShapeParamsInvalid> for AppError {
    fn from(err: AddShapeParamsInvalid) -> Self {
        match err {
            AddShapeParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_params().with_message(format!("{}", e)))
            }
        }
    }
}

impl From<GetShapeParamsInvalid> for AppError {
    fn from(err: GetShapeParamsInvalid) -> Self {
        match &err {
            GetShapeParamsInvalid::InvalidFormat(e) => AppError::invalid_params()
                .with_message(&format!("{}", err))
                .with_context(&e),
        }
    }
}

struct ShapeWrapper(Shape);

impl TryFrom<DbShape> for ShapeWrapper {
    type Error = AppError;

    fn try_from(value: DbShape) -> Result<Self, Self::Error> {
        let id: Uuid = value
            .id
            .parse()
            .map_err(|e| AppError::internal_error().with_context(&e))?;
        let name = value.name;
        let geometry: Geometry = serde_json::from_str(&value.geo)
            .map_err(|e| AppError::internal_error().with_context(&e))?;

        let shape = Shape::new(id, name, geometry);
        Ok(Self(shape))
    }
}
