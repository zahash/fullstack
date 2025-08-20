#[cfg(feature = "error-response")]
mod error_response;
#[cfg(feature = "error-response")]
pub use error_response::ErrorResponse;

#[cfg(feature = "error-kind")]
mod error_kind;
#[cfg(feature = "error-kind")]
pub use error_kind::ErrorKind;
