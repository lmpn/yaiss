use std::sync::Arc;

use axum::{
    body::Body,
    routing::{delete, get, post},
    Router,
};

use crate::{
    data_storage::images::images_sqlite_ds::ImagesSqliteDS,
    services::images::{
        batch_delete_image::BatchDeleteImage, batch_query_image_service::BatchQueryImage,
        delete_image::DeleteImage, query_image_service::QueryImage, upload_images::UploadImages,
    },
    state::State,
};

use self::{
    batch_query_image_handler::DynBatchQueryImageService,
    delete_image_handler::DynDeleteImagesService, query_image_handler::DynQueryImageService,
};

pub mod batch_delete_image_handler;
pub mod batch_query_image_handler;
pub mod delete_image_handler;
pub mod get_image_content_handler;
pub mod query_image_handler;
pub mod upload_images_handler;

pub fn router(state: State) -> Router<(), Body> {
    let storage = ImagesSqliteDS::new(state.pool());
    let batch_delete_image_service = Arc::new(BatchDeleteImage::new(storage))
        as batch_delete_image_handler::DynBatchDeleteImageService;
    let storage = ImagesSqliteDS::new(state.pool());
    let upload_images_service = Arc::new(UploadImages::new(
        storage,
        state.images_base_path().to_string(),
    )) as upload_images_handler::DynUploadImagesService;
    let storage = ImagesSqliteDS::new(state.pool());
    let delete_image_service = Arc::new(DeleteImage::new(storage)) as DynDeleteImagesService;
    let storage = ImagesSqliteDS::new(state.pool());
    let query_image_service = Arc::new(QueryImage::new(storage)) as DynQueryImageService;
    let storage = ImagesSqliteDS::new(state.pool());
    let batch_query_image_service =
        Arc::new(BatchQueryImage::new(storage)) as DynBatchQueryImageService;
    let images_routes = Router::new()
        .route("/", post(upload_images_handler::upload_images_handler))
        .with_state(upload_images_service)
        .route(
            "/batch_delete",
            post(batch_delete_image_handler::batch_delete_image_handler),
        )
        .with_state(batch_delete_image_service)
        .route(
            "/:identifier",
            get(query_image_handler::query_image_handler),
        )
        .with_state(query_image_service.clone())
        .route(
            "/content/:identifier",
            get(get_image_content_handler::get_image_content_handler),
        )
        .with_state(query_image_service)
        .route(
            "/",
            get(batch_query_image_handler::batch_query_image_handler),
        )
        .with_state(batch_query_image_service)
        .route(
            "/:identifier",
            delete(delete_image_handler::delete_image_handler),
        )
        .with_state(delete_image_service);
    let images_router = Router::new().nest("/images", images_routes);
    Router::new().nest("/api/v1", images_router)
}
