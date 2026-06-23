use crate::{AstNode, ConversionOptions, LlmBackend, TableCell, TableRow};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct LlmRequest {
    pub backend: LlmBackend,
    pub task: String,
    pub input: String,
    pub images: Vec<LlmImage>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LlmResponse {
    pub text: String,
    pub backend: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LlmImage {
    pub mime_type: String,
    pub data_base64: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LlmSendConfirmation {
    pub destination: String,
    pub content_bytes: usize,
    pub consent_granted: bool,
    pub message: String,
}

pub trait LlmProvider {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse>;
}

pub struct DefaultLlmProvider;

impl LlmProvider for DefaultLlmProvider {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse> {
        match &request.backend {
            LlmBackend::Ollama(model) => complete_ollama(model, &request.input, &request.images),
            LlmBackend::OpenAi(model) => complete_openai(
                "https://api.openai.com/v1",
                model,
                &required_env("ATOM_OPENAI_API_KEY")?,
                &request.input,
                &request.images,
                "openai",
            ),
            LlmBackend::Anthropic(model) => complete_anthropic(
                model,
                &required_env("ATOM_ANTHROPIC_API_KEY")?,
                &request.input,
                &request.images,
            ),
            LlmBackend::Gemini(model) => complete_gemini(
                model,
                &required_env("ATOM_GEMINI_API_KEY")?,
                &request.input,
                &request.images,
            ),
            LlmBackend::OpenAiCompatible { name, endpoint } => {
                let endpoint = if endpoint.is_empty() {
                    required_env("ATOM_OPENAI_COMPATIBLE_ENDPOINT")?
                } else {
                    endpoint.clone()
                };
                let api_key = required_env("ATOM_OPENAI_COMPATIBLE_API_KEY")?;
                complete_openai(
                    &endpoint,
                    name,
                    &api_key,
                    &request.input,
                    &request.images,
                    "openai-compatible",
                )
            }
            LlmBackend::None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no LLM backend selected",
            )),
        }
    }
}

pub fn complete_with(provider: &dyn LlmProvider, request: &LlmRequest) -> io::Result<LlmResponse> {
    provider.complete(request)
}

pub fn backend_name(backend: &LlmBackend) -> &'static str {
    match backend {
        LlmBackend::None => "none",
        LlmBackend::Anthropic(_) => "anthropic",
        LlmBackend::Gemini(_) => "gemini",
        LlmBackend::OpenAi(_) => "openai",
        LlmBackend::Ollama(_) => "ollama",
        LlmBackend::OpenAiCompatible { .. } => "openai-compatible",
    }
}

pub fn build_send_confirmation(
    backend: &LlmBackend,
    content: &str,
    consent_granted: bool,
) -> Option<LlmSendConfirmation> {
    if matches!(backend, LlmBackend::None | LlmBackend::Ollama(_)) {
        return None;
    }
    let destination = match backend {
        LlmBackend::Anthropic(_) => "Anthropic".to_string(),
        LlmBackend::Gemini(_) => "Gemini".to_string(),
        LlmBackend::OpenAi(_) => "OpenAI".to_string(),
        LlmBackend::OpenAiCompatible { endpoint, name } if !endpoint.is_empty() => {
            endpoint.clone().to_string()
        }
        LlmBackend::OpenAiCompatible { name, .. } => name.clone(),
        LlmBackend::None | LlmBackend::Ollama(_) => unreachable!(),
    };
    Some(LlmSendConfirmation {
        destination: destination.clone(),
        content_bytes: content.len(),
        consent_granted,
        message: if consent_granted {
            format!(
                "external send consent granted for {destination}; {} byte(s) will be sent",
                content.len()
            )
        } else {
            format!(
                "external send consent is required for {destination}; {} byte(s) would be sent",
                content.len()
            )
        },
    })
}

pub fn restructure_with_provider(
    provider: &dyn LlmProvider,
    backend: &LlmBackend,
    ast: &[AstNode],
) -> io::Result<Vec<AstNode>> {
    run_markdown_transform(provider, backend, "restructure", ast, &BTreeMap::new())
}

