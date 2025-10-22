#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    message: String,

    #[cfg(feature = "error-kind")]
    kind: &'static str,

    #[cfg_attr(feature = "openapi", schema(example = "2025-08-01T12:34:56Z"))]
    datetime: Option<String>,

    #[cfg_attr(
        feature = "openapi",
        schema(example = "Please check the response headers for `x-trace-id`")
    )]
    help: &'static str,
}

impl ErrorResponse {
    const HELP: &str = "Please check the response headers for `x-trace-id`, include the datetime and raise a support ticket.";

    pub fn new(
        message: impl Into<String>,
        #[cfg(feature = "error-kind")] kind: &'static str,
    ) -> Self {
        Self {
            message: message.into(),
            datetime: time::OffsetDateTime::now_utc()
                .format(&time::macros::format_description!(
                    "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
                ))
                .ok(),
            help: Self::HELP,

            #[cfg(feature = "error-kind")]
            kind,
        }
    }
}

impl<#[cfg(not(feature = "error-kind"))] E, #[cfg(feature = "error-kind")] E: crate::ErrorKind>
    From<E> for ErrorResponse
where
    E: std::error::Error,
{
    fn from(error: E) -> Self {
        Self::new(error.to_string(), error.kind())
    }
}
