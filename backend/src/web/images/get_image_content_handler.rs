use std::sync::Arc;

use axum::{
    body::{self, BoxBody, StreamBody},
    http::{Response, StatusCode},
    Json,
};
use serde::Serialize;
use serde_json::json;
use tokio_util::io::ReaderStream;

use crate::{
    error::YaissError,
    services::images::ports::incoming::query_image_service::{
        QueryImageService, QueryImageServiceError,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct ImageJson {
    id: i64,
    updated_on: String,
}

pub(crate) type DynQueryImageService = Arc<dyn QueryImageService + Sync + Send>;
pub async fn get_image_content_handler(
    axum::extract::State(service): axum::extract::State<DynQueryImageService>,
    identifier: axum::extract::Path<i64>,
) -> Result<Response<BoxBody>, YaissError> {
    let service = service.clone();
    let builder = Response::builder();
    let builder = match service.query_image(identifier.0).await {
        Err(e) => {
            let message = e.to_string();
            tracing::error!("{}", message);
            let code = if e == QueryImageServiceError::ImageNotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            builder
                .status(code)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(body::boxed(
                    Json(json!({
                        "error": message,
                    }))
                    .to_string(),
                ))
        }
        Ok(image) => {
            let file = tokio::fs::File::open(image.path()).await?;
            let stream = ReaderStream::new(file);
            let body = StreamBody::new(stream);
            builder
                .status(StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "image/qoi")
                .body(body::boxed(body))
        }
    };
    builder.map_err(|e| e.into())
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{body::Body, routing::get, Router};
    use axum_test_helper::TestClient;
    use chrono::Utc;
    use mockall::{mock, predicate};
    use reqwest::StatusCode;
    use serde_json::{json, Value};

    use crate::{
        services::images::{
            domain::image::Image,
            ports::incoming::query_image_service::{QueryImageService, QueryImageServiceError},
        },
        web::images::get_image_content_handler::{self},
    };

    mock! {
        pub Service {}
        #[async_trait]
        impl QueryImageService for Service {
            async fn query_image(&self, index: i64) -> Result<Image, QueryImageServiceError>;
        }
    }

    pub fn app(service: MockService) -> TestClient {
        let query_image_service =
            Arc::new(service) as get_image_content_handler::DynQueryImageService;
        let router = Router::new()
            .route(
                "/:identifier",
                get(get_image_content_handler::get_image_content_handler),
            )
            .with_state(query_image_service);
        TestClient::new(router)
    }

    #[tokio::test]
    async fn on_image_existing_return_ok() {
        let now = Utc::now();
        let mut mock_service = MockService::new();
        mock_service
            .expect_query_image()
            .with(predicate::eq(1))
            .returning(move |_i| Ok(Image::new(1, "Cargo.toml".to_string(), now)));
        let app = app(mock_service);
        let response = app.get("/1").send().await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.bytes().await;
        let e = tokio::fs::read("Cargo.toml".to_string()).await.unwrap();
        assert_eq!(body.to_vec(), e);
    }

    #[tokio::test]
    async fn on_internal_error_return_internal_server_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_query_image()
            .returning(move |_i| Err(QueryImageServiceError::InternalError));
        let app = app(mock_service);
        let response = app.get("/1").body(Body::empty()).send().await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Internal error"}));
    }

    #[tokio::test]
    async fn on_image_not_found_return_not_found_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_query_image()
            .returning(move |_i| Err(QueryImageServiceError::ImageNotFound));
        let app = app(mock_service);
        let response = app
            .get("/1")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Image not found", }));
    }
}
