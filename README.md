# atom

Anything to Markdown.

atom は、HTML、PDF、Office 文書などを、人が読みやすい構造化
Markdown に変換するための CLI ツールです。

デフォルトでは外部 API に文書を送信しません。LLM を使った再構造化や
翻訳は明示的に有効化した場合だけ実行されます。

## 特徴

- 1つのコマンドで複数形式の文書を Markdown に変換
- CommonMark、GFM、markdownlint 準拠などの方言を選択可能
- 複雑なテーブルは Markdown 表に無理に押し込まず HTML table に退避
- 変換時の警告、メタデータ、処理時間を JSON report として出力
- OCR と LLM はオプション扱いで、ローカル完結を優先

## インストール

Rust が入っている環境では、リポジトリからそのままビルドできます。

```bash
cargo build --release
```

開発環境を揃えたい場合は、Dev Containers を利用できます。

```bash
code .
```

VS Code で開いたあと、`Reopen in Container` を選択してください。
コンテナには Rust、make、just、Poppler、Tesseract、LibreOffice などが
含まれます。

## 基本的な使い方

標準出力へ変換結果を出す場合:

```bash
atom input.html
```

ファイルへ保存する場合:

```bash
atom input.docx -o output.md
```

Markdown 方言を指定する場合:

```bash
atom input.html --flavor gfm -o output.md
atom input.docx --flavor markdownlint -o output.md
```

変換レポートを JSON で保存する場合:

```bash
atom input.html -o output.md --report report.json
```

警告をエラーとして扱う場合:

```bash
atom input.pdf --strict -o output.md
```

## オプション

| オプション | 説明 |
| --- | --- |
| `-o, --output <PATH>` | 出力先。省略時は標準出力 |
| `-f, --format <FMT>` | 出力形式。`md`、`mdx`、`html` |
| `--flavor <FLAVOR>` | Markdown 方言を指定 |
| `--extract-media <DIR>` | 画像などのメディア抽出先 |
| `--inline-base64-media` | 対応可能なメディアを Base64 埋め込み |
| `--ocr <ENGINE>` | OCR エンジンを指定 |
| `--llm <MODEL>` | LLM バックエンド。`claude-*`、`gpt-*`、`ollama:*`、`none` |
| `--restructure` | LLM で構造を再整形 |
| `--translate <LANG>` | LLM で指定言語へ翻訳 |
| `--report <PATH>` | 変換レポート JSON の保存先 |
| `--strict` | warning をエラーとして扱う |
| `--config <PATH>` | 設定ファイルを読み込む |
| `--allow-external-send` | クラウド LLM への送信を許可 |

## 設定ファイル

設定ファイルは TOML 風の `key = "value"` 形式です。
例は [atom.config.toml.example](atom.config.toml.example) を参照してください。
実際の `atom.config.toml` はローカル専用で、Git管理外です。

```toml
flavor = "gfm"
format = "markdown"
strict = false
consent_external_send = false
evaluation_root = "evaluation/inputs"
evaluation_output_root = "evaluation/outputs"
evaluation_report_path = "evaluation/reports/report.json"
```

## 対応状況

| 入力形式 | 状態 |
| --- | --- |
| HTML | 基本的な見出し、段落、リスト、コード、テーブルに対応 |
| DOCX | OOXML の本文、見出し、リスト、画像、キャプション、テーブルを順次対応中 |
| PDF | テキスト抽出とOCR連携のための境界を実装中 |
| PPTX | OOXML スライドテキスト抽出を実装中 |
| XLSX | OOXML シートテーブル抽出を実装中 |

現在の実装は初期段階です。複雑なレイアウト、厳密なPDF構造復元、
商用OCRエンジン連携、LLMプロバイダ呼び出しは継続実装中です。

## ローカル完結と外部送信

atom は、通常の変換では文書を外部へ送信しません。

クラウド LLM を使う場合は、`--llm` に加えて
`--allow-external-send` を指定してください。指定しない場合、外部送信が
必要な LLM 処理はスキップされ、レポートに warning が残ります。

ローカル LLM を使う場合:

```bash
atom input.pdf --llm ollama:llama3 --restructure -o output.md
```

クラウド LLM を明示的に許可する場合:

```bash
atom input.pdf --llm claude-opus --restructure --allow-external-send -o output.md
```

## 開発者向け

よく使うコマンドは `make` から実行できます。

```bash
make test
make lint
make clippy
make verify
```

固定fixtureの回帰確認はCIにも含まれます。

```bash
make regression-test
```

実文書評価と性能確認はCIとは分けて実行します。

```bash
make bench
make corpus-eval
```

`just` を使う場合も同等の入口があります。

```bash
just test
just eval
```

要件と実行計画は以下を参照してください。

- [docs/requirements.md](docs/requirements.md)
- [docs/implementation-plan.md](docs/implementation-plan.md)
- [docs/tasks.md](docs/tasks.md)

## ライセンス

[LICENSE](LICENSE) を参照してください。
