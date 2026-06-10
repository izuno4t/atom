mod bytes;
mod decoder;
mod encryption;
mod streams;

use super::{
    AtomPdfTextBackend, PdfExtractBackend, PdfOxideFormWordsBackend, PdfOxideTextBackend,
    PdfTextBackend, PdfTextExtraction, PdfTextObject,
};
pub(super) use decoder::{
    RawDecodeResources, decode_adobe_japan_identity, decode_pdf_string_heuristic,
};
use decoder::{collect_raw_pdf_text_lines, raw_font_decoders_for_page};
use std::panic::AssertUnwindSafe;
use streams::collect_scanned_pdf_stream_text_lines;

static ATOM_PDF_PANIC_HOOK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub struct RawContentTextBackend;

impl PdfTextBackend for AtomPdfTextBackend {
    fn name(&self) -> &str {
        "atom-pdf-text"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        extract_atom_pdf_text(bytes)
    }
}

impl PdfTextBackend for RawContentTextBackend {
    fn name(&self) -> &str {
        "raw-content"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        extract_raw_content_text(bytes)
    }
}

fn extract_atom_pdf_text(bytes: &[u8]) -> PdfTextExtraction {
    for extraction in [
        extract_with_suppressed_panic_hook(|| PdfOxideFormWordsBackend.extract_text(bytes)),
        extract_with_suppressed_panic_hook(|| PdfOxideTextBackend.extract_text(bytes)),
    ] {
        if !extraction.objects.is_empty() && !extraction.ocr_required {
            return extraction;
        }
    }

    if bytes.len() < 16 * 1024 * 1024 {
        let extraction = PdfExtractBackend.extract_text(bytes);
        if !extraction.objects.is_empty() {
            return extraction;
        }
    }

    extract_raw_content_text(bytes)
}

fn extract_raw_content_text(bytes: &[u8]) -> PdfTextExtraction {
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
            collect_raw_pdf_text_lines(&content.operations, &font_decoders, &resources, &mut lines);
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

fn extract_with_suppressed_panic_hook(
    operation: impl FnOnce() -> PdfTextExtraction,
) -> PdfTextExtraction {
    let _guard = ATOM_PDF_PANIC_HOOK_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(AssertUnwindSafe(operation));
    std::panic::set_hook(previous_hook);
    result.unwrap_or_else(|_| PdfTextExtraction {
        objects: Vec::new(),
        extraction_failed: true,
        ocr_required: true,
    })
}
