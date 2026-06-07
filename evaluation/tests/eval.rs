use anything_to_markdown::{AstNode, TableCell, TableRow};
use atom_evaluation::{
    OcrCerCase, evaluate_heading_recall, evaluate_lint_score, evaluate_ocr_cer_by_group,
    evaluate_structure_fidelity, evaluate_table_integrity, evaluate_translation_structure_preserve,
};

#[test]
fn structure_fidelity_detects_heading_level_and_table_cell_loss() {
    let expected = vec![
        AstNode::Heading {
            level: 1,
            text: "Title".to_string(),
        },
        AstNode::Paragraph("Body".to_string()),
        AstNode::Table {
            rows: vec![TableRow {
                cells: vec![
                    TableCell {
                        text: "A".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: None,
                    },
                    TableCell {
                        text: "B".to_string(),
                        rowspan: 1,
                        colspan: 1,
                        image: None,
                    },
                ],
            }],
        },
    ];
    let actual = vec![
        AstNode::Heading {
            level: 2,
            text: "Title".to_string(),
        },
        AstNode::Paragraph("Body".to_string()),
        AstNode::Table {
            rows: vec![TableRow {
                cells: vec![TableCell {
                    text: "A".to_string(),
                    rowspan: 1,
                    colspan: 1,
                    image: None,
                }],
            }],
        },
    ];

    let score = evaluate_structure_fidelity(&expected, &actual);

    assert_eq!(score.errors, 2);
    assert!((score.score - 0.6666666666666667).abs() < f64::EPSILON);
}

#[test]
fn structure_fidelity_penalizes_unexpected_extra_nodes() {
    let expected = vec![
        AstNode::Heading {
            level: 1,
            text: "Title".to_string(),
        },
        AstNode::Paragraph("Body".to_string()),
    ];
    let actual = vec![
        AstNode::Heading {
            level: 1,
            text: "Title".to_string(),
        },
        AstNode::Paragraph("Body".to_string()),
        AstNode::Paragraph("Unexpected".to_string()),
    ];

    let score = evaluate_structure_fidelity(&expected, &actual);

    assert_eq!(score.errors, 1);
    assert!((score.score - 0.6666666666666667).abs() < f64::EPSILON);
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("edit distance 1 over 3 structural node(s)"))
    );
}

#[test]
fn structure_fidelity_penalizes_changed_parent_child_relationships() {
    let expected = vec![AstNode::List {
        ordered: false,
        items: vec![vec![AstNode::List {
            ordered: false,
            items: vec![vec![AstNode::Text("Nested".to_string())]],
        }]],
    }];
    let actual = vec![AstNode::List {
        ordered: false,
        items: vec![vec![
            AstNode::List {
                ordered: false,
                items: vec![vec![]],
            },
            AstNode::Text("Nested".to_string()),
        ]],
    }];

    let score = evaluate_structure_fidelity(&expected, &actual);

    assert_eq!(score.errors, 2);
    assert!((score.score - 0.6).abs() < f64::EPSILON);
}

#[test]
fn lint_score_detects_common_markdown_layout_errors() {
    let markdown = "# Title\nBody\n- item\n| A | B |\n| --- | --- |\n";

    let score = evaluate_lint_score(markdown);

    assert_eq!(score.errors, 4);
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("heading"))
    );
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("list"))
    );
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("table"))
    );
}

#[test]
fn heading_recall_requires_heading_markers_levels_and_text() {
    let expected = vec![
        AstNode::Heading {
            level: 1,
            text: "Title".to_string(),
        },
        AstNode::Heading {
            level: 2,
            text: "Details".to_string(),
        },
        AstNode::List {
            ordered: false,
            items: vec![vec![AstNode::Heading {
                level: 3,
                text: "Nested".to_string(),
            }]],
        },
    ];
    let markdown = "# Title\n\n### Details\n\nNested\n";

    let score = evaluate_heading_recall(&expected, markdown);

    assert_eq!(score.errors, 2);
    assert!((score.score - 0.3333333333333333).abs() < f64::EPSILON);
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("h2 Details"))
    );
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("h3 Nested"))
    );
}

#[test]
fn table_integrity_detects_pipe_table_shape_errors() {
    let markdown = "| A | B |\n| --- | --- |\n| 1 |\n";

    let score = evaluate_table_integrity(markdown);

    assert_eq!(score.score, 0.0);
    assert_eq!(score.errors, 1);
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("pipe table row 3"))
    );
}

#[test]
fn table_integrity_accepts_complex_html_table_and_detects_unclosed_table() {
    let valid = "<table><tr><td rowspan=\"2\">A<br>B</td><td colspan=\"2\"><img src=\"x.png\" alt=\"x\"></td></tr></table>\n";
    let invalid = "<table><tr><td>A</td></tr>\n";

    assert_eq!(evaluate_table_integrity(valid).score, 1.0);

    let score = evaluate_table_integrity(invalid);

    assert_eq!(score.score, 0.0);
    assert_eq!(score.errors, 1);
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("unclosed html table"))
    );
}

#[test]
fn ocr_cer_can_be_aggregated_by_language_and_orientation() {
    let cases = vec![
        OcrCerCase {
            language: "ja".to_string(),
            orientation: "vertical".to_string(),
            expected: "日本語".to_string(),
            actual: "日本吾".to_string(),
        },
        OcrCerCase {
            language: "en".to_string(),
            orientation: "horizontal".to_string(),
            expected: "text".to_string(),
            actual: "text".to_string(),
        },
    ];

    let scores = evaluate_ocr_cer_by_group(&cases);

    assert_eq!(scores.len(), 2);
    assert_eq!(scores[0].name, "ocr_cer:en:horizontal");
    assert_eq!(scores[0].score, 1.0);
    assert_eq!(scores[1].name, "ocr_cer:ja:vertical");
    assert_eq!(scores[1].errors, 1);
    assert!((scores[1].score - 0.6666666666666667).abs() < f64::EPSILON);
}

#[test]
fn translation_structure_preserve_detects_code_block_loss() {
    let before = "# Title\n\n```rust\ncargo test\n```\n\n- item\n";
    let after = "# タイトル\n\ncargo test\n\n- 項目\n";

    let score = evaluate_translation_structure_preserve(before, after);

    assert!(score.errors >= 1);
    assert!(score.score < 1.0);
    assert!(
        score
            .warnings
            .iter()
            .any(|warning| warning.contains("code"))
    );
}
