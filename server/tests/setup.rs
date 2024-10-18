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

#[macro_export]
macro_rules! request {
    ( $method:ident $url:expr; $($header:expr => $value:expr)* ) => {{
        let mut req = axum::http::Request::builder()
            .uri($url)
            .method(stringify!($method));

        $(
            req = req.header($header, $value);
        )*

        req.body(()).expect("unable to build request")
    }};

    ( $method:ident $url:expr ; $($header:expr => $value:expr)* ; $body:expr ) => {{
        let mut req = axum::http::Request::builder()
            .uri($url)
            .method(stringify!($method));

        $(
            req = req.header($header, $value);
        )*

        req.body(axum::body::Body::from($body)).expect("unable to build request")
    }};
}

#[macro_export]
macro_rules! status {
    (  $pool:ident $status:literal $req:expr ) => {{
        let resp = server::server($pool.clone()).oneshot($req).await.unwrap();
        assert_eq!($status, resp.status().as_u16());
        resp
    }};
}
