//! Timer display component.

use crate::demo::pomodoro_timer::core::{PomodoroTimer, TimerState, Phase};
use egui::{self, Ui, Color32, RichText, Stroke, vec2, Pos2};

/// Timer display style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayStyle {
    Digital,
    Circular,
}

/// Timer display renderer.
pub struct TimerDisplay {
    style: DisplayStyle,
    show_phase: bool,
}

impl TimerDisplay {
    /// Create new digital display.
    pub fn new_digital() -> Self {
        Self {
            style: DisplayStyle::Digital,
            show_phase: true,
        }
    }

    /// Create new circular display.
    pub fn new_circular() -> Self {
        Self {
            style: DisplayStyle::Circular,
            show_phase: true,
        }
    }

    /// Render timer display.
    pub fn render(&self, ui: &mut Ui, timer: &PomodoroTimer) {
        match self.style {
            DisplayStyle::Digital => self.render_digital(ui, timer),
            DisplayStyle::Circular => self.render_circular(ui, timer),
        }
    }

    fn render_digital(&self, ui: &mut Ui, timer: &PomodoroTimer) {
        ui.vertical_centered(|ui| {
            let remaining = timer.remaining();
            let minutes = remaining.as_secs() / 60;
            let seconds = remaining.as_secs() % 60;

            // Time display
            ui.label(
                RichText::new(format!("{:02}:{:02}", minutes, seconds))
                    .size(48.0)
                    .strong()
            );

            // Phase label
            if self.show_phase {
                let (phase_text, color) = match timer.state() {
                    TimerState::Running { phase, .. } => match phase {
                        Phase::Work => ("工作中", Color32::GREEN),
                        Phase::Break => ("休息中", Color32::BLUE),
                    },
                    TimerState::Paused { phase, .. } => match phase {
                        Phase::Work => ("已暂停", Color32::YELLOW),
                        Phase::Break => ("休息暂停", Color32::YELLOW),
                    },
                    TimerState::Idle => ("准备开始", Color32::GRAY),
                    TimerState::Completed { .. } => ("已完成", Color32::LIGHT_GREEN),
                };

                ui.label(RichText::new(phase_text).color(color).size(16.0));
            }
        });
    }

    fn render_circular(&self, ui: &mut Ui, timer: &PomodoroTimer) {
        ui.vertical_centered(|ui| {
            let desired_size = vec2(200.0, 200.0);
            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::click());

            let progress = timer.progress();
            let remaining = timer.remaining();
            let minutes = remaining.as_secs() / 60;
            let seconds = remaining.as_secs() % 60;

            // Draw circular progress
            let painter = ui.painter_at(rect);
            let center = rect.center();
            let radius = rect.width().min(rect.height()) / 2.0 - 10.0;
            let stroke = Stroke::new(8.0, Color32::DARK_GRAY);

            // Background circle
            painter.circle(center, radius, Color32::TRANSPARENT, stroke);

            // Progress arc using PathShape
            if progress > 0.0 {
                let color = match timer.state() {
                    TimerState::Running { phase: Phase::Work, .. } => Color32::GREEN,
                    TimerState::Running { phase: Phase::Break, .. } => Color32::BLUE,
                    TimerState::Paused { .. } => Color32::YELLOW,
                    _ => Color32::GRAY,
                };

                let start_angle = -std::f32::consts::FRAC_PI_2; // Top
                let end_angle = start_angle + (progress * 2.0 * std::f32::consts::PI);

                // Create arc path
                let num_segments = 64;
                let arc_path: Vec<Pos2> = (0..=num_segments)
                    .map(|i| {
                        let t = i as f32 / num_segments as f32;
                        let angle = start_angle + t * (end_angle - start_angle);
                        center + vec2(angle.cos(), angle.sin()) * radius
                    })
                    .collect();

                painter.add(egui::epaint::PathShape::line(
                    arc_path,
                    Stroke::new(8.0, color),
                ));
            }

            // Center text
            let time_text = format!("{:02}:{:02}", minutes, seconds);
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                time_text,
                egui::FontId::proportional(32.0),
                Color32::WHITE,
            );

            // Phase label below
            if self.show_phase {
                let (phase_text, color) = match timer.state() {
                    TimerState::Running { phase, .. } => match phase {
                        Phase::Work => ("工作中", Color32::GREEN),
                        Phase::Break => ("休息中", Color32::BLUE),
                    },
                    TimerState::Paused { .. } => ("已暂停", Color32::YELLOW),
                    TimerState::Idle => ("准备开始", Color32::GRAY),
                    TimerState::Completed { .. } => ("已完成", Color32::LIGHT_GREEN),
                };

                let label_pos = center + vec2(0.0, radius + 20.0);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    phase_text,
                    egui::FontId::proportional(14.0),
                    color,
                );
            }
        });
    }
}

impl Default for TimerDisplay {
    fn default() -> Self {
        Self::new_circular()
    }
}
