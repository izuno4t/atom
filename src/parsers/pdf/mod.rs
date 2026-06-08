use crate::AstNode;

mod diagnosis;
mod legacy;
mod pdf_oxide_backend;
mod pdf_rs;
mod quality;
mod raw_content;
mod structure;
mod types;

pub use diagnosis::{diagnose_no_extractable_text, is_encrypted_pdf, pdf_security_description};
pub use raw_content::RawContentTextBackend;
pub use structure::infer_headings;
pub use types::{
    InternalPdfTextBackend, LenientPdfExtractBackend, LopdfTextBackend, PdfExtractBackend,
    PdfNoTextDiagnosis, PdfOxideFormWordsBackend, PdfOxideTextBackend, PdfParseResult,
    PdfRsTextBackend, PdfTextBackend, PdfTextExtraction, PdfTextObject,
};

use quality::{pdf_result_is_usable, pdf_result_score, pdf_text_looks_incomplete};
use structure::infer_nodes_from_text_objects;

pub fn parse_pdf(bytes: &[u8], warnings: &mut Vec<String>) -> Vec<AstNode> {
    parse_pdf_with_backend(bytes, &InternalPdfTextBackend, warnings).ast
}

pub fn parse_pdf_with_embedded_backend(bytes: &[u8], warnings: &mut Vec<String>) -> PdfParseResult {
    let backends: [&dyn PdfTextBackend; 5] = [
        &PdfOxideFormWordsBackend,
        &PdfRsTextBackend,
        &RawContentTextBackend,
        &PdfOxideTextBackend,
        &InternalPdfTextBackend,
    ];
    parse_pdf_with_ordered_backends(bytes, &backends, warnings)
}

pub fn parse_pdf_with_ordered_backends(
    bytes: &[u8],
    backends: &[&dyn PdfTextBackend],
    warnings: &mut Vec<String>,
) -> PdfParseResult {
    let mut best_result = None;
    for backend in backends {
        let result = parse_pdf_with_backend(bytes, *backend, warnings);
        if pdf_result_is_usable(&result) && !result.ocr_required {
            return result;
        }
        if best_result
            .as_ref()
            .is_none_or(|best| pdf_result_score(&result) > pdf_result_score(best))
        {
            best_result = Some(result);
        }
    }
    best_result.unwrap_or_else(|| parse_pdf_with_backend(bytes, &InternalPdfTextBackend, warnings))
}

pub fn extract_lopdf_page_texts(bytes: &[u8]) -> Option<Vec<Vec<PdfTextObject>>> {
    legacy::extract_lopdf_page_texts(bytes)
}

pub fn infer_nodes_from_pdf_text_objects(
    objects: Vec<PdfTextObject>,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    infer_nodes_from_text_objects(objects, warnings)
}

pub fn parse_pdf_with_backend(
    bytes: &[u8],
    backend: &dyn PdfTextBackend,
    warnings: &mut Vec<String>,
) -> PdfParseResult {
    let lossy = String::from_utf8_lossy(bytes);
    warnings.push(
        "PDF parser extracts text objects; coordinates and layout inference are limited."
            .to_string(),
    );
    if lossy.contains("/StructTreeRoot") {
        warnings.push(
            "PDF tagged structure detected; logical reading order should be preferred.".to_string(),
        );
    } else {
        warnings.push(
            "PDF tag tree was not detected; falling back to content stream order.".to_string(),
        );
    }

    let extraction = backend.extract_text(bytes);
    let mut ocr_required = extraction.ocr_required || extraction.objects.is_empty();
    let ast = if extraction.objects.is_empty() {
        let message = format!(
            "PDF text extraction produced no text with backend {}. A full PDF backend or OCR may be required.",
            backend.name()
        );
        warnings.push(message.clone());
        vec![AstNode::Paragraph(message)]
    } else {
        infer_nodes_from_text_objects(extraction.objects, warnings)
    };

    if !ocr_required && pdf_text_looks_incomplete(bytes, &ast) {
        ocr_required = true;
        warnings.push(
            "PDF text extraction appears incomplete because CID fonts lack Unicode maps; OCR is required."
                .to_string(),
        );
    }

    PdfParseResult {
        ast,
        backend: backend.name().to_string(),
        extraction_failed: extraction.extraction_failed,
        ocr_required,
    }
}
