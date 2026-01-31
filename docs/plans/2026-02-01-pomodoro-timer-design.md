# Pomodoro Timer - Design Document

**Date:** 2026-02-01
**Status:** Design Approved
**Author:** AI Assistant

---

## Overview

A Pomodoro timer with automatic page-turning functionality for egui. Combines the Pomodoro technique (25 minutes work, 5 minutes break) with content pagination for focused reading sessions.

## Features

- Standard 25/5 minute Pomodoro timer (customizable)
- Visual countdown with circular progress indicator
- Automatic page-turning when timer completes
- Session history and statistics
- Multiple content source support

## Architecture

```
pomodoro_timer/
├── core/           # Timer logic, config, session records
├── content/        # Content pagination and loading
├── ui/             # UI components
└── mod.rs          # Main application
```

### Layer Responsibilities

| Layer | Responsibility | Dependencies |
|-------|---------------|--------------|
| Core  | Timer state machine, config, storage | None |
| Content | Pagination, content loading | Core |
| UI    | Rendering, user interaction | Core, Content |
| Module | State management, orchestration | All |

## Core Layer

### Timer State Machine

```rust
pub enum TimerState {
    Idle,
    Running { remaining: Duration },
    Paused { remaining: Duration },
    Break { remaining: Duration },
    Completed,
}

pub struct PomodoroTimer {
    state: TimerState,
    phase: Phase,          // Work | Break
    config: TimerConfig,
}
```

**Key Methods:**
- `start()`, `pause()`, `resume()`, `reset()`
- `tick(delta)` - Update timer, return true if completed
- `remaining()` - Get remaining time
- `progress()` - Get 0.0-1.0 progress

### Configuration

```rust
pub struct TimerConfig {
    pub work_duration: Duration,      // Default: 25 min
    pub break_duration: Duration,     // Default: 5 min
    pub long_break_duration: Duration, // Default: 15 min
    pub auto_start_break: bool,
    pub auto_start_work: bool,
}
```

### Storage

```rust
pub trait Store: Send + Sync {
    fn save_session(&mut self, session: Session) -> Result<()>;
    fn get_sessions(&self, range: DateRange) -> Vec<Session>;
    fn get_today_stats(&self) -> SessionStats;
}
```

## Content Layer

### Pagination

```rust
pub trait Pager: Send + Sync {
    fn total_pages(&self) -> usize;
    fn current_page(&self) -> usize;
    fn current_content(&self) -> Cow<str>;
    fn navigate(&mut self, direction: PageDirection) -> Result<()>;
    fn has_next(&self) -> bool;
    fn has_previous(&self) -> bool;
}

pub enum PageDirection {
    Next, Previous, First, Last, JumpTo(usize),
}
```

### Text Pager

```rust
pub struct TextPager {
    pages: Vec<String>,
    current: usize,
}

impl TextPager {
    pub fn by_chars(text: String, chars_per_page: usize) -> Self;
    pub fn by_paragraphs(text: String, paragraphs_per_page: usize) -> Self;
}
```

## UI Layer

### Components

| Component | Responsibility |
|-----------|---------------|
| `TimerDisplay` | Countdown display (digital/circular) |
| `ProgressRenderer` | Progress bar visualization |
| `ControlsRenderer` | Start/Pause/Reset buttons |
| `ContentView` | Content display with pagination |
| `StatsDisplay` | Session statistics |

### UI Actions

```rust
pub enum TimerAction {
    Start, Pause, Resume, Reset, Skip,
    PageNext, PagePrev,
    ToggleAutoPage,
}
```

## Data Flow

```
1. update_timer() - Calculate delta, tick timer
2. Check completion - Trigger auto-page if enabled
3. Render UI - Collect user actions
4. Handle actions - Update state
5. Request repaint - Loop continues
```

## Implementation Plan

### Phase 1: MVP (2-3 days)
- Basic timer (work/break)
- Digital countdown display
- Start/Pause/Reset buttons
- Simple paragraph-based pagination
- Manual page navigation

### Phase 2: Core Features (2-3 days)
- Circular progress indicator
- Automatic page-turning trigger
- Session recording
- Statistics display

### Phase 3: Enhancements (2-3 days)
- Configuration UI
- History view
- Theme switching
- Animations

## Extension Points

- Implement `Store` trait for different backends (SQLite, file)
- Implement `Pager` trait for different content types (images, PDF)
- Custom display styles and layouts

## Technical Considerations

### Timer Precision
Use `Instant::now()` for actual elapsed time, not assumed frame rate.

### Pagination Strategy
Start with paragraph-based pagination (simple, reliable).
Consider UI-height-based pagination for advanced use cases.

### State Management
Maintain single-direction data flow to avoid inconsistencies.

## Estimated Code Size

- Core layer: ~400 lines
- Content layer: ~200 lines
- UI layer: ~330 lines
- Module integration: ~200 lines

**Total: ~1,200 lines**

## References

- Project template: `crates/egui_demo_lib/src/demo/COMPONENT_TEMPLATE.md`
- Reference module: `crates/egui_demo_lib/src/demo/clipboard_history/`
- egui documentation: https://docs.rs/egui/
