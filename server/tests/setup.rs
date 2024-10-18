use sqlx::SqlitePool;

pub async fn pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("unable to connect to test db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("unable to run migrations");

    pool
}
