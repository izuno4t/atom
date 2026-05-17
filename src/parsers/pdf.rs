use crate::AstNode;

static PDF_EXTRACT_PANIC_HOOK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn parse_pdf(bytes: &[u8], warnings: &mut Vec<String>) -> Vec<AstNode> {
    parse_pdf_with_backend(bytes, &InternalPdfTextBackend, warnings).ast
}

pub fn parse_pdf_with_embedded_backend(bytes: &[u8], warnings: &mut Vec<String>) -> PdfParseResult {
    let primary = parse_pdf_with_backend(bytes, &PdfExtractBackend, warnings);
    if !primary.extraction_failed && !primary.ocr_required {
        return primary;
    }

    let fallback = parse_pdf_with_backend(bytes, &InternalPdfTextBackend, warnings);
    let fallback_has_text = fallback.ast.iter().any(|node| match node {
        AstNode::Paragraph(text) | AstNode::Text(text) => {
            !text.starts_with("PDF text extraction produced no text")
        }
        AstNode::Heading { .. } => true,
        _ => true,
    });
    if fallback_has_text { fallback } else { primary }
}

pub fn parse_pdf_with_backend(
    bytes: &[u8],
    backend: &dyn PdfTextBackend,
    warnings: &mut Vec<String>,
) -> PdfParseResult {
    let lossy = String::from_utf8_lossy(bytes);
    warnings.push(
        "PDF parser extracts text objects; coordinates and layout inference are limited."
            .to_string(),
    );
    if lossy.contains("/StructTreeRoot") {
        warnings.push(
            "PDF tagged structure detected; logical reading order should be preferred.".to_string(),
        );
    } else {
        warnings.push(
            "PDF tag tree was not detected; falling back to content stream order.".to_string(),
        );
    }
    let extraction = backend.extract_text(bytes);
    let ocr_required = extraction.ocr_required || extraction.objects.is_empty();
    let ast = if extraction.objects.is_empty() {
        let message = format!(
            "PDF text extraction produced no text with backend {}. A full PDF backend or OCR may be required.",
            backend.name()
        );
        warnings.push(message.clone());
        vec![AstNode::Paragraph(message)]
    } else {
        infer_nodes_from_text_objects(extraction.objects, warnings)
    };
    PdfParseResult {
        ast,
        backend: backend.name().to_string(),
        extraction_failed: extraction.extraction_failed,
        ocr_required,
    }
}

#[derive(Clone, Debug)]
pub struct PdfTextObject {
    pub text: String,
    pub font_size: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct PdfTextExtraction {
    pub objects: Vec<PdfTextObject>,
    pub extraction_failed: bool,
    pub ocr_required: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PdfParseResult {
    pub ast: Vec<AstNode>,
    pub backend: String,
    pub extraction_failed: bool,
    pub ocr_required: bool,
}

pub trait PdfTextBackend {
    fn name(&self) -> &str;
    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction;
}

pub struct InternalPdfTextBackend;

pub struct PdfExtractBackend;

impl PdfTextBackend for PdfExtractBackend {
    fn name(&self) -> &str {
        "pdf-extract"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let extracted = extract_text_from_mem_catching_panics(bytes);
        match extracted {
            Ok(Ok(text)) => {
                let objects = text
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(|line| PdfTextObject {
                        text: line.to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    })
                    .collect::<Vec<_>>();
                PdfTextExtraction {
                    ocr_required: objects.is_empty(),
                    objects,
                    extraction_failed: false,
                }
            }
            Ok(Err(_)) | Err(_) => PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            },
        }
    }
}

fn extract_text_from_mem_catching_panics(
    bytes: &[u8],
) -> Result<Result<String, pdf_extract::OutputError>, Box<dyn std::any::Any + Send>> {
    let _guard = PDF_EXTRACT_PANIC_HOOK_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(bytes));
    std::panic::set_hook(previous_hook);
    result
}

impl PdfTextBackend for InternalPdfTextBackend {
    fn name(&self) -> &str {
        "internal-text-objects"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let lossy = String::from_utf8_lossy(bytes);
        let objects = extract_text_objects(&lossy);
        PdfTextExtraction {
            ocr_required: objects.is_empty(),
            objects,
            extraction_failed: false,
        }
    }
}

fn extract_text_objects(input: &str) -> Vec<PdfTextObject> {
    let mut objects = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find("BT") {
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("ET") else {
            break;
        };
        let block = &after_start[..end];
        if is_probably_text_block(block) {
            objects.extend(extract_block_text_objects(block));
        }
        rest = &after_start[end + 2..];
    }
    objects
}

fn is_probably_text_block(block: &str) -> bool {
    let char_count = block.chars().count().max(1);
    let replacement_count = block
        .chars()
        .filter(|character| *character == '\u{fffd}')
        .count();
    let control_count = block
        .chars()
        .filter(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        .count();
    (replacement_count <= 20 || replacement_count * 20 <= char_count)
        && control_count * 10 <= char_count
}

fn extract_block_text_objects(block: &str) -> Vec<PdfTextObject> {
    let mut objects = Vec::new();
    let mut current_font_size = None;
    let mut current_x = None;
    let mut current_y = None;
    for line in block.lines() {
        if let Some(font_size) = parse_font_size(line) {
            current_font_size = Some(font_size);
        }
        if let Some((x, y)) = parse_text_position(line) {
            current_x = Some(x);
            current_y = Some(y);
        }
        let text = extract_pdf_string_tokens(line).trim().to_string();
        if !text.is_empty() {
            objects.push(PdfTextObject {
                text,
                font_size: current_font_size,
                x: current_x,
                y: current_y,
            });
        }
    }
    objects
}

fn parse_font_size(line: &str) -> Option<f32> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let tf_index = tokens.iter().position(|token| *token == "Tf")?;
    if tf_index == 0 {
        return None;
    }
    tokens.get(tf_index - 1)?.parse::<f32>().ok()
}

