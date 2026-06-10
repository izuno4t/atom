use std::collections::BTreeMap;
use std::io::Read;

use super::RawDecodeResources;
use super::bytes::{contains_bytes, find_bytes, rfind_bytes};
use super::decoder::collect_raw_pdf_text_lines;
use super::encryption::raw_pdf_rc4_encryption;

pub(super) fn collect_scanned_pdf_stream_text_lines(
    bytes: &[u8],
    resources: &RawDecodeResources,
    lines: &mut Vec<String>,
) {
    for data in flate_pdf_content_streams(bytes).into_iter().take(128) {
        let Ok(content) = lopdf::content::Content::decode(&data) else {
            continue;
        };
        collect_raw_pdf_text_lines(&content.operations, &BTreeMap::new(), resources, lines);
        if lines.len() >= 20_000 {
            break;
        }
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
            if compressed.len() > 16 * 1024 * 1024 {
                search_start = data_end + b"endstream".len();
                continue;
            }
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
    if !contains_bytes(dict, b"/FlateDecode") || is_non_text_stream(dict) {
        return None;
    }
    parse_object_id_before_obj(&context[..obj_offset])
}

fn is_non_text_stream(dict: &[u8]) -> bool {
    [
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

fn parse_ascii_u32(bytes: &[u8]) -> Option<u32> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn parse_ascii_u16(bytes: &[u8]) -> Option<u16> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}
