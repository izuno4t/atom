use crate::AstNode;

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
    infer_pdf_block_structure(paragraph_nodes, warnings)
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
