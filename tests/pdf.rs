use anything_to_markdown::pdf::{
    InternalPdfTextBackend, LenientPdfExtractBackend, PdfTextBackend, PdfTextExtraction,
    PdfTextObject,
};
use anything_to_markdown::{AstNode, Flavor, TableCell, TableRow, markdown, pdf};

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
fn pdf_commonmark_output_merges_line_breaks_inside_sentences() {
    let objects = vec![
        PdfTextObject {
            text:
                "単体テストを今見てみると、多くのことにいかに無駄な時間を費やしてしまったのか、とい"
                    .to_string(),
            font_size: None,
            x: None,
            y: None,
        },
        PdfTextObject {
            text: "う思いに駆られてしまいます。".to_string(),
            font_size: None,
            x: None,
            y: None,
        },
        PdfTextObject {
            text: "次の文です。".to_string(),
            font_size: None,
            x: None,
            y: None,
        },
    ];
    let mut warnings = Vec::new();

    let ast = pdf::infer_nodes_from_pdf_text_objects(objects, &mut warnings);
    let actual = markdown::write_markdown(&ast, Flavor::CommonMark);

    assert!(actual.contains("無駄な時間を費やしてしまったのか、という思いに駆られてしまいます。"));
    assert!(actual.contains("駆られてしまいます。\n\n次の文です。"));
}

#[test]
fn pdf_pipe_table_lines_become_table_nodes() {
    let objects = vec![
        pdf_text("| No | Question | Owner | Answer |"),
        pdf_text("| --- | --- | --- | --- |"),
        pdf_text("| 1 | Why test? | Team | Because quality matters. |"),
        pdf_text("| 2 | How often? | Ops | Every release cycle. |"),
    ];
    let mut warnings = Vec::new();

    let ast = pdf::infer_nodes_from_pdf_text_objects(objects, &mut warnings);

    assert_eq!(
        ast,
        vec![AstNode::Table {
            rows: vec![
                table_row(["No", "Question", "Owner", "Answer"]),
                table_row(["1", "Why test?", "Team", "Because quality matters."]),
                table_row(["2", "How often?", "Ops", "Every release cycle."]),
            ],
        }]
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

#[test]
fn lenient_pdf_extract_backend_extracts_text_showing_operators() {
    let extraction = LenientPdfExtractBackend.extract_text(&valid_minimal_text_pdf());

    let texts = extraction
        .objects
        .iter()
        .map(|object| object.text.as_str())
        .collect::<Vec<_>>();
    let text = texts.join("\n");
    assert!(text.contains("Fixture Title"), "{text:?}");
    assert!(text.contains("Fixture body text."), "{text:?}");
    assert!(!extraction.ocr_required);
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
fn pdf_parser_marks_incomplete_cid_font_text_as_ocr_required() {
    struct PartialJapaneseBackend;

    impl PdfTextBackend for PartialJapaneseBackend {
        fn name(&self) -> &str {
            "partial-japanese"
        }

        fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
            PdfTextExtraction {
                objects: vec![pdf::PdfTextObject {
                    text: "Vol.157 AI".to_string(),
                    font_size: None,
                    x: None,
                    y: None,
                }],
                extraction_failed: false,
                ocr_required: false,
            }
        }
    }

    let bytes = b"%PDF-1.7
1 0 obj
<< /Type /Font /Subtype /Type0 /Encoding /Identity-H /BaseFont /AAAAAD+HiraKakuProN-W3 >>
endobj
2 0 obj
<< /Type /Font /Subtype /Type0 /Encoding /Identity-H /BaseFont /AAAAAG+HiraMinProN-W3 >>
endobj";
    let mut warnings = Vec::new();

    let result = pdf::parse_pdf_with_backend(bytes, &PartialJapaneseBackend, &mut warnings);

    assert!(result.ocr_required);
    assert!(
        warnings
            .iter()
            .any(|warning| { warning.contains("CID fonts lack Unicode maps") })
    );
}

#[test]
fn pdf_parser_tries_next_backend_when_primary_extracts_no_text() {
    struct EmptyBackend;
    struct TextBackend;

    impl PdfTextBackend for EmptyBackend {
        fn name(&self) -> &str {
            "empty-backend"
        }

        fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
            PdfTextExtraction {
                objects: Vec::new(),
                extraction_failed: false,
                ocr_required: true,
            }
        }
    }

    impl PdfTextBackend for TextBackend {
        fn name(&self) -> &str {
            "text-backend"
        }

        fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
            PdfTextExtraction {
                objects: vec![pdf::PdfTextObject {
                    text: "Recovered text".to_string(),
                    font_size: None,
                    x: None,
                    y: None,
                }],
                extraction_failed: false,
                ocr_required: false,
            }
        }
    }

    let mut warnings = Vec::new();
    let backends: [&dyn PdfTextBackend; 2] = [&EmptyBackend, &TextBackend];

    let result = pdf::parse_pdf_with_ordered_backends(b"%PDF-1.7", &backends, &mut warnings);

    assert_eq!(result.backend, "text-backend");
    assert!(!result.ocr_required);
    assert_eq!(
        result.ast,
        vec![AstNode::Paragraph("Recovered text".to_string())]
    );
}