pub fn translate_with_provider(
    provider: &dyn LlmProvider,
    backend: &LlmBackend,
    language: &str,
    ast: &[AstNode],
) -> io::Result<Vec<AstNode>> {
    run_markdown_transform(
        provider,
        backend,
        &format!("translate:{language}"),
        ast,
        &BTreeMap::new(),
    )
}

fn run_markdown_transform(
    provider: &dyn LlmProvider,
    backend: &LlmBackend,
    task: &str,
    ast: &[AstNode],
    prompts: &BTreeMap<String, String>,
) -> io::Result<Vec<AstNode>> {
    let markdown = render_markdown_for_llm(ast);
    let input = build_prompt(task, &markdown, prompts);
    let response = complete_with(
        provider,
        &LlmRequest {
            backend: backend.clone(),
            task: task.to_string(),
            input,
            images: Vec::new(),
        },
    )?;
    Ok(parse_markdown_blocks(&response.text))
}

fn build_prompt(task: &str, markdown: &str, prompts: &BTreeMap<String, String>) -> String {
    if let Some(template) = prompt_template_for_task(task, prompts) {
        return apply_prompt_template(template, task, markdown);
    }
    if task == "vision:describe" {
        return "Convert the attached image into concise, human-readable Markdown. Preserve visible text, figure labels, table-like structure, and captions. Do not invent content that is not visible. Return only Markdown.\n\n".to_string()
            + markdown;
    }
    if let Some(language) = task.strip_prefix("translate:") {
        return format!(
            "Translate the following Markdown to {language}. Preserve Markdown block boundaries, headings, lists, tables, images, links, and footnotes. Do not add content that is not present in the source. Return only Markdown.\n\n{markdown}"
        );
    }
    "Rewrite the following conversion result as readable Markdown. Preserve all source facts, keep existing headings/lists/tables/images/footnotes when present, and do not add content that is not present in the source. Return only Markdown.\n\n".to_string()
        + markdown
}

fn prompt_template_for_task<'a>(
    task: &str,
    prompts: &'a BTreeMap<String, String>,
) -> Option<&'a str> {
    if let Some(prompt) = prompts.get(task) {
        return Some(prompt);
    }
    if task.starts_with("translate:") {
        return prompts.get("translate").map(String::as_str);
    }
    if task == "vision:describe" {
        return prompts
            .get("image-description")
            .or_else(|| prompts.get("vision"))
            .map(String::as_str);
    }
    prompts.get("default").map(String::as_str)
}

fn apply_prompt_template(template: &str, task: &str, markdown: &str) -> String {
    let language = task.strip_prefix("translate:").unwrap_or("");
    let rendered = template
        .replace("{input}", markdown)
        .replace("{markdown}", markdown)
        .replace("{language}", language);
    if rendered == template {
        format!("{template}\n\n{markdown}")
    } else {
        rendered
    }
}

fn parse_markdown_blocks(markdown: &str) -> Vec<AstNode> {
    let mut nodes = Vec::new();
    let mut paragraph = Vec::new();
    let mut code_language = None::<String>;
    let mut code_lines = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim_end();
        if let Some(language) = &code_language {
            if trimmed == "```" {
                nodes.push(AstNode::CodeBlock {
                    language: (!language.is_empty()).then(|| language.clone()),
                    code: code_lines.join("\n"),
                });
                code_language = None;
                code_lines.clear();
            } else {
                code_lines.push(trimmed.to_string());
            }
            continue;
        }
        if let Some(language) = trimmed.strip_prefix("```") {
            flush_paragraph(&mut paragraph, &mut nodes);
            code_language = Some(language.trim().to_string());
            continue;
        }
        if trimmed.trim().is_empty() {
            flush_paragraph(&mut paragraph, &mut nodes);
            continue;
        }
        if let Some(node) = parse_heading(trimmed) {
            flush_paragraph(&mut paragraph, &mut nodes);
            nodes.push(node);
        } else if let Some(node) = parse_image(trimmed) {
            flush_paragraph(&mut paragraph, &mut nodes);
            nodes.push(node);
        } else if let Some(node) = parse_footnote(trimmed) {
            flush_paragraph(&mut paragraph, &mut nodes);
            nodes.push(node);
        } else if let Some(node) = parse_list_line(trimmed) {
            flush_paragraph(&mut paragraph, &mut nodes);
            merge_list_node(&mut nodes, node);
        } else if is_table_line(trimmed) {
            flush_paragraph(&mut paragraph, &mut nodes);
            merge_table_line(&mut nodes, trimmed);
        } else {
            paragraph.push(trimmed.to_string());
        }
    }
    flush_paragraph(&mut paragraph, &mut nodes);
    if let Some(language) = code_language {
        nodes.push(AstNode::CodeBlock {
            language: (!language.is_empty()).then_some(language),
            code: code_lines.join("\n"),
        });
    }
    nodes
}

