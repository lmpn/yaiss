use std::sync::Arc;

use axum::{
    body::Body,
    routing::{delete, post},
    Router,
};

use crate::{
    data_storage::images::images_sqlite_ds::ImagesSqliteDS,
    services::images::{
        batch_delete_image::BatchDeleteImage, delete_image::DeleteImage,
        upload_images::UploadImages,
    },
    state::State,
};

use self::delete_image_handler::DynDeleteImagesService;

pub mod batch_delete_image_handler;
pub mod delete_image_handler;
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
    let images_routes = Router::new()
        .route("/", post(upload_images_handler::upload_image))
        .with_state(upload_images_service)
        .route(
            "/batch_delete",
            post(batch_delete_image_handler::batch_delete_image),
        )
        .with_state(batch_delete_image_service)
        .route(
            "/:identifier",
            delete(delete_image_handler::delete_image_handler),
        )
        .with_state(delete_image_service);
    let images_router = Router::new().nest("/images", images_routes);
    Router::new().nest("/api/v1", images_router)
}
