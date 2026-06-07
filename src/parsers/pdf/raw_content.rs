use std::collections::BTreeMap;
use std::io::Read;

use md5::{Digest as _, Md5};

use super::{PdfTextBackend, PdfTextExtraction, PdfTextObject};

pub struct RawContentTextBackend;

impl PdfTextBackend for RawContentTextBackend {
    fn name(&self) -> &str {
        "raw-content"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let resources = RawDecodeResources::new();
        let mut lines = Vec::new();
        if let Ok(mut document) = lopdf::Document::load_mem(bytes) {
            if document.is_encrypted() {
                document.trailer.remove(b"Encrypt");
            }
            for object_id in document.get_pages().values() {
                let Ok(content_bytes) = document.get_page_content(*object_id) else {
                    continue;
                };
                let Ok(content) = lopdf::content::Content::decode(&content_bytes) else {
                    continue;
                };
                let font_decoders = raw_font_decoders_for_page(&document, *object_id);
                collect_raw_pdf_text_lines(
                    &content.operations,
                    &font_decoders,
                    &resources,
                    &mut lines,
                );
            }
        }
        if lines.is_empty() {
            collect_scanned_pdf_stream_text_lines(bytes, &resources, &mut lines);
        }
        let objects = lines
            .into_iter()
            .map(|text| PdfTextObject {
                text,
                font_size: None,
                x: None,
                y: None,
            })
            .collect::<Vec<_>>();
        PdfTextExtraction {
            ocr_required: objects.is_empty(),
            extraction_failed: objects.is_empty(),
            objects,
        }
    }
}

struct RawDecodeResources {
    adobe_japan_ucs2: Option<hayro_cmap::CMap>,
    rksj_h: Option<hayro_cmap::CMap>,
}

impl RawDecodeResources {
    fn new() -> Self {
        Self {
            adobe_japan_ucs2: predefined_cmap(hayro_cmap::CMapName::AdobeJapan1Ucs2),
            rksj_h: predefined_cmap(hayro_cmap::CMapName::N90msRksjH),
        }
    }
}

fn collect_scanned_pdf_stream_text_lines(
    bytes: &[u8],
    resources: &RawDecodeResources,
    lines: &mut Vec<String>,
) {
    for data in flate_pdf_content_streams(bytes) {
        let Ok(content) = lopdf::content::Content::decode(&data) else {
            continue;
        };
        collect_raw_pdf_text_lines(&content.operations, &BTreeMap::new(), resources, lines);
    }
}

fn flate_pdf_content_streams(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut streams = Vec::new();
    let encryption = raw_pdf_rc4_encryption(bytes);
    let mut search_start = 0;
    while let Some(stream_offset) = find_bytes(&bytes[search_start..], b"stream") {
        let stream_start = search_start + stream_offset;
        let data_start = skip_pdf_line_break(bytes, stream_start + b"stream".len());
        let Some(end_offset) = find_bytes(&bytes[data_start..], b"endstream") else {
            break;
        };
        let data_end = data_start + end_offset;
        if let Some(object_id) = text_content_stream_object_id(bytes, stream_start) {
            let compressed = trim_pdf_stream_data(&bytes[data_start..data_end]);
            if let Some(data) = inflate_zlib_stream(compressed) {
                streams.push(data);
            } else if let Some(encryption) = &encryption {
                let decrypted = encryption.decrypt_object_bytes(object_id, compressed);
                if let Some(data) = inflate_zlib_stream(&decrypted) {
                    streams.push(data);
                }
            }
        }
        search_start = data_end + b"endstream".len();
    }
    streams
}

fn text_content_stream_object_id(bytes: &[u8], stream_start: usize) -> Option<(u32, u16)> {
    let dict_start = stream_start.saturating_sub(4096);
    let context = &bytes[dict_start..stream_start];
    let obj_offset = rfind_bytes(context, b"obj")?;
    let dict = &context[obj_offset..];
    if !contains_bytes(dict, b"/FlateDecode") {
        return None;
    }
    if [
        b"/Subtype/Image".as_slice(),
        b"/Subtype /Image".as_slice(),
        b"/Subtype/XML".as_slice(),
        b"/Subtype /XML".as_slice(),
        b"/FontFile".as_slice(),
        b"/Type/XObject".as_slice(),
        b"/Type /XObject".as_slice(),
        b"/Metadata".as_slice(),
    ]
    .iter()
    .any(|needle| contains_bytes(dict, needle))
    {
        return None;
    }
    parse_object_id_before_obj(&context[..obj_offset])
}

