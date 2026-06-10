# Benchmark

`benchmark/` は、MarkItDown、Pandoc、Docling など外部ツールとの比較や
事前棚卸しを行う場所です。

`evaluation/` は atom 本体のfixture評価と回帰検出に使います。
外部ツール比較のrunnerやscriptは `benchmark/` に置きます。

## Layout

| Path | Purpose |
| ---- | ---- |
| `methods/` | 比較手法、改善サイクル、golden reviewの方針 |
| `scripts/` | 比較や棚卸し用の実行スクリプト |
| `tools/` | 外部ツールrunnerとDockerfile |

## Method Documents

- [Benchmark Method](methods/benchmark.md)
- [Golden Review](methods/golden-review.md)
- [評価から再評価までの改善サイクル](methods/improvement-cycle.md)
- [ディレクトリ別評価実行計画](methods/directory-evaluation-plan.md)

## MarkItDown Inventory

指定したディレクトリ直下の通常ファイルをMarkItDownでMarkdown化し、TSVで
棚卸しします。サブディレクトリは辿りません。MarkItDownで処理できない
ファイルも `error` として記録します。
M6のMarkItDown比較では、`benchmark/scripts/atom_supported_extensions.py` に
定義した `atom` 対応拡張子だけを対象にします。
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

## Directory Evaluation

複数ディレクトリを同じ条件で評価する場合は
`benchmark/scripts/run_directory_evaluation.py` を使います。このrunnerは
状態表を読み、ディレクトリ単位でMarkItDown棚卸し、`atom`棚卸し、inspection、
comparison、analysisを実行します。

```bash
benchmark/.venv/bin/python benchmark/scripts/run_directory_evaluation.py \
  --status-table benchmark/reports/directory-evaluation-status.tsv \
  --jobs 4 \
  --file-jobs 4 \
  --rerun-all
```

ディレクトリ別の棚卸しと比較結果は `benchmark/reports/by-directory/` に、
Markdown出力は `benchmark/outputs/by-directory/` に保存します。どちらも
Git管理外です。

人手確認用のサンプル抽出は次で実行します。

```bash
benchmark/.venv/bin/python benchmark/scripts/select_review_samples.py \
  --limit-per-bucket 10
```

## MarkItDown Output Inspection

MarkItDownが生成したMarkdownを元ファイルから抽出したテキストと比較し、
言語一致と大きな欠落・改変の疑いを検査します。

```bash
benchmark/.venv/bin/python benchmark/scripts/inspect_markitdown_outputs.py
```

検査結果は `benchmark/reports/markitdown-inspection.tsv` に書き出し、
概要を `benchmark/reports/markitdown-report.md` に追記します。

## atom Inventory and Comparison

同じ入力ディレクトリ直下の通常ファイルをatomでMarkdown化し、MarkItDownと
同じ検査基準で確認します。

```bash
cargo build
benchmark/.venv/bin/python benchmark/scripts/atom_inventory.py
benchmark/.venv/bin/python benchmark/scripts/inspect_markitdown_outputs.py \
  --inventory benchmark/reports/atom-inventory.tsv \
  --markdown-root benchmark/outputs/atom \
  --out benchmark/reports/atom-inspection.tsv \
  --report benchmark/reports/atom-report.md
benchmark/.venv/bin/python benchmark/scripts/compare_atom_markitdown.py
```

比較結果は `benchmark/reports/atom-vs-markitdown-report.md` に書き出します。
