use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Debug, clap::Parser)]
struct Args {
    /// The port number on which the server will listen for incoming connections.
    /// Example: `8080`
    #[arg(long, env("PORT"))]
    port: u16,

    /// The database connection URL used by the server.
    /// Example: `sqlite:///tmp/data/data.db` (or) `/tmp/data/data.db` (or) `./data.db`
    #[arg(long, env("DATABASE_URL"))]
    database_url: String,

    #[cfg(feature = "serve-dir")]
    /// The directory where the server's UI files are located.
    /// This should point to a valid local path containing frontend assets.
    /// Example: `./ui` or `/var/www/html`
    #[arg(long, env("UI_DIR"))]
    serve_dir: std::path::PathBuf,

    #[cfg(feature = "rate-limit")]
    /// The rate limit in the form of a string, e.g. "1/s", "10/min", "100/hour".
    /// Example: "10/min"
    #[arg(long, env("RATE_LIMIT"))]
    rate_limit: server::RateLimiterConfig,

    #[cfg(feature = "smtp")]
    /// The SMTP relay server used for sending emails.
    /// This should be a valid SMTP server address.
    /// Example: `"smtp.gmail.com"`
    #[arg(long, env("SMTP_RELAY"))]
    smtp_relay: String,

    #[cfg(feature = "smtp")]
    /// The port on which the SMTP relay server listens.
    /// Common defaults are `587` for real providers or `1025` for local testing.
    /// If unset, the default port for the relay host will be used.
    #[arg(long, env("SMTP_PORT"))]
    smtp_port: Option<u16>,

    #[cfg(feature = "smtp")]
    /// The username for authenticating with the SMTP server.
    /// Example: `"user@example.com"`
    #[arg(long, env("SMTP_USERNAME"))]
    smtp_username: Option<String>,

    #[cfg(feature = "smtp")]
    /// The password for the SMTP server.
    /// This should be kept secure and **not logged**.
    /// Example: `"supersecretpassword"`
    #[arg(long, env("SMTP_PASSWORD"))]
    smtp_password: Option<String>,

    #[cfg(feature = "smtp")]
    /// Directory containing files that define sender email addresses.
    ///
    /// Each file's basename (with or without an extension) represents
    /// a logical sender identifier (e.g. `noreply`), and the file's
    /// content is the actual email address to use (e.g. `noreply@yourdomain.com`).
    ///
    /// Example: senders/noreply.txt (contains: noreply@yourdomain.com)
    #[arg(long, env("SMTP_SENDERS_DIR"))]
    smtp_senders_dir: std::path::PathBuf,

    #[cfg(feature = "smtp")]
    #[arg(long, env("SMTP_TEMPLATES_DIR"))]
    smtp_templates_dir: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env(/* RUST_LOG env var sets logging level */))
        .init();

    let args = Args::parse();

    let server_opts = server::ServerOpts {
        database_url: args.database_url,
        port: args.port,

        #[cfg(feature = "rate-limit")]
        rate_limiter: args.rate_limit,

        #[cfg(feature = "serve-dir")]
        serve_dir: args.serve_dir,

        #[cfg(feature = "smtp")]
        smtp: server::SMTPConfig {
            relay: args.smtp_relay,
            port: args.smtp_port,
            username: args.smtp_username,
            password: args.smtp_password,
            senders_dir: args.smtp_senders_dir,
            templates_dir: args.smtp_templates_dir,
        },
    };

    if let Err(e) = server::serve(server_opts).await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
