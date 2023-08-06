use crate::images::domain::image::Image;
use async_trait::async_trait;
// #[automock(type Index = i64;)]
#[async_trait]
pub trait ImagesDataStorage {
    type Index;
    async fn query_images(&self, count: i64, offset: i64) -> anyhow::Result<Vec<Image>>;
    async fn query_image(&self, index: Self::Index) -> anyhow::Result<Image>;
    async fn delete_image(&self, index: Self::Index) -> anyhow::Result<()>;
    async fn batch_delete_image(&self, index: Vec<Self::Index>) -> anyhow::Result<Vec<String>>;
    async fn insert_image(&self, record: &Image) -> anyhow::Result<()>;
}
