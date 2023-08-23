use std::{error::Error, fmt::Display};

use async_trait::async_trait;

#[async_trait]
pub trait DeleteImageService {
    async fn delete_image(&self, index: i64) -> Result<(), DeleteImageServiceError>;
}

#[derive(Debug, PartialEq)]
pub enum DeleteImageServiceError {
    ImageNotFound,
    InternalError,
}

impl Display for DeleteImageServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteImageServiceError::ImageNotFound => f.write_str("Image not found"),
            DeleteImageServiceError::InternalError => f.write_str("Internal error"),
        }
    }
}
impl Error for DeleteImageServiceError {}
