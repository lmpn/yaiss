use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::images_data_storage::{Image, ImagesDataStorage};

struct ImageSqliteDS {
    pool: SqlitePool,
}

#[async_trait]
impl ImagesDataStorage for ImageSqliteDS {
    type Index = i64;

    async fn query_images(&self, count: i64, offset: i64) -> anyhow::Result<Vec<Image>> {
        if count < 0 || offset < 0 {}
        let recs = sqlx::query!(
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
        .await?;

        let images = recs
            .into_iter()
            .map(|record| {
                let updated_on = record
                    .updated_on
                    .parse::<DateTime<Utc>>()
                    .unwrap_or(Utc::now());
                Image::new(record.id, record.path, updated_on)
            })
            .collect();
        Ok(images)
    }

    async fn query_image(&self, index: Self::Index) -> anyhow::Result<Image> {
        let record = sqlx::query!(
            r#"
                SELECT id, path, updated_on FROM images 
                    WHERE id = ?1
            "#,
            index
        )
        .fetch_one(&self.pool)
        .await?;

        let created_on = record
            .updated_on
            .parse::<DateTime<Utc>>()
            .unwrap_or(Utc::now());
        let image = Image::new(record.id, record.path, created_on);
        Ok(image)
    }

    async fn delete_image(&self, index: Self::Index) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM images WHERE id = ?1
            "#,
            index
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn insert_image(&self, record: &Image) -> anyhow::Result<()> {
        let id = record.id();
        let path = record.path();
        let updated_on = record.updated_on().to_string();
        if id == 0 {
            sqlx::query!(
                r#"
                INSERT INTO images (path, updated_on) VALUES (?1, ?2)
            "#,
                path,
                updated_on
            )
            .execute(&self.pool)
            .await?;
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
            .await?;
        }

        Ok(())
    }
}

impl ImageSqliteDS {
    #[allow(dead_code)]
    fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    async fn repository() -> ImageSqliteDS {
        let pool = SqlitePool::connect(&std::env::var("TEST_DB_URL").unwrap())
            .await
            .unwrap();
        let storage = ImageSqliteDS::new(pool);
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
        repository: impl std::future::Future<Output = ImageSqliteDS>,
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
        repository: impl std::future::Future<Output = ImageSqliteDS>,
        #[case] expected_image: Image,
    ) {
        let repository = repository.await;
        let index: <ImageSqliteDS as ImagesDataStorage>::Index = expected_image.id();
        // Query image
        let image = repository.query_image(index).await.unwrap();
        assert_eq!(expected_image, image);
    }

    #[rstest]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: no rows returned by a query that expected to return at least one row"
    )]
    #[tokio::test]
    async fn test_delete_image(repository: impl std::future::Future<Output = ImageSqliteDS>) {
        let repository = repository.await;
        // Insert some test images
        let image = Image::new(5, "path/to/image5".to_string(), Utc::now());
        repository.insert_image(&image).await.unwrap();

        // Delete image
        repository.delete_image(5).await.unwrap();

        // Query images
        repository.query_image(5).await.unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_insert_image(repository: impl std::future::Future<Output = ImageSqliteDS>) {
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
}
