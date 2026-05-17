use std::collections::BTreeMap;

use crate::{AstNode, Flavor, TableRow, escape_html};

pub fn write_markdown(ast: &[AstNode], flavor: Flavor) -> String {
    let normalized_ast;
    let ast = if matches!(flavor, Flavor::Markdownlint) {
        normalized_ast = merge_interrupted_paragraphs(ast);
        normalized_ast.as_slice()
    } else {
        ast
    };
    let mut output = String::new();
    let mut heading_state = HeadingState::default();
    let mut nodes = ast.iter().peekable();
    if let Some(AstNode::Paragraph(text)) = nodes.peek()
        && should_promote_first_paragraph(text)
    {
        let text = normalize_heading_text(&normalize_inline_text(text));
        let text = unique_heading_text(&text, &mut heading_state);
        output.push_str("# ");
        output.push_str(&text);
        output.push_str("\n\n");
        heading_state.h1_written = true;
        heading_state.last_level = 1;
        nodes.next();
    }
    for node in nodes {
        write_node(node, flavor, &mut output, 0, &mut heading_state);
    }
    while output.ends_with("\n\n") {
        output.pop();
    }
    output
}

fn merge_interrupted_paragraphs(ast: &[AstNode]) -> Vec<AstNode> {
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

fn starts_markdown_block(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('#')
        || trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed.starts_with("```")
        || ordered_list_marker_punctuation_index(trimmed).is_some()
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

#[derive(Default)]
struct HeadingState {
    h1_written: bool,
    last_level: u8,
    headings: BTreeMap<String, usize>,
}

fn write_node(
    node: &AstNode,
    flavor: Flavor,
    output: &mut String,
    depth: usize,
    heading_state: &mut HeadingState,
) {
    match node {
        AstNode::Heading { level, text } => {
            let normalized_level = normalize_heading_level(*level, heading_state);
            let text = normalize_heading_text(&normalize_inline_text(text));
            let text = unique_heading_text(&text, heading_state);
            let (text, overflow) = split_long_heading(&text, normalized_level);
            output.push_str(&"#".repeat(normalized_level as usize));
            output.push(' ');
            output.push_str(&text);
            output.push_str("\n\n");
            if let Some(overflow) = overflow {
                write_wrapped_text(&overflow, output);
                output.push_str("\n\n");
            }
        }
        AstNode::Paragraph(text) => {
            write_wrapped_text(&normalize_inline_text(text), output);
            output.push_str("\n\n");
        }
        AstNode::Text(text) => output.push_str(&normalize_inline_text(text)),
        AstNode::List { ordered, items } => {
            for (index, item) in items.iter().enumerate() {
                let indent = "  ".repeat(depth);
                let marker = if *ordered {
                    if matches!(flavor, Flavor::Markdownlint) {
                        "1. ".to_string()
                    } else {
                        format!("{}. ", index + 1)
                    }
                } else {
                    "- ".to_string()
                };
                if let Some(text) = inline_nodes_to_text(item) {
                    let prefix = format!("{indent}{marker}");
                    let continuation = " ".repeat(prefix.chars().count());
                    write_wrapped_prefixed_text(
                        &normalize_inline_text(&text),
                        output,
                        &prefix,
                        &continuation,
                    );
                    output.push('\n');
                    continue;
                }
                output.push_str(&indent);
                if *ordered {
                    if matches!(flavor, Flavor::Markdownlint) {
                        output.push_str("1. ");
                    } else {
                        output.push_str(&format!("{}. ", index + 1));
                    }
                } else {
                    output.push_str("- ");
                }
                write_inline_nodes(item, flavor, output, heading_state);
                output.push('\n');
            }
            output.push('\n');
        }
        AstNode::Table { rows } => write_table(rows, flavor, output),
        AstNode::Image { alt, path, title } => {
            output.push_str("![");
            output.push_str(alt);
            output.push_str("](");
            output.push_str(&markdown_link_destination(path));
            if let Some(title) = title {
                output.push_str(" \"");
                output.push_str(title);
                output.push('"');
            }
            output.push_str(")\n\n");
        }
        AstNode::CodeBlock { language, code } => {
            output.push_str("```");
            output.push_str(language.as_deref().unwrap_or(""));
            output.push('\n');
            output.push_str(code.trim_end());
            output.push_str("\n```\n\n");
        }
        AstNode::Footnote { label, text } => {
            output.push_str("[^");
            output.push_str(label);
            output.push_str("]: ");
            write_wrapped_text(&normalize_inline_text(text), output);
            output.push_str("\n\n");
        }
        AstNode::RawHtml(html) => {
            output.push_str(html.trim());
            output.push('\n');
        }
    }
}

fn markdown_link_destination(path: &str) -> String {
    if path
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '(' | ')' | '<' | '>'))
    {
        format!("<{}>", path.replace('<', "%3C").replace('>', "%3E"))
    } else {
        path.to_string()
    }
}

