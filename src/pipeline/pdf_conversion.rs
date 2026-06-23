use std::collections::BTreeMap;
use std::io;

use crate::*;

pub(crate) fn convert_pdf_bytes(
    bytes: &[u8],
    options: &ConversionOptions,
    warnings: &mut Vec<String>,
    metadata: &mut Vec<(String, String)>,
) -> io::Result<(Vec<AstNode>, pdf::PdfParseResult)> {
    let mut result = pdf::parse_pdf_with_embedded_backend(bytes, warnings);
    if options.ocr != OcrEngine::None
        && !pdf::is_encrypted_pdf(bytes)
        && let Some(ocr_result) =
            try_pdf_ocr_for_pages_without_text(bytes, options, &result, warnings)?
    {
        result = ocr_result;
    }

    metadata.push(("pdf_backend".to_string(), result.backend.clone()));
    metadata.push((
        "pdf_extraction_failed".to_string(),
        result.extraction_failed.to_string(),
    ));
    metadata.push((
        "pdf_ocr_required".to_string(),
        result.ocr_required.to_string(),
    ));

    reject_unusable_pdf_result(bytes, &result)?;
    Ok((result.ast.clone(), result))
}

fn try_pdf_ocr_for_pages_without_text(
    bytes: &[u8],
    options: &ConversionOptions,
    current_result: &pdf::PdfParseResult,
    warnings: &mut Vec<String>,
) -> io::Result<Option<pdf::PdfParseResult>> {
    if options.ocr == OcrEngine::None {
        return Ok(None);
    }

    let Some(page_texts) = pdf::extract_lopdf_page_texts(bytes) else {
        return Ok(None);
    };
    let missing_pages = page_texts
        .iter()
        .enumerate()
        .filter_map(|(index, objects)| objects.is_empty().then_some(index))
        .collect::<Vec<_>>();
    let pages_requiring_ocr = if missing_pages.is_empty() && current_result.ocr_required {
        (0..page_texts.len()).collect::<Vec<_>>()
    } else {
        missing_pages
    };
    if pages_requiring_ocr.is_empty() {
        return Ok(None);
    }

    let Some(backend) = ocr::backend_for_engine(&options.ocr)? else {
        return Ok(None);
    };
    warnings.push(format!(
        "PDF OCR fallback selected for {} page(s) requiring OCR.",
        pages_requiring_ocr.len()
    ));
    let ocr_pages = ocr::recognize_pdf_pages(bytes, &pages_requiring_ocr, backend.as_ref())?;
    let ocr_by_page = ocr_pages.into_iter().collect::<BTreeMap<_, _>>();
    let objects = merge_pdf_page_text_with_ocr(page_texts, &ocr_by_page);
    if objects.is_empty() {
        return Ok(None);
    }

    Ok(Some(pdf::PdfParseResult {
        ast: pdf::infer_nodes_from_pdf_text_objects(objects, warnings),
        backend: format!("{}+{}", current_result.backend, ocr_name(&options.ocr)),
        extraction_failed: current_result.extraction_failed,
        ocr_required: false,
    }))
}

fn merge_pdf_page_text_with_ocr(
    page_texts: Vec<Vec<pdf::PdfTextObject>>,
    ocr_by_page: &BTreeMap<usize, String>,
) -> Vec<pdf::PdfTextObject> {
    let mut objects = Vec::new();
    for (page_index, page_objects) in page_texts.into_iter().enumerate() {
        if page_objects.is_empty() || ocr_by_page.contains_key(&page_index) {
            if let Some(text) = ocr_by_page.get(&page_index) {
                objects.extend(
                    text.lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty())
                        .map(|line| pdf::PdfTextObject {
                            text: line.to_string(),
                            font_size: None,
                            x: None,
                            y: None,
                        }),
                );
            }
        } else {
            objects.extend(page_objects);
        }
    }
    objects
}

fn reject_unusable_pdf_result(bytes: &[u8], result: &pdf::PdfParseResult) -> io::Result<()> {
    if result.ocr_required && pdf::is_encrypted_pdf(bytes) {
        let security_description = pdf::pdf_security_description(bytes);
        return Err(io::Error::other(format!(
            "PDF text extraction produced no text after trying Rust PDF backends; {security_description}. Last backend: {}. atom's current Rust PDF backends cannot extract text from this protected PDF; provide an unprotected copy or use a backend that can ignore extraction restrictions.",
            result.backend
        )));
    }
    if result.ocr_required {
        let diagnosis = pdf::diagnose_no_extractable_text(bytes);
        if pdf_result_contains_extracted_text(result) {
            return Err(io::Error::other(format!(
                "PDF text extraction appears incomplete after trying Rust PDF backends. Last backend: {}. {}",
                result.backend,
                diagnosis.message()
            )));
        }
        return Err(io::Error::other(format!(
            "PDF text extraction produced no text after trying Rust PDF backends. Last backend: {}. {}",
            result.backend,
            diagnosis.message()
        )));
    }
    Ok(())
}

fn pdf_result_contains_extracted_text(result: &pdf::PdfParseResult) -> bool {
    result.ast.iter().any(|node| match node {
        AstNode::Paragraph(text) | AstNode::Text(text) => {
            !text.starts_with("PDF text extraction produced no text")
        }
        AstNode::Heading { .. }
        | AstNode::List { .. }
        | AstNode::Table { .. }
        | AstNode::Image { .. }
        | AstNode::CodeBlock { .. }
        | AstNode::Footnote { .. }
        | AstNode::RawHtml(_) => true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_text_replaces_pages_without_extractable_text() {
        let page_texts = vec![
            Vec::new(),
            vec![pdf::PdfTextObject {
                text: "native page text".to_string(),
                font_size: None,
                x: None,
                y: None,
            }],
        ];
        let ocr_by_page = BTreeMap::from([
            (0, "OCR title\nOCR body".to_string()),
            (1, "ignored".to_string()),
        ]);

        let objects = merge_pdf_page_text_with_ocr(page_texts, &ocr_by_page);

        assert_eq!(
            objects
                .iter()
                .map(|object| object.text.as_str())
                .collect::<Vec<_>>(),
            vec!["OCR title", "OCR body", "ignored"]
        );
    }
}
