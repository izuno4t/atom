use crate::{Flavor, LlmBackend, OcrEngine, OutputFormat};

pub(crate) fn flavor_name(flavor: Flavor) -> &'static str {
    match flavor {
        Flavor::CommonMark => "commonmark",
        Flavor::Gfm => "gfm",
        Flavor::Markdownlint => "markdownlint",
        Flavor::HedgeDoc => "hedgedoc",
    }
}

pub(crate) fn format_name(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Markdown => "markdown",
        OutputFormat::Mdx => "mdx",
        OutputFormat::Html => "html",
    }
}

pub(crate) fn ocr_name(engine: &OcrEngine) -> &str {
    match engine {
        OcrEngine::None => "none",
        OcrEngine::Auto => "auto",
        OcrEngine::OcrRs => "ocr-rs",
        OcrEngine::NdlOcrLite => "ndlocr-lite",
        OcrEngine::NdlKoten => "ndl-koten",
        OcrEngine::Tesseract => "tesseract",
        OcrEngine::Surya => "surya",
        OcrEngine::External(command) => command,
    }
}

pub(crate) fn llm_destination(llm: &LlmBackend) -> Option<String> {
    match llm {
        LlmBackend::None => None,
        LlmBackend::Anthropic(_) => Some("Anthropic".to_string()),
        LlmBackend::OpenAi(_) => Some("OpenAI".to_string()),
        LlmBackend::Ollama(_) => Some("local".to_string()),
        LlmBackend::OpenAiCompatible { endpoint, .. } if !endpoint.is_empty() => {
            Some(endpoint.clone())
        }
        LlmBackend::OpenAiCompatible { name, .. } => Some(name.clone()),
    }
}
