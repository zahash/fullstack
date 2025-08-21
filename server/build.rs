fn main() {
    warn_feature(
        "await-tasks",
        "Awaiting background tasks in production blocks responses and increases latency \
                    — only enable for testing to make sure task is completed before test ends",
    );
    warn_feature(
        "smtp--no-tls",
        "SMTP is running in insecure mode. TLS certificate validation is disabled \
                    — only enable for local/testing",
    );
}

fn warn_feature(feature: &str, message: &str) {
    if std::env::var_os(&format!(
        "CARGO_FEATURE_{}",
        feature.replace("-", "_").to_uppercase()
    ))
    .is_some()
    {
        println!(
            "cargo:warning=crate compiled with `{}` feature enabled. {}",
            feature, message
        );
    }
}
