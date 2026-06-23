use std::fs;
use std::io;
use std::path::Path;

use anything_to_markdown::llm::{self, LlmProvider, LlmRequest, LlmResponse};
use anything_to_markdown::{AstNode, LlmBackend, TableCell, TableRow};

struct StubLlm;

impl LlmProvider for StubLlm {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse> {
        Ok(LlmResponse {
            text: format!("{}: ok", request.task),
            backend: "stub".to_string(),
        })
    }
}

struct RestructureLlm;

impl LlmProvider for RestructureLlm {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse> {
        assert_eq!(request.task, "restructure");
        Ok(LlmResponse {
            text: "# Fixed Title\n\nFixed body.".to_string(),
            backend: "stub".to_string(),
        })
    }
}

struct TranslateLlm;

impl LlmProvider for TranslateLlm {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse> {
        assert_eq!(request.task, "translate:ja");
        Ok(LlmResponse {
            text: "# タイトル\n\n本文。".to_string(),
            backend: "stub".to_string(),
        })
    }
}

struct StructuredLlm;

impl LlmProvider for StructuredLlm {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse> {
        assert!(request.input.contains("- Source item"));
        assert!(request.input.contains("| Metric | Value |"));
        assert!(
            request
                .input
                .contains("![Chart](chart.png \"Quarterly chart\")")
        );
        assert!(request.input.contains("[^a]: Source note"));
        Ok(LlmResponse {
            text: r#"# Report

- Source item

| Metric | Value |
| --- | --- |
| Revenue | 42 |

![Chart](chart.png "Quarterly chart")

```text
code sample
```

[^a]: Source note"#
                .to_string(),
            backend: "stub".to_string(),
        })
    }
}

struct PromptCheckingVisionLlm;

impl LlmProvider for PromptCheckingVisionLlm {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse> {
        assert_eq!(request.task, "vision:describe");
        assert!(request.input.starts_with("Custom vision prompt:"));
        assert!(request.input.contains("scan.png"));
        assert_eq!(request.images.len(), 1);
        Ok(LlmResponse {
            text: "# Diagram\n\nA visible workflow diagram.".to_string(),
            backend: "stub".to_string(),
        })
    }
}

