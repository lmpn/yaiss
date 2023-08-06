use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
};
use tracing::info;

use crate::{
    error::YaissError,
    images::{
        data_storage::images_sqlite_ds::ImagesSqliteDS,
        services::delete_images_service::DeleteImagesService,
    },
    state::State,
};

pub async fn delete_images(
    axum::extract::State(state): axum::extract::State<State>,
    identifiers: axum::extract::Json<Vec<i64>>,
) -> Result<Response<Body>, YaissError> {
    info!("{:?}", identifiers.0);
    let storage = ImagesSqliteDS::new(state.pool());
    let service = DeleteImagesService::new(storage);
    let builder = Response::builder();
    let builder = match service.delete_images(identifiers.0).await {
        Ok(()) => builder.status(StatusCode::OK).body(Body::empty()),
        Err(e) => builder
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::Body::from(e.to_string())),
    };
    builder.map_err(|e| e.into())
}