fn parse_object_id_before_obj(bytes: &[u8]) -> Option<(u32, u16)> {
    let mut parts = bytes
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .rev();
    let generation = parse_ascii_u16(parts.next()?)?;
    let object_number = parse_ascii_u32(parts.next()?)?;
    Some((object_number, generation))
}

fn skip_pdf_line_break(bytes: &[u8], index: usize) -> usize {
    match bytes.get(index..index + 2) {
        Some(b"\r\n") => index + 2,
        _ if bytes
            .get(index)
            .is_some_and(|byte| matches!(*byte, b'\r' | b'\n')) =>
        {
            index + 1
        }
        _ => index,
    }
}

fn trim_pdf_stream_data(bytes: &[u8]) -> &[u8] {
    let bytes = bytes.strip_suffix(b"\r\n").unwrap_or(bytes);
    bytes.strip_suffix(b"\n").unwrap_or(bytes)
}

fn inflate_zlib_stream(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = flate2::read::ZlibDecoder::new(bytes);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).ok()?;
    Some(output)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn rfind_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    find_bytes(haystack, needle).is_some()
}

struct RawPdfRc4Encryption {
    file_key: Vec<u8>,
}

impl RawPdfRc4Encryption {
    fn decrypt_object_bytes(&self, object_id: (u32, u16), bytes: &[u8]) -> Vec<u8> {
        let mut hasher = Md5::new();
        hasher.update(&self.file_key);
        hasher.update(&object_id.0.to_le_bytes()[..3]);
        hasher.update(&object_id.1.to_le_bytes()[..2]);
        let key_len = (self.file_key.len() + 5).min(16);
        let key = hasher.finalize()[..key_len].to_vec();
        rc4_decrypt(&key, bytes)
    }
}

fn raw_pdf_rc4_encryption(bytes: &[u8]) -> Option<RawPdfRc4Encryption> {
    let encryption_dict = raw_standard_encryption_dict(bytes)?;
    let revision = parse_integer_after(encryption_dict, b"/R")?;
    let version = parse_integer_after(encryption_dict, b"/V")?;
    let key_length = parse_integer_after(encryption_dict, b"/Length").unwrap_or(40);
    if revision != 2 || version != 1 || key_length != 40 {
        return None;
    }
    let owner_value = parse_literal_string_after(encryption_dict, b"/O")?;
    let permissions = parse_integer_after(encryption_dict, b"/P")? as i32 as u32;
    let file_id = parse_first_file_id(bytes)?;

    let mut hasher = Md5::new();
    hasher.update(PDF_PASSWORD_PADDING);
    hasher.update(owner_value);
    hasher.update(permissions.to_le_bytes());
    hasher.update(file_id);
    let file_key = hasher.finalize()[..5].to_vec();
    Some(RawPdfRc4Encryption { file_key })
}

fn raw_standard_encryption_dict(bytes: &[u8]) -> Option<&[u8]> {
    let filter_offset = find_bytes(bytes, b"/Filter/Standard")?;
    let start = rfind_bytes(&bytes[..filter_offset], b"obj")?;
    let end = filter_offset + find_bytes(&bytes[filter_offset..], b"endobj")?;
    Some(&bytes[start..end])
}

fn parse_integer_after(bytes: &[u8], key: &[u8]) -> Option<i64> {
    let start = find_bytes(bytes, key)? + key.len();
    let bytes = &bytes[start..];
    let number_start = bytes.iter().position(|byte| !byte.is_ascii_whitespace())?;
    let bytes = &bytes[number_start..];
    let number_len = bytes
        .iter()
        .position(|byte| !byte.is_ascii_digit() && *byte != b'-')?;
    std::str::from_utf8(&bytes[..number_len]).ok()?.parse().ok()
}

fn parse_literal_string_after<'a>(bytes: &'a [u8], key: &[u8]) -> Option<Vec<u8>> {
    let key_offset = find_bytes(bytes, key)? + key.len();
    let open_offset = key_offset + bytes[key_offset..].iter().position(|byte| *byte == b'(')?;
    parse_pdf_literal_string(&bytes[open_offset..])
}