#[test]
fn llm_backend_boundary_accepts_replaceable_provider() {
    let response = llm::complete_with(
        &StubLlm,
        &LlmRequest {
            backend: LlmBackend::Ollama("llama3".to_string()),
            task: "restructure".to_string(),
            input: " body ".to_string(),
            images: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(response.text, "restructure: ok");
    assert_eq!(response.backend, "stub");
}

#[test]
fn llm_backend_names_cover_supported_providers() {
    assert_eq!(
        llm::backend_name(&LlmBackend::Anthropic("claude-opus".to_string())),
        "anthropic"
    );
    assert_eq!(
        llm::backend_name(&LlmBackend::OpenAi("gpt-4".to_string())),
        "openai"
    );
    assert_eq!(
        llm::backend_name(&LlmBackend::Ollama("llama3".to_string())),
        "ollama"
    );
    assert_eq!(
        llm::backend_name(&LlmBackend::OpenAiCompatible {
            name: "internal".to_string(),
            endpoint: "https://example.invalid".to_string(),
        }),
        "openai-compatible"
    );
}

#[test]
fn llm_send_confirmation_describes_destination_content_and_consent() {
    let confirmation = llm::build_send_confirmation(
        &LlmBackend::OpenAi("gpt-4".to_string()),
        "# Title\n\nBody",
        false,
    )
    .unwrap();

    assert_eq!(confirmation.destination, "OpenAI");
    assert_eq!(confirmation.content_bytes, 13);
    assert!(!confirmation.consent_granted);
    assert!(
        confirmation
            .message
            .contains("external send consent is required")
    );
}

#[test]
fn llm_restructure_filter_replaces_ast_from_markdown_response() {
    let ast = vec![AstNode::Paragraph("Fixed Title\nFixed body.".to_string())];

    let restructured = llm::restructure_with_provider(
        &RestructureLlm,
        &LlmBackend::Ollama("llama3".to_string()),
        &ast,
    )
    .unwrap();

    assert_eq!(
        restructured,
        vec![
            AstNode::Heading {
                level: 1,
                text: "Fixed Title".to_string(),
            },
            AstNode::Paragraph("Fixed body.".to_string()),
        ]
    );
}

#[test]
fn llm_translation_filter_replaces_ast_from_markdown_response() {
    let ast = vec![
        AstNode::Heading {
            level: 1,
            text: "Title".to_string(),
        },
        AstNode::Paragraph("Body.".to_string()),
    ];

    let translated = llm::translate_with_provider(
        &TranslateLlm,
        &LlmBackend::Ollama("llama3".to_string()),
        "ja",
        &ast,
    )
    .unwrap();

    assert_eq!(
        translated,
        vec![
            AstNode::Heading {
                level: 1,
                text: "タイトル".to_string(),
            },
            AstNode::Paragraph("本文。".to_string()),
        ]
    );
}

#[test]
fn llm_restructure_preserves_structural_attributes_in_prompt_and_response() {
    let ast = vec![
        AstNode::Heading {
            level: 1,
            text: "Report".to_string(),
        },
        AstNode::List {
            ordered: false,
            items: vec![vec![AstNode::Text("Source item".to_string())]],
        },
        AstNode::Table {
            rows: vec![
                TableRow {
                    cells: vec![
                        TableCell {
                            text: "Metric".to_string(),
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
                            text: "Revenue".to_string(),
                            rowspan: 1,
                            colspan: 1,
                            image: None,
                        },
                        TableCell {
                            text: "42".to_string(),
                            rowspan: 1,
                            colspan: 1,
                            image: None,
                        },
                    ],
                },
            ],
        },
        AstNode::Image {
            alt: "Chart".to_string(),
            path: "chart.png".to_string(),
            title: Some("Quarterly chart".to_string()),
        },
        AstNode::CodeBlock {
            language: Some("text".to_string()),
            code: "code sample".to_string(),
        },
        AstNode::Footnote {
            label: "a".to_string(),
            text: "Source note".to_string(),
        },
    ];

    let restructured = llm::restructure_with_provider(
        &StructuredLlm,
        &LlmBackend::Ollama("llama3".to_string()),
        &ast,
    )
    .unwrap();

    assert!(matches!(restructured[1], AstNode::List { .. }));
    assert!(matches!(restructured[2], AstNode::Table { .. }));
    assert!(matches!(restructured[3], AstNode::Image { .. }));
    assert!(matches!(restructured[4], AstNode::CodeBlock { .. }));
    assert!(matches!(restructured[5], AstNode::Footnote { .. }));
}

#[test]
fn image_description_uses_task_specific_prompt_configuration() {
    let mut prompts = std::collections::BTreeMap::new();
    prompts.insert(
        "image-description".to_string(),
        "Custom vision prompt: {input}".to_string(),
    );

    let nodes = llm::describe_image_with_prompts(
        &PromptCheckingVisionLlm,
        &LlmBackend::Ollama("llava".to_string()),
        llm::LlmImage {
            mime_type: "image/png".to_string(),
            data_base64: "AAE=".to_string(),
            source: "scan.png".to_string(),
        },
        "Input image file: scan.png",
        &prompts,
    )
    .unwrap();

    assert_eq!(
        nodes,
        vec![
            AstNode::Heading {
                level: 1,
                text: "Diagram".to_string(),
            },
            AstNode::Paragraph("A visible workflow diagram.".to_string()),
        ]
    );
}

#[test]
fn llm_diff_is_saved_as_unified_text() {
    let path = Path::new("target/llm-diff-test/restructure.diff");
    let _ = fs::remove_file(path);

    llm::save_diff(path, "# Old\n\nBody", "# New\n\nBody").unwrap();

    let diff = fs::read_to_string(path).unwrap();
    assert!(diff.contains("--- before"));
    assert!(diff.contains("+++ after"));
    assert!(diff.contains("-# Old"));
    assert!(diff.contains("+# New"));
}

#[test]
fn llm_filter_rejects_missing_backend_without_mutating_ast() {
    let mut ast = vec![AstNode::Paragraph("Body".to_string())];
    let mut warnings = Vec::new();
    let options = anything_to_markdown::ConversionOptions {
        restructure: true,
        ..Default::default()
    };

    llm::apply_llm_filters(&mut ast, &options, &mut warnings).unwrap();

    assert_eq!(ast, vec![AstNode::Paragraph("Body".to_string())]);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("no LLM backend was selected"))
    );
}

#[test]
fn cloud_llm_filter_requires_external_send_consent() {
    let mut ast = vec![AstNode::Paragraph("Body".to_string())];
    let mut warnings = Vec::new();
    let options = anything_to_markdown::ConversionOptions {
        llm: LlmBackend::OpenAi("gpt-4".to_string()),
        restructure: true,
        ..Default::default()
    };

    llm::apply_llm_filters(&mut ast, &options, &mut warnings).unwrap();

    assert_eq!(ast, vec![AstNode::Paragraph("Body".to_string())]);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("external send consent is required"))
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("LLM filter skipped"))
    );
}
