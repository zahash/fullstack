pub fn axum_error_response(
    status: axum::http::StatusCode,
    err: impl std::error::Error,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    use time::format_description::well_known::Iso8601;

    let now_iso8601 = time::OffsetDateTime::now_utc()
        .format(&Iso8601::DATE_TIME_OFFSET)
        .inspect_err(|e| {
            tracing::warn!(
                "unable to format OffsetDateTime::now_utc() as Iso8601 :: {:?}",
                e
            )
        })
        .ok();

    tracing::info!("{:?}", err);
    (
        status,
        axum::Json(serde_json::json!({
            "error": err.to_string(),
            "help": "Please check the response headers for `x-request-id`, include the datetime and raise a support ticket.",
            "datetime": now_iso8601
        })),
    )
        .into_response()
}
