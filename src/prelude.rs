pub use color_eyre::eyre::WrapErr;
pub use tracing::{debug, error, info, instrument, warn};

pub type Result<T> = core::result::Result<T, crate::internal_service_error::AppError>;