fn parse_pdf_literal_string(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.first() != Some(&b'(') {
        return None;
    }
    let mut output = Vec::new();
    let mut depth = 1usize;
    let mut index = 1usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                index += 1;
                let byte = *bytes.get(index)?;
                match byte {
                    b'n' => output.push(b'\n'),
                    b'r' => output.push(b'\r'),
                    b't' => output.push(b'\t'),
                    b'b' => output.push(0x08),
                    b'f' => output.push(0x0c),
                    b'\n' => {}
                    b'\r' => {
                        if bytes.get(index + 1) == Some(&b'\n') {
                            index += 1;
                        }
                    }
                    b'0'..=b'7' => {
                        let mut value = byte - b'0';
                        for _ in 0..2 {
                            if let Some(next @ b'0'..=b'7') = bytes.get(index + 1).copied() {
                                value = value.saturating_mul(8).saturating_add(next - b'0');
                                index += 1;
                            } else {
                                break;
                            }
                        }
                        output.push(value);
                    }
                    escaped => output.push(escaped),
                }
            }
            b'(' => {
                depth += 1;
                output.push(b'(');
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(output);
                }
                output.push(b')');
            }
            byte => output.push(byte),
        }
        index += 1;
    }
    None
}

fn parse_first_file_id(bytes: &[u8]) -> Option<Vec<u8>> {
    let id_offset = find_bytes(bytes, b"/ID")?;
    let first_open = id_offset + find_bytes(&bytes[id_offset..], b"<")?;
    let first_close = first_open + find_bytes(&bytes[first_open..], b">")?;
    decode_pdf_hex_string(&bytes[first_open + 1..first_close])
}

fn decode_pdf_hex_string(bytes: &[u8]) -> Option<Vec<u8>> {
    let nibbles = bytes
        .iter()
        .filter(|byte| !byte.is_ascii_whitespace())
        .map(|byte| byte.to_ascii_lowercase())
        .map(|byte| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let mut output = Vec::new();
    for chunk in nibbles.chunks(2) {
        let high = chunk[0];
        let low = *chunk.get(1).unwrap_or(&0);
        output.push((high << 4) | low);
    }
    Some(output)
}

fn parse_ascii_u32(bytes: &[u8]) -> Option<u32> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn parse_ascii_u16(bytes: &[u8]) -> Option<u16> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn rc4_decrypt(key: &[u8], input: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return input.to_vec();
    }
    let mut state = [0_u8; 256];
    for (index, value) in state.iter_mut().enumerate() {
        *value = index as u8;
    }
    let mut j = 0_u8;
    for index in 0..256 {
        j = j
            .wrapping_add(state[index])
            .wrapping_add(key[index % key.len()]);
        state.swap(index, j as usize);
    }
    let mut i = 0_u8;
    let mut j = 0_u8;
    input
        .iter()
        .map(|byte| {
            i = i.wrapping_add(1);
            j = j.wrapping_add(state[i as usize]);
            state.swap(i as usize, j as usize);
            let key_byte = state[(state[i as usize].wrapping_add(state[j as usize])) as usize];
            byte ^ key_byte
        })
        .collect()
}

const PDF_PASSWORD_PADDING: &[u8; 32] = &[
    0x28, 0xbf, 0x4e, 0x5e, 0x4e, 0x75, 0x8a, 0x41, 0x64, 0x00, 0x4e, 0x56, 0xff, 0xfa, 0x01, 0x08,
    0x2e, 0x2e, 0x00, 0xb6, 0xd0, 0x68, 0x3e, 0x80, 0x2f, 0x0c, 0xa9, 0xfe, 0x64, 0x53, 0x69, 0x7a,
];

#[derive(Clone, Copy)]
enum RawFontDecoder {
    Heuristic,
    AdobeJapanIdentity,
    AdobeJapanRksj,
}

fn raw_font_decoders_for_page(
    document: &lopdf::Document,
    object_id: lopdf::ObjectId,
) -> BTreeMap<Vec<u8>, RawFontDecoder> {
    document
        .get_page_fonts(object_id)
        .map(|fonts| {
            fonts
                .into_iter()
                .map(|(name, font)| (name, raw_font_decoder(font)))
                .collect()
        })
        .unwrap_or_default()
}

