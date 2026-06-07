use std::fs;
use std::process::Command;

#[test]
fn corpus_eval_outputs_comparison_report() {
    let root = "../target/corpus-eval-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("../target/corpus-eval-test/report.json")
        .arg("--output-root")
        .arg("../target/corpus-eval-test/outputs")
        .arg("--limit")
        .arg("1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string("../target/corpus-eval-test/report.json").unwrap();
    assert!(report.contains("\"tool\":\"atom\""));
    assert!(report.contains("\"summary\""));
    assert!(report.contains("\"superiority_claim\""));
    assert!(fs::read_dir("../target/corpus-eval-test/outputs/atom").is_ok());
    let index = fs::read_to_string("../target/corpus-eval-test/outputs/review-index.md").unwrap();
    assert!(index.contains("# Corpus Evaluation Review Index"));
    assert!(index.contains("sample.html"));
}

#[test]
fn corpus_eval_reads_local_evaluation_paths_from_config() {
    let root = "../target/corpus-eval-config-test/input";
    let output_root = "../target/corpus-eval-config-test/outputs";
    let report_path = "../target/corpus-eval-config-test/report.json";
    let config_path = "../target/corpus-eval-config-test/atom-evaluation.config.toml";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();
    fs::write(
        config_path,
        format!(
            r#"
evaluation_root = "{root}"
evaluation_output_root = "{output_root}"
evaluation_report_path = "{report_path}"
"#
        ),
    )
    .unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--config")
        .arg(config_path)
        .arg("--limit")
        .arg("1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string(report_path).unwrap();
    assert!(report.contains("\"tool\":\"atom\""));
    assert!(fs::read_dir(format!("{output_root}/atom")).is_ok());
}

#[test]
fn corpus_eval_can_filter_by_extension() {
    let root = "../target/corpus-eval-filter-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();
    fs::write(format!("{root}/sample.txt"), "plain text").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("../target/corpus-eval-filter-test/report.json")
        .arg("--output-root")
        .arg("../target/corpus-eval-filter-test/outputs")
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
    let report = fs::read_to_string("../target/corpus-eval-filter-test/report.json").unwrap();
    assert!(report.contains("\"txt\":1"));
    assert!(!report.contains("\"html\":1"));
}

#[test]
fn corpus_eval_does_not_select_markdown_inputs() {
    let root = "../target/corpus-eval-md-filter-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.md"), "# Already markdown").unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("../target/corpus-eval-md-filter-test/report.json")
        .arg("--output-root")
        .arg("../target/corpus-eval-md-filter-test/outputs")
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
    let report = fs::read_to_string("../target/corpus-eval-md-filter-test/report.json").unwrap();
    assert!(report.contains("\"html\":1"));
    assert!(!report.contains("\"md\":1"));
    assert!(!report.contains("sample.md"));
}

#[test]
fn corpus_eval_marks_too_large_inputs_excluded() {
    let root = "../target/corpus-eval-too-large-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("../target/corpus-eval-too-large-test/report.json")
        .arg("--output-root")
        .arg("../target/corpus-eval-too-large-test/outputs")
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
    let report = fs::read_to_string("../target/corpus-eval-too-large-test/report.json").unwrap();
    assert!(report.contains("\"status\":\"too_large\""));
    assert!(report.contains("\"judgment\":\"excluded: too_large\""));
}

