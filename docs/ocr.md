# OCR

atom の OCR（光学文字認識）の挙動・設定・エンジン選択をまとめる。要件は
[requirements.md](requirements.md) の F6 を参照。

## デフォルトの挙動

- OCR は既定で無効（`ocr = off`）。利用者が `--ocr on` または設定で有効化する。
- 有効化したときにエンジン未指定なら `ocr-rs`（in-process）を使う。このとき
  モデルが未設定でも初回に自動ダウンロードするため、追加設定なしで動く。
- 有効時も、スキャン PDF（テキストを抽出できないページ）を検出したときだけ OCR を
  適用する。通常のテキスト PDF では OCR は走らない。

## 入力種別による処理の分岐

OCR が適用されるかどうかは入力種別で決まる。

| 入力 | 処理 | OCR 適用 |
| --- | --- | --- |
| PDF（テキストあり） | PDF テキスト抽出 | なし |
| PDF（テキストの無いページ） | 抽出後、無テキストページのみ OCR | あり（`--ocr` のエンジン） |
| PDF（暗号化） | 抽出のみ | なし（OCR 対象外） |
| 画像単体（PNG / JPG など） | VLM（`--llm`）で変換 | なし（`--ocr` は画像単体では無視） |

ポイント。

- OCR が走るのは PDF 経路の「テキストを抽出できないページ」だけ。
  実装は `src/pipeline/pdf_conversion.rs` の `try_pdf_ocr_for_pages_without_text`。
- 画像単体の入力は VLM 経路（`src/pipeline/converter.rs` の
  `convert_image_file`）に入り、`--ocr` は適用されない。画像を OCR したい場合は
  1ページ PDF に包んで渡す。

## OCR の有効・無効とエンジン選択

`--ocr <値>`（CLI）または設定ファイルの `ocr = <値>` で指定する。設定ファイルは
`--config <PATH>` か、ユーザー設定 `~/.atom/config.toml`（`ATOM_HOME` があれば
そちら）を読む。

| 値 | 意味 | 実行形態 |
| --- | --- | --- |
| `off` / `none`（既定） | OCR 無効 | — |
| `on` / `auto` | 有効・エンジン未指定 → `ocr-rs` | in-process |
| `ocr-rs` | 明示的に ocr-rs | in-process |
| `paddleocr-v6` / `surya` / `tesseract` / 任意コマンド名 | 指定エンジン | subprocess |

## ocr-rs（既定・in-process）

- 推論は MNN、モデルは PP-OCRv5 mobile（fp16）。
- モデルが未設定なら、初回に自動ダウンロードして
  `~/.atom/models/ocr-rs`（`ATOM_HOME` があれば `$ATOM_HOME/models/ocr-rs`）に
  キャッシュする。2回目以降はキャッシュを使う。
- 環境変数でモデルを差し替えられる。3点すべて指定したときだけ有効。

  ```bash
  export ATOM_OCR_RS_DET_MODEL=/path/det.mnn
  export ATOM_OCR_RS_REC_MODEL=/path/rec.mnn
  export ATOM_OCR_RS_CHARSET=/path/keys.txt
  ```

### モデルの出典とライセンス

埋め込みはせず初回ダウンロードする。取得元は
[zibo-chen/rust-paddle-ocr](https://github.com/zibo-chen/rust-paddle-ocr)
（Apache-2.0）。元モデルは PaddleOCR（PP-OCRv5 mobile、Apache-2.0）由来。

## subprocess エンジン（PP-OCRv6 など）

atom は `--ocr <名前>` を `名前 <画像パス>` として実行し、標準出力を認識テキストと
して読む（`src/integrations/ocr.rs` の `run_subprocess`）。`<名前>` は PATH 上の
コマンド名か絶対パス。

PP-OCRv6 を使う場合は、ONNX Runtime で動くラッパーを用意してある。

```bash
atom <input.pdf> --ocr /path/to/benchmark/tools/paddleocr-v6/paddleocr-v6
```

セットアップは [benchmark/tools/paddleocr-v6/README.md](../benchmark/tools/paddleocr-v6/README.md)、
評価は [benchmark/methods/ocr-engine-evaluation.md](../benchmark/methods/ocr-engine-evaluation.md)
を参照。

## ネットワーク（プロキシ・独自CA）

モデルダウンロードと LLM API の通信は共有 HTTP ヘルパー
（`src/integrations/http.rs`）を通り、プロキシと独自CAを環境変数から読む。

- プロキシ: `ALL_PROXY` / `HTTP_PROXY` / `HTTPS_PROXY`、および除外用の `NO_PROXY`。
- 独自CA: `ATOM_CA_BUNDLE` / `SSL_CERT_FILE` / `CURL_CA_BUNDLE` のいずれかが指す
  PEM バンドルをルート証明書として使う（curl / OpenSSL と同じく既定のルートを
  置き換える）。TLS 傍受プロキシ環境を想定。

## 実装メモ

- ocr-rs（MNN）は初期化時に `The device supports: ...` のバナーをネイティブ側から
  標準出力へ出す。atom の出力に混ざらないよう、OCR 実行中だけ標準出力（fd 1）を
  退避して抑止している（`src/integrations/ocr.rs` の `StdoutSilencer`、Unix のみ）。
