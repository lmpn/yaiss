use crate::services::images::{
    domain::image::Image,
    ports::{
        incoming::upload_images_service::UploadImagesService,
        outgoing::insert_image_port::InsertImagePort,
    },
};
use async_trait::async_trait;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

pub struct UploadImages<Storage>
where
    Storage: InsertImagePort + Sync + Send,
{
    storage: Storage,
    base_path: String,
}

#[async_trait]
impl<Storage> UploadImagesService for UploadImages<Storage>
where
    Storage: InsertImagePort + Sync + Send,
{
    async fn upload_image(&self, buffer: Vec<u8>) -> anyhow::Result<()> {
        let image = image::io::Reader::new(Cursor::new(buffer))
            .with_guessed_format()?
            .decode()?;
        let mut bytes = vec![];
        image.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Qoi)?;
        let path = self.generate_path();
        tokio::fs::write(&path, bytes).await?;
        let image = Image::new(
            0,
            path.to_str().expect("Invalid path for image").to_string(),
            Utc::now(),
        );
        self.storage.insert_image(&image).await?;
        Ok(())
    }
}

impl<Storage> UploadImages<Storage>
where
    Storage: InsertImagePort + Sync + Send,
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
}

#[cfg(test)]
mod tests {
    use std::{env, io::Cursor};

    use async_trait::async_trait;
    use mockall::mock;

    use crate::services::images::{
        domain::image::Image,
        ports::{
            incoming::upload_images_service::UploadImagesService,
            outgoing::insert_image_port::InsertImagePort,
        },
        upload_images::UploadImages,
    };

    mock! {
        DS {}
        #[async_trait]
        impl InsertImagePort for DS {
            async fn insert_image(&self, record: &Image) -> anyhow::Result<()>;
        }
    }

    #[tokio::test]
    async fn test_upload_image_with_empty_buffer() {
        let mut mock = MockDS::new();
        mock.expect_insert_image()
            .returning(|_i| anyhow::Result::Ok(()));
        let uis = UploadImages::new(mock, "data".to_string());
        let v = uis.upload_image(vec![]).await;
        assert!(v.is_err());
    }

    fn gen_img() -> (Vec<u8>, Vec<u8>) {
        let imgx = 5;
        let imgy = 2;

        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let r = (0.3 * x as f32) as u8;
            let b = (0.3 * y as f32) as u8;
            *pixel = image::Rgb([r, 0, b]);
        }

        let mut png_bytes = vec![];
        imgbuf
            .write_to(
                &mut Cursor::new(&mut png_bytes),
                image::ImageOutputFormat::Png,
            )
            .expect("failed to generate image");
        let mut qoi_bytes = vec![];
        imgbuf
            .write_to(
                &mut Cursor::new(&mut qoi_bytes),
                image::ImageOutputFormat::Qoi,
            )
            .expect("failed to generate image");
        println!("{}", qoi_bytes.len());
        return (png_bytes, qoi_bytes);
    }

    #[tokio::test]
    async fn test_upload_image_with_generated_buffer() {
        let mut mock = MockDS::new();
        mock.expect_insert_image()
            .returning(|_i| anyhow::Result::Ok(()));
        let path = env::current_dir().unwrap();
        let uis = UploadImages::new(mock, path.display().to_string());
        let (input, expected) = gen_img();
        let result = uis.upload_image(input.clone()).await;
        assert!(result.is_ok());
        let paths = std::fs::read_dir("./").unwrap();
        for path in paths {
            let p = path.unwrap().path();
            if "qoi" == p.extension().unwrap_or_default() {
                let buffer = std::fs::read(p.clone());
                assert_eq!(expected, buffer.unwrap());
                std::fs::remove_file(p).unwrap();
            }
        }
    }
}