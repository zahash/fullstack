pub use askama::{Error, Template};

#[cfg(feature = "verify-email")]
#[derive(askama::Template)]
#[template(path = "verify-email.html")]
pub struct VerifyEmail<'verification_token> {
    pub verification_token: &'verification_token str,
}
