//! Filtering and search abstraction for clipboard items.
//!
//! This module provides a trait-based filtering system that allows for
//! flexible, composable filtering of clipboard items.
//!
//! # Extension Points
//!
//! - Implement [`Filter`] trait for regex search
//! - Implement [`Filter`] trait for fuzzy search
//! - Implement [`Filter`] trait for date range filtering
//! - Implement [`Filter`] trait for tag-based filtering
//!
//! # Examples
//!
//! ```
//! use clipboard_history::core::filter::{Filter, TextSearchFilter};
//! use clipboard_history::core::item::ClipboardItem;
//!
//! let filter = TextSearchFilter::new("hello".to_string());
//! let item = ClipboardItem::from_text("Hello, world!".to_string());
//!
//! assert!(filter.matches(&item));
//! ```

use crate::demo::clipboard_history::core::item::{ClipboardItem, ContentType};

// -----------------------------------------------------------------------------
// Filter trait
// -----------------------------------------------------------------------------

/// Filter trait for clipboard items.
///
/// This trait defines the interface for filtering clipboard items.
/// Implementations can provide various filtering strategies.
///
/// # Extension Points
///
/// - **Regex search**: Match items using regular expressions
/// - **Fuzzy search**: Approximate string matching
/// - **Date range**: Filter by timestamp
/// - **Tags**: Filter by user-assigned tags
///
/// # Examples
///
/// ```
/// use clipboard_history::core::filter::{Filter, TextSearchFilter};
/// use clipboard_history::core::item::ClipboardItem;
///
/// let filter = TextSearchFilter::new("test".to_string());
/// let item = ClipboardItem::from_text("This is a test".to_string());
///
/// assert!(filter.matches(&item));
/// ```
pub trait Filter: Send + Sync {
    /// Test if an item matches this filter.
    fn matches(&self, item: &ClipboardItem) -> bool;

    /// Get a description of this filter for UI display.
    fn description(&self) -> String;

    /// Clone as a boxed trait object.
    fn clone_box(&self) -> FilterBox;
}

/// Boxed filter trait object.
///
/// This type alias is used when you need to store filters dynamically
/// or return them from functions.
pub type FilterBox = Box<dyn Filter>;

impl Clone for FilterBox {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// -----------------------------------------------------------------------------
// TextSearchFilter
// -----------------------------------------------------------------------------

/// Filter by text search in title and content.
///
/// This filter performs case-insensitive substring matching on both
/// the item's title and content.
///
/// # Examples
///
/// ```
/// use clipboard_history::core::filter::{Filter, TextSearchFilter};
/// use clipboard_history::core::item::ClipboardItem;
///
/// let filter = TextSearchFilter::new("world".to_string());
/// let item = ClipboardItem::from_text("Hello, World!".to_string());
///
/// assert!(filter.matches(&item));
/// assert_eq!(filter.description(), "Search: \"world\"");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextSearchFilter {
    query: String,
    case_sensitive: bool,
}

impl TextSearchFilter {
    /// Create a new case-insensitive text search filter.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::filter::TextSearchFilter;
    ///
    /// let filter = TextSearchFilter::new("test".to_string());
    /// ```
    pub fn new(query: String) -> Self {
        Self {
            query,
            case_sensitive: false,
        }
    }

    /// Set whether the search should be case-sensitive.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::filter::TextSearchFilter;
    ///
    /// let filter = TextSearchFilter::new("Test".to_string())
    ///     .case_sensitive(true);
    /// ```
    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }

    /// Get the current search query.
    pub fn query(&self) -> &str {
        &self.query
    }
}

impl Default for TextSearchFilter {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl Filter for TextSearchFilter {
    fn matches(&self, item: &ClipboardItem) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let (title, content, query) = if self.case_sensitive {
            (
                item.title.clone(),
                item.content.clone(),
                self.query.clone(),
            )
        } else {
            (
                item.title.to_lowercase(),
                item.content.to_lowercase(),
                self.query.to_lowercase(),
            )
        };

        title.contains(&query) || content.contains(&query)
    }

