use std::{error::Error, fmt::Display};

use async_trait::async_trait;

#[async_trait]
pub trait UploadImagesService {
    async fn upload_image(&self, buffer: Vec<u8>) -> Result<(), UploadImagesServiceError>;
}

#[derive(Debug, PartialEq)]
pub enum UploadImagesServiceError {
    InternalError,
    UnsupportedFormatError,
    DecodingError,
}

impl Display for UploadImagesServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadImagesServiceError::InternalError => f.write_str("Internal error"),
            UploadImagesServiceError::UnsupportedFormatError => {
                f.write_str("Unsupported format error")
            }
            UploadImagesServiceError::DecodingError => f.write_str("Decoding error"),
        }
    }
}
impl Error for UploadImagesServiceError {}
