use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anything_to_markdown::{
    ConversionOptions, Converter, apply_config, apply_user_config, parse_flavor, parse_format,
    parse_llm, parse_ocr,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("atom: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1).peekable();
    let mut input = None;
    let mut output = None;
    let mut options = ConversionOptions::default();
    let _ = apply_user_config(&mut options)?;
    let mut extract_media_from_output = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => output = args.next().map(PathBuf::from),
            "-f" | "--format" => {
                if let Some(value) = args.next() {
                    options.format = parse_format(&value).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("unknown format: {value}"),
                        )
                    })?;
                }
            }
            "--flavor" => {
                if let Some(value) = args.next() {
                    options.flavor = parse_flavor(&value).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("unknown flavor: {value}"),
                        )
                    })?;
                }
            }
            "--extract-media" => extract_media_from_output = true,
            "--inline-base64-media" => options.inline_base64_media = true,
            "--ocr" => {
                if let Some(value) = args.next() {
                    options.ocr = parse_ocr(&value);
                }
            }
            "--llm" => {
                if let Some(value) = args.next() {
                    options.llm = parse_llm(&value);
                }
            }
            "--restructure" => options.restructure = true,
            "--translate" => options.translate = args.next(),
            "--report" => options.report_path = args.next().map(PathBuf::from),
            "--strict" => options.strict = true,
            "--config" => {
                if let Some(value) = args.next() {
                    let config_path = PathBuf::from(value);
                    apply_config(&mut options, &config_path)?;
                }
            }
            "--allow-external-send" => options.consent_external_send = true,
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                print_version();
                return Ok(());
            }
            value if value.starts_with('-') => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown option: {value}"),
                ));
            }
            value => input = Some(PathBuf::from(value)),
        }
    }

    let input =
        input.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing input path"))?;
    if extract_media_from_output && options.inline_base64_media {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--extract-media and --inline-base64-media are mutually exclusive",
        ));
    }
    if extract_media_from_output {
        let output_path = output.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--extract-media requires -o/--output",
            )
        })?;
        options.extract_media = Some(media_dir_for_output(output_path));
    }
    let converter = Converter::new().with_options(options.clone());
    let result = converter.convert_file(&input)?;

    if options.strict && !result.report.warnings.is_empty() {
        return Err(io::Error::other(format!(
            "strict mode failed with {} warning(s)",
            result.report.warnings.len()
        )));
    }

    if let Some(report_path) = &options.report_path {
        fs::write(report_path, result.report.to_json())?;
    }

    if let Some(output_path) = output {
        fs::write(output_path, result.markdown)?;
    } else {
        io::stdout().write_all(result.markdown.as_bytes())?;
    }

    Ok(())
}

fn media_dir_for_output(output_path: &std::path::Path) -> PathBuf {
    let mut media_dir = output_path.to_path_buf();
    media_dir.set_extension("");
    media_dir
}

fn print_help() {
    println!(
        "\
atom [INPUT] [OPTIONS]

Options:
  -o, --output <PATH>         Output path, stdout when omitted
  -f, --format <FMT>          md, mdx, html
  --flavor <FLAVOR>           commonmark, gfm, markdownlint, hedgedoc
  --extract-media             Extract media to a directory named after output file
  --inline-base64-media       Embed media as base64 where supported
  --ocr <ENGINE>              auto, ocr-rs, ndlocr-lite, ndl-koten, tesseract, surya, none
  --llm <MODEL>               claude-*, gpt-*, ollama:*, none
  --restructure               Apply LLM restructure filter
  --translate <LANG>          Translate with selected LLM
  --report <PATH>             Write conversion report JSON
  --strict                    Treat warnings as errors
  --config <PATH>             Load atom.config.toml-style config
                              User config is also read from ~/.atom/atom.config.toml
  --allow-external-send       Allow selected cloud LLM backend to receive input
  -V, --version               Print version
"
    );
}

fn print_version() {
    println!("atom {}", env!("CARGO_PKG_VERSION"));
}
