use anyhow::Ok;
use tracing::info;

use crate::images::data_storage::images_data_storage::{self, ImagesDataStorage};

pub struct DeleteImagesService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    storage: Storage,
}

impl<Storage> DeleteImagesService<Storage>
where
    Storage: ImagesDataStorage + Send + Sync,
{
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn delete_images(
        &self,
        indexes: Vec<<Storage as images_data_storage::ImagesDataStorage>::Index>,
    ) -> anyhow::Result<()> {
        let len = indexes.len();
        let max = 50;
        if len > max {
            anyhow::bail!("Cannot delete more than 50 images");
        } else {
            let paths = self.storage.delete_images(indexes).await?;
            info!("{:?}", paths);
            paths
                .into_iter()
                .for_each(|path| std::fs::remove_file(path).unwrap());
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
        services::delete_images_service::DeleteImagesService,
    };

    mock! {
        DS {}
        #[async_trait]
        impl ImagesDataStorage for DS {
            type Index=i64;
            async fn query_images(&self, count: i64, offset: i64) -> anyhow::Result<Vec<Image>>;
            async fn query_image(&self, index: <Self as ImagesDataStorage>::Index) -> anyhow::Result<Image>;
            async fn delete_image(&self, index: <Self as ImagesDataStorage>::Index) -> anyhow::Result<()>;
            async fn delete_images(&self, index: Vec<<Self as ImagesDataStorage>::Index>) -> anyhow::Result<Vec<String>>;
            async fn insert_image(&self, record: &Image) -> anyhow::Result<()>;
        }
    }

    #[tokio::test]
    async fn test_delete_images() {
        let path = env::current_dir().unwrap();
        std::fs::write(path.join("1"), "some content").unwrap();
        std::fs::write(path.join("2"), "some content").unwrap();
        let mut mock = MockDS::new();
        mock.expect_delete_images().returning(move |_i| {
            anyhow::Result::Ok(vec![
                path.join("1").to_str().unwrap().to_string(),
                path.join("2").to_str().unwrap().to_string(),
            ])
        });
        let suu = DeleteImagesService::new(mock);
        let result = suu.delete_images(vec![1, 2]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_images_ds_error() {
        let mut mock = MockDS::new();
        mock.expect_delete_images()
            .returning(move |_i| anyhow::bail!("error"));
        let suu = DeleteImagesService::new(mock);
        let result = suu.delete_images(vec![1, 2]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_images_more_than_fifty_error() {
        let mock = MockDS::new();
        let indexes = vec![0; 51];
        let suu = DeleteImagesService::new(mock);
        let result = suu.delete_images(indexes).await;
        assert!(result.is_err());
    }
}
