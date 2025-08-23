use std::{ffi::OsStr, fs};

use clap::Parser;
use sqlx::{AnyPool, any::install_default_drivers};

#[derive(Debug, clap::Parser)]
struct Args {
    /// The database connection URL used by the server.
    /// Example: `sqlite:///tmp/data/data.db` (or) `/tmp/data/data.db` (or) `./data.db`
    #[arg(long, env("DATABASE_URL"))]
    database_url: String,

    /// Directory containing SQL files with initial seed data.
    ///
    /// Each file in this directory should be a valid SQL script
    /// that can be executed to populate the database with initial data.
    #[arg(long, env("SEED_DIR"))]
    seed_dir: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    install_default_drivers();

    let pool = AnyPool::connect(&args.database_url)
        .await
        .unwrap_or_else(|e| exit(e, "Failed to connect to database"));

    let entries =
        fs::read_dir(&args.seed_dir).unwrap_or_else(|e| exit(e, "Failed to read seed directory"));

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                warn(err, "Failed to read seed directory entry");
                continue;
            }
        };

        if !entry.path().is_file() || entry.path().extension() != Some(OsStr::new("sql")) {
            continue;
        }

        let content = match fs::read_to_string(entry.path()) {
            Ok(content) => content,
            Err(err) => {
                warn(
                    err,
                    format!("Failed to read seed file: {}", entry.path().display()),
                );
                continue;
            }
        };

        let query_result = match sqlx::query(&content).execute(&pool).await {
            Ok(query_result) => query_result,
            Err(err) => {
                warn(
                    err,
                    format!("Failed to execute seed file: {}", entry.path().display()),
                );
                continue;
            }
        };

        println!(
            "Executed {} :: {} rows affected",
            entry.path().display(),
            query_result.rows_affected()
        );
    }
}

#[inline(always)]
fn exit(err: impl std::error::Error, context: impl AsRef<str>) -> ! {
    eprintln!("{} :: {}", context.as_ref(), err);
    std::process::exit(1)
}

#[inline(always)]
fn warn(err: impl std::error::Error, context: impl AsRef<str>) {
    eprintln!("{} :: {}", context.as_ref(), err);
}
