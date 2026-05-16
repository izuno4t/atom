use std::fs;
use std::path::Path;

#[test]
fn fixture_corpora_has_three_layers_and_manifests() {
    let root = Path::new("tests/fixtures");
    for layer in ["unit", "integration", "regression"] {
        let layer_dir = root.join(layer);
        assert!(layer_dir.is_dir(), "{layer} fixture layer is missing");
        assert!(
            layer_dir.join("README.md").is_file(),
            "{layer} fixture README is missing"
        );
    }

    for layer in ["integration", "regression"] {
        let manifest_path = root.join(layer).join("MANIFEST.md");
        let manifest = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", manifest_path.display()));
        assert!(
            manifest.contains("## Source and License Register"),
            "{} must document source and license tracking",
            manifest_path.display()
        );
        assert!(
            !manifest.contains("TBD"),
            "{} must not contain unknown fixture provenance",
            manifest_path.display()
        );
    }
}
