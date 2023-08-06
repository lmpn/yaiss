use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde_json::json;

use crate::{
    error::YaissError,
    images::{
        data_storage::images_sqlite_ds::ImagesSqliteDS,
        services::delete_image_service::{DeleteImageService, DeleteImageServiceError},
    },
    state::State,
};

pub async fn delete_image_handler(
    axum::extract::State(state): axum::extract::State<State>,
    identifier: axum::extract::Path<i64>,
) -> Result<Response<Body>, YaissError> {
    let storage = ImagesSqliteDS::new(state.pool());
    let service = DeleteImageService::new(storage);
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
