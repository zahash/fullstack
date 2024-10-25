#[macro_export]
macro_rules! request {
    ( $method:ident $url:expr ; $($header:expr => $value:expr)* ; $($body:expr)? ) => {{
        let mut req = axum::http::Request::builder()
            .method(stringify!($method))
            .uri($url);

        $(
            req = req.header($header, $value);
        )*

        req.body(axum::body::Body::from($( $body )?)).expect("unable to build request")
    }};
}

#[macro_export]
macro_rules! fixture {
    ( $pool:ident ; $( $req:expr ; )* ) => {{
        $(
            let resp = crate::send!( $pool $req );
            let status = resp.status();
            let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();

            assert!(status.is_success(), ":FIXTURE: status:{} :: {:?}", status, body);
        )*
    }};
}

#[macro_export]
macro_rules! send {
    ( $pool:ident $req:expr ) => {{
        use tower::ServiceExt;
        server::server(server::AppState {
            pool: $pool.clone(),
            rate_limiter: std::sync::Arc::new(server::RateLimiter::nolimit()),
            mailer: std::sync::Arc::new(lettre::FileTransport::new("/tmp")),
        })
        .oneshot($req)
        .await
        .expect("failed to send request")
    }};
}

#[macro_export]
macro_rules! status {
    ( 2xx ) => {{
        |resp: axum::http::Response<axum::body::Body>| {
            let status = resp.status();
            assert!(status.is_success(), "expected 2xx status, got {}", status);
            resp
        }
    }};

    ( $status:literal ) => {{
        |resp: axum::http::Response<axum::body::Body>| {
            assert_eq!(
                resp.status().as_u16(),
                $status,
                "expected status {}, got {}",
                $status,
                resp.status()
            );
            resp
        }
    }};
}

#[macro_export]
macro_rules! t {
    ( $e:expr ) => { $e };
    ( $e:expr => $f:expr ) => { $f($e) };
    ( $e:expr => $f:expr => $($g:tt)+ ) => { t! { $f($e) => $($g)+ } };
}

// https://docs.rs/crate/pipe_macro/latest/source/
// macro_rules! pipe {
//     ($e:expr) => {$e};

//     ($e:expr => $f:tt) => { $f($e) };
//     ($e:expr => $f:tt?) => { $f($e)? };
//     ($e:expr => $f:tt.await) => { $f($e).await };
//     ($e:expr => $f:tt.await?) => { $f($e).await? };

//     ($e:expr => $f:tt => $($g:tt)+) => { pipe! { $f($e) => $($g)+ } };
//     ($e:expr => $f:tt? => $($g:tt)+) => { pipe! { $f($e)? => $($g)+ } };
//     ($e:expr => $f:tt.await => $($g:tt)+) => { pipe! { $f($e).await => $($g)+ } };
//     ($e:expr => $f:tt.await? => $($g:tt)+) => { pipe! { $f($e).await? => $($g)+ } };

//     ($e:expr => $s:tt.$f:tt) => { $s.$f($e) };
//     ($e:expr => $s:tt.$f:tt?) => { $s.$f($e)? };
//     ($e:expr => $s:tt.$f:tt.await) => { $s.$f($e).await };
//     ($e:expr => $s:tt.$f:tt.await?) => { $s.$f($e).await? };

//     ($e:expr => $s:tt.$f:tt => $($g:tt)+) => { pipe! { $s.$f($e) => $($g)+ } };
//     ($e:expr => $s:tt.$f:tt? => $($g:tt)+) => { pipe! { $s.$f($e)? => $($g)+ } };
//     ($e:expr => $s:tt.$f:tt.await => $($g:tt)+) => { pipe! { $s.$f($e).await => $($g)+ } };
//     ($e:expr => $s:tt.$f:tt.await? => $($g:tt)+) => { pipe! { $s.$f($e).await? => $($g)+ } };
// }
