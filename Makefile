INSTALL_DIR ?= $(HOME)/bin
PDF_PROBE_INPUT ?=

.PHONY: default test regression-test bench pdf-probe review verify ci fmt lint spell clippy install

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

pdf-probe:
	@test -n "$(PDF_PROBE_INPUT)" || (echo "PDF_PROBE_INPUT is required" && exit 2)
	cargo run -p atom-evaluation --bin atom-pdf-probe -- "$(PDF_PROBE_INPUT)"

fmt:
	cargo fmt

lint:
	markdownlint-cli2 README.md docs/*.md evaluation/*.md evaluation/methods/*.md benchmark/*.md benchmark/tools/*.md CLAUDE.md AGENTS.md tests/fixtures/**/README.md tests/fixtures/**/MANIFEST.md benches/README.md

spell:
	cspell

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

verify: fmt clippy test regression-test lint spell

ci: verify
