use std::{error::Error, fmt::Display};

use crate::images::data_storage::images_data_storage::{self, ImagesDataStorage};
#[derive(Debug)]
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
pub struct BatchDeleteImageService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    storage: Storage,
}

impl<Storage> BatchDeleteImageService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn batch_delete_image(
        &self,
        indexes: Vec<<Storage as images_data_storage::ImagesDataStorage>::Index>,
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
            if err.is_some() {
                return Err(err.unwrap());
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use async_trait::async_trait;
    use mockall::mock;

    use crate::images::{
        data_storage::images_data_storage::ImagesDataStorage, domain::image::Image,
        services::batch_delete_image_service::BatchDeleteImageService,
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
        let suu = BatchDeleteImageService::new(mock);
        let result = suu.batch_delete_image(vec![1, 2]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_delete_image_ds_error() {
        let mut mock = MockDS::new();
        mock.expect_batch_delete_image()
            .returning(move |_i| anyhow::bail!("error"));
        let suu = BatchDeleteImageService::new(mock);
        let result = suu.batch_delete_image(vec![1, 2]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_delete_image_more_than_fifty_error() {
        let mock = MockDS::new();
        let indexes = vec![0; 51];
        let suu = BatchDeleteImageService::new(mock);
        let result = suu.batch_delete_image(indexes).await;
        assert!(result.is_err());
    }
}
