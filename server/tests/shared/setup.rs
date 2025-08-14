pub async fn pool() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = sqlx::Pool::<sqlx::Sqlite>::connect("sqlite::memory:")
        .await
        .expect("unable to connect to test db");

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("unable to run migrations");

    pool
}

#[cfg(feature = "tracing")]
static TRACING_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(feature = "tracing")]
pub fn tracing_init() {
    TRACING_INIT.call_once(|| {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer().with_test_writer())
            .init();
    });
}
