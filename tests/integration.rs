use anything_to_markdown::{
    AstNode, ConversionOptions, Converter, Flavor, LlmBackend, OutputFormat, TableCell, TableRow,
    docx, markdown, ooxml,
};
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use std::process::Command;
use zip::{ZipWriter, write::SimpleFileOptions};

#[test]
fn converts_ast_to_commonmark() {
    let ast = vec![
        AstNode::Heading {
            level: 1,
            text: "Title".to_string(),
        },
        AstNode::Paragraph("Hello world".to_string()),
        AstNode::List {
            ordered: false,
            items: vec![vec![AstNode::Text("item".to_string())]],
        },
    ];

    let actual = markdown::write_markdown(&ast, Flavor::CommonMark);

    assert_eq!(actual, "# Title\n\nHello world\n\n- item\n");
}

#[test]
fn cli_prints_version_without_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_atom"))
        .arg("--version")
        .output()
        .expect("atom binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("atom {}", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn markdown_writer_normalizes_lint_sensitive_output() {
    let long_text = "これは非常に長い説明文です".repeat(12);
    let ast = vec![
        AstNode::Paragraph("【手順書】GlobalProtect MacOS 操作手順書".to_string()),
        AstNode::Paragraph(format!(
            "{long_text} https://info2.nri-net.com/cgi-bin/sample nc-info-admin-all@nri-net.com"
        )),
    ];

    let actual = markdown::write_markdown(&ast, Flavor::Markdownlint);

    assert!(actual.starts_with("# 【手順書】GlobalProtect MacOS 操作手順書\n"));
    assert!(actual.lines().all(|line| line.chars().count() <= 80));
    assert!(actual.contains("<https://info2.nri-net.com/cgi-bin/sample>"));
    assert!(actual.contains("<nc-info-admin-all@nri-net.com>"));
}

#[test]
fn markdown_writer_keeps_single_h1() {
    let ast = vec![
        AstNode::Heading {
            level: 1,
            text: "Document".to_string(),
        },
        AstNode::Heading {
            level: 1,
            text: "Section".to_string(),
        },
        AstNode::List {
            ordered: false,
            items: vec![vec![AstNode::Heading {
                level: 1,
                text: "Nested".to_string(),
            }]],
        },
    ];

    let actual = markdown::write_markdown(&ast, Flavor::Markdownlint);

    assert_eq!(actual.matches("\n# ").count(), 0);
    assert_eq!(
        actual.lines().filter(|line| line.starts_with("# ")).count(),
        1
    );
    assert!(actual.contains("## Section"));
    assert!(actual.contains("## Nested"));
}

#[test]
fn markdown_writer_makes_duplicate_headings_unique() {
    let ast = vec![
        AstNode::Heading {
            level: 2,
            text: "5.1 既存システム見直し方針".to_string(),
        },
        AstNode::Heading {
            level: 2,
            text: "5.1 既存システム見直し方針".to_string(),
        },
    ];

    let actual = markdown::write_markdown(&ast, Flavor::Markdownlint);

    assert!(actual.contains("# 5.1 既存システム見直し方針\n"));
    assert!(actual.contains("## 5.1 既存システム見直し方針 (2)\n"));
}

#[test]
fn markdown_writer_merges_line_breaks_inside_sentences() {
    let ast = vec![
        AstNode::Heading {
            level: 1,
            text: "Document".to_string(),
        },
        AstNode::Paragraph("資料内の価格とAWS公式ウェブサイト記載の価格に相".to_string()),
        AstNode::Paragraph("違があった場合".to_string()),
        AstNode::Paragraph("Spring web".to_string()),
        AstNode::Paragraph("application continues.".to_string()),
        AstNode::Paragraph("次の文です。".to_string()),
    ];

    let actual = markdown::write_markdown(&ast, Flavor::Markdownlint);

    assert!(actual.contains("価格に相違があった場合 Spring web application continues."));
    assert!(actual.contains("continues.\n\n次の文です。"));
}

#[test]
fn markdownlint_writer_escapes_pdf_text_that_looks_like_markdown_syntax() {
    let ast = vec![
        AstNode::Heading {
            level: 1,
            text: "Document".to_string(),
        },
        AstNode::Paragraph("Copyright © 2021 Example, Inc.".to_string()),
        AstNode::Paragraph("www.example.com (cm)[sepal_length] <IrisRecord>".to_string()),
        AstNode::Paragraph("Contact:info@example.com".to_string()),
        AstNode::Paragraph("[5.1, 3.5, 1.4, 0.2]:Iris-setosa".to_string()),
        AstNode::Paragraph("#not-a-heading".to_string()),
        AstNode::Paragraph("5.".to_string()),
    ];

    let actual = markdown::write_markdown(&ast, Flavor::Markdownlint);

    assert!(actual.starts_with("# Document\n\n"));
    assert!(actual.contains("<www.example.com>"));
    assert!(actual.contains("Contact:<info@example.com>"));
    assert!(actual.contains("(cm)\\[sepal\\_length\\]"));
    assert!(actual.contains("&lt;IrisRecord&gt;"));
    assert!(actual.contains("\\[5.1, 3.5, 1.4, 0.2\\]:Iris-setosa"));
    assert!(actual.contains("\\#not-a-heading"));
    assert!(actual.contains("5\\."));
}

