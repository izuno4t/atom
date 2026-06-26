use crate::{ConversionOptions, Flavor, LlmBackend, OcrEngine, OutputFormat};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn load_config(path: &Path) -> io::Result<ConversionOptions> {
    let mut options = ConversionOptions::default();
    apply_config(&mut options, path)?;
    options.config_path = Some(path.to_path_buf());
    Ok(options)
}

pub fn apply_config(options: &mut ConversionOptions, path: &Path) -> io::Result<()> {
    let text = fs::read_to_string(path)?;
    for line in text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        let key = key.trim();
        match key {
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
            _ => {
                if let Some(prompt_task) = key.strip_prefix("llm.prompt_path.") {
                    let prompt = read_prompt_file(path, value)?;
                    options.llm_prompts.insert(prompt_task.to_string(), prompt);
                } else if let Some(prompt_task) = key.strip_prefix("llm_prompt_path_") {
                    let prompt = read_prompt_file(path, value)?;
                    options
                        .llm_prompts
                        .insert(prompt_task.replace('_', "-").to_string(), prompt);
                }
            }
        }
    }
    options.config_path = Some(path.to_path_buf());
    Ok(())
}

fn read_prompt_file(config_path: &Path, value: &str) -> io::Result<String> {
    let path = PathBuf::from(value);
    let path = if path.is_absolute() {
        path
    } else {
        config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    };
    fs::read_to_string(path)
}

pub fn apply_user_config(options: &mut ConversionOptions) -> io::Result<Option<PathBuf>> {
    for path in user_config_paths() {
        if path.exists() {
            apply_config(options, &path)?;
            return Ok(Some(path));
        }
    }
    Ok(None)
}

pub fn user_config_paths() -> Vec<PathBuf> {
    let atom_home = env::var_os("ATOM_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".atom")));
    let Some(atom_home) = atom_home else {
        return Vec::new();
    };
    vec![atom_home.join("config.toml")]
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
        "off" | "none" | "false" => OcrEngine::None,
        "on" | "auto" | "true" => OcrEngine::Auto,
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
    } else if let Some(model) = value.strip_prefix("gemini:") {
        LlmBackend::Gemini(model.to_string())
    } else if let Some(value) = value.strip_prefix("openai-compatible:") {
        if let Some((name, endpoint)) = value.split_once('@') {
            LlmBackend::OpenAiCompatible {
                name: name.to_string(),
                endpoint: endpoint.to_string(),
            }
        } else {
            LlmBackend::OpenAiCompatible {
                name: "openai-compatible".to_string(),
                endpoint: value.to_string(),
            }
        }
    } else if value.starts_with("gemini-") {
        LlmBackend::Gemini(value.to_string())
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
