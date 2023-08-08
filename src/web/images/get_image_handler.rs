use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde::Serialize;
use serde_json::json;

use crate::{
    error::YaissError,
    services::images::{
        domain::image::Image,
        ports::incoming::query_image_service::{QueryImageService, QueryImageServiceError},
    },
};

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

pub(crate) type DynQueryImageService = Arc<dyn QueryImageService + Sync + Send>;
pub async fn get_image(
    axum::extract::State(service): axum::extract::State<DynQueryImageService>,
    identifier: axum::extract::Path<i64>,
) -> Result<Response<Body>, YaissError> {
    let service = service.clone();
    let builder = Response::builder();
    let builder = match service.query_image(identifier.0).await {
        Ok(image) => {
            let image_json = ImageJson::from(image);
            let body = Json(json!(image_json)).to_string();
            builder.status(StatusCode::OK).body(body::Body::from(body))
        }
        Err(QueryImageServiceError::InternalError) => {
            let body = Json(json!({
                "error": "internal error deleting image",
            }))
            .to_string();
            builder
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(body::Body::from(body))
        }
        Err(QueryImageServiceError::ImageNotFound) => {
            let body = Json(json!({
                "error": "image not found",
            }))
            .to_string();
            builder
                .status(StatusCode::NOT_FOUND)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(body::Body::from(body))
        }
    };
    builder.map_err(|e| e.into())
}
