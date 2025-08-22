use clap::Parser;

// TODO: introduce other databases, like postgres and mysql

#[derive(Debug, clap::Parser)]
struct Serve {
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
    #[arg(long, env("SERVE_DIR"))]
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
    let mut args_os = std::env::args_os().skip(1).peekable();

    if let Some(arg) = args_os.peek()
        && arg == "features"
    {
        println!("{:?}", features());
        return;
    }

    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::{EnvFilter, fmt};

        fmt()
            .with_env_filter(EnvFilter::from_default_env(/* RUST_LOG env var sets logging level */))
            .init()
    };

    #[cfg(feature = "tracing")]
    warn_dangerous_features();

    #[cfg(feature = "profiles")]
    load_profile();

    let args = Serve::parse();

    let port = args.port;
    let opts = server::ServerOpts::from(args);

    match server::router(opts).await {
        Err(err) => exit(err),
        Ok(router) => {
            if let Err(err) = server::serve(router, port).await {
                exit(err)
            }
        }
    }
}

const fn features() -> &'static [&'static str] {
    &[
        #[cfg(feature = "await-tasks")]
        "await-tasks",
        #[cfg(feature = "client-ip")]
        "client-ip",
        #[cfg(feature = "openapi")]
        "openapi",
        #[cfg(feature = "profiles")]
        "profiles",
        #[cfg(feature = "rate-limit")]
        "rate-limit",
        #[cfg(feature = "serve-dir")]
        "serve-dir",
        #[cfg(feature = "smtp")]
        "smtp",
        #[cfg(feature = "smtp--no-tls")]
        "smtp--no-tls",
        #[cfg(feature = "tracing")]
        "tracing",
    ]
}

#[cfg(feature = "profiles")]
fn load_profile() {
    use std::{
        env::{VarError, var},
        path::Path,
    };

    fn load_profile_from_filename(filename: impl AsRef<Path>) {
        match dotenvy::from_filename_override(&filename) {
            Ok(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("loaded profile {:?}", filename.as_ref());
            }
            Err(dotenvy::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
                #[cfg(feature = "tracing")]
                tracing::warn!("{:?} not found", filename.as_ref());
            }
            Err(err) => exit(err),
        };
    }

    match var("RUST_PROFILE") {
        Ok(profile) => {
            let profile = profile.to_lowercase();

            #[cfg(feature = "tracing")]
            tracing::info!("RUST_PROFILE `{profile}`");

            load_profile_from_filename(".env");
            if profile != "default" {
                load_profile_from_filename(format!(".env.{profile}"));
            }
        }
        Err(err) => match err {
            VarError::NotPresent => {
                #[cfg(feature = "tracing")]
                tracing::warn!("RUST_PROFILE not found");
            }
            VarError::NotUnicode(_) => exit(err),
        },
    };
}

#[cfg(feature = "tracing")]
fn warn_dangerous_features() {
    use std::{fs, path::PathBuf};

    // DANGEROUS_FEATURES_DIR is set by build.rs
    let dir = PathBuf::from(env!("DANGEROUS_FEATURES_DIR"));
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(message) = fs::read_to_string(entry.path()) {
                tracing::warn!(
                    "crate compiled with `{}` feature enabled. {}",
                    entry.file_name().to_string_lossy(),
                    message
                );
            }
        }
    }
}

fn exit(err: impl std::error::Error) {
    eprintln!("{err}");
    std::process::exit(1);
}

impl From<Serve> for server::ServerOpts {
    fn from(serve: Serve) -> Self {
        server::ServerOpts {
            database: server::DatabaseConfig {
                url: serve.database_url,
            },

            #[cfg(feature = "rate-limit")]
            rate_limiter: serve.rate_limit,

            #[cfg(feature = "serve-dir")]
            serve_dir: serve.serve_dir,

            #[cfg(feature = "smtp")]
            smtp: server::SmtpConfig {
                relay: serve.smtp_relay,
                port: serve.smtp_port,
                username: serve.smtp_username,
                password: serve.smtp_password,
                senders_dir: serve.smtp_senders_dir,
                templates_dir: serve.smtp_templates_dir,
            },
        }
    }
}
