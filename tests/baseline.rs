use std::fs;
use std::process::Command;

#[test]
fn compare_baseline_fails_on_eval_regression_and_accepts_clean_report() {
    let bin = std::env::var("CARGO_BIN_EXE_bonjil-compare-baseline")
        .expect("bonjil-compare-baseline binary path is missing");
    let dir = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("baseline-test");
    fs::create_dir_all(&dir).unwrap();
    let thresholds_path = dir.join("thresholds.toml");
    fs::write(
        &thresholds_path,
        "structure_fidelity = 0.90\nheading_recall = 0.90\ntable_integrity = 0.90\n",
    )
    .unwrap();

    let failing_report = dir.join("failing-report.json");
    fs::write(
        &failing_report,
        "{\"summary\":{\"total_fixtures\":1,\"passed\":0,\"failed\":1,\"lint_total_errors\":0},\"failures\":[{\"metric\":\"golden\",\"score\":0.0,\"expected\":1.0}]}\n",
    )
    .unwrap();
    let failing = Command::new(&bin)
        .arg(&failing_report)
        .arg(&thresholds_path)
        .output()
        .unwrap();
    assert!(
        !failing.status.success(),
        "baseline comparison must fail when report has failures"
    );

    let passing_report = dir.join("passing-report.json");
    fs::write(
        &passing_report,
        "{\"summary\":{\"total_fixtures\":1,\"passed\":1,\"failed\":0,\"lint_total_errors\":0},\"failures\":[]}\n",
    )
    .unwrap();
    let passing = Command::new(&bin)
        .arg(&passing_report)
        .arg(&thresholds_path)
        .output()
        .unwrap();
    assert!(
        passing.status.success(),
        "baseline comparison must accept a clean report: {}",
        String::from_utf8_lossy(&passing.stderr)
    );
}
