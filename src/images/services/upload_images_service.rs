use crate::images::{data_storage::images_data_storage::ImagesDataStorage, domain::image::Image};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

pub struct UploadImagesService<Storage>
where
    Storage: ImagesDataStorage + Sync + Send,
{
    storage: Storage,
    base_path: String,
}

impl<Storage> UploadImagesService<Storage>
where
    Storage: ImagesDataStorage + Sync + Send,
{
    pub fn new(storage: Storage, base_path: String) -> Self {
        Self { storage, base_path }
    }

    fn generate_path(&self) -> PathBuf {
        let image_filename = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>();
        Path::new::<std::path::Path>(self.base_path.as_ref())
            .join(image_filename)
            .with_extension("qoi")
    }

    pub async fn upload_image(&self, buffer: Vec<u8>) -> anyhow::Result<()> {
        let image = image::io::Reader::new(Cursor::new(buffer))
            .with_guessed_format()?
            .decode()?;
        let mut bytes = vec![];
        image.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Qoi)?;
        let path = self.generate_path();
        tokio::fs::write(&path, bytes).await?;
        let image = Image::new(0, path.to_str().unwrap().to_string(), Utc::now());
        self.storage.insert_image(&image).await?;
        Ok(())
    }
}
