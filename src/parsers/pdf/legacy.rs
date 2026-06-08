mod internal_text;

use super::types::{
    LenientPdfExtractBackend, LopdfTextBackend, PdfExtractBackend, PdfTextBackend,
    PdfTextExtraction, PdfTextObject, pdf_text_extraction_from_plain_text,
};

static PDF_EXTRACT_PANIC_HOOK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl PdfTextBackend for PdfExtractBackend {
    fn name(&self) -> &str {
        "pdf-extract"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let extracted = extract_text_from_mem_catching_panics(bytes);
        match extracted {
            Ok(Ok(text)) => pdf_text_extraction_from_plain_text(text),
            Ok(Err(_)) | Err(_) => PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            },
        }
    }
}

impl PdfTextBackend for LenientPdfExtractBackend {
    fn name(&self) -> &str {
        "pdf-extract-lenient"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let document = match lopdf::Document::load_mem(bytes) {
            Ok(document) => document,
            Err(_) => {
                return PdfTextExtraction {
                    objects: Vec::new(),
                    extraction_failed: true,
                    ocr_required: true,
                };
            }
        };
        let mut text = String::new();
        let extracted = catch_pdf_extract_panic(std::panic::AssertUnwindSafe(|| {
            let mut output = pdf_extract::PlainTextOutput::new(&mut text);
            pdf_extract::output_doc(&document, &mut output)
        }));
        match extracted {
            Ok(Ok(())) => pdf_text_extraction_from_plain_text(text),
            Ok(Err(_)) | Err(_) => PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            },
        }
    }
}

impl PdfTextBackend for LopdfTextBackend {
    fn name(&self) -> &str {
        "lopdf"
    }

    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction {
        let mut document = match lopdf::Document::load_mem(bytes) {
            Ok(document) => document,
            Err(_) => {
                return PdfTextExtraction {
                    objects: Vec::new(),
                    extraction_failed: true,
                    ocr_required: true,
                };
            }
        };

        if document.is_encrypted() && document.decrypt("").is_err() {
            return PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            };
        }

        let pages = document.get_pages().keys().copied().collect::<Vec<_>>();
        match document.extract_text(&pages) {
            Ok(text) => {
                let objects = text
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(|line| PdfTextObject {
                        text: line.to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    })
                    .collect::<Vec<_>>();
                PdfTextExtraction {
                    ocr_required: objects.is_empty(),
                    objects,
                    extraction_failed: false,
                }
            }
            Err(_) => PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: true,
                ocr_required: true,
            },
        }
    }
}

pub(super) fn extract_lopdf_page_texts(bytes: &[u8]) -> Option<Vec<Vec<PdfTextObject>>> {
    let document = lopdf::Document::load_mem(bytes).ok()?;
    if document.is_encrypted() {
        return None;
    }
    let pages = document.get_pages().keys().copied().collect::<Vec<_>>();
    Some(
        pages
            .into_iter()
            .map(|page| {
                document
                    .extract_text(&[page])
                    .ok()
                    .map(|text| {
                        text.lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty())
                            .map(|line| PdfTextObject {
                                text: line.to_string(),
                                font_size: None,
                                x: None,
                                y: None,
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            })
            .collect(),
    )
}

fn extract_text_from_mem_catching_panics(
    bytes: &[u8],
) -> Result<Result<String, pdf_extract::OutputError>, Box<dyn std::any::Any + Send>> {
    catch_pdf_extract_panic(|| pdf_extract::extract_text_from_mem(bytes))
}

fn catch_pdf_extract_panic<R, F>(operation: F) -> Result<R, Box<dyn std::any::Any + Send>>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    let _guard = PDF_EXTRACT_PANIC_HOOK_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(operation);
    std::panic::set_hook(previous_hook);
    result
}
