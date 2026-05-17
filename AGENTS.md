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

## Agent-Specific Instructions

Follow `AGENTS.local.md` for repository-local workflow rules. Base
implementation work on `docs/requirements.md`, update `docs/tasks.md`
statuses for tracked tasks, run `make ci` before marking implementation
tasks complete, and do not weaken evaluation thresholds to mask failures.
Do not run `git commit` or `git push`; leave version-control publishing
to the repository owner.
