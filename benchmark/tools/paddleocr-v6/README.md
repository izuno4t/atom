# paddleocr-v6

PP-OCRv6 を `paddlepaddle` なしの ONNX Runtime バックエンドで実行するランナー。
OCRエンジン比較（[methods/ocr-engine-evaluation.md](../../methods/ocr-engine-evaluation.md)）
で PP-OCRv6 を実測するために使う。

## 環境構築

`paddlepaddle` や `onnxruntime` のホイール対応の都合で、隔離した Python 3.12 系の
venv を使う。リポジトリ既定の `benchmark/.venv`（Python 3.14）とは別に
`benchmark/.venv-ocr` を用意する。

```bash
pyenv local 3.12.12   # 3.12 系を pyenv で用意済みとする
python -m venv benchmark/.venv-ocr
benchmark/.venv-ocr/bin/python -m pip install --upgrade pip
benchmark/.venv-ocr/bin/python -m pip install pillow numpy onnxruntime paddleocr
```

導入されるのは `paddleocr` 3.7 系と `paddlex` 3.7 系。`paddle` 本体は入れない。

## モデル

PP-OCRv6 の ONNX variants は Hugging Face の `PaddlePaddle/PP-OCRv6_<tier>_*_onnx`
にある。`run.py` が未取得時に `snapshot_download` で
`benchmark/ocr-eval/models/` へ取得する。`tier` は `tiny` `small` `medium`。

各リポジトリは `inference.onnx` `inference.yml` `inference.json` を含み、これが
そのまま paddlex の `*_model_dir` になる。

## 実行

```bash
benchmark/.venv-ocr/bin/python benchmark/tools/paddleocr-v6/run.py \
  --image foo.png --tier medium
```

認識テキストを行ごとに標準出力へ出す。

## ONNX Runtime バックエンドの要点

- PaddleOCR の共通引数 `engine="onnxruntime"` で ONNX を選ぶ。
  `PaddlePredictorOption(run_mode=...)` は paddle 静的グラフ専用で onnxruntime を
  含まないため、そちらでは指定できない。
- `text_detection_model_dir` と `text_recognition_model_dir` に ONNX ディレクトリを
  渡す。
- doc orientation、doc unwarping、textline orientation は無効化している。これらを
  有効にすると別モデルの取得が必要になる。
