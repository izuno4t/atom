use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

use bonjil::{ConversionOptions, convert_bytes};

fn main() {
    if let Err(error) = run() {
        eprintln!("bonjil-bench: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tests/fixtures/unit/html/basic.html"));
    let iterations = args
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(10)
        .max(1);
    let bytes = fs::read(&input)?;
    let started = Instant::now();
    for _ in 0..iterations {
        let _ = convert_bytes(
            input.to_string_lossy().as_ref(),
            &bytes,
            ConversionOptions::default(),
        )?;
    }
    println!(
        concat!(
            "{{",
            "\"input\":\"{}\",",
            "\"iterations\":{},",
            "\"elapsed_ms\":{},",
            "\"bytes\":{}",
            "}}"
        ),
        escape_json(&input.to_string_lossy()),
        iterations,
        started.elapsed().as_millis(),
        bytes.len()
    );
    Ok(())
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
