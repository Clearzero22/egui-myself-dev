# Clipboard History Refactoring Plan

> **Version**: 1.0
> **Date**: 2025-01-30
> **Status**: Ready for Implementation

---

## 1. Overview

### 1.1 Refactoring Goals

1. **Modularity**: Split 510-line monolithic file into well-organized modules
2. **Extensibility**: Design traits for future feature additions
3. **Testability**: Isolate components for unit testing
4. **Maintainability**: Clear separation of concerns
5. **SOLID Principles**: Apply SRP, OCP, DIP throughout

### 1.2 Current State Analysis

**File**: `crates/egui_demo_lib/src/demo/clipboard_history.rs` (510 lines)

| Component | Lines | Responsibility | Issues |
|-----------|-------|----------------|--------|
| `ClipboardHistory` struct | ~20 | State container | ❌ Too many fields mixed |
| `ClipboardItem` struct | ~10 | Data model | ❌ No behavior encapsulation |
| `FilterType` enum | ~10 | Type filter | ❌ Not extensible |
| Demo/View impl | ~180 | UI rendering | ❌ UI logic mixed with business logic |
| Helper methods | ~150 | Clipboard operations | ❌ Hardcoded arboard |
| Static function | ~60 | Item card rendering | ❌ Workaround for borrow checker |

**Problems Identified**:
- Single Responsibility Principle violations
- No abstraction for clipboard backend
- No abstraction for storage
- UI code intertwined with business logic
- Difficult to test individual components
- Hard to extend with new features

### 1.3 Target Architecture

```
demo/clipboard_history/
├── mod.rs                    # Public API + Demo trait impl (~80 lines)
├── core/
│   ├── mod.rs               # Core exports (~30 lines)
│   ├── item.rs              # ClipboardItem + ContentType (~120 lines)
│   ├── store.rs             # Store trait + MemoryStore (~150 lines)
│   └── filter.rs            # Filter trait + implementations (~100 lines)
├── clipboard/
│   ├── mod.rs               # Clipboard exports (~20 lines)
│   ├── backend.rs           # Backend trait + types (~80 lines)
│   └── arboard.rs           # arboard implementation (~100 lines)
└── ui/
    ├── mod.rs               # UI exports (~20 lines)
    ├── view.rs              # View trait impl (~150 lines)
    ├── dialog.rs            # Dialog components (~80 lines)
    └── card.rs              # Item card rendering (~60 lines)
```

**Total**: ~990 lines (vs 510) but with:
- Clear separation of concerns
- Extensible trait-based design
- Testable components
- Documentation for each module

---

## 2. Detailed Module Design

### 2.1 `core/item.rs` - Data Model

**Purpose**: Encapsulate clipboard item data and content type detection

**Exports**:
```rust
pub use self::content_type::ContentType;
pub use self::item::ClipboardItem;
```

**Dependencies**: `std` only

---

#### `content_type.rs` (within `item.rs`)

