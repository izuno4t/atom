//! ocr-rs(PP-OCRv5 mobile, MNN) を画像に対して raw に実行し、認識テキストを
//! 標準出力へ返すプローブ。atom 本体 `src/integrations/ocr.rs` の OcrRsBackend と
//! 同じ呼び出し(モデル3点 + 既定設定 None, 行を "\n" 連結)を再現する。
//!
//! 使い方:
//!   ocr-rs-probe <image> <det.mnn> <rec.mnn> <charset.txt> <out.txt>
//!
//! 認識テキストは out.txt に書く。MNN は初期化時に能力バナーを標準出力へ
//! 出すため、結果を stdout に混ぜずファイルへ分離する。

use std::path::PathBuf;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        eprintln!("usage: ocr-rs-probe <image> <det.mnn> <rec.mnn> <charset.txt> <out.txt>");
        exit(2);
    }
    let image_path = PathBuf::from(&args[1]);
    let det = PathBuf::from(&args[2]);
    let rec = PathBuf::from(&args[3]);
    let charset = PathBuf::from(&args[4]);
    let out_path = PathBuf::from(&args[5]);

    let image = match image::open(&image_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("open image failed: {e}");
            exit(1);
        }
    };
    let engine = match ocr_rs::OcrEngine::new(&det, &rec, &charset, None) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("engine init failed: {e}");
            exit(1);
        }
    };
    let results = match engine.recognize(&image) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("recognize failed: {e}");
            exit(1);
        }
    };
    let text = results
        .into_iter()
        .map(|r| r.text)
        .collect::<Vec<_>>()
        .join("\n");
    if let Err(e) = std::fs::write(&out_path, text) {
        eprintln!("write output failed: {e}");
        exit(1);
    }
}
