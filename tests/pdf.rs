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
