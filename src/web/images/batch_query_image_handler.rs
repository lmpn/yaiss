use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::YaissError,
    services::images::{
        domain::image::Image,
        ports::incoming::batch_query_image_service::{
            BatchQueryImageService, BatchQueryImageServiceError,
        },
    },
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pagination {
    pub count: i64,
    pub offset: i64,
}

impl Pagination {
    pub fn new(count: i64, offset: i64) -> Self {
        Self { count, offset }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            count: 50,
            offset: 0,
        }
    }
}

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

pub(crate) type DynBatchQueryImageService = Arc<dyn BatchQueryImageService + Send + Sync>;
pub async fn batch_query_image_handler(
    axum::extract::State(service): axum::extract::State<DynBatchQueryImageService>,
    pagination: Option<axum::extract::Query<Pagination>>,
) -> Result<Response<Body>, YaissError> {
    let service = service.clone();
    let builder = Response::builder();
    let pagination = pagination.unwrap_or_default();
    let builder = match service
        .batch_query_image(pagination.count, pagination.offset)
        .await
    {
        Ok(images) => {
            let images = images
                .into_iter()
                .map(ImageJson::from)
                .collect::<Vec<ImageJson>>();
            let body = Json(json!({ "images": images })).to_string();
            builder.status(StatusCode::OK).body(body::Body::from(body))
        }
        Err(e) => {
            let status = match e {
                BatchQueryImageServiceError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
                BatchQueryImageServiceError::TooManyImagesRequested => StatusCode::BAD_REQUEST,
                BatchQueryImageServiceError::InvalidRequest => StatusCode::BAD_REQUEST,
                BatchQueryImageServiceError::NoRecordsFound => StatusCode::NOT_FOUND,
            };
            builder.status(status).body(body::Body::from(
                Json(json!({
                    "error": e.to_string(),
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
    use axum::{routing::get, Router};
    use axum_test_helper::TestClient;
    use chrono::Utc;
    use mockall::{mock, predicate};
    use reqwest::StatusCode;
    use serde_json::{json, Value};

    use crate::{
        services::images::{
            domain::image::Image,
            ports::incoming::batch_query_image_service::{
                BatchQueryImageService, BatchQueryImageServiceError,
            },
        },
        web::images::batch_query_image_handler::{self, ImageJson},
    };

    mock! {
        pub Service {}
        #[async_trait]
        impl BatchQueryImageService for Service {
            async fn batch_query_image(&self, count: i64, offset: i64) -> Result<Vec<Image>, BatchQueryImageServiceError>;
        }
    }

    pub fn app(service: MockService) -> TestClient {
        let batch_query_image_service =
            Arc::new(service) as batch_query_image_handler::DynBatchQueryImageService;
        let router = Router::new()
            .route(
                "/",
                get(batch_query_image_handler::batch_query_image_handler),
            )
            .with_state(batch_query_image_service);
        TestClient::new(router)
    }

    #[tokio::test]
    async fn on_image_existing_return_ok() {
        let now = Utc::now();
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_query_image()
            .with(predicate::eq(50), predicate::eq(0))
            .returning(move |_i, _j| Ok(vec![Image::new(1, "some/path".to_string(), now)]));
        let app = app(mock_service);
        let response = app
            .get("/?count=50&offset=0")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        let image_json: ImageJson = Image::new(1, "some/path".to_string(), now).into();
        let json: Value = json!({ "images": vec![image_json] });
        assert_eq!(body, json);
    }

    #[tokio::test]
    async fn on_internal_error_return_internal_server_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_query_image()
            .with(predicate::eq(50), predicate::eq(0))
            .returning(move |_i, _j| Err(BatchQueryImageServiceError::InternalError));
        let app = app(mock_service);
        let response = app
            .get("/?count=50&offset=0")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Internal error"}));
    }

    #[tokio::test]
    async fn on_image_not_found_return_not_found_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_query_image()
            .with(predicate::eq(50), predicate::eq(0))
            .returning(move |_i, _j| Err(BatchQueryImageServiceError::NoRecordsFound));
        let app = app(mock_service);
        let response = app
            .get("/?count=50&offset=0")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "No records found", }));
    }

    #[tokio::test]
    async fn on_too_many_images_request_return_bad_request_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_query_image()
            .with(predicate::eq(51), predicate::eq(0))
            .returning(move |_i, _j| Err(BatchQueryImageServiceError::TooManyImagesRequested));
        let app = app(mock_service);
        let response = app
            .get("/?count=51&offset=0")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Too many images requested", }));
    }

    #[tokio::test]
    async fn on_invalid_pagination_return_bad_request_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_query_image()
            .with(predicate::eq(0), predicate::eq(0))
            .returning(move |_i, _j| Err(BatchQueryImageServiceError::InvalidRequest));
        let app = app(mock_service);
        let response = app
            .get("/?count=0&offset=0")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Count or offset are below zero", }));
    }
}
