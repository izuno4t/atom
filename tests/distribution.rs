use std::fs;

#[test]
fn release_workflow_builds_cross_platform_artifacts() {
    let workflow =
        fs::read_to_string(".github/workflows/release.yml").expect("release workflow must exist");

    for required in [
        "ubuntu-latest",
        "macos-latest",
        "windows-latest",
        "cargo build --release",
        "actions/upload-artifact",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow must contain {required}"
        );
    }
}
