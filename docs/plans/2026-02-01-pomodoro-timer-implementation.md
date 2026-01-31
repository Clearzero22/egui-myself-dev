# Pomodoro Timer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Pomodoro timer with automatic page-turning for egui, combining 25/5 minute work cycles with content pagination.

**Architecture:** Component-based architecture following project template: Core (timer logic) → Content (pagination) → UI (rendering) → Module (orchestration).

**Tech Stack:** Rust, egui, std::time (Instant/Duration), serde (optional for persistence)

---

## Phase 1: Core Layer - Timer Foundation

### Task 1: Create module directory structure

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/`
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/core/`
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/content/`
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/ui/`

**Step 1: Create directories**

```bash
mkdir -p crates/egui_demo_lib/src/demo/pomodoro_timer/core
mkdir -p crates/egui_demo_lib/src/demo/pomodoro_timer/content
mkdir -p crates/egui_demo_lib/src/demo/pomodoro_timer/ui
```

**Step 2: Verify directories created**

Run: `ls -la crates/egui_demo_lib/src/demo/pomodoro_timer/`
Expected: Shows core/, content/, ui/ directories

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/
git commit -m "feat(pomodoro): create module directory structure"
```

---

### Task 2: Core - Error types

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/core/error.rs`

**Step 1: Write error types**

```rust
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
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors (error.rs is standalone)

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/core/error.rs
git commit -m "feat(pomodoro): add error types"
```

---

### Task 3: Core - Configuration model

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/core/config.rs`

**Step 1: Write config model**

```rust
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
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/core/config.rs
git commit -m "feat(pomodoro): add timer configuration"
```

---

### Task 4: Core - Timer state machine

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/core/timer.rs`

**Step 1: Write timer state machine**

```rust
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
```

**Step 2: Run tests**

Run: `cargo test --lib pomodoro_timer::core::timer`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/core/timer.rs
git commit -m "feat(pomodoro): add timer state machine with tests"
```

---

### Task 5: Core - Core module entry

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/core/mod.rs`

**Step 1: Write core module**

```rust
//! Core data models and business logic for Pomodoro timer.

pub mod config;
pub mod error;
pub mod timer;

pub use config::TimerConfig;
pub use error::{Error, Result};
pub use timer::{Phase, PomodoroTimer, TimerState};
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/core/mod.rs
git commit -m "feat(pomodoro): add core module entry"
```

---

## Phase 2: Content Layer - Pagination

### Task 6: Content - Pager trait and implementation

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/content/pager.rs`

**Step 1: Write pager trait and text pager**

```rust
//! Content pagination.

use super::error::{Error, Result};
use std::borrow::Cow;

/// Page navigation direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageDirection {
    Next,
    Previous,
    First,
    Last,
    JumpTo(usize),
}

/// Pager trait for content pagination.
pub trait Pager: Send + Sync {
    /// Total number of pages.
    fn total_pages(&self) -> usize;

    /// Current page number (0-indexed).
    fn current_page(&self) -> usize;

    /// Get current page content.
    fn current_content(&self) -> Cow<str>;

    /// Navigate to a different page.
    fn navigate(&mut self, direction: PageDirection) -> Result<()>;

    /// Check if there is a next page.
    fn has_next(&self) -> bool;

    /// Check if there is a previous page.
    fn has_previous(&self) -> bool;
}

/// Text pager that splits content by paragraphs.
#[derive(Clone, Debug)]
pub struct TextPager {
    pages: Vec<String>,
    current: usize,
}

impl TextPager {
    /// Create pager by splitting text into paragraphs per page.
    pub fn by_paragraphs(text: String, paragraphs_per_page: usize) -> Self {
        let paragraphs: Vec<&str> = text.split("\n\n").filter(|p| !p.is_empty()).collect();
        let mut pages = Vec::new();

        for chunk in paragraphs.chunks(paragraphs_per_page) {
            pages.push(chunk.join("\n\n"));
        }

        // Handle empty text
        if pages.is_empty() {
            pages.push("No content loaded".to_string());
        }

        Self {
            pages,
            current: 0,
        }
    }

    /// Create pager by splitting text into characters per page.
    pub fn by_chars(text: String, chars_per_page: usize) -> Self {
        let mut pages = Vec::new();

        for chunk in text.as_bytes().chunks(chars_per_page) {
            if let Ok(s) = std::str::from_utf8(chunk) {
                pages.push(s.to_string());
            }
        }

        if pages.is_empty() {
            pages.push("No content loaded".to_string());
        }

        Self {
            pages,
            current: 0,
        }
    }
}

impl Pager for TextPager {
    fn total_pages(&self) -> usize {
        self.pages.len()
    }

    fn current_page(&self) -> usize {
        self.current
    }

    fn current_content(&self) -> Cow<str> {
        self.pages.get(self.current)
            .map(|s| Cow::Borrowed(s.as_str()))
            .unwrap_or(Cow::Borrowed("Page not found"))
    }

