use time::{format_description::well_known::Iso8601, OffsetDateTime};

pub fn now_iso8601() -> Option<String> {
    OffsetDateTime::now_utc()
        .format(&Iso8601::DATE_TIME_OFFSET)
        .inspect_err(|e| {
            tracing::warn!(
                "unable to format OffsetDateTime::now_utc() as Iso8601 :: {:?}",
                e
            )
        })
        .ok()
}