fn flush_paragraph(lines: &mut Vec<String>, nodes: &mut Vec<AstNode>) {
    if !lines.is_empty() {
        nodes.push(AstNode::Paragraph(lines.join("\n")));
        lines.clear();
    }
}

fn parse_heading(line: &str) -> Option<AstNode> {
    let hashes = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    ((1..=6).contains(&hashes) && line.chars().nth(hashes) == Some(' ')).then(|| AstNode::Heading {
        level: hashes as u8,
        text: line[hashes + 1..].trim().to_string(),
    })
}

fn parse_image(line: &str) -> Option<AstNode> {
    let rest = line.strip_prefix("![")?;
    let (alt, rest) = rest.split_once("](")?;
    let destination = rest.strip_suffix(')')?.trim();
    let (path, title) = if let Some((path, title)) = destination.split_once(" \"") {
        (path.trim(), title.strip_suffix('"').map(str::to_string))
    } else {
        (destination, None)
    };
    Some(AstNode::Image {
        alt: alt.to_string(),
        path: path.trim_matches(['<', '>']).to_string(),
        title,
    })
}

fn parse_footnote(line: &str) -> Option<AstNode> {
    let rest = line.strip_prefix("[^")?;
    let (label, text) = rest.split_once("]:")?;
    Some(AstNode::Footnote {
        label: label.to_string(),
        text: text.trim().to_string(),
    })
}

fn parse_list_line(line: &str) -> Option<AstNode> {
    let (ordered, text) = if let Some(text) = line.strip_prefix("- ") {
        (false, text)
    } else if let Some((marker, text)) = line.split_once(". ") {
        if marker.chars().all(|character| character.is_ascii_digit()) {
            (true, text)
        } else {
            return None;
        }
    } else {
        return None;
    };
    Some(AstNode::List {
        ordered,
        items: vec![vec![AstNode::Text(text.trim().to_string())]],
    })
}

fn merge_list_node(nodes: &mut Vec<AstNode>, node: AstNode) {
    let AstNode::List { ordered, mut items } = node else {
        return;
    };
    if let Some(AstNode::List {
        ordered: previous_ordered,
        items: previous_items,
    }) = nodes.last_mut()
        && *previous_ordered == ordered
    {
        previous_items.append(&mut items);
        return;
    }
    nodes.push(AstNode::List { ordered, items });
}

fn is_table_line(line: &str) -> bool {
    line.starts_with('|') && line.ends_with('|') && line.matches('|').count() >= 2
}

fn merge_table_line(nodes: &mut Vec<AstNode>, line: &str) {
    if line.trim_matches('|').split('|').all(|cell| {
        cell.trim()
            .chars()
            .all(|character| character == '-' || character == ':')
    }) {
        return;
    }
    let row = TableRow {
        cells: line
            .trim_matches('|')
            .split('|')
            .map(|cell| TableCell {
                text: cell.trim().to_string(),
                rowspan: 1,
                colspan: 1,
                image: None,
            })
            .collect(),
    };
    if let Some(AstNode::Table { rows }) = nodes.last_mut() {
        rows.push(row);
    } else {
        nodes.push(AstNode::Table { rows: vec![row] });
    }
}

