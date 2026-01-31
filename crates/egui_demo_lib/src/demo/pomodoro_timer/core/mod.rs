mod config;
mod error;
mod timer;

pub use config::TimerConfig;
pub use error::{Error, Result};
pub use timer::{Phase, PomodoroTimer, TimerState};
