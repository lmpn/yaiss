use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::{
    error::YaissError,
    services::images::{
        domain::image::Image,
        ports::incoming::batch_query_image_service::{
            BatchQueryImageService, BatchQueryImageServiceError,
        },
    },
};

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub count: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            count: 50,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageJson {
    id: i64,
    updated_on: String,
}

impl From<Image> for ImageJson {
    fn from(value: Image) -> Self {
        Self {
            id: value.id(),
            updated_on: value.updated_on().to_string(),
        }
    }
}

pub(crate) type DynBatchQueryImageService = Arc<dyn BatchQueryImageService + Send + Sync>;
pub async fn batch_get_image(
    axum::extract::State(service): axum::extract::State<DynBatchQueryImageService>,
    pagination: Option<axum::extract::Query<Pagination>>,
) -> Result<Response<Body>, YaissError> {
    let service = service.clone();
    let builder = Response::builder();
    let pagination = pagination.unwrap_or_default();
    let builder = match service
        .batch_query_image(pagination.count, pagination.offset)
        .await
    {
        Ok(images) => {
            let images = images
                .into_iter()
                .map(ImageJson::from)
                .collect::<Vec<ImageJson>>();
            let body = Json(json!({ "images": images })).to_string();
            builder.status(StatusCode::OK).body(body::Body::from(body))
        }
        Err(BatchQueryImageServiceError::InternalError) => builder
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::Body::from(
                BatchQueryImageServiceError::InternalError.to_string(),
            )),
        Err(BatchQueryImageServiceError::InvalidRequest) => builder
            .status(StatusCode::BAD_REQUEST)
            .body(body::Body::from(
                BatchQueryImageServiceError::InvalidRequest.to_string(),
            )),
        Err(BatchQueryImageServiceError::TooManyImagesRequested) => builder
            .status(StatusCode::BAD_REQUEST)
            .body(body::Body::from(
                BatchQueryImageServiceError::TooManyImagesRequested.to_string(),
            )),
        Err(BatchQueryImageServiceError::NoRecordsFound) => {
            info!("heee");
            builder.status(StatusCode::NOT_FOUND).body(body::Body::from(
                BatchQueryImageServiceError::NoRecordsFound.to_string(),
            ))
        }
    };
    builder.map_err(|e| e.into())
}
