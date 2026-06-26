use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flavor {
    CommonMark,
    Gfm,
    Markdownlint,
    HedgeDoc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Markdown,
    Mdx,
    Html,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OcrEngine {
    None,
    Auto,
    OcrRs,
    NdlOcrLite,
    NdlKoten,
    Tesseract,
    Surya,
    External(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LlmBackend {
    None,
    Anthropic(String),
    Gemini(String),
    OpenAi(String),
    Ollama(String),
    OpenAiCompatible { name: String, endpoint: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConversionOptions {
    pub flavor: Flavor,
    pub format: OutputFormat,
    pub extract_media: Option<std::path::PathBuf>,
    pub inline_base64_media: bool,
    pub ocr: OcrEngine,
    pub llm: LlmBackend,
    pub restructure: bool,
    pub translate: Option<String>,
    pub report_path: Option<std::path::PathBuf>,
    pub strict: bool,
    pub config_path: Option<std::path::PathBuf>,
    pub consent_external_send: bool,
    pub llm_prompts: BTreeMap<String, String>,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            flavor: Flavor::CommonMark,
            format: OutputFormat::Markdown,
            extract_media: None,
            inline_base64_media: false,
            // 既定で OCR 有効。エンジン未指定(Auto)はスキャンPDF検出時に ocr-rs を
            // 用いる。明示的に無効化するには `ocr = off`(None) を指定する。
            ocr: OcrEngine::Auto,
            llm: LlmBackend::None,
            restructure: false,
            translate: None,
            report_path: None,
            strict: false,
            config_path: None,
            consent_external_send: false,
            llm_prompts: BTreeMap::new(),
        }
    }
}
