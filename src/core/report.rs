use super::json::{escape_json, json_array, json_option};
use crate::AstNode;

#[derive(Clone, Debug, PartialEq)]
pub struct ConversionResult {
    pub ast: Vec<AstNode>,
    pub markdown: String,
    pub report: ConversionReport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConversionReport {
    pub input_path: String,
    pub input_format: String,
    pub output_format: String,
    pub flavor: String,
    pub warnings: Vec<String>,
    pub metadata: Vec<(String, String)>,
    pub elapsed_ms: u128,
    pub used_ocr: bool,
    pub ocr_engine: Option<String>,
    pub used_llm: bool,
    pub llm_destination: Option<String>,
    pub media: Vec<String>,
    pub media_candidates: Vec<MediaCandidate>,
    pub features: Vec<String>,
}

impl ConversionReport {
    pub fn to_json(&self) -> String {
        let warnings = json_array(&self.warnings);
        let metadata = self
            .metadata
            .iter()
            .map(|(key, value)| {
                format!(
                    "{{\"key\":\"{}\",\"value\":\"{}\"}}",
                    escape_json(key),
                    escape_json(value)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let media = json_array(&self.media);
        let media_candidates = self
            .media_candidates
            .iter()
            .map(MediaCandidate::to_json)
            .collect::<Vec<_>>()
            .join(",");
        let features = json_array(&self.features);
        format!(
            concat!(
                "{{",
                "\"input_path\":\"{}\",",
                "\"input_format\":\"{}\",",
                "\"output_format\":\"{}\",",
                "\"flavor\":\"{}\",",
                "\"warnings\":{},",
                "\"metadata\":[{}],",
                "\"elapsed_ms\":{},",
                "\"used_ocr\":{},",
                "\"ocr_engine\":{},",
                "\"used_llm\":{},",
                "\"llm_destination\":{},",
                "\"media\":{},",
                "\"media_candidates\":[{}],",
                "\"features\":{}",
                "}}\n"
            ),
            escape_json(&self.input_path),
            escape_json(&self.input_format),
            escape_json(&self.output_format),
            escape_json(&self.flavor),
            warnings,
            metadata,
            self.elapsed_ms,
            self.used_ocr,
            json_option(self.ocr_engine.as_deref()),
            self.used_llm,
            json_option(self.llm_destination.as_deref()),
            media,
            media_candidates,
            features
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaCandidate {
    pub media_id: String,
    pub path: String,
    pub caption: Option<String>,
    pub source: String,
    pub confidence: f64,
}

impl MediaCandidate {
    fn to_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"media_id\":\"{}\",",
                "\"path\":\"{}\",",
                "\"caption\":{},",
                "\"source\":\"{}\",",
                "\"confidence\":{}",
                "}}"
            ),
            escape_json(&self.media_id),
            escape_json(&self.path),
            json_option(self.caption.as_deref()),
            escape_json(&self.source),
            self.confidence
        )
    }
}
