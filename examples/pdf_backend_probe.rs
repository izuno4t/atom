use std::env;
use std::fs;
use std::time::Instant;

use anything_to_markdown::pdf::{
    self, InternalPdfTextBackend, LenientPdfExtractBackend, LopdfTextBackend, PdfExtractBackend,
    PdfOxideFormWordsBackend, PdfOxideTextBackend, PdfRsTextBackend, PdfTextBackend,
    RawContentTextBackend,
};

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("usage: cargo run --example pdf_backend_probe -- <pdf>"));
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    let filter = env::args().nth(2);
    let backends: [&dyn PdfTextBackend; 8] = [
        &PdfOxideFormWordsBackend,
        &PdfRsTextBackend,
        &PdfOxideTextBackend,
        &PdfExtractBackend,
        &LopdfTextBackend,
        &LenientPdfExtractBackend,
        &RawContentTextBackend,
        &InternalPdfTextBackend,
    ];

    println!("path\tbackend\telapsed_ms\tobjects\tchars\tfailed\tocr_required\twarnings");
    for backend in backends {
        if filter
            .as_ref()
            .is_some_and(|name| backend.name() != name.as_str())
        {
            continue;
        }
        let mut warnings = Vec::new();
        let started = Instant::now();
        let result = pdf::parse_pdf_with_backend(&bytes, backend, &mut warnings);
        let elapsed = started.elapsed().as_millis();
        let chars = result
            .ast
            .iter()
            .map(|node| format!("{node:?}").chars().count())
            .sum::<usize>();
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            path,
            result.backend,
            elapsed,
            result.ast.len(),
            chars,
            result.extraction_failed,
            result.ocr_required,
            warnings.len()
        );
    }
}
