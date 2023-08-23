use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde::Serialize;
use serde_json::json;
use tracing::error;

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
pub async fn query_image_handler(
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
        Err(e) => {
            let message = e.to_string();
            error!("{}", message);
            let code = if e == QueryImageServiceError::ImageNotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            builder
                .status(code)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(body::Body::from(
                    Json(json!({
                        "error": message,
                    }))
                    .to_string(),
                ))
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
        web::images::query_image_handler::{self, ImageJson},
    };

    mock! {
        pub Service {}
        #[async_trait]
        impl QueryImageService for Service {
            async fn query_image(&self, index: i64) -> Result<Image, QueryImageServiceError>;
        }
    }

    pub fn app(service: MockService) -> TestClient {
        let query_image_service = Arc::new(service) as query_image_handler::DynQueryImageService;
        let router = Router::new()
            .route(
                "/:identifier",
                get(query_image_handler::query_image_handler),
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
            .returning(move |_i| Ok(Image::new(1, "some/path".to_string(), now)));
        let app = app(mock_service);
        let response = app.get("/1").send().await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        let image_json: ImageJson = Image::new(1, "some/path".to_string(), now).into();
        let json: Value = json!(image_json);
        assert_eq!(body, json);
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
