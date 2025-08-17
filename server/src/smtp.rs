use std::{path::PathBuf, str::FromStr, sync::Arc};

use axum::{Json, response::IntoResponse};
use contextual::Context;
use email::Email;
use extra::ErrorResponse;
use http::StatusCode;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart},
    transport::smtp::response::Response,
};
use tera::Tera;
use time::OffsetDateTime;
use token::Token;

const EMAIL_VERIFICATION_TOKEN_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60);

// TODO: JWT could be a very good candidate for Verification token
// it is stateless and only used once.
// it would be a no-op if used more than once.
pub type VerificationToken = Token<4>;

#[derive(Clone)]
pub struct Smtp {
    pub transport: AsyncSmtpTransport<Tokio1Executor>,
    pub senders: Arc<SmtpSenders>,
    pub tera: Arc<Tera>,
}

pub struct SmtpSenders {
    dir: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum SmtpSendersError {
    #[error("{0}")]
    EmailFormat(&'static str),

    #[error("{0}")]
    Io(#[from] contextual::Error<std::io::Error>),
}

impl SmtpSenders {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub async fn get(&self, sender: &str) -> Result<Email, SmtpSendersError> {
        let content = std::fs::read_to_string(self.dir.join(sender))
            .or_else(|_| std::fs::read_to_string(self.dir.join(format!("{sender}.txt"))))
            .context(format!("smtp sender `{sender}`"))?;

        Email::from_str(content.trim()).map_err(SmtpSendersError::EmailFormat)
    }
}

pub async fn initiate_email_verification(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    smtp: &Smtp,
    email: &Email,
) -> Result<Option<Response>, InitiateEmailVerificationError> {
    let record = sqlx::query!(
        r#"SELECT email_verified FROM users WHERE email = ? LIMIT 1"#,
        email
    )
    .fetch_optional(pool)
    .await
    .context("check if email already verified")?;

    let Some(record) = record else {
        return Err(InitiateEmailVerificationError::EmailDoesNotExist(
            email.clone(),
        ));
    };

    // no-op if email already verified
    if record.email_verified {
        return Ok(None);
    }

    let verification_token = VerificationToken::random();
    let verification_token_hash = verification_token.hash_sha256();
    let verification_token_encoded = verification_token.base64encoded();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = created_at + EMAIL_VERIFICATION_TOKEN_TTL;

    let message = {
        let noreply: Email = smtp
            .senders
            .get("noreply")
            .await
            .context("SmtpSenders::get `noreply`")?;

        let from = Mailbox::new(Some("noreply".into()), noreply.into());
        let to = Mailbox::new(None, email.clone().into());

        let subject = "Verify your Email";

        let plain_text_content = format!("verfication token: {verification_token_encoded}");
        let html_content = {
            let mut context = tera::Context::new();
            context.insert("verification_token", &verification_token_encoded);
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

    sqlx::query!(
        r#"
        DELETE FROM email_verification_tokens
        where email = ?
        "#,
        email
    )
    .execute(pool)
    .await
    .context("delete existing email verification tokens")?;

    sqlx::query!(
        r#"
        INSERT INTO email_verification_tokens
        (token_hash, email, created_at, expires_at)
        VALUES (?, ?, ?, ?)
        "#,
        verification_token_hash,
        email,
        created_at,
        expires_at
    )
    .execute(pool)
    .await
    .context("insert email verification token")?;

    let response = smtp
        .transport
        .send(message)
        .await
        .context("send verification email")?;

    Ok(Some(response))
}

#[derive(thiserror::Error, Debug)]
pub enum InitiateEmailVerificationError {
    #[error("email `{0}` does not exist")]
    EmailDoesNotExist(Email),

    #[error("{0}")]
    SmtpSenders(#[from] contextual::Error<SmtpSendersError>),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),

    #[error("{0}")]
    EmailTemplate(#[from] contextual::Error<tera::Error>),

    #[error("{0}")]
    EmailContent(#[from] contextual::Error<lettre::error::Error>),

    #[error("{0}")]
    SmtpTransport(#[from] contextual::Error<lettre::transport::smtp::Error>),
}

impl IntoResponse for InitiateEmailVerificationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            InitiateEmailVerificationError::EmailDoesNotExist(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::NOT_FOUND, Json(ErrorResponse::from(self))).into_response()
            }
            InitiateEmailVerificationError::SmtpSenders(_)
            | InitiateEmailVerificationError::Sqlx(_)
            | InitiateEmailVerificationError::EmailTemplate(_)
            | InitiateEmailVerificationError::EmailContent(_)
            | InitiateEmailVerificationError::SmtpTransport(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
