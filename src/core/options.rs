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
            // 既定では OCR 無効。`ocr = on`(Auto) で有効化するとエンジン未指定でも
            // ocr-rs を使い、モデル未設定なら初回に自動ダウンロードする。
            ocr: OcrEngine::None,
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
