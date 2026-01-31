//! Core data models and business logic for Pomodoro timer.

pub mod config;
pub mod error;
pub mod timer;

pub use config::TimerConfig;
pub use error::{Error, Result};
pub use timer::{Phase, PomodoroTimer, TimerState};
