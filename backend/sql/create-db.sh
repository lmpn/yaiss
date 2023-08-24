#!/bin/bash
DB_URL=${DB_URL:-sqlite:backend/sql/images.db}

sqlx db create --database-url "$DB_URL"
sqlx migrate run --source backend/sql/migrations --database-url "$DB_URL"