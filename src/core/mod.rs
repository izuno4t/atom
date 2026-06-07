mod ast;
mod config;
mod json;
mod naming;
mod options;
mod report;
mod text;

pub use ast::{AstNode, TableCell, TableRow};
pub use config::{load_config, parse_flavor, parse_format, parse_llm, parse_ocr};
pub(crate) use naming::{flavor_name, format_name, llm_destination, ocr_name};
pub use options::{ConversionOptions, Flavor, LlmBackend, OcrEngine, OutputFormat};
pub use report::{ConversionReport, ConversionResult, MediaCandidate};
pub(crate) use text::{decode_entities, escape_html, strip_tags};
