# 日本語OCRエンジン評価（ocr-rs と PP-OCRv6）

`atom` に追加するOCRエンジンとして PP-OCRv6 を検討するための、実測ベースの
評価記録と導入方針をまとめる。比較対象は現行の `ocr-rs` と PP-OCRv6 の2つ。

## 目的

- PP-OCRv6 を `atom` に組み込む価値があるかを、机上ではなく実測で判断する。
- 現行 `ocr-rs` を基準（baseline）として、日本語の文字認識精度を比較する。
- 導入する場合の方式（in-process と subprocess）と優先順位を整理する。

## 対象エンジン

| エンジン | 実体 | 推論基盤 | 実行形態 | サイズ感 |
| --- | --- | --- | --- | --- |
| `ocr-rs` | PP-OCRv5 mobile（DBNet系 det + CRNN系 rec） | MNN | in-process（Rust crate） | mobile（軽量） |
| PP-OCRv6 | PP-OCRv6 medium（det + rec） | ONNX Runtime | 本評価では Python から呼出 | 34.5M params |

補足。

- `ocr-rs`（crate `ocr-rs = 2.2.2`）は `atom` の現行エンジンで、MNN形式の
  `.mnn` モデルと `ppocr_keys` 形式の文字セットを要求する。
- PP-OCRv6 は ONNX variants が公式提供され、`paddlepaddle` なしで
  ONNX Runtime バックエンドだけで動作する。詳細は
  [PP-OCRv6 紹介記事](https://huggingface.co/blog/PaddlePaddle/pp-ocrv6) を参照。
- Surya は今回の実測対象外。導入手順のみ
  [tools/surya/README.md](../tools/surya/README.md) に整備した。

## 評価前に判明した `atom` 側の現状

実装を読んだ結果、評価設計に影響する事実が2つあった。

1. 画像単体（PNG など）の入力は OCR 経路に入らない。
   `src/pipeline/converter.rs` の `convert_image_file` は VLM（`--llm`）専用で、
   `--ocr` は画像単体では無視される。OCR が走るのは PDF 経路
   （`src/integrations/ocr.rs` の `recognize_pdf_pages`）のみ。
2. `ocr-rs` はモデルを同梱しておらず、`ATOM_OCR_RS_DET_MODEL` などの環境変数で
   モデルパスを与える必要がある。リポジトリにモデル取得手順がなく、素の状態では
   動作しない。取得手順は [tools/ocr-rs-probe/README.md](../tools/ocr-rs-probe/README.md)
   に整備した。

このため本評価では、`atom` 本体を経由せず各エンジンを直接実行して比較した
（raw 同士の公平比較）。

## 評価方法

### フィクスチャ（合成生成）

既知の日本語テキストを画像へ描画し、描画した元テキストをそのまま正解
（ground truth）とした。CER を正確かつ再現可能に測れ、機密文書を使わずに済む。
生成器は [ocr-eval/gen_fixtures.py](../ocr-eval/gen_fixtures.py)。

- 文面。一般文・技術文・数字記号混在・カタカナ英字混在・縦書き用の5種。
- フォント。ヒラギノ角ゴシック・ヒラギノ明朝・ヒラギノ丸ゴの3種。
- 向き。横書き10枚、縦書き2枚（計12枚）。

### 指標（CER）

`atom` 本体の `evaluation/src/metrics.rs` にある `evaluate_ocr_cer` を忠実に
再現した。

```text
score = 1 - min(levenshtein(expected, actual) / max(len(expected), 1), 1)
```

文字単位の Levenshtein 距離を使う。値域は0から1で、1が完全一致。正規化は2系統で
報告する。

- `strip`。空白と改行を全除去し、純粋な文字認識を測る。
- `nfkc`。`strip` に加えて NFKC 折り畳みを行い、全角と半角や互換文字の差を吸収する。

ドライバは [ocr-eval/compare.py](../ocr-eval/compare.py)。

## 結果

### PP-OCRv6 medium と ocr-rs（向き別、strip）

| 向き | ocr-rs | PP-OCRv6 medium |
| --- | --- | --- |
| 横書き（10枚） | 0.9946 | 0.9946 |
| 縦書き（2枚） | 0.0000 | 0.5455 |
| 総合（12枚） | 0.8894 | 0.9471 |

### 横書きフィクスチャ別（strip と nfkc）

| フィクスチャ | ocr-rs strip | v6 strip | ocr-rs nfkc | v6 nfkc |
| --- | --- | --- | --- | --- |
| novel（3フォント） | 1.000 | 1.000 | 1.000 | 1.000 |
| tech（3フォント） | 1.000 | 1.000 | 1.000 | 1.000 |
| number（2フォント） | 0.974 | 0.974 | 1.000 | 1.000 |
| katakana（2フォント） | 1.000 | 1.000 | 1.000 | 1.000 |

### PP-OCRv6 のサイズ別（tier 別、strip）

| tier | params | 横書き | 縦書き | 総合 |
| --- | --- | --- | --- | --- |
| tiny | 1.5M | 0.5806 | 0.1591 | 0.5361 |
| medium | 34.5M | 0.9946 | 0.5455 | 0.9471 |

## 考察

- クリーンな横書き日本語では `ocr-rs` と PP-OCRv6 medium は完全に互角で、
  ともにほぼ完全一致だった。この難易度では精度差がつかない。
- `number` の 0.974 は両エンジンが等しく `%` を全角 `％` に変換したことが原因で、
  認識誤りではない。`nfkc` 折り畳みで両者とも 1.000 になる。`atom` の CER 指標は
  幅を正規化しないため、後処理で幅を揃える余地がある。
- 縦書きは `ocr-rs` が空またはゴミ出力で全滅した。PP-OCRv6 medium は文字自体は
  認識するが列順を誤り、部分一致（0.5455）にとどまる。素の構成ではどちらも縦書きに
  正しく対応しない。
- PP-OCRv6 tiny（1.5M、最小 tier）は日本語が崩れる。`は` を `法`、`である` を
  `飞而` のように、中国語寄りの字形へ引っ張られた誤認識が多い。これはハーネスの
  不具合ではなく、1.5M に50言語を詰めた最小モデルの実際の限界。
- ただし `ocr-rs`（PP-OCRv5 mobile、det 4.6MB + rec 16MB）は tiny（1.5M）と
  サイズ対等ではなく、より大きい中位 mobile に当たる。よって「軽量同士で PP-OCRv6 が
  劣る」ではなく「最小 tier は日本語に使えない／`ocr-rs` は medium 級と互角の良い
  mobile」と読むべき。サイズ対等の比較は `ocr-rs` 対 PP-OCRv6 small または medium。

## 結論と導入方針

- クリーン横書きだけを見ると PP-OCRv6 を入れる積極的な理由は乏しい（互角）。
  PP-OCRv6 を選ぶ根拠は、多言語対応（50言語）と、実スキャンや劣化文書・多様な
  レイアウトでの頑健性に置くべきで、本評価のクリーン合成文では裏取りできていない。
- 採用するなら medium 以上。tiny は日本語が崩れるため置き換えにならない。

### 実行方式（in-process と subprocess）

PP-OCRv6 は ONNX があるため、Rust から `ort` で直接動かす in-process と、
PaddleOCR の onnxruntime を外部プロセスで呼ぶ subprocess の両方が取れる。

| 観点 | in-process（`ort` + ONNX） | subprocess（PaddleOCR onnxruntime） |
| --- | --- | --- |
| 配布 | 単一バイナリ。外部ランタイム不要 | Python と paddleocr・onnxruntime（venv 約 600MB）を利用者に要求 |
| 速度 | モデル常駐。プロセス生成や一時ファイル受け渡しがない | 呼出しごとにプロセス生成とモデル再ロード。常駐サーバー化しない限り遅い |
| 実装コスト | det 後処理（DBNet の sigmoid・閾値・輪郭・unclip）と CTC・文字セットを Rust 実装。det 段で `opencv` 依存 | PaddleOCR の完成済みパイプラインをそのまま使う。CV や CTC の自前実装が不要 |
| 多言語・縦書き | 自前実装の範囲に依存 | 公式パイプラインの対応をそのまま得られる |
| atom の思想との整合 | 高い（単一バイナリ・外部送信は既定で無効） | 低い（重い外部ランタイム前提） |

推奨。

- 本採用の方式は **in-process（`ort`）が本命**。`atom` は単一バイナリ配布の Rust
  CLI で、利用者に Python 環境を強いる subprocess は配布性を大きく損なう。`atom` が
  subprocess にしているのは Surya など「Python 実装しかない」エンジンで、ONNX のある
  PP-OCRv6 をわざわざ subprocess にする理由は薄い。
- ただし in-process は実装コストが高い。最初は **rec 優先**で既存のラスタライズ・
  レイアウト処理を det 代わりに流用すると、`opencv` 依存を避けて着手できる。det の
  完全な in-process 化は後段。
- 一方、PP-OCRv6 の優位性（多言語・実スキャン頑健性）は本評価で未実証。優位性の
  有無をまず確かめるだけなら、完成済みパイプラインを即使える subprocess が最短。
  つまり「subprocess で優位性を検証 → 本採用は in-process で実装」の二段が合理的。

## 実装状況（この評価を受けた結論）

- `ocr-rs` を atom の既定の in-process エンジンとして整備した。OCR は既定で無効で、
  `--ocr on` 等で有効化するとエンジン未指定でも `ocr-rs` を使い、モデル未設定なら
  初回に自動ダウンロードする。
- PP-OCRv6 は「選択肢」として subprocess（`paddleocr-v6` ラッパー）で追加し、
  `atom <pdf> --ocr paddleocr-v6` の動作を検証した。in-process(`ort`)化は技術検証
  まで（[tools/paddleocr-v6-ort-probe](../tools/paddleocr-v6-ort-probe/README.md)）。
- 利用方法・設定・ネットワーク（プロキシ／独自CA）は [docs/ocr.md](../../docs/ocr.md)
  にまとめた。

## 残課題

- 本評価はクリーンな合成文のみ。PP-OCRv6 の優位が出るはずの実スキャン・低解像度・
  小さな文字・稀少漢字・多言語の評価は未実施。判断を固めるにはこれらの追加が要る。
- 縦書きは別系統の対応が必要で、本評価の構成では結論を出さない。

## 再現手順

```bash
# venv 準備（pyenv 3.12 系。詳細は tools/paddleocr-v6/README.md）
pyenv local 3.12.12
python -m venv benchmark/.venv-ocr
benchmark/.venv-ocr/bin/python -m pip install pillow numpy onnxruntime paddleocr

# ocr-rs プローブのビルド（詳細は tools/ocr-rs-probe/README.md）
cd benchmark/tools/ocr-rs-probe && cargo build --release && cd -

# フィクスチャ生成と比較
benchmark/.venv-ocr/bin/python benchmark/ocr-eval/gen_fixtures.py
benchmark/.venv-ocr/bin/python benchmark/ocr-eval/compare.py \
  --engines ocr-rs,paddleocr-v6 --tier medium
```
