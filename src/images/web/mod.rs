use axum::{
    body::Body,
    routing::{delete, post},
    Router,
};

use crate::state::State;

pub mod batch_delete_image_handler;
pub mod delete_image_handler;
pub mod upload_images_handler;

pub fn router() -> Router<State, Body> {
    let images_routes = Router::new()
        .route("/", post(upload_images_handler::upload_image))
        .route(
            "/batch_delete",
            post(batch_delete_image_handler::batch_delete_image),
        )
        .route(
            "/:identifier",
            delete(delete_image_handler::delete_image_handler),
        );
    let images_router = Router::new().nest("/images", images_routes);
    Router::new().nest("/api/v1", images_router)
}
