mod links;

use links::{angle_bracket_bare_link, angle_bracket_embedded_urls, repair_split_url_tokens};
pub(super) use links::{contains_url_prefix, is_angle_wrapped_autolink};

pub(super) fn normalize_inline_text(text: &str) -> String {
    let tokens = repair_split_url_tokens(text.split_whitespace());
    let normalized = tokens
        .iter()
        .map(|token| angle_bracket_bare_link(token))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = angle_bracket_embedded_urls(&normalized);
    escape_markdown_ambiguities(&normalized)
}

fn escape_markdown_ambiguities(text: &str) -> String {
    let mut escaped = text.replace(")[", ")\\[");
    let trimmed = escaped.trim_start();
    let leading_whitespace = escaped.len() - trimmed.len();
    if let Some(marker_index) = ordered_list_marker_punctuation_index(trimmed) {
        escaped.insert(leading_whitespace + marker_index, '\\');
    } else if trimmed.starts_with('#')
        || looks_like_link_reference_definition(trimmed)
        || looks_like_thematic_break(trimmed)
        || looks_like_unordered_list_marker(trimmed)
    {
        escaped.insert(leading_whitespace, '\\');
    }
    escaped
}

fn looks_like_link_reference_definition(text: &str) -> bool {
    let Some(close) = text.find("]:") else {
        return false;
    };
    text.starts_with('[') && close > 1
}

fn looks_like_thematic_break(text: &str) -> bool {
    let marker = text
        .chars()
        .find(|character| matches!(character, '-' | '_' | '*'));
    let Some(marker) = marker else {
        return false;
    };
    let marker_count = text
        .chars()
        .filter(|character| *character == marker)
        .count();
    marker_count >= 3
        && text
            .chars()
            .all(|character| character == marker || character.is_whitespace())
}

fn looks_like_unordered_list_marker(text: &str) -> bool {
    matches!(text, "- " | "* " | "+ ")
        || text.starts_with("- ")
        || text.starts_with("* ")
        || text.starts_with("+ ")
}

pub(super) fn ordered_list_marker_punctuation_index(text: &str) -> Option<usize> {
    let mut saw_digit = false;
    for (index, character) in text.char_indices() {
        if character.is_ascii_digit() {
            saw_digit = true;
            continue;
        }
        if saw_digit
            && matches!(character, '.' | ')')
            && (text[index + character.len_utf8()..].is_empty()
                || text[index + character.len_utf8()..].starts_with(' '))
        {
            return Some(index);
        }
        return None;
    }
    None
}
