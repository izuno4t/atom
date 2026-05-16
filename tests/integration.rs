use bonjil::{
    AstNode, ConversionOptions, Converter, Flavor, OutputFormat, TableCell, TableRow, docx,
    evaluate_heading_recall, evaluate_lint_score, evaluate_structure_fidelity,
    evaluate_table_integrity, evaluate_translation_structure_preserve, markdown, office,
};

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
    let ast = bonjil::html::parse_html(
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
fn warns_when_image_caption_is_missing() {
    let mut warnings = Vec::new();
    let ast = bonjil::html::parse_html(r#"<img src="media/chart.png" alt="Chart">"#, &mut warnings);

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
}

#[test]
fn conversion_report_json_lists_used_features() {
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        ocr: bonjil::OcrEngine::NdlOcrLite,
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
fn evaluation_functions_return_structured_scores() {
    let expected = vec![AstNode::Heading {
        level: 1,
        text: "Title".to_string(),
    }];
    let actual = expected.clone();
    let markdown = "# Title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n";

    assert_eq!(evaluate_structure_fidelity(&expected, &actual).score, 1.0);
    assert_eq!(evaluate_heading_recall(&expected, markdown).score, 1.0);
    assert_eq!(evaluate_table_integrity(markdown).score, 1.0);
    assert_eq!(evaluate_lint_score(markdown).errors, 0);
    assert_eq!(
        evaluate_translation_structure_preserve("# A\n\n- x\n", "# B\n\n- y\n").score,
        1.0
    );
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
fn converts_pptx_slide_xml_bytes_to_markdown() {
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::CommonMark,
        ..ConversionOptions::default()
    });
    let result = converter
        .convert_bytes(
            "slides.pptx",
            br#"<p:sld><p:cSld><p:spTree><a:p><a:r><a:t>Slide title</a:t></a:r></a:p><a:p><a:r><a:t>Slide body</a:t></a:r></a:p></p:spTree></p:cSld></p:sld>"#,
        )
        .unwrap();

    assert_eq!(result.markdown, "# Slide title\n\nSlide body\n");
    assert_eq!(result.report.input_format, "pptx");
}

#[test]
fn converts_xlsx_sheet_xml_bytes_to_markdown_table() {
    let converter = Converter::new().with_options(ConversionOptions {
        flavor: Flavor::Gfm,
        ..ConversionOptions::default()
    });
    let result = converter
        .convert_bytes(
            "sheet.xlsx",
            br#"<worksheet><sheetData><row><c><v>Name</v></c><c><v>Value</v></c></row><row><c><v>Alpha</v></c><c><v>42</v></c></row></sheetData></worksheet>"#,
        )
        .unwrap();

    assert_eq!(
        result.markdown,
        "| Name | Value |\n| --- | --- |\n| Alpha | 42 |\n"
    );
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
fn parses_docx_heading_paragraph_list_fixture() {
    let xml = include_str!("fixtures/unit/docx/heading-paragraph-list.document.xml");
    let expected = include_str!("fixtures/unit/docx/heading-paragraph-list.expected.md");
    let mut warnings = Vec::new();

    let ast = docx::parse_document_xml(xml, &mut warnings);
    let rendered = markdown::write_markdown(&ast, Flavor::CommonMark);

    assert_eq!(rendered, expected);
}

#[test]
fn parses_xlsx_and_pptx_xml_to_structured_ast() {
    let shared = r#"<sst><si><t>Name</t></si><si><t>Value</t></si></sst>"#;
    let sheet = r#"<worksheet><sheetData><row><c t="s"><v>0</v></c><c><v>42</v></c></row></sheetData></worksheet>"#;
    let slide = r#"<p:sld><p:cSld><p:spTree><a:p><a:r><a:t>Slide title</a:t></a:r></a:p></p:spTree></p:cSld></p:sld>"#;

    let xlsx = office::parse_xlsx_sheet_xml(sheet, shared);
    let pptx = office::parse_pptx_slide_xml(slide);

    assert!(matches!(xlsx.first(), Some(AstNode::Table { .. })));
    assert_eq!(
        markdown::write_markdown(&pptx, Flavor::CommonMark),
        "# Slide title\n"
    );
}
