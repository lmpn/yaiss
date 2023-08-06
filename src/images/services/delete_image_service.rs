use std::{error::Error, fmt::Display};

use crate::images::data_storage::images_data_storage::{self, ImagesDataStorage};

#[derive(Debug)]
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
pub struct DeleteImageService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    storage: Storage,
}

impl<Storage> DeleteImageService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn delete_image(
        &self,
        index: <Storage as images_data_storage::ImagesDataStorage>::Index,
    ) -> Result<(), DeleteImageServiceError> {
        let path = match self.storage.delete_image(index).await {
            Ok(path) => path,
            Err(_) => return Err(DeleteImageServiceError::ImageNotFound.into()),
        };
        if let Err(_) = std::fs::remove_file(path) {
            return Err(DeleteImageServiceError::InternalError.into());
        }
        Ok(())
    }
}
