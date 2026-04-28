pub(crate) type AppResult<T> = Result<T, Box<dyn core::error::Error + Send + Sync + 'static>>;
