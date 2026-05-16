use std::fs;
use std::path::Path;

use bonjil::{Flavor, markdown, pdf};

#[test]
fn pdf_unit_fixtures_match_expected_markdown() {
    let fixture_dir = Path::new("tests/fixtures/unit/pdf");
    let pdf_path = fixture_dir.join("text-heading.pdf");
    let expected_path = fixture_dir.join("text-heading.expected.md");
    let bytes = fs::read(&pdf_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", pdf_path.display()));
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", expected_path.display()));
    let mut warnings = Vec::new();

    let ast = pdf::parse_pdf(&bytes, &mut warnings);
    let actual = markdown::write_markdown(&ast, Flavor::Gfm);

    assert_eq!(actual, expected);
}
