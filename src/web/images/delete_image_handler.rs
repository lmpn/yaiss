use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde_json::json;
use tracing::error;

use crate::{
    error::YaissError, services::images::ports::incoming::delete_image_service::DeleteImageService,
    services::images::ports::incoming::delete_image_service::DeleteImageServiceError,
};

pub(crate) type DynDeleteImagesService = Arc<dyn DeleteImageService + Send + Sync>;

pub async fn delete_image_handler(
    axum::extract::State(service): axum::extract::State<DynDeleteImagesService>,
    identifier: axum::extract::Path<i64>,
) -> Result<Response<Body>, YaissError> {
    let service = service.clone();
    let builder = Response::builder();
    let builder = match service.delete_image(identifier.0).await {
        Ok(()) => builder.status(StatusCode::OK).body(Body::empty()),
        Err(e) => {
            let message = e.to_string();
            error!("{}", message);
            let body = Json(json!({
                "error": message,
            }))
            .to_string();
            let code = if e == DeleteImageServiceError::ImageNotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            builder
                .status(code)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(body::Body::from(body))
        }
    };
    builder.map_err(|e| e.into())
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{body::Body, routing::delete, Router};
    use axum_test_helper::TestClient;
    use mockall::{mock, predicate};
    use reqwest::StatusCode;
    use serde_json::{json, Value};

    use crate::{
        services::images::ports::incoming::delete_image_service::{
            DeleteImageService, DeleteImageServiceError,
        },
        web::images::delete_image_handler,
    };

    mock! {
        pub Service {}
        #[async_trait]
        impl DeleteImageService for Service {
            async fn delete_image(&self, index: i64) -> Result<(), DeleteImageServiceError>;
        }
    }

    pub fn app(service: MockService) -> TestClient {
        let delete_image_service =
            Arc::new(service) as delete_image_handler::DynDeleteImagesService;
        let router = Router::new()
            .route(
                "/:identifier",
                delete(delete_image_handler::delete_image_handler),
            )
            .with_state(delete_image_service);
        TestClient::new(router)
    }

    #[tokio::test]
    async fn on_image_deleted_return_ok() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_delete_image()
            .with(predicate::eq(1))
            .returning(move |_i| Ok(()));
        let app = app(mock_service);
        let response = app.delete("/1").send().await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.bytes().await;
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn on_internal_error_return_internal_server_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_delete_image()
            .returning(move |_i| Err(DeleteImageServiceError::InternalError));
        let app = app(mock_service);
        let response = app.delete("/1").body(Body::empty()).send().await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Internal error"}));
    }

    #[tokio::test]
    async fn on_image_not_found_return_not_found_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_delete_image()
            .returning(move |_i| Err(DeleteImageServiceError::ImageNotFound));
        let app = app(mock_service);
        let response = app
            .delete("/1")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.bytes().await;
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Image not found", }));
    }
}
