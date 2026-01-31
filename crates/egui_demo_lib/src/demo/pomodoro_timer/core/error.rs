//! Error types for Pomodoro timer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TimerAlreadyRunning,
    TimerNotRunning,
    InvalidDuration,
    NoContentLoaded,
    PageNotFound(usize),
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