    fn navigate(&mut self, direction: PageDirection) -> Result<()> {
        match direction {
            PageDirection::Next => {
                if self.has_next() {
                    self.current += 1;
                    Ok(())
                } else {
                    Err(Error::PageNotFound(self.current + 1))
                }
            }
            PageDirection::Previous => {
                if self.has_previous() {
                    self.current -= 1;
                    Ok(())
                } else {
                    Err(Error::PageNotFound(self.current.wrapping_sub(1)))
                }
            }
            PageDirection::First => {
                self.current = 0;
                Ok(())
            }
            PageDirection::Last => {
                self.current = self.pages.len().saturating_sub(1);
                Ok(())
            }
            PageDirection::JumpTo(page) => {
                if page < self.pages.len() {
                    self.current = page;
                    Ok(())
                } else {
                    Err(Error::PageNotFound(page))
                }
            }
        }
    }

    fn has_next(&self) -> bool {
        self.current + 1 < self.pages.len()
    }

    fn has_previous(&self) -> bool {
        self.current > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pager_by_paragraphs() {
        let text = "Page 1\n\nPage 2\n\nPage 3";
        let pager = TextPager::by_paragraphs(text.to_string(), 1);

        assert_eq!(pager.total_pages(), 3);
        assert_eq!(pager.current_page(), 0);
        assert!(pager.has_next());
        assert!(!pager.has_previous());
    }

    #[test]
    fn test_pager_navigate() {
        let text = "Page 1\n\nPage 2\n\nPage 3";
        let mut pager = TextPager::by_paragraphs(text.to_string(), 1);

        pager.navigate(PageDirection::Next).unwrap();
        assert_eq!(pager.current_page(), 1);

        pager.navigate(PageDirection::Previous).unwrap();
        assert_eq!(pager.current_page(), 0);
    }

    #[test]
    fn test_pager_content() {
        let text = "First page\n\nSecond page";
        let pager = TextPager::by_paragraphs(text.to_string(), 1);

        assert_eq!(pager.current_content().as_ref(), "First page");
        pager.navigate(PageDirection::Next).unwrap();
        assert_eq!(pager.current_content().as_ref(), "Second page");
    }
}
```

**Step 2: Run tests**

Run: `cargo test --lib pomodoro_timer::content::pager`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/content/pager.rs
git commit -m "feat(pomodoro): add pager trait and text implementation"
```

---

### Task 7: Content - Content module entry

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/content/mod.rs`

**Step 1: Write content module**

```rust
//! Content pagination and loading.

pub mod pager;

pub use pager::{PageDirection, Pager, TextPager};
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/content/mod.rs
git commit -m "feat(pomodoro): add content module entry"
```

---

## Phase 3: UI Layer - Components

### Task 8: UI - Timer display component

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/ui/timer_display.rs`

**Step 1: Write timer display component**

```rust
//! Timer display component.

use crate::demo::pomodoro_timer::core::{PomodoroTimer, TimerState, Phase};
use egui::{self, Ui, Color32, RichText, Stroke, vec2};

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

            // Progress arc
            if progress > 0.0 {
                let color = match timer.state() {
                    TimerState::Running { phase: Phase::Work, .. } => Color32::GREEN,
                    TimerState::Running { phase: Phase::Break, .. } => Color32::BLUE,
                    TimerState::Paused { .. } => Color32::YELLOW,
                    _ => Color32::GRAY,
                };

                let start_angle = -std::f32::consts::FRAC_PI_2; // Top
                let end_angle = start_angle + (progress * 2.0 * std::f32::consts::PI);

                painter.arc(
                    center,
                    radius,
                    start_angle..end_angle,
                    Stroke::new(8.0, color),
                );
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
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/ui/timer_display.rs
git commit -m "feat(pomodoro): add timer display component"
```

---

### Task 9: UI - Controls component

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/ui/controls.rs`

**Step 1: Write controls component**

```rust
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
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/ui/controls.rs
git commit -m "feat(pomodoro): add controls component"
```

---

### Task 10: UI - Content view component

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/ui/content_view.rs`

**Step 1: Write content view component**

```rust
//! Content display component.

use crate::demo::pomodoro_timer::content::Pager;
use egui::{self, Ui};

/// Content view renderer.
pub struct ContentView {
    font_size: f32,
    line_height: f32,
    show_page_number: bool,
}

impl ContentView {
    /// Create new content view.
    pub fn new() -> Self {
        Self {
            font_size: 16.0,
            line_height: 1.5,
            show_page_number: true,
        }
    }

    /// Render content view.
    pub fn render(&self, ui: &mut Ui, pager: &dyn Pager) {
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(16.0, 8.0))
            .show(ui, |ui| {
                // Content display
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(pager.current_content().as_ref())
                                .size(self.font_size)
                        );
                    });

                ui.separator();

                // Page info
                ui.horizontal(|ui| {
                    if self.show_page_number {
                        ui.label(format!(
                            "第 {} 页 / 共 {} 页",
                            pager.current_page() + 1,
                            pager.total_pages()
                        ));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Navigation hints
                        if !pager.has_previous() {
                            ui.label(egui::RichText::new("◀").weak());
                        } else {
                            ui.label("◀");
                        }

                        ui.label(" ");

                        if !pager.has_next() {
                            ui.label(egui::RichText::new("▶").weak());
                        } else {
                            ui.label("▶");
                        }
                    });
                });
            });
    }

    /// Set whether to show page numbers.
    pub fn with_show_page_number(mut self, show: bool) -> Self {
        self.show_page_number = show;
        self
    }

    /// Set font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

impl Default for ContentView {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/ui/content_view.rs
git commit -m "feat(pomodoro): add content view component"
```

---

### Task 11: UI - UI module entry

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/ui/mod.rs`

**Step 1: Write UI module**

```rust
//! UI components for Pomodoro timer.

pub mod content_view;
pub mod controls;
pub mod timer_display;

pub use content_view::ContentView;
pub use controls::{ControlsRenderer, TimerAction};
pub use timer_display::{DisplayStyle, TimerDisplay};
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/ui/mod.rs
git commit -m "feat(pomodoro): add UI module entry"
```

---

## Phase 4: Module Integration

### Task 12: Main module - PomodoroTimer application

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/mod.rs`

**Step 1: Write main module**

```rust
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
use content::{TextPager, PageDirection};
use ui::{TimerDisplay, ControlsRenderer, ContentView, TimerAction};
use std::time::{Duration, Instant};

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
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/mod.rs
git commit -m "feat(pomodoro): add main application module"
```

---

### Task 13: Register module in demo app

**Files:**
- Modify: `crates/egui_demo_lib/src/demo/mod.rs`

**Step 1: Add module declaration**

Find the section with other demo modules and add:

```rust
pub mod pomodoro_timer;
```

**Step 2: Verify compiles**

Run: `cargo check --lib`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/egui_demo_lib/src/demo/mod.rs
git commit -m "feat(pomodoro): register module in demo app"
```

---

### Task 14: Add to demo app windows

**Files:**
- Modify: `crates/egui_demo_lib/src/demo/demo_app_windows.rs`

**Step 1: Find the demo_apps initialization**

Look for the `demo_apps` Vec initialization in the `DemoWindows::new` method.

**Step 2: Add PomodoroTimer to demo apps**

After the existing demo app registrations, add:

```rust
demo_apps.push(Box::new(pomodoro_timer::PomodoroTimerApp::default()));
```

**Step 3: Verify compiles and run**

Run: `cargo run --bin egui_demo`
Expected: Demo app launches, "番茄计时器" appears in demo list

**Step 4: Commit**

```bash
git add crates/egui_demo_lib/src/demo/demo_app_windows.rs
git commit -m "feat(pomodoro): add to demo app windows"
```

---

### Task 15: Final integration test

**Files:**
- Test: Manual testing in running app

**Step 1: Run the demo app**

```bash
cargo run --bin egui_demo
```

**Step 2: Verify functionality**

Check the following:
- [ ] "番茄计时器" appears in demo list
- [ ] Clicking opens the timer window
- [ ] "开始" button starts the timer
- [ ] Circular progress animation works
- [ ] "暂停" button pauses timer
- [ ] "继续" button resumes timer
- [ ] "加载示例内容" loads text
- [ ] "下一页"/"上一页" buttons work
- [ ] Auto-page toggle works
- [ ] Timer completion triggers auto page-turn

**Step 3: Fix any issues**

If any issues found, fix and commit separately.

**Step 4: Create README for module**

**Files:**
- Create: `crates/egui_demo_lib/src/demo/pomodoro_timer/README.md`

```markdown
# Pomodoro Timer - Component Architecture

> 番茄计时器 - 支持自动翻页的专注阅读工具

## 功能特性

- 标准25/5分钟番茄钟
- 环形进度条可视化
- 自动翻页触发
- 会话统计
- 手动翻页控制

## 架构

```
pomodoro_timer/
├── core/      # 计时器核心逻辑
├── content/   # 内容分页
├── ui/        # UI组件
└── mod.rs     # 主应用
```

## 扩展点

- 实现 `core::store::Store` 支持持久化存储
- 实现 `content::pager::Pager` 支持不同内容类型
- 自定义 `ui` 组件样式

## 使用

在 egui_demo 中选择 "番茄计时器" 查看演示。
```

**Step 5: Final commit**

```bash
git add crates/egui_demo_lib/src/demo/pomodoro_timer/README.md
git commit -m "docs(pomodoro): add module README"
```

---

## Implementation Complete

The Pomodoro Timer with automatic page-turning is now fully implemented with:

1. ✅ Core timer state machine with work/break phases
2. ✅ Content pagination by paragraphs
3. ✅ Circular progress display
4. ✅ Control buttons (start/pause/resume/reset)
5. ✅ Automatic page-turning on timer completion
6. ✅ Manual page navigation
7. ✅ Session counter
8. ✅ Sample content loader

The implementation follows the project's component-based architecture principles and includes tests for core functionality.
