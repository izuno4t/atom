use crate::pipeline::input_detection::extension;
use crate::pipeline::ooxml_conversion::convert_ooxml_file;
use crate::pipeline::pdf_conversion::convert_pdf_bytes;
use crate::*;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;

pub struct Converter {
    options: ConversionOptions,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            options: ConversionOptions::default(),
        }
    }

    pub fn with_options(mut self, options: ConversionOptions) -> Self {
        self.options = options;
        self
    }

    pub fn convert_file<P: AsRef<Path>>(&self, input: P) -> io::Result<ConversionResult> {
        validate_media_options(&self.options)?;
        let path = input.as_ref();
        let ext = extension(path);
        match ext.as_str() {
            "docx" | "pptx" | "xlsx" | "xlsm" => {
                return convert_ooxml_file(path, ext, &self.options);
            }
            "md" | "txt" | "csv" | "xml" | "svg" | "gdoc" => {}
            "pdf" | "ai" | "html" | "htm" | "" => {}
            other => return Ok(self.unsupported_file_result(path, other)),
        }
        let bytes = fs::read(path)?;
        self.convert_bytes(&path.to_string_lossy(), &bytes)
    }

    fn unsupported_file_result(&self, path: &Path, input_format: &str) -> ConversionResult {
        let started = Instant::now();
        let ast = vec![unsupported_node(input_format)];
        let rendered = render(&ast, &self.options);
        ConversionResult {
            ast,
            markdown: rendered,
            report: ConversionReport {
                input_path: path.to_string_lossy().to_string(),
                input_format: input_format.to_string(),
                output_format: format_name(self.options.format).to_string(),
                flavor: flavor_name(self.options.flavor).to_string(),
                warnings: vec![format!("unsupported input format: {input_format}")],
                metadata: vec![("bytes".to_string(), "not_read".to_string())],
                elapsed_ms: started.elapsed().as_millis(),
                used_ocr: false,
                ocr_engine: None,
                used_llm: false,
                llm_destination: None,
                media: Vec::new(),
                media_candidates: Vec::new(),
                features: report_features(&self.options, &[]),
            },
        }
    }

    pub fn convert_bytes(&self, input_name: &str, bytes: &[u8]) -> io::Result<ConversionResult> {
        validate_media_options(&self.options)?;
        let started = Instant::now();
        let mut warnings = Vec::new();
        let input_format = detect_format(input_name, bytes);
        let mut metadata = vec![("bytes".to_string(), bytes.len().to_string())];
        let parse_format = parse_format(&input_format, bytes, &mut metadata);
        let mut media = Vec::new();

        if let Some(media_dir) = &self.options.extract_media {
            fs::create_dir_all(media_dir)?;
            media.push(media_dir.to_string_lossy().to_string());
        }

        let mut pdf_report = None;
        let mut ast = match parse_format.as_str() {
            "html" => html::parse_html(
                std::str::from_utf8(bytes).unwrap_or_default(),
                &mut warnings,
            ),
            "pdf" => {
                let (ast, result) =
                    convert_pdf_bytes(bytes, &self.options, &mut warnings, &mut metadata)?;
                pdf_report = Some(result);
                ast
            }
            "docx" => {
                warnings.push(
                    "DOCX byte conversion cannot unzip in-memory input; use convert_file for DOCX."
                        .to_string(),
                );
                vec![unsupported_node("DOCX in-memory input")]
            }
            "pptx" => {
                let text = std::str::from_utf8(bytes).unwrap_or_default();
                if text.contains("<p:sld") {
                    ooxml::parse_pptx_slide_xml(text)
                } else {
                    warnings.push(format!(
                        "could not read {} package from in-memory bytes",
                        input_format
                    ));
                    vec![unsupported_node(&input_format)]
                }
            }
            "xlsx" => {
                let text = std::str::from_utf8(bytes).unwrap_or_default();
                if text.contains("<worksheet") {
                    ooxml::parse_xlsx_sheet_xml(text, "")
                } else {
                    warnings.push(format!(
                        "could not read {} package from in-memory bytes",
                        input_format
                    ));
                    vec![unsupported_node(&input_format)]
                }
            }
            "md" | "txt" | "csv" | "xml" | "svg" | "gdoc" => {
                text_like_ast(&input_format, String::from_utf8_lossy(bytes).as_ref())
            }
            _ => {
                warnings.push(format!("unsupported input format: {input_format}"));
                vec![unsupported_node(&input_format)]
            }
        };

        if self.options.ocr != OcrEngine::None {
            warnings.push(format!(
                "OCR engine selected: {}",
                ocr_name(&self.options.ocr)
            ));
        }

        if self.options.restructure || self.options.translate.is_some() {
            llm::apply_llm_filters(&mut ast, &self.options, &mut warnings)?;
        }

        media.extend(collect_media_paths(&ast));
        media.sort();
        media.dedup();
        let media_candidates = collect_media_candidates(&ast);
        metadata.push(("nodes".to_string(), ast.len().to_string()));
        let mut features = report_features(&self.options, &media);
        if let Some(result) = &pdf_report {
            features.push(format!("pdf_backend:{}", result.backend));
            if result.ocr_required {
                features.push("pdf:ocr_required".to_string());
            }
            if result.extraction_failed {
                features.push("pdf:extraction_failed".to_string());
            }
        }
        let rendered = render(&ast, &self.options);
        let report = ConversionReport {
            input_path: input_name.to_string(),
            input_format,
            output_format: format_name(self.options.format).to_string(),
            flavor: flavor_name(self.options.flavor).to_string(),
            warnings,
            metadata,
            elapsed_ms: started.elapsed().as_millis(),
            used_ocr: self.options.ocr != OcrEngine::None,
            ocr_engine: (self.options.ocr != OcrEngine::None)
                .then(|| ocr_name(&self.options.ocr).to_string()),
            used_llm: self.options.llm != LlmBackend::None,
            llm_destination: llm_destination(&self.options.llm),
            media,
            media_candidates,
            features,
        };
        Ok(ConversionResult {
            ast,
            markdown: rendered,
            report,
        })
    }
}

