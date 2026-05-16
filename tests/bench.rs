use std::process::Command;

#[test]
fn bench_binary_outputs_machine_readable_metrics() {
    let bin =
        std::env::var("CARGO_BIN_EXE_bonjil-bench").expect("bonjil-bench binary path is missing");
    let output = Command::new(bin)
        .arg("tests/fixtures/unit/html/basic.html")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"input\""));
    assert!(stdout.contains("\"iterations\""));
    assert!(stdout.contains("\"elapsed_ms\""));
    assert!(stdout.contains("\"bytes\""));
}
