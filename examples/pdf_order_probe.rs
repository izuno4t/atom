use std::env;
use std::fs;
use std::io::{self, Read};
use std::time::Instant;

use anything_to_markdown::pdf::{
    self, AtomPdfTextBackend, InternalPdfTextBackend, PdfOxideFormWordsBackend,
    PdfOxideTextBackend, PdfRsTextBackend, PdfTextBackend, RawContentTextBackend,
};

fn main() {
    let candidates = env::args().skip(1).collect::<Vec<_>>();
    if candidates.is_empty() {
        panic!(
            "usage: cargo run --example pdf_order_probe -- <candidate>... < paths.txt\n\
             candidates: current, raw-before-rs, raw-first, atom-composite"
        );
    }

    let mut stdin = String::new();
    io::stdin()
        .read_to_string(&mut stdin)
        .expect("failed to read paths from stdin");

    println!(
        "path\tcandidate\telapsed_ms\tbackend\tobjects\tchars\textraction_failed\tocr_required\twarnings"
    );
    for path in stdin.lines().map(str::trim).filter(|path| !path.is_empty()) {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                for candidate in &candidates {
                    println!(
                        "{}\t{}\t0\tmissing_input\t0\t0\ttrue\ttrue\tfailed to read input: {}",
                        path, candidate, error
                    );
                }
                continue;
            }
        };
        for candidate in &candidates {
            let mut warnings = Vec::new();
            let started = Instant::now();
            let result = parse_candidate(candidate, &bytes, &mut warnings);
            let elapsed_ms = started.elapsed().as_millis();
            let chars = result
                .ast
                .iter()
                .map(|node| format!("{node:?}").chars().count())
                .sum::<usize>();
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                path,
                candidate,
                elapsed_ms,
                result.backend,
                result.ast.len(),
                chars,
                result.extraction_failed,
                result.ocr_required,
                warnings.len()
            );
        }
    }
}

fn parse_candidate(
    candidate: &str,
    bytes: &[u8],
    warnings: &mut Vec<String>,
) -> pdf::PdfParseResult {
    match candidate {
        "current" => {
            let backends: [&dyn PdfTextBackend; 5] = [
                &PdfOxideFormWordsBackend,
                &PdfRsTextBackend,
                &RawContentTextBackend,
                &PdfOxideTextBackend,
                &InternalPdfTextBackend,
            ];
            pdf::parse_pdf_with_ordered_backends(bytes, &backends, warnings)
        }
        "raw-before-rs" => {
            let backends: [&dyn PdfTextBackend; 5] = [
                &PdfOxideFormWordsBackend,
                &RawContentTextBackend,
                &PdfRsTextBackend,
                &PdfOxideTextBackend,
                &InternalPdfTextBackend,
            ];
            pdf::parse_pdf_with_ordered_backends(bytes, &backends, warnings)
        }
        "raw-first" => {
            let backends: [&dyn PdfTextBackend; 5] = [
                &RawContentTextBackend,
                &PdfOxideFormWordsBackend,
                &PdfRsTextBackend,
                &PdfOxideTextBackend,
                &InternalPdfTextBackend,
            ];
            pdf::parse_pdf_with_ordered_backends(bytes, &backends, warnings)
        }
        "atom-composite" => pdf::parse_pdf_with_backend(bytes, &AtomPdfTextBackend, warnings),
        other => panic!("unknown candidate: {other}"),
    }
}
