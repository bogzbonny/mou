#[must_use]
pub fn strip_markdown_tags(text: &str) -> String {
    if text.starts_with("```markdown") && text.ends_with("```") {
        text[12..text.len() - 3].trim().to_string()
    } else {
        text.to_string()
    }
}
