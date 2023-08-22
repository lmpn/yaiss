use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde_json::json;

use crate::{
    error::YaissError, services::images::ports::incoming::delete_image_service::DeleteImageService,
    services::images::ports::incoming::delete_image_service::DeleteImageServiceError,
};

pub(crate) type DynDeleteImagesService = Arc<dyn DeleteImageService + Send + Sync>;

pub async fn delete_image_handler(
    axum::extract::State(service): axum::extract::State<DynDeleteImagesService>,
    identifier: axum::extract::Path<i64>,
) -> Result<Response<Body>, YaissError> {
    let service = service.clone();
    let builder = Response::builder();
    let builder = match service.delete_image(identifier.0).await {
        Ok(()) => builder.status(StatusCode::OK).body(Body::empty()),
        Err(DeleteImageServiceError::InternalError) => {
            let body = Json(json!({
                "error": "internal error deleting image",
            }))
            .to_string();
            builder
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(body::Body::from(body))
        }
        Err(DeleteImageServiceError::ImageNotFound) => {
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

#[cfg(test)]
mod tests {}
