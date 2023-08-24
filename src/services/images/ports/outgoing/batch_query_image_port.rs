use crate::services::images::domain::image::Image;
use async_trait::async_trait;
use std::{error::Error, fmt::Display};
#[async_trait]
pub trait BatchQueryImagesPort {
    async fn query_images(&self, count: i64, offset: i64) -> Result<Vec<Image>, QueryError>;
}

#[derive(Debug)]
pub enum QueryError {
    RecordNotFound,
    InternalError,
}

impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecordNotFound => write!(f, "Record not found"),
            Self::InternalError => write!(f, "Internal error"),
        }
    }
}

impl Error for QueryError {}

#[derive(Debug)]
pub enum DeleteError {
    RecordNotFound,
    InternalError,
}

impl Display for DeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecordNotFound => write!(f, "Record not found"),
            Self::InternalError => write!(f, "Internal error"),
        }
    }
}
