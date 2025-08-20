use axum::body::{Body, to_bytes};
use http::{Request, Response};
use sqlx::{Pool, Sqlite};
use tower::ServiceExt;

pub mod macros;

pub struct TestClient {
    pool: Pool<Sqlite>,
}

impl TestClient {
    pub async fn new() -> Self {
        let pool = sqlx::Pool::<sqlx::Sqlite>::connect("sqlite::memory:")
            .await
            .expect("unable to connect to test db");

        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .expect("unable to run migrations");

        Self { pool }
    }

    pub async fn send(&self, request: Request<Body>) -> Asserter {
        let response = server::server(self.pool.clone())
            .oneshot(request)
            .await
            .unwrap(/* Infallible */);
        Asserter::from(response)
    }
}

pub struct Asserter {
    response: Response<Body>,
}

impl Asserter {
    pub fn into_response(self) -> Response<Body> {
        self.response
    }

    pub fn status(self, expected: u16) -> Self {
        assert_eq!(
            self.response.status().as_u16(),
            expected,
            "expected status {}, got {}",
            expected,
            self.response.status()
        );
        self
    }

    pub fn is_success(self) -> Self {
        assert!(
            self.response.status().is_success(),
            "expected 2xx status, got {}",
            self.response.status()
        );
        self
    }

    pub fn is_client_error(self) -> Self {
        assert!(
            self.response.status().is_client_error(),
            "expected 4xx status, got {}",
            self.response.status()
        );
        self
    }

    pub fn is_server_error(self) -> Self {
        assert!(
            self.response.status().is_server_error(),
            "expected 5xx status, got {}",
            self.response.status()
        );
        self
    }

    pub async fn json_body<T>(self, f: impl FnOnce(T))
    where
        T: serde::de::DeserializeOwned,
    {
        f(self.into_deserialized_json_body::<T>().await)
    }

    pub async fn into_deserialized_json_body<T>(self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let body_bytes = to_bytes(self.response.into_body(), usize::MAX)
            .await
            .expect("unable to read response body");

        serde_json::from_slice::<T>(&body_bytes).expect("unable to deserialize response body")
    }
}

impl From<Response<Body>> for Asserter {
    fn from(response: Response<Body>) -> Self {
        Self { response }
    }
}

impl From<Asserter> for Response<Body> {
    fn from(asserter: Asserter) -> Self {
        asserter.response
    }
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
