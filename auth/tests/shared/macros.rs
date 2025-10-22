#[macro_export]
macro_rules! request {
    ( $method:ident $url:expr ; $($header:expr => $value:expr)* ; $($body:expr)? ) => {{
        #[allow(unused_mut)]
        let mut req = axum::http::Request::builder()
            .method(stringify!($method))
            .uri($url);

        $(
            req = req.header($header, $value);
        )*

        req.body( axum::body::Body::from(( $( $body )? )) ).expect("unable to build request")
    }};
}