fn text_like_ast(input_format: &str, text: &str) -> Vec<AstNode> {
    match input_format {
        "md" | "txt" => vec![AstNode::RawHtml(text.trim().to_string())],
        "csv" => fenced_text("csv", text),
        "xml" | "svg" => fenced_text("xml", text),
        "gdoc" => fenced_text("json", text),
        _ => vec![unsupported_node(input_format)],
    }
}

fn parse_format(input_format: &str, bytes: &[u8], metadata: &mut Vec<(String, String)>) -> String {
    if input_format == "ai" && bytes.starts_with(b"%PDF") {
        metadata.push(("container_format".to_string(), "pdf".to_string()));
        return "pdf".to_string();
    }
    input_format.to_string()
}

fn fenced_text(language: &str, text: &str) -> Vec<AstNode> {
    vec![AstNode::CodeBlock {
        language: Some(language.to_string()),
        code: text.trim().to_string(),
    }]
}

fn validate_media_options(options: &ConversionOptions) -> io::Result<()> {
    if options.extract_media.is_some() && options.inline_base64_media {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "extract_media and inline_base64_media are mutually exclusive",
        ));
    }
    Ok(())
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn convert_bytes(
    input_name: &str,
    bytes: &[u8],
    options: ConversionOptions,
) -> io::Result<ConversionResult> {
    Converter::new()
        .with_options(options)
        .convert_bytes(input_name, bytes)
}

pub fn convert_reader<R: Read>(
    input_name: &str,
    mut reader: R,
    options: ConversionOptions,
) -> io::Result<ConversionResult> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    convert_bytes(input_name, &bytes, options)
}
