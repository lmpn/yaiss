use async_trait::async_trait;

use crate::services::images::domain::image::Image;
#[async_trait]
pub trait QueryImagesPort {
    async fn query_images(&self, count: i64, offset: i64) -> anyhow::Result<Vec<Image>>;
}
