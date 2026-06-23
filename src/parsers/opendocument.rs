use crate::{AstNode, TableCell, TableRow, decode_entities, strip_tags};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use zip::ZipArchive;

pub fn parse_opendocument_file(
    path: &Path,
    warnings: &mut Vec<String>,
) -> io::Result<Vec<AstNode>> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let mut content = String::new();
    archive
        .by_name("content.xml")
        .map_err(zip_error)?
        .read_to_string(&mut content)?;
    Ok(parse_content_xml(&content, warnings))
}

pub fn parse_content_xml(content_xml: &str, warnings: &mut Vec<String>) -> Vec<AstNode> {
    let mut nodes = Vec::new();
    let mut rest = content_xml;
    while let Some(start) = next_odf_block_start(rest) {
        let after = &rest[start..];
        if after.starts_with("<text:h") {
            if let Some((opening, body, end)) = extract_element(after, "</text:h>") {
                let level = attr_value(&opening, "text:outline-level")
                    .and_then(|value| value.parse::<u8>().ok())
                    .unwrap_or(1)
                    .clamp(1, 6);
                push_text_node(
                    &mut nodes,
                    AstNode::Heading {
                        level,
                        text: odf_text(&body),
                    },
                );
                rest = &after[end..];
                continue;
            }
        } else if after.starts_with("<text:p") {
            if let Some((_opening, body, end)) = extract_element(after, "</text:p>") {
                push_text_node(&mut nodes, AstNode::Paragraph(odf_text(&body)));
                rest = &after[end..];
                continue;
            }
        } else if after.starts_with("<table:table")
            && let Some((_opening, body, end)) = extract_element(after, "</table:table>")
        {
            let rows = parse_table_rows(&body);
            if !rows.is_empty() {
                nodes.push(AstNode::Table { rows });
                warnings.push("OpenDocument table structure restored from content.xml".to_string());
            }
            rest = &after[end..];
            continue;
        }
        rest = &after[1..];
    }
    if nodes.is_empty() {
        warnings
            .push("OpenDocument content.xml contained no supported structural nodes".to_string());
    }
    nodes
}

fn next_odf_block_start(input: &str) -> Option<usize> {
    ["<text:h", "<text:p", "<table:table"]
        .into_iter()
        .filter_map(|needle| input.find(needle))
        .min()
}

fn parse_table_rows(input: &str) -> Vec<TableRow> {
    extract_elements(input, "<table:table-row", "</table:table-row>")
        .into_iter()
        .map(|row| TableRow {
            cells: extract_elements(&row, "<table:table-cell", "</table:table-cell>")
                .into_iter()
                .map(|cell| TableCell {
                    text: odf_text(&cell),
                    rowspan: 1,
                    colspan: 1,
                    image: None,
                })
                .collect(),
        })
        .filter(|row| row.cells.iter().any(|cell| !cell.text.is_empty()))
        .collect()
}

fn push_text_node(nodes: &mut Vec<AstNode>, node: AstNode) {
    let keep = match &node {
        AstNode::Heading { text, .. } | AstNode::Paragraph(text) => !text.trim().is_empty(),
        _ => true,
    };
    if keep {
        nodes.push(node);
    }
}

fn odf_text(input: &str) -> String {
    decode_entities(&strip_tags(input))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_elements(input: &str, open: &str, close: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find(open) {
        let after = &rest[start..];
        let Some((_opening, body, end)) = extract_element(after, close) else {
            break;
        };
        result.push(body);
        rest = &after[end..];
    }
    result
}

fn extract_element(input: &str, close: &str) -> Option<(String, String, usize)> {
    let open_end = input.find('>')?;
    let opening = input[..=open_end].to_string();
    let body_start = open_end + 1;
    let body_end_rel = input[body_start..].find(close)?;
    let body_end = body_start + body_end_rel;
    Some((
        opening,
        input[body_start..body_end].to_string(),
        body_end + close.len(),
    ))
}

fn attr_value(input: &str, name: &str) -> Option<String> {
    let pattern = format!("{name}=\"");
    let start = input.find(&pattern)? + pattern.len();
    let end = input[start..].find('"')?;
    Some(input[start..start + end].to_string())
}

fn zip_error(error: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