#[test]
fn markdownlint_writer_uses_stable_ordered_list_prefixes() {
    let ast = vec![
        AstNode::Heading {
            level: 3,
            text: "Document".to_string(),
        },
        AstNode::List {
            ordered: true,
            items: vec![
                vec![AstNode::Text("First".to_string())],
                vec![AstNode::Text("Second".to_string())],
            ],
        },
    ];

    let actual = markdown::write_markdown(&ast, Flavor::Markdownlint);

    assert!(actual.contains("1. First\n1. Second\n"));
}

#[test]
fn markdown_writer_normalizes_heading_order_and_trailing_punctuation() {
    let ast = vec![
        AstNode::Heading {
            level: 1,
            text: "Document".to_string(),
        },
        AstNode::Heading {
            level: 4,
            text: "Skipped level.".to_string(),
        },
    ];

    let actual = markdown::write_markdown(&ast, Flavor::Markdownlint);

    assert!(actual.contains("# Document\n"));
    assert!(actual.contains("## Skipped level\n"));
}

#[test]
fn converts_simple_html_to_markdown() {
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        format: OutputFormat::Markdown,
        ..ConversionOptions::default()
    });

    let result = converter
        .convert_bytes("sample.html", b"<h1>Title</h1><p>Body</p>")
        .unwrap();

    assert_eq!(result.markdown, "# Title\n\nBody\n");
    assert_eq!(result.report.input_format, "html");
}

#[test]
fn converts_html_image_to_markdown_image() {
    let mut warnings = Vec::new();
    let ast = anything_to_markdown::html::parse_html(
        r#"<main><img src="media/chart.png" alt="Chart" title="Figure 1"></main>"#,
        &mut warnings,
    );
    let markdown = markdown::write_markdown(&ast, Flavor::Gfm);

    assert_eq!(
        ast,
        vec![AstNode::Image {
            alt: "Chart".to_string(),
            path: "media/chart.png".to_string(),
            title: Some("Figure 1".to_string()),
        }]
    );
    assert_eq!(markdown, "![Chart](media/chart.png \"Figure 1\")\n");
}

#[test]
fn markdown_writer_wraps_complex_image_destinations() {
    let ast = vec![AstNode::Image {
        alt: "image".to_string(),
        path: "画面設計書_C0300_お知らせ(一覧)/image2.png".to_string(),
        title: None,
    }];

    let markdown = markdown::write_markdown(&ast, Flavor::Gfm);

    assert_eq!(
        markdown,
        "![image](<画面設計書_C0300_お知らせ(一覧)/image2.png>)\n"
    );
}

#[test]
fn warns_when_image_caption_is_missing() {
    let mut warnings = Vec::new();
    let ast = anything_to_markdown::html::parse_html(
        r#"<img src="media/chart.png" alt="Chart">"#,
        &mut warnings,
    );

    assert_eq!(
        ast,
        vec![AstNode::Image {
            alt: "Chart".to_string(),
            path: "media/chart.png".to_string(),
            title: None,
        }]
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("caption inference failed"))
    );
}

#[test]
fn conversion_report_lists_referenced_media() {
    let converter = Converter::new();

    let result = converter
        .convert_bytes(
            "page.html",
            br#"<img src="media/chart.png" alt="Chart" title="Figure 1">"#,
        )
        .unwrap();

    assert_eq!(result.report.media, vec!["media/chart.png".to_string()]);
    assert_eq!(result.report.media_candidates.len(), 1);
    assert_eq!(
        result.report.media_candidates[0].caption.as_deref(),
        Some("Figure 1")
    );
}

#[test]
fn conversion_report_json_lists_used_features() {
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        ocr: anything_to_markdown::OcrEngine::NdlOcrLite,
        extract_media: Some("target/media-report-test".into()),
        ..ConversionOptions::default()
    });

    let result = converter
        .convert_bytes(
            "page.html",
            br#"<img src="media/chart.png" alt="Chart" title="Figure 1">"#,
        )
        .unwrap();
    let report = result.report.to_json();

    assert!(report.contains("\"features\""));
    assert!(report.contains("\"ocr:ndlocr-lite\""));
    assert!(report.contains("\"extract_media\""));
    assert!(report.contains("\"media:referenced\""));
    assert!(report.contains("\"media_candidates\""));
    assert!(report.contains("\"caption\":\"Figure 1\""));
}

#[test]
fn pdf_conversion_report_records_backend_and_ocr_requirement() {
    let converter = Converter::new();
    let bytes = minimal_text_pdf();

    let result = converter.convert_bytes("text-heading.pdf", &bytes).unwrap();

    assert!(
        result
            .report
            .metadata
            .iter()
            .any(|(key, value)| { key == "pdf_backend" && value == "atom-pdf-text" })
    );
    assert!(
        result
            .report
            .metadata
            .iter()
            .any(|(key, value)| { key == "pdf_ocr_required" && value == "false" })
    );
    assert!(
        result
            .report
            .features
            .iter()
            .any(|feature| { feature == "pdf_backend:atom-pdf-text" })
    );
}

