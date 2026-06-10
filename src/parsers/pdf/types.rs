use crate::AstNode;

#[derive(Clone, Debug)]
pub struct PdfTextObject {
    pub text: String,
    pub font_size: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct PdfTextExtraction {
    pub objects: Vec<PdfTextObject>,
    pub extraction_failed: bool,
    pub ocr_required: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PdfParseResult {
    pub ast: Vec<AstNode>,
    pub backend: String,
    pub extraction_failed: bool,
    pub ocr_required: bool,
}

pub trait PdfTextBackend {
    fn name(&self) -> &str;
    fn extract_text(&self, bytes: &[u8]) -> PdfTextExtraction;
}

pub struct AtomPdfTextBackend;

pub struct InternalPdfTextBackend;

pub struct PdfExtractBackend;

pub struct PdfOxideFormWordsBackend;

pub struct PdfOxideTextBackend;

pub struct PdfRsTextBackend;

pub struct LenientPdfExtractBackend;

pub struct LopdfTextBackend;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdfNoTextDiagnosis {
    ImageOnly,
    MissingUnicodeMaps,
    Unknown,
}

impl PdfNoTextDiagnosis {
    pub fn message(self) -> &'static str {
        match self {
            Self::ImageOnly => {
                "PDF contains page images or outlined/vector text but no extractable text layer. OCR is required."
            }
            Self::MissingUnicodeMaps => {
                "PDF text uses embedded fonts without Unicode maps, so glyphs cannot be converted back to text."
            }
            Self::Unknown => {
                "PDF text extraction failed for a non-encrypted PDF; cause could not be classified."
            }
        }
    }
}

pub(super) fn failed_pdf_text_extraction() -> PdfTextExtraction {
    PdfTextExtraction {
        objects: Vec::new(),
        extraction_failed: true,
        ocr_required: true,
    }
}

pub(super) fn pdf_text_extraction_from_plain_text(text: String) -> PdfTextExtraction {
    let objects = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| PdfTextObject {
            text: line.to_string(),
            font_size: None,
            x: None,
            y: None,
        })
        .collect::<Vec<_>>();
    PdfTextExtraction {
        ocr_required: objects.is_empty(),
        objects,
        extraction_failed: false,
    }
}
