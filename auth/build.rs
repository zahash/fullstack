use std::{fs, path::PathBuf};

const DANGEROUS_FEATURES: &[(&str, &str)] = &[
    (
        "await-tasks",
        "Awaiting background tasks in production blocks responses and increases latency \
                    — only enable for testing to make sure task is completed before test ends",
    ),
    (
        "smtp--no-tls",
        "SMTP is running in insecure mode. TLS certificate validation is disabled \
                    — only enable for local/testing",
    ),
];

fn main() {
    let target_dir =
        PathBuf::from(std::env::var("OUT_DIR").unwrap(/* Infallible */)).join("dangerous_features");
    println!(
        "cargo:rustc-env=DANGEROUS_FEATURES_DIR={}",
        target_dir.display()
    );

    // Ensure fresh directory
    let _ = fs::remove_dir_all(&target_dir);
    fs::create_dir_all(&target_dir).unwrap();

    for (feature, message) in DANGEROUS_FEATURES {
        if std::env::var_os(format!(
            "CARGO_FEATURE_{}",
            feature.replace("-", "_").to_uppercase()
        ))
        .is_some()
        {
            println!(
                "cargo:warning=crate compiled with `{}` feature enabled. {}",
                feature, message
            );

            let file_path = target_dir.join(feature);
            fs::write(&file_path, message).unwrap_or_else(|_| {
                panic!("failed to write dangerous feature `{}` to file", feature)
            });
        }
    }
}
