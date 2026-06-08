mod form_words;

use super::types::{
    PdfOxideFormWordsBackend, PdfOxideTextBackend, PdfTextBackend, PdfTextExtraction,
    failed_pdf_text_extraction, pdf_text_extraction_from_plain_text,
};
use form_words::extract_pdf_oxide_form_words_page;

impl PdfTextBackend for PdfOxideFormWordsBackend {
    fn name(&self) -> &str {
        "pdf-oxide-form-words"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let Ok(document) = pdf_oxide::PdfDocument::from_bytes(bytes.to_vec()) else {
            return failed_pdf_text_extraction();
        };
        let Ok(page_count) = document.page_count() else {
            return failed_pdf_text_extraction();
        };

        let mut chunks = Vec::new();
        let mut extraction_failed = false;
        for page_index in 0..page_count {
            let page_text = match extract_pdf_oxide_form_words_page(&document, page_index) {
                Ok(Some(content)) => content,
                Ok(None) => match document.extract_text(page_index) {
                    Ok(content) => content,
                    Err(_) => {
                        extraction_failed = true;
                        String::new()
                    }
                },
                Err(_) => {
                    extraction_failed = true;
                    String::new()
                }
            };
            if !page_text.trim().is_empty() {
                chunks.push(page_text);
            }
        }

        let mut extraction = pdf_text_extraction_from_plain_text(chunks.join("\n\n"));
        extraction.extraction_failed = extraction_failed && extraction.objects.is_empty();
        if extraction.objects.is_empty() {
            extraction.ocr_required = true;
        }
        extraction
    }
}

impl PdfTextBackend for PdfOxideTextBackend {
    fn name(&self) -> &str {
        "pdf-oxide-text"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let Ok(document) = pdf_oxide::PdfDocument::from_bytes(bytes.to_vec()) else {
            return failed_pdf_text_extraction();
        };
        let Ok(page_count) = document.page_count() else {
            return failed_pdf_text_extraction();
        };

        let mut text = String::new();
        let mut extraction_failed = false;
        for page_index in 0..page_count {
            match document.extract_text(page_index) {
                Ok(page_text) => {
                    if !text.is_empty() && !text.ends_with('\n') {
                        text.push('\n');
                    }
                    text.push_str(&page_text);
                }
                Err(_) => extraction_failed = true,
            }
        }

        let mut extraction = pdf_text_extraction_from_plain_text(text);
        extraction.extraction_failed = extraction_failed && extraction.objects.is_empty();
        if extraction.objects.is_empty() {
            extraction.ocr_required = true;
        }
        extraction
    }
}
