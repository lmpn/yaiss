use std::sync::Arc;

use axum::{
    body::{self, Body},
    http::{Response, StatusCode},
    Json,
};
use serde_json::json;

use crate::{
    error::YaissError,
    services::images::ports::incoming::batch_delete_image_service::BatchDeleteImageService,
};
pub(crate) type DynBatchDeleteImageService = Arc<dyn BatchDeleteImageService + Send + Sync>;
pub async fn batch_delete_image_handler(
    axum::extract::State(service): axum::extract::State<DynBatchDeleteImageService>,
    identifiers: axum::extract::Json<Vec<i64>>,
) -> Result<Response<Body>, YaissError> {
    let service = service.clone();
    let builder = Response::builder();
    let builder = match service.batch_delete_image(identifiers.0).await {
        Ok(()) => builder.status(StatusCode::OK).body(Body::empty()),
        Err(e) => {
            let body = Json(json!({
                "error": e.to_string(),
            }))
            .to_string();
            builder
                .status(StatusCode::INTERNAL_SERVER_ERROR)
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
    use axum::{body::Body, http::Request, routing::delete, Json, Router};
    use mockall::{mock, predicate};
    use reqwest::StatusCode;
    use serde_json::{json, Value};

    use crate::{
        services::images::ports::incoming::batch_delete_image_service::{
            BatchDeleteImageService, BatchDeleteImageServiceError,
        },
        web::images::batch_delete_image_handler,
    };
    use tower::ServiceExt; // for `oneshot` and `ready`

    mock! {
        pub Service {}
        #[async_trait]
        impl BatchDeleteImageService for Service {
            async fn batch_delete_image(&self, indexes: Vec<i64>) -> Result<(), BatchDeleteImageServiceError>;
        }
    }

    pub fn app(service: MockService) -> Router<(), Body> {
        let batch_delete_image_service =
            Arc::new(service) as batch_delete_image_handler::DynBatchDeleteImageService;
        Router::new()
            .route(
                "/",
                delete(batch_delete_image_handler::batch_delete_image_handler),
            )
            .with_state(batch_delete_image_service)
    }

    #[tokio::test]
    async fn on_images_deleted_return_ok() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_delete_image()
            .with(predicate::eq(vec![1i64, 2, 3, 4]))
            .returning(move |_i| Ok(()));
        let app = app(mock_service);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(Json(json!(vec![1i64, 2, 3, 4])).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn on_internal_error_return_internal_server_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_delete_image()
            .with(predicate::eq(vec![1i64, 2, 3, 4]))
            .returning(move |_i| Err(BatchDeleteImageServiceError::InternalError));
        let app = app(mock_service);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(Json(json!(vec![1i64, 2, 3, 4])).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"error": "Internal error"}));
    }

    #[tokio::test]
    async fn on_too_many_images_return_internal_server_code() {
        let mut mock_service = MockService::new();
        mock_service
            .expect_batch_delete_image()
            .with(predicate::eq(vec![0i64; 60]))
            .returning(move |_i| Err(BatchDeleteImageServiceError::TooManyImagesToDelete(50)));
        let app = app(mock_service);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(Json(json!(vec![0i64; 60])).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            json!({ "error": "Too many images to delete. Max: 50", })
        );
    }
}
