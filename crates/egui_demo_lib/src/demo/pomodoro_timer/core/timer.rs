//! Pomodoro timer state machine.

use super::{config::TimerConfig, error::{Error, Result}};
use std::time::{Duration, Instant};

/// Timer phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Work,
    Break,
}

/// Timer state.
#[derive(Clone, Debug, PartialEq)]
pub enum TimerState {
    Idle,
    Running { remaining: Duration, phase: Phase },
    Paused { remaining: Duration, phase: Phase },
    Completed { phase: Phase },
}

/// Pomodoro timer.
pub struct PomodoroTimer {
    state: TimerState,
    config: TimerConfig,
    sessions_completed: usize,
    start_time: Option<Instant>,
    total_duration: Duration,
}

impl PomodoroTimer {
    /// Create new timer with default config.
    pub fn new(config: TimerConfig) -> Self {
        Self {
            state: TimerState::Idle,
            config,
            sessions_completed: 0,
            start_time: None,
            total_duration: Duration::ZERO,
        }
    }

    /// Start the timer.
    pub fn start(&mut self) -> Result<()> {
        if !matches!(self.state, TimerState::Idle | TimerState::Completed { .. }) {
            return Err(Error::TimerAlreadyRunning);
        }

        let phase = if self.sessions_completed % self.config.sessions_until_long_break == 0
            && self.sessions_completed > 0
        {
            Phase::Break
        } else {
            Phase::Work
        };

        let duration = match phase {
            Phase::Work => self.config.work_duration,
            Phase::Break => self.config.break_duration,
        };

        self.state = TimerState::Running {
            remaining: duration,
            phase,
        };
        self.start_time = Some(Instant::now());
        self.total_duration = duration;
        Ok(())
    }

    /// Pause the timer.
    pub fn pause(&mut self) -> Result<()> {
        match &self.state {
            TimerState::Running { remaining, phase } => {
                self.state = TimerState::Paused {
                    remaining: *remaining,
                    phase: *phase,
                };
                self.start_time = None;
                Ok(())
            }
            _ => Err(Error::TimerNotRunning),
        }
    }

    /// Resume the timer.
    pub fn resume(&mut self) -> Result<()> {
        match &self.state {
            TimerState::Paused { remaining, phase } => {
                self.state = TimerState::Running {
                    remaining: *remaining,
                    phase: *phase,
                };
                self.start_time = Some(Instant::now());
                Ok(())
            }
            _ => Err(Error::TimerNotRunning),
        }
    }

    /// Reset the timer.
    pub fn reset(&mut self) {
        self.state = TimerState::Idle;
        self.start_time = None;
        self.sessions_completed = 0;
    }

    /// Update timer with elapsed time. Returns true if timer completed.
    pub fn tick(&mut self, delta: Duration) -> bool {
        if let TimerState::Running { remaining, phase } = &mut self.state {
            if *remaining > delta {
                *remaining -= delta;
                false
            } else {
                // Timer completed
                let completed_phase = *phase;
                self.state = TimerState::Completed { phase: completed_phase };

                if completed_phase == Phase::Work {
                    self.sessions_completed += 1;
                }
                true
            }
        } else {
            false
        }
    }

    /// Get remaining time.
    pub fn remaining(&self) -> Duration {
        match &self.state {
            TimerState::Running { remaining, .. } => *remaining,
            TimerState::Paused { remaining, .. } => *remaining,
            _ => Duration::ZERO,
        }
    }

    /// Get progress (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        if self.total_duration.is_zero() {
            return 0.0;
        }
        let remaining = self.remaining().as_secs_f32();
        let total = self.total_duration.as_secs_f32();
        1.0 - (remaining / total)
    }

    /// Get current state.
    pub fn state(&self) -> &TimerState {
        &self.state
    }

    /// Get number of completed sessions.
    pub fn sessions_completed(&self) -> usize {
        self.sessions_completed
    }

    /// Check if timer is running.
    pub fn is_running(&self) -> bool {
        matches!(self.state, TimerState::Running { .. })
    }

    /// Check if timer is paused.
    pub fn is_paused(&self) -> bool {
        matches!(self.state, TimerState::Paused { .. })
    }
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self::new(TimerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_create() {
        let timer = PomodoroTimer::default();
        assert!(matches!(timer.state(), TimerState::Idle));
        assert_eq!(timer.sessions_completed(), 0);
    }

    #[test]
    fn test_timer_start() {
        let mut timer = PomodoroTimer::default();
        timer.start().unwrap();
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = PomodoroTimer::default();
        timer.start().unwrap();
        timer.pause().unwrap();
        assert!(timer.is_paused());
        timer.resume().unwrap();
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = PomodoroTimer::default();
        timer.start().unwrap();
        let initial = timer.remaining();
        timer.tick(Duration::from_secs(1));
        assert_eq!(timer.remaining(), initial - Duration::from_secs(1));
    }

    #[test]
    fn test_timer_progress() {
        let mut timer = PomodoroTimer::default();
        timer.start().unwrap();
        timer.tick(Duration::from_secs(60 * 12)); // 12 minutes
        assert!((timer.progress() - 0.48).abs() < 0.01); // ~48%
    }
}
