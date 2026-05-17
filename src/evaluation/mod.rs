use crate::{AstNode, MetricScore, OcrCerCase};
use std::collections::BTreeMap;

pub fn evaluate_structure_fidelity(expected: &[AstNode], actual: &[AstNode]) -> MetricScore {
    let expected_tree = EvalTree::document(expected);
    let actual_tree = EvalTree::document(actual);
    let total = expected_tree
        .structural_size()
        .max(actual_tree.structural_size())
        .max(1);
    let distance = ordered_tree_edit_distance(&expected_tree, &actual_tree);
    let score = 1.0 - (distance as f64 / total as f64).min(1.0);
    let warnings = (distance > 0)
        .then(|| format!("structure edit distance {distance} over {total} structural node(s)"))
        .into_iter()
        .collect();
    MetricScore {
        name: "structure_fidelity".to_string(),
        score,
        errors: distance,
        warnings,
    }
}

pub fn evaluate_heading_recall(expected: &[AstNode], markdown: &str) -> MetricScore {
    let expected_headings = collect_headings(expected);
    let actual_headings = markdown
        .lines()
        .filter_map(parse_markdown_heading)
        .collect::<Vec<_>>();
    let missing = expected_headings
        .iter()
        .filter(|expected| {
            !actual_headings.iter().any(|actual| {
                expected.level == actual.level && expected.text.trim() == actual.text.trim()
            })
        })
        .collect::<Vec<_>>();
    let total = expected_headings.len().max(1);
    let found = expected_headings.len().saturating_sub(missing.len());
    MetricScore {
        name: "heading_recall".to_string(),
        score: found as f64 / total as f64,
        errors: missing.len(),
        warnings: missing
            .into_iter()
            .map(|heading| format!("missing heading h{} {}", heading.level, heading.text))
            .collect(),
    }
}

pub fn evaluate_table_integrity(markdown: &str) -> MetricScore {
    let mut warnings = Vec::new();
    let mut errors = 0;
    let has_pipe_table = evaluate_pipe_tables(markdown, &mut warnings, &mut errors);
    let has_html_table = evaluate_html_tables(markdown, &mut warnings, &mut errors);
    if !has_pipe_table && !has_html_table {
        errors += 1;
        warnings.push("no table detected".to_string());
    }
    let score = if errors == 0 { 1.0 } else { 0.0 };
    MetricScore {
        name: "table_integrity".to_string(),
        score,
        errors,
        warnings,
    }
}

pub fn evaluate_lint_score(markdown: &str) -> MetricScore {
    let mut errors = 0;
    let mut warnings = Vec::new();
    if !markdown.ends_with('\n') {
        errors += 1;
        warnings.push("Markdown must end with a trailing newline.".to_string());
    }
    if markdown.lines().any(|line| line.ends_with(' ')) {
        errors += 1;
        warnings.push("Markdown contains trailing spaces.".to_string());
    }
    let lines = markdown.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            if index > 0 && !lines[index - 1].trim().is_empty() {
                errors += 1;
                warnings.push("Markdown heading must be preceded by a blank line.".to_string());
            }
            if index + 1 < lines.len() && !lines[index + 1].trim().is_empty() {
                errors += 1;
                warnings.push("Markdown heading must be followed by a blank line.".to_string());
            }
        }
        if trimmed.starts_with("- ") {
            if index > 0 && !lines[index - 1].trim().is_empty() {
                errors += 1;
                warnings.push("Markdown list must be preceded by a blank line.".to_string());
            }
            if index + 1 < lines.len() && !lines[index + 1].trim().is_empty() {
                errors += 1;
                warnings.push("Markdown list must be followed by a blank line.".to_string());
            }
        }
        if trimmed.starts_with('|') {
            let previous_is_table = index > 0 && lines[index - 1].trim().starts_with('|');
            let next_is_table = index + 1 < lines.len() && lines[index + 1].trim().starts_with('|');
            if !previous_is_table && index > 0 && !lines[index - 1].trim().is_empty() {
                errors += 1;
                warnings.push("Markdown table must be preceded by a blank line.".to_string());
            }
            if !next_is_table && index + 1 < lines.len() && !lines[index + 1].trim().is_empty() {
                errors += 1;
                warnings.push("Markdown table must be followed by a blank line.".to_string());
            }
        }
    }
    MetricScore {
        name: "lint_score".to_string(),
        score: if errors == 0 { 1.0 } else { 0.0 },
        errors,
        warnings,
    }
}