fn render_markdown_for_llm(ast: &[AstNode]) -> String {
    ast.iter()
        .map(|node| match node {
            AstNode::Heading { level, text } => format!("{} {text}", "#".repeat(*level as usize)),
            AstNode::Paragraph(text) | AstNode::Text(text) => text.clone(),
            AstNode::List { ordered, items } => items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let marker = if *ordered {
                        format!("{}. ", index + 1)
                    } else {
                        "- ".to_string()
                    };
                    format!("{marker}{}", render_markdown_for_llm(item))
                })
                .collect::<Vec<_>>()
                .join("\n"),
            AstNode::CodeBlock { language, code } => {
                format!("```{}\n{}\n```", language.as_deref().unwrap_or(""), code)
            }
            AstNode::Table { rows } => render_table_for_llm(rows),
            AstNode::Image { alt, path, title } => {
                if let Some(title) = title {
                    format!("![{alt}]({path} \"{title}\")")
                } else {
                    format!("![{alt}]({path})")
                }
            }
            AstNode::Footnote { label, text } => format!("[^{label}]: {text}"),
            AstNode::RawHtml(html) => html.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_table_for_llm(rows: &[TableRow]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let rendered_rows = rows
        .iter()
        .map(|row| {
            format!(
                "| {} |",
                row.cells
                    .iter()
                    .map(|cell| cell.text.replace('|', "\\|"))
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        })
        .collect::<Vec<_>>();
    let separator = format!(
        "| {} |",
        rows[0]
            .cells
            .iter()
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let mut output = Vec::with_capacity(rendered_rows.len() + 1);
    output.push(rendered_rows[0].clone());
    output.push(separator);
    output.extend(rendered_rows.into_iter().skip(1));
    output.join("\n")
}

pub fn save_diff(path: &Path, before: &str, after: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render_diff(before, after))
}

fn render_diff(before: &str, after: &str) -> String {
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let mut diff = String::from("--- before\n+++ after\n@@\n");
    let max_len = before_lines.len().max(after_lines.len());
    for index in 0..max_len {
        match (before_lines.get(index), after_lines.get(index)) {
            (Some(left), Some(right)) if left == right => {
                diff.push_str(&format!(" {left}\n"));
            }
            (Some(left), Some(right)) => {
                diff.push_str(&format!("-{left}\n+{right}\n"));
            }
            (Some(left), None) => diff.push_str(&format!("-{left}\n")),
            (None, Some(right)) => diff.push_str(&format!("+{right}\n")),
            (None, None) => {}
        }
    }
    diff
}

pub fn apply_llm_filters(
    ast: &mut Vec<AstNode>,
    options: &ConversionOptions,
    warnings: &mut Vec<String>,
) -> io::Result<()> {
    if options.llm == LlmBackend::None {
        warnings.push("LLM options requested but no LLM backend was selected.".to_string());
        return Ok(());
    }
    let content_preview = ast
        .iter()
        .map(|node| format!("{node:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    if let Some(confirmation) = build_send_confirmation(
        &options.llm,
        &content_preview,
        options.consent_external_send,
    ) {
        warnings.push(confirmation.message);
    }
    if !options.consent_external_send && !matches!(options.llm, LlmBackend::Ollama(_)) {
        warnings.push(
            "LLM filter skipped because external send consent is not configured.".to_string(),
        );
        return Ok(());
    }
    let provider = DefaultLlmProvider;
    if options.restructure {
        apply_one_filter(
            ast,
            &provider,
            &options.llm,
            "restructure",
            &options.llm_prompts,
            warnings,
        )?;
        enrich_images_with_vlm(ast, &provider, &options.llm, &options.llm_prompts, warnings)?;
    }
    if let Some(language) = &options.translate {
        apply_one_filter(
            ast,
            &provider,
            &options.llm,
            &format!("translate:{language}"),
            &options.llm_prompts,
            warnings,
        )?;
    }
    Ok(())
}

fn apply_one_filter(
    ast: &mut Vec<AstNode>,
    provider: &dyn LlmProvider,
    backend: &LlmBackend,
    task: &str,
    prompts: &BTreeMap<String, String>,
    warnings: &mut Vec<String>,
) -> io::Result<()> {
    let before_ast = ast.clone();
    let before = render_markdown_for_llm(&before_ast);
    let candidate = match run_markdown_transform(provider, backend, task, &before_ast, prompts) {
        Ok(candidate) => candidate,
        Err(error) => {
            warnings.push(format!("LLM {task} failed: {error}"));
            return Ok(());
        }
    };
    if let Some(reason) = validate_candidate(task, &before_ast, &candidate) {
        warnings.push(format!("LLM {task} response rejected: {reason}"));
        return Ok(());
    }
    let after = render_markdown_for_llm(&candidate);
    let safe_task = task.replace(':', "-");
    save_diff(
        Path::new("target")
            .join("atom-llm-diffs")
            .join(format!("{safe_task}.diff"))
            .as_path(),
        &before,
        &after,
    )?;
    *ast = candidate;
    warnings.push(format!("LLM {task} response accepted and diff saved."));
    Ok(())
}

pub fn describe_image_with_provider(
    provider: &dyn LlmProvider,
    backend: &LlmBackend,
    image: LlmImage,
    context: &str,
) -> io::Result<Vec<AstNode>> {
    describe_image_with_prompts(provider, backend, image, context, &BTreeMap::new())
}

pub fn describe_image_with_prompts(
    provider: &dyn LlmProvider,
    backend: &LlmBackend,
    image: LlmImage,
    context: &str,
    prompts: &BTreeMap<String, String>,
) -> io::Result<Vec<AstNode>> {
    let response = complete_with(
        provider,
        &LlmRequest {
            backend: backend.clone(),
            task: "vision:describe".to_string(),
            input: build_prompt("vision:describe", context, prompts),
            images: vec![image],
        },
    )?;
    Ok(parse_markdown_blocks(&response.text))
}

pub fn image_from_path(path: &Path) -> io::Result<LlmImage> {
    let bytes = fs::read(path)?;
    Ok(LlmImage {
        mime_type: image_mime_type(path),
        data_base64: encode_base64(&bytes),
        source: path.to_string_lossy().to_string(),
    })
}

fn enrich_images_with_vlm(
    ast: &mut [AstNode],
    provider: &dyn LlmProvider,
    backend: &LlmBackend,
    prompts: &BTreeMap<String, String>,
    warnings: &mut Vec<String>,
) -> io::Result<()> {
    for node in ast {
        match node {
            AstNode::Image { alt, path, title } if title.is_none() => {
                let image_path = Path::new(path);
                if image_path.exists() {
                    let image = image_from_path(image_path)?;
                    let context = format!("Existing image alt text: {alt}");
                    match describe_image_with_prompts(provider, backend, image, &context, prompts) {
                        Ok(nodes) if !nodes.is_empty() => {
                            let description = render_markdown_for_llm(&nodes);
                            if alt.trim().is_empty() {
                                *alt = description
                                    .lines()
                                    .next()
                                    .unwrap_or("image")
                                    .chars()
                                    .take(80)
                                    .collect();
                            }
                            *title = Some(description);
                            warnings.push(format!("VLM caption inferred for image {path}."));
                        }
                        Ok(_) => {
                            warnings.push(format!("VLM returned no caption for image {path}."))
                        }
                        Err(error) => {
                            warnings
                                .push(format!("VLM caption inference failed for {path}: {error}"));
                        }
                    }
                }
            }
            AstNode::List { items, .. } => {
                for item in items {
                    enrich_images_with_vlm(item, provider, backend, prompts, warnings)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_candidate(task: &str, before: &[AstNode], after: &[AstNode]) -> Option<String> {
    if after.is_empty() || render_markdown_for_llm(after).trim().is_empty() {
        return Some("empty Markdown response".to_string());
    }
    if task == "restructure" {
        let missing = missing_structural_kinds(before, after);
        if !missing.is_empty() {
            return Some(format!(
                "structured attributes missing from response: {}",
                missing.join(", ")
            ));
        }
        let before_headings = heading_count(before);
        let after_headings = heading_count(after);
        if before_headings > 1 && after_headings * 2 < before_headings {
            return Some("heading count dropped below half of the source".to_string());
        }
        let before_len = text_len(before);
        let after_len = text_len(after);
        if before_len > 0 && after_len * 2 < before_len {
            return Some("body text dropped below half of the source".to_string());
        }
    } else if task.starts_with("translate:") && before.len() > 1 && after.len() <= 1 {
        return Some("translated Markdown collapsed to one block".to_string());
    } else if task == "vision:describe" && !contains_descriptive_text(after) {
        return Some("image description did not contain descriptive text".to_string());
    }
    None
}

fn missing_structural_kinds(before: &[AstNode], after: &[AstNode]) -> Vec<&'static str> {
    ["list", "table", "image", "code", "footnote"]
        .into_iter()
        .filter(|kind| structural_count(before, kind) > 0 && structural_count(after, kind) == 0)
        .collect()
}

fn structural_count(ast: &[AstNode], kind: &str) -> usize {
    ast.iter()
        .map(|node| match node {
            AstNode::List { items, .. } => {
                usize::from(kind == "list")
                    + items
                        .iter()
                        .map(|item| structural_count(item, kind))
                        .sum::<usize>()
            }
            AstNode::Table { .. } => usize::from(kind == "table"),
            AstNode::Image { .. } => usize::from(kind == "image"),
            AstNode::CodeBlock { .. } => usize::from(kind == "code"),
            AstNode::Footnote { .. } => usize::from(kind == "footnote"),
            _ => 0,
        })
        .sum()
}

fn contains_descriptive_text(ast: &[AstNode]) -> bool {
    ast.iter().any(|node| match node {
        AstNode::Heading { text, .. } | AstNode::Paragraph(text) | AstNode::Text(text) => {
            !text.trim().is_empty()
        }
        AstNode::List { items, .. } => items.iter().any(|item| contains_descriptive_text(item)),
        AstNode::Table { rows } => rows
            .iter()
            .flat_map(|row| row.cells.iter())
            .any(|cell| !cell.text.trim().is_empty()),
        AstNode::Image { alt, title, .. } => {
            !alt.trim().is_empty()
                || title
                    .as_deref()
                    .is_some_and(|title| !title.trim().is_empty())
        }
        AstNode::CodeBlock { code, .. } | AstNode::Footnote { text: code, .. } => {
            !code.trim().is_empty()
        }
        AstNode::RawHtml(html) => !html.trim().is_empty(),
    })
}

fn heading_count(ast: &[AstNode]) -> usize {
    ast.iter()
        .map(|node| match node {
            AstNode::Heading { .. } => 1,
            AstNode::List { items, .. } => items.iter().map(|item| heading_count(item)).sum(),
            _ => 0,
        })
        .sum()
}

fn text_len(ast: &[AstNode]) -> usize {
    render_markdown_for_llm(ast)
        .chars()
        .filter(|character| !character.is_whitespace())
        .count()
}

fn complete_ollama(model: &str, prompt: &str, images: &[LlmImage]) -> io::Result<LlmResponse> {
    let body = json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "images": images.iter().map(|image| image.data_base64.clone()).collect::<Vec<_>>()
    })
    .to_string();
    let response = post_json(
        "http://127.0.0.1:11434/api/generate",
        &[],
        &body,
        Duration::from_secs(120),
    )?;
    let value: serde_json::Value = serde_json::from_str(&response).map_err(io::Error::other)?;
    let text = value
        .get("response")
        .and_then(|value| value.as_str())
        .ok_or_else(|| io::Error::other("Ollama response did not contain response text"))?;
    Ok(LlmResponse {
        text: text.to_string(),
        backend: "ollama".to_string(),
    })
}

fn complete_openai(
    endpoint: &str,
    model: &str,
    api_key: &str,
    prompt: &str,
    images: &[LlmImage],
    backend_name: &str,
) -> io::Result<LlmResponse> {
    let body = openai_chat_body(model, prompt, images).to_string();
    let response = post_json(
        &chat_completion_url(endpoint),
        &[("Authorization", format!("Bearer {api_key}"))],
        &body,
        Duration::from_secs(120),
    )?;
    Ok(LlmResponse {
        text: extract_openai_text(&response)?,
        backend: backend_name.to_string(),
    })
}

fn complete_anthropic(
    model: &str,
    api_key: &str,
    prompt: &str,
    images: &[LlmImage],
) -> io::Result<LlmResponse> {
    let body = anthropic_messages_body(model, prompt, images).to_string();
    let response = post_json(
        "https://api.anthropic.com/v1/messages",
        &[
            ("x-api-key", api_key.to_string()),
            (
                "anthropic-version",
                std::env::var("ANTHROPIC_VERSION").unwrap_or_else(|_| "2023-06-01".to_string()),
            ),
        ],
        &body,
        Duration::from_secs(120),
    )?;
    Ok(LlmResponse {
        text: extract_anthropic_text(&response)?,
        backend: "anthropic".to_string(),
    })
}

fn complete_gemini(
    model: &str,
    api_key: &str,
    prompt: &str,
    images: &[LlmImage],
) -> io::Result<LlmResponse> {
    let body = gemini_interactions_body(model, prompt, images).to_string();
    let response = post_json(
        "https://generativelanguage.googleapis.com/v1beta/interactions",
        &[("x-goog-api-key", api_key.to_string())],
        &body,
        Duration::from_secs(120),
    )?;
    Ok(LlmResponse {
        text: extract_gemini_text(&response)?,
        backend: "gemini".to_string(),
    })
}

fn post_json(
    url: &str,
    headers: &[(&str, String)],
    body: &str,
    timeout: Duration,
) -> io::Result<String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build();
    let agent = config.new_agent();
    let mut request = agent
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json");
    for (name, value) in headers {
        request = request.header(*name, value.as_str());
    }
    request
        .send(body)
        .map_err(|error| io::Error::other(error.to_string()))?
        .body_mut()
        .read_to_string()
        .map_err(io::Error::other)
}

fn openai_chat_body(model: &str, prompt: &str, images: &[LlmImage]) -> serde_json::Value {
    if images.is_empty() {
        return json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
        });
    }
    let mut content = vec![json!({"type": "text", "text": prompt})];
    content.extend(images.iter().map(|image| {
        json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:{};base64,{}", image.mime_type, image.data_base64),
                "detail": "auto"
            }
        })
    }));
    json!({
        "model": model,
        "messages": [{"role": "user", "content": content}],
    })
}

