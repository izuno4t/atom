use std::env;
use std::path::PathBuf;
use std::time::Instant;

use pdf::content::{Op, TextDrawAdjusted};
use pdf::error::PdfError;
use pdf::file::FileOptions;

fn main() {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("usage: pdf-rs-text-probe <pdf>"));

    let started = Instant::now();
    let result = run(&path);
    let elapsed_ms = started.elapsed().as_millis();
    match result {
        Ok(stats) => {
            println!(
                "path\tstatus\telapsed_ms\tpages\tops\ttext_ops\tchars\terrors\tfonts\tto_unicode_fonts\tfont_sample\tsample\n{}\tok\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                path.display(),
                elapsed_ms,
                stats.pages,
                stats.ops,
                stats.text_ops,
                stats.chars,
                stats.errors,
                stats.fonts,
                stats.to_unicode_fonts,
                stats.font_sample,
                stats.sample.replace('\t', "\\t").replace('\n', "\\n")
            );
        }
        Err(error) => {
            println!(
                "path\tstatus\telapsed_ms\tpages\tops\ttext_ops\tchars\terrors\tfonts\tto_unicode_fonts\tfont_sample\tsample\n{}\terror\t{}\t0\t0\t0\t0\t{}\t0\t0\t\t",
                path.display(),
                elapsed_ms,
                error.to_string().replace('\t', "\\t").replace('\n', "\\n")
            );
        }
    }
}

#[derive(Default)]
struct Stats {
    pages: usize,
    ops: usize,
    text_ops: usize,
    chars: usize,
    errors: usize,
    fonts: usize,
    to_unicode_fonts: usize,
    font_sample: String,
    sample: String,
}

fn run(path: &PathBuf) -> Result<Stats, PdfError> {
    let file = FileOptions::cached().open(path)?;
    let resolver = file.resolver();
    let mut stats = Stats::default();

    for page in file.pages() {
        stats.pages += 1;
        let page = page?;
        if let Some(resources) = page.resources.as_ref() {
            for (name, font) in &resources.fonts {
                stats.fonts += 1;
                if stats.font_sample.is_empty() {
                    stats.font_sample = format!("{name:?}");
                }
                if let Ok(font) = font.load(&resolver)
                    && font.to_unicode(&resolver).is_some()
                {
                    stats.to_unicode_fonts += 1;
                }
            }
        }
        let Some(content) = page.contents.as_ref() else {
            continue;
        };
        let ops = match content.operations(&resolver) {
            Ok(ops) => ops,
            Err(_) => {
                stats.errors += 1;
                continue;
            }
        };
        stats.ops += ops.len();
        for op in ops {
            match op {
                Op::TextDraw { text } => {
                    stats.text_ops += 1;
                    let text = text.to_string_lossy();
                    stats.chars += text.chars().count();
                    append_sample(&mut stats.sample, &text);
                }
                Op::TextDrawAdjusted { array } => {
                    stats.text_ops += 1;
                    for item in array {
                        if let TextDrawAdjusted::Text(text) = item {
                            let text = text.to_string_lossy();
                            stats.chars += text.chars().count();
                            append_sample(&mut stats.sample, &text);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(stats)
}

fn append_sample(sample: &mut String, text: &str) {
    if sample.chars().count() >= 400 {
        return;
    }
    sample.push_str(text);
    sample.push('\n');
}
