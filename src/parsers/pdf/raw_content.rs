mod bytes;
mod decoder;
mod encryption;
mod streams;

use super::{PdfTextBackend, PdfTextExtraction, PdfTextObject};
pub(super) use decoder::{
    RawDecodeResources, decode_adobe_japan_identity, decode_pdf_string_heuristic,
};
use decoder::{collect_raw_pdf_text_lines, raw_font_decoders_for_page};
use streams::collect_scanned_pdf_stream_text_lines;

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
