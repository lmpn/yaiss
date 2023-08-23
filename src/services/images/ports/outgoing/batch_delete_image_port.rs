use async_trait::async_trait;
#[async_trait]
pub trait BatchDeleteImagePort {
    async fn batch_delete_image(&self, index: Vec<i64>) -> anyhow::Result<Vec<String>>;
}
