use crate::AstNode;

mod raw_content;

pub use raw_content::RawContentTextBackend;

static PDF_EXTRACT_PANIC_HOOK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn parse_pdf(bytes: &[u8], warnings: &mut Vec<String>) -> Vec<AstNode> {
    parse_pdf_with_backend(bytes, &InternalPdfTextBackend, warnings).ast
}

pub fn is_encrypted_pdf(bytes: &[u8]) -> bool {
    lopdf::Document::load_mem(bytes)
        .map(|document| document.is_encrypted())
        .unwrap_or_else(|_| String::from_utf8_lossy(bytes).contains("/Encrypt"))
}

pub fn pdf_security_description(bytes: &[u8]) -> &'static str {
    let lossy = String::from_utf8_lossy(bytes);
    if lossy.contains("/Filter/Standard") && lossy.contains("/P ") {
        "the PDF has a Standard security handler and may be permission-restricted"
    } else {
        "the PDF is encrypted"
    }
}

pub fn diagnose_no_extractable_text(bytes: &[u8]) -> PdfNoTextDiagnosis {
    let lossy = String::from_utf8_lossy(bytes);
    let has_image = contains_pdf_name(&lossy, "/Subtype", "Image")
        || lossy.contains("/Subtype/Image")
        || lossy.contains("/ImageB")
        || lossy.contains("/ImageC")
        || lossy.contains("/ImageI");
    let has_font = lossy.contains("/Font");
    let has_text_procset = lossy.contains("/PDF/Text") || lossy.contains("/PDF /Text");
    let has_to_unicode = lossy.contains("/ToUnicode");

    if has_image && !has_font && !has_text_procset {
        PdfNoTextDiagnosis::ImageOnly
    } else if (has_font || has_text_procset) && (!has_to_unicode || has_unmapped_cid_fonts(bytes)) {
        PdfNoTextDiagnosis::MissingUnicodeMaps
    } else {
        PdfNoTextDiagnosis::Unknown
    }
}

pub fn parse_pdf_with_embedded_backend(bytes: &[u8], warnings: &mut Vec<String>) -> PdfParseResult {
    if is_encrypted_pdf(bytes) {
        let backends: [&dyn PdfTextBackend; 5] = [
            &RawContentTextBackend,
            &LenientPdfExtractBackend,
            &PdfExtractBackend,
            &LopdfTextBackend,
            &InternalPdfTextBackend,
        ];
        parse_pdf_with_ordered_backends(bytes, &backends, warnings)
    } else {
        let backends: [&dyn PdfTextBackend; 5] = [
            &PdfExtractBackend,
            &LopdfTextBackend,
            &LenientPdfExtractBackend,
            &RawContentTextBackend,
            &InternalPdfTextBackend,
        ];
        parse_pdf_with_ordered_backends(bytes, &backends, warnings)
    }
}

pub fn parse_pdf_with_ordered_backends(
    bytes: &[u8],
    backends: &[&dyn PdfTextBackend],
    warnings: &mut Vec<String>,
) -> PdfParseResult {
    let mut best_result = None;
    for backend in backends {
        let result = parse_pdf_with_backend(bytes, *backend, warnings);
        if pdf_result_has_text(&result) && !result.ocr_required {
            return result;
        }
        if best_result
            .as_ref()
            .is_none_or(|best| pdf_result_score(&result) > pdf_result_score(best))
        {
            best_result = Some(result);
        }
    }
    best_result.unwrap_or_else(|| parse_pdf_with_backend(bytes, &InternalPdfTextBackend, warnings))
}

