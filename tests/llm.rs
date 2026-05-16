use std::fs;
use std::io;
use std::path::Path;

use bonjil::llm::{self, LlmProvider, LlmRequest, LlmResponse};
use bonjil::{AstNode, LlmBackend};

struct StubLlm;

impl LlmProvider for StubLlm {
    fn complete(&self, request: &LlmRequest) -> io::Result<LlmResponse> {
        Ok(LlmResponse {
            text: format!("{}:{}", request.task, request.input),
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

#[test]
fn llm_backend_boundary_accepts_replaceable_provider() {
    let response = llm::complete_with(
        &StubLlm,
        &LlmRequest {
            backend: LlmBackend::Ollama("llama3".to_string()),
            task: "restructure".to_string(),
            input: " body ".to_string(),
        },
    )
    .unwrap();

    assert_eq!(response.text, "restructure: body ");
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
