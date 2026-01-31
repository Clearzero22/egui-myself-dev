//! Pomodoro timer demo with automatic page-turning.

pub mod content;
pub mod core;
pub mod ui;

pub use content::{PageDirection, Pager, TextPager};
pub use core::{Error, Result, Phase, PomodoroTimer, TimerConfig, TimerState};

// Temporary wrapper for demo compatibility - will be replaced by Task 10
#[derive(Default)]
pub struct PomodoroTimerDemo {
    _placeholder: (),
}

impl crate::Demo for PomodoroTimerDemo {
    fn name(&self) -> &'static str {
        "🍅 Pomodoro Timer (New)"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .show(ctx, |ui| {
                ui.label("Pomodoro Timer - Under Construction");
                ui.label("This demo is being rebuilt with component architecture.");
            });
    }
}

// Type alias for demo compatibility
pub type PomodoroTimerDemoApp = PomodoroTimerDemo;
