use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
};
use tracing::info;

use crate::{
    error::YaissError,
    services::images::ports::incoming::batch_delete_image_service::BatchDeleteImageService,
};
pub(crate) type DynBatchDeleteImageService = Arc<dyn BatchDeleteImageService + Send + Sync>;
pub async fn batch_delete_image(
    axum::extract::State(service): axum::extract::State<DynBatchDeleteImageService>,
    identifiers: axum::extract::Json<Vec<i64>>,
) -> Result<Response<Body>, YaissError> {
    let service = service.clone();
    info!("{:?}", identifiers.0);
    let builder = Response::builder();
    let builder = match service.batch_delete_image(identifiers.0).await {
        Ok(()) => builder.status(StatusCode::OK).body(Body::empty()),
        Err(e) => builder
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::Body::from(e.to_string())),
    };
    builder.map_err(|e| e.into())
}
