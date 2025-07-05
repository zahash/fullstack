use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct Boxer {
    context: String,
    source: Box<dyn Error>,
}

impl Boxer {
    pub fn new<E: Error + 'static>(context: String, source: E) -> Self {
        Boxer {
            context,
            source: Box::new(source),
        }
    }

    #[inline]
    pub fn context(&self) -> &str {
        &self.context
    }

    #[inline]
    pub fn source(&self) -> &(dyn Error + 'static) {
        &*self.source
    }
}

impl Error for Boxer {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source())
    }
}

impl Display for Boxer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error:")?;
        writeln!(f, "• {}", self.context)?;
        let mut source = <Boxer as Error>::source(self);
        while let Some(err) = source {
            writeln!(f, "↳ {}", err)?;
            source = err.source();
        }
        Ok(())
    }
}
