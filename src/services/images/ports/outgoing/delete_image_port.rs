use async_trait::async_trait;
// #[automock(type Index = i64;)]
#[async_trait]
pub trait DeleteImagePort {
    async fn delete_image(&self, index: i64) -> anyhow::Result<String>;
}
