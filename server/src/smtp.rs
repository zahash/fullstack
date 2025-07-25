use std::{path::PathBuf, str::FromStr, sync::Arc};

use axum::{Json, response::IntoResponse};
use contextual::Context;
use dashcache::DashCache;
use data_access::DataAccess;
use email::Email;
use extra::json_error_response;
use http::StatusCode;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart},
    transport::smtp::response::Response,
};
use tag::Tag;
use tera::Tera;
use time::OffsetDateTime;
use token::Token;

const EMAIL_VERIFICATION_TOKEN_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60);

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
    data_access: &DataAccess,
    smtp: &Smtp,
    email: &Email,
) -> Result<Response, InitiateEmailVerificationError> {
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

    #[derive(Debug, Clone)]
    struct Row {
        user_id: i64,
    }

    let user_id = data_access
        .read(
            |pool| {
                sqlx::query_as!(
                    Row,
                    r#"
                    SELECT id as "user_id!" FROM users 
                    WHERE email = ? LIMIT 1
                    "#,
                    email
                )
                .fetch_optional(pool)
            },
            "user_id__from__email",
            email.clone(),
            |value| match value {
                Some(row) => vec![Tag {
                    table: "users",
                    primary_key: Some(row.user_id),
                }],
                None => vec![Tag {
                    table: "users",
                    primary_key: None,
                }],
            },
            DashCache::new,
        )
        .await
        .context("email -> user_id")?
        .ok_or(InitiateEmailVerificationError::EmailDoesNotExist(
            email.clone(),
        ))?
        .user_id;

    data_access
        .write(
            |pool| {
                sqlx::query!(
                    r#"
                    INSERT INTO email_verification_tokens
                    (token_hash, user_id, created_at, expires_at)
                    VALUES (?, ?, ?, ?)
                    RETURNING id as "id!"
                    "#,
                    verification_token_hash,
                    user_id,
                    created_at,
                    expires_at
                )
                .fetch_one(pool)
            },
            |value| {
                vec![
                    Tag {
                        table: "email_verification_tokens",
                        primary_key: None,
                    },
                    Tag {
                        table: "email_verification_tokens",
                        primary_key: Some(value.id),
                    },
                ]
            },
        )
        .await
        .context("insert email_verification_token")?;

    let response = smtp
        .transport
        .send(message)
        .await
        .context("send verification email")?;

    Ok(response)
}

#[derive(thiserror::Error, Debug)]
pub enum InitiateEmailVerificationError {
    #[error("email does not exist :: {0}")]
    EmailDoesNotExist(Email),

    #[error("{0}")]
    SmtpSenders(#[from] contextual::Error<SmtpSendersError>),

    #[error("{0}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),

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
                tracing::info!("{:?}", self);
                (StatusCode::NOT_FOUND, Json(json_error_response(self))).into_response()
            }
            InitiateEmailVerificationError::SmtpSenders(_)
            | InitiateEmailVerificationError::DataAccess(_)
            | InitiateEmailVerificationError::EmailTemplate(_)
            | InitiateEmailVerificationError::EmailContent(_)
            | InitiateEmailVerificationError::SmtpTransport(_) => {
                tracing::error!("{:?}", self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