#[test]
fn pdf_conversion_errors_when_non_encrypted_pdf_has_unknown_no_text_cause() {
    let converter = Converter::new();

    let error = converter
        .convert_bytes("empty.pdf", b"%PDF-1.7")
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("PDF text extraction produced no text")
    );
    assert!(error.to_string().contains("non-encrypted PDF"));
    assert!(error.to_string().contains("cause could not be classified"));
}

#[test]
fn pdf_conversion_errors_when_non_encrypted_pdf_is_image_only() {
    let converter = Converter::new();

    let error = converter
        .convert_bytes(
            "image-only.pdf",
            b"%PDF-1.7\n1 0 obj\n<</ProcSet[/PDF/ImageB]/XObject<</Im0 2 0 R>>>>\nendobj\n2 0 obj\n<</Subtype/Image/Type/XObject>>\nendobj",
        )
        .unwrap_err();

    assert!(error.to_string().contains("PDF contains page images"));
    assert!(error.to_string().contains("no extractable text layer"));
}

#[test]
fn pdf_conversion_errors_when_non_encrypted_pdf_lacks_unicode_maps() {
    let converter = Converter::new();

    let error = converter
        .convert_bytes(
            "font-without-unicode.pdf",
            b"%PDF-1.7\n1 0 obj\n<</ProcSet[/PDF/Text]/Font<</F1 2 0 R>>>>\nendobj\n2 0 obj\n<</Type/Font/Subtype/Type0/Encoding/Identity-H>>\nendobj",
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("embedded fonts without Unicode maps")
    );
}

#[test]
fn pdf_conversion_errors_when_text_extraction_has_no_text_and_pdf_is_encrypted() {
    let converter = Converter::new();

    let error = converter
        .convert_bytes("encrypted.pdf", b"%PDF-1.7\ntrailer\n<</Encrypt 1 0 R>>")
        .unwrap_err();

    assert!(error.to_string().contains("the PDF is encrypted"));
    assert!(
        error
            .to_string()
            .contains("PDF text extraction produced no text")
    );
}

#[test]
fn pdf_conversion_describes_standard_security_handler_without_claiming_password_required() {
    let converter = Converter::new();

    let error = converter
        .convert_bytes(
            "permission-protected.pdf",
            b"%PDF-1.7\ntrailer\n<</Encrypt 1 0 R>>\n1 0 obj\n<</Filter/Standard/P -1324>>\nendobj",
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("Standard security handler and may be permission-restricted")
    );
    assert!(!error.to_string().contains("Decrypt the PDF"));
}

fn minimal_text_pdf() -> Vec<u8> {
    let content = "BT\n/F1 24 Tf\n72 720 Td\n(Fixture Title) Tj\n/F1 11 Tf\n0 -24 Td\n(Fixture body text.) Tj\nET\n";
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
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

#[test]
fn converts_html_in_document_order_and_ignores_scripts() {
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        format: OutputFormat::Markdown,
        ..ConversionOptions::default()
    });

    let result = converter
        .convert_bytes(
            "sample.html",
            br#"
              <h1>Title</h1>
              <script>alert("skip")</script>
              <p>Intro</p>
              <ul><li>First</li><li>Second</li></ul>
              <pre><code>cargo test</code></pre>
              <h2>Next</h2>
            "#,
        )
        .unwrap();

    assert_eq!(
        result.markdown,
        "# Title\n\nIntro\n\n- First\n- Second\n\n```text\ncargo test\n```\n\n## Next\n"
    );
}

#[test]
fn falls_back_to_html_table_for_complex_tables() {
    let ast = vec![AstNode::Table {
        rows: vec![TableRow {
            cells: vec![TableCell {
                text: "A".to_string(),
                rowspan: 2,
                colspan: 1,
                image: None,
            }],
        }],
    }];

    let actual = markdown::write_markdown(&ast, Flavor::Gfm);

    assert!(actual.contains("<table>"));
    assert!(actual.contains("rowspan=\"2\""));
}

#[test]
fn falls_back_to_html_table_for_image_cells() {
    let ast = vec![AstNode::Table {
        rows: vec![TableRow {
            cells: vec![TableCell {
                text: "Diagram".to_string(),
                rowspan: 1,
                colspan: 1,
                image: Some("media/diagram.png".to_string()),
            }],
        }],
    }];

    let actual = markdown::write_markdown(&ast, Flavor::Gfm);

    assert!(actual.starts_with("<table>"));
    assert!(actual.contains("<img src=\"media/diagram.png\" alt=\"Diagram\">"));
}

#[test]
fn markdown_flavor_controls_simple_table_output() {
    let ast = vec![AstNode::Table {
        rows: vec![
            TableRow {
                cells: vec![
                    TableCell {
                        text: "Name".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: None,
                    },
                    TableCell {
                        text: "Value".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: None,
                    },
                ],
            },
            TableRow {
                cells: vec![
                    TableCell {
                        text: "Alpha".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: None,
                    },
                    TableCell {
                        text: "1".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: None,
                    },
                ],
            },
        ],
    }];

    let gfm = markdown::write_markdown(&ast, Flavor::Gfm);
    let commonmark = markdown::write_markdown(&ast, Flavor::CommonMark);

    assert!(gfm.starts_with("| Name | Value |"));
    assert!(commonmark.starts_with("<table>"));
}

