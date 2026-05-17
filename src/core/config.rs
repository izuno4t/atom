use crate::{ConversionOptions, Flavor, LlmBackend, OcrEngine, OutputFormat};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn load_config(path: &Path) -> io::Result<ConversionOptions> {
    let text = fs::read_to_string(path)?;
    let mut options = ConversionOptions::default();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match key.trim() {
            "flavor" => options.flavor = parse_flavor(value).unwrap_or(options.flavor),
            "format" => options.format = parse_format(value).unwrap_or(options.format),
            "ocr" => options.ocr = parse_ocr(value),
            "llm" => options.llm = parse_llm(value),
            "translate" => options.translate = Some(value.to_string()),
            "extract_media" => options.extract_media = Some(PathBuf::from(value)),
            "inline_base64_media" => options.inline_base64_media = value == "true",
            "restructure" => options.restructure = value == "true",
            "strict" => options.strict = value == "true",
            "consent_external_send" => options.consent_external_send = value == "true",
            _ => {}
        }
    }
    Ok(options)
}

pub fn parse_flavor(value: &str) -> Option<Flavor> {
    match value {
        "commonmark" | "CommonMark" => Some(Flavor::CommonMark),
        "gfm" | "GFM" => Some(Flavor::Gfm),
        "markdownlint" => Some(Flavor::Markdownlint),
        "hedgedoc" | "hackmd" => Some(Flavor::HedgeDoc),
        _ => None,
    }
}

pub fn parse_format(value: &str) -> Option<OutputFormat> {
    match value {
        "md" | "markdown" => Some(OutputFormat::Markdown),
        "mdx" => Some(OutputFormat::Mdx),
        "html" => Some(OutputFormat::Html),
        _ => None,
    }
}

pub fn parse_ocr(value: &str) -> OcrEngine {
    match value {
        "none" => OcrEngine::None,
        "auto" => OcrEngine::Auto,
        "ocr-rs" => OcrEngine::OcrRs,
        "ndlocr-lite" => OcrEngine::NdlOcrLite,
        "ndl-koten" => OcrEngine::NdlKoten,
        "tesseract" => OcrEngine::Tesseract,
        "surya" => OcrEngine::Surya,
        other => OcrEngine::External(other.to_string()),
    }
}

pub fn parse_llm(value: &str) -> LlmBackend {
    if value == "none" {
        LlmBackend::None
    } else if let Some(model) = value.strip_prefix("ollama:") {
        LlmBackend::Ollama(model.to_string())
    } else if value.starts_with("gpt-") {
        LlmBackend::OpenAi(value.to_string())
    } else if value.starts_with("claude-") {
        LlmBackend::Anthropic(value.to_string())
    } else {
        LlmBackend::OpenAiCompatible {
            name: value.to_string(),
            endpoint: String::new(),
        }
    }
}
