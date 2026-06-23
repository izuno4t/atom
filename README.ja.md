# atom ✨

[![CI][ci-badge]][ci]
[![Release][release-badge]][release]
[![Homebrew][homebrew-badge]][homebrew]
[![License][license-badge]][license]
[![Rust][rust-badge]][rust]
[![Platforms][platforms-badge]][release]

📄 Anything to Markdown。ローカルファイルから LLM 支援の文書整理まで。

atom は、HTML、PDF、Office 文書、OpenDocument、画像、OCR が必要な文書を、
人が読みやすい構造化 Markdown に変換するための CLI ツールです。

デフォルトでは外部 API に文書を送信しません。LLM、VLM、クラウド OCR を使う
処理は、明示的に有効化し、クラウド送信の場合は外部送信を許可した場合だけ
実行されます。

atom は、雑多な元資料をきれいな Markdown に移すための CLI です。

- 📚 HTML、PDF、DOCX、PPTX、XLSX、OpenDocument を読みやすい Markdown として
  保存できます。
- 🧱 見出し、リスト、表、画像参照、コードブロック、脚注などの構造を、
  元ファイルから取れる範囲で保ちます。
- 🖼️ 図、スクリーンショット、イラストを VLM で説明し、Markdown にできます。
- 🔎 スキャン PDF や画像に OCR をかけ、認識したテキストを Markdown にします。
- 🪄 設定済み LLM で、変換済み Markdown の再整形や翻訳ができます。

```bash
atom report.pdf --ocr auto --llm ollama:llama3 --restructure -o report.md
```

英語版は [README.md](README.md) を参照してください。

## 💡 モチベーション

変換元の文書が最初から適切に構造化されているなら、atom を作る必要は
あまりありません。その場合は既存の汎用 document converter で十分なことが
多いです。

atom が対象にするのは、現実によくある次のような文書です。

- 見た目は表でも、内部構造はテキストボックスや絶対配置の集合になっている
- 見出しが style ではなく、フォントサイズや太字だけで表現されている
- PDF 化によって段落、リスト、表、脚注、キャプションの意味が失われている
- PowerPoint の視覚順と XML 順が一致しない
- Excel の結合セル、空セル、複数ヘッダー行で Markdown 表が壊れやすい
- 図表番号、キャプション、画像入りセルが汎用 converter では脱落する
- OCR や LLM を使えば読めるが、ローカル完結で再現可能な処理にしにくい

atom の目標は、単に「LLM が読める Markdown」ではありません。人が確認し、
編集し、diff を取り、信頼できる構造化 Markdown を作ることです。

## ✨ 特長

- 📦 Web ページ、Office 文書、画像、スキャン PDF まで幅広く入力できます。
- 🏠 通常の変換、LLM 再整形、VLM 画像説明、OCR はローカル優先です。
- 🔐 クラウド LLM/VLM へ文書テキストや画像 byte を送る前に、明示的な許可を
  要求します。
- ⚙️ `~/.atom/config.toml` にユーザー共通設定を置き、再整形、翻訳、画像説明、
  OCR 後処理の prompt file を処理ごとに分けられます。
- 🚀 macOS、Linux、Windows 向けの release package と、macOS 向け Homebrew
  install に対応します。

## 🧭 対応入力

| 入力形式 | atom が抽出する内容 |
| --- | --- |
| HTML | 文書構造、リンク、表、コードブロック、画像 |
| DOCX | OOXML の本文、見出し、リスト、表、画像、キャプション |
| PDF | 組み込み text extraction、layout inference、OCR fallback 境界 |
| PPTX | スライドテキスト、リスト、視覚順のテキストボックス |
| XLSX | シート表、結合セル、複数ヘッダー |
| ODT / ODS / ODP | OpenDocument の `content.xml` |
| 画像 | vision-capable backend による Markdown 説明 |
| スキャン文書 | 設定済み OCR engine によるテキスト |

## 🚀 インストール

### 🍎 macOS