#[test]
fn reports_unsupported_formats_without_external_calls() {
    let converter = Converter::new();
    let result = converter
        .convert_bytes("slides.pptx", b"not a real pptx")
        .unwrap();

    assert!(
        result
            .report
            .warnings
            .iter()
            .any(|warning| warning.contains("could not read"))
    );
    assert!(result.markdown.contains("Unsupported input format"));
    assert!(!result.report.used_llm);
}

#[test]
fn converts_markdown_input_as_passthrough() {
    let converter = Converter::new();
    let result = converter
        .convert_bytes("notes.md", b"# Already markdown")
        .unwrap();

    assert_eq!(result.report.input_format, "md");
    assert_eq!(result.markdown, "# Already markdown\n");
    assert!(result.report.warnings.is_empty());
}

#[test]
fn converts_text_like_inputs_to_markdown() {
    let converter = Converter::new();
    let result = converter
        .convert_bytes("links.gdoc", br#"{"url":"https://example.com"}"#)
        .unwrap();

    assert_eq!(result.report.input_format, "gdoc");
    assert!(result.markdown.starts_with("```json"));
    assert!(!result.markdown.contains("Unsupported input format"));
}

#[test]
fn converts_pdf_compatible_ai_file_with_pdf_parser() {
    let root = Path::new("target/ai-pdf-compatible-test");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).unwrap();
    let ai = root.join("sample.ai");
    fs::copy("tests/fixtures/unit/pdf/text-heading.pdf", &ai).unwrap();

    let result = Converter::new().convert_file(&ai).unwrap();

    assert_eq!(result.report.input_format, "ai");
    assert!(result.markdown.contains("Fixture Title"));
    assert!(!result.markdown.contains("Unsupported input format"));
    assert!(
        result
            .report
            .metadata
            .iter()
            .any(|(key, value)| key == "container_format" && value == "pdf")
    );
}

#[test]
fn converts_pptx_slide_xml_bytes_to_markdown() {
    let slide = include_bytes!("fixtures/unit/pptx/simple-slide.slide.xml");
    let expected = include_str!("fixtures/unit/pptx/simple-slide.expected.md");
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::CommonMark,
        ..ConversionOptions::default()
    });
    let result = converter.convert_bytes("slides.pptx", slide).unwrap();

    assert_eq!(result.markdown, expected);
    assert_eq!(result.report.input_format, "pptx");
}

#[test]
fn converts_xlsx_sheet_xml_bytes_to_markdown_table() {
    let sheet = include_bytes!("fixtures/unit/xlsx/simple-sheet.worksheet.xml");
    let expected = include_str!("fixtures/unit/xlsx/simple-sheet.expected.md");
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        ..ConversionOptions::default()
    });
    let result = converter.convert_bytes("sheet.xlsx", sheet).unwrap();

    assert_eq!(result.markdown, expected);
    assert_eq!(result.report.input_format, "xlsx");
}

#[test]
fn converts_realistic_pptx_slide_xml_fixture_to_markdown() {
    let slide = include_bytes!("fixtures/unit/pptx/meeting-slide.slide.xml");
    let expected = include_str!("fixtures/unit/pptx/meeting-slide.expected.md");
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::CommonMark,
        ..ConversionOptions::default()
    });
    let result = converter.convert_bytes("meeting.pptx", slide).unwrap();

    assert_eq!(result.markdown, expected);
    assert_eq!(result.report.input_format, "pptx");
}

#[test]
fn converts_realistic_xlsx_sheet_xml_fixture_to_markdown() {
    let sheet = include_bytes!("fixtures/unit/xlsx/budget-sheet.worksheet.xml");
    let expected = include_str!("fixtures/unit/xlsx/budget-sheet.expected.md");
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        ..ConversionOptions::default()
    });
    let result = converter.convert_bytes("budget.xlsx", sheet).unwrap();

    assert_eq!(result.markdown, expected);
    assert_eq!(result.report.input_format, "xlsx");
}

#[test]
fn parses_docx_tables_images_and_captions_from_xml() {
    let xml = include_str!("fixtures/unit/docx/table-image-caption.document.xml");
    let rels = include_str!("fixtures/unit/docx/table-image-caption.rels.xml");
    let expected = include_str!("fixtures/unit/docx/table-image-caption.expected.md");
    let mut warnings = Vec::new();

    let ast = docx::parse_document_xml_with_rels(xml, rels, &mut warnings);
    let rendered = markdown::write_markdown(&ast, Flavor::Gfm);

    assert_eq!(rendered, expected);
}

#[test]
fn parses_docx_tables_in_document_order() {
    let xml = r#"
        <w:document><w:body>
          <w:p><w:r><w:t>Before table</w:t></w:r></w:p>
          <w:tbl>
            <w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr>
          </w:tbl>
          <w:p><w:r><w:t>After table</w:t></w:r></w:p>
        </w:body></w:document>
    "#;
    let mut warnings = Vec::new();

    let ast = docx::parse_document_xml(xml, &mut warnings);

    assert_eq!(
        ast,
        vec![
            AstNode::Paragraph("Before table".to_string()),
            AstNode::Table {
                rows: vec![TableRow {
                    cells: vec![TableCell {
                        text: "Cell".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: None,
                    }],
                }],
            },
            AstNode::Paragraph("After table".to_string()),
        ]
    );
}