pub fn evaluate_ocr_cer(expected: &str, actual: &str) -> MetricScore {
    let distance = levenshtein(expected, actual);
    let total = expected.chars().count().max(1);
    MetricScore {
        name: "ocr_cer".to_string(),
        score: 1.0 - (distance as f64 / total as f64).min(1.0),
        errors: distance,
        warnings: Vec::new(),
    }
}

pub fn evaluate_ocr_cer_by_group(cases: &[OcrCerCase]) -> Vec<MetricScore> {
    let mut grouped = BTreeMap::<(&str, &str), (String, String)>::new();
    for case in cases {
        let entry = grouped
            .entry((case.language.as_str(), case.orientation.as_str()))
            .or_default();
        entry.0.push_str(&case.expected);
        entry.1.push_str(&case.actual);
    }
    grouped
        .into_iter()
        .map(|((language, orientation), (expected, actual))| {
            let mut score = evaluate_ocr_cer(&expected, &actual);
            score.name = format!("ocr_cer:{language}:{orientation}");
            score
        })
        .collect()
}

pub fn evaluate_translation_structure_preserve(before: &str, after: &str) -> MetricScore {
    let before_markers = structure_markers(before);
    let after_markers = structure_markers(after);
    let total = before_markers.len().max(1);
    let matched = before_markers
        .iter()
        .zip(after_markers.iter())
        .filter(|(left, right)| left == right)
        .count();
    let warnings = before_markers
        .iter()
        .zip(
            after_markers
                .iter()
                .chain(std::iter::repeat(&"missing".to_string())),
        )
        .filter(|(left, right)| left != right)
        .map(|(left, right)| format!("translation structure mismatch: {left} != {right}"))
        .collect::<Vec<_>>();
    MetricScore {
        name: "translation_structure_preserve".to_string(),
        score: matched as f64 / total as f64,
        errors: total.saturating_sub(matched),
        warnings,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EvalTree {
    label: String,
    children: Vec<EvalTree>,
}

impl EvalTree {
    fn document(nodes: &[AstNode]) -> Self {
        Self {
            label: "document".to_string(),
            children: nodes.iter().map(Self::from_ast).collect(),
        }
    }

    fn from_ast(node: &AstNode) -> Self {
        match node {
            AstNode::Heading { level, .. } => Self::leaf(format!("heading:{level}")),
            AstNode::Paragraph(_) => Self::leaf("paragraph"),
            AstNode::List { ordered, items } => Self {
                label: format!("list:{}", if *ordered { "ordered" } else { "unordered" }),
                children: items
                    .iter()
                    .map(|item| Self {
                        label: "list-item".to_string(),
                        children: item.iter().map(Self::from_ast).collect(),
                    })
                    .collect(),
            },
            AstNode::Text(_) => Self::leaf("text"),
            AstNode::Table { rows } => Self {
                label: "table".to_string(),
                children: rows
                    .iter()
                    .map(|row| Self {
                        label: "table-row".to_string(),
                        children: row
                            .cells
                            .iter()
                            .map(|cell| {
                                Self::leaf(format!(
                                    "table-cell:rowspan={}:colspan={}:image={}",
                                    cell.rowspan,
                                    cell.colspan,
                                    cell.image.is_some()
                                ))
                            })
                            .collect(),
                    })
                    .collect(),
            },
            AstNode::Image { title, .. } => Self::leaf(format!("image:title={}", title.is_some())),
            AstNode::CodeBlock { language, .. } => {
                Self::leaf(format!("code:language={}", language.is_some()))
            }
            AstNode::Footnote { .. } => Self::leaf("footnote"),
            AstNode::RawHtml(_) => Self::leaf("raw-html"),
        }
    }

    fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
        }
    }

    fn size(&self) -> usize {
        1 + self.children.iter().map(Self::size).sum::<usize>()
    }

    fn structural_size(&self) -> usize {
        self.children.iter().map(Self::size).sum()
    }
}

fn ordered_tree_edit_distance(expected: &EvalTree, actual: &EvalTree) -> usize {
    let rename_cost = usize::from(expected.label != actual.label);
    rename_cost + ordered_forest_edit_distance(&expected.children, &actual.children)
}

