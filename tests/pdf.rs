use bonjil::pdf::{InternalPdfTextBackend, PdfTextBackend, PdfTextExtraction};
use bonjil::{AstNode, pdf};

#[test]
fn parses_text_pdf_showing_operators_without_leaking_pdf_syntax() {
    let bytes = br#"%PDF-1.7
1 0 obj
<< /Length 81 >>
stream
BT
/F1 16 Tf
72 720 Td
(Document Title) Tj
T*
(Body text.) Tj
ET
endstream
endobj
%%EOF
"#;
    let mut warnings = Vec::new();

    let ast = pdf::parse_pdf(bytes, &mut warnings);

    assert_eq!(
        ast,
        vec![
            AstNode::Paragraph("Document Title".to_string()),
            AstNode::Paragraph("Body text.".to_string()),
        ]
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("text objects"))
    );
}

#[test]
fn infers_pdf_heading_from_larger_font_size() {
    let bytes = br#"%PDF-1.7
stream
BT
/F1 24 Tf
72 720 Td
(Document Title) Tj
/F1 11 Tf
0 -24 Td
(Body text.) Tj
ET
endstream
"#;
    let mut warnings = Vec::new();

    let ast = pdf::parse_pdf(bytes, &mut warnings);

    assert_eq!(
        ast,
        vec![
            AstNode::Heading {
                level: 1,
                text: "Document Title".to_string(),
            },
            AstNode::Paragraph("Body text.".to_string()),
        ]
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("heading inference"))
    );
}

#[test]
fn reports_tagged_pdf_structure_when_available() {
    let bytes = br#"%PDF-1.7
1 0 obj
<< /Type /Catalog /StructTreeRoot 2 0 R >>
endobj
stream
BT
/F1 12 Tf
72 720 Td
(Tagged paragraph) Tj
ET
endstream
"#;
    let mut warnings = Vec::new();

    let ast = pdf::parse_pdf(bytes, &mut warnings);

    assert_eq!(
        ast,
        vec![AstNode::Paragraph("Tagged paragraph".to_string())]
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("tagged structure"))
    );
}

#[test]
fn skips_binary_like_pdf_text_fragments_with_warning() {
    let bytes = "%PDF-1.7
stream
BT
/F1 12 Tf
(正常な本文) Tj
(abc\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}def) Tj
ET
endstream
"
    .as_bytes();
    let mut warnings = Vec::new();

    let ast = pdf::parse_pdf(bytes, &mut warnings);

    assert_eq!(ast, vec![AstNode::Paragraph("正常な本文".to_string())]);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("binary-like"))
    );
}

#[test]
fn parses_pdf_hex_strings_and_tj_arrays() {
    let bytes = br#"%PDF-1.7
stream
BT
/F1 22 Tf
72 720 Td
<FEFF65E5672C8A9E30BF30A430C830EB> Tj
/F1 11 Tf
0 -24 Td
[(Body ) 120 <0074006500780074> (.)] TJ
ET
endstream
"#;
    let mut warnings = Vec::new();

    let ast = pdf::parse_pdf(bytes, &mut warnings);

    assert_eq!(
        ast,
        vec![
            AstNode::Heading {
                level: 1,
                text: "日本語タイトル".to_string(),
            },
            AstNode::Paragraph("Body text.".to_string()),
        ]
    );
}

#[test]
fn internal_pdf_backend_preserves_basic_text_coordinates() {
    let bytes = br#"%PDF-1.7
stream
BT
/F1 12 Tf
72 720 Td
(Positioned text) Tj
ET
endstream
"#;

    let extraction = InternalPdfTextBackend.extract_text(bytes);

    assert_eq!(extraction.objects.len(), 1);
    assert_eq!(extraction.objects[0].text, "Positioned text");
    assert_eq!(extraction.objects[0].x, Some(72.0));
    assert_eq!(extraction.objects[0].y, Some(720.0));
}

struct StubPdfBackend;

impl PdfTextBackend for StubPdfBackend {
    fn name(&self) -> &str {
        "stub-pdf-backend"
    }

    fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
        PdfTextExtraction {
            objects: vec![pdf::PdfTextObject {
                text: "Stub heading".to_string(),
                font_size: Some(24.0),
                x: Some(72.0),
                y: Some(720.0),
            }],
            extraction_failed: false,
            ocr_required: false,
        }
    }
}

#[test]
fn pdf_parser_accepts_replaceable_text_backend() {
    let mut warnings = Vec::new();

    let result = pdf::parse_pdf_with_backend(b"%PDF-1.7", &StubPdfBackend, &mut warnings);

    assert_eq!(result.backend, "stub-pdf-backend");
    assert!(!result.extraction_failed);
    assert!(!result.ocr_required);
    assert_eq!(
        result.ast,
        vec![AstNode::Paragraph("Stub heading".to_string())]
    );
}

#[test]
fn pdf_parser_does_not_claim_ocr_is_required_when_internal_backend_fails() {
    let mut warnings = Vec::new();

    let result = pdf::parse_pdf_with_backend(
        b"%PDF-1.7\n1 0 obj\n<< /Length 3 >>\nstream\n...\nendstream",
        &InternalPdfTextBackend,
        &mut warnings,
    );

    assert!(result.ocr_required);
    assert_eq!(
        result.ast,
        vec![AstNode::Paragraph(
            "PDF text extraction produced no text with backend internal-text-objects. A full PDF backend or OCR may be required.".to_string(),
        )]
    );
    assert!(
        warnings
            .iter()
            .any(|warning| { warning.contains("A full PDF backend or OCR may be required") })
    );
}
