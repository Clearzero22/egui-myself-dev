//! Pomodoro Timer - Automatic Page-Turning Edition
//!
//! Combines Pomodoro technique with content pagination for focused reading.
//!
//! # Features
//!
//! - Standard 25/5 minute Pomodoro timer (customizable)
//! - Visual countdown with circular progress indicator
//! - Automatic page-turning when timer completes
//! - Manual page navigation controls
//!
//! # Architecture
//!
//! - [`core`]: Timer logic, configuration
//! - [`content`]: Content pagination
//! - [`ui`]: UI components

pub mod content;
pub mod core;
pub mod ui;

use core::{PomodoroTimer, TimerConfig};
use content::{TextPager, PageDirection, pager::Pager};
use ui::{TimerDisplay, ControlsRenderer, ContentView, TimerAction};
use std::time::Instant;

/// Pomodoro Timer application.
pub struct PomodoroTimerApp {
    // Core layer
    timer: PomodoroTimer,
    config: TimerConfig,

    // Content layer
    pager: Option<TextPager>,
    auto_page_enabled: bool,

    // UI layer
    display: TimerDisplay,
    controls: ControlsRenderer,
    content_view: ContentView,

    // Application state
    pending_actions: Vec<TimerAction>,
    last_update: Option<Instant>,
}

impl PomodoroTimerApp {
    /// Create new application.
    pub fn new() -> Self {
        Self {
            timer: PomodoroTimer::new(TimerConfig::default()),
            config: TimerConfig::default(),
            pager: None,
            auto_page_enabled: true,
            display: TimerDisplay::new_circular(),
            controls: ControlsRenderer::default(),
            content_view: ContentView::default(),
            pending_actions: Vec::new(),
            last_update: None,
        }
    }

    /// Load text content for pagination.
    pub fn load_text(&mut self, text: String, paragraphs_per_page: usize) {
        self.pager = Some(TextPager::by_paragraphs(text, paragraphs_per_page));
    }

    /// Load sample content.
    pub fn load_sample_content(&mut self) {
        let sample = r#"第一章：绪论

这是第一页的内容。番茄工作法是一种时间管理方法，由Francesco Cirillo在1980年代创立。

该方法使用定时器将工作分解为25分钟的时间间隔，称为"番茄时间"，间隔之间有短暂的休息。

第二章：基本原理

番茄工作法的基本原理非常简单。选择一个任务，设置定时器为25分钟，开始工作，直到定时器响起。

当你完成一个番茄时间后，在纸上做一个标记，然后进行短暂的休息，通常是5分钟。

第三章：进阶技巧

完成四个番茄时间后，进行一次较长的休息，通常是15到30分钟。

这有助于保持大脑清醒，并防止疲劳积累。记住，番茄时间不可分割，必须完整完成。

第四章：常见问题

如果在一个番茄时间内被打断怎么办？根据打断的性质，你可能需要暂停当前的番茄时间。

如果是内部打断（比如想起其他事情），记录下来然后继续。如果是外部打断，可能需要重新开始。

第五章：总结

番茄工作法的核心是专注和休息的平衡。通过规律的工作和休息节奏，可以提高生产力。

现在你已经了解了番茄工作法的基本原理，是时候开始实践了！"#;

        self.load_text(sample.to_string(), 1);
    }

    /// Update timer state.
    fn update_timer(&mut self) {
        if let Some(last) = self.last_update {
            let delta = last.elapsed();
            let completed = self.timer.tick(delta);

            if completed {
                self.on_timer_complete();

                // Auto page-turn
                if self.auto_page_enabled {
                    if let Some(ref mut pager) = self.pager {
                        let _ = pager.navigate(PageDirection::Next);
                    }
                }
            }
        }
        self.last_update = Some(Instant::now());
    }

    /// Handle timer completion.
    fn on_timer_complete(&mut self) {
        // Could add sound notification, visual feedback, etc.
    }

    /// Handle pending actions.
    fn handle_actions(&mut self) {
        for action in self.pending_actions.drain(..) {
            match action {
                TimerAction::Start => {
                    let _ = self.timer.start();
                }
                TimerAction::Pause => {
                    let _ = self.timer.pause();
                }
                TimerAction::Resume => {
                    let _ = self.timer.resume();
                }
                TimerAction::Reset => {
                    self.timer.reset();
                }
                TimerAction::PageNext => {
                    if let Some(ref mut pager) = self.pager {
                        let _ = pager.navigate(PageDirection::Next);
                    }
                }
                TimerAction::PagePrev => {
                    if let Some(ref mut pager) = self.pager {
                        let _ = pager.navigate(PageDirection::Previous);
                    }
                }
                TimerAction::ToggleAutoPage => {
                    self.auto_page_enabled = !self.auto_page_enabled;
                }
                TimerAction::Skip => {
                    // Skip to next phase
                }
            }
        }
    }
}

impl Default for PomodoroTimerApp {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Demo trait implementation
// -----------------------------------------------------------------------------

impl crate::Demo for PomodoroTimerApp {
    fn name(&self) -> &'static str {
        "番茄计时器"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(true)
            .default_size([500.0, 600.0])
            .show(ctx, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}

// -----------------------------------------------------------------------------
// View trait implementation
// -----------------------------------------------------------------------------

impl crate::View for PomodoroTimerApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Update timer
        self.update_timer();

        // Request continuous repaint
        ui.ctx().request_repaint();

        // Main layout
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);

                // Timer display
                self.display.render(ui, &self.timer);

                ui.add_space(16.0);

                // Controls
                let actions = self.controls.render(
                    ui,
                    self.timer.state(),
                    self.auto_page_enabled
                );
                self.pending_actions.extend(actions);

                ui.add_space(16.0);

                // Session info
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "已完成: {} 个番茄",
                        self.timer.sessions_completed()
                    ));
                });

                // Content view (if loaded)
                if let Some(ref pager) = self.pager {
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("阅读内容:");
                    self.content_view.render(ui, pager);
                } else {
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("点击下方按钮加载示例内容").weak());
                }

                ui.add_space(8.0);

                if ui.button("加载示例内容").clicked() {
                    self.load_sample_content();
                }
            });
        });

        // Handle actions
        self.handle_actions();
    }
}
