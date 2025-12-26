/// Logging utilities
pub mod logging;

/// Global error handling
pub mod error;

// Re-export commonly used types
pub use error::{AppError, AppResult, ErrorCode, ErrorContext};

#[cfg(test)]
mod logging_test;
