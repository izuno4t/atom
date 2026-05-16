use std::fs;

#[test]
fn agent_guardrails_document_expected_and_eval_rules() {
    let claude = fs::read_to_string("CLAUDE.md").expect("CLAUDE.md must exist");
    let agents = fs::read_to_string("AGENTS.local.md").expect("AGENTS.local.md must exist");
    let combined = format!("{claude}\n{agents}");

    for required in [
        "expected.md",
        "自動更新しない",
        "評価関数",
        "しきい値を下げ",
        "docs/tasks.md",
        "ステータス",
    ] {
        assert!(
            combined.contains(required),
            "guardrails must mention {required}"
        );
    }
}
