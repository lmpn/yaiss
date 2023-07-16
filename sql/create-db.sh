#!/bin/bash
DATABASE_URL=${DATABASE_URL:-sqlite:sql/images.db}

sqlx db create --database-url "$DATABASE_URL"
sqlx migrate run --source sql/migrations --database-url "$DATABASE_URL"