fn normalize_heading_level(level: u8, heading_state: &mut HeadingState) -> u8 {
    let mut level = level.clamp(1, 6);
    if !heading_state.h1_written && heading_state.last_level == 0 {
        level = 1;
        heading_state.h1_written = true;
    } else if level == 1 {
        if heading_state.h1_written {
            level = 2;
        } else {
            heading_state.h1_written = true;
        }
    }
    if heading_state.last_level > 0 && level > heading_state.last_level + 1 {
        level = heading_state.last_level + 1;
    }
    heading_state.last_level = level;
    level
}

fn unique_heading_text(text: &str, heading_state: &mut HeadingState) -> String {
    let count = heading_state.headings.entry(text.to_string()).or_insert(0);
    *count += 1;
    if *count == 1 {
        text.to_string()
    } else {
        format!("{text} ({count})")
    }
}

fn write_inline_nodes(
    nodes: &[AstNode],
    flavor: Flavor,
    output: &mut String,
    heading_state: &mut HeadingState,
) {
    for node in nodes {
        match node {
            AstNode::Text(text) | AstNode::Paragraph(text) => {
                output.push_str(&normalize_inline_text(text));
            }
            _ => write_node(node, flavor, output, 1, heading_state),
        }
    }
}

fn inline_nodes_to_text(nodes: &[AstNode]) -> Option<String> {
    let mut text = String::new();
    for node in nodes {
        match node {
            AstNode::Text(value) | AstNode::Paragraph(value) => {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(value);
            }
            _ => return None,
        }
    }
    Some(text)
}

fn should_promote_first_paragraph(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty()
        && trimmed.chars().count() <= 80
        && !trimmed.starts_with("Unsupported input format:")
        && !trimmed.ends_with('。')
        && !trimmed.ends_with('.')
        && !trimmed.ends_with(',')
        && !trimmed.ends_with('、')
}

fn normalize_heading_text(text: &str) -> String {
    text.trim_end_matches(|character| {
        matches!(
            character,
            '.' | ',' | ';' | ':' | '!' | '?' | '。' | '、' | '；' | '：' | '！' | '？'
        )
    })
    .to_string()
}

fn split_long_heading(text: &str, level: u8) -> (String, Option<String>) {
    let available = 80usize.saturating_sub(level as usize + 1);
    if text.chars().count() <= available {
        return (text.to_string(), None);
    }
    let mut split_at = None;
    let mut count = 0;
    for (index, character) in text.char_indices() {
        count += 1;
        if character.is_whitespace() && count <= available {
            split_at = Some(index);
        }
        if count > available {
            break;
        }
    }
    let Some(index) = split_at else {
        return (text.chars().take(available).collect(), None);
    };
    let heading = text[..index].trim_end().to_string();
    let overflow = text[index..].trim().to_string();
    (heading, (!overflow.is_empty()).then_some(overflow))
}

fn write_wrapped_text(text: &str, output: &mut String) {
    let mut line = String::new();
    for token in text.split_whitespace() {
        if line.is_empty() {
            push_wrapped_token(token, output, &mut line);
        } else if line.chars().count() + 1 + token.chars().count() <= 80 {
            line.push(' ');
            line.push_str(token);
        } else {
            output.push_str(line.trim_end());
            output.push('\n');
            line.clear();
            push_wrapped_token(token, output, &mut line);
        }
    }
    if !line.is_empty() {
        output.push_str(line.trim_end());
    }
}

fn push_wrapped_token(token: &str, output: &mut String, line: &mut String) {
    if token.chars().count() <= 80 || is_angle_wrapped_autolink(token) || contains_url_prefix(token)
    {
        line.push_str(token);
        return;
    }
    for character in token.chars() {
        if line.chars().count() >= 80 {
            output.push_str(line);
            output.push('\n');
            line.clear();
        }
        line.push(character);
    }
}

