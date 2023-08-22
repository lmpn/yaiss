use std::{io, sync::Arc};

use axum::{
    body::{self},
    debug_handler,
    http::StatusCode,
    response::Response,
};
use futures::TryStreamExt;
use tokio_util::io::StreamReader;

use crate::{
    error::YaissError,
    services::images::ports::incoming::upload_images_service::UploadImagesService,
};

pub(crate) type DynUploadImagesService = Arc<dyn UploadImagesService + Send + Sync>;

#[debug_handler]
pub async fn upload_image(
    axum::extract::State(service): axum::extract::State<DynUploadImagesService>,
    mut multipart: axum::extract::Multipart,
) -> Result<Response<body::Body>, YaissError> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let mp_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let reader = StreamReader::new(mp_with_io_error);
        futures::pin_mut!(reader);
        let mut buffer = vec![];
        tokio::io::copy(&mut reader, &mut buffer).await?;
        let service = service.clone();
        let handle = tokio::task::spawn(async move { service.upload_image(buffer).await });
        handle.await??;
    }
    Response::builder()
        .status(StatusCode::CREATED)
        .body(body::Body::empty())
        .map_err(|e| e.into())
}