fn ordered_forest_edit_distance(expected: &[EvalTree], actual: &[EvalTree]) -> usize {
    let mut costs: Vec<usize> = (0..=actual.len())
        .map(|index| actual[..index].iter().map(EvalTree::size).sum())
        .collect::<Vec<_>>();
    for expected_node in expected {
        let mut previous_diagonal = costs[0];
        costs[0] += expected_node.size();
        for (j, actual_node) in actual.iter().enumerate() {
            let previous_up = costs[j + 1];
            let delete_cost = previous_up + expected_node.size();
            let insert_cost = costs[j] + actual_node.size();
            let replace_cost =
                previous_diagonal + ordered_tree_edit_distance(expected_node, actual_node);
            costs[j + 1] = delete_cost.min(insert_cost).min(replace_cost);
            previous_diagonal = previous_up;
        }
    }
    costs[actual.len()]
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HeadingRef {
    level: u8,
    text: String,
}

fn collect_headings(nodes: &[AstNode]) -> Vec<HeadingRef> {
    let mut headings = Vec::new();
    for node in nodes {
        match node {
            AstNode::Heading { level, text } => headings.push(HeadingRef {
                level: *level,
                text: text.clone(),
            }),
            AstNode::List { items, .. } => {
                for item in items {
                    headings.extend(collect_headings(item));
                }
            }
            _ => {}
        }
    }
    headings
}

fn parse_markdown_heading(line: &str) -> Option<HeadingRef> {
    let trimmed = line.trim_start();
    let marker_count = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&marker_count) {
        return None;
    }
    let content = trimmed.get(marker_count..)?;
    if !content.starts_with(' ') {
        return None;
    }
    let text = content.trim().trim_end_matches('#').trim();
    if text.is_empty() {
        return None;
    }
    Some(HeadingRef {
        level: marker_count as u8,
        text: text.to_string(),
    })
}

fn evaluate_pipe_tables(markdown: &str, warnings: &mut Vec<String>, errors: &mut usize) -> bool {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut found = false;
    let mut index = 0;
    while index < lines.len() {
        if !lines[index].trim_start().starts_with('|') {
            index += 1;
            continue;
        }

        let start = index;
        while index < lines.len() && lines[index].trim_start().starts_with('|') {
            index += 1;
        }
        let table_lines = &lines[start..index];
        if table_lines.len() < 2 || !table_lines.iter().any(|line| is_pipe_separator(line)) {
            continue;
        }

        found = true;
        let expected_cells = pipe_cell_count(table_lines[0]);
        for (offset, line) in table_lines.iter().enumerate() {
            let actual_cells = pipe_cell_count(line);
            if actual_cells != expected_cells {
                *errors += 1;
                warnings.push(format!(
                    "pipe table row {} has {} cells, expected {}",
                    start + offset + 1,
                    actual_cells,
                    expected_cells
                ));
            }
        }
    }
    found
}

fn evaluate_html_tables(markdown: &str, warnings: &mut Vec<String>, errors: &mut usize) -> bool {
    let lower = markdown.to_ascii_lowercase();
    let open_count = lower.matches("<table").count();
    let close_count = lower.matches("</table>").count();
    if open_count == 0 && close_count == 0 {
        return false;
    }
    if open_count > close_count {
        *errors += open_count - close_count;
        warnings.push("unclosed html table".to_string());
    } else if close_count > open_count {
        *errors += close_count - open_count;
        warnings.push("html table close tag without open tag".to_string());
    }
    true
}

fn is_pipe_separator(line: &str) -> bool {
    let cells = pipe_cells(line);
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            trimmed.len() >= 3
                && trimmed
                    .chars()
                    .all(|character| matches!(character, '-' | ':' | ' '))
        })
}

fn pipe_cell_count(line: &str) -> usize {
    pipe_cells(line).len()
}

fn pipe_cells(line: &str) -> Vec<&str> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

fn structure_markers(markdown: &str) -> Vec<String> {
    let mut in_code = false;
    markdown
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                if in_code {
                    in_code = false;
                    None
                } else {
                    in_code = true;
                    Some("code".to_string())
                }
            } else if in_code {
                None
            } else if trimmed.starts_with('#') {
                Some("#".repeat(trimmed.chars().take_while(|ch| *ch == '#').count()))
            } else if trimmed.starts_with("- ") {
                Some("-".to_string())
            } else if trimmed.starts_with('|') {
                Some("|".to_string())
            } else {
                None
            }
        })
        .collect()
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut costs = (0..=right_chars.len()).collect::<Vec<_>>();
    for (i, left_char) in left.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, right_char) in right_chars.iter().enumerate() {
            let old = costs[j + 1];
            costs[j + 1] = if left_char == *right_char {
                last
            } else {
                1 + last.min(old).min(costs[j])
            };
            last = old;
        }
    }
    costs[right_chars.len()]
}
