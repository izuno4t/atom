INSTALL_DIR ?= $(HOME)/bin
EVAL_CONFIG ?= evaluation/atom-evaluation.config.toml
EVAL_LIMIT ?= 20
EVAL_PER_EXT ?= 5
EVAL_TOOLS ?= atom
EVAL_TIMEOUT_MS ?= 120000
EVAL_MAX_BYTES ?= 52428800
EVAL_FULL_LIMIT ?= 200
EVAL_FULL_PER_EXT ?= 40
EVAL_FULL_TOOLS ?= pandoc,markitdown,docling,pymupdf4llm,mammoth-js

.PHONY: default test regression-test bench corpus-eval corpus-eval-full review verify ci fmt lint spell clippy install

default: test

install:
	cargo build --release -p anything-to-markdown --bin atom
	install -d "$(INSTALL_DIR)"
	install target/release/atom "$(INSTALL_DIR)/atom"

test:
	cargo test --workspace

regression-test:
	cargo test -p anything-to-markdown --test integration
	cargo run -p atom-evaluation --bin atom-eval -- tests/fixtures/unit/docx target/eval-report.json
	cat target/eval-report.json
	cargo run -p atom-evaluation --bin atom-compare-baseline -- target/eval-report.json tests/thresholds.toml

review:
	cargo test --workspace

bench:
	cargo run -p atom-evaluation --bin atom-bench -- tests/fixtures/unit/html/basic.html 10

corpus-eval:
	cargo run -p atom-evaluation --bin atom-corpus-eval -- \
		--config "$(EVAL_CONFIG)" \
		--limit "$(EVAL_LIMIT)" \
		--per-ext "$(EVAL_PER_EXT)" \
		--tools "$(EVAL_TOOLS)" \
		--timeout-ms "$(EVAL_TIMEOUT_MS)" \
		--max-bytes "$(EVAL_MAX_BYTES)"

corpus-eval-full:
	cargo run -p atom-evaluation --bin atom-corpus-eval -- \
		--config "$(EVAL_CONFIG)" \
		--limit "$(EVAL_FULL_LIMIT)" \
		--per-ext "$(EVAL_FULL_PER_EXT)" \
		--tools "$(EVAL_FULL_TOOLS)" \
		--timeout-ms "$(EVAL_TIMEOUT_MS)" \
		--max-bytes "$(EVAL_MAX_BYTES)"

fmt:
	cargo fmt

lint:
	markdownlint-cli2 README.md docs/*.md evaluation/*.md evaluation/methods/*.md evaluation/tool-runners/*.md CLAUDE.md AGENTS.md tests/fixtures/**/README.md tests/fixtures/**/MANIFEST.md benches/README.md

spell:
	cspell

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

verify: fmt clippy test regression-test lint spell

ci: verify