fn parse_text_position(line: &str) -> Option<(f32, f32)> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let td_index = tokens
        .iter()
        .position(|token| matches!(*token, "Td" | "TD"))?;
    if td_index < 2 {
        return None;
    }
    let x = tokens.get(td_index - 2)?.parse::<f32>().ok()?;
    let y = tokens.get(td_index - 1)?.parse::<f32>().ok()?;
    Some((x, y))
}

fn extract_pdf_string_tokens(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '(' => output.push_str(&read_literal_string(&mut chars)),
            '<' if chars.peek() != Some(&'<') => {
                let hex = read_hex_string(&mut chars);
                output.push_str(&decode_hex_string(&hex));
            }
            _ => {}
        }
    }
    output
}

fn read_literal_string<I>(chars: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut value = String::new();
    let mut escaped = false;
    for next in chars.by_ref() {
        if escaped {
            value.push(match next {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                '(' | ')' | '\\' => next,
                other => other,
            });
            escaped = false;
        } else if next == '\\' {
            escaped = true;
        } else if next == ')' {
            break;
        } else {
            value.push(next);
        }
    }
    value
}

fn read_hex_string<I>(chars: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut value = String::new();
    for next in chars.by_ref() {
        if next == '>' {
            break;
        }
        if !next.is_whitespace() {
            value.push(next);
        }
    }
    value
}

fn decode_hex_string(hex: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = hex.chars().filter(|character| {
        character.is_ascii_digit() || matches!(character, 'a'..='f' | 'A'..='F')
    });
    while let Some(high) = chars.next() {
        let low = chars.next().unwrap_or('0');
        let pair = format!("{high}{low}");
        if let Ok(byte) = u8::from_str_radix(&pair, 16) {
            bytes.push(byte);
        }
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode_utf16be(&bytes[2..]);
    }
    if bytes.len() >= 2 && bytes.iter().step_by(2).all(|byte| *byte == 0) {
        return decode_utf16be(&bytes);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn decode_utf16be(bytes: &[u8]) -> String {
    let units = bytes
        .chunks(2)
        .filter(|chunk| chunk.len() == 2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&units)
}

fn infer_nodes_from_text_objects(
    objects: Vec<PdfTextObject>,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    let original_count = objects.len();
    let objects = objects
        .into_iter()
        .filter(|object| is_probably_human_text(&object.text))
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

fn infer_pdf_block_structure(nodes: Vec<AstNode>, warnings: &mut Vec<String>) -> Vec<AstNode> {
    let mut output = Vec::new();
    let mut pending_list: Option<(bool, Vec<Vec<AstNode>>)> = None;

    for node in nodes {
        let AstNode::Paragraph(text) = node else {
            flush_pending_pdf_list(&mut output, &mut pending_list, warnings);
            output.push(node);
            continue;
        };

        if let Some((ordered, item)) = parse_pdf_list_item(&text) {
            match &mut pending_list {
                Some((current_ordered, items)) if *current_ordered == ordered => {
                    items.push(vec![AstNode::Text(item)]);
                }
                _ => {
                    flush_pending_pdf_list(&mut output, &mut pending_list, warnings);
                    pending_list = Some((ordered, vec![vec![AstNode::Text(item)]]));
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
    output
}

fn flush_pending_pdf_list(
    output: &mut Vec<AstNode>,
    pending_list: &mut Option<(bool, Vec<Vec<AstNode>>)>,
    warnings: &mut Vec<String>,
) {
    if let Some((ordered, items)) = pending_list.take() {
        if items.len() >= 2 {
            warnings.push(format!(
                "PDF list inference grouped {} item(s).",
                items.len()
            ));
            output.push(AstNode::List { ordered, items });
        } else if let Some(item) = items.into_iter().next() {
            let text = item
                .into_iter()
                .filter_map(|node| match node {
                    AstNode::Text(text) => Some(text),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");
            let prefix = if ordered { "1. " } else { "- " };
            output.push(AstNode::Paragraph(format!("{prefix}{text}")));
        }
    }
}

fn parse_pdf_list_item(text: &str) -> Option<(bool, String)> {
    let trimmed = text.trim();
    for marker in ["- ", "• ", "・ "] {
        if let Some(item) = trimmed.strip_prefix(marker) {
            return Some((false, item.trim().to_string()));
        }
    }
    let (number, rest) = trimmed.split_once(". ")?;
    if !number.is_empty() && number.chars().all(|character| character.is_ascii_digit()) {
        return Some((true, rest.trim().to_string()));
    }
    None
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
    let first = trimmed.split_whitespace().next()?;
    if first
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        let dot_count = first.chars().filter(|character| *character == '.').count();
        if first.trim_matches('.').is_empty() || dot_count > 5 {
            return None;
        }
        if first.ends_with('.') {
            return None;
        }
        let has_digit = first.chars().any(|character| character.is_ascii_digit());
        if has_digit {
            return Some((dot_count + 1).min(6) as u8);
        }
    }
    None
}

fn is_probably_human_text(text: &str) -> bool {
    if text
        .chars()
        .filter(|character| *character == '\u{fffd}')
        .count()
        >= 2
    {
        return false;
    }
    let char_count = text.chars().count();
    if char_count == 0 {
        return false;
    }
    let control_count = text
        .chars()
        .filter(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        .count();
    if control_count * 4 > char_count {
        return false;
    }
    if char_count > 4000 && !text.chars().any(|character| character.is_whitespace()) {
        return false;
    }
    true
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
