# Evaluation

このディレクトリは、`atom` の評価に必要なリソースを集約する。

## ディレクトリ構成

| パス | 目的 | Git管理 |
| ---- | ---- | ---- |
| `bin/` | 評価用Cargoバイナリ | 管理対象 |
| `methods/` | atom本体評価の補助資料 | 管理対象 |
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
| `atom-pdf-probe` | PDF抽出方式の成立性と速度を確認する検証用probe |

外部ツールとの比較、MarkItDownなどを使う事前棚卸し、比較tool runnerは
`benchmark/` に置く。

## 実行例

定型コマンドは `make` から実行できる。

```bash
make bench
make pdf-probe PDF_PROBE_INPUT=/path/to/input.pdf
```
