use crate::AstNode;

use super::PdfParseResult;
use super::diagnosis::has_unmapped_cid_fonts;

pub(super) fn pdf_result_has_text(result: &PdfParseResult) -> bool {
    result.ast.iter().any(|node| match node {
        AstNode::Paragraph(text) | AstNode::Text(text) => {
            !text.starts_with("PDF text extraction produced no text")
        }
        AstNode::Heading { .. } => true,
        _ => true,
    })
}

pub(super) fn pdf_result_is_usable(result: &PdfParseResult) -> bool {
    pdf_result_has_text(result) && !pdf_result_looks_mojibake(result)
}

pub(super) fn pdf_result_score(result: &PdfParseResult) -> usize {
    if !pdf_result_has_text(result) {
        return 0;
    }
    let mut score = ast_text(&result.ast)
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if result.extraction_failed {
        score = score.saturating_sub(1_000);
    }
    if result.ocr_required {
        score = score.saturating_sub(500);
    }
    if pdf_result_looks_mojibake(result) {
        score = score.saturating_sub(10_000);
    }
    score
}

pub(super) fn pdf_text_looks_incomplete(bytes: &[u8], ast: &[AstNode]) -> bool {
    if !has_unmapped_cid_fonts(bytes) {
        return false;
    }
    let extracted = ast_text(ast);
    let cjk_count = extracted
        .chars()
        .filter(|character| {
            matches!(
                *character as u32,
                0x3040..=0x30ff | 0x3400..=0x9fff | 0xf900..=0xfaff
            )
        })
        .count();
    let text_len = extracted
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    text_len < 2_000 && cjk_count < 20
}

fn pdf_result_looks_mojibake(result: &PdfParseResult) -> bool {
    let text = ast_text(&result.ast);
    let total = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if total < 100 {
        return false;
    }
    let suspicious = text
        .chars()
        .filter(|character| {
            matches!(
                *character as u32,
                0x0370..=0x03ff | 0x0b80..=0x0bff | 0x0d80..=0x0dff
            )
        })
        .count();
    let cjk = text
        .chars()
        .filter(|character| {
            matches!(
                *character as u32,
                0x3040..=0x30ff | 0x3400..=0x9fff | 0xf900..=0xfaff
            )
        })
        .count();
    suspicious > 30 && suspicious > cjk * 2
}

fn ast_text(nodes: &[AstNode]) -> String {
    let mut text = String::new();
    for node in nodes {
        append_ast_text(node, &mut text);
        text.push('\n');
    }
    text
}

fn append_ast_text(node: &AstNode, output: &mut String) {
    match node {
        AstNode::Heading { text, .. } | AstNode::Paragraph(text) | AstNode::Text(text) => {
            output.push_str(text);
        }
        AstNode::List { items, .. } => {
            for item in items {
                for child in item {
                    append_ast_text(child, output);
                    output.push(' ');
                }
            }
        }
        AstNode::Table { rows } => {
            for row in rows {
                for cell in &row.cells {
                    output.push_str(&cell.text);
                    output.push(' ');
                }
            }
        }
        AstNode::Image { alt, title, .. } => {
            output.push_str(alt);
            if let Some(caption) = title {
                output.push(' ');
                output.push_str(caption);
            }
        }
        AstNode::CodeBlock { code, .. } => output.push_str(code),
        AstNode::RawHtml(html) => output.push_str(html),
        AstNode::Footnote { label, text } => {
            output.push_str(label);
            output.push(' ');
            output.push_str(text);
        }
    }
}
