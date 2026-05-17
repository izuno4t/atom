use std::fs;

#[test]
fn agent_guardrails_document_expected_and_eval_rules() {
    let claude = fs::read_to_string("CLAUDE.md").expect("CLAUDE.md must exist");
    let agents = fs::read_to_string("AGENTS.md").expect("AGENTS.md must exist");
    let combined = format!("{claude}\n{agents}");

    for required in [
        "expected.md",
        "Do not update",
        "evaluation functions",
        "thresholds",
        "docs/tasks.md",
        "statuses",
    ] {
        assert!(
            combined.contains(required),
            "guardrails must mention {required}"
        );
    }
}
