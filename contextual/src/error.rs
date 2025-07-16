use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct Error<E> {
    pub context: String,
    pub source: E,
}

impl<E> std::error::Error for Error<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl<E> Display for Error<E>
where
    E: std::error::Error + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error:")?;
        writeln!(f, "• {}", self.context)?;
        let mut source = <Self as std::error::Error>::source(self);
        while let Some(err) = source {
            writeln!(f, "↳ {err}")?;
            source = err.source();
        }
        Ok(())
    }
}
