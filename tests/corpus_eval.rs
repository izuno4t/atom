use std::fs;
use std::process::Command;

#[test]
fn corpus_eval_outputs_comparison_report() {
    let root = "target/corpus-eval-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("target/corpus-eval-test/report.json")
        .arg("--output-root")
        .arg("target/corpus-eval-test/outputs")
        .arg("--limit")
        .arg("1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string("target/corpus-eval-test/report.json").unwrap();
    assert!(report.contains("\"tool\":\"atom\""));
    assert!(report.contains("\"summary\""));
    assert!(report.contains("\"superiority_claim\""));
    assert!(fs::read_dir("target/corpus-eval-test/outputs/atom").is_ok());
    let index = fs::read_to_string("target/corpus-eval-test/outputs/review-index.md").unwrap();
    assert!(index.contains("# Corpus Evaluation Review Index"));
    assert!(index.contains("sample.html"));
}

#[test]
fn corpus_eval_can_filter_by_extension() {
    let root = "target/corpus-eval-filter-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();
    fs::write(format!("{root}/sample.txt"), "plain text").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("target/corpus-eval-filter-test/report.json")
        .arg("--output-root")
        .arg("target/corpus-eval-filter-test/outputs")
        .arg("--limit")
        .arg("10")
        .arg("--per-ext")
        .arg("10")
        .arg("--ext")
        .arg("txt")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string("target/corpus-eval-filter-test/report.json").unwrap();
    assert!(report.contains("\"txt\":1"));
    assert!(!report.contains("\"html\":1"));
}

#[test]
fn corpus_eval_does_not_select_markdown_inputs() {
    let root = "target/corpus-eval-md-filter-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.md"), "# Already markdown").unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("target/corpus-eval-md-filter-test/report.json")
        .arg("--output-root")
        .arg("target/corpus-eval-md-filter-test/outputs")
        .arg("--limit")
        .arg("10")
        .arg("--per-ext")
        .arg("10")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string("target/corpus-eval-md-filter-test/report.json").unwrap();
    assert!(report.contains("\"html\":1"));
    assert!(!report.contains("\"md\":1"));
    assert!(!report.contains("sample.md"));
}

#[test]
fn corpus_eval_marks_too_large_inputs_excluded() {
    let root = "target/corpus-eval-too-large-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("target/corpus-eval-too-large-test/report.json")
        .arg("--output-root")
        .arg("target/corpus-eval-too-large-test/outputs")
        .arg("--limit")
        .arg("1")
        .arg("--max-bytes")
        .arg("1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string("target/corpus-eval-too-large-test/report.json").unwrap();
    assert!(report.contains("\"status\":\"too_large\""));
    assert!(report.contains("\"judgment\":\"excluded: too_large\""));
}

#[test]
fn corpus_eval_can_mark_external_tool_timeout() {
    let root = "target/corpus-eval-timeout-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("target/corpus-eval-timeout-test/report.json")
        .arg("--output-root")
        .arg("target/corpus-eval-timeout-test/outputs")
        .arg("--limit")
        .arg("1")
        .arg("--tools")
        .arg("pandoc")
        .arg("--timeout-ms")
        .arg("0")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string("target/corpus-eval-timeout-test/report.json").unwrap();
    assert!(report.contains("\"tool\":\"pandoc\""));
    assert!(report.contains("\"status\":\"timeout\""));
}
