use std::fs;

#[test]
fn release_workflow_builds_cross_platform_artifacts() {
    let workflow =
        fs::read_to_string(".github/workflows/release.yml").expect("release workflow must exist");

    for required in [
        "ubuntu-latest",
        "macos-26-arm64",
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
        "tags:",
        "v*",
        "gh release upload \"$GITHUB_REF_NAME\"",
        "gh release upload",
        "List packages",
        "Formula/atom.rb",
        "Compress-Archive",
        "zip -r",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow must contain {required}"
        );
    }

    for forbidden in [
        "actions/upload-artifact",
        "actions/download-artifact",
        "softprops/action-gh-release",
        "macos-latest",
        "workflow_dispatch",
        "branches:",
        "- main",
        "@v4",
        "@v2",
    ] {
        assert!(
            !workflow.contains(forbidden),
            "release workflow must not contain {forbidden}"
        );
    }
}

#[test]
fn ci_workflow_uses_current_node_actions() {
    let workflow = fs::read_to_string(".github/workflows/ci.yml").expect("CI workflow must exist");

    for required in ["actions/checkout@v5", "actions/setup-node@v5"] {
        assert!(
            workflow.contains(required),
            "CI workflow must contain {required}"
        );
    }

    for forbidden in ["@v4", "@v2"] {
        assert!(
            !workflow.contains(forbidden),
            "CI workflow must not contain {forbidden}"
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