#[test]
fn corpus_eval_can_mark_external_tool_timeout() {
    let root = "../target/corpus-eval-timeout-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(format!("{root}/sample.html"), "<h1>Title</h1><p>Body</p>").unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("../target/corpus-eval-timeout-test/report.json")
        .arg("--output-root")
        .arg("../target/corpus-eval-timeout-test/outputs")
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
    let report = fs::read_to_string("../target/corpus-eval-timeout-test/report.json").unwrap();
    assert!(report.contains("\"tool\":\"pandoc\""));
    assert!(report.contains("\"status\":\"timeout\""));
    assert!(report.contains("\"status_by_extension\""));
    assert!(report.contains("\"html\":{\"atom\":{\"ok\":1},\"pandoc\":{\"timeout\":1}}"));
    assert!(report.contains("\"status_by_tool\""));
    assert!(report.contains("\"atom\":{\"ok\":1}"));
    assert!(report.contains("\"pandoc\":{\"timeout\":1}"));
    assert!(report.contains("\"failure_reasons\""));
    assert!(report.contains("\"external tool timed out before execution\":1"));
    assert!(report.contains("\"review_candidates\""));
    assert!(report.contains("\"fewer_than_two_tools_succeeded\""));
    let index =
        fs::read_to_string("../target/corpus-eval-timeout-test/outputs/review-index.md").unwrap();
    assert!(index.contains("## Review Candidates"));
    let summary =
        fs::read_to_string("../target/corpus-eval-timeout-test/outputs/evaluation-summary.md")
            .unwrap();
    assert!(summary.contains("# Corpus Evaluation Summary"));
    assert!(summary.contains("## Status By Tool"));
}

#[test]
fn corpus_eval_summarizes_markdown_structure_metrics() {
    let root = "../target/corpus-eval-structure-test/input";
    fs::create_dir_all(root).unwrap();
    fs::write(
        format!("{root}/sample.html"),
        "<h1>Title</h1><p>Body</p><ul><li>One</li></ul><pre><code>x</code></pre>",
    )
    .unwrap();

    let bin = std::env::var("CARGO_BIN_EXE_atom-corpus-eval")
        .expect("atom-corpus-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg("../target/corpus-eval-structure-test/report.json")
        .arg("--output-root")
        .arg("../target/corpus-eval-structure-test/outputs")
        .arg("--limit")
        .arg("1")
        .arg("--tools")
        .arg("atom")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string("../target/corpus-eval-structure-test/report.json").unwrap();
    assert!(report.contains("\"structure_average_by_tool\""));
    assert!(report.contains("\"atom\":{\"bytes\":"));
    assert!(report.contains("\"paragraphs\":1"));
    assert!(report.contains("\"list_items\":1"));
    assert!(report.contains("\"code_blocks\":1"));
    assert!(report.contains("\"warning_count\":0"));
    assert!(report.contains("\"report_feature_count\":"));
    assert!(report.contains("\"short_outputs\""));
}

#[test]
fn llm_eval_dry_run_writes_prompt_requests_from_review_candidates() {
    let root = "../target/llm-eval-dry-run-test";
    fs::create_dir_all(format!("{root}/outputs/atom")).unwrap();
    fs::write(format!("{root}/outputs/atom/sample.md"), "# Title\n\nBody").unwrap();
    fs::write(
        format!("{root}/report.json"),
        format!(
            r#"{{"summary":{{"review_candidates":[{{"input":"sample.html","priority":"medium","reasons":["fewer_than_two_tools_succeeded"],"output_paths":[{{"tool":"atom","path":"{root}/outputs/atom/sample.md"}}]}}]}}}}"#
        ),
    )
    .unwrap();

    let bin =
        std::env::var("CARGO_BIN_EXE_atom-llm-eval").expect("atom-llm-eval binary path is missing");
    let output = Command::new(bin)
        .arg("--report")
        .arg(format!("{root}/report.json"))
        .arg("--out")
        .arg(format!("{root}/llm-eval.jsonl"))
        .arg("--dry-run")
        .arg("--limit")
        .arg("1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let requests = fs::read_to_string(format!("{root}/llm-eval.jsonl")).unwrap();
    assert!(requests.contains("\"schema_version\":\"atom.llm-eval.v1\""));
    assert!(requests.contains("\"prompt_kind\":\"single_markdown_score\""));
    assert!(requests.contains("Evaluate the Markdown output on its own"));
}
