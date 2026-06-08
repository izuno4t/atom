use super::raw_content;
use super::types::{
    PdfRsTextBackend, PdfTextBackend, PdfTextExtraction, failed_pdf_text_extraction,
    pdf_text_extraction_from_plain_text,
};

impl PdfTextBackend for PdfRsTextBackend {
    fn name(&self) -> &str {
        "pdf-rs-text"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        match extract_pdf_rs_text(bytes) {
            Some(text) if pdf_rs_text_looks_extracted(&text) => {
                pdf_text_extraction_from_plain_text(text)
            }
            None => failed_pdf_text_extraction(),
            Some(_) => failed_pdf_text_extraction(),
        }
    }
}

fn extract_pdf_rs_text(bytes: &[u8]) -> Option<String> {
    let file = ::pdf::file::FileOptions::cached()
        .load(bytes.to_vec())
        .ok()?;
    let resolver = file.resolver();
    let decode_resources = raw_content::RawDecodeResources::new();
    let mut pages = Vec::new();

    for page in file.pages() {
        let page = page.ok()?;
        let Some(content) = page.contents.as_ref() else {
            continue;
        };
        let operations = content.operations(&resolver).ok()?;
        let mut page_text = String::new();
        let mut line = String::new();
        let mut current_font = None::<::pdf::primitive::Name>;

        for operation in operations {
            match operation {
                ::pdf::content::Op::TextFont { name, .. } => {
                    current_font = Some(name);
                }
                ::pdf::content::Op::TextDraw { text } => {
                    append_pdf_rs_string(
                        &text,
                        current_font.as_ref(),
                        page.resources.as_ref(),
                        &resolver,
                        &decode_resources,
                        &mut line,
                    );
                }
                ::pdf::content::Op::TextDrawAdjusted { array } => {
                    for item in array {
                        if let ::pdf::content::TextDrawAdjusted::Text(text) = item {
                            append_pdf_rs_string(
                                &text,
                                current_font.as_ref(),
                                page.resources.as_ref(),
                                &resolver,
                                &decode_resources,
                                &mut line,
                            );
                        }
                    }
                }
                ::pdf::content::Op::MoveTextPosition { .. }
                | ::pdf::content::Op::TextNewline
                | ::pdf::content::Op::EndText => {
                    push_pdf_rs_line(&mut line, &mut page_text);
                }
                _ => {}
            }
        }
        push_pdf_rs_line(&mut line, &mut page_text);
        if !page_text.trim().is_empty() {
            pages.push(page_text);
        }
    }

    let text = pages.join("\n\n");
    (!text.trim().is_empty()).then_some(text)
}

fn append_pdf_rs_string(
    text: &::pdf::primitive::PdfString,
    current_font: Option<&::pdf::primitive::Name>,
    resources: Option<&::pdf::object::MaybeRef<::pdf::object::Resources>>,
    resolver: &impl ::pdf::object::Resolve,
    decode_resources: &raw_content::RawDecodeResources,
    output: &mut String,
) {
    if let Some(decoded) = decode_pdf_rs_string_with_font(text, current_font, resources, resolver)
        && !decoded.is_empty()
    {
        output.push_str(&decoded);
        return;
    }
    if let Some(decoded) =
        raw_content::decode_adobe_japan_identity(text.as_bytes(), decode_resources)
        && pdf_rs_fragment_looks_text(&decoded)
    {
        output.push_str(&decoded);
        return;
    }
    let decoded = raw_content::decode_pdf_string_heuristic(text.as_bytes(), decode_resources);
    if pdf_rs_fragment_looks_text(&decoded) {
        output.push_str(&decoded);
    }
}

fn decode_pdf_rs_string_with_font(
    text: &::pdf::primitive::PdfString,
    current_font: Option<&::pdf::primitive::Name>,
    resources: Option<&::pdf::object::MaybeRef<::pdf::object::Resources>>,
    resolver: &impl ::pdf::object::Resolve,
) -> Option<String> {
    let font_name = current_font?;
    let resources = resources?;
    let font = resources.fonts.get(font_name)?.load(resolver).ok()?;
    let unicode = font.to_unicode(resolver)?.ok()?;
    decode_pdf_rs_to_unicode(text.as_bytes(), &unicode)
}

fn decode_pdf_rs_to_unicode(bytes: &[u8], unicode: &::pdf::font::ToUnicodeMap) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let mut output = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let mut matched = false;
        if let Some(next) = bytes.get(index + 1) {
            let code = u16::from_be_bytes([bytes[index], *next]);
            if let Some(text) = unicode.get(code) {
                output.push_str(text);
                index += 2;
                matched = true;
            }
        }
        if !matched {
            let code = bytes[index] as u16;
            if let Some(text) = unicode.get(code) {
                output.push_str(text);
                matched = true;
            }
            index += 1;
        }
        if !matched {
            return None;
        }
    }
    Some(output)
}

fn push_pdf_rs_line(line: &mut String, output: &mut String) {
    let normalized = line
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    line.clear();
    if normalized.is_empty() {
        return;
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(&normalized);
}

fn pdf_rs_fragment_looks_text(text: &str) -> bool {
    let total = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if total == 0 {
        return false;
    }
    let text_like = text
        .chars()
        .filter(|character| {
            character.is_alphanumeric()
                || matches!(
                    *character,
                    '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{ff00}'..='\u{ffef}'
                )
        })
        .count();
    text_like * 2 >= total
}

fn pdf_rs_text_looks_extracted(text: &str) -> bool {
    let total = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if total < 100 {
        return false;
    }
    let letters = text
        .chars()
        .filter(|character| character.is_alphabetic())
        .count();
    let japanese = text
        .chars()
        .filter(|character| {
            matches!(
                *character,
                '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{ff00}'..='\u{ffef}'
            )
        })
        .count();
    let controls = text
        .chars()
        .filter(|character| {
            character.is_control() && *character != '\n' && *character != '\r' && *character != '\t'
        })
        .count();

    controls * 20 < total && (letters + japanese) * 3 > total
}