fn raw_font_decoder(font: &lopdf::Dictionary) -> RawFontDecoder {
    let encoding = font.get(b"Encoding").and_then(lopdf::Object::as_name);
    let base_font = font.get(b"BaseFont").and_then(lopdf::Object::as_name);
    if encoding
        .as_ref()
        .is_ok_and(|name| matches!(*name, b"Identity-H" | b"Identity-V"))
        && base_font
            .as_ref()
            .is_ok_and(|name| is_probably_adobe_japan_font(name))
    {
        RawFontDecoder::AdobeJapanIdentity
    } else if encoding
        .as_ref()
        .is_ok_and(|name| matches!(*name, b"90ms-RKSJ-H" | b"90ms-RKSJ-V"))
    {
        RawFontDecoder::AdobeJapanRksj
    } else {
        RawFontDecoder::Heuristic
    }
}

fn is_probably_adobe_japan_font(name: &[u8]) -> bool {
    [
        b"Ryumin".as_slice(),
        b"ShinGo".as_slice(),
        b"Gothic".as_slice(),
        b"Midashi".as_slice(),
        b"FutoGo".as_slice(),
        b"Mincho".as_slice(),
    ]
    .iter()
    .any(|needle| name.windows(needle.len()).any(|window| window == *needle))
}

fn collect_raw_pdf_text_lines(
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

fn decode_adobe_japan_identity(bytes: &[u8], resources: &RawDecodeResources) -> Option<String> {
    let unicode = resources.adobe_japan_ucs2.as_ref()?;
    let mut output = String::new();
    for chunk in bytes.chunks_exact(2) {
        let cid = u32::from(u16::from_be_bytes([chunk[0], chunk[1]]));
        append_bf_string(&mut output, unicode.lookup_bf_string(cid));
    }
    Some(normalize_raw_pdf_text(&output))
}

fn decode_adobe_japan_rksj(bytes: &[u8], resources: &RawDecodeResources) -> Option<String> {
    let encoding = resources.rksj_h.as_ref()?;
    let unicode = resources.adobe_japan_ucs2.as_ref()?;
    let mut output = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let mut consumed = 1;
        let mut code = bytes[index] as u32;
        let mut cid = None;
        if let Some(next) = bytes.get(index + 1) {
            let two_byte_code = ((bytes[index] as u32) << 8) | *next as u32;
            cid = encoding.lookup_cid_code(two_byte_code, 2);
            if cid.is_some() {
                consumed = 2;
                code = two_byte_code;
            }
        }
        let cid = cid.or_else(|| encoding.lookup_cid_code(code, 1))?;
        append_bf_string(&mut output, unicode.lookup_bf_string(cid));
        index += consumed;
    }
    Some(normalize_raw_pdf_text(&output))
}

fn predefined_cmap(name: hayro_cmap::CMapName<'_>) -> Option<hayro_cmap::CMap> {
    let data = hayro_cmap::load_embedded(name)?;
    hayro_cmap::CMap::parse(data, hayro_cmap::load_embedded)
}

fn append_bf_string(output: &mut String, value: Option<hayro_cmap::BfString>) {
    match value {
        Some(hayro_cmap::BfString::Char(character)) => output.push(character),
        Some(hayro_cmap::BfString::String(text)) => output.push_str(&text),
        None => {}
    }
}

fn decode_pdf_string_heuristic(bytes: &[u8], resources: &RawDecodeResources) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return decode_utf16be(&bytes[2..]);
    }
    let utf16be = decode_utf16be(bytes);
    if japanese_signal_score(&utf16be) > 2 {
        return utf16be;
    }
    if let Some(rksj) = decode_adobe_japan_rksj(bytes, resources)
        && japanese_signal_score(&rksj) > 0
    {
        return rksj;
    }
    let (shift_jis, _, had_errors) = encoding_rs::SHIFT_JIS.decode(bytes);
    if !had_errors && japanese_signal_score(&shift_jis) > 0 {
        return normalize_raw_pdf_text(&shift_jis);
    }
    String::from_utf8(bytes.to_vec())
        .map(|text| normalize_raw_pdf_text(&text))
        .unwrap_or_else(|_| normalize_raw_pdf_text(&shift_jis))
}

fn decode_utf16be(bytes: &[u8]) -> String {
    let units = bytes
        .chunks(2)
        .filter(|chunk| chunk.len() == 2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&units)
}

fn japanese_signal_score(text: &str) -> usize {
    text.chars()
        .filter(|character| {
            matches!(
                *character,
                '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{ff00}'..='\u{ffef}'
            )
        })
        .take(8)
        .count()
}

fn normalize_raw_pdf_text(text: &str) -> String {
    text.chars()
        .filter(|character| {
            !character.is_control()
                || *character == '\n'
                || *character == '\r'
                || *character == '\t'
        })
        .collect::<String>()
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