fn anthropic_messages_body(model: &str, prompt: &str, images: &[LlmImage]) -> serde_json::Value {
    let mut content = Vec::new();
    for image in images {
        content.push(json!({
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": image.mime_type,
                "data": image.data_base64
            }
        }));
    }
    content.push(json!({"type": "text", "text": prompt}));
    json!({
        "model": model,
        "max_tokens": 2048,
        "messages": [{"role": "user", "content": content}],
    })
}

fn gemini_interactions_body(model: &str, prompt: &str, images: &[LlmImage]) -> serde_json::Value {
    if images.is_empty() {
        return json!({
            "model": model,
            "input": prompt,
        });
    }
    let mut input = vec![json!({"type": "text", "text": prompt})];
    input.extend(images.iter().map(|image| {
        json!({
            "type": "image",
            "data": image.data_base64,
            "mime_type": image.mime_type,
        })
    }));
    json!({
        "model": model,
        "input": input,
    })
}

fn extract_openai_text(response: &str) -> io::Result<String> {
    let value: serde_json::Value = serde_json::from_str(response).map_err(io::Error::other)?;
    let content = &value["choices"][0]["message"]["content"];
    if let Some(text) = content.as_str() {
        return Ok(text.to_string());
    }
    if let Some(parts) = content.as_array() {
        let text = parts
            .iter()
            .filter_map(|part| part.get("text").and_then(|text| text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            return Ok(text);
        }
    }
    Err(io::Error::other(
        "OpenAI response did not contain message content",
    ))
}

fn extract_gemini_text(response: &str) -> io::Result<String> {
    let value: serde_json::Value = serde_json::from_str(response).map_err(io::Error::other)?;
    for key in ["output_text", "text"] {
        if let Some(text) = value.get(key).and_then(|value| value.as_str()) {
            return Ok(text.to_string());
        }
    }
    let mut text = Vec::new();
    collect_text_values(&value, &mut text);
    if text.is_empty() {
        Err(io::Error::other(
            "Gemini response did not contain text content",
        ))
    } else {
        Ok(text.join("\n"))
    }
}

fn collect_text_values<'a>(value: &'a serde_json::Value, output: &mut Vec<&'a str>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(|value| value.as_str()) {
                output.push(text);
            }
            for value in object.values() {
                collect_text_values(value, output);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_text_values(value, output);
            }
        }
        _ => {}
    }
}

