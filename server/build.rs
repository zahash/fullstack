use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env").unwrap();
    println!("cargo:rerun-if-changed=migrations");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    if !Sqlite::database_exists(&database_url)
        .await
        .expect("failed to check if database exists")
    {
        Sqlite::create_database(&database_url)
            .await
            .expect("failed to create database");
    }

    sqlx::migrate!("./migrations")
        .run(
            &SqlitePool::connect(&database_url)
                .await
                .expect("failed to connect to database"),
        )
        .await
        .expect("failed to run migrations");
}