#[test]
fn pdf_no_text_diagnosis_detects_image_only_pdf() {
    let diagnosis = pdf::diagnose_no_extractable_text(
        b"%PDF-1.7\n<</ProcSet[/PDF/ImageB]/XObject<</Im0 2 0 R>>>>\n<</Subtype/Image/Type/XObject>>",
    );

    assert_eq!(diagnosis, pdf::PdfNoTextDiagnosis::ImageOnly);
}

#[test]
fn pdf_no_text_diagnosis_detects_image_pdf_even_with_text_procset() {
    let diagnosis = pdf::diagnose_no_extractable_text(
        b"%PDF-1.7\n<</ProcSet[/PDF/Text/ImageB]/XObject<</Im0 2 0 R>>>>\n<</Subtype/Image/Type/XObject>>",
    );

    assert_eq!(diagnosis, pdf::PdfNoTextDiagnosis::ImageOnly);
    assert!(
        diagnosis
            .message()
            .contains("outlined/vector text but no extractable text layer")
    );
}

#[test]
fn pdf_no_text_diagnosis_detects_spaced_pdf_name_image_subtype() {
    let diagnosis = pdf::diagnose_no_extractable_text(
        b"%PDF-1.7\n<</ProcSet[/PDF/Text]/XObject<</Im0 2 0 R>>>>\n<</Subtype /Image /Type /XObject>>",
    );

    assert_eq!(diagnosis, pdf::PdfNoTextDiagnosis::ImageOnly);
}

#[test]
fn pdf_no_text_diagnosis_detects_missing_unicode_maps() {
    let diagnosis = pdf::diagnose_no_extractable_text(
        b"%PDF-1.7\n<</ProcSet[/PDF/Text]/Font<</F1 2 0 R>>>>\n<</Type/Font/Subtype/Type0/Encoding/Identity-H>>",
    );

    assert_eq!(diagnosis, pdf::PdfNoTextDiagnosis::MissingUnicodeMaps);
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

#[test]
fn infers_pdf_headings_and_lists_from_section_numbers_without_font_metadata() {
    struct LinesOnlyBackend;

    impl PdfTextBackend for LinesOnlyBackend {
        fn name(&self) -> &str {
            "lines-only"
        }

        fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
            PdfTextExtraction {
                objects: vec![
                    pdf::PdfTextObject {
                        text: "1 VPN接続の概要".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "本文です。".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "1.1 事前準備".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "- VPNクライアントをインストールする".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "- 証明書を用意する".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                ],
                extraction_failed: false,
                ocr_required: false,
            }
        }
    }

    let mut warnings = Vec::new();

    let result = pdf::parse_pdf_with_backend(b"%PDF-1.7", &LinesOnlyBackend, &mut warnings);

    assert_eq!(
        result.ast,
        vec![
            AstNode::Heading {
                level: 1,
                text: "1 VPN接続の概要".to_string(),
            },
            AstNode::Paragraph("本文です。".to_string()),
            AstNode::Heading {
                level: 2,
                text: "1.1 事前準備".to_string(),
            },
            AstNode::List {
                ordered: false,
                items: vec![
                    vec![AstNode::Text(
                        "VPNクライアントをインストールする".to_string()
                    )],
                    vec![AstNode::Text("証明書を用意する".to_string())],
                ],
            },
        ]
    );
    assert!(warnings.iter().any(|warning| {
        warning.contains("PDF heading inference treated '1 VPN接続の概要' as h1 by section number")
    }));
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("PDF list inference grouped 2 item(s)"))
    );
}

