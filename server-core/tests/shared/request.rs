use axum::{body::Body, http::Request};

use crate::request;

pub fn signup(username: &str, email: &str, password: &str) -> Request<Body> {
    request!(
        POST "/signup";
        "content-type" => "application/x-www-form-urlencoded";
        format!("username={}&email={}&password={}", username, email, password)
    )
}

pub fn login(username: &str, password: &str) -> Request<Body> {
    request!(
        POST "/login";
        "content-type" => "application/x-www-form-urlencoded";
        format!("username={}&password={}&remember=false", username, password)
    )
}
