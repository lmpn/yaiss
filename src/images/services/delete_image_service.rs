use std::{error::Error, fmt::Display};

use crate::images::data_storage::images_data_storage::{self, ImagesDataStorage};

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

#[cfg(test)]
mod tests {
    use std::env;

    use async_trait::async_trait;
    use mockall::mock;

    use crate::images::{
        data_storage::images_data_storage::ImagesDataStorage,
        domain::image::Image,
        services::delete_image_service::{DeleteImageService, DeleteImageServiceError},
    };

    mock! {
        DS {}
        #[async_trait]
        impl ImagesDataStorage for DS {
            type Index=i64;
            async fn query_images(&self, count: i64, offset: i64) -> anyhow::Result<Vec<Image>>;
            async fn query_image(&self, index: <Self as ImagesDataStorage>::Index) -> anyhow::Result<Image>;
            async fn delete_image(&self, index: <Self as ImagesDataStorage>::Index) -> anyhow::Result<String>;
            async fn batch_delete_image(&self, index: Vec<<Self as ImagesDataStorage>::Index>) -> anyhow::Result<Vec<String>>;
            async fn insert_image(&self, record: &Image) -> anyhow::Result<()>;
        }
    }

    #[tokio::test]
    async fn test_delete_image() {
        let path = env::current_dir().unwrap();
        std::fs::write(path.join("1"), "some content").unwrap();
        let mut mock = MockDS::new();
        mock.expect_delete_image()
            .returning(move |_i| anyhow::Result::Ok(path.join("1").to_str().unwrap().to_string()));
        let suu = DeleteImageService::new(mock);
        let result = suu.delete_image(1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_image_ds_error() {
        let mut mock = MockDS::new();
        mock.expect_delete_image()
            .returning(move |_i| anyhow::bail!("error"));
        let suu = DeleteImageService::new(mock);
        let result = suu.delete_image(1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(DeleteImageServiceError::ImageNotFound));
    }
}
