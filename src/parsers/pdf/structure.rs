use crate::{AstNode, TableCell, TableRow};

use super::PdfTextObject;

mod block;
mod filter;

use block::infer_pdf_block_structure;
use filter::{is_pdf_repeated_noise_text, is_probably_human_text};

pub(super) fn infer_nodes_from_text_objects(
    objects: Vec<PdfTextObject>,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    let original_count = objects.len();
    let objects = objects
        .into_iter()
        .filter(|object| is_probably_human_text(&object.text))
        .filter(|object| !is_pdf_repeated_noise_text(&object.text))
        .collect::<Vec<_>>();
    if objects.len() < original_count {
        warnings.push(format!(
            "PDF parser skipped {} binary-like text fragment(s).",
            original_count - objects.len()
        ));
    }
    let max_font_size = objects
        .iter()
        .filter_map(|object| object.font_size)
        .fold(0.0_f32, f32::max);
    let min_font_size = objects
        .iter()
        .filter_map(|object| object.font_size)
        .filter(|size| *size > 0.0)
        .fold(f32::MAX, f32::min);
    let can_infer_headings = max_font_size.is_finite()
        && min_font_size.is_finite()
        && max_font_size >= min_font_size + 4.0;

    let paragraph_nodes = objects
        .into_iter()
        .map(|object| {
            if can_infer_headings && object.font_size == Some(max_font_size) {
                warnings.push(format!(
                    "PDF heading inference treated '{}' as h1 by font size.",
                    object.text
                ));
                AstNode::Heading {
                    level: 1,
                    text: object.text,
                }
            } else {
                AstNode::Paragraph(object.text)
            }
        })
        .collect::<Vec<_>>();
    let block_nodes = infer_pdf_block_structure(paragraph_nodes, warnings);
    merge_pdf_interrupted_paragraphs(infer_pdf_pipe_tables(block_nodes))
}

pub fn infer_headings(text: &str) -> Vec<AstNode> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.len() < 80
                && trimmed.chars().any(|ch| ch.is_uppercase())
                && !trimmed.ends_with('.')
            {
                AstNode::Heading {
                    level: 2,
                    text: trimmed.to_string(),
                }
            } else {
                AstNode::Paragraph(trimmed.to_string())
            }
        })
        .collect()
}

fn merge_pdf_interrupted_paragraphs(nodes: Vec<AstNode>) -> Vec<AstNode> {
    let mut merged = Vec::new();
    for node in nodes {
        if let AstNode::Paragraph(next) = &node
            && let Some(AstNode::Paragraph(previous)) = merged.last_mut()
            && should_merge_pdf_interrupted_paragraph(previous, next)
        {
            append_pdf_interrupted_paragraph(previous, next);
            continue;
        }
        merged.push(node);
    }
    merged
}

fn infer_pdf_pipe_tables(nodes: Vec<AstNode>) -> Vec<AstNode> {
    let mut output = Vec::new();
    let mut idx = 0;
    while idx < nodes.len() {
        if let Some((table, next_idx)) = parse_pdf_pipe_table(&nodes, idx) {
            output.push(table);
            idx = next_idx;
            continue;
        }
        output.push(nodes[idx].clone());
        idx += 1;
    }
    output
}

fn parse_pdf_pipe_table(nodes: &[AstNode], start: usize) -> Option<(AstNode, usize)> {
    let header = pipe_table_cells(paragraph_text(nodes.get(start)?)?)?;
    let separator = pipe_table_cells(paragraph_text(nodes.get(start + 1)?)?)?;
    if header.len() < 2 || header.len() != separator.len() || !is_pipe_table_separator(&separator) {
        return None;
    }

    let mut rows = vec![table_row_from_cells(header)];
    let mut idx = start + 2;
    while let Some(text) = nodes.get(idx).and_then(paragraph_text) {
        let Some(cells) = pipe_table_cells(text) else {
            break;
        };
        if cells
            == rows[0]
                .cells
                .iter()
                .map(|cell| cell.text.clone())
                .collect::<Vec<_>>()
            && nodes
                .get(idx + 1)
                .and_then(paragraph_text)
                .and_then(pipe_table_cells)
                .is_some_and(|next_cells| {
                    next_cells.len() == cells.len() && is_pipe_table_separator(&next_cells)
                })
        {
            break;
        }
        if cells.len() != rows[0].cells.len() || is_pipe_table_separator(&cells) {
            break;
        }
        rows.push(table_row_from_cells(cells));
        idx += 1;
    }

    (rows.len() >= 2).then_some((AstNode::Table { rows }, idx))
}

fn paragraph_text(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::Paragraph(text) => Some(text),
        _ => None,
    }
}

fn pipe_table_cells(text: &str) -> Option<Vec<String>> {
    let trimmed = text.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    (cells.len() >= 2).then_some(cells)
}

fn is_pipe_table_separator(cells: &[String]) -> bool {
    cells.iter().all(|cell| {
        let trimmed = cell.trim();
        trimmed.len() >= 3
            && trimmed
                .chars()
                .all(|character| matches!(character, '-' | ':' | ' '))
            && trimmed.chars().any(|character| character == '-')
    })
}

fn table_row_from_cells(cells: Vec<String>) -> TableRow {
    TableRow {
        cells: cells
            .into_iter()
            .map(|text| TableCell {
                text,
                rowspan: 1,
                colspan: 1,
                image: None,
            })
            .collect(),
    }
}

fn should_merge_pdf_interrupted_paragraph(previous: &str, next: &str) -> bool {
    let previous = previous.trim_end();
    let next = next.trim_start();
    if previous.is_empty() || next.is_empty() || starts_pdf_markdown_block(next) {
        return false;
    }
    if previous.chars().count() < 24 {
        return false;
    }
    !ends_pdf_sentence_or_block(previous)
}

fn append_pdf_interrupted_paragraph(previous: &mut String, next: &str) {
    let previous_trimmed_len = previous.trim_end().len();
    previous.truncate(previous_trimmed_len);
    let next = next.trim_start();
    if should_insert_pdf_join_space(previous, next) {
        previous.push(' ');
    }
    previous.push_str(next);
}

fn starts_pdf_markdown_block(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('#')
        || trimmed.starts_with('|')
        || trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed.starts_with("```")
        || ordered_list_marker_punctuation_index(trimmed).is_some()
}

fn ends_pdf_sentence_or_block(text: &str) -> bool {
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

fn should_insert_pdf_join_space(previous: &str, next: &str) -> bool {
    let previous_char = previous.chars().next_back();
    let next_char = next.chars().next();
    matches!((previous_char, next_char), (Some(left), Some(right)) if needs_pdf_word_boundary_space(left, right))
}

fn needs_pdf_word_boundary_space(left: char, right: char) -> bool {
    (left.is_ascii_alphanumeric() && right.is_ascii_alphanumeric())
        || (!left.is_ascii() && right.is_ascii_alphanumeric())
        || (left.is_ascii_alphanumeric() && !right.is_ascii())
        || (left.is_ascii_alphabetic() && right == '&')
        || (left == '&' && right.is_ascii_alphabetic())
}

fn ordered_list_marker_punctuation_index(text: &str) -> Option<usize> {
    let punctuation_index = text.find(['.', ')'])?;
    if punctuation_index == 0 {
        return None;
    }
    text[..punctuation_index]
        .chars()
        .all(|character| character.is_ascii_digit())
        .then_some(punctuation_index)
}
