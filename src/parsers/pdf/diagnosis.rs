use super::PdfNoTextDiagnosis;

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
        || lossy.contains("/Subtype /Image")
        || lossy.contains("/ImageB")
        || lossy.contains("/ImageC")
        || lossy.contains("/ImageI");
    let has_font = contains_pdf_name(&lossy, "/Type", "Font")
        || lossy.contains("/Type/Font")
        || lossy.contains("/BaseFont")
        || lossy.contains("/FontDescriptor")
        || lossy.contains("/Subtype/Type0")
        || lossy.contains("/Subtype /Type0");
    let has_to_unicode = lossy.contains("/ToUnicode");

    if has_image && !has_font {
        PdfNoTextDiagnosis::ImageOnly
    } else if has_font && (!has_to_unicode || has_unmapped_cid_fonts(bytes)) {
        PdfNoTextDiagnosis::MissingUnicodeMaps
    } else {
        PdfNoTextDiagnosis::Unknown
    }
}

fn contains_pdf_name(input: &str, key: &str, value: &str) -> bool {
    input.match_indices(key).any(|(index, _)| {
        input[index + key.len()..]
            .trim_start()
            .trim_start_matches('/')
            .starts_with(value)
    })
}

pub(super) fn has_unmapped_cid_fonts(bytes: &[u8]) -> bool {
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
