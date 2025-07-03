use std::error::Error;

use crate::Boxer;

pub trait Context<T> {
    fn context(self, context: impl ToString) -> Result<T, Boxer>;
}

impl<T, E: Error + 'static> Context<T> for Result<T, E> {
    fn context(self, context: impl ToString) -> Result<T, Boxer> {
        self.map_err(|e| Boxer::new(context.to_string(), e))
    }
}
