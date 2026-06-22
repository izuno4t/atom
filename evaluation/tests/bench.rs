use std::process::Command;

#[test]
fn bench_binary_outputs_machine_readable_metrics() {
    // CARGO_BIN_EXE_<name> はコンパイル時に cargo が設定する env! 用の値であり、
    // 実行時環境には存在しないため std::env::var では取得できない。
    let bin = env!("CARGO_BIN_EXE_atom-bench");
    let output = Command::new(bin)
        .arg("../tests/fixtures/unit/html/basic.html")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"input\""));
    assert!(stdout.contains("\"iterations\""));
    assert!(stdout.contains("\"elapsed_ms\""));
    assert!(stdout.contains("\"bytes\""));
}