pub fn extract_lopdf_page_texts(bytes: &[u8]) -> Option<Vec<Vec<PdfTextObject>>> {
    let document = lopdf::Document::load_mem(bytes).ok()?;
    if document.is_encrypted() {
        return None;
    }
    let pages = document.get_pages().keys().copied().collect::<Vec<_>>();
    Some(
        pages
            .into_iter()
            .map(|page| {
                document
                    .extract_text(&[page])
                    .ok()
                    .map(|text| {
                        text.lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty())
                            .map(|line| PdfTextObject {
                                text: line.to_string(),
                                font_size: None,
                                x: None,
                                y: None,
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            })
            .collect(),
    )
}

pub fn infer_nodes_from_pdf_text_objects(
    objects: Vec<PdfTextObject>,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    infer_nodes_from_text_objects(objects, warnings)
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
    let mut ocr_required = extraction.ocr_required || extraction.objects.is_empty();
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
    if !ocr_required && pdf_text_looks_incomplete(bytes, &ast) {
        ocr_required = true;
        warnings.push(
            "PDF text extraction appears incomplete because CID fonts lack Unicode maps; OCR is required."
                .to_string(),
        );
    }
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

pub struct LenientPdfExtractBackend;

pub struct LopdfTextBackend;

type PdfPendingListItem = (String, Vec<AstNode>);
type PdfPendingList = Option<(bool, Vec<PdfPendingListItem>)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdfNoTextDiagnosis {
    ImageOnly,
    MissingUnicodeMaps,
    Unknown,
}

impl PdfNoTextDiagnosis {
    pub fn message(self) -> &'static str {
        match self {
            Self::ImageOnly => {
                "PDF contains page images but no extractable text layer. OCR is required."
            }
            Self::MissingUnicodeMaps => {
                "PDF text uses embedded fonts without Unicode maps, so glyphs cannot be converted back to text."
            }
            Self::Unknown => {
                "PDF text extraction failed for a non-encrypted PDF; cause could not be classified."
            }
        }
    }
}

impl PdfTextBackend for PdfExtractBackend {
    fn name(&self) -> &str {
        "pdf-extract"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let extracted = extract_text_from_mem_catching_panics(bytes);
        match extracted {
            Ok(Ok(text)) => pdf_text_extraction_from_plain_text(text),
            Ok(Err(_)) | Err(_) => PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            },
        }
    }
}

fn pdf_text_extraction_from_plain_text(text: String) -> PdfTextExtraction {
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

impl PdfTextBackend for LenientPdfExtractBackend {
    fn name(&self) -> &str {
        "pdf-extract-lenient"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let document = match lopdf::Document::load_mem(bytes) {
            Ok(document) => document,
            Err(_) => {
                return PdfTextExtraction {
                    objects: Vec::new(),
                    extraction_failed: true,
                    ocr_required: true,
                };
            }
        };
        let mut text = String::new();
        let extracted = catch_pdf_extract_panic(std::panic::AssertUnwindSafe(|| {
            let mut output = pdf_extract::PlainTextOutput::new(&mut text);
            pdf_extract::output_doc(&document, &mut output)
        }));
        match extracted {
            Ok(Ok(())) => pdf_text_extraction_from_plain_text(text),
            Ok(Err(_)) | Err(_) => PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            },
        }
    }
}

fn contains_pdf_name(input: &str, key: &str, value: &str) -> bool {
    input
        .match_indices(key)
        .any(|(index, _)| input[index + key.len()..].trim_start().starts_with(value))
}

impl PdfTextBackend for LopdfTextBackend {
    fn name(&self) -> &str {
        "lopdf"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let mut document = match lopdf::Document::load_mem(bytes) {
            Ok(document) => document,
            Err(_) => {
                return PdfTextExtraction {
                    objects: Vec::new(),
                    extraction_failed: true,
                    ocr_required: true,
                };
            }
        };

        if document.is_encrypted() && document.decrypt("").is_err() {
            return PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            };
        }

        let pages = document.get_pages().keys().copied().collect::<Vec<_>>();
        match document.extract_text(&pages) {
            Ok(text) => {
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
            Err(_) => PdfTextExtraction {
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
    catch_pdf_extract_panic(|| pdf_extract::extract_text_from_mem(bytes))
}

fn catch_pdf_extract_panic<R, F>(operation: F) -> Result<R, Box<dyn std::any::Any + Send>>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    let _guard = PDF_EXTRACT_PANIC_HOOK_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(operation);
    std::panic::set_hook(previous_hook);
    result
}

fn pdf_result_has_text(result: &PdfParseResult) -> bool {
    result.ast.iter().any(|node| match node {
        AstNode::Paragraph(text) | AstNode::Text(text) => {
            !text.starts_with("PDF text extraction produced no text")
        }
        AstNode::Heading { .. } => true,
        _ => true,
    })
}

fn pdf_result_score(result: &PdfParseResult) -> usize {
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
    score
}

fn pdf_text_looks_incomplete(bytes: &[u8], ast: &[AstNode]) -> bool {
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

fn has_unmapped_cid_fonts(bytes: &[u8]) -> bool {
    let lossy = String::from_utf8_lossy(bytes);
    let type0_count =
        lossy.matches("/Subtype /Type0").count() + lossy.matches("/Subtype/Type0").count();
    if type0_count == 0 {
        return false;
    }
    let to_unicode_count = lossy.matches("/ToUnicode").count();
    type0_count > to_unicode_count
        && (lossy.contains("Hira")
            || lossy.contains("Heisei")
            || lossy.contains("YuGothic")
            || lossy.contains("YuMincho")
            || lossy.contains("KozMin")
            || lossy.contains("Gothic")
            || lossy.contains("Mincho"))
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

fn infer_pdf_block_structure(nodes: Vec<AstNode>, warnings: &mut Vec<String>) -> Vec<AstNode> {
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
            if ordered
                && items.iter().all(|(marker, _)| marker == "1")
                && items.iter().all(|(marker, item)| {
                    pdf_section_heading_level(&pdf_list_item_paragraph(ordered, marker, item))
                        .is_some()
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
        } else if let Some(item) = items.into_iter().next() {
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

fn is_pdf_repeated_noise_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.chars().all(|character| character.is_ascii_digit()) && trimmed.chars().count() <= 4 {
        return true;
    }
    if trimmed.contains('©') || trimmed.to_ascii_lowercase().contains("copyright") {
        return true;
    }
    if looks_like_pdf_date_footer(trimmed) {
        return true;
    }
    false
}

fn looks_like_pdf_date_footer(text: &str) -> bool {
    let mut parts = text.split_whitespace();
    let Some(date) = parts.next() else {
        return false;
    };
    let Some(time) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && date.len() == 10
        && date.chars().nth(4) == Some('/')
        && date.chars().nth(7) == Some('/')
        && time.len() == 5
        && time.chars().nth(2) == Some(':')
        && date
            .chars()
            .filter(|character| *character != '/')
            .all(|character| character.is_ascii_digit())
        && time
            .chars()
            .filter(|character| *character != ':')
            .all(|character| character.is_ascii_digit())
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