```rust
//! Content type detection and classification
//!
//! Extension points:
//! - Custom content types via ContentType::Custom
//! - Language detection for code
//! - File type detection

use std::path::Path;

/// Represents the type of clipboard content
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ContentType {
    Text,
    Url,
    Image,
    Code { language: String },
    File { extension: String },
    Custom(String),
}

impl ContentType {
    /// Detect content type from raw string
    pub fn detect(content: &str) -> Self {
        let trimmed = content.trim();

        // URL detection
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return Self::Url;
        }

        // File path detection
        if trimmed.starts_with('/') || trimmed.starts_with("~/") || trimmed[1..].starts_with(":\\") {
            if let Some(ext) = Path::new(trimmed).extension() {
                return Self::File {
                    extension: ext.to_string_lossy().to_string(),
                };
            }
        }

        // Code detection (heuristic)
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() > 1 {
            let first_line = lines[0].trim();
            // Common code patterns
            if first_line.starts_with("fn ") || first_line.starts_with("def ")
                || first_line.starts_with("function ") || first_line.starts_with("class ")
                || first_line.starts_with("import ") || first_line.starts_with("use ")
                || first_line.starts_with("pub ") || first_line.starts_with("let ")
                || trimmed.contains(" => ") || trimmed.contains("{") {
                return Self::Code {
                    language: "unknown".to_string(),
                };
            }
        }

        Self::Text
    }

    /// Get display icon for this content type
    pub fn icon(&self) -> &str {
        match self {
            Self::Text => "📄",
            Self::Url => "🔗",
            Self::Image => "🖼️",
            Self::Code { .. } => "💻",
            Self::File { .. } => "📁",
            Self::Custom(_) => "📦",
        }
    }

    /// Get display label for this content type
    pub fn label(&self) -> &str {
        match self {
            Self::Text => "Text",
            Self::Url => "Link",
            Self::Image => "Image",
            Self::Code { .. } => "Code",
            Self::File { .. } => "File",
            Self::Custom(_) => "Other",
        }
    }

    /// Check if this is an image type
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image)
    }

    /// Check if this is a text-based type
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text | Self::Url | Self::Code { .. })
    }
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Text
    }
}

// Convert from legacy FilterType
impl From<crate::filter::FilterType> for ContentType {
    fn from(value: crate::filter::FilterType) -> Self {
        match value {
            crate::filter::FilterType::All => Self::Text,
            crate::filter::FilterType::Text => Self::Text,
            crate::filter::FilterType::Url => Self::Url,
            crate::filter::FilterType::Image => Self::Image,
        }
    }
}
```

---

#### `item.rs` (main part)

```rust
//! Clipboard item data model

use super::content_type::ContentType;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single clipboard history item
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ClipboardItem {
    /// Display icon (emoji)
    pub icon: String,
    /// Display title (preview)
    pub title: String,
    /// Full content
    pub content: String,
    /// Timestamp string (HH:MM format)
    pub timestamp: String,
    /// Content type classification
    pub content_type: ContentType,
}

impl ClipboardItem {
    /// Create a new ClipboardItem from text content
    pub fn from_text(content: String) -> Self {
        let content_type = ContentType::detect(&content);
        let icon = content_type.icon().to_string();
        let title = Self::generate_title(&content);
        let timestamp = Self::current_timestamp();

        Self {
            icon,
            title,
            content,
            timestamp,
            content_type,
        }
    }

    /// Create a new ClipboardItem from image data
    pub fn from_image(width: u32, height: u32) -> Self {
        Self {
            icon: "🖼️".to_string(),
            title: format!("Screenshot ({}x{})", width, height),
            content: format!("[Image: {}x{} pixels]", width, height),
            timestamp: Self::current_timestamp(),
            content_type: ContentType::Image,
        }
    }

    /// Create a ClipboardItem with custom content type
    pub fn with_type(content: String, content_type: ContentType) -> Self {
        let icon = content_type.icon().to_string();
        let title = Self::generate_title(&content);

        Self {
            icon,
            title,
            content,
            timestamp: Self::current_timestamp(),
            content_type,
        }
    }

    /// Get a preview of the content (truncated)
    pub fn preview(&self, max_len: usize) -> String {
        if self.content.len() > max_len {
            format!("{}...", &self.content[..max_len.saturating_sub(3)])
        } else {
            self.content.clone()
        }
    }

    /// Check if item matches search query
    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let query_lower = query.to_lowercase();
        self.title.to_lowercase().contains(&query_lower)
            || self.content.to_lowercase().contains(&query_lower)
    }

    // --- Private helpers ---

    fn generate_title(content: &str) -> String {
        let preview = content.lines().next().unwrap_or(content);
        if preview.len() > 40 {
            format!("{}...", &preview[..37])
        } else {
            preview.to_string()
        }
    }

    fn current_timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!(
            "{:02}:{:02}",
            (now % 86400 / 3600) as u32,
            (now % 3600 / 60) as u32
        )
    }
}

impl Default for ClipboardItem {
    fn default() -> Self {
        Self::from_text("Empty".to_string())
    }
}
```

---

### 2.2 `core/store.rs` - Storage Abstraction

**Purpose**: Abstract storage operations for extensibility

