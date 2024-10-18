use axum::{body::Body, http::Request};

use crate::request;

pub fn signup<'a>(username: &'a str, password: &'a str) -> impl Fn() -> Request<Body> + 'a {
    move || {
        request!(
            POST "/signup";
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&password={}", username, password)
        )
    }
}

pub fn login<'a>(username: &'a str, password: &'a str) -> impl Fn() -> Request<Body> + 'a {
    move || {
        request!(
            POST "/login";
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&password={}&remember=false", username, password)
        )
    }
}
