# Repository Guidelines

## Project Structure & Module Organization

atom is a Rust CLI for Anything to Markdown. It converts HTML, PDF, and
Office documents into structured Markdown. Core library code lives in
`src/lib.rs`; the CLI entry point is `src/main.rs`. Evaluation binaries are in
`evaluation/bin/`, with reports and corpora under `evaluation/`. Tests
live in `tests/`; fixtures and reviewed expected Markdown are under
`tests/fixtures/`. Requirements and work tracking are in `docs/`.

## Build, Test, and Development Commands

- `cargo build --release`: build optimized CLI binaries.
- `cargo test`: run the Rust test suite.
- `make test`: use the local test entry point.
- `make regression-test`: run integration tests and fixture evaluation.
- `make lint`: run markdownlint on docs and fixtures.
- `make clippy`: run Rust static checks with warnings denied.
- `make verify`: run fmt, clippy, tests, regression, lint, and spell check.
- `just test` / `just eval`: shorter common workflows.

## Development Workflow

For improvement work, use this loop:

1. Run `just test` to check the current test state.
2. Run `just eval` to inspect conversion report JSON.
3. Triage failing fixtures or warnings to the input parser, AST, writer, or
   evaluation function.
4. Apply the smallest correction that addresses the cause.
5. Run `just ci` to confirm there is no regression.

## Coding Style & Naming Conventions

Use Rust 2024 edition conventions and format code with `cargo fmt`.
Prefer clear module boundaries, explicit CLI errors, and stable report
schemas. Use `snake_case` for Rust functions, modules, and test names.
Use kebab-case for generated files and CLI-facing examples.
Keep `src/lib.rs` limited to module declarations and public re-exports.
Organize `src/` by functional responsibility:
`core/` for AST, options, reports, config, naming, JSON, and text helpers;
`pipeline/` for conversion control, input detection, media reference
collection, and report feature assembly; `parsers/` for input document
parsers; `writers/` for Markdown/HTML output writers; `evaluation/` for
metric functions; and `integrations/` for LLM/OCR/media backends. Name
modules by responsibility, not by a vague bucket such as `format`.

For OOXML files, keep shared package/XML helpers under `src/ooxml/`.
Expose document-type behavior through specific modules such as `docx`,
`pptx`, and `xlsx`. Do not create an `office` bucket unless it contains
actual cross-Office behavior; otherwise split by document type.

## Testing Guidelines

Add or update tests in `tests/` for behavior changes. Use fixtures for
document conversion regressions, and keep expected outputs
human-reviewed. Do not update `tests/fixtures/**/*.expected.md` only to
hide failures; document the input or writer change that justifies it.
Run `make regression-test` when output, scoring, or fixtures change.
Do not weaken evaluation functions to hide failures, and do not lower
`tests/thresholds.toml` thresholds to mask regressions. Check evaluation
JSON, diffs, and warnings before changing implementation.
Use `tests/inputs/` for local pre-evaluation conversion inputs. Files
placed there are not fixtures, are not expected-output baselines, and
must stay out of Git except for the directory README and `.gitkeep`.

## Commit & Pull Request Guidelines

Recent history uses concise imperative subjects, sometimes with prefixes
such as `feat:`. Keep commits focused, for example
`feat: add PPTX list extraction` or `Fix PDF heading inference`. Pull
requests should summarize changes, list verification commands, link
issues or tasks, and include output samples when conversion behavior
changes.

## Security & Configuration Tips

By default, do not send documents to external LLM or OCR services. Cloud
LLM use must be explicit with `--allow-external-send`. Use
`atom.config.toml.example` as the configuration reference, and avoid
committing private documents, credentials, or proprietary corpora.
If external sending is added, explicitly document the destination, sent
content, and consent setting.

## Agent-Specific Instructions

Base implementation work on `docs/requirements.md`. Update
`docs/tasks.md` statuses to 🚧 when starting and ✅ when completing
tracked tasks. Do not start tasks whose DependsOn entries are incomplete.
Run `make ci` before marking implementation tasks complete. Do not run
`git commit` or `git push`; leave version-control publishing to the
repository owner.

## Parser Implementation Policy

PDF, Office, image, and OCR-related format support must assume that the
installed `atom` binary provides the basic reader capability.
Do not design normal document conversion so that users must separately
install ad hoc external CLI tools.

Current implementation boundaries:

- DOCX has a minimal implementation that reads `word/document.xml` through
  `unzip`.
- PDF currently uses heuristic text extraction only.
- PPTX and XLSX have entry points and warnings implemented; structural
  conversion is planned for later expansion.
- OCR and LLM boundaries are implemented, and external calls are handled only
  when explicitly configured.

Prefer Rust crates that can be compiled into the binary when they are
sufficient. When existing Rust crates are not sufficient for quality or
coverage, consult proven implementations in other languages such as
Python, JavaScript, Java, or Go before designing the Rust implementation.
Record the referenced libraries, official specifications, or official
documentation in the implementation report or related docs.

Do not decide parser or backend execution order from the performance of a
single file or a small hand-picked sample. Before changing backend order,
verify the impact on the supported evaluation set, including success status,
speed distribution, failure classification, and outliers. Treat an individual
file's performance as a hypothesis for root-cause analysis, not as a decision
criterion by itself. If a special case is necessary, express it as a reusable
document-structure condition such as page count, object structure, extraction
quality, or detected PDF features, not as a file-name or sample-specific rule.

OCR engines are an exception when the engine itself is external. In that
case, missing backends must be reported clearly with the required backend
name and setup implication.
