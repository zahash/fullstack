#[cfg(all(feature = "cli", feature = "env"))]
compile_error!("features `server/cli` and `server/env` are mutually exclusive");

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env(/* RUST_LOG env var sets logging level */))
        .init();

    #[cfg(feature = "env")]
    let server_opts = {
        let database_url = get_env_var::<String>("DATABASE_URL")?;
        let port = get_env_var::<u16>("PORT")?;

        #[cfg(feature = "rate-limit")]
        let rate_limiter = {
            let rate_limit = get_env_var::<String>("RATE_LIMIT")?;
            let (limit, interval) = parse_rate_limit(&rate_limit)?;
            server::RateLimiterConfig { limit, interval }
        };

        #[cfg(feature = "ui")]
        let ui_dir = get_env_var::<std::path::PathBuf>("UI_DIR")?;

        #[cfg(feature = "email")]
        let smtp = {
            let relay = get_env_var::<String>("SMTP_RELAY")?;
            let username = get_env_var::<String>("SMTP_USERNAME")?;
            let password = get_env_var::<String>("SMTP_PASSWORD")?;
            server::SMTPConfig {
                relay,
                username,
                password,
            }
        };

        server::ServerOpts {
            database_url,
            port,

            #[cfg(feature = "rate-limit")]
            rate_limiter,

            #[cfg(feature = "ui")]
            ui_dir,

            #[cfg(feature = "email")]
            smtp,
        }
    };

    #[cfg(feature = "cli")]
    let server_opts = {
        use clap::Parser;

        let args = Args::parse();

        #[cfg(feature = "rate-limit")]
        let rate_limiter = {
            let (limit, interval) = parse_rate_limit(&args.rate_limit)?;
            server::RateLimiterConfig { limit, interval }
        };

        server::ServerOpts {
            database_url: args.database_url,
            port: args.port,

            #[cfg(feature = "rate-limit")]
            rate_limiter,

            #[cfg(feature = "ui")]
            ui_dir: args.ui_dir,

            #[cfg(feature = "email")]
            smtp: server::SMTPConfig {
                relay: args.smtp_relay,
                username: args.smtp_username,
                password: args.smtp_password,
            },
        }
    };

    server::serve(server_opts).await.map_err(|e| e.into())
}

#[cfg(feature = "cli")]
#[derive(Debug, clap::Parser)]
struct Args {
    /// The port number on which the server will listen for incoming connections.
    /// Example: `8080`
    #[arg(long)]
    port: u16,

    /// The database connection URL used by the server.
    /// Example: `sqlite:///tmp/data/data.db` (or) `/tmp/data/data.db` (or) `./data.db`
    #[arg(long)]
    database_url: String,

    #[cfg(feature = "ui")]
    /// The directory where the server's UI files are located.
    /// This should point to a valid local path containing frontend assets.
    /// Example: `./ui` or `/var/www/html`
    #[arg(long)]
    ui_dir: std::path::PathBuf,

    #[cfg(feature = "rate-limit")]
    /// The rate limit in the form of a string, e.g. "1/s", "10/min", "100/hour".
    /// Example: "10/min"
    #[arg(long)]
    rate_limit: String,

    #[cfg(feature = "email")]
    /// The SMTP relay server used for sending emails.
    /// This should be a valid SMTP server address.
    /// Example: `"smtp.gmail.com"`
    #[arg(long)]
    smtp_relay: String,

    #[cfg(feature = "email")]
    /// The username for authenticating with the SMTP server.
    /// Example: `"user@example.com"`
    #[arg(long)]
    smtp_username: String,

    #[cfg(feature = "email")]
    /// The password for the SMTP server.
    /// This should be kept secure and **not logged**.
    /// Example: `"supersecretpassword"`
    #[arg(long)]
    smtp_password: String,
}

#[cfg(feature = "rate-limit")]
/// Parse a rate limit string like "10/s", "100/min", "1000/hour" into (limit, interval)
fn parse_rate_limit(s: &str) -> Result<(usize, std::time::Duration), Box<dyn std::error::Error>> {
    use std::time::Duration;

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

#[cfg(feature = "env")]
fn get_env_var<T: std::str::FromStr>(name: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    use boxer::Context;

    std::env::var(name)
        .context(format!("env var `{name}`"))?
        .parse::<T>()
        .context(format!(
            "cannot parse env var `{name}` as {}",
            std::any::type_name::<T>()
        ))
        .map_err(|e| e.into())
}

#[cfg(feature = "env")]
#[allow(dead_code)]
fn get_opt_env_var<T: std::str::FromStr>(
    name: &str,
) -> Result<Option<T>, Box<dyn std::error::Error>>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    use boxer::{Boxer, Context};

    match std::env::var(name) {
        Ok(val) => val
            .parse::<T>()
            .context(format!(
                "cannot parse env var `{}` as {}",
                name,
                std::any::type_name::<T>()
            ))
            .map(Some)
            .map_err(|e| e.into()),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(Boxer::new(format!("env var `{name}`"), e).into()),
    }
}
