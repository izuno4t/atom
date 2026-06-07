# Evaluation

このディレクトリは、`atom` の評価に必要なリソースを集約する。

## ディレクトリ構成

| パス | 目的 | Git管理 |
| ---- | ---- | ---- |
| `bin/` | 評価用Cargoバイナリ | 管理対象 |
| `tools.md` | 評価対象ツールの選定 | 管理対象 |
| `methods/evaluation.md` | 評価手法 | 管理対象 |
| `methods/benchmark.md` | ベンチマーク手法 | 管理対象 |
| `tool-runners/` | 比較ツールのDockerfileとラッパー実装 | 管理対象 |
| `inputs/` | 評価入力ファイルの置き場 | 中身は管理外 |
| `outputs/` | 人が確認するMarkdown出力の置き場 | 中身は管理外 |
| `reports/` | JSONレポートや集計結果の置き場 | 中身は管理外 |

`inputs/`、`outputs/`、`reports/` は `.gitkeep` だけを管理し、実ファイルは
Git管理外にする。

## 評価用バイナリ

評価用の実行ファイルは `evaluation/bin/` に置く。評価ツールは
`atom-evaluation` crate に分離し、変換本体の公開APIとは独立して管理する。
実行時は `cargo run -p atom-evaluation --bin ...` を使う。

| バイナリ | 目的 |
| ---- | ---- |
| `atom-eval` | fixture と期待Markdownを比較する評価レポート生成 |
| `atom-compare-baseline` | 評価レポートをしきい値と比較する回帰検出 |
| `atom-bench` | 変換処理の簡易ベンチマーク |
| `atom-corpus-eval` | 実ディレクトリの文書を使った既存ツール比較 |
| `atom-llm-eval` | `atom-corpus-eval` の人手レビュー候補をローカルLLMで採点 |
| `atom-pdf-probe` | PDF抽出方式の成立性と速度を確認する検証用probe |

`atom-corpus-eval` の比較対象ツールは Docker コンテナとして実行する。
比較ツールのDockerfileとラッパー実装は `tool-runners/` に置く。
既定のDockerイメージは次の通り。

- `pandoc`: `atom-eval-pandoc:latest`
- `markitdown`: `atom-eval-markitdown:latest`
- `docling`: `atom-eval-docling:latest`
- `pymupdf4llm`: `atom-eval-pymupdf4llm:latest`
- `mammoth-js`: `atom-eval-mammoth-js:latest`

イメージを差し替える場合は、次の環境変数を指定する。

- `ATOM_EVAL_PANDOC_IMAGE`
- `ATOM_EVAL_MARKITDOWN_IMAGE`
- `ATOM_EVAL_DOCLING_IMAGE`
- `ATOM_EVAL_PYMUPDF4LLM_IMAGE`
- `ATOM_EVAL_MAMMOTH_JS_IMAGE`

比較ツールランナーは、Markdownとreport JSONを `evaluation/outputs/` 配下に
ファイルとして書き出す。標準出力は短い実行サマリだけに使う。

## 比較ツールイメージのビルド

```bash
docker build -t atom-eval-pandoc:latest evaluation/tool-runners/pandoc
docker build -t atom-eval-markitdown:latest evaluation/tool-runners/markitdown
docker build -t atom-eval-docling:latest evaluation/tool-runners/docling
docker build -t atom-eval-pymupdf4llm:latest evaluation/tool-runners/pymupdf4llm
docker build -t atom-eval-mammoth-js:latest evaluation/tool-runners/mammoth-js
```

## 実行例

定型コマンドは `make` から実行できる。

```bash
make bench
make corpus-eval
make corpus-eval-full
make llm-eval LLM_EVAL_DRY_RUN=1
make pdf-probe PDF_PROBE_INPUT=/path/to/input.pdf
```

`make corpus-eval` は初回確認向けに、既定では `atom` 単独、20件、
形式ごと5件までを評価する。件数やtoolは `make` 変数で上書きできる。

```bash
make corpus-eval EVAL_LIMIT=100 EVAL_PER_EXT=20 EVAL_TOOLS=atom
```

`make corpus-eval-full` は比較ツール込みの標準評価を実行する。Docker
イメージが必要な比較ツールは、未準備の場合 `missing` としてreportに記録される。

評価対象ドキュメントの実パスは、Git管理外の
`evaluation/atom-evaluation.config.toml` に保存できる。
`evaluation/atom-evaluation.config.toml.example` をコピーして、ローカル環境の
パスへ書き換える。

```toml
evaluation_root = "evaluation/inputs"
evaluation_output_root = "evaluation/outputs"
evaluation_report_path = "evaluation/reports/report.json"
```

```bash
cargo run -p atom-evaluation --bin atom-corpus-eval -- \
  --root evaluation/inputs \
  --out evaluation/reports/report.json \
  --output-root evaluation/outputs \
  --limit 30 \
  --per-ext 5 \
  --tools pandoc,markitdown
```

設定ファイルの評価パスを使う場合は、次のように実行する。

```bash
cargo run -p atom-evaluation --bin atom-corpus-eval -- \
  --config evaluation/atom-evaluation.config.toml \
  --limit 200 \
  --per-ext 40 \
  --tools pandoc,markitdown,docling,pymupdf4llm,mammoth-js \
  --timeout-ms 120000 \
  --max-bytes 52428800
```

形式別の重点評価コマンドは [methods/evaluation.md](methods/evaluation.md) を参照する。
PDFだけを100件評価する場合は次のように実行する。

```bash
cargo run -p atom-evaluation --bin atom-corpus-eval -- \
  --root evaluation/inputs \
  --out evaluation/reports/pdf-100-report.json \
  --output-root evaluation/outputs \
  --limit 100 \
  --per-ext 100 \
  --ext pdf \
  --tools docling,pymupdf4llm
```

出力Markdownは `evaluation/outputs/<tool>/` に保存されるため、人が直接確認できる。

LLMを使う評価は、先に `make corpus-eval-full` で `review_candidates` を含む
report JSONを作ってから実行する。Ollama構築、JSONLスキーマ、プロンプトは
[methods/llm-evaluation.md](methods/llm-evaluation.md) を参照する。
