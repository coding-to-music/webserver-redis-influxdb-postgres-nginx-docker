use crate::app::AppError;
use chrono::Utc;
use std::{convert::TryFrom, sync::Arc};
use uuid::Uuid;
use webserver_contracts::shape::Shape;
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

        let id = get_id(&shape)?;

        let exists = self.shape_db.get_shape(&id.to_string())?.is_some();

        if exists {
            return Ok(AddShapeResult::new(false));
        }

        let name = get_name(&shape)?;
        let geo = serde_json::to_string(shape.geo())
            .map_err(|e| AppError::invalid_params("invalid geojson").with_context(&e))?;

        let result = self
            .shape_db
            .insert_shape(&id.to_string(), name, &geo, created_s)?;

        Ok(AddShapeResult::new(result))
    }
}

fn get_id(shape: &Shape) -> Result<Uuid, AppError> {
    let id = shape
        .get_property("id")
        .ok_or(AppError::invalid_params("feature missing property 'id'"))?;
    let id = id
        .as_str()
        .ok_or(AppError::invalid_params("property 'id' has wrong type"))?;
    let uuid: Uuid = id.parse().map_err(|e| {
        AppError::invalid_params("property 'id' has invalid value").with_context(&e)
    })?;
    Ok(uuid)
}

fn get_name(shape: &Shape) -> Result<Option<&str>, AppError> {
    let name = match shape.get_property("name") {
        Some(n) => n,
        None => {
            return Ok(None);
        }
    };

    let name = name
        .as_str()
        .ok_or(AppError::invalid_params("property 'name' has wrong type"))?;

    Ok(Some(name))
}

impl From<AddShapeParamsInvalid> for AppError {
    fn from(e: AddShapeParamsInvalid) -> Self {
        match e {
            AddShapeParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_params().with_message(format!("{}", e)))
            }
            AddShapeParamsInvalid::InvalidFeature(message) => {
                AppError::from(JsonRpcError::invalid_params().with_message(format!("{}", message)))
            }
        }
    }
}
