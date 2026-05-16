use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bonjil::{Flavor, docx, evaluate_lint_score, markdown};

fn main() {
    if let Err(error) = run() {
        eprintln!("bonjil-eval: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let fixture_dir = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tests/fixtures/unit/docx"));
    let output_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/eval-report.json"));

    let mut cases = find_document_xml_fixtures(&fixture_dir)?;
    cases.sort();
    let mut results = Vec::new();

    for document_path in cases {
        let stem = document_path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".document.xml"))
            .ok_or_else(|| io::Error::other("invalid fixture file name"))?
            .to_string();
        let expected_path = fixture_dir.join(format!("{stem}.expected.md"));
        let rels_path = fixture_dir.join(format!("{stem}.rels.xml"));
        let diff_path = PathBuf::from(format!("target/diffs/{stem}.diff"));
        let document_xml = fs::read_to_string(&document_path)?;
        let rels_xml = fs::read_to_string(&rels_path).unwrap_or_default();
        let expected = fs::read_to_string(&expected_path)?;
        let mut warnings = Vec::new();
        let ast = docx::parse_document_xml_with_rels(&document_xml, &rels_xml, &mut warnings);
        let actual = markdown::write_markdown(&ast, Flavor::Gfm);
        let lint = evaluate_lint_score(&actual);
        let passed = actual == expected && lint.errors == 0;
        if !passed {
            write_diff(&diff_path, &expected, &actual)?;
        }
        results.push(FixtureResult {
            fixture: stem,
            passed,
            lint_errors: lint.errors,
            diff_path: diff_path.to_string_lossy().to_string(),
        });
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, render_report(&results))?;
    Ok(())
}

struct FixtureResult {
    fixture: String,
    passed: bool,
    lint_errors: usize,
    diff_path: String,
}

fn find_document_xml_fixtures(dir: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".document.xml"))
        })
        .collect())
}

fn write_diff(path: &Path, expected: &str, actual: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!(
            "--- expected\n+++ actual\n@@\n{}\n--- actual\n{}\n",
            expected, actual
        ),
    )
}

fn render_report(results: &[FixtureResult]) -> String {
    let passed = results.iter().filter(|result| result.passed).count();
    let failed = results.len().saturating_sub(passed);
    let lint_total_errors = results
        .iter()
        .map(|result| result.lint_errors)
        .sum::<usize>();
    let failures = results
        .iter()
        .filter(|result| !result.passed)
        .map(|result| {
            format!(
                "{{\"fixture\":\"{}\",\"metric\":\"golden\",\"score\":0.0,\"expected\":1.0,\"diff_path\":\"{}\"}}",
                escape_json(&result.fixture),
                escape_json(&result.diff_path)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"summary\":{{",
            "\"total_fixtures\":{},",
            "\"passed\":{},",
            "\"failed\":{},",
            "\"lint_total_errors\":{}",
            "}},",
            "\"failures\":[{}]",
            "}}\n"
        ),
        results.len(),
        passed,
        failed,
        lint_total_errors,
        failures
    )
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
