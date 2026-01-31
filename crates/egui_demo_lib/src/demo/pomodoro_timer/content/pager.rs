//! Content pagination.

//! # Extension Points
//!
//! Implement [`Pager`] trait for custom pagination strategies:
//! - **MarkdownPager**: Split by headings
//! - **CodePager**: Split by functions/classes  
//! - **ImagePager**: Paginate image galleries

use crate::demo::pomodoro_timer::core::error::{Error, Result};
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
        let paragraphs: Vec<&str> = text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();
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
        let chars: Vec<char> = text.chars().collect();
        
        for chunk in chars.chunks(chars_per_page) {
            pages.push(chunk.iter().collect());
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
        let mut pager = TextPager::by_paragraphs(text.to_string(), 1);

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
        let mut pager = TextPager::by_paragraphs(text.to_string(), 1);

        assert_eq!(pager.current_content().as_ref(), "First page");
        pager.navigate(PageDirection::Next).unwrap();
        assert_eq!(pager.current_content().as_ref(), "Second page");
    }
}
