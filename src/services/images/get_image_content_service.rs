use std::{error::Error, fmt::Display};

use crate::images::data_storage::images_data_storage::{self, ImagesDataStorage, QueryError};

#[derive(Debug, PartialEq)]
pub enum GetImageContentServiceError {
    ImageNotFound,
    InternalError,
}

impl From<QueryError> for GetImageContentServiceError {
    fn from(value: QueryError) -> Self {
        match value {
            QueryError::RecordNotFound => GetImageContentServiceError::ImageNotFound,
            QueryError::InternalError => GetImageContentServiceError::InternalError,
        }
    }
}

impl Display for GetImageContentServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetImageContentServiceError::ImageNotFound => f.write_str("Image not found"),
            GetImageContentServiceError::InternalError => f.write_str("Internal error"),
        }
    }
}
impl Error for GetImageContentServiceError {}
pub struct GetImageContentService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    storage: Storage,
}

impl<Storage> GetImageContentService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn get_image_content(
        &self,
        index: <Storage as images_data_storage::ImagesDataStorage>::Index,
    ) -> Result<String, GetImageContentServiceError> {
        let image = self.storage.query_image(index).await?;
        Ok(image.path().to_string())
    }
}

#[cfg(test)]
mod tests {}
