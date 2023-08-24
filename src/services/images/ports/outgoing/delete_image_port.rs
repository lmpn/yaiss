use std::{error::Error, fmt::Display};

use async_trait::async_trait;
// #[automock(type Index = i64;)]
#[async_trait]
pub trait DeleteImagePort {
    async fn delete_image(&self, index: i64) -> Result<String, DeleteImageError>;
}

#[derive(Debug)]
pub enum DeleteImageError {
    RecordNotFound,
    InternalError,
}

impl Display for DeleteImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecordNotFound => write!(f, "Record not found"),
            Self::InternalError => write!(f, "Internal error"),
        }
    }
}

impl Error for DeleteImageError {}