    fn description(&self) -> String {
        if self.query.is_empty() {
            "All items".to_string()
        } else {
            format!("Search: \"{}\"", self.query)
        }
    }

    fn clone_box(&self) -> FilterBox {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// TypeFilter
// -----------------------------------------------------------------------------

/// Filter by content type.
///
/// This filter matches items whose content type is in the allowed list.
///
/// # Examples
///
/// ```
/// use clipboard_history::core::filter::{Filter, TypeFilter};
/// use clipboard_history::core::item::{ClipboardItem, ContentType};
///
/// let filter = TypeFilter::with_types(vec![ContentType::Url, ContentType::Image]);
/// let url_item = ClipboardItem::from_text("https://example.com".to_string());
///
/// assert!(filter.matches(&url_item));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeFilter {
    allowed_types: Vec<ContentType>,
}

impl TypeFilter {
    /// Create a new empty type filter (matches nothing).
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::filter::TypeFilter;
    ///
    /// let filter = TypeFilter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            allowed_types: Vec::new(),
        }
    }

    /// Create a type filter with specific allowed types.
    ///
    /// # Arguments
    ///
    /// * `types` - List of content types to allow
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::filter::TypeFilter;
    /// use clipboard_history::core::item::ContentType;
    ///
    /// let filter = TypeFilter::with_types(vec![
    ///     ContentType::Text,
    ///     ContentType::Url,
    /// ]);
    /// ```
    pub fn with_types(types: Vec<ContentType>) -> Self {
        Self {
            allowed_types: types,
        }
    }

    /// Create a filter that matches all content types.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::filter::TypeFilter;
    ///
    /// let filter = TypeFilter::all();
    /// assert!(filter.matches_all());
    /// ```
    pub fn all() -> Self {
        Self {
            allowed_types: vec![
                ContentType::Text,
                ContentType::Url,
                ContentType::Image,
                ContentType::Code {
                    language: "any".to_string(),
                },
                ContentType::File {
                    extension: "any".to_string(),
                },
            ],
        }
    }

    /// Create a filter that matches only text content.
    pub fn text_only() -> Self {
        Self::with_types(vec![ContentType::Text])
    }

    /// Create a filter that matches only URLs.
    pub fn url_only() -> Self {
        Self::with_types(vec![ContentType::Url])
    }

    /// Create a filter that matches only images.
    pub fn image_only() -> Self {
        Self::with_types(vec![ContentType::Image])
    }

    /// Check if this filter matches all types.
    pub fn matches_all(&self) -> bool {
        self.allowed_types.len() >= 5
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

        self.allowed_types.iter().any(|allowed| item.matches_type(allowed))
    }

    fn description(&self) -> String {
        if self.allowed_types.is_empty() {
            "All types".to_string()
        } else if self.matches_all() {
            "All types".to_string()
        } else {
            format!("Types: {}", self.allowed_types.len())
        }
    }

    fn clone_box(&self) -> FilterBox {
        Box::new(self.clone())
    }
}

// -----------------------------------------------------------------------------
// CompositeFilter
// -----------------------------------------------------------------------------

/// Logical operator for composite filters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicOperator {
    /// All filters must match (AND)
    And,
    /// At least one filter must match (OR)
    Or,
}

/// Combine multiple filters with AND/OR logic.
///
/// This filter allows you to combine multiple filters using logical
/// operators for more complex filtering.
///
/// # Examples
///
/// ```
/// use clipboard_history::core::filter::{Filter, CompositeFilter, TextSearchFilter, TypeFilter, LogicOperator};
/// use clipboard_history::core::item::{ClipboardItem, ContentType};
///
/// // Match items that are URLs AND contain "example"
/// let text_filter: FilterBox = Box::new(TextSearchFilter::new("example".to_string()));
/// let type_filter: FilterBox = Box::new(TypeFilter::url_only());
/// let composite = CompositeFilter::and(vec![text_filter, type_filter]);
/// ```
#[derive(Clone)]
pub struct CompositeFilter {
    filters: Vec<FilterBox>,
    operator: LogicOperator,
}

