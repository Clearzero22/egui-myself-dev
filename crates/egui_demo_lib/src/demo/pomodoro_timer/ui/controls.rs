//! Control buttons component.

use crate::demo::pomodoro_timer::core::TimerState;
use egui::{self, Ui};

/// Timer action triggered by controls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimerAction {
    Start,
    Pause,
    Resume,
    Reset,
    Skip,
    PageNext,
    PagePrev,
    ToggleAutoPage,
}

/// Controls renderer.
pub struct ControlsRenderer {
    show_labels: bool,
}

impl ControlsRenderer {
    /// Create new controls renderer.
    pub fn new() -> Self {
        Self {
            show_labels: true,
        }
    }

    /// Render control buttons.
    pub fn render(&self, ui: &mut Ui, state: &TimerState, auto_page: bool) -> Vec<TimerAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            ui.separator();
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

            // Start button
            if matches!(state, TimerState::Idle | TimerState::Completed { .. }) {
                if ui.button("▶ 开始").clicked() {
                    actions.push(TimerAction::Start);
                }
            }

            // Pause button
            if matches!(state, TimerState::Running { .. }) {
                if ui.button("⏸ 暂停").clicked() {
                    actions.push(TimerAction::Pause);
                }
            }

            // Resume button
            if matches!(state, TimerState::Paused { .. }) {
                if ui.button("▶ 继续").clicked() {
                    actions.push(TimerAction::Resume);
                }
            }

            // Reset button
            if !matches!(state, TimerState::Idle) {
                if ui.button("⟲ 重置").clicked() {
                    actions.push(TimerAction::Reset);
                }
            }

            // Auto-page toggle
            ui.separator();
            let label = if auto_page { "自动翻页: 开" } else { "自动翻页: 关" };
            if ui.button(label).clicked() {
                actions.push(TimerAction::ToggleAutoPage);
            }
        });

        // Page navigation controls
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("翻页:");
            if ui.button("◀ 上一页").clicked() {
                actions.push(TimerAction::PagePrev);
            }
            if ui.button("下一页 ▶").clicked() {
                actions.push(TimerAction::PageNext);
            }
        });

        actions
    }
}

impl Default for ControlsRenderer {
    fn default() -> Self {
        Self::new()
    }
}
