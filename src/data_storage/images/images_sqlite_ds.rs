use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use tracing::error;

use crate::services::images::{
    domain::image::Image,
    ports::outgoing::{
        batch_delete_image_port::{BatchDeleteError, BatchDeleteImagePort},
        batch_query_image_port::{self, BatchQueryImagesPort},
        delete_image_port::{DeleteImageError, DeleteImagePort},
        insert_image_port::{InsertImageError, InsertImagePort},
        query_image_port::{self, QueryImagePort},
    },
};

impl From<sqlx::Error> for query_image_port::QueryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => query_image_port::QueryError::RecordNotFound,
            _ => query_image_port::QueryError::InternalError,
        }
    }
}

impl From<sqlx::Error> for batch_query_image_port::QueryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => batch_query_image_port::QueryError::RecordNotFound,
            _ => batch_query_image_port::QueryError::InternalError,
        }
    }
}

impl From<sqlx::Error> for BatchDeleteError {
    fn from(_value: sqlx::Error) -> Self {
        BatchDeleteError::InternalError
    }
}

impl From<sqlx::Error> for DeleteImageError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => DeleteImageError::RecordNotFound,
            _ => DeleteImageError::InternalError,
        }
    }
}

impl From<sqlx::Error> for InsertImageError {
    fn from(_value: sqlx::Error) -> Self {
        InsertImageError::InternalError
    }
}

pub struct ImagesSqliteDS {
    pool: SqlitePool,
}

#[async_trait]
impl QueryImagePort for ImagesSqliteDS {
    async fn query_image(&self, index: i64) -> Result<Image, query_image_port::QueryError> {
        let record = match sqlx::query!(
            r#"
                        SELECT id, path, updated_on FROM images 
                            WHERE id = ?1
                    "#,
            index
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(record) => record,
            Err(e) => {
                error!(
                    "Error querying image: {}; message: {}",
                    index,
                    e.to_string()
                );
                return Err(e.into());
            }
        };

        let created_on = record
            .updated_on
            .parse::<DateTime<Utc>>()
            .unwrap_or(Utc::now());
        let image = Image::new(record.id, record.path, created_on);
        Ok(image)
    }
}
#[async_trait]
impl BatchQueryImagesPort for ImagesSqliteDS {
    async fn query_images(
        &self,
        count: i64,
        offset: i64,
    ) -> Result<Vec<Image>, batch_query_image_port::QueryError> {
        if count < 0 || offset < 0 {}
        let recs = match sqlx::query!(
            r#"
                SELECT id, path, updated_on FROM images 
                    ORDER BY updated_on
                    LIMIT ?1
                    OFFSET ?2
            "#,
            count,
            offset
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(records) => records,
            Err(e) => {
                error!(
                    "Error querying {} images with offset {}; message: {}",
                    count,
                    offset,
                    e.to_string()
                );
                return Err(e.into());
            }
        };

        let images = recs
            .into_iter()
            .map(|record| {
                let updated_on = record
                    .updated_on
                    .map(|e| e.parse::<DateTime<Utc>>().unwrap_or(Utc::now()))
                    .unwrap_or(Utc::now());
                Image::new(record.id.unwrap(), record.path.unwrap(), updated_on)
            })
            .collect();
        Ok(images)
    }
}
#[async_trait]
impl DeleteImagePort for ImagesSqliteDS {
    async fn delete_image(&self, index: i64) -> Result<String, DeleteImageError> {
        let record = match sqlx::query!(r#"DELETE FROM images WHERE id = ?1 RETURNING path"#, index)
            .fetch_one(&self.pool)
            .await
        {
            Ok(record) => record,
            Err(e) => {
                error!("Error deleting image {}; message: {}", index, e.to_string());
                return Err(e.into());
            }
        };
        Ok(record.path)
    }
}
#[async_trait]
impl BatchDeleteImagePort for ImagesSqliteDS {
    async fn batch_delete_image(&self, indexes: Vec<i64>) -> Result<Vec<String>, BatchDeleteError> {
        let query = format!(
            "DELETE FROM images WHERE id in ({}) RETURNING path",
            itertools::join(&indexes, ",")
        );
        let records = match sqlx::query(&query).fetch_all(&__self.pool).await {
            Ok(records) => records,
            Err(e) => {
                error!(
                    "Error deleting images {:?}; message: {}",
                    indexes,
                    e.to_string()
                );
                return Err(e.into());
            }
        };
        let records = records
            .into_iter()
            .map(|record| record.get::<String, &str>("path"))
            .collect::<Vec<String>>();

        Ok(records)
    }
}
#[async_trait]
impl InsertImagePort for ImagesSqliteDS {
    async fn insert_image(&self, record: &Image) -> Result<(), InsertImageError> {
        let id = record.id();
        let path = record.path();
        let updated_on = record.updated_on().to_string();
        let result = if id == 0 {
            sqlx::query!(
                r#"
                INSERT INTO images (path, updated_on) VALUES (?1, ?2)
            "#,
                path,
                updated_on
            )
            .execute(&self.pool)
            .await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO images (id, path, updated_on) VALUES (?1, ?2, ?3)
            "#,
                id,
                path,
                updated_on
            )
            .execute(&self.pool)
            .await
        };

