use crate::AstNode;

use super::headings::should_promote_first_paragraph;
use super::inline::ordered_list_marker_punctuation_index;

pub(super) fn merge_interrupted_paragraphs(ast: &[AstNode]) -> Vec<AstNode> {
    let mut merged = Vec::new();
    for node in ast {
        let merging_first_node = merged.len() == 1;
        if let AstNode::Paragraph(next) = node
            && let Some(AstNode::Paragraph(previous)) = merged.last_mut()
            && should_merge_interrupted_paragraph(previous, next, merging_first_node)
        {
            append_interrupted_paragraph(previous, next);
            continue;
        }
        merged.push(node.clone());
    }
    merged
}

fn should_merge_interrupted_paragraph(
    previous: &str,
    next: &str,
    previous_is_first_node: bool,
) -> bool {
    let previous = previous.trim_end();
    let next = next.trim_start();
    if previous.is_empty() || next.is_empty() {
        return false;
    }
    if previous_is_first_node && should_promote_first_paragraph(previous) {
        return false;
    }
    if starts_markdown_block(next) {
        return false;
    }
    !ends_sentence_or_block(previous)
}

fn append_interrupted_paragraph(previous: &mut String, next: &str) {
    let previous_trimmed_len = previous.trim_end().len();
    previous.truncate(previous_trimmed_len);
    let next = next.trim_start();
    if should_insert_join_space(previous, next) {
        previous.push(' ');
    }
    previous.push_str(next);
}

fn starts_markdown_block(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('#')
        || trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed.starts_with("```")
        || ordered_list_marker_punctuation_index(trimmed).is_some()
}

fn ends_sentence_or_block(text: &str) -> bool {
    let trimmed = text.trim_end();
    if trimmed.ends_with("  ") {
        return true;
    }
    trimmed.chars().next_back().is_some_and(|character| {
        matches!(
            character,
            '.' | '。'
                | '!'
                | '！'
                | '?'
                | '？'
                | ':'
                | '：'
                | ';'
                | '；'
                | ')'
                | '）'
                | ']'
                | '】'
                | '"'
                | '”'
        )
    })
}

fn should_insert_join_space(previous: &str, next: &str) -> bool {
    let previous_char = previous.chars().next_back();
    let next_char = next.chars().next();
    matches!((previous_char, next_char), (Some(left), Some(right)) if needs_word_boundary_space(left, right))
}

fn needs_word_boundary_space(left: char, right: char) -> bool {
    (left.is_ascii_alphanumeric() && right.is_ascii_alphanumeric())
        || (!left.is_ascii() && right.is_ascii_alphanumeric())
        || (left.is_ascii_alphanumeric() && !right.is_ascii())
        || (left.is_ascii_alphabetic() && right == '&')
        || (left == '&' && right.is_ascii_alphabetic())
}