#[test]
fn parses_docx_after_self_closing_paragraphs() {
    let xml = r#"
        <w:document><w:body>
          <w:p w:rsidR="1"/>
          <w:p><w:r><w:t>Visible paragraph</w:t></w:r></w:p>
        </w:body></w:document>
    "#;
    let mut warnings = Vec::new();

    let ast = docx::parse_document_xml(xml, &mut warnings);

    assert_eq!(
        ast,
        vec![AstNode::Paragraph("Visible paragraph".to_string())]
    );
}

#[test]
fn parses_docx_vml_images_through_relationships() {
    let xml = r#"
        <w:document><w:body>
          <w:p>
            <w:r><w:pict><v:shape><v:imagedata r:id="rVml"/></v:shape></w:pict></w:r>
          </w:p>
          <w:tbl>
            <w:tr><w:tc><w:p><w:r><w:pict><v:shape><v:imagedata r:id="rCell"/></v:shape></w:pict></w:r></w:p></w:tc></w:tr>
          </w:tbl>
        </w:body></w:document>
    "#;
    let rels = r#"
        <Relationships>
          <Relationship Id="rVml" Target="media/vml.png"/>
          <Relationship Id="rCell" Target="media/cell.png"/>
        </Relationships>
    "#;
    let mut warnings = Vec::new();

    let ast = docx::parse_document_xml_with_rels(xml, rels, &mut warnings);

    assert_eq!(
        ast,
        vec![
            AstNode::Image {
                alt: "image".to_string(),
                path: "media/vml.png".to_string(),
                title: None,
            },
            AstNode::Table {
                rows: vec![TableRow {
                    cells: vec![TableCell {
                        text: "".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: Some("media/cell.png".to_string()),
                    }],
                }],
            },
        ]
    );
}

#[test]
fn extracts_docx_media_files_when_requested() {
    let package_path = Path::new("target/docx-media-extract-test/input.docx");
    let media_dir = Path::new("target/docx-media-extract-test/output");
    let _ = fs::remove_dir_all("target/docx-media-extract-test");
    fs::create_dir_all(package_path.parent().unwrap()).unwrap();

    let mut bytes = Cursor::new(Vec::new());
    let mut archive = ZipWriter::new(&mut bytes);
    let options = SimpleFileOptions::default();
    archive.start_file("word/document.xml", options).unwrap();
    archive
        .write_all(
            br#"<w:document><w:body><w:p><w:r><w:drawing><a:blip r:embed="rImg"/></w:drawing></w:r></w:p></w:body></w:document>"#,
        )
        .unwrap();
    archive
        .start_file("word/_rels/document.xml.rels", options)
        .unwrap();
    archive
        .write_all(br#"<Relationships><Relationship Id="rImg" Target="media/image1.png"/></Relationships>"#)
        .unwrap();
    archive
        .start_file("word/media/image1.png", options)
        .unwrap();
    archive.write_all(b"png-bytes").unwrap();
    archive.finish().unwrap();
    fs::write(package_path, bytes.into_inner()).unwrap();

    let converter = Converter::new().with_options(ConversionOptions {
        extract_media: Some(media_dir.to_path_buf()),
        ..ConversionOptions::default()
    });
    let result = converter.convert_file(package_path).unwrap();

    assert!(result.markdown.contains("output/image1.png"));
    assert!(!result.markdown.contains("word/media/image1.png"));
    assert_eq!(
        fs::read(media_dir.join("image1.png")).unwrap(),
        b"png-bytes"
    );
}

#[test]
fn embeds_docx_media_as_base64_when_requested() {
    let package_path = Path::new("target/docx-media-inline-test/input.docx");
    let _ = fs::remove_dir_all("target/docx-media-inline-test");
    fs::create_dir_all(package_path.parent().unwrap()).unwrap();

    let mut bytes = Cursor::new(Vec::new());
    let mut archive = ZipWriter::new(&mut bytes);
    let options = SimpleFileOptions::default();
    archive.start_file("word/document.xml", options).unwrap();
    archive
        .write_all(
            br#"<w:document><w:body><w:p><w:r><w:drawing><a:blip r:embed="rImg"/></w:drawing></w:r></w:p></w:body></w:document>"#,
        )
        .unwrap();
    archive
        .start_file("word/_rels/document.xml.rels", options)
        .unwrap();
    archive
        .write_all(br#"<Relationships><Relationship Id="rImg" Target="media/image1.png"/></Relationships>"#)
        .unwrap();
    archive
        .start_file("word/media/image1.png", options)
        .unwrap();
    archive.write_all(b"png-bytes").unwrap();
    archive.finish().unwrap();
    fs::write(package_path, bytes.into_inner()).unwrap();

    let converter = Converter::new().with_options(ConversionOptions {
        inline_base64_media: true,
        ..ConversionOptions::default()
    });
    let result = converter.convert_file(package_path).unwrap();

    assert!(
        result
            .markdown
            .contains("data:image/png;base64,cG5nLWJ5dGVz")
    );
    assert!(!result.markdown.contains("media/image1.png"));
}

#[test]
fn parses_docx_heading_paragraph_list_fixture() {
    let xml = include_str!("fixtures/unit/docx/heading-paragraph-list.document.xml");
    let expected = include_str!("fixtures/unit/docx/heading-paragraph-list.expected.md");
    let mut warnings = Vec::new();

    let ast = docx::parse_document_xml(xml, &mut warnings);
    let rendered = markdown::write_markdown(&ast, Flavor::CommonMark);

    assert_eq!(rendered, expected);
}

#[test]
fn parses_docx_hyperlinks_footnotes_and_comments() {
    let xml = r#"
        <w:document><w:body>
          <w:p>
            <w:r><w:t>See reference </w:t></w:r>
            <w:hyperlink r:id="rLink"><w:r><w:t>site</w:t></w:r></w:hyperlink>
            <w:r><w:footnoteReference w:id="2"/></w:r>
            <w:r><w:commentReference w:id="4"/></w:r>
          </w:p>
        </w:body></w:document>
    "#;
    let rels =
        r#"<Relationships><Relationship Id="rLink" Target="https://example.com"/></Relationships>"#;
    let footnotes = r#"
        <w:footnotes>
          <w:footnote w:id="2"><w:p><w:r><w:t>Footnote text</w:t></w:r></w:p></w:footnote>
        </w:footnotes>
    "#;
    let comments = r#"
        <w:comments>
          <w:comment w:id="4"><w:p><w:r><w:t>Comment text</w:t></w:r></w:p></w:comment>
        </w:comments>
    "#;
    let mut warnings = Vec::new();

    let ast =
        docx::parse_document_xml_with_rels_and_notes(xml, rels, footnotes, comments, &mut warnings);

    assert_eq!(
        ast,
        vec![
            AstNode::Paragraph("See reference site <https://example.com>".to_string()),
            AstNode::Footnote {
                label: "2".to_string(),
                text: "Footnote text".to_string(),
            },
            AstNode::Footnote {
                label: "comment:4".to_string(),
                text: "Comment text".to_string(),
            },
        ]
    );
}

#[test]
fn parses_xlsx_and_pptx_xml_to_structured_ast() {
    let shared = include_str!("fixtures/unit/xlsx/shared-strings.xml");
    let sheet = include_str!("fixtures/unit/xlsx/shared-string-sheet.worksheet.xml");
    let slide = include_str!("fixtures/unit/pptx/simple-slide.slide.xml");

    let xlsx = ooxml::parse_xlsx_sheet_xml(sheet, shared);
    let pptx = ooxml::parse_pptx_slide_xml(slide);

    assert!(matches!(xlsx.first(), Some(AstNode::Table { .. })));
    assert_eq!(
        markdown::write_markdown(&pptx, Flavor::CommonMark),
        include_str!("fixtures/unit/pptx/simple-slide.expected.md")
    );
}

#[test]
fn exposes_docx_parser_under_ooxml_namespace() {
    let xml = include_str!("fixtures/unit/docx/heading-paragraph-list.document.xml");
    let mut warnings = Vec::new();

    let ast = ooxml::docx::parse_document_xml(xml, &mut warnings);

    assert!(matches!(ast.first(), Some(AstNode::Heading { .. })));
}

#[test]
fn parses_pptx_visual_order_from_shape_coordinates() {
    let slide = include_str!("fixtures/unit/pptx/visual-order-shapes.slide.xml");
    let expected = include_str!("fixtures/unit/pptx/visual-order-shapes.expected.md");
    let mut warnings = Vec::new();

    let ast = ooxml::parse_pptx_slide_xml_with_rels(slide, "", &mut warnings);
    let rendered = markdown::write_markdown(&ast, Flavor::CommonMark);

    assert_eq!(rendered, expected);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("visual order"))
    );
}