fn write_wrapped_prefixed_text(text: &str, output: &mut String, prefix: &str, continuation: &str) {
    let mut line = prefix.to_string();
    for token in text.split_whitespace() {
        if line == prefix || line == continuation {
            line.push_str(token);
        } else if line.chars().count() + 1 + token.chars().count() <= 80
            || is_angle_wrapped_autolink(token)
            || contains_url_prefix(token)
        {
            line.push(' ');
            line.push_str(token);
        } else {
            output.push_str(line.trim_end());
            output.push('\n');
            line.clear();
            line.push_str(continuation);
            line.push_str(token);
        }
    }
    output.push_str(line.trim_end());
}

fn normalize_inline_text(text: &str) -> String {
    let tokens = repair_split_url_tokens(text.split_whitespace());
    let normalized = tokens
        .iter()
        .map(|token| angle_bracket_bare_link(token))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = angle_bracket_embedded_urls(&normalized);
    escape_markdown_ambiguities(&normalized)
}

fn repair_split_url_tokens<'a>(tokens: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut repaired = Vec::<String>::new();
    for token in tokens {
        if let Some(previous) = repaired.last_mut()
            && should_join_split_url(previous, token)
        {
            previous.push_str(token);
            continue;
        }
        repaired.push(token.to_string());
    }
    repaired
}

fn should_join_split_url(previous: &str, next: &str) -> bool {
    let next_core = next.strip_suffix('>').unwrap_or(next);
    contains_url_prefix(previous)
        && next_core.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(
                    character,
                    '/' | '.' | '_' | '-' | '~' | '?' | '#' | '&' | '=' | '%' | ':'
                )
        })
        && (previous.ends_with('/') || previous.ends_with(".h") || previous.ends_with('-'))
}

fn contains_url_prefix(text: &str) -> bool {
    text.contains("http://") || text.contains("https://") || text.contains("www.")
}

fn angle_bracket_bare_link(token: &str) -> String {
    let (prefix, core, suffix) = split_surrounding_punctuation(token);
    if is_angle_wrapped_autolink(core) {
        return token.to_string();
    }
    if core.starts_with("http://")
        || core.starts_with("https://")
        || core.starts_with("www.")
        || looks_like_email(core)
    {
        format!("{prefix}<{core}>{suffix}")
    } else if contains_url_prefix(token) {
        escape_inline_angle_brackets(token)
    } else if let Some(token) = angle_bracket_embedded_email(token) {
        token
    } else {
        escape_markdown_text_markers(&escape_inline_angle_brackets(token))
    }
}

fn angle_bracket_embedded_email(token: &str) -> Option<String> {
    let at = token.find('@')?;
    let start = token[..at]
        .char_indices()
        .rev()
        .find(|(_, character)| !is_email_local_character(*character))
        .map(|(index, character)| index + character.len_utf8())
        .unwrap_or(0);
    let end = token[at + 1..]
        .char_indices()
        .find(|(_, character)| !is_email_domain_character(*character))
        .map(|(index, _)| at + 1 + index)
        .unwrap_or(token.len());
    let candidate = &token[start..end];
    if !looks_like_email(candidate) {
        return None;
    }
    let mut wrapped = String::new();
    wrapped.push_str(&escape_markdown_text_markers(
        &escape_inline_angle_brackets(&token[..start]),
    ));
    wrapped.push('<');
    wrapped.push_str(candidate);
    wrapped.push('>');
    wrapped.push_str(&escape_markdown_text_markers(
        &escape_inline_angle_brackets(&token[end..]),
    ));
    Some(wrapped)
}

fn is_email_local_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '%' | '+' | '-')
}

fn is_email_domain_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '-')
}

fn escape_markdown_text_markers(token: &str) -> String {
    token
        .replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('`', "\\`")
}

fn is_angle_wrapped_autolink(token: &str) -> bool {
    let Some(inner) = token
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
    else {
        return false;
    };
    inner.starts_with("http://")
        || inner.starts_with("https://")
        || inner.starts_with("www.")
        || looks_like_email(inner)
}