        match result {
            Ok(_) => (),
            Err(e) => {
                error!(
                    "Error inserting image {:?}; message: {}",
                    record,
                    e.to_string()
                );
                return Err(e.into());
            }
        }
        Ok(())
    }
}

impl ImagesSqliteDS {
    #[allow(dead_code)]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    async fn repository() -> ImagesSqliteDS {
        let pool = SqlitePool::connect(
            &std::env::var("DATABASE_URL").unwrap_or("sql/test.db".to_string()),
        )
        .await
        .unwrap();
        let storage = ImagesSqliteDS::new(pool);
        let image1 = image1();
        let image2 = image2();
        let image3 = image3();
        let _ = storage.insert_image(&image1).await;
        let _ = storage.insert_image(&image2).await;
        let _ = storage.insert_image(&image3).await;
        return storage;
    }

    fn image1() -> Image {
        Image::new(
            1,
            "path/to/image1".to_string(),
            "2023-07-12T19:29:11.113508Z"
                .parse::<DateTime<chrono::Utc>>()
                .unwrap(),
        )
    }
    fn image2() -> Image {
        Image::new(
            2,
            "path/to/image2".to_string(),
            "2023-07-12T19:29:11.113508Z"
                .parse::<DateTime<chrono::Utc>>()
                .unwrap(),
        )
    }
    fn image3() -> Image {
        Image::new(
            3,
            "path/to/image3".to_string(),
            "2023-07-12T19:29:11.113508Z"
                .parse::<DateTime<chrono::Utc>>()
                .unwrap(),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_query_images(
        repository: impl std::future::Future<Output = ImagesSqliteDS>,
        #[values(image1())] image1: Image,
        #[values(image2())] image2: Image,
        #[values(image3())] image3: Image,
    ) {
        // Query images
        let repository = repository.await;
        let images = repository.query_images(2, 0).await.unwrap();
        assert_eq!(images.len(), 2);
        assert!(images.contains(&image1));
        assert!(images.contains(&image2));

        let images = repository.query_images(2, 1).await.unwrap();
        assert_eq!(images.len(), 2);
        assert!(images.contains(&image2));
        assert!(images.contains(&image3));

        let images = repository.query_images(1, 2).await.unwrap();
        assert_eq!(images.len(), 1);
        assert!(images.contains(&image3));
    }

    #[rstest]
    #[case(image1())]
    #[case(image2())]
    #[case(image3())]
    #[tokio::test]
    async fn test_query_image(
        repository: impl std::future::Future<Output = ImagesSqliteDS>,
        #[case] expected_image: Image,
    ) {
        let repository = repository.await;
        let index = expected_image.id();
        // Query image
        let image = repository.query_image(index).await.unwrap();
        assert_eq!(expected_image, image);
    }

    #[rstest]
    #[tokio::test]
    async fn test_delete_image(repository: impl std::future::Future<Output = ImagesSqliteDS>) {
        let repository = repository.await;
        // Insert some test images
        let image = Image::new(5, "path/to/image5".to_string(), Utc::now());
        repository.insert_image(&image).await.unwrap();

        // Delete image
        let path = repository.delete_image(5).await.unwrap();
        assert_eq!(path, "path/to/image5".to_string());
    }

    #[rstest]
    #[tokio::test]
    async fn test_insert_image(repository: impl std::future::Future<Output = ImagesSqliteDS>) {
        let repository = repository.await;
        // Insert image
        let image = Image::new(
            4,
            "path/to/image4".to_string(),
            "2023-07-12T20:38:39.443964Z"
                .parse::<DateTime<chrono::Utc>>()
                .unwrap(),
        );
        repository.insert_image(&image).await.unwrap();

        let queried_image = repository.query_image(4).await.unwrap();
        assert_eq!(queried_image, image);
    }

    #[rstest]
    #[tokio::test]
    async fn test_batch_delete_image(
        repository: impl std::future::Future<Output = ImagesSqliteDS>,
    ) {
        let repository = repository.await;
        // Insert some test images
        let image = Image::new(7, "path/to/image7".to_string(), Utc::now());
        let image2 = Image::new(8, "path/to/image8".to_string(), Utc::now());
        repository.insert_image(&image).await.unwrap();
        repository.insert_image(&image2).await.unwrap();

        // Delete image
        let paths = repository.batch_delete_image(vec![7, 8]).await.unwrap();
        assert!(paths.contains(&"path/to/image7".to_string()));
        assert!(paths.contains(&"path/to/image8".to_string()));
        // Query images
        let image7_error = repository.query_image(7).await;
        let image8_error = repository.query_image(8).await;
        assert!(image7_error.is_err());
        assert!(image8_error.is_err());
    }
}
