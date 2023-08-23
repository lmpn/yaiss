use async_trait::async_trait;
use tracing::error;

use super::ports::{
    incoming::delete_image_service::{DeleteImageService, DeleteImageServiceError},
    outgoing::delete_image_port::DeleteImagePort,
};

pub struct DeleteImage<Storage>
where
    Storage: DeleteImagePort + Send + Sync,
{
    storage: Storage,
}

#[async_trait]
impl<Storage> DeleteImageService for DeleteImage<Storage>
where
    Storage: DeleteImagePort + Send + Sync,
{
    async fn delete_image(&self, index: i64) -> Result<(), DeleteImageServiceError> {
        let path = match self.storage.delete_image(index).await {
            Ok(path) => path,
            Err(_) => return Err(DeleteImageServiceError::ImageNotFound),
        };
        if std::fs::remove_file(&path).is_err() {
            error!("Error removing file {}", path);
            return Err(DeleteImageServiceError::InternalError);
        }
        Ok(())
    }
}

impl<Storage> DeleteImage<Storage>
where
    Storage: DeleteImagePort + Send + Sync,
{
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use async_trait::async_trait;
    use mockall::mock;

    use crate::services::images::{
        delete_image::DeleteImage,
        ports::{
            incoming::delete_image_service::{DeleteImageService, DeleteImageServiceError},
            outgoing::delete_image_port::{DeleteImageError, DeleteImagePort},
        },
    };

    mock! {
        DS {}
        #[async_trait]
        impl DeleteImagePort for DS {
            async fn delete_image(&self, index: i64) -> Result<String, DeleteImageError>;
        }
    }

    #[tokio::test]
    async fn test_delete_image() {
        let path = env::current_dir().unwrap();
        std::fs::write(path.join("2"), "some content").unwrap();
        let mut mock = MockDS::new();
        mock.expect_delete_image()
            .returning(move |_i| anyhow::Result::Ok(path.join("2").to_str().unwrap().to_string()));
        let suu = DeleteImage::new(mock);
        let result = suu.delete_image(1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_image_ds_error() {
        let mut mock = MockDS::new();
        mock.expect_delete_image()
            .returning(move |_i| Err(DeleteImageError::RecordNotFound));
        let suu = DeleteImage::new(mock);
        let result = suu.delete_image(1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(DeleteImageServiceError::ImageNotFound));
    }

    #[tokio::test]
    async fn test_delete_image_fs_error() {
        let path = env::current_dir().unwrap();
        let mut mock = MockDS::new();
        mock.expect_delete_image()
            .returning(move |_i| anyhow::Result::Ok(path.join("1").to_str().unwrap().to_string()));
        let suu = DeleteImage::new(mock);
        let result = suu.delete_image(1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(DeleteImageServiceError::InternalError));
    }
}
