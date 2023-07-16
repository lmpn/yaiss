use axum::{body::Body, routing::post, Router};

use crate::state::State;

pub mod upload_images_handler;

pub fn router() -> Router<State, Body> {
    let images_routes = Router::new().route("/", post(upload_images_handler::upload_image));
    let images_router = Router::new().nest("/images", images_routes);
    Router::new().nest("/api/v1", images_router)
}
