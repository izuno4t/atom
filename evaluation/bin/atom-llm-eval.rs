use std::env;
use std::io;
use std::path::PathBuf;

use atom_evaluation::llm_eval::{self, LlmEvalOptions};

fn main() {
    if let Err(error) = run() {
        eprintln!("atom-llm-eval: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let options = Args::parse().into_options();
    llm_eval::run(options)?;
    println!("{}", Args::parse_output_path());
    Ok(())
}

struct Args {
    report_path: PathBuf,
    output_path: PathBuf,
    ollama_url: String,
    model: String,
    limit: usize,
    dry_run: bool,
}

impl Args {
    fn parse() -> Self {
        let mut report_path = PathBuf::from("evaluation/reports/report.json");
        let mut output_path = PathBuf::from("evaluation/reports/llm-eval.jsonl");
        let mut ollama_url = "http://127.0.0.1:11434".to_string();
        let mut model = "qwen2.5:7b-instruct".to_string();
        let mut limit = 20;
        let mut dry_run = false;
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--report" => {
                    if let Some(value) = args.next() {
                        report_path = PathBuf::from(value);
                    }
                }
                "--out" => {
                    if let Some(value) = args.next() {
                        output_path = PathBuf::from(value);
                    }
                }
                "--ollama-url" => {
                    if let Some(value) = args.next() {
                        ollama_url = value;
                    }
                }
                "--model" => {
                    if let Some(value) = args.next() {
                        model = value;
                    }
                }
                "--limit" => {
                    if let Some(value) = args.next().and_then(|value| value.parse().ok()) {
                        limit = value;
                    }
                }
                "--dry-run" => dry_run = true,
                _ => {}
            }
        }
        Self {
            report_path,
            output_path,
            ollama_url,
            model,
            limit,
            dry_run,
        }
    }

    fn into_options(self) -> LlmEvalOptions {
        LlmEvalOptions {
            report_path: self.report_path,
            output_path: self.output_path,
            ollama_url: self.ollama_url,
            model: self.model,
            limit: self.limit,
            dry_run: self.dry_run,
        }
    }

    fn parse_output_path() -> String {
        let mut output_path = PathBuf::from("evaluation/reports/llm-eval.jsonl");
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--out"
                && let Some(value) = args.next()
            {
                output_path = PathBuf::from(value);
            }
        }
        output_path.display().to_string()
    }
}
