#[cfg(feature = "context")]
pub trait Context<T> {
    fn context<C>(self, context: C) -> Result<T, crate::InternalError>
    where
        C: std::fmt::Display + Send + Sync + 'static;
}

#[cfg(feature = "context")]
impl<T, E> Context<T> for Result<T, E>
where
    E: Into<anyhow::Error> + std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, crate::InternalError>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| crate::InternalError(anyhow::Error::from(e).context(context)))
    }
}
