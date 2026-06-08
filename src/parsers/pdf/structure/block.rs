use crate::AstNode;

type PdfPendingListItem = (String, Vec<AstNode>);
type PdfPendingList = Option<(bool, Vec<PdfPendingListItem>)>;

pub(super) fn infer_pdf_block_structure(
    nodes: Vec<AstNode>,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    let mut output = Vec::new();
    let mut pending_list: PdfPendingList = None;

    for node in nodes {
        let AstNode::Paragraph(text) = node else {
            flush_pending_pdf_list(&mut output, &mut pending_list, warnings);
            output.push(node);
            continue;
        };

        if let Some((ordered, marker, item)) = parse_pdf_list_item(&text) {
            match &mut pending_list {
                Some((current_ordered, items)) if *current_ordered == ordered => {
                    items.push((marker, vec![AstNode::Text(item)]));
                }
                _ => {
                    flush_pending_pdf_list(&mut output, &mut pending_list, warnings);
                    pending_list = Some((ordered, vec![(marker, vec![AstNode::Text(item)])]));
                }
            }
            continue;
        }

        flush_pending_pdf_list(&mut output, &mut pending_list, warnings);
        if let Some(level) = pdf_section_heading_level(&text) {
            warnings.push(format!(
                "PDF heading inference treated '{}' as h{} by section number.",
                text, level
            ));
            output.push(AstNode::Heading { level, text });
        } else {
            output.push(AstNode::Paragraph(text));
        }
    }

    flush_pending_pdf_list(&mut output, &mut pending_list, warnings);
    renumber_repeated_pdf_one_headings(output, warnings)
}

fn flush_pending_pdf_list(
    output: &mut Vec<AstNode>,
    pending_list: &mut PdfPendingList,
    warnings: &mut Vec<String>,
) {
    if let Some((ordered, items)) = pending_list.take() {
        if items.len() >= 2 {
            flush_multi_item_pdf_list(output, ordered, items, warnings);
        } else if let Some(item) = items.into_iter().next() {
            flush_single_pdf_list_item(output, ordered, item, warnings);
        }
    }
}

fn flush_multi_item_pdf_list(
    output: &mut Vec<AstNode>,
    ordered: bool,
    items: Vec<PdfPendingListItem>,
    warnings: &mut Vec<String>,
) {
    if ordered
        && items.iter().all(|(marker, _)| marker == "1")
        && items.iter().all(|(marker, item)| {
            pdf_section_heading_level(&pdf_list_item_paragraph(ordered, marker, item)).is_some()
        })
    {
        for (marker, item) in items {
            let paragraph = pdf_list_item_paragraph(ordered, &marker, &item);
            let level = pdf_section_heading_level(&paragraph).unwrap_or(2);
            warnings.push(format!(
                "PDF heading inference treated '{}' as h{} by repeated numbered item.",
                paragraph, level
            ));
            output.push(AstNode::Heading {
                level,
                text: paragraph,
            });
        }
        return;
    }

    warnings.push(format!(
        "PDF list inference grouped {} item(s).",
        items.len()
    ));
    output.push(AstNode::List {
        ordered,
        items: items.into_iter().map(|(_, item)| item).collect(),
    });
}

fn flush_single_pdf_list_item(
    output: &mut Vec<AstNode>,
    ordered: bool,
    item: PdfPendingListItem,
    warnings: &mut Vec<String>,
) {
    let paragraph = pdf_list_item_paragraph(ordered, &item.0, &item.1);
    if ordered && let Some(level) = pdf_section_heading_level(&paragraph) {
        warnings.push(format!(
            "PDF heading inference treated '{}' as h{} by single numbered item.",
            paragraph, level
        ));
        output.push(AstNode::Heading {
            level,
            text: paragraph,
        });
    } else {
        output.push(AstNode::Paragraph(paragraph));
    }
}

fn pdf_list_item_paragraph(ordered: bool, marker: &str, item: &[AstNode]) -> String {
    let text = item
        .iter()
        .filter_map(|node| match node {
            AstNode::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    let prefix = if ordered {
        format!("{marker}. ")
    } else {
        "- ".to_string()
    };
    format!("{prefix}{text}")
}

fn parse_pdf_list_item(text: &str) -> Option<(bool, String, String)> {
    let trimmed = text.trim();
    for marker in ["- ", "• ", "・ "] {
        if let Some(item) = trimmed.strip_prefix(marker) {
            return Some((false, marker.trim().to_string(), item.trim().to_string()));
        }
    }
    let (number, rest) = trimmed.split_once(". ")?;
    if !number.is_empty() && number.chars().all(|character| character.is_ascii_digit()) {
        return Some((true, number.to_string(), rest.trim().to_string()));
    }
    None
}

fn renumber_repeated_pdf_one_headings(
    nodes: Vec<AstNode>,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    let repeated_one_count = nodes
        .iter()
        .filter(|node| match node {
            AstNode::Heading { text, .. } => text.starts_with("1. "),
            _ => false,
        })
        .count();
    if repeated_one_count < 3 {
        return nodes;
    }

    let mut next = 1;
    nodes
        .into_iter()
        .map(|node| match node {
            AstNode::Heading { level, text } if text.starts_with("1. ") => {
                let rest = text.trim_start_matches("1. ").trim();
                let renumbered = format!("{next}. {rest}");
                next += 1;
                warnings.push(format!(
                    "PDF heading inference renumbered repeated heading '{}' to '{}'.",
                    text, renumbered
                ));
                AstNode::Heading {
                    level,
                    text: renumbered,
                }
            }
            other => other,
        })
        .collect()
}

fn pdf_section_heading_level(text: &str) -> Option<u8> {
    let trimmed = text.trim();
    if trimmed.chars().count() > 80 || trimmed.ends_with('。') || trimmed.ends_with('.') {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix('第') {
        let (number, suffix_rest) = rest.split_once('章')?;
        if !number.is_empty() && !suffix_rest.trim().is_empty() {
            return Some(1);
        }
    }
    let mut parts = trimmed.split_whitespace();
    let first = parts.next()?;
    let has_rest = parts.next().is_some();
    if first
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        if !has_rest {
            return None;
        }
        let dot_count = first.chars().filter(|character| *character == '.').count();
        if first.trim_matches('.').is_empty() || dot_count > 5 {
            return None;
        }
        let has_digit = first.chars().any(|character| character.is_ascii_digit());
        if has_digit {
            return Some((dot_count + 1).min(6) as u8);
        }
    }
    None
}