fn escape_inline_angle_brackets(token: &str) -> String {
    if !(token.contains('<') || token.contains('>')) {
        return token.to_string();
    }
    token.replace('<', "&lt;").replace('>', "&gt;")
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

fn angle_bracket_embedded_urls(text: &str) -> String {
    let mut output = String::new();
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let Some(relative_start) = find_next_embedded_url_start(rest) else {
            output.push_str(rest);
            break;
        };
        let start = index + relative_start;
        output.push_str(&text[index..start]);
        if start > 0 && text[..start].ends_with('<') {
            output.push_str(&text[start..start + url_prefix_len(&text[start..])]);
            index = start + url_prefix_len(&text[start..]);
            continue;
        }
        let end = find_embedded_url_end(text, start);
        output.push('<');
        output.push_str(&text[start..end]);
        output.push('>');
        index = end;
    }
    output
}

fn find_next_embedded_url_start(text: &str) -> Option<usize> {
    ["http://", "https://", "www."]
        .iter()
        .filter_map(|prefix| text.find(prefix))
        .min()
}

fn url_prefix_len(text: &str) -> usize {
    if text.starts_with("https://") {
        "https://".len()
    } else if text.starts_with("http://") {
        "http://".len()
    } else {
        "www.".len()
    }
}

fn find_embedded_url_end(text: &str, start: usize) -> usize {
    for (offset, character) in text[start..].char_indices() {
        if character.is_whitespace()
            || matches!(
                character,
                '<' | '>' | '"' | '\'' | '“' | '”' | '‘' | '’' | ')' | '）'
            )
            || !character.is_ascii()
        {
            return start + offset;
        }
    }
    text.len()
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

fn ordered_list_marker_punctuation_index(text: &str) -> Option<usize> {
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

fn split_surrounding_punctuation(token: &str) -> (&str, &str, &str) {
    let prefix_len = token
        .char_indices()
        .find(|(_, character)| character.is_alphanumeric() || *character == 'h')
        .map(|(index, _)| index)
        .unwrap_or(0);
    let mut suffix_start = token.len();
    for (index, character) in token.char_indices().rev() {
        if character.is_alphanumeric() || matches!(character, '/' | '-') {
            suffix_start = index + character.len_utf8();
            break;
        }
    }
    if prefix_len >= suffix_start {
        return ("", token, "");
    }
    (
        &token[..prefix_len],
        &token[prefix_len..suffix_start],
        &token[suffix_start..],
    )
}

fn looks_like_email(token: &str) -> bool {
    let Some((local, domain)) = token.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && domain
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
}

fn write_table(rows: &[TableRow], flavor: Flavor, output: &mut String) {
    let requires_html = rows.iter().flat_map(|row| &row.cells).any(|cell| {
        cell.rowspan > 1 || cell.colspan > 1 || cell.image.is_some() || cell.text.contains('\n')
    }) || matches!(flavor, Flavor::CommonMark | Flavor::HedgeDoc);
    if requires_html {
        write_html_table(rows, output);
        return;
    }

    if rows.is_empty() {
        return;
    }
    let header = &rows[0];
    output.push('|');
    for cell in &header.cells {
        output.push(' ');
        output.push_str(&cell.text);
        output.push_str(" |");
    }
    output.push('\n');
    output.push('|');
    for _ in &header.cells {
        output.push_str(" --- |");
    }
    output.push('\n');
    for row in rows.iter().skip(1) {
        output.push('|');
        for cell in &row.cells {
            output.push(' ');
            output.push_str(&cell.text);
            output.push_str(" |");
        }
        output.push('\n');
    }
    output.push('\n');
}

fn write_html_table(rows: &[TableRow], output: &mut String) {
    output.push_str("<table>\n");
    for row in rows {
        output.push_str("<tr>");
        for cell in &row.cells {
            output.push_str("<td");
            if cell.rowspan > 1 {
                output.push_str(&format!(" rowspan=\"{}\"", cell.rowspan));
            }
            if cell.colspan > 1 {
                output.push_str(&format!(" colspan=\"{}\"", cell.colspan));
            }
            output.push('>');
            if let Some(path) = &cell.image {
                output.push_str("<img src=\"");
                output.push_str(&escape_html(path));
                output.push_str("\" alt=\"");
                output.push_str(&escape_html(&cell.text));
                output.push_str("\">");
            } else {
                output.push_str(&escape_html(&cell.text));
            }
            output.push_str("</td>");
        }
        output.push_str("</tr>\n");
    }
    output.push_str("</table>\n\n");
}
