use std::env;
use std::fs;
use std::time::Instant;

fn main() {
    let path = env::args().nth(1).expect("usage: atom-pdf-probe <pdf>");
    let bytes = fs::read(path).expect("read pdf");

    let started = Instant::now();
    let document = lopdf::Document::load_mem(&bytes).expect("load pdf");
    let load_elapsed = started.elapsed();

    let mut text = String::new();
    let extract_started = Instant::now();
    {
        let mut output = pdf_extract::PlainTextOutput::new(&mut text);
        pdf_extract::output_doc(&document, &mut output).expect("extract text");
    }
    let extract_elapsed = extract_started.elapsed();

    let chars = text.chars().count();
    let lines = text.lines().filter(|line| !line.trim().is_empty()).count();
    println!(
        "encrypted={}\tload={:.3}s\textract={:.3}s\tchars={}\tlines={}",
        document.is_encrypted(),
        load_elapsed.as_secs_f64(),
        extract_elapsed.as_secs_f64(),
        chars,
        lines
    );
    println!("{}", text.lines().take(8).collect::<Vec<_>>().join("\n"));
}
