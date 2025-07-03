use std::{path::PathBuf, str::FromStr, time::Duration};

use anyhow::Context;
use clap::{Parser, Subcommand};

use server::{RateLimiterConfig, ServerOpts, serve};
use tracing_subscriber::filter::LevelFilter;

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

        /// The rate limit in the form of a string, e.g. "1/s", "10/min", "100/hour".
        /// Example: "10/min"
        #[arg(long)]
        rate_limit: String,
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

/// Parse a rate limit string like "10/s", "100/min", "1000/hour" into (limit, interval)
fn parse_rate_limit(s: &str) -> Result<(usize, std::time::Duration), Box<dyn std::error::Error>> {
    let Some((first, second)) = s.trim().split_once('/') else {
        return Err("invalid rate limit format".into());
    };
    let limit = first.parse::<usize>()?;
    let interval = match second.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Duration::from_secs(1),
        "m" | "min" | "minute" | "minutes" => Duration::from_secs(60),
        "h" | "hr" | "hour" | "hours" => Duration::from_secs(60 * 60),
        _ => return Err("invalid rate interval".into()),
    };
    Ok((limit, interval))
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
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();

    match Args::parse().cmd {
        Command::Server {
            port,
            database_url,
            ui_dir,
            rate_limit,
            // smtp_relay,
            // smtp_username,
            // smtp_password,
        } => {
            let (limit, interval) = match parse_rate_limit(&rate_limit) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Invalid rate limit string: {e}");
                    std::process::exit(1);
                }
            };
            return serve(ServerOpts {
                database_url,
                port,
                ui_dir,
                rate_limiter: RateLimiterConfig { limit, interval },
                // smtp: server_core::SMTPConfig {
                //     relay: smtp_relay,
                //     username: smtp_username,
                //     password: smtp_password,
                // },
            })
            .await
            .map_err(|e| e.into());
        }
    }
}
