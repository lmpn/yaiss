#!/bin/bash
DB_URL=${DB_URL:-backend/sqlite:sql/images.db}
sqlx database drop -y --database-url "${DB_URL}"