**Exports**:
```rust
pub use self::memory::MemoryStore;
pub use self::traits::{Store, Error, Result};
```

**Dependencies**: `std`, `crate::core::item`

---

#### `traits.rs` (within `store.rs`)

```rust
//! Storage abstraction traits

use crate::core::item::ClipboardItem;

/// Error type for store operations
#[derive(Debug)]
pub enum Error {
    NotFound(usize),
    CapacityExceeded,
    Io(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(idx) => write!(f, "Item not found: {}", idx),
            Self::CapacityExceeded => write!(f, "Storage capacity exceeded"),
            Self::Io(msg) => write!(f, "IO error: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for store operations
pub type Result<T> = std::result::Result<T, Error>;

/// Storage abstraction for clipboard items
///
/// Extension points:
/// - Database persistence (SQLite, sled)
/// - File-based storage (JSON, bincode)
/// - Cloud synchronization
pub trait Store: Send + Sync {
    /// Add a new item to the store
    fn add(&mut self, item: ClipboardItem) -> Result<()>;

    /// Remove item at index
    fn remove(&mut self, index: usize) -> Result<()>;

    /// Get all items
    fn get_all(&self) -> Vec<ClipboardItem>;

    /// Get item at index (returns copy)
    fn get(&self, index: usize) -> Option<ClipboardItem>;

    /// Update item at index
    fn update(&mut self, index: usize, item: ClipboardItem) -> Result<()>;

    /// Clear all items
    fn clear(&mut self) -> Result<()>;

    /// Get number of items
    fn len(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool;

    /// Get maximum capacity
    fn max_capacity(&self) -> Option<usize> {
        None
    }
}
```

---

#### `memory.rs` (within `store.rs`)

```rust
//! In-memory storage implementation

use super::traits::{Store, Error, Result};
use crate::core::item::ClipboardItem;

/// In-memory clipboard item storage
pub struct MemoryStore {
    items: Vec<ClipboardItem>,
    max_items: usize,
}

impl MemoryStore {
    /// Create a new MemoryStore with default capacity (50)
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_items: 50,
        }
    }

    /// Create a MemoryStore with custom capacity
    pub fn with_capacity(max_items: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_items),
            max_items,
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for MemoryStore {
    fn add(&mut self, item: ClipboardItem) -> Result<()> {
        self.items.insert(0, item);
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }
        Ok(())
    }

    fn remove(&mut self, index: usize) -> Result<()> {
        if index >= self.items.len() {
            return Err(Error::NotFound(index));
        }
        self.items.remove(index);
        Ok(())
    }

    fn get_all(&self) -> Vec<ClipboardItem> {
        self.items.clone()
    }

    fn get(&self, index: usize) -> Option<ClipboardItem> {
        self.items.get(index).cloned()
    }

    fn update(&mut self, index: usize, item: ClipboardItem) -> Result<()> {
        if index >= self.items.len() {
            return Err(Error::NotFound(index));
        }
        self.items[index] = item;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.items.clear();
        Ok(())
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn max_capacity(&self) -> Option<usize> {
        Some(self.max_items)
    }
}
```

---

### 2.3 `core/filter.rs` - Filtering System

**Purpose**: Extensible filtering and search

**Exports**:
```rust
pub use self::traits::{Filter, FilterBox};
pub use self::types::{TypeFilter, TextSearchFilter, CompositeFilter, LogicOperator};
```

**Dependencies**: `std`, `crate::core::item`

---

#### `traits.rs` (within `filter.rs`)

```rust
//! Filter abstraction traits

use crate::core::item::ClipboardItem;

/// Filter trait for clipboard items
///
/// Extension points:
/// - Regex search
/// - Fuzzy search
/// - Date range filtering
/// - Tag-based filtering
pub trait Filter: Send + Sync {
    /// Test if item matches this filter
    fn matches(&self, item: &ClipboardItem) -> bool;

    /// Get description for UI display
    fn description(&self) -> String;

    /// Clone as boxed trait object
    fn clone_box(&self) -> FilterBox;
}

/// Boxed filter trait object
pub type FilterBox = Box<dyn Filter>;

impl Clone for FilterBox {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Implement for common types
impl<F: Filter + Clone + 'static> Filter for F {
    fn matches(&self, item: &ClipboardItem) -> bool {
        (**self).matches(item)
    }

    fn description(&self) -> String {
        (**self).description()
    }

    fn clone_box(&self) -> FilterBox {
        Box::new(self.clone())
    }
}
```

