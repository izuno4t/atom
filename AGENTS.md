# Repository Guidelines

## Project Structure & Module Organization

`bonjil` is a Rust CLI that converts HTML, PDF, and Office documents
into structured Markdown. Core library code lives in `src/lib.rs`; the
CLI entry point is `src/main.rs`. Evaluation binaries are in
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

## Coding Style & Naming Conventions

Use Rust 2024 edition conventions and format code with `cargo fmt`.
Prefer clear module boundaries, explicit CLI errors, and stable report
schemas. Use `snake_case` for Rust functions, modules, and test names.
Use kebab-case for generated files and CLI-facing examples.

## Testing Guidelines

Add or update tests in `tests/` for behavior changes. Use fixtures for
document conversion regressions, and keep expected outputs
human-reviewed. Do not update `tests/fixtures/**/*.expected.md` only to
hide failures; document the input or writer change that justifies it.
Run `make regression-test` when output, scoring, or fixtures change.
Do not weaken evaluation functions to hide failures, and do not lower
`tests/thresholds.toml` thresholds to mask regressions. Check evaluation
JSON, diffs, and warnings before changing implementation.

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
`bonjil.toml.example` as the configuration reference, and avoid
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
installed `bonjil` / `bj` binary provides the basic reader capability.
Do not design normal document conversion so that users must separately
install ad hoc external CLI tools.

Prefer Rust crates that can be compiled into the binary when they are
sufficient. When existing Rust crates are not sufficient for quality or
coverage, consult proven implementations in other languages such as
Python, JavaScript, Java, or Go before designing the Rust implementation.
Record the referenced libraries, official specifications, or official
documentation in the implementation report or related docs.

OCR engines are an exception when the engine itself is external. In that
case, missing backends must be reported clearly with the required backend
name and setup implication.
