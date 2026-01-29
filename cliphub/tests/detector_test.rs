use cliphub::clipboard::detect_content_type;

#[test]
fn test_detect_rust_code() {
    let code = r#"fn main() {
        println!("Hello");
    }"#;

    let result = detect_content_type(code);
    assert!(result.language.as_ref().is_some_and(|l| l == "rust"));
}

#[test]
fn test_detect_url() {
    let url = "https://example.com";
    let result = detect_content_type(url);
    assert!(result.is_url);
}

#[test]
fn test_detect_plain_text() {
    let text = "Just some plain text";
    let result = detect_content_type(text);
    assert!(!result.is_url);
    assert!(result.language.is_none());
}
