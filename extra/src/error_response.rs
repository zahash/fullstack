#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    message: String,

    #[cfg_attr(feature = "openapi", schema(example = "2025-08-01T12:34:56Z"))]
    datetime: Option<String>,

    #[cfg_attr(
        feature = "openapi",
        schema(example = "Please check the response headers for `x-request-id`")
    )]
    help: &'static str,
}

impl ErrorResponse {
    const HELP: &str = "Please check the response headers for `x-request-id`, include the datetime and raise a support ticket.";
}

impl<E: std::error::Error> From<E> for ErrorResponse {
    fn from(error: E) -> Self {
        Self {
            message: error.to_string(),
            datetime: time::OffsetDateTime::now_utc()
                .format(&time::macros::format_description!(
                    "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
                ))
                .ok(),
            help: Self::HELP,
        }
    }
}
