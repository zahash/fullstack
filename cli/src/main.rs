use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use clap::{Parser, Subcommand};

use server::{ServerOpts, run};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the server with the specified configuration.
    Server {
        /// The port number on which the server will listen for incoming connections.
        /// Example: `8080`
        #[arg(long)]
        port: u16,

        /// The database connection URL used by the server.
        /// Example: `sqlite:///tmp/data/data.db` (or) `/tmp/data/data.db` (or) `./data.db`
        #[arg(long)]
        database_url: String,

        /// The directory where the server's UI files are located.
        /// This should point to a valid local path containing frontend assets.
        /// Example: `./ui` or `/var/www/html`
        #[arg(long)]
        ui_dir: PathBuf,
        // /// The SMTP relay server used for sending emails.
        // /// This should be a valid SMTP server address.
        // /// Example: `"smtp.gmail.com"`
        // #[arg(long)]
        // smtp_relay: String,

        // /// The username for authenticating with the SMTP server.
        // /// Example: `"user@example.com"`
        // #[arg(long)]
        // smtp_username: String,

        // /// The password for the SMTP server.
        // /// This should be kept secure and **not logged**.
        // /// Example: `"supersecretpassword"`
        // #[arg(long)]
        // smtp_password: String,
    },
}

#[allow(dead_code)]
fn get_var<T: FromStr>(name: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    Ok(std::env::var(name)?.parse::<T>().with_context(|| {
        format!(
            "cannot parse env var `{}` as {}",
            name,
            std::any::type_name::<T>()
        )
    })?)
}

#[allow(dead_code)]
fn get_opt_var<T: FromStr>(name: &str) -> Result<Option<T>, Box<dyn std::error::Error>>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(name) {
        Ok(val) => Ok(val
            .parse::<T>()
            .with_context(|| {
                format!(
                    "cannot parse env var `{}` as {}",
                    name,
                    std::any::type_name::<T>()
                )
            })
            .map(Some)?),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    match Args::parse().cmd {
        Command::Server {
            port,
            database_url,
            ui_dir,
            // smtp_relay,
            // smtp_username,
            // smtp_password,
        } => {
            run(ServerOpts {
                database_url,
                port,
                ui_dir,
                // smtp: server::SMTPConfig {
                //     relay: smtp_relay,
                //     username: smtp_username,
                //     password: smtp_password,
                // },
            })
            .await
        }
    }
}
