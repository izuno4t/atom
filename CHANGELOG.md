# Changelog

## v1.0.0

### New Features

- Convert HTML, PDF, Office, OpenDocument, image, and scanned document inputs to
  structured Markdown.
- Preserve common document structure, including headings, lists, tables, image
  references, code blocks, and footnotes where available.
- Support LLM-assisted restructuring, translation, image description, and OCR
  post-processing.
- Support local-first operation by default, with explicit consent required for
  cloud LLM/VLM input sending.

### Distribution

- Provide release packages for macOS, Linux, and Windows.
- Provide Homebrew installation through `izuno4t/tap`.
- Include `config.toml.example` in release packages.

### Documentation

- Add English and Japanese README documentation for installation,
  configuration, LLM providers, OCR, and external-send behavior.

## v0.1.2

### Bug Fixes

- Fix GitHub Release asset publishing.
- Fix Homebrew tap update preparation.
- Use the supported `macos-26` runner label for release builds.

### Documentation

- Improve the English and Japanese READMEs.

## v0.1.1

### Bug Fixes

- Fix GitHub Release asset publishing.
- Fix Homebrew tap update preparation.
- Use the supported `macos-26` runner label for release builds.

### Documentation

- Improve the English and Japanese READMEs.

## v0.1.0

### New Features

- Initial release of the `atom` command-line converter.
- Convert HTML, PDF, Office, OpenDocument, images, and scanned documents to
  structured Markdown.
- Support LLM-assisted restructuring, image descriptions, translation, and OCR
  workflows with explicit external-send consent.

### Distribution

- Add GitHub Release packaging for macOS, Linux, and Windows.
- Add Homebrew tap release preparation.
- Include `config.toml.example` in release packages.