---

#### `types.rs` (within `filter.rs`)

```rust
//! Built-in filter implementations

use super::traits::Filter;
use crate::core::item::ClipboardItem;
use crate::core::item::content_type::ContentType;

// -----------------------------------------------------------------------------
// Text Search Filter
// -----------------------------------------------------------------------------

/// Filter by text search in title and content
#[derive(Clone, Debug)]
pub struct TextSearchFilter {
    query: String,
    case_sensitive: bool,
}

impl TextSearchFilter {
    pub fn new(query: String) -> Self {
        Self {
            query,
            case_sensitive: false,
        }
    }

    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }
}

impl Filter for TextSearchFilter {
    fn matches(&self, item: &ClipboardItem) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let search_in = || {
            let title = if self.case_sensitive {
                item.title.clone()
            } else {
                item.title.to_lowercase()
            };
            let content = if self.case_sensitive {
                item.content.clone()
            } else {
                item.content.to_lowercase()
            };
            let query = if self.case_sensitive {
                self.query.clone()
            } else {
                self.query.to_lowercase()
            };
            (title, content, query)
        };

        let (title, content, query) = search_in();
        title.contains(&query) || content.contains(&query)
    }

    fn description(&self) -> String {
        if self.query.is_empty() {
            "All items".to_string()
        } else {
            format!("Search: \"{}\"", self.query)
        }
    }
}

// -----------------------------------------------------------------------------
// Type Filter
// -----------------------------------------------------------------------------

/// Filter by content type
#[derive(Clone, Debug)]
pub struct TypeFilter {
    allowed_types: Vec<ContentType>,
}

impl TypeFilter {
    pub fn new() -> Self {
        Self {
            allowed_types: vec![ContentType::Text],
        }
    }

    pub fn with_types(types: Vec<ContentType>) -> Self {
        Self { allowed_types: types }
    }

    pub fn all() -> Self {
        Self {
            allowed_types: vec![
                ContentType::Text,
                ContentType::Url,
                ContentType::Image,
                ContentType::Code { language: "any".to_string() },
                ContentType::File { extension: "any".to_string() },
            ],
        }
    }
}

impl Default for TypeFilter {
    fn default() -> Self {
        Self::all()
    }
}

impl Filter for TypeFilter {
    fn matches(&self, item: &ClipboardItem) -> bool {
        if self.allowed_types.is_empty() {
            return true;
        }

        self.allowed_types.iter().any(|allowed| {
            match (&item.content_type, allowed) {
                (ContentType::Text, ContentType::Text) => true,
                (ContentType::Url, ContentType::Url) => true,
                (ContentType::Image, ContentType::Image) => true,
                (ContentType::Code { .. }, ContentType::Code { .. }) => true,
                (ContentType::File { .. }, ContentType::File { .. }) => true,
                _ => false,
            }
        })
    }

    fn description(&self) -> String {
        if self.allowed_types.is_empty() {
            "All types".to_string()
        } else {
            format!("Types: {}", self.allowed_types.len())
        }
    }
}

// -----------------------------------------------------------------------------
// Composite Filter
// -----------------------------------------------------------------------------

/// Logical operator for composite filters
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicOperator {
    And,
    Or,
}

/// Combine multiple filters with AND/OR logic
#[derive(Clone, Debug)]
pub struct CompositeFilter {
    filters: Vec<FilterBox>,
    operator: LogicOperator,
}

impl CompositeFilter {
    pub fn and(filters: Vec<FilterBox>) -> Self {
        Self {
            filters,
            operator: LogicOperator::And,
        }
    }

    pub fn or(filters: Vec<FilterBox>) -> Self {
        Self {
            filters,
            operator: LogicOperator::Or,
        }
    }
}

impl Filter for CompositeFilter {
    fn matches(&self, item: &ClipboardItem) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        match self.operator {
            LogicOperator::And => self.filters.iter().all(|f| f.matches(item)),
            LogicOperator::Or => self.filters.iter().any(|f| f.matches(item)),
        }
    }

    fn description(&self) -> String {
        let op = if self.operator == LogicOperator::And { "AND" } else { "OR" };
        format!("Composite ({}: {} filters)", op, self.filters.len())
    }
}
```

