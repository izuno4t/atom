# atom

Anything to Markdown.

atom は、HTML、PDF、Office 文書、OpenDocument、画像、OCR が必要な文書を、
人が読みやすい構造化 Markdown に変換するための CLI ツールです。

デフォルトでは外部 API に文書を送信しません。LLM、VLM、クラウド OCR を使う
処理は、明示的に有効化し、クラウド送信の場合は外部送信を許可した場合だけ
実行されます。

英語版は [README.md](README.md) を参照してください。

## インストール

### macOS

Homebrew でインストールします。

```bash
brew tap izuno4t/atom
brew install atom
```

Homebrew package は example config を `share/atom` に配置します。

```bash
mkdir -p ~/.atom
cp "$(brew --prefix)/share/atom/config.toml.example" ~/.atom/config.toml
```

### Linux / Windows

[最新の GitHub Release](https://github.com/izuno4t/atom/releases/latest) から
利用する環境向けの ZIP をダウンロードしてください。

各リリース ZIP には次のファイルが含まれます。

- `atom` 実行ファイル。Windows では `atom.exe`
- `config.toml.example`

ユーザー共通設定を使う場合は、同梱された `config.toml.example` を
`~/.atom/config.toml` にコピーしてください。例はリポジトリ内の
[config.toml.example](config.toml.example) でも確認できます。

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

warning をエラーとして扱う場合:

```bash
atom input.pdf --strict -o output.md
```

## LLM、画像、OCR

### Provider setup

LLM/VLM backend は `--llm` で選びます。

API key は `~/.atom/config.toml` ではなく環境変数から読みます。
設定ファイルには provider、model 名、prompt file path を置き、secret は
shell、CI、secret manager に置く設計です。これは credential 漏えい防止の
ためです。
secret 値を直接 command に入力すると shell history に残る可能性があるため、
その形での入力は避けてください。
atom は `ATOM_*` の環境変数名だけを使い、他の tool 向けに設定された API key を
誤って再利用しないようにします。

| Provider | Selector | Env var |
| --- | --- | --- |
| Ollama | `ollama:<model>` | 不要 |
| OpenAI API | `gpt-*` | `ATOM_OPENAI_API_KEY` |
| Anthropic | `claude-*` | `ATOM_ANTHROPIC_API_KEY` |
| Gemini | `gemini:<model>` または `gemini-*` | `ATOM_GEMINI_API_KEY` |
| OpenAI 互換 | `openai-compatible:*` | `ATOM_OPENAI_COMPATIBLE_API_KEY` |

クラウド provider では、文書テキストと、画像入力の場合は画像 byte を
選択した provider へ送るため、`--allow-external-send` が必要です。

OpenAI 互換 gateway は `openai-compatible:<name>@<endpoint>` で指定します。

OpenAI API:

```bash
atom input.pdf --llm gpt-4o-mini --restructure --allow-external-send -o output.md
```

Gemini:

```bash
atom input.pdf --llm gemini:gemini-2.5-flash --restructure \
  --allow-external-send -o output.md
```

OpenAI 互換 gateway は、OpenAI chat completions API 形式を公開している
service 向けです。一般的な OpenAPI schema があるだけでは足りず、
OpenAI 互換の chat completion request を受けられる endpoint が必要です。

```bash
atom input.pdf --llm openai-compatible:gateway@https://llm.example.com/v1 \
  --restructure --allow-external-send -o output.md
```

### 文書構造を LLM で整える

`--restructure` は、通常の変換結果を LLM へ渡して、構造化された Markdown へ
整えます。見出し、リスト、表、画像参照、コードブロック、脚注などの構造が
失われた応答は採用されません。

ローカル Ollama の例:

```bash
atom input.pdf --llm ollama:llama3 --restructure -o output.md
```

### 画像やイラストを Markdown にする

画像ファイルを入力すると、選択した vision-capable LLM/VLM に画像の内容説明を
依頼して Markdown を生成します。

ローカル VLM の例:

```bash
atom diagram.png --llm ollama:llava -o diagram.md
```

クラウド VLM の例:

```bash
atom scan.png --llm gemini:gemini-2.5-flash --allow-external-send -o scan.md
```

### OCR が必要な文書を読む

PDF や画像にテキスト認識が必要な場合は `--ocr` を指定します。

```bash
atom scanned.pdf --ocr tesseract -o scanned.md
```

OCR エンジンには `auto`、`ocr-rs`、`ndlocr-lite`、`ndl-koten`、`tesseract`、
`surya`、`none`、または外部コマンド名を指定できます。外部 OCR エンジンが
見つからない場合は、必要な backend 名と setup 上の含意を warning として
報告します。

## 設定

atom が読むユーザー共通設定ファイルは 1 つです。

```text
~/.atom/config.toml
```

設定は次の順で適用され、後の値が前の値を上書きします。

1. アプリ内の既定値
2. `~/.atom/config.toml`
3. `--config <PATH>`
4. 明示した CLI オプション

設定形式は TOML 風の `key = "value"` です。

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

prompt path は設定ファイルが置かれたディレクトリから解決されます。

| Prompt path key | 使われる処理 |
| --- | --- |
| `llm.prompt_path.restructure` | `--restructure` で変換済み Markdown を整える時 |
| `llm.prompt_path.translate` | `--translate <LANG>` で Markdown を翻訳する時 |
| `llm.prompt_path.image-description` | 画像入力、または VLM で画像 caption を生成する時 |
| `llm.prompt_path.ocr-postprocess` | OCR text を Markdown 出力前に後処理する時 |

プロンプトファイルでは次の placeholder を使えます。

| Placeholder | 意味 |
| --- | --- |
| `{input}` | 入力 Markdown、OCR テキスト、画像 context |
| `{markdown}` | `{input}` の alias |
| `{language}` | `--translate` の翻訳先言語 |

## オプション

| オプション | 説明 |
| --- | --- |
| `-o, --output <PATH>` | 出力先。省略時は標準出力 |
| `-f, --format <FMT>` | 出力形式。`md`、`mdx`、`html` |
| `--flavor <FLAVOR>` | Markdown 方言を指定 |
| `--extract-media <DIR>` | メディア抽出先 |
| `--inline-base64-media` | 対応可能なメディアを Base64 埋め込み |
| `--ocr <ENGINE>` | OCR エンジンを指定 |
| `--llm <MODEL>` | LLM backend を指定 |
| `--restructure` | LLM で構造を再整形 |
| `--translate <LANG>` | LLM で指定言語へ翻訳 |
| `--report <PATH>` | 変換レポート JSON の保存先 |
| `--strict` | warning をエラーとして扱う |
| `--config <PATH>` | 追加の設定ファイルを読み込む |
| `--allow-external-send` | 選択したクラウド LLM/VLM backend への送信を許可 |

## 対応入力

| 入力形式 | 状態 |
| --- | --- |
| HTML | 見出し、段落、リスト、コード、テーブル |
| DOCX | OOXML の本文、見出し、リスト、画像、キャプション、テーブル |
| PDF | 組み込み text extraction、layout inference、OCR fallback 境界 |
| PPTX | OOXML スライドテキスト、リスト、視覚順読み出し |
| XLSX | OOXML シートテーブル、結合セル、複数ヘッダー |
| ODT / ODS / ODP | `content.xml` から見出し、段落、表を抽出 |
| 画像 | vision-capable backend による Markdown 説明生成 |

## 外部送信

通常の変換では文書を外部へ送信しません。

クラウド LLM/VLM 処理は、`--allow-external-send` または
`consent_external_send = true` がない場合はスキップされます。スキップ時は
変換レポートに warning が残ります。`--strict` 時はその warning がエラーとして
扱われます。

## 開発

開発環境、テスト、リリース確認は [CONTRIBUTE.md](CONTRIBUTE.md) を参照してください。
