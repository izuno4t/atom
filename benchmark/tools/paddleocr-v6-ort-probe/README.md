# paddleocr-v6-ort-probe

PP-OCRv6 を Rust から in-process で動かせるか検証するプローブ（rec 優先）。
`ort`（ONNX Runtime バインディング）で rec.onnx を実行し、前処理と CTC デコードを
Rust 実装する。atom 本体へ組み込む前の技術検証であり、結果は
[methods/ocr-engine-evaluation.md](../../methods/ocr-engine-evaluation.md) の
導入方針を裏付ける。

## 検証結果（Phase 1: rec）

単一行フィクスチャ（`benchmark/ocr-eval/fixtures-line/`）で、Rust(ort) の出力が
Python 版 golden（[ocr-eval/rec_golden.py](../../ocr-eval/rec_golden.py)）と一致した。

- `本システムは画像を解析する` 一致
- `OCRエンジンPP-OCRv6` 一致
- `売上は前年比123%増` は Python golden と同一。PaddleOCR 本体のみ全角 `％` で、
  これは `%` と `％` が両方辞書にある境界グリフでのリサイズ実装差（bilinear の
  差）であって、CTC や前処理のロジック誤りではない。

結論。rec.onnx の in-process 実行と CTC デコードは Rust で問題なく成立する。

## 実装メモ（ハマりどころ）

- `ort` 2.x はプレリリースのため、`ort = "2"` では解決されない。
  `ort = "2.0.0-rc.10"` のように rc を明示する（解決は rc.12）。
- `ort` rc.12 は `ndarray` 0.17 に依存する。プローブ側も `ndarray = "0.17"` に
  揃えないと、`TensorArrayData` が「別バージョンの `ndarray` 型」に対して未実装と
  なりコンパイルできない。
- 推論 API は
  `session.run(ort::inputs![TensorRef::from_array_view(&array)?])` で、
  出力は `outputs[0].try_extract_tensor::<f32>()` が `(&Shape, &[f32])` を返す。
  `session.run` は `&mut self`。

## rec 仕様（inference.yml より）

- 入力 `x`: `[1,3,48,W]`、CHW、`img_mode=BGR`。
- RecResizeImg `image_shape=[3,48,320]`: 高さ48・縦横比維持・最大幅320・0パディング。
- 正規化: `(px/255 - 0.5) / 0.5`。
- 出力: `[1,T,18710]`、CTCLabelDecode。ラベルは index0=blank、1..=18708=辞書、
  18709=space。

## 実行

```bash
cargo build --release
target/release/paddleocr-v6-ort-probe \
  ../../ocr-eval/models/PP-OCRv6_medium_rec/inference.onnx \
  ../../ocr-eval/models/ppocr_keys_v6.txt \
  ../../ocr-eval/fixtures-line/line-tech.png \
  /tmp/out.txt
```

## 残り（Phase 2）

- 検出（DBNet）を pure-Rust で後処理（sigmoid・閾値・連結成分・unclip）し、
  `opencv` 依存を避けて det + rec の end-to-end にする。
- atom 本体に `OcrEngine::PaddleOcrV6` と `ort` バックエンドとして統合する。
  `ort`（プレリリース）を atom の依存に加える判断が必要。
