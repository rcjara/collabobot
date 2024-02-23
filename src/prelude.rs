pub use color_eyre::eyre::{Result, WrapErr};
pub use tracing::{debug, error, info, instrument, warn};

pub type AppResult<T> = core::result::Result<T, crate::internal_service_error::AppError>;
