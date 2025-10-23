pub mod check_availability;

#[cfg(feature = "smtp")]
pub mod verify_email;

#[cfg(feature = "smtp")]
pub mod initiate_verification;

use email::Email;
use sqlx::{Executor, Sqlite};

#[cfg(feature = "smtp")]
pub fn verification_link(
    secret: &[u8],
    host: &str,
    token: &signature::Signed<Email>,
) -> Result<String, signature::EncodeError> {
    Ok(format!(
        "{host}/{}?token={}",
        verify_email::PATH,
        token.encode(secret)?
    ))
}

#[cfg(feature = "smtp")]
pub fn verification_token(email: Email) -> signature::Signed<Email> {
    signature::Signed::new(email).with_ttl(std::time::Duration::from_secs(60 * 60))
}

#[cfg(feature = "smtp")]
pub async fn send_verification_email(
    smtp: &crate::smtp::Smtp,
    email: &Email,
    verification_link: &str,
) -> Result<lettre::transport::smtp::response::Response, SendVerificationEmailError> {
    use contextual::Context;
    use lettre::{
        AsyncTransport, Message,
        message::{Mailbox, MultiPart},
    };

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

#[cfg(feature = "smtp")]
#[derive(thiserror::Error, Debug)]
pub enum SendVerificationEmailError {
    #[error("{0}")]
    SmtpSenders(#[from] contextual::Error<crate::smtp::SmtpSendersError>),

    #[error("{0}")]
    EmailTemplate(#[from] contextual::Error<tera::Error>),

    #[error("{0}")]
    EmailContent(#[from] contextual::Error<lettre::error::Error>),

    #[error("{0}")]
    SmtpTransport(#[from] contextual::Error<lettre::transport::smtp::Error>),
}

#[cfg(feature = "smtp")]
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

#[cfg(feature = "smtp")]
impl axum::response::IntoResponse for SendVerificationEmailError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SendVerificationEmailError::SmtpSenders(_)
            | SendVerificationEmailError::EmailTemplate(_)
            | SendVerificationEmailError::EmailContent(_)
            | SendVerificationEmailError::SmtpTransport(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub async fn exists<'a, E: Executor<'a, Database = Sqlite>>(
    ex: E,
    email: &Email,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT id as "user_id!" FROM users WHERE email = ? LIMIT 1"#,
        email
    )
    .fetch_optional(ex)
    .await?;

    match row {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
