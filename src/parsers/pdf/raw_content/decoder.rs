use std::collections::BTreeMap;

mod text_lines;

pub(super) use text_lines::collect_raw_pdf_text_lines;

pub(in crate::parsers::pdf) struct RawDecodeResources {
    rksj_h: Option<hayro_cmap::CMap>,
}

impl RawDecodeResources {
    pub(in crate::parsers::pdf) fn new() -> Self {
        Self {
            rksj_h: predefined_cmap(hayro_cmap::CMapName::N90msRksjH),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum RawFontDecoder {
    Heuristic,
    AdobeJapanIdentity,
    AdobeJapanRksj,
}

pub(super) fn raw_font_decoders_for_page(
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

pub(in crate::parsers::pdf) fn decode_adobe_japan_identity(
    bytes: &[u8],
    _resources: &RawDecodeResources,
) -> Option<String> {
    let mut output = String::new();
    for chunk in bytes.chunks_exact(2) {
        let cid = u16::from_be_bytes([chunk[0], chunk[1]]);
        append_adobe_japan1_cid(&mut output, cid);
    }
    Some(normalize_raw_pdf_text(&output))
}

pub(in crate::parsers::pdf) fn decode_adobe_japan_rksj(
    bytes: &[u8],
    resources: &RawDecodeResources,
) -> Option<String> {
    let encoding = resources.rksj_h.as_ref()?;
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
        append_adobe_japan1_cid(&mut output, cid as u16);
        index += consumed;
    }
    Some(normalize_raw_pdf_text(&output))
}

pub(in crate::parsers::pdf) fn decode_pdf_string_heuristic(
    bytes: &[u8],
    resources: &RawDecodeResources,
) -> String {
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

fn append_adobe_japan1_cid(output: &mut String, cid: u16) {
    if let Some(codepoint) = pdf_oxide::fonts::cid_mappings::lookup_adobe_japan1(cid)
        && let Some(character) = char::from_u32(codepoint)
    {
        output.push(character);
        return;
    }
    if let Some(character) = supplemental_adobe_japan1_char(cid) {
        output.push(character);
        return;
    }
    output.push_str("(cid:");
    output.push_str(&cid.to_string());
    output.push(')');
}

fn supplemental_adobe_japan1_char(cid: u16) -> Option<char> {
    match cid {
        7744 => Some('槌'),
        7789 => Some('蔽'),
        _ => None,
    }
}

fn predefined_cmap(name: hayro_cmap::CMapName<'_>) -> Option<hayro_cmap::CMap> {
    let data = hayro_cmap::load_embedded(name)?;
    hayro_cmap::CMap::parse(data, hayro_cmap::load_embedded)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_supplemental_adobe_japan1_cids() {
        let resources = RawDecodeResources::new();
        let text = decode_adobe_japan_identity(&[0x1e, 0x40, 0x1e, 0x6d], &resources)
            .expect("CID text should decode");

        assert_eq!(text, "槌蔽");
    }
}
