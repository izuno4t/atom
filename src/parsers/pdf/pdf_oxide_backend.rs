mod form_words;

use super::types::{
    PdfOxideFormWordsBackend, PdfOxideTextBackend, PdfTextBackend, PdfTextExtraction,
    failed_pdf_text_extraction, pdf_text_extraction_from_plain_text,
};
use form_words::extract_pdf_oxide_form_words_document;
use std::any::Any;
use std::panic::AssertUnwindSafe;

static PDF_OXIDE_PANIC_HOOK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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

        let (text, extraction_failed) = match catch_pdf_oxide_unwind(|| {
            extract_pdf_oxide_form_words_document(&document, page_count)
        }) {
            Ok(Ok(text)) => (text, false),
            Ok(Err(_)) | Err(_) => (String::new(), true),
        };

        let mut extraction = pdf_text_extraction_from_plain_text(text);
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
            match catch_pdf_oxide_unwind(|| document.extract_text(page_index)) {
                Ok(Ok(page_text)) => {
                    if !text.is_empty() && !text.ends_with('\n') {
                        text.push('\n');
                    }
                    text.push_str(&page_text);
                }
                Ok(Err(_)) | Err(_) => {
                    extraction_failed = true;
                }
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

fn catch_pdf_oxide_unwind<T>(
    operation: impl FnOnce() -> T,
) -> Result<T, Box<dyn Any + Send + 'static>> {
    let _guard = PDF_OXIDE_PANIC_HOOK_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(AssertUnwindSafe(operation));
    std::panic::set_hook(previous_hook);
    result
}
