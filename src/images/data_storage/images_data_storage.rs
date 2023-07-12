use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(PartialEq, Debug, Clone)]
pub struct Image {
    id: i64,
    path: String,
    updated_on: DateTime<Utc>,
}

impl Image {
    pub fn new(id: i64, path: String, updated_on: DateTime<Utc>) -> Self {
        Self {
            id,
            path,
            updated_on,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    pub fn updated_on(&self) -> DateTime<Utc> {
        self.updated_on
    }
}

#[async_trait]
pub trait ImagesDataStorage {
    type Index;
    async fn query_images(&self, count: i64, offset: i64) -> anyhow::Result<Vec<Image>>;
    async fn query_image(&self, index: Self::Index) -> anyhow::Result<Image>;
    async fn delete_image(&self, index: Self::Index) -> anyhow::Result<()>;
    async fn insert_image(&self, record: &Image) -> anyhow::Result<()>;
}
