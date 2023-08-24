use std::{error::Error, fmt::Display};

use async_trait::async_trait;

use crate::services::images::domain::image::Image;

#[async_trait]
pub trait BatchQueryImageService {
    async fn batch_query_image(
        &self,
        count: i64,
        offset: i64,
    ) -> Result<Vec<Image>, BatchQueryImageServiceError>;
}

#[derive(Debug, PartialEq)]
pub enum BatchQueryImageServiceError {
    TooManyImagesRequested,
    InternalError,
    InvalidRequest,
    NoRecordsFound,
}

impl Display for BatchQueryImageServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchQueryImageServiceError::InternalError => f.write_str("Internal error"),
            BatchQueryImageServiceError::TooManyImagesRequested => {
                f.write_str("Too many images requested")
            }
            BatchQueryImageServiceError::InvalidRequest => {
                f.write_str("Count or offset are below zero")
            }
            BatchQueryImageServiceError::NoRecordsFound => f.write_str("No records found"),
        }
    }
}
impl Error for BatchQueryImageServiceError {}
