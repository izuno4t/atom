# atom

Anything to Markdown.

atom is a command-line converter that turns HTML, PDF, Office, OpenDocument,
images, and scanned documents into structured Markdown.

By default, atom does not send documents to external services. LLM, VLM, and
cloud OCR features run only when you explicitly enable the relevant option and,
for cloud providers, allow external sending.

Japanese documentation is available in [README.ja.md](README.ja.md).

## Install

### macOS

Install with Homebrew:

```bash
brew tap izuno4t/tap
brew install izuno4t/tap/atom
```

The Homebrew package installs the example config under `share/atom`:

```bash
mkdir -p ~/.atom
cp "$(brew --prefix)/share/atom/config.toml.example" ~/.atom/config.toml
```

### Linux and Windows

Download the ZIP archive for your platform from the
[latest GitHub release](https://github.com/izuno4t/atom/releases/latest).

Each release ZIP contains:

- the `atom` executable, or `atom.exe` on Windows
- `config.toml.example`

Copy the included `config.toml.example` to `~/.atom/config.toml` when you want
user-level defaults. The example file is also kept in this repository as
[config.toml.example](config.toml.example).

## Quick Start

Convert to standard output:

```bash
atom input.html
```

Write Markdown to a file:

```bash
atom input.docx -o output.md
```

Choose a Markdown flavor:

```bash
atom input.html --flavor gfm -o output.md
atom input.docx --flavor markdownlint -o output.md
```

Save a JSON conversion report:

```bash
atom input.html -o output.md --report report.json
```

Treat warnings as errors:

```bash
atom input.pdf --strict -o output.md
```

## LLM, Images, and OCR

### Provider Setup

Choose the LLM/VLM backend with `--llm`.

API keys are read from environment variables, not from `~/.atom/config.toml`.
Keep provider choice, model names, and prompt file paths in config; keep
secrets in your shell, CI, or secret manager to prevent credential leakage.
atom uses `ATOM_*` environment variable names so it does not accidentally reuse
API keys configured for other tools. Avoid typing secret values directly into
commands because shell history may record them.

| Provider | Selector | Env var |
| --- | --- | --- |
| Ollama | `ollama:<model>` | Not required |
| OpenAI API | `gpt-*` | `ATOM_OPENAI_API_KEY` |
| Anthropic | `claude-*` | `ATOM_ANTHROPIC_API_KEY` |
| Gemini | `gemini:<model>` or `gemini-*` | `ATOM_GEMINI_API_KEY` |
| OpenAI-compatible | `openai-compatible:*` | `ATOM_OPENAI_COMPATIBLE_API_KEY` |

Cloud providers require `--allow-external-send` because document text and,
for image input, image bytes are sent to the selected provider.

Use `openai-compatible:<name>@<endpoint>` for OpenAI-compatible gateways.

OpenAI API:

```bash
atom input.pdf --llm gpt-4o-mini --restructure --allow-external-send -o output.md
```

Gemini:

```bash
atom input.pdf --llm gemini:gemini-2.5-flash --restructure \
  --allow-external-send -o output.md
```

OpenAI-compatible gateways are for services that expose the OpenAI chat
completions API shape. A generic OpenAPI schema is not enough; the endpoint
must accept OpenAI-compatible chat completion requests.

```bash
atom input.pdf --llm openai-compatible:gateway@https://llm.example.com/v1 \
  --restructure --allow-external-send -o output.md
```

### Restructure Converted Documents

`--restructure` sends the converted Markdown to the selected LLM and asks it to
preserve structural attributes such as headings, lists, tables, image
references, code blocks, and footnotes.

Local Ollama example:

```bash
atom input.pdf --llm ollama:llama3 --restructure -o output.md
```

### Generate Markdown from Images

When the input is an image, atom asks the selected vision-capable LLM/VLM to
describe the visible content as Markdown.

Local VLM example:

```bash
atom diagram.png --llm ollama:llava -o diagram.md
```

Cloud VLM example:

```bash
atom scan.png --llm gemini:gemini-2.5-flash --allow-external-send -o scan.md
```

### Convert Scanned Documents with OCR

Use `--ocr` when a PDF or image needs text recognition.

```bash
atom scanned.pdf --ocr tesseract -o scanned.md
```

Supported OCR selectors are `auto`, `ocr-rs`, `ndlocr-lite`, `ndl-koten`,
`tesseract`, `surya`, `none`, or an external command name. External OCR engines
must be installed separately and are reported clearly when missing.

## Configuration

atom reads one user-level configuration file:

```text
~/.atom/config.toml
```

Configuration is applied in this order; later values override earlier values:

1. built-in defaults
2. `~/.atom/config.toml`
3. `--config <PATH>`
4. explicit CLI options

The config format is a simple TOML-style `key = "value"` file:

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

Prompt paths are resolved from the directory that contains the config file.

| Prompt path key | Used when |
| --- | --- |
| `llm.prompt_path.restructure` | `--restructure` rewrites converted Markdown |
| `llm.prompt_path.translate` | `--translate <LANG>` translates Markdown |
| `llm.prompt_path.image-description` | image input or VLM captioning |
| `llm.prompt_path.ocr-postprocess` | OCR text cleanup before Markdown output |

Prompt files can use these placeholders:

| Placeholder | Meaning |
| --- | --- |
| `{input}` | source Markdown, OCR text, or image context |
| `{markdown}` | alias for `{input}` |
| `{language}` | translation target for `--translate` |

## Options

| Option | Description |
| --- | --- |
| `-o, --output <PATH>` | Output path. stdout when omitted |
| `-f, --format <FMT>` | Output format: `md`, `mdx`, `html` |
| `--flavor <FLAVOR>` | Markdown flavor |
| `--extract-media <DIR>` | Media extraction destination |
| `--inline-base64-media` | Embed supported media as Base64 |
| `--ocr <ENGINE>` | OCR engine selector |
| `--llm <MODEL>` | LLM backend selector |
| `--restructure` | Restructure Markdown with the selected LLM |
| `--translate <LANG>` | Translate Markdown with the selected LLM |
| `--report <PATH>` | Write conversion report JSON |
| `--strict` | Treat warnings as errors |
| `--config <PATH>` | Load an additional config file |
| `--allow-external-send` | Allow selected cloud LLM/VLM input sending |

## Supported Inputs

| Input | Status |
| --- | --- |
| HTML | Headings, paragraphs, lists, code blocks, and tables |
| DOCX | OOXML body text, headings, lists, images, captions, and tables |
| PDF | Built-in text extraction, layout inference, and OCR fallback boundary |
| PPTX | OOXML slide text, lists, and visual-order extraction |
| XLSX | OOXML sheet tables, merged cells, and multi-header tables |
| ODT / ODS / ODP | Headings, paragraphs, and tables from `content.xml` |
| Images | Markdown descriptions through the selected vision-capable backend |

## External Sending

Normal conversion does not send documents outside the machine.

Cloud LLM/VLM processing is skipped unless `--allow-external-send` or
`consent_external_send = true` is configured. When skipped, atom records a
warning in the conversion report. With `--strict`, that warning becomes an
error.

## Contributing

Development setup, test commands, and release checks live in
[CONTRIBUTE.md](CONTRIBUTE.md).
