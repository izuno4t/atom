use super::super::types::{
    InternalPdfTextBackend, PdfTextBackend, PdfTextExtraction, PdfTextObject,
};

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
