set -e

sqlx database create
sqlx migrate run --source auth/migrations
