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

指定したディレクトリ直下の対象ファイルだけをMarkItDownでMarkdown化し、
TSVで棚卸しします。サブディレクトリは辿りません。

```bash
uv venv benchmark/.venv
uv pip install --python benchmark/.venv/bin/python -r benchmark/requirements-markitdown.txt

benchmark/.venv/bin/python benchmark/scripts/markitdown_inventory.py \
  --input-dir /path/to/input-directory \
  --out benchmark/reports/markitdown-inventory.tsv \
  --output-root benchmark/outputs/markitdown
```

TSVの列は次の通りです。

```text
path	status	elapsed_ms	chars	sha256	error
```
