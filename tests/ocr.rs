use std::io;
use std::path::Path;

use anything_to_markdown::OcrEngine;
use anything_to_markdown::ocr::{self, OcrBackend};

struct StubOcr;

impl OcrBackend for StubOcr {
    fn recognize(&self, input: &Path) -> io::Result<String> {
        Ok(format!("recognized:{}", input.display()))
    }
}

#[test]
fn ocr_engine_boundary_accepts_replaceable_backend() {
    let text = ocr::recognize_with(&StubOcr, Path::new("scan.pdf")).unwrap();

    assert_eq!(text, "recognized:scan.pdf");
}

#[test]
fn ndlocr_lite_subprocess_command_is_exposed() {
    assert_eq!(
        ocr::command_for_engine(&OcrEngine::NdlOcrLite).unwrap(),
        "ndlocr-lite"
    );
    assert!(ocr::command_for_engine(&OcrEngine::None).is_none());
}

#[test]
fn ocr_rs_backend_requires_model_environment() {
    if std::env::var_os("ATOM_OCR_RS_DET_MODEL").is_some()
        && std::env::var_os("ATOM_OCR_RS_REC_MODEL").is_some()
        && std::env::var_os("ATOM_OCR_RS_CHARSET").is_some()
    {
        return;
    }

    let backend = match ocr::backend_for_engine(&OcrEngine::OcrRs) {
        Ok(_) => panic!("ocr-rs backend should require model environment"),
        Err(error) => error,
    };

    assert!(backend.to_string().contains("ATOM_OCR_RS_DET_MODEL"));
    assert!(ocr::command_for_engine(&OcrEngine::OcrRs).is_none());
}
