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
