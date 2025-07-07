pub trait Context<T, E> {
    fn context(self, context: impl ToString) -> Result<T, crate::Error<E>>;
    fn context_with(self, context: impl FnOnce() -> String) -> Result<T, crate::Error<E>>;
}

impl<T, E> Context<T, E> for Result<T, E> {
    fn context(self, context: impl ToString) -> Result<T, crate::Error<E>> {
        self.map_err(|e| crate::Error {
            context: context.to_string(),
            source: e,
        })
    }

    fn context_with(self, context: impl FnOnce() -> String) -> Result<T, crate::Error<E>> {
        self.map_err(|e| crate::Error {
            context: context(),
            source: e,
        })
    }
}
