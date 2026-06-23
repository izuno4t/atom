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

LLMで見出し、リスト、表、画像参照、脚注などの構造を整える場合:

```bash
atom input.pdf --llm ollama:llama3 --restructure -o output.md
```

画像やイラストを説明文書にする場合:

```bash
atom diagram.png --llm ollama:llava -o output.md
```

テキストレイヤーのないPDFをOCRで読む場合:

```bash
atom scanned.pdf --ocr tesseract -o output.md
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

ユーザー共通設定は次のどちらかに置けます。

- `~/.atom/atom.config.toml`
- `~/.atom/config.toml`

優先順位は、明示したCLI引数、`--config <PATH>`、`~/.atom` 配下のユーザー共通設定、
アプリ内の既定値の順です。

```toml
flavor = "gfm"
format = "markdown"
strict = false
consent_external_send = false
llm = "ollama:llama3"
llm.prompt_path.restructure = "prompts/restructure.md"
llm.prompt_path.translate = "prompts/translate.md"
llm.prompt_path.image-description = "prompts/image-description.md"
llm.prompt_path.ocr-postprocess = "prompts/ocr-postprocess.md"
```

プロンプトをカスタマイズする場合は、設定ファイルへ本文を直接書かず、
プロンプトファイルのパスを指定します。既定プロンプトはアプリ内の既定値を使います。
相対パスは設定ファイルが置かれたディレクトリから解決されます。

プロンプトファイルでは `{input}` に入力MarkdownやOCRテキスト、`{language}` に
翻訳先言語が入ります。例えば `~/.atom/prompts/translate.md` を指定すると、
翻訳時だけそのファイルを使えます。

## LLM、画像、OCR

### ローカルLLMで文書構造を整える

`--restructure` は、通常の変換結果をLLMへ渡して、構造化されたMarkdownへ整えます。
見出し、リスト、表、画像参照、コードブロック、脚注などの構造が失われた応答は
採用されません。

```bash
atom report.pdf --llm ollama:llama3 --restructure -o report.md
```

### 画像やイラストをMarkdownにする

画像ファイルを入力すると、VLMに画像の内容説明を依頼してMarkdownを生成します。
ローカルのOllamaモデルを使う場合は外部送信の同意は不要です。

```bash
atom chart.png --llm ollama:llava -o chart.md
```

クラウドLLM/VLMへ画像や文書を送る場合は、必ず `--allow-external-send` が必要です。

### OCRが必要なPDFを読む

PDFにテキストレイヤーがない場合、`--ocr` を指定するとOCR fallbackを試します。
OCR結果はPDFページ本文としてMarkdownへ統合されます。
OCRエンジンには `ocr-rs`、`ndlocr-lite`、`ndl-koten`、`tesseract`、`surya`、
または外部コマンド名を指定できます。

```bash
atom scanned.pdf --ocr tesseract -o scanned.md
```

`ocr-rs` を使う場合は、モデルファイルのパスを環境変数で指定します。

```bash
export ATOM_OCR_RS_DET_MODEL=/path/to/det.onnx
export ATOM_OCR_RS_REC_MODEL=/path/to/rec.onnx
export ATOM_OCR_RS_CHARSET=/path/to/charset.txt
atom scanned.pdf --ocr ocr-rs -o scanned.md
```

## 対応状況

| 入力形式 | 状態 |
| --- | --- |
| HTML | 基本的な見出し、段落、リスト、コード、テーブルに対応 |
| DOCX | OOXML の本文、見出し、リスト、画像、キャプション、テーブルに対応 |
| PDF | Rust組み込みバックエンドによるテキスト抽出、レイアウト推論、OCR fallback境界に対応 |
| PPTX | OOXML スライドテキスト、リスト、視覚順読み出しに対応 |
| XLSX | OOXML シートテーブル、結合セル、複数ヘッダーに対応 |
| ODT / ODS / ODP | OpenDocument の `content.xml` から見出し、段落、表を抽出 |

現在の実装では、複雑なPDFの完全な論理構造復元と、商用OCRエンジンの
環境構築は継続実装中です。

## ローカル完結と外部送信

atom は、通常の変換では文書を外部へ送信しません。

クラウド LLM を使う場合は、`--llm` に加えて
`--allow-external-send` を指定してください。指定しない場合、外部送信が
必要な LLM 処理はスキップされ、レポートに warning が残ります。
`--strict` 時は、その warning がエラーとして扱われます。

ローカル LLM を使う場合:

```bash
atom input.pdf --llm ollama:llama3 --restructure -o output.md
```

画像入力をローカル VLM でMarkdown化する場合:

```bash
atom scan.png --llm ollama:llava -o output.md
```

クラウド LLM を明示的に許可する場合:

```bash
atom input.pdf --llm claude-opus --restructure --allow-external-send -o output.md
```

OpenAI は `OPENAI_API_KEY`、Anthropic は `ANTHROPIC_API_KEY` を参照します。
OpenAI互換endpointは次の形式で指定できます。

```bash
atom input.pdf --llm openai-compatible:local@https://llm.example.com/v1 \
  --restructure --allow-external-send -o output.md
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
