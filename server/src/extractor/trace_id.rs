// use anyhow::{anyhow, Context};
// use axum::{
//     async_trait,
//     extract::FromRequestParts,
//     http::{request::Parts, HeaderValue},
// };
// use tracing::Span;

// use crate::{error::AppError, types::TraceId};

// pub const TRACE_ID_HEADER_NAME: &'static str = "X-Trace-ID";

// #[async_trait]
// impl<S> FromRequestParts<S> for TraceId {
//     type Rejection = AppError;

//     async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
//         // let headers = &mut parts.headers;

//         // match headers.get(TRACE_ID_HEADER_NAME) {
//         //     Some(header_value) => match header_value.to_str() {
//         //         Ok(trace_id) => Ok(TraceId(trace_id.to_string())),
//         //         Err(e) => Err(AppError::Internal(
//         //             anyhow!(e).context(format!("{} HeaderValue to_str", TRACE_ID_HEADER_NAME)),
//         //         )),
//         //     },
//         //     None => {
//         //         let trace_id = TraceId::new();
//         //         headers.insert(
//         //             TRACE_ID_HEADER_NAME,
//         //             HeaderValue::from_str(trace_id.as_str()).context("TraceId to HeaderValue")?,
//         //         );
//         //         tracing::info!("Generated Trace ID: {}", trace_id.as_str());
//         //         Ok(trace_id)
//         //     }
//         // }
//         // Retrieve the trace ID from the active span
//         let trace_id = Span::current().metadata();

//         if let Some(trace_id) = trace_id {
//             Ok(TraceId(trace_id.to_string()))
//         } else {
//             Err(AppError::Internal("Trace ID not found in span".to_string()))
//         }
//     }
// }
