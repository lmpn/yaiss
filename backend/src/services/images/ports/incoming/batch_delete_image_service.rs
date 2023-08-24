use std::{error::Error, fmt::Display};

use async_trait::async_trait;

#[async_trait]
pub trait BatchDeleteImageService {
    async fn batch_delete_image(
        &self,
        indexes: Vec<i64>,
    ) -> Result<(), BatchDeleteImageServiceError>;
}

#[derive(Debug, PartialEq)]
pub enum BatchDeleteImageServiceError {
    TooManyImagesToDelete(u64),
    InternalError,
}

impl Display for BatchDeleteImageServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchDeleteImageServiceError::TooManyImagesToDelete(max) => {
                f.write_str(format!("Too many images to delete. Max: {}", max).as_str())
            }
            BatchDeleteImageServiceError::InternalError => f.write_str("Internal error"),
        }
    }
}
impl Error for BatchDeleteImageServiceError {}