Homebrew でインストールします。

```bash
brew tap izuno4t/tap
brew install izuno4t/tap/atom
```

Homebrew package は example config を `share/atom` に配置します。

```bash
mkdir -p ~/.atom
cp "$(brew --prefix)/share/atom/config.toml.example" ~/.atom/config.toml
```

### 🐧 Linux / 🪟 Windows

[最新の GitHub Release](https://github.com/izuno4t/atom/releases/latest) から
利用する環境向けの ZIP をダウンロードしてください。

各リリース ZIP には次のファイルが含まれます。

- `atom` 実行ファイル。Windows では `atom.exe`
- `config.toml.example`

ユーザー共通設定を使う場合は、同梱された `config.toml.example` を
`~/.atom/config.toml` にコピーしてください。例はリポジトリ内の
[config.toml.example](config.toml.example) でも確認できます。

## ⚡ 基本的な使い方

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

よく使う workflow:

- 📄 DOCX レポートを変換する。

  ```bash
  atom report.docx -o report.md
  ```

- 🔎 PDF を OCR fallback 付きで抽出する。

  ```bash
  atom scanned.pdf --ocr auto -o scanned.md
  ```

- 🖼️ 画像をローカルで説明する。

  ```bash
  atom diagram.png --llm ollama:llava -o diagram.md
  ```

- 🪄 変換済み Markdown を再整形する。

  ```bash
  atom input.pdf --llm ollama:llama3 --restructure -o output.md
  ```

- 🌐 クラウド model で翻訳する。

  ```bash
  atom input.docx --llm gpt-4o-mini --translate ja \
    --allow-external-send -o output.md
  ```

## 🧠 LLM、画像、OCR

### 🔌 Provider setup

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

### 🪄 文書構造を LLM で整える

`--restructure` は、通常の変換結果を LLM へ渡して、構造化された Markdown へ
整えます。見出し、リスト、表、画像参照、コードブロック、脚注などの構造が
失われた応答は採用されません。

ローカル Ollama の例:

```bash
atom input.pdf --llm ollama:llama3 --restructure -o output.md
```

### 🖼️ 画像やイラストを Markdown にする

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

### 🔎 OCR が必要な文書を読む

PDF や画像にテキスト認識が必要な場合は `--ocr` を指定します。

```bash
atom scanned.pdf --ocr tesseract -o scanned.md
```

OCR エンジンには `auto`、`ocr-rs`、`ndlocr-lite`、`ndl-koten`、`tesseract`、
`surya`、`none`、または外部コマンド名を指定できます。外部 OCR エンジンが
見つからない場合は、必要な backend 名と setup 上の含意を warning として
報告します。

## ⚙️ 設定

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

## 🧰 オプション

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

## 🔐 外部送信

通常の変換では文書を外部へ送信しません。

クラウド LLM/VLM 処理は、`--allow-external-send` または
`consent_external_send = true` がない場合はスキップされます。スキップ時は
変換レポートに warning が残ります。`--strict` 時はその warning がエラーとして
扱われます。

## 🤝 開発

開発環境、テスト、リリース確認は [CONTRIBUTE.md](CONTRIBUTE.md) を参照してください。

## 📜 ライセンス

atom は [MIT License](LICENSE) で配布されています。

[ci-badge]: https://github.com/izuno4t/atom/actions/workflows/ci.yml/badge.svg
[ci]: https://github.com/izuno4t/atom/actions/workflows/ci.yml
[homebrew-badge]: https://img.shields.io/badge/Homebrew-izuno4t%2Ftap-fbb040.svg
[homebrew]: https://github.com/izuno4t/homebrew-tap
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license]: LICENSE
[platforms-badge]: https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-4c6fff.svg
[release-badge]: https://img.shields.io/github/v/release/izuno4t/atom?include_prereleases
[release]: https://github.com/izuno4t/atom/releases
[rust-badge]: https://img.shields.io/badge/Rust-2024-orange.svg
[rust]: https://www.rust-lang.org/
