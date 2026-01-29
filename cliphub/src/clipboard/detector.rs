use crate::types::ContentType;

pub struct ContentTypeDetection {
    pub content_type: ContentType,
    pub language: Option<String>,
    pub is_url: bool,
}

pub fn detect_content_type(content: &str) -> ContentTypeDetection {
    // Check for URL first
    if is_url(content) {
        return ContentTypeDetection {
            content_type: ContentType::Url {
                url: content.to_string(),
                title: None
            },
            language: None,
            is_url: true,
        };
    }

    // Check for code patterns
    if let Some(lang) = detect_code_language(content) {
        return ContentTypeDetection {
            content_type: ContentType::Code { language: lang.clone() },
            language: Some(lang),
            is_url: false,
        };
    }

    // Default to text
    ContentTypeDetection {
        content_type: ContentType::Text,
        language: None,
        is_url: false,
    }
}

fn is_url(text: &str) -> bool {
    text.starts_with("http://") || text.starts_with("https://")
}

fn detect_code_language(content: &str) -> Option<String> {
    let lower_content = content.to_lowercase();

    // Quick heuristics
    if lower_content.contains("fn main") || lower_content.contains("pub fn") {
        return Some("rust".to_string());
    }
    if lower_content.contains("def ") && lower_content.contains(":") {
        return Some("python".to_string());
    }
    if lower_content.contains("function ") || lower_content.contains("=>") {
        return Some("javascript".to_string());
    }

    None
}
