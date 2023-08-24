use async_trait::async_trait;

use crate::services::images::domain::image::Image;
use std::{error::Error, fmt::Display};
// #[automock(type Index = i64;)]
#[async_trait]
pub trait InsertImagePort {
    async fn insert_image(&self, record: &Image) -> Result<(), InsertImageError>;
}

#[derive(Debug)]
pub enum InsertImageError {
    InternalError,
}

impl Display for InsertImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalError => write!(f, "Internal error"),
        }
    }
}

impl Error for InsertImageError {}
