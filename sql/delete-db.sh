#!/bin/bash
DATABASE_URL=${DATABASE_URL:-sqlite:sql/images.db}
sqlx database drop -y --database-url "sqlite:sql/images.db"