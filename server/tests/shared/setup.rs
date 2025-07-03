use cache::CacheRegistry;
use data_access::DataAccess;
use sqlx::SqlitePool;

pub async fn data_access() -> DataAccess {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("unable to connect to test db");

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("unable to run migrations");

    DataAccess::new(pool, CacheRegistry::new())
}