---

### 2.4 `clipboard/` - Clipboard Integration

#### `backend.rs`

```rust
//! Clipboard backend abstraction

use std::sync::mpsc;

/// Clipboard content representation
#[derive(Clone, Debug)]
pub enum ClipboardContent {
    Text(String),
    Image { width: u32, height: u32 },
    Files(Vec<String>),
    Unknown,
}

/// Clipboard backend trait
///
/// Extension points:
/// - Platform-specific implementations (Wayland, X11, macOS, Windows)
/// - Custom clipboard managers integration
pub trait Backend: Send + Sync {
    /// Get current clipboard content
    fn get_content(&self) -> ClipboardContent;

    /// Set text content
    fn set_text(&self, text: &str) -> bool;

    /// Subscribe to clipboard changes (optional, event-driven)
    fn subscribe(&self) -> Option<mpsc::Receiver<ClipboardContent>> {
        None
    }

    /// Check if backend supports images
    fn supports_images(&self) -> bool {
        false
    }

    /// Backend name for debugging
    fn name(&self) -> &str {
        "unknown"
    }
}
```

#### `arboard.rs`

```rust
//! arboard clipboard backend implementation

use super::backend::{Backend, ClipboardContent};
use std::sync::{Arc, Mutex};

pub struct ArboardBackend {
    last_content: Arc<Mutex<String>>,
}

impl ArboardBackend {
    pub fn new() -> Self {
        Self {
            last_content: Arc::default(),
        }
    }
}

impl Default for ArboardBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for ArboardBackend {
    fn get_content(&self) -> ClipboardContent {
        // Try text first
        if let Ok(clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    return ClipboardContent::Text(text);
                }
            }

            // Try image
            if let Ok(image) = clipboard.get_image() {
                return ClipboardContent::Image {
                    width: image.width as u32,
                    height: image.height as u32,
                };
            }
        }

        ClipboardContent::Unknown
    }

    fn set_text(&self, text: &str) -> bool {
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(text))
            .is_ok()
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "arboard"
    }
}

// Helper methods for deduplication
impl ArboardBackend {
    pub fn is_new_content(&self, content: &str) -> bool {
        let last = self.last_content.lock().unwrap();
        content != *last && !content.is_empty()
    }

    pub fn mark_content(&self, content: String) {
        *self.last_content.lock().unwrap() = content;
    }

    pub fn is_new_image(&self, width: u32, height: u32) -> bool {
        let key = format!("IMG_{}x{}", width, height);
        self.is_new_content(&key)
    }
}
```

---

### 2.5 `ui/` - User Interface Components

#### `view.rs`

