pub fn json_error_response(err: impl std::error::Error) -> serde_json::Value {
    use time::format_description::well_known::Iso8601;

    let now_iso8601 = time::OffsetDateTime::now_utc()
        .format(&Iso8601::DATE_TIME_OFFSET)
        .ok();

    serde_json::json!({
        "error": err.to_string(),
        "help": "Please check the response headers for `x-request-id`, include the datetime and raise a support ticket.",
        "datetime": now_iso8601
    })
}
