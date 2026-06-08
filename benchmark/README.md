# Benchmark

`benchmark/` は、MarkItDown、Pandoc、Docling など外部ツールとの比較や
事前棚卸しを行う場所です。

`evaluation/` は atom 本体の評価に使い、外部ツール比較のrunnerやscriptは
ここに置きます。

## Layout

| Path | Purpose |
| ---- | ---- |
| `scripts/` | 比較や棚卸し用の実行スクリプト |
| `tools/` | 外部ツールrunnerとDockerfile |

## MarkItDown Inventory

指定したディレクトリ直下の通常ファイルをMarkItDownでMarkdown化し、TSVで
棚卸しします。サブディレクトリは辿りません。MarkItDownで処理できない
ファイルも `error` として記録します。
既存のTSVがある場合は記録済みのパスをスキップして再開します。
`per_file_timeout_seconds` を超えたファイルは `timeout` として記録します。
`retry_statuses` に `error,timeout` のように指定すると、該当ステータスだけ
既存TSVから外して再処理します。

設定は `benchmark/benchmark.config.toml` に置きます。この実ファイルは
Git管理外です。

```bash
cp benchmark/benchmark.config.toml.example benchmark/benchmark.config.toml
uv venv benchmark/.venv
UV_CACHE_DIR=benchmark/.uv-cache uv pip install --prerelease=allow \
  --python benchmark/.venv/bin/python \
  -r benchmark/requirements-markitdown.txt

benchmark/.venv/bin/python benchmark/scripts/markitdown_inventory.py
```

TSVの列は `path`, `status`, `elapsed_ms`, `chars`, `sha256`, `error` です。

## MarkItDown Output Inspection

MarkItDownが生成したMarkdownを元ファイルから抽出したテキストと比較し、
言語一致と大きな欠落・改変の疑いを検査します。

```bash
benchmark/.venv/bin/python benchmark/scripts/inspect_markitdown_outputs.py
```

検査結果は `benchmark/reports/markitdown-inspection.tsv` に書き出し、
概要を `benchmark/reports/markitdown-report.md` に追記します。
