use crate::images::domain::image::Image;
use async_trait::async_trait;
#[async_trait]
pub trait ImagesDataStorage {
    type Index;
    async fn query_images(&self, count: i64, offset: i64) -> anyhow::Result<Vec<Image>>;
    async fn query_image(&self, index: Self::Index) -> anyhow::Result<Image>;
    async fn delete_image(&self, index: Self::Index) -> anyhow::Result<()>;
    async fn insert_image(&self, record: &Image) -> anyhow::Result<()>;
}