impl CompositeFilter {
    /// Create a filter that combines all filters with AND logic.
    ///
    /// # Arguments
    ///
    /// * `filters` - List of filters to combine
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::filter::{CompositeFilter, TextSearchFilter};
    ///
    /// let filter = CompositeFilter::and(vec![
    ///     Box::new(TextSearchFilter::new("test".to_string())),
    /// ]);
    /// ```
    pub fn and(filters: Vec<FilterBox>) -> Self {
        Self {
            filters,
            operator: LogicOperator::And,
        }
    }

    /// Create a filter that combines all filters with OR logic.
    ///
    /// # Arguments
    ///
    /// * `filters` - List of filters to combine
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::filter::{CompositeFilter, TextSearchFilter};
    ///
    /// let filter = CompositeFilter::or(vec![
    ///     Box::new(TextSearchFilter::new("test".to_string())),
    /// ]);
    /// ```
    pub fn or(filters: Vec<FilterBox>) -> Self {
        Self {
            filters,
            operator: LogicOperator::Or,
        }
    }

    /// Get the logical operator used by this filter.
    pub fn operator(&self) -> LogicOperator {
        self.operator
    }

    /// Get the number of filters in this composite.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if this composite filter is empty.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
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
        let op = if self.operator == LogicOperator::And {
            "AND"
        } else {
            "OR"
        };
        format!("Composite ({}: {} filters)", op, self.filters.len())
    }

    fn clone_box(&self) -> FilterBox {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_search_filter_case_insensitive() {
        let filter = TextSearchFilter::new("HELLO".to_string());
        let item = ClipboardItem::from_text("hello world".to_string());

        assert!(filter.matches(&item));
    }

    #[test]
    fn test_text_search_filter_case_sensitive() {
        let filter = TextSearchFilter::new("HELLO".to_string()).case_sensitive(true);
        let item_lower = ClipboardItem::from_text("hello world".to_string());
        let item_upper = ClipboardItem::from_text("HELLO world".to_string());

        assert!(!filter.matches(&item_lower));
        assert!(filter.matches(&item_upper));
    }

    #[test]
    fn test_text_search_empty_query() {
        let filter = TextSearchFilter::new(String::new());
        let item = ClipboardItem::from_text("Anything".to_string());

        assert!(filter.matches(&item));
    }

    #[test]
    fn test_type_filter_url() {
        let filter = TypeFilter::url_only();
        let url_item = ClipboardItem::from_text("https://example.com".to_string());
        let text_item = ClipboardItem::from_text("Just text".to_string());

        assert!(filter.matches(&url_item));
        assert!(!filter.matches(&text_item));
    }

    #[test]
    fn test_type_filter_all() {
        let filter = TypeFilter::all();
        let item = ClipboardItem::from_text("Anything".to_string());

        assert!(filter.matches(&item));
    }

    #[test]
    fn test_composite_filter_and() {
        let text_filter: FilterBox = Box::new(TextSearchFilter::new("example".to_string()));
        let type_filter: FilterBox = Box::new(TypeFilter::url_only());
        let composite = CompositeFilter::and(vec![text_filter, type_filter]);

        let matching_item = ClipboardItem::from_text("https://example.com".to_string());
        let non_matching_item = ClipboardItem::from_text("https://other.com".to_string());

        assert!(composite.matches(&matching_item));
        assert!(!composite.matches(&non_matching_item));
    }

    #[test]
    fn test_composite_filter_or() {
        let text_filter: FilterBox = Box::new(TextSearchFilter::new("hello".to_string()));
        let type_filter: FilterBox = Box::new(TypeFilter::url_only());
        let composite = CompositeFilter::or(vec![text_filter, type_filter]);

        let text_item = ClipboardItem::from_text("hello world".to_string());
        let url_item = ClipboardItem::from_text("https://example.com".to_string());
        let other_item = ClipboardItem::from_text("random text".to_string());

        assert!(composite.matches(&text_item));
        assert!(composite.matches(&url_item));
        assert!(!composite.matches(&other_item));
    }

    #[test]
    fn test_composite_filter_empty() {
        let composite = CompositeFilter::and(vec![]);
        let item = ClipboardItem::from_text("Anything".to_string());

        assert!(composite.matches(&item));
    }
}
