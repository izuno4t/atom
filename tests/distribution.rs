use std::fs;

#[test]
fn release_workflow_builds_cross_platform_artifacts() {
    let workflow =
        fs::read_to_string(".github/workflows/release.yml").expect("release workflow must exist");

    for required in [
        "ubuntu-latest",
        "macos-latest",
        "windows-latest",
        "permissions:",
        "contents: write",
        "make verify",
        "cargo build --release --locked",
        "config.toml.example",
        "atom-${version}-source.tar.gz",
        "atom-${version}-source.tar.gz.sha256",
        "update-homebrew-tap",
        "HOMEBREW_TAP_GITHUB_TOKEN",
        "actions/download-artifact",
        "Formula/atom.rb",
        "Compress-Archive",
        "zip -r",
        "actions/upload-artifact",
        "softprops/action-gh-release",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow must contain {required}"
        );
    }
}

#[test]
fn make_install_installs_config_example_for_packagers() {
    let makefile = fs::read_to_string("Makefile").expect("Makefile must exist");

    for required in [
        "SHARE_DIR",
        "install config.toml.example",
        "$(SHARE_DIR)/config.toml.example",
    ] {
        assert!(
            makefile.contains(required),
            "Makefile must contain {required}"
        );
    }
}
