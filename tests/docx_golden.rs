use std::fs;
use std::path::{Path, PathBuf};

use bonjil::{Flavor, docx, markdown};

#[test]
fn docx_unit_fixtures_match_expected_markdown() {
    let fixture_dir = Path::new("tests/fixtures/unit/docx");
    let mut cases = find_document_xml_fixtures(fixture_dir);
    cases.sort();

    assert!(!cases.is_empty(), "DOCX unit fixtures must not be empty");

    for document_path in cases {
        let stem = document_path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".document.xml"))
            .expect("fixture name must end with .document.xml");
        let expected_path = fixture_dir.join(format!("{stem}.expected.md"));
        let rels_path = fixture_dir.join(format!("{stem}.rels.xml"));
        let document_xml = fs::read_to_string(&document_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", document_path.display()));
        let rels_xml = fs::read_to_string(&rels_path).unwrap_or_default();
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", expected_path.display()));
        let mut warnings = Vec::new();

        let ast = docx::parse_document_xml_with_rels(&document_xml, &rels_xml, &mut warnings);
        let actual = markdown::write_markdown(&ast, Flavor::Gfm);

        assert_eq!(
            actual, expected,
            "fixture {stem} did not match expected Markdown"
        );
    }
}

fn find_document_xml_fixtures(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".document.xml"))
        })
        .collect()
}
