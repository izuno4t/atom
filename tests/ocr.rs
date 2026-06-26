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
fn ocr_rs_engine_is_in_process_without_subprocess_command() {
    // ocr-rs(Auto も同様)は in-process エンジンで、外部コマンドを持たない。
    // モデルは未設定なら初回ダウンロードでキャッシュするため、ここでは通信を
    // 伴う backend 構築は呼ばず、コマンドが無いことだけを確認する。
    assert!(ocr::command_for_engine(&OcrEngine::OcrRs).is_none());
    assert!(ocr::command_for_engine(&OcrEngine::Auto).is_none());
}
