use super::docx_body_order::{DocxBlock, extract_body_blocks};
use super::docx_media::extract_image_target;
use super::docx_relationships::{attr_value, extract_attr_values_for_tag, relationship_target};
use crate::{AstNode, TableCell, TableRow, decode_entities, strip_tags};

pub fn parse_document_xml(xml: &str, warnings: &mut Vec<String>) -> Vec<AstNode> {
    parse_document_xml_with_rels(xml, "", warnings)
}

pub fn parse_document_xml_with_rels(
    xml: &str,
    rels_xml: &str,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    parse_document_xml_with_rels_and_notes(xml, rels_xml, "", "", warnings)
}

pub fn parse_document_xml_with_rels_and_notes(
    xml: &str,
    rels_xml: &str,
    footnotes_xml: &str,
    comments_xml: &str,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    let mut ast = Vec::new();
    let mut pending_caption: Option<String> = None;

    for block in extract_body_blocks(xml) {
        match block {
            DocxBlock::Paragraph(paragraph) => parse_paragraph(
                &paragraph,
                rels_xml,
                footnotes_xml,
                comments_xml,
                warnings,
                &mut pending_caption,
                &mut ast,
            ),
            DocxBlock::Table(table) => ast.push(AstNode::Table {
                rows: parse_table(&table, rels_xml),
            }),
        }
    }
    if ast.is_empty() {
        warnings.push("DOCX document.xml contained no supported paragraphs.".to_string());
    }
    ast
}

fn parse_paragraph(
    paragraph: &str,
    rels_xml: &str,
    footnotes_xml: &str,
    comments_xml: &str,
    warnings: &mut Vec<String>,
    pending_caption: &mut Option<String>,
    ast: &mut Vec<AstNode>,
) {
    let style = extract_style(paragraph);
    let text = extract_text_with_relationships(paragraph, rels_xml);
    if let Some(target) = extract_image_target(paragraph, rels_xml) {
        let alt = pending_caption
            .clone()
            .unwrap_or_else(|| "image".to_string());
        ast.push(AstNode::Image {
            alt,
            path: target,
            title: pending_caption.take(),
        });
        return;
    }
    if text.trim().is_empty() {
        return;
    }
    if let Some(level) = heading_level(style.as_deref()) {
        ast.push(AstNode::Heading { level, text });
    } else if paragraph.contains("<w:numPr") {
        ast.push(AstNode::List {
            ordered: false,
            items: vec![vec![AstNode::Text(text)]],
        });
    } else {
        if is_caption(&text) {
            *pending_caption = Some(text.clone());
        }
        if let Some(style) = style.as_deref()
            && !is_known_paragraph_style(style)
        {
            warnings.push(format!("unmapped docx paragraph style: {style}"));
        }
        ast.push(AstNode::Paragraph(text));
    }
    for id in extract_reference_ids(paragraph, "w:footnoteReference") {
        if let Some(text) = note_text_by_id(footnotes_xml, "w:footnote", &id) {
            ast.push(AstNode::Footnote { label: id, text });
        }
    }
    for id in extract_reference_ids(paragraph, "w:commentReference") {
        if let Some(text) = note_text_by_id(comments_xml, "w:comment", &id) {
            ast.push(AstNode::Footnote {
                label: format!("comment:{id}"),
                text,
            });
        }
    }
}

fn extract_blocks(input: &str, open: &str, close: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find(open) {
        let after = &rest[start..];
        let Some(open_end) = after.find('>') else {
            break;
        };
        let body_start = start + open_end + 1;
        let Some(end_rel) = rest[body_start..].find(close) else {
            break;
        };
        let end = body_start + end_rel;
        result.push(rest[body_start..end].to_string());
        rest = &rest[end + close.len()..];
    }
    result
}

fn extract_text(paragraph: &str) -> String {
    extract_blocks(paragraph, "<w:t", "</w:t>")
        .into_iter()
        .map(|part| decode_entities(&strip_tags(&part)))
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}

