use async_trait::async_trait;

use crate::services::images::domain::image::Image;
// #[automock(type Index = i64;)]
#[async_trait]
pub trait InsertImagePort {
    async fn insert_image(&self, record: &Image) -> anyhow::Result<()>;
}
