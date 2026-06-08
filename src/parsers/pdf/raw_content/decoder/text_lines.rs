use std::collections::BTreeMap;

use super::{
    RawDecodeResources, RawFontDecoder, decode_adobe_japan_identity, decode_adobe_japan_rksj,
    decode_pdf_string_heuristic, japanese_signal_score,
};

pub(in crate::parsers::pdf::raw_content) fn collect_raw_pdf_text_lines(
    operations: &[lopdf::content::Operation],
    font_decoders: &BTreeMap<Vec<u8>, RawFontDecoder>,
    resources: &RawDecodeResources,
    lines: &mut Vec<String>,
) {
    let mut current = String::new();
    let mut current_decoder = RawFontDecoder::Heuristic;
    for operation in operations {
        match operation.operator.as_str() {
            "Tf" => {
                if let Some(lopdf::Object::Name(name)) = operation.operands.first() {
                    current_decoder = font_decoders
                        .get(name)
                        .copied()
                        .unwrap_or(RawFontDecoder::Heuristic);
                }
            }
            "Tj" | "'" => {
                append_pdf_string_operand(
                    operation.operands.first(),
                    current_decoder,
                    resources,
                    &mut current,
                );
                push_pdf_text_line(&mut current, lines);
            }
            "\"" => {
                append_pdf_string_operand(
                    operation.operands.get(2),
                    current_decoder,
                    resources,
                    &mut current,
                );
                push_pdf_text_line(&mut current, lines);
            }
            "TJ" => {
                if let Some(lopdf::Object::Array(items)) = operation.operands.first() {
                    for item in items {
                        append_pdf_string_operand(
                            Some(item),
                            current_decoder,
                            resources,
                            &mut current,
                        );
                    }
                    push_pdf_text_line(&mut current, lines);
                }
            }
            "Td" | "TD" | "T*" | "ET" => push_pdf_text_line(&mut current, lines),
            _ => {}
        }
    }
    push_pdf_text_line(&mut current, lines);
}

fn append_pdf_string_operand(
    operand: Option<&lopdf::Object>,
    decoder: RawFontDecoder,
    resources: &RawDecodeResources,
    output: &mut String,
) {
    if let Some(lopdf::Object::String(bytes, _)) = operand {
        let decoded = match decoder {
            RawFontDecoder::Heuristic => decode_pdf_string_heuristic(bytes, resources),
            RawFontDecoder::AdobeJapanIdentity => decode_adobe_japan_identity(bytes, resources)
                .unwrap_or_else(|| decode_pdf_string_heuristic(bytes, resources)),
            RawFontDecoder::AdobeJapanRksj => decode_adobe_japan_rksj(bytes, resources)
                .unwrap_or_else(|| decode_pdf_string_heuristic(bytes, resources)),
        };
        if !decoded.is_empty() {
            output.push_str(&decoded);
        }
    }
}

fn push_pdf_text_line(current: &mut String, lines: &mut Vec<String>) {
    let line = current
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    current.clear();
    if line.chars().any(|character| !character.is_control()) && !is_raw_pdf_noise_line(&line) {
        lines.push(line);
    }
}

fn is_raw_pdf_noise_line(line: &str) -> bool {
    if line == "@@" || line.contains("䁀") || line.contains("㽥") {
        return true;
    }
    let total = line.chars().count();
    if total < 8 {
        return false;
    }

    let japanese = japanese_signal_score(line);
    let ascii_alnum = line
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .count();
    let replacement = line.matches('\u{fffd}').count();
    let symbolic = line
        .chars()
        .filter(|character| {
            !character.is_alphanumeric()
                && !character.is_whitespace()
                && !matches!(
                    *character,
                    '、' | '。' | '，' | '．' | '・' | '「' | '」' | '（' | '）' | '【' | '】'
                )
        })
        .count();
    japanese == 0 && (ascii_alnum * 3 < total || replacement > 0 || symbolic * 2 > total)
}
