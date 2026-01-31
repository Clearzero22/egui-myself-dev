//! # Error types
//!
//! This module defines the core error types for the Pomodoro timer module.
//! Errors are organized by domain:
//! - **Timer errors**: State management issues
//! - **Validation errors**: Invalid input parameters
//! - **Content errors**: Content loading and pagination issues
//! - **Storage errors**: Backend operation failures
//!
//! # Examples
//!
//! ```
//! use egui_demo_lib::demo::pomodoro_timer::core::{Error, Result};
//!
//! fn start_timer(running: bool) -> Result<()> {
//!     if running {
//!         return Err(Error::TimerAlreadyRunning);
//!     }
//!     Ok(())
//! }
//! ```

/// Core error type for Pomodoro timer operations.
///
/// This enum represents all possible errors that can occur in the core layer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Timer is already running and cannot be started again.
    TimerAlreadyRunning,
    
    /// Timer operation requires running timer but none is active.
    TimerNotRunning,
    
    /// Duration value is invalid (e.g., negative, exceeds bounds).
    InvalidDuration,
    
    /// Content failed to load or is empty.
    NoContentLoaded,
    
    /// Requested page index does not exist in paginated content.
    PageNotFound(usize),
    
    /// Storage backend operation failed with a message.
    StoreError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TimerAlreadyRunning => write!(f, "Timer is already running"),
            Error::TimerNotRunning => write!(f, "Timer is not running"),
            Error::InvalidDuration => write!(f, "Invalid duration specified"),
            Error::NoContentLoaded => write!(f, "No content loaded"),
            Error::PageNotFound(page) => write!(f, "Page {} not found", page),
            Error::StoreError(msg) => write!(f, "Store error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
