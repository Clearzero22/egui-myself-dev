//! Timer configuration.

use std::time::Duration;

/// Timer configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimerConfig {
    /// Work session duration (default: 25 minutes)
    pub work_duration: Duration,
    /// Short break duration (default: 5 minutes)
    pub break_duration: Duration,
    /// Long break duration (default: 15 minutes)
    pub long_break_duration: Duration,
    /// Number of work sessions before long break (default: 4)
    pub sessions_until_long_break: usize,
    /// Auto-start break after work session
    pub auto_start_break: bool,
    /// Auto-start work after break
    pub auto_start_work: bool,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
            auto_start_break: false,
            auto_start_work: false,
        }
    }
}
