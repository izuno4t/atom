use std::fs;
use std::path::{Path, PathBuf};

use bonjil::{Flavor, html, markdown};

#[test]
fn html_unit_fixtures_match_expected_markdown() {
    let fixture_dir = Path::new("tests/fixtures/unit/html");
    let mut cases = find_html_fixtures(fixture_dir);
    cases.sort();

    assert!(!cases.is_empty(), "HTML unit fixtures must not be empty");

    for html_path in cases {
        let stem = html_path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("HTML fixture must have a UTF-8 stem");
        let expected_path = fixture_dir.join(format!("{stem}.expected.md"));
        let html = fs::read_to_string(&html_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", html_path.display()));
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", expected_path.display()));
        let mut warnings = Vec::new();

        let ast = html::parse_html(&html, &mut warnings);
        let actual = markdown::write_markdown(&ast, Flavor::Gfm);

        assert_eq!(
            actual, expected,
            "fixture {stem} did not match expected Markdown"
        );
    }
}

fn find_html_fixtures(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("html"))
        .collect()
}
