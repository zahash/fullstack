use std::time::Duration;

use axum::response::IntoResponse;
use contextual::Context;
use email::Email;
use http::StatusCode;
use lettre::{
    AsyncTransport, Message,
    message::{Mailbox, MultiPart},
    transport::smtp::response::Response,
};
use token::signed;

pub fn verification_link(
    secret: &[u8],
    host: &str,
    token: &signed::Signed<Email>,
) -> Result<String, signed::EncodeError> {
    // TODO: /verify-email is hardcoded here and it is defined in two places
    //          once here and once more in the email verification handler
    //          maybe move this whole module closer to the email verification handlers
    Ok(format!(
        "{host}/verify-email?token={}",
        token.encode(secret)?
    ))
}

pub fn verification_token(email: Email) -> signed::Signed<Email> {
    signed::Signed::new(email).with_ttl(Duration::from_secs(60 * 60))
}

pub async fn send_verification_email(
    smtp: &super::Smtp,
    email: &Email,
    verification_link: &str,
) -> Result<Response, SendVerificationEmailError> {
    let message = {
        let noreply: Email = smtp
            .senders
            .get("noreply")
            .await
            .context("SmtpSenders::get `noreply`")?;

        let from = Mailbox::new(Some("noreply".into()), noreply.into());
        let to = Mailbox::new(None, email.clone().into());

        let subject = "Verify your Email";

        let plain_text_content = format!("verfication link: {verification_link}");
        let html_content = {
            let mut context = tera::Context::new();
            context.insert("verification_link", &verification_link);
            smtp.tera
                .render("verify-email.html", &context)
                .context("render verify-email template")?
        };

        Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(
                plain_text_content,
                html_content,
            ))
            .context("verify-email message builder")?
    };

    let response = smtp
        .transport
        .send(message)
        .await
        .context("send verification email")?;

    Ok(response)
}

#[derive(thiserror::Error, Debug)]
pub enum SendVerificationEmailError {
    #[error("{0}")]
    SmtpSenders(#[from] contextual::Error<super::SmtpSendersError>),

    #[error("{0}")]
    EmailTemplate(#[from] contextual::Error<tera::Error>),

    #[error("{0}")]
    EmailContent(#[from] contextual::Error<lettre::error::Error>),

    #[error("{0}")]
    SmtpTransport(#[from] contextual::Error<lettre::transport::smtp::Error>),
}

impl extra::ErrorKind for SendVerificationEmailError {
    fn kind(&self) -> &'static str {
        match self {
            SendVerificationEmailError::SmtpSenders(_) => "email.verification.smtp-senders",
            SendVerificationEmailError::EmailTemplate(_) => "email.verification.email-template",
            SendVerificationEmailError::EmailContent(_) => "email.verification.email-content",
            SendVerificationEmailError::SmtpTransport(_) => "email.verification.smtp-transport",
        }
    }
}

impl IntoResponse for SendVerificationEmailError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SendVerificationEmailError::SmtpSenders(_)
            | SendVerificationEmailError::EmailTemplate(_)
            | SendVerificationEmailError::EmailContent(_)
            | SendVerificationEmailError::SmtpTransport(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
