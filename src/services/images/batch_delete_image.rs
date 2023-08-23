use async_trait::async_trait;

use super::ports::{
    incoming::batch_delete_image_service::{BatchDeleteImageService, BatchDeleteImageServiceError},
    outgoing::batch_delete_image_port::BatchDeleteImagePort,
};

pub struct BatchDeleteImage<Storage>
where
    Storage: BatchDeleteImagePort + Send + Sync,
{
    storage: Storage,
}

#[async_trait]
impl<Storage> BatchDeleteImageService for BatchDeleteImage<Storage>
where
    Storage: BatchDeleteImagePort + Send + Sync,
{
    async fn batch_delete_image(
        &self,
        indexes: Vec<i64>,
    ) -> Result<(), BatchDeleteImageServiceError> {
        let len = indexes.len();
        let max = 50;
        if len > max {
            Err(BatchDeleteImageServiceError::TooManyImagesToDelete(
                max as u64,
            ))
        } else {
            let paths = match self.storage.batch_delete_image(indexes).await {
                Ok(paths) => paths,
                Err(_) => return Err(BatchDeleteImageServiceError::InternalError),
            };

            let mut err: Option<BatchDeleteImageServiceError> = None;
            for path in paths {
                let result = std::fs::remove_file(path);
                if result.is_err() {
                    err = Some(BatchDeleteImageServiceError::InternalError);
                }
            }
            if let Some(error) = err {
                return Err(error);
            }
            Ok(())
        }
    }
}

impl<Storage> BatchDeleteImage<Storage>
where
    Storage: BatchDeleteImagePort + Send + Sync,
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
        batch_delete_image::BatchDeleteImage,
        ports::{
            incoming::batch_delete_image_service::{
                BatchDeleteImageService, BatchDeleteImageServiceError,
            },
            outgoing::batch_delete_image_port::{BatchDeleteError, BatchDeleteImagePort},
        },
    };

    mock! {
        DS {}
        #[async_trait]
        impl BatchDeleteImagePort for DS {
            async fn batch_delete_image(&self, index: Vec<i64>) -> Result<Vec<String>, BatchDeleteError>;
        }
    }

    #[tokio::test]
    async fn test_batch_delete_image() {
        let path = env::current_dir().unwrap();
        std::fs::write(path.join("1"), "some content").unwrap();
        std::fs::write(path.join("2"), "some content").unwrap();
        let mut mock = MockDS::new();
        mock.expect_batch_delete_image().returning(move |_i| {
            anyhow::Result::Ok(vec![
                path.join("1").to_str().unwrap().to_string(),
                path.join("2").to_str().unwrap().to_string(),
            ])
        });
        let suu = BatchDeleteImage::new(mock);
        let result = suu.batch_delete_image(vec![1, 2]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_delete_image_fs_error() {
        let path = env::current_dir().unwrap();
        let mut mock = MockDS::new();
        mock.expect_batch_delete_image().returning(move |_i| {
            anyhow::Result::Ok(vec![
                path.join("1").to_str().unwrap().to_string(),
                path.join("2").to_str().unwrap().to_string(),
            ])
        });
        let suu = BatchDeleteImage::new(mock);
        let result = suu.batch_delete_image(vec![1, 2]).await;
        assert!(result.is_err());
        assert_eq!(result, Err(BatchDeleteImageServiceError::InternalError));
    }

    #[tokio::test]
    async fn test_batch_delete_image_ds_error() {
        let mut mock = MockDS::new();
        mock.expect_batch_delete_image()
            .returning(move |_i| Err(BatchDeleteError::InternalError));
        let suu = BatchDeleteImage::new(mock);
        let result = suu.batch_delete_image(vec![1, 2]).await;
        assert!(result.is_err());
        assert_eq!(result, Err(BatchDeleteImageServiceError::InternalError));
    }

    #[tokio::test]
    async fn test_batch_delete_image_more_than_fifty_error() {
        let mock = MockDS::new();
        let indexes = vec![0; 51];
        let suu = BatchDeleteImage::new(mock);
        let result = suu.batch_delete_image(indexes).await;
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(BatchDeleteImageServiceError::TooManyImagesToDelete(50))
        );
    }
}