#[test]
fn pdf_section_number_in_sentence_is_not_promoted_to_heading() {
    struct LinesOnlyBackend;

    impl PdfTextBackend for LinesOnlyBackend {
        fn name(&self) -> &str {
            "lines-only"
        }

        fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
            PdfTextExtraction {
                objects: vec![pdf::PdfTextObject {
                    text: "1. VPNを設定します。".to_string(),
                    font_size: None,
                    x: None,
                    y: None,
                }],
                extraction_failed: false,
                ocr_required: false,
            }
        }
    }

    let mut warnings = Vec::new();

    let result = pdf::parse_pdf_with_backend(b"%PDF-1.7", &LinesOnlyBackend, &mut warnings);

    assert_eq!(
        result.ast,
        vec![AstNode::Paragraph("1. VPNを設定します。".to_string())]
    );
}

#[test]
fn pdf_repeated_page_noise_is_not_promoted_or_kept() {
    struct LinesOnlyBackend;

    impl PdfTextBackend for LinesOnlyBackend {
        fn name(&self) -> &str {
            "lines-only"
        }

        fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
            PdfTextExtraction {
                objects: vec![
                    pdf::PdfTextObject {
                        text: "GitHub Copilot のテクニック集".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "3".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "© ZOZO, Inc.".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "2026/04/03 17:44".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "1. ショートカットの活用".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                ],
                extraction_failed: false,
                ocr_required: false,
            }
        }
    }

    let mut warnings = Vec::new();

    let result = pdf::parse_pdf_with_backend(b"%PDF-1.7", &LinesOnlyBackend, &mut warnings);

    assert_eq!(
        result.ast,
        vec![
            AstNode::Paragraph("GitHub Copilot のテクニック集".to_string()),
            AstNode::Heading {
                level: 2,
                text: "1. ショートカットの活用".to_string(),
            },
        ]
    );
}

#[test]
fn pdf_repeated_one_headings_are_renumbered_in_order() {
    struct LinesOnlyBackend;

    impl PdfTextBackend for LinesOnlyBackend {
        fn name(&self) -> &str {
            "lines-only"
        }

        fn extract_text(&self, _bytes: &[u8]) -> PdfTextExtraction {
            PdfTextExtraction {
                objects: vec![
                    pdf::PdfTextObject {
                        text: "1. ショートカットの活用".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "1. Neighboring Tabs の活用".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                    pdf::PdfTextObject {
                        text: "1. 記号の活用".to_string(),
                        font_size: None,
                        x: None,
                        y: None,
                    },
                ],
                extraction_failed: false,
                ocr_required: false,
            }
        }
    }

    let mut warnings = Vec::new();

    let result = pdf::parse_pdf_with_backend(b"%PDF-1.7", &LinesOnlyBackend, &mut warnings);

    assert_eq!(
        result.ast,
        vec![
            AstNode::Heading {
                level: 2,
                text: "1. ショートカットの活用".to_string(),
            },
            AstNode::Heading {
                level: 2,
                text: "2. Neighboring Tabs の活用".to_string(),
            },
            AstNode::Heading {
                level: 2,
                text: "3. 記号の活用".to_string(),
            },
        ]
    );
    assert!(
        warnings.iter().any(|warning| {
            warning.contains("PDF heading inference renumbered repeated heading")
        })
    );
}

fn valid_minimal_text_pdf() -> Vec<u8> {
    let content = "BT\n/F1 24 Tf\n72 720 Td\n(Fixture Title) Tj\nT*\n(Fixture body text.) Tj\nET\n";
    let widths = std::iter::repeat_n("600", 256)
        .collect::<Vec<_>>()
        .join(" ");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
        format!(
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding /FirstChar 0 /LastChar 255 /Widths [{}] >>",
            widths
        ),
        format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content.len(),
            content
        ),
    ];
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = vec![0_usize];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", index + 1, object));
    }
    let xref_offset = pdf.len();
    pdf.push_str("xref\n0 6\n0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str("trailer\n<< /Size 6 /Root 1 0 R >>\n");
    pdf.push_str("start");
    pdf.push_str(&format!("xref\n{xref_offset}\n"));
    pdf.push_str("%%EOF\n");
    pdf.into_bytes()
}

fn pdf_text(text: &str) -> pdf::PdfTextObject {
    pdf::PdfTextObject {
        text: text.to_string(),
        font_size: None,
        x: None,
        y: None,
    }
}

fn table_row<const N: usize>(cells: [&str; N]) -> TableRow {
    TableRow {
        cells: cells
            .into_iter()
            .map(|text| TableCell {
                text: text.to_string(),
                rowspan: 1,
                colspan: 1,
                image: None,
            })
            .collect(),
    }
}
