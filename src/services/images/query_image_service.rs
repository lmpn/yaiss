use super::{
    domain::image::Image,
    ports::{
        incoming::query_image_service::{QueryImageService, QueryImageServiceError},
        outgoing::query_image_port::{QueryError, QueryImagePort},
    },
};
use async_trait::async_trait;
impl From<QueryError> for QueryImageServiceError {
    fn from(value: QueryError) -> Self {
        match value {
            QueryError::RecordNotFound => QueryImageServiceError::ImageNotFound,
            QueryError::InternalError => QueryImageServiceError::InternalError,
        }
    }
}

pub struct QueryImage<Storage>
where
    Storage: QueryImagePort + Send + Sync,
{
    storage: Storage,
}

#[async_trait]
impl<Storage> QueryImageService for QueryImage<Storage>
where
    Storage: QueryImagePort + Send + Sync,
{
    async fn query_image(&self, index: i64) -> Result<Image, QueryImageServiceError> {
        self.storage
            .query_image(index)
            .await
            .map_err(|err| err.into())
    }
}

impl<Storage> QueryImage<Storage>
where
    Storage: QueryImagePort + Send + Sync,
{
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;

    use crate::services::images::{
        domain::image::Image,
        ports::{
            incoming::query_image_service::{QueryImageService, QueryImageServiceError},
            outgoing::query_image_port::{QueryError, QueryImagePort},
        },
        query_image_service::QueryImage,
    };

    mock! {
        DS {}
        #[async_trait]
        impl QueryImagePort for DS {
            async fn query_image(&self, index: i64) -> Result<Image, QueryError>;
        }
    }

    #[tokio::test]
    async fn test_query_image() {
        let mut mock = MockDS::new();
        let image = Image::new(1, "some/path".to_string(), Utc::now());
        let image_clone = image.clone();
        mock.expect_query_image()
            .returning(move |_i| Ok(image.clone()));
        let suu = QueryImage::new(mock);
        let result = suu.query_image(1).await;
        assert!(result.is_ok());
        assert_eq!(image_clone, result.unwrap())
    }

    #[tokio::test]
    async fn test_query_image_ds_error() {
        let mut mock = MockDS::new();
        mock.expect_query_image()
            .returning(move |_i| Err(QueryError::InternalError));
        let suu = QueryImage::new(mock);
        let result = suu.query_image(1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(QueryImageServiceError::InternalError));
    }

    #[tokio::test]
    async fn test_query_image_not_found() {
        let mut mock = MockDS::new();
        mock.expect_query_image()
            .returning(move |_i| Err(QueryError::RecordNotFound));
        let suu = QueryImage::new(mock);
        let result = suu.query_image(1).await;
        assert!(result.is_err());
        assert_eq!(result, Err(QueryImageServiceError::ImageNotFound));
    }
}
