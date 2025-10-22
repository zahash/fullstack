use auth::ServerOpts;
use axum::{
    Router,
    body::{Body, to_bytes},
};
use http::{Request, Response};
use sqlx::{Pool, Sqlite, sqlite::SqliteConnectOptions};
use tempfile::{TempDir, tempdir};
use tower::Service;

pub mod macros;

pub struct TestClient {
    router: Router,

    // hold TempDir because the temporary directory will be deleted on Drop
    _temp_dir: TempDir,
}

impl TestClient {
    pub async fn default() -> Self {
        let temp_dir = tempdir().expect("unable to create temp dir");

        let database_config = auth::DatabaseConfig {
            url: {
                let path = temp_dir.path().join("test.db");
                path.to_string_lossy().to_string()
            },
        };
        Self::prepare_database(&database_config).await;

        // let secrets = Secret

        let router = auth::router(ServerOpts {
            database: database_config,

            secrets_dir: {
                let dir = temp_dir.path().join("secrets");
                Self::prepare_secrets(&dir);
                dir
            },

            #[cfg(feature = "rate-limit")]
            rate_limiter: auth::RateLimiterConfig {
                limit: usize::MAX,
                interval: std::time::Duration::from_secs(0),
            },

            #[cfg(feature = "serve-dir")]
            serve_dir: temp_dir.path().to_owned(),

            #[cfg(feature = "smtp")]
            smtp: {
                let senders_dir = temp_dir.path().join("senders");
                Self::prepare_senders(&senders_dir);

                auth::SmtpConfig {
                    relay: "127.0.0.1".into(),
                    port: Some(1025),
                    username: None,
                    password: None,
                    senders_dir,
                    templates_dir: "../templates".into(),
                }
            },
        })
        .await
        .expect("unable to create router");

        Self {
            router,
            _temp_dir: temp_dir,
        }
    }

    pub async fn send(&mut self, request: Request<Body>) -> Asserter {
        let response = self.router
            .call(request)
            .await
            .unwrap(/* Infallible */);
        Asserter::from(response)
    }

    async fn prepare_database(config: &auth::DatabaseConfig) {
        let pool = Pool::<Sqlite>::connect_with(
            SqliteConnectOptions::new()
                .filename(&config.url)
                .create_if_missing(true),
        )
        .await
        .expect("unable to connect to test db");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("unable to run migrations");
    }

    fn prepare_secrets(dir: &std::path::Path) {
        std::fs::create_dir_all(dir).expect("unable to create secrets dir");
        std::fs::write(dir.join("hmac"), vec![0; 1]).expect("unable to create hmac secret");
    }

    #[cfg(feature = "smtp")]
    fn prepare_senders(dir: &std::path::Path) {
        std::fs::create_dir_all(dir).expect("unable to create senders dir");

        for sender in ["noreply"] {
            std::fs::write(dir.join(sender), format!("{}@example.com", sender))
                .expect(&format!("unable to create `{sender}` sender"));
        }
    }
}

pub struct Asserter {
    response: Response<Body>,
}

impl Asserter {
    pub fn into_response(self) -> Response<Body> {
        self.response
    }

    pub fn inspect(self) -> Self {
        println!("{:#?}", self.response);
        self
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