fn extract_text_with_relationships(paragraph: &str, rels_xml: &str) -> String {
    let mut text = extract_text(paragraph);
    for id in extract_reference_ids(paragraph, "w:hyperlink")
        .into_iter()
        .chain(extract_attr_values_for_tag(
            paragraph,
            "w:hyperlink",
            "r:id",
        ))
    {
        let Some(target) = relationship_target(rels_xml, &id) else {
            continue;
        };
        if !text.contains(&target) {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push('<');
            text.push_str(&target);
            text.push('>');
        }
    }
    text
}

fn extract_style(paragraph: &str) -> Option<String> {
    let marker = "w:pStyle";
    let start = paragraph.find(marker)?;
    let rest = &paragraph[start..];
    let attr = "w:val=\"";
    let value_start = rest.find(attr)? + attr.len();
    let value_end = rest[value_start..].find('"')?;
    Some(rest[value_start..value_start + value_end].to_string())
}

fn heading_level(style: Option<&str>) -> Option<u8> {
    let style = style?.to_ascii_lowercase();
    for level in 1..=6 {
        if style.contains(&format!("heading{level}")) || style.contains(&format!("見出し{level}"))
        {
            return Some(level);
        }
    }
    None
}

fn parse_table(table: &str, rels_xml: &str) -> Vec<TableRow> {
    extract_blocks(table, "<w:tr", "</w:tr>")
        .into_iter()
        .map(|row| TableRow {
            cells: extract_blocks(&row, "<w:tc", "</w:tc>")
                .into_iter()
                .map(|cell| TableCell {
                    text: extract_text(&cell),
                    rowspan: if cell.contains("<w:vMerge") { 2 } else { 1 },
                    colspan: extract_grid_span(&cell).unwrap_or(1),
                    image: extract_image_target(&cell, rels_xml),
                })
                .collect(),
        })
        .collect()
}

fn extract_grid_span(cell: &str) -> Option<usize> {
    let marker = "w:gridSpan";
    let start = cell.find(marker)?;
    let rest = &cell[start..];
    let attr = "w:val=\"";
    let value_start = rest.find(attr)? + attr.len();
    let value_end = rest[value_start..].find('"')?;
    rest[value_start..value_start + value_end].parse().ok()
}

fn extract_reference_ids(paragraph: &str, tag: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = paragraph;
    let marker = format!("<{tag} ");
    while let Some(start) = rest.find(&marker) {
        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            break;
        };
        if let Some(id) = attr_value(&after[..=end], "w:id") {
            ids.push(id);
        }
        rest = &after[end + 1..];
    }
    ids
}

fn note_text_by_id(notes_xml: &str, tag: &str, id: &str) -> Option<String> {
    let marker = format!("<{tag} ");
    let close = format!("</{tag}>");
    let mut rest = notes_xml;
    while let Some(start) = rest.find(&marker) {
        let after = &rest[start..];
        let Some(open_end) = after.find('>') else {
            break;
        };
        let opening = &after[..=open_end];
        let body_start = start + open_end + 1;
        let Some(end_rel) = rest[body_start..].find(&close) else {
            break;
        };
        let end = body_start + end_rel;
        if attr_value(opening, "w:id").as_deref() == Some(id) {
            let text = extract_text(&rest[body_start..end]);
            return (!text.trim().is_empty()).then_some(text);
        }
        rest = &rest[end + close.len()..];
    }
    None
}

fn is_caption(text: &str) -> bool {
    let lower = text.trim().to_ascii_lowercase();
    lower.starts_with("figure ")
        || lower.starts_with("fig. ")
        || lower.starts_with("図 ")
        || lower.starts_with("図表")
}

fn is_known_paragraph_style(style: &str) -> bool {
    heading_level(Some(style)).is_some() || style.eq_ignore_ascii_case("caption")
}
