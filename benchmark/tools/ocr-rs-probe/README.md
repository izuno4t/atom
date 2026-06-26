# ocr-rs-probe

`atom` の現行OCRエンジン `ocr-rs`（PP-OCRv5 mobile、MNN）を画像に対して raw に
実行する単機能プローブ。`src/integrations/ocr.rs` の `OcrRsBackend` と同じ呼出し
（モデル3点 + 既定設定、行を改行連結）を再現する。OCRエンジン比較
（[methods/ocr-engine-evaluation.md](../../methods/ocr-engine-evaluation.md)）で
baseline を実測するために使う。

## モデルの取得

`ocr-rs` はモデルを同梱しない。`.mnn` 形式の検出・認識モデルと文字セットを
[zibo-chen/rust-paddle-ocr](https://github.com/zibo-chen/rust-paddle-ocr) の
`models/` から取得する。

```bash
mkdir -p benchmark/ocr-eval/models/ocr-rs
cd benchmark/ocr-eval/models/ocr-rs
BASE=https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/main/models
curl -sSL -O "$BASE/PP-OCRv5_mobile_det.mnn"
curl -sSL -O "$BASE/PP-OCRv5_mobile_rec.mnn"
curl -sSL -O "$BASE/ppocr_keys_v5.txt"
```

det・rec・charset は同じ PP-OCR バージョンと言語で揃える必要がある。多言語の
rec モデルと対応する `.txt` も同 `models/` に用意されている。

## ビルド

MNN（C++）をコンパイルするため `cmake` と `clang` が必要。

```bash
cd benchmark/tools/ocr-rs-probe
cargo build --release
```

`atom` ワークスペースに取り込まれないよう、`Cargo.toml` に空の `[workspace]` を
置いて独立クレートにしている（既存の `*-probe` と同じ規約）。

## 実行

```bash
target/release/ocr-rs-probe <image> <det.mnn> <rec.mnn> <charset.txt> <out.txt>
```

認識テキストは `out.txt` に書き出す。MNN が初期化時に能力バナーを標準出力へ出す
ため、結果は標準出力に混ぜずファイルへ分離している。

## `atom` 本体で使う場合

`atom` の `OcrRsBackend::from_env` は次の環境変数を要求する。

```bash
export ATOM_OCR_RS_DET_MODEL=.../PP-OCRv5_mobile_det.mnn
export ATOM_OCR_RS_REC_MODEL=.../PP-OCRv5_mobile_rec.mnn
export ATOM_OCR_RS_CHARSET=.../ppocr_keys_v5.txt
```

OCR は PDF 経路でのみ走る。画像単体は VLM 経路に入るため、画像は1ページPDFに
包んでから `atom input.pdf --ocr ocr-rs` のように渡す。
