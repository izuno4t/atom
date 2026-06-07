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

設定は `benchmark/benchmark.config.toml` に置きます。この実ファイルは
Git管理外です。

```bash
cp benchmark/benchmark.config.toml.example benchmark/benchmark.config.toml
uv venv benchmark/.venv
uv pip install --python benchmark/.venv/bin/python -r benchmark/requirements-markitdown.txt

benchmark/.venv/bin/python benchmark/scripts/markitdown_inventory.py
```

TSVの列は `path`, `status`, `elapsed_ms`, `chars`, `sha256`, `error` です。