fn extract_anthropic_text(response: &str) -> io::Result<String> {
    let value: serde_json::Value = serde_json::from_str(response).map_err(io::Error::other)?;
    let text = value["content"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(|part| {
            (part.get("type").and_then(|value| value.as_str()) == Some("text"))
                .then(|| part.get("text").and_then(|value| value.as_str()))
                .flatten()
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() {
        Err(io::Error::other(
            "Anthropic response did not contain text content",
        ))
    } else {
        Ok(text)
    }
}

fn chat_completion_url(endpoint: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if endpoint.ends_with("/chat/completions") {
        endpoint.to_string()
    } else {
        format!("{endpoint}/chat/completions")
    }
}

fn required_env(name: &str) -> io::Result<String> {
    std::env::var(name).map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{name} must be set for selected LLM backend"),
        )
    })
}

fn image_mime_type(path: &Path) -> String {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png",
    }
    .to_string()
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_chat_body_uses_text_content_without_images() {
        let body = openai_chat_body("gpt-4o-mini", "hello", &[]);

        assert_eq!(body["model"], "gpt-4o-mini");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "hello");
    }

    #[test]
    fn openai_chat_body_sends_images_as_data_urls() {
        let body = openai_chat_body(
            "gpt-4o",
            "describe",
            &[LlmImage {
                mime_type: "image/png".to_string(),
                data_base64: "AAE=".to_string(),
                source: "chart.png".to_string(),
            }],
        );

        assert_eq!(body["messages"][0]["content"][0]["type"], "text");
        assert_eq!(body["messages"][0]["content"][1]["type"], "image_url");
        assert_eq!(
            body["messages"][0]["content"][1]["image_url"]["url"],
            "data:image/png;base64,AAE="
        );
    }

    #[test]
    fn anthropic_messages_body_sends_base64_images() {
        let body = anthropic_messages_body(
            "claude-sonnet",
            "describe",
            &[LlmImage {
                mime_type: "image/jpeg".to_string(),
                data_base64: "AAE=".to_string(),
                source: "scan.jpg".to_string(),
            }],
        );

        assert_eq!(body["messages"][0]["content"][0]["type"], "image");
        assert_eq!(
            body["messages"][0]["content"][0]["source"]["media_type"],
            "image/jpeg"
        );
        assert_eq!(body["messages"][0]["content"][1]["type"], "text");
    }

    #[test]
    fn gemini_interactions_body_sends_text_and_inline_images() {
        let body = gemini_interactions_body(
            "gemini-2.5-flash",
            "describe",
            &[LlmImage {
                mime_type: "image/png".to_string(),
                data_base64: "AAE=".to_string(),
                source: "chart.png".to_string(),
            }],
        );

        assert_eq!(body["model"], "gemini-2.5-flash");
        assert_eq!(body["input"][0]["type"], "text");
        assert_eq!(body["input"][1]["type"], "image");
        assert_eq!(body["input"][1]["mime_type"], "image/png");
        assert_eq!(body["input"][1]["data"], "AAE=");
    }

    #[test]
    fn extracts_text_from_provider_responses() {
        let openai = r##"{"choices":[{"message":{"content":"# Title\n\nBody"}}]}"##;
        let anthropic =
            r##"{"content":[{"type":"text","text":"# Title"},{"type":"text","text":"Body"}]}"##;
        let gemini = r##"{"output":[{"content":[{"type":"text","text":"# Title"},{"type":"text","text":"Body"}]}]}"##;

        assert_eq!(extract_openai_text(openai).unwrap(), "# Title\n\nBody");
        assert_eq!(extract_anthropic_text(anthropic).unwrap(), "# Title\nBody");
        assert_eq!(extract_gemini_text(gemini).unwrap(), "# Title\nBody");
    }
}
