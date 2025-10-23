use std::{path::PathBuf, str::FromStr, sync::Arc};

use contextual::Context;
use email::Email;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use tera::Tera;

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
