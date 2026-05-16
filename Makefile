.PHONY: default test eval review bench compare-baseline ci fmt lint clippy

default: test

test:
	cargo test

eval:
	cargo test --test integration
	cargo run --bin bonjil-eval -- tests/fixtures/unit/docx target/eval-report.json
	cat target/eval-report.json

review:
	cargo test

bench:
	cargo run --bin bonjil-bench -- tests/fixtures/unit/html/basic.html 10

compare-baseline:
	cargo run --bin bonjil-compare-baseline -- target/eval-report.json tests/thresholds.toml

fmt:
	cargo fmt

lint:
	markdownlint-cli2 README.md docs/*.md CLAUDE.md AGENTS.local.md tests/fixtures/**/*.md benches/README.md

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

ci: fmt clippy test eval bench compare-baseline lint
