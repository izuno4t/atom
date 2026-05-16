default:
    cargo test

test:
    cargo test

eval:
    cargo test --test integration
    cargo run --bin bonjil-eval -- tests/fixtures/unit/docx target/eval-report.json
    cat target/eval-report.json

review:
    cargo test

bench:
    cargo run --bin bonjil -- tests/fixtures/unit/html/basic.html >/dev/null

compare-baseline:
    cargo test

ci: test eval bench compare-baseline
