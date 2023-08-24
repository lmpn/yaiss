use std::{error::Error, fmt::Display};

use async_trait::async_trait;
#[async_trait]
pub trait BatchDeleteImagePort {
    async fn batch_delete_image(&self, index: Vec<i64>) -> Result<Vec<String>, BatchDeleteError>;
}

#[derive(Debug)]
pub enum BatchDeleteError {
    TooManyImagesToDelete,
    InternalError,
}

impl Display for BatchDeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyImagesToDelete => write!(f, "Too many images to delete"),
            Self::InternalError => write!(f, "Internal error"),
        }
    }
}

impl Error for BatchDeleteError {}
