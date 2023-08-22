use async_trait::async_trait;

#[async_trait]
pub trait UploadImagesService {
    async fn upload_image(&self, buffer: Vec<u8>) -> anyhow::Result<()>;
}
