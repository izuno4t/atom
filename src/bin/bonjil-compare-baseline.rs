use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("bonjil-compare-baseline: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let report_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/eval-report.json"));
    let thresholds_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tests/thresholds.toml"));

    let report = fs::read_to_string(&report_path)?;
    let thresholds = parse_thresholds(&fs::read_to_string(&thresholds_path)?)?;
    for key in ["structure_fidelity", "heading_recall", "table_integrity"] {
        if !thresholds.contains_key(key) {
            return Err(io::Error::other(format!("missing threshold: {key}")));
        }
    }

    let failed = json_summary_usize(&report, "failed")?;
    let lint_total_errors = json_summary_usize(&report, "lint_total_errors")?;
    if failed > 0 {
        return Err(io::Error::other(format!(
            "evaluation baseline failed: {failed} fixture(s) below baseline"
        )));
    }
    if lint_total_errors > 0 {
        return Err(io::Error::other(format!(
            "evaluation baseline failed: {lint_total_errors} lint error(s)"
        )));
    }

    println!(
        "evaluation baseline passed: {} threshold(s), report {}",
        thresholds.len(),
        report_path.display()
    );
    Ok(())
}

fn parse_thresholds(input: &str) -> io::Result<HashMap<String, f64>> {
    let mut thresholds = HashMap::new();
    for (line_index, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once('=').ok_or_else(|| {
            io::Error::other(format!("invalid threshold line {}", line_index + 1))
        })?;
        let value = value
            .trim()
            .parse::<f64>()
            .map_err(|error| io::Error::other(format!("invalid threshold value: {error}")))?;
        if !(0.0..=1.0).contains(&value) {
            return Err(io::Error::other(format!(
                "threshold must be between 0.0 and 1.0: {}",
                key.trim()
            )));
        }
        thresholds.insert(key.trim().to_string(), value);
    }
    if thresholds.is_empty() {
        return Err(io::Error::other("threshold file is empty"));
    }
    Ok(thresholds)
}

fn json_summary_usize(input: &str, key: &str) -> io::Result<usize> {
    let marker = format!("\"{key}\":");
    let start = input
        .find(&marker)
        .ok_or_else(|| io::Error::other(format!("missing report summary key: {key}")))?
        + marker.len();
    let value = input[start..]
        .chars()
        .skip_while(|character| character.is_ascii_whitespace())
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    value
        .parse::<usize>()
        .map_err(|error| io::Error::other(format!("invalid report summary value: {error}")))
}
