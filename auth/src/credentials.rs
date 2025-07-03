pub trait Credentials {
    type Error;

    fn try_from_headers(headers: &http::HeaderMap) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized;
}
