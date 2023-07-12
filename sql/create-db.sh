sqlx db create --database-url "sqlite:sql/images.db"
sqlx migrate run --source sql/migrations --database-url "sqlite:sql/images.db"