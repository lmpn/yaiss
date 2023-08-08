use async_trait::async_trait;

use super::{
    domain::image::Image,
    ports::{
        incoming::batch_query_image_service::{
            BatchQueryImageService, BatchQueryImageServiceError,
        },
        outgoing::batch_query_image_port::{BatchQueryImagesPort, QueryError},
    },
};

const MAX_IMAGES: usize = 50;

impl From<QueryError> for BatchQueryImageServiceError {
    fn from(value: QueryError) -> Self {
        match value {
            QueryError::RecordNotFound => BatchQueryImageServiceError::NoRecordsFound,
            QueryError::InternalError => BatchQueryImageServiceError::InternalError,
        }
    }
}

pub struct BatchQueryImage<Storage>
where
    Storage: BatchQueryImagesPort + Send + Sync,
{
    storage: Storage,
}

#[async_trait]
impl<Storage> BatchQueryImageService for BatchQueryImage<Storage>
where
    Storage: BatchQueryImagesPort + Send + Sync,
{
    async fn batch_query_image(
        &self,
        count: i64,
        offset: i64,
    ) -> Result<Vec<Image>, BatchQueryImageServiceError> {
        if count <= 0 || offset < 0 {
            return Err(BatchQueryImageServiceError::InvalidRequest);
        }

        if count as usize > MAX_IMAGES {
            return Err(BatchQueryImageServiceError::TooManyImagesRequested);
        }
        self.storage
            .query_images(count, offset)
            .await
            .map_err(|err| err.into())
    }
}

impl<Storage> BatchQueryImage<Storage>
where
    Storage: BatchQueryImagesPort + Send + Sync,
{
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::mock;

    use crate::services::images::{
        batch_query_image_service::BatchQueryImage,
        domain::image::Image,
        ports::{
            incoming::batch_query_image_service::{
                BatchQueryImageService, BatchQueryImageServiceError,
            },
            outgoing::batch_query_image_port::{BatchQueryImagesPort, QueryError},
        },
    };

    mock! {
        DS {}
        #[async_trait]
        impl BatchQueryImagesPort for DS {
            async fn query_images(&self, count: i64, offset: i64) -> Result<Vec<Image>, QueryError>;
        }
    }

    #[tokio::test]
    async fn test_batch_get_image_count_is_zero() {
        let mock = MockDS::new();
        let suu = BatchQueryImage::new(mock);
        let result = suu.batch_query_image(0, 0).await;
        assert!(result.is_err());
        assert_eq!(result, Err(BatchQueryImageServiceError::InvalidRequest));
    }

    #[tokio::test]
    async fn test_batch_get_image_offset_below_zero() {
        let mock = MockDS::new();
        let suu = BatchQueryImage::new(mock);
        let result = suu.batch_query_image(0, -1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(BatchQueryImageServiceError::InvalidRequest));
    }

    #[tokio::test]
    async fn test_batch_get_image_ds_error() {
        let mut mock = MockDS::new();
        mock.expect_query_images()
            .returning(move |_c, _o| Err(QueryError::InternalError));
        let suu = BatchQueryImage::new(mock);
        let result = suu.batch_query_image(1, 1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(BatchQueryImageServiceError::InternalError));
    }

    #[tokio::test]
    async fn test_batch_get_image_too_many_images() {
        let mock = MockDS::new();
        let suu = BatchQueryImage::new(mock);
        let result = suu.batch_query_image(100, 1).await;
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(BatchQueryImageServiceError::TooManyImagesRequested)
        );
    }

    #[tokio::test]
    async fn test_batch_get_image_not_found() {
        let mut mock = MockDS::new();
        mock.expect_query_images()
            .returning(move |_c, _o| Err(QueryError::RecordNotFound));
        let suu = BatchQueryImage::new(mock);
        let result = suu.batch_query_image(1, 1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(BatchQueryImageServiceError::NoRecordsFound));
    }
}
