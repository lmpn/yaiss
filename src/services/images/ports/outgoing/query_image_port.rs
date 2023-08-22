use async_trait::async_trait;

use crate::services::images::domain::image::Image;
#[async_trait]
pub trait QueryImagePort {
    type Index;
    async fn query_image(&self, index: Self::Index) -> anyhow::Result<Image>;
}
