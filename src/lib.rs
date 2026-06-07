mod core;
mod evaluation;
mod integrations;
mod parsers;
mod pipeline;
pub mod writers;

pub use core::{
    AstNode, ConversionOptions, ConversionReport, ConversionResult, Flavor, LlmBackend,
    MediaCandidate, MetricScore, OcrCerCase, OcrEngine, OutputFormat, TableCell, TableRow,
    load_config, parse_flavor, parse_format, parse_llm, parse_ocr,
};
pub use evaluation::{
    evaluate_heading_recall, evaluate_lint_score, evaluate_ocr_cer, evaluate_ocr_cer_by_group,
    evaluate_structure_fidelity, evaluate_table_integrity, evaluate_translation_structure_preserve,
};
pub use integrations::{llm, media, ocr};
pub use parsers::ooxml::{docx, pptx, xlsx};
pub use parsers::{html, ooxml, pdf};
pub use pipeline::{Converter, convert_bytes, convert_reader};
pub use writers::markdown;

pub(crate) use core::{
    decode_entities, escape_html, flavor_name, format_name, llm_destination, ocr_name, strip_tags,
};
pub(crate) use pipeline::{
    collect_media_candidates, collect_media_paths, detect_format, report_features, unsupported_node,
};
pub(crate) use writers::render;
