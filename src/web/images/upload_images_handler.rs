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
pub async fn upload_images_handler(
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
        handle
            .await
            .map_err(|e| {
                tracing::error!("{}", e.to_string());
                e
            })
            .map_err(|e| {
                tracing::error!("{}", e.to_string());
                e
            })??
    }
    Response::builder()
        .status(StatusCode::CREATED)
        .body(body::Body::empty())
        .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{routing::post, Router};
    use axum_test_helper::TestClient;
    use mockall::{mock, predicate};
    use reqwest::StatusCode;

    use crate::{
        services::images::ports::incoming::upload_images_service::{
            UploadImagesService, UploadImagesServiceError,
        },
        web::images::upload_images_handler,
    };

    mock! {
        pub Service {}
        #[async_trait]
        impl UploadImagesService for Service {
            async fn upload_image(&self, buffer: Vec<u8>) -> Result<(), UploadImagesServiceError>;
        }
    }

    pub fn app(service: MockService) -> TestClient {
        let upload_images_service =
            Arc::new(service) as upload_images_handler::DynUploadImagesService;
        let router = Router::new()
            .route("/", post(upload_images_handler::upload_images_handler))
            .with_state(upload_images_service);
        TestClient::new(router)
    }

    #[tokio::test]
    async fn on_ok_return_created_code() {
        let mut mock_service = MockService::new();
        let data = [0u8; 1024].to_vec();
        mock_service
            .expect_upload_image()
            .with(predicate::eq(data.clone()))
            .returning(move |_i| Ok(()));
        let app = app(mock_service);
        let form = reqwest::multipart::Form::new().part(
            "upload",
            reqwest::multipart::Part::bytes(data).file_name("file"),
        );
        let response = app.post("/").multipart(form).send().await;

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn on_internal_error_return_internal_server_error_code() {
        let mut mock_service = MockService::new();
        let data = [0u8; 1024].to_vec();
        mock_service
            .expect_upload_image()
            .with(predicate::eq(data.clone()))
            .returning(move |_i| Err(UploadImagesServiceError::InternalError));
        let app = app(mock_service);
        let form = reqwest::multipart::Form::new().part(
            "upload",
            reqwest::multipart::Part::bytes(data).file_name("file"),
        );
        let response = app.post("/").multipart(form).send().await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn on_decoding_error_return_internal_server_error_code() {
        let mut mock_service = MockService::new();
        let data = [0u8; 1024].to_vec();
        mock_service
            .expect_upload_image()
            .with(predicate::eq(data.clone()))
            .returning(move |_i| Err(UploadImagesServiceError::DecodingError));
        let app = app(mock_service);
        let form = reqwest::multipart::Form::new().part(
            "upload",
            reqwest::multipart::Part::bytes(data).file_name("file"),
        );
        let response = app.post("/").multipart(form).send().await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