#[test]
fn parses_pptx_split_runs_without_breaking_japanese_words() {
    let slide = include_str!("fixtures/unit/pptx/split-run-japanese.slide.xml");
    let expected = include_str!("fixtures/unit/pptx/split-run-japanese.expected.md");
    let mut warnings = Vec::new();

    let ast = ooxml::parse_pptx_slide_xml_with_rels(slide, "", &mut warnings);
    let rendered = markdown::write_markdown(&ast, Flavor::CommonMark);

    assert_eq!(rendered, expected);
}

#[test]
fn parses_pptx_body_bullets_as_markdown_list() {
    let slide = r#"
<p:sld>
  <p:cSld><p:spTree>
    <p:sp>
      <p:nvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>
      <p:txBody><a:p><a:r><a:t>VPN Setup</a:t></a:r></a:p></p:txBody>
    </p:sp>
    <p:sp>
      <p:nvSpPr><p:nvPr><p:ph type="body"/></p:nvPr></p:nvSpPr>
      <p:txBody>
        <a:p><a:pPr><a:buChar char="•"/></a:pPr><a:r><a:t>Install client</a:t></a:r></a:p>
        <a:p><a:pPr><a:buChar char="•"/></a:pPr><a:r><a:t>Import certificate</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree></p:cSld>
</p:sld>
"#;
    let mut warnings = Vec::new();

    let ast = ooxml::parse_pptx_slide_xml_with_rels(slide, "", &mut warnings);

    assert_eq!(
        ast,
        vec![
            AstNode::Heading {
                level: 1,
                text: "VPN Setup".to_string(),
            },
            AstNode::List {
                ordered: false,
                items: vec![
                    vec![AstNode::Text("Install client".to_string())],
                    vec![AstNode::Text("Import certificate".to_string())],
                ],
            },
        ]
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("pptx list structure restored"))
    );
}

