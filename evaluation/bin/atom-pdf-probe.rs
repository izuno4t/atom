use std::env;
use std::fs;
use std::panic;
use std::time::Instant;

fn main() {
    let path = env::args().nth(1).expect("usage: atom-pdf-probe <pdf>");
    let bytes = fs::read(path).expect("read pdf");

    let started = Instant::now();
    let document = lopdf::Document::load_mem(&bytes).expect("load pdf");
    let load_elapsed = started.elapsed();
    println!(
        "objects={}\tpages={}",
        document.objects.len(),
        document.get_pages().len()
    );
    let mut plain_document = document.clone();
    plain_document.trailer.remove(b"Encrypt");
    let mut readable_pages = 0usize;
    let mut readable_bytes = 0usize;
    for object_id in plain_document.get_pages().values() {
        if let Ok(content) = plain_document.get_page_content(*object_id) {
            readable_pages += 1;
            readable_bytes += content.len();
        }
    }
    println!("raw-pages={readable_pages}\traw-content-bytes={readable_bytes}");
    if env::var_os("ATOM_PDF_PROBE_RAW_ONLY").is_some() {
        return;
    }

    let mut text = String::new();
    let extract_started = Instant::now();
    let extracted = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut output = pdf_extract::PlainTextOutput::new(&mut text);
        pdf_extract::output_doc(&document, &mut output)
    }));
    let extract_elapsed = extract_started.elapsed();

    let chars = text.chars().count();
    let lines = text.lines().filter(|line| !line.trim().is_empty()).count();
    println!(
        "mode=plain\tencrypted={}\tload={:.3}s\textract={:.3}s\tchars={}\tlines={}\tstatus={}",
        document.is_encrypted(),
        load_elapsed.as_secs_f64(),
        extract_elapsed.as_secs_f64(),
        chars,
        lines,
        extraction_status(extracted)
    );
    println!("{}", text.lines().take(8).collect::<Vec<_>>().join("\n"));

    let mut decrypted = document.clone();
    let mut decrypted_text = String::new();
    let decrypted_started = Instant::now();
    let decrypted_extracted = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut output = pdf_extract::PlainTextOutput::new(&mut decrypted_text);
        pdf_extract::output_doc_encrypted(&mut decrypted, &mut output, "")
    }));
    let decrypted_error = match &decrypted_extracted {
        Ok(Err(error)) => format!("{error}"),
        Err(_) => "panic".to_string(),
        Ok(Ok(())) => String::new(),
    };
    let decrypted_elapsed = decrypted_started.elapsed();
    println!(
        "mode=empty-password\tencrypted={}\textract={:.3}s\tchars={}\tlines={}\tstatus={}\terror={}",
        decrypted.is_encrypted(),
        decrypted_elapsed.as_secs_f64(),
        decrypted_text.chars().count(),
        decrypted_text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        extraction_status(decrypted_extracted),
        decrypted_error
    );
    println!(
        "{}",
        decrypted_text
            .lines()
            .take(8)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn extraction_status(
    extracted: Result<Result<(), pdf_extract::OutputError>, Box<dyn std::any::Any + Send>>,
) -> &'static str {
    match extracted {
        Ok(Ok(())) => "ok",
        Ok(Err(_)) => "error",
        Err(_) => "panic",
    }
}