```rust
//! Main View trait implementation

use super::dialog::DialogManager;
use super::card::ItemCardRenderer;
use crate::core::{Store, ClipboardHistory};
use crate::clipboard::backend::Backend;

impl crate::View for ClipboardHistory {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.heading("📋 Clipboard History");
        ui.label("Real-time clipboard monitoring - copy anything to see it here!");
        ui.separator();

        // Action bar
        self.render_action_bar(ui);

        // Auto-capture
        if self.auto_capture {
            self.auto_capture_clipboard();
        }

        // Dialogs
        self.dialogs.render(ui, self);

        // Search and filter
        self.render_search_bar(ui);

        // Item list
        self.render_item_list(ui);

        // Status bar
        self.render_status_bar(ui);
    }
}

impl ClipboardHistory {
    fn render_action_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Auto-capture:");
            ui.checkbox(&mut self.auto_capture, "Enabled");

            ui.separator();

            if ui.button("➕ Add Content").clicked() {
                self.dialogs.show_add = true;
                if let Some(content) = self.clipboard.get_text_content() {
                    self.new_item_content = content;
                }
            }

            if ui.button("📋 Paste Current").clicked() {
                self.paste_from_clipboard();
            }
        });
    }

    fn render_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("Search content..."));

            ui.separator();

            ui.label("Filter:");
            ui.radio_value(&mut self.filter_mode, FilterMode::All, "All");
            ui.radio_value(&mut self.filter_mode, FilterMode::Text, "Text");
            ui.radio_value(&mut self.filter_mode, FilterMode::Url, "URLs");
            ui.radio_value(&mut self.filter_mode, FilterMode::Image, "Images");
        });
        ui.separator();
    }

    fn render_item_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .max_height(400.0)
            .show(ui, |ui| {
                let items = self.store.get_all();
                let filtered = self.apply_filters(&items);

                let card_renderer = ItemCardRenderer::new();
                for (idx, item) in filtered.into_iter().enumerate() {
                    card_renderer.render(ui, &item, idx, &mut self.pending_actions);
                }

                // Execute pending actions
                self.execute_actions();
            });
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            let count = self.store.len();
            ui.label(format!("Showing {} items", count));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑️ Clear All").clicked() {
                    let _ = self.store.clear();
                }
                if ui.button("📋 Copy Latest").clicked() {
                    if let Some(item) = self.store.get(0) {
                        self.clipboard.set_text(&item.content);
                    }
                }
            });
        });
    }
}
```

---

## 3. Implementation Steps

### Step 1: Create directory structure
```bash
mkdir -p crates/egui_demo_lib/src/demo/clipboard_history/{core,clipboard,ui}
```

### Step 2: Implement core/item.rs
- Create `content_type` module
- Create `ClipboardItem` struct
- Add tests

### Step 3: Implement core/store.rs
- Create `traits` module
- Create `MemoryStore` implementation
- Add tests

### Step 4: Implement core/filter.rs
- Create `traits` module
- Create filter implementations
- Add tests

### Step 5: Implement clipboard/
- Create `backend` trait
- Create `arboard` implementation

### Step 6: Implement ui/
- Create dialog components
- Create card renderer
- Create view implementation

### Step 7: Update mod.rs
- Re-export public API
- Implement Demo trait
- Wire everything together

### Step 8: Update demo/mod.rs
- Update module declaration (no change needed)

### Step 9: Test and verify
- Compile check
- Run application
- Test all features

### Step 10: Documentation
- Add module-level docs
- Add example usage

---

## 4. Testing Checklist

- [ ] Code compiles without errors
- [ ] Application starts successfully
- [ ] Auto-capture works for text
- [ ] Auto-capture works for images
- [ ] Manual "Paste Current" works
- [ ] "Add Content" dialog works
- [ ] Edit dialog works
- [ ] Delete button works
- [ ] Copy button works
- [ ] Search functionality works
- [ ] Filter by type works
- [ ] "Clear All" works
- [ ] "Copy Latest" works
- [ ] No memory leaks
- [ ] No borrow checker errors

---

## 5. Future Extensions

With this architecture, the following become straightforward:

1. **Database persistence**: Implement `Store` trait for SQLite
2. **File storage**: Implement `Store` trait for JSON/bincode
3. **Custom backends**: Implement `Backend` trait for Wayland
4. **Regex search**: Extend `Filter` trait
5. **Tags system**: Add tag field to `ClipboardItem`
6. **Shortcuts**: Add key bindings to UI module
7. **Theming**: Extract colors to `theme.rs`
8. **Export/Import**: Add commands to UI
9. **Cloud sync**: Implement `Store` trait for cloud backend
10. **Undo/Redo**: Use command pattern in actions

---

## 6. Migration Notes

**No breaking changes** to the public API:
- `ClipboardHistory` struct remains public
- `Demo` and `View` trait implementations unchanged
- All existing functionality preserved

**Internal changes**:
- All functionality split into modules
- Traits for extensibility
- Better error handling

---

*This refactoring plan is designed to be executed step-by-step with verification at each stage.*
