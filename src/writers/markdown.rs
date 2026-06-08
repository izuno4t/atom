use crate::{AstNode, Flavor};

mod headings;
mod inline;
mod paragraphs;
mod tables;
mod wrap;

use headings::{
    HeadingState, normalize_heading_level, normalize_heading_text, should_promote_first_paragraph,
    split_long_heading, unique_heading_text,
};
use inline::normalize_inline_text;
use paragraphs::merge_interrupted_paragraphs;
use tables::write_table;
use wrap::{write_wrapped_prefixed_text, write_wrapped_text};

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
