use std::{error::Error, fmt::Display};

use async_trait::async_trait;

use crate::services::images::domain::image::Image;

#[async_trait]
pub trait QueryImageService {
    async fn query_image(&self, id: i64) -> Result<Image, QueryImageServiceError>;
}

#[derive(Debug, PartialEq)]
pub enum QueryImageServiceError {
    ImageNotFound,
    InternalError,
}

impl Display for QueryImageServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryImageServiceError::ImageNotFound => f.write_str("Image not found"),
            QueryImageServiceError::InternalError => f.write_str("Internal error"),
        }
    }
}
impl Error for QueryImageServiceError {}