#[test]
fn parses_xlsx_merged_headers_inline_strings_and_formula_values() {
    let sheet = include_str!("fixtures/unit/xlsx/merged-header-sheet.worksheet.xml");
    let shared = include_str!("fixtures/unit/xlsx/merged-header-sheet.shared-strings.xml");
    let expected = include_str!("fixtures/unit/xlsx/merged-header-sheet.expected.md");
    let mut warnings = Vec::new();

    let ast = ooxml::parse_xlsx_sheet_xml_with_warnings(sheet, shared, &mut warnings);
    let rendered = markdown::write_markdown(&ast, Flavor::Gfm);

    assert_eq!(rendered, expected);
    assert!(warnings.iter().any(|warning| warning.contains("formula")));
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("mergeCells"))
    );
}

#[test]
fn parses_xlsx_shared_strings_without_phonetic_readings() {
    let sheet = include_str!("fixtures/unit/xlsx/phonetic-shared-string.worksheet.xml");
    let shared = include_str!("fixtures/unit/xlsx/phonetic-shared-string.shared-strings.xml");
    let expected = include_str!("fixtures/unit/xlsx/phonetic-shared-string.expected.md");
    let mut warnings = Vec::new();

    let ast = ooxml::parse_xlsx_sheet_xml_with_warnings(sheet, shared, &mut warnings);
    let rendered = markdown::write_markdown(&ast, Flavor::Gfm);

    assert_eq!(rendered, expected);
}

#[test]
fn trims_xlsx_empty_table_edges() {
    let sheet = r#"
        <worksheet><sheetData>
          <row r="1"><c r="A1"><v></v></c><c r="B1"><v></v></c><c r="C1"><v></v></c></row>
          <row r="2"><c r="A2"><v></v></c><c r="B2" t="inlineStr"><is><t>Name</t></is></c><c r="C2" t="inlineStr"><is><t>Value</t></is></c></row>
          <row r="3"><c r="A3"><v></v></c><c r="B3" t="inlineStr"><is><t>VPN</t></is></c><c r="C3"><v>1</v></c></row>
          <row r="4"><c r="A4"><v></v></c><c r="B4"><v></v></c><c r="C4"><v></v></c></row>
        </sheetData></worksheet>
    "#;

    let ast = ooxml::parse_xlsx_sheet_xml(sheet, "");

    assert_eq!(
        ast,
        vec![AstNode::Table {
            rows: vec![
                TableRow {
                    cells: vec![
                        TableCell {
                            text: "Name".to_string(),
                            rowspan: 1,
                            colspan: 1,
                            image: None,
                        },
                        TableCell {
                            text: "Value".to_string(),
                            rowspan: 1,
                            colspan: 1,
                            image: None,
                        },
                    ],
                },
                TableRow {
                    cells: vec![
                        TableCell {
                            text: "VPN".to_string(),
                            rowspan: 1,
                            colspan: 1,
                            image: None,
                        },
                        TableCell {
                            text: "1".to_string(),
                            rowspan: 1,
                            colspan: 1,
                            image: None,
                        },
                    ],
                },
            ],
        }]
    );
}

#[test]
fn converts_xlsx_multiple_sheets_with_sheet_headings() {
    let root = Path::new("target/xlsx-multiple-sheets-test");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root.join("xl/worksheets")).unwrap();
    fs::write(
        root.join("xl/worksheets/sheet1.xml"),
        r#"<worksheet><sheetData><row><c t="inlineStr"><is><t>Name</t></is></c></row></sheetData></worksheet>"#,
    )
    .unwrap();
    fs::write(
        root.join("xl/worksheets/sheet2.xml"),
        r#"<worksheet><sheetData><row><c t="inlineStr"><is><t>Status</t></is></c></row></sheetData></worksheet>"#,
    )
    .unwrap();
    let xlsx = root.join("sample.xlsx");
    zip_fixture(
        root,
        &xlsx,
        &["xl/worksheets/sheet1.xml", "xl/worksheets/sheet2.xml"],
    );

    let result = Converter::new().convert_file(&xlsx).unwrap();

    assert!(result.markdown.contains("# Sheet 1"));
    assert!(result.markdown.contains("# Sheet 2"));
    assert!(
        result
            .report
            .metadata
            .iter()
            .any(|(key, value)| key == "worksheets" && value == "2")
    );
}

#[test]
fn converts_xlsm_with_xlsx_worksheet_parser() {
    let root = Path::new("target/xlsm-parser-test");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root.join("xl/worksheets")).unwrap();
    fs::write(
        root.join("xl/worksheets/sheet1.xml"),
        r#"<worksheet><sheetData><row><c t="inlineStr"><is><t>Status</t></is></c></row></sheetData></worksheet>"#,
    )
    .unwrap();
    let xlsm = root.join("sample.xlsm");
    zip_fixture(root, &xlsm, &["xl/worksheets/sheet1.xml"]);

    let result = Converter::new().convert_file(&xlsm).unwrap();

    assert_eq!(result.report.input_format, "xlsm");
    assert!(result.markdown.contains("Status"));
    assert!(!result.markdown.contains("Unsupported input format"));
}

