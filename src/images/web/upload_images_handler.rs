use std::io;

use axum::{body::Empty, debug_handler, http::StatusCode, response::Response};
use futures::TryStreamExt;
use tokio_util::io::StreamReader;

use crate::{
    error::YaissError,
    images::{
        data_storage::images_sqlite_ds::ImagesSqliteDS,
        services::upload_images_service::UploadImagesService,
    },
    state::State,
};

#[debug_handler]
pub async fn upload_image(
    axum::extract::State(state): axum::extract::State<State>,
    mut multipart: axum::extract::Multipart,
) -> Result<Response<Empty<axum::body::Bytes>>, YaissError> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let mp_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let reader = StreamReader::new(mp_with_io_error);
        futures::pin_mut!(reader);
        let mut buffer = vec![];
        tokio::io::copy(&mut reader, &mut buffer).await?;

        let storage = ImagesSqliteDS::new(state.pool());
        let service = UploadImagesService::new(storage, state.images_base_path().to_string());

        let handle = tokio::task::spawn(async move { service.upload_image(buffer).await });
        handle.await??;
    }
    Response::builder()
        .status(StatusCode::CREATED)
        .body(Empty::new())
        .map_err(|e| e.into())
}
