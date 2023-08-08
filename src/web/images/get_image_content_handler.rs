use axum::{
    body::{self, boxed, Body, StreamBody},
    http::{Response, StatusCode},
    Json,
};
use serde::Serialize;
use serde_json::json;
use tokio_util::io::{ReaderStream, StreamReader};

use crate::{
    error::YaissError,
    images::{
        data_storage::images_sqlite_ds::ImagesSqliteDS,
        domain::image::Image,
        services::get_image_content_service::{
            GetImageContentService, GetImageContentServiceError,
        },
    },
    state::State,
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

pub async fn get_image_content(
    axum::extract::State(state): axum::extract::State<State>,
    identifier: axum::extract::Path<i64>,
) -> Result<Response<Body>, YaissError> {
    let storage = ImagesSqliteDS::new(state.pool());
    let service = GetImageContentService::new(storage);
    let builder = Response::builder();
    let builder = match service.get_image_content(identifier.0).await {
        Ok(image) => {
            let file = tokio::fs::File::open("Cargo.toml").await?;
            let stream = ReaderStream::new(file);
            let body = StreamBody::new(stream);

            return builder
                .status(StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "image/qoi")
                .header(
                    axum::http::header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"image.qoi\""),
                )
                .body(body::Body::from(body));
        }
        Err(GetImageContentServiceError::InternalError) => {
            let body = Json(json!({
                "error": "internal error getting image",
            }))
            .to_string();
            builder
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(body::Body::from(body))
        }
        Err(GetImageContentServiceError::ImageNotFound) => {
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