#[test]
fn converts_ooxml_packages_with_parts_and_relationships() {
    let root = Path::new("target/ooxml-package-test");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root.join("ppt/slides/_rels")).unwrap();
    fs::create_dir_all(root.join("xl/worksheets")).unwrap();
    fs::create_dir_all(root.join("xl")).unwrap();
    fs::write(
        root.join("ppt/slides/slide1.xml"),
        include_str!("fixtures/unit/pptx/visual-order-shapes.slide.xml"),
    )
    .unwrap();
    fs::write(
        root.join("ppt/slides/_rels/slide1.xml.rels"),
        "<Relationships></Relationships>",
    )
    .unwrap();
    fs::write(
        root.join("xl/worksheets/sheet1.xml"),
        include_str!("fixtures/unit/xlsx/merged-header-sheet.worksheet.xml"),
    )
    .unwrap();
    fs::write(
        root.join("xl/sharedStrings.xml"),
        include_str!("fixtures/unit/xlsx/merged-header-sheet.shared-strings.xml"),
    )
    .unwrap();

    let pptx = root.join("sample.pptx");
    let xlsx = root.join("sample.xlsx");
    zip_fixture(
        root,
        &pptx,
        &["ppt/slides/slide1.xml", "ppt/slides/_rels/slide1.xml.rels"],
    );
    zip_fixture(
        root,
        &xlsx,
        &["xl/worksheets/sheet1.xml", "xl/sharedStrings.xml"],
    );

    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        ..ConversionOptions::default()
    });
    let pptx_result = converter.convert_file(&pptx).unwrap();
    let xlsx_result = converter.convert_file(&xlsx).unwrap();

    assert_eq!(pptx_result.report.input_format, "pptx");
    assert!(
        pptx_result
            .report
            .metadata
            .iter()
            .any(|(key, value)| key == "slides" && value == "1")
    );
    assert_eq!(xlsx_result.report.input_format, "xlsx");
    assert!(xlsx_result.markdown.contains("Region Summary"));
}

#[test]
fn converts_opendocument_package_content_xml_to_markdown() {
    let root = Path::new("target/opendocument-package-test");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join("content.xml"),
        r#"<office:document-content>
  <office:body>
    <office:text>
      <text:h text:outline-level="1">Report</text:h>
      <text:p>OpenDocument body</text:p>
      <table:table>
        <table:table-row>
          <table:table-cell><text:p>Name</text:p></table:table-cell>
          <table:table-cell><text:p>Status</text:p></table:table-cell>
        </table:table-row>
        <table:table-row>
          <table:table-cell><text:p>Atom</text:p></table:table-cell>
          <table:table-cell><text:p>Done</text:p></table:table-cell>
        </table:table-row>
      </table:table>
    </office:text>
  </office:body>
</office:document-content>"#,
    )
    .unwrap();
    let odt = root.join("sample.odt");
    zip_fixture(root, &odt, &["content.xml"]);

    let result = Converter::new().convert_file(&odt).unwrap();

    assert_eq!(result.report.input_format, "odt");
    assert!(result.markdown.contains("# Report"));
    assert!(result.markdown.contains("OpenDocument body"));
    assert!(result.markdown.contains("Atom"));
    assert!(!result.markdown.contains("Unsupported input format"));
}

#[test]
fn image_input_requires_vlm_backend_without_external_send_by_default() {
    let root = Path::new("target/image-input-vlm-test");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).unwrap();
    let image = root.join("scan.png");
    fs::write(&image, [0x89, b'P', b'N', b'G']).unwrap();

    let result = Converter::new().convert_file(&image).unwrap();

    assert_eq!(result.report.input_format, "png");
    assert!(
        result
            .markdown
            .contains("Unsupported input format: image input")
    );
    assert!(
        result
            .report
            .warnings
            .iter()
            .any(|warning| warning.contains("requires --llm"))
    );
}

#[test]
fn image_input_does_not_send_to_cloud_vlm_without_consent() {
    let root = Path::new("target/image-input-cloud-consent-test");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).unwrap();
    let image = root.join("scan.png");
    fs::write(&image, [0x89, b'P', b'N', b'G']).unwrap();

    let result = Converter::new()
        .with_options(ConversionOptions {
            llm: LlmBackend::OpenAi("gpt-4o".to_string()),
            ..Default::default()
        })
        .convert_file(&image)
        .unwrap();

    assert_eq!(result.report.input_format, "png");
    assert!(
        result
            .markdown
            .contains("Unsupported input format: image input")
    );
    assert!(
        result
            .report
            .warnings
            .iter()
            .any(|warning| warning.contains("external send consent is required"))
    );
    assert!(
        result
            .report
            .warnings
            .iter()
            .any(|warning| warning.contains("VLM image conversion skipped"))
    );
}

fn zip_fixture(root: &Path, output: &Path, parts: &[&str]) {
    let mut bytes = Cursor::new(Vec::new());
    let mut archive = ZipWriter::new(&mut bytes);
    let options = SimpleFileOptions::default();
    for part in parts {
        archive.start_file(*part, options).unwrap();
        let content = fs::read(root.join(part)).unwrap();
        archive.write_all(&content).unwrap();
    }
    archive.finish().unwrap();
    fs::write(output, bytes.into_inner()).unwrap();
}
