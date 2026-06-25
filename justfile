default:
    cargo test --workspace

test:
    cargo test --workspace

eval:
    cargo test -p anything-to-markdown --test integration
    cargo run -p atom-evaluation --bin atom-eval -- tests/fixtures/unit/docx target/eval-report.json
    cat target/eval-report.json

review:
    cargo test --workspace

bench:
    cargo run -p anything-to-markdown --bin atom -- tests/fixtures/unit/html/basic.html >/dev/null

compare-baseline:
    cargo test --workspace

ci: test eval bench compare-baseline

bump VERSION:
    ./scripts/bump-version.sh {{VERSION}}
