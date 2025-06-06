mod access_token;
mod email;
mod health;
mod login;
mod logout;
mod private;
mod signup;
mod username;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use server_core::{AppState, mw_client_ip, mw_handle_leaked_5xx, mw_rate_limiter, span};

use access_token::{check_access_token, generate_access_token};
use email::check_email_availability;
use health::{health, sysinfo};
use login::login;
use logout::logout;
use private::private;
use signup::signup;
use username::check_username_availability;

pub fn server(state: AppState) -> Router {
    Router::new()
        .nest(
            "/check",
            Router::new()
                .route("/username-availability", get(check_username_availability))
                .route("/email-availability", get(check_email_availability))
                .route("/access-token", get(check_access_token)),
        )
        .route("/health", get(health))
        .route("/sysinfo", get(sysinfo))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/access-token", post(generate_access_token))
        .route("/private", get(private))
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(mw_handle_leaked_5xx))
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(from_fn(mw_client_ip))
                .layer(TraceLayer::new_for_http().make_span_with(span))
                .layer(from_fn_with_state(state.clone(), mw_rate_limiter)),
        )
}
