use md5::{Digest as _, Md5};

use super::bytes::{find_bytes, rfind_bytes};

pub(super) struct RawPdfRc4Encryption {
    file_key: Vec<u8>,
}

impl RawPdfRc4Encryption {
    pub(super) fn decrypt_object_bytes(&self, object_id: (u32, u16), bytes: &[u8]) -> Vec<u8> {
        let mut hasher = Md5::new();
        hasher.update(&self.file_key);
        hasher.update(&object_id.0.to_le_bytes()[..3]);
        hasher.update(&object_id.1.to_le_bytes()[..2]);
        let key_len = (self.file_key.len() + 5).min(16);
        let key = hasher.finalize()[..key_len].to_vec();
        rc4_decrypt(&key, bytes)
    }
}

pub(super) fn raw_pdf_rc4_encryption(bytes: &[u8]) -> Option<RawPdfRc4Encryption> {
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

fn parse_literal_string_after(bytes: &[u8], key: &[u8]) -> Option<Vec<u8>> {
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
                    b'0'..=b'7' => read_octal_escape(bytes, &mut index, byte, &mut output),
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

fn read_octal_escape(bytes: &[u8], index: &mut usize, first: u8, output: &mut Vec<u8>) {
    let mut value = first - b'0';
    for _ in 0..2 {
        if let Some(next @ b'0'..=b'7') = bytes.get(*index + 1).copied() {
            value = value.saturating_mul(8).saturating_add(next - b'0');
            *index += 1;
        } else {
            break;
        }
    }
    output.push(value);
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
