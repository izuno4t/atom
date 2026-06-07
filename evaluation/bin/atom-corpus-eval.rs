use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anything_to_markdown::{ConversionOptions, Converter, Flavor};

const SUPPORTED_EXTENSIONS: &[&str] =
    &["pdf", "html", "htm", "docx", "pptx", "xlsx", "epub", "txt"];

fn main() {
    if let Err(error) = run() {
        eprintln!("atom-corpus-eval: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args = Args::parse()?;
    let files = select_files(&args.root, args.limit, args.per_ext, &args.extensions)?;
    let output_root = args.output_root;
    fs::create_dir_all(&output_root)?;

    let mut cases = Vec::new();
    for file in files {
        cases.push(evaluate_file(
            &file,
            &output_root,
            &args.tools,
            args.max_bytes,
            args.timeout_ms,
        )?);
    }

    let summary = summarize(&cases);
    fs::write(
        output_root.join("review-index.md"),
        render_review_index(&summary, &cases),
    )?;
    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, render_report(&args.root, &summary, &cases))?;
    println!("{}", args.out.display());
    Ok(())
}

struct Args {
    root: PathBuf,
    out: PathBuf,
    output_root: PathBuf,
    limit: usize,
    per_ext: usize,
    extensions: Option<Vec<String>>,
    tools: Vec<String>,
    timeout_ms: u64,
    max_bytes: u64,
}

#[derive(Default)]
struct EvalConfig {
    root: Option<PathBuf>,
    out: Option<PathBuf>,
    output_root: Option<PathBuf>,
}

impl Args {
    fn parse() -> io::Result<Self> {
        let raw_args = env::args().skip(1).collect::<Vec<_>>();
        let (config_path, explicit_config) = config_path_from_args(&raw_args);
        let config = if config_path.exists() {
            load_eval_config(&config_path)?
        } else if explicit_config {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("config file not found: {}", config_path.display()),
            ));
        } else {
            EvalConfig::default()
        };

        let mut root = config
            .root
            .unwrap_or_else(|| PathBuf::from("evaluation/inputs"));
        let mut out = config
            .out
            .unwrap_or_else(|| PathBuf::from("evaluation/reports/report.json"));
        let mut output_root = config
            .output_root
            .unwrap_or_else(|| PathBuf::from("evaluation/outputs"));
        let mut limit = 30;
        let mut per_ext = 5;
        let mut extensions = None;
        let mut tools = vec!["pandoc".to_string(), "markitdown".to_string()];
        let mut timeout_ms = 120_000;
        let mut max_bytes = 50 * 1024 * 1024;
        let mut args = raw_args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--config" => {
                    let _ = args.next();
                }
                "--root" => {
                    if let Some(value) = args.next() {
                        root = PathBuf::from(value);
                    }
                }
                "--out" => {
                    if let Some(value) = args.next() {
                        out = PathBuf::from(value);
                    }
                }
                "--output-root" => {
                    if let Some(value) = args.next() {
                        output_root = PathBuf::from(value);
                    }
                }
                "--limit" => {
                    if let Some(value) = args.next().and_then(|value| value.parse().ok()) {
                        limit = value;
                    }
                }
                "--per-ext" => {
                    if let Some(value) = args.next().and_then(|value| value.parse().ok()) {
                        per_ext = value;
                    }
                }
                "--ext" => {
                    if let Some(value) = args.next() {
                        extensions = Some(
                            value
                                .split(',')
                                .map(|item| item.trim().trim_start_matches('.').to_lowercase())
                                .filter(|item| !item.is_empty())
                                .collect(),
                        );
                    }
                }
                "--tools" => {
                    if let Some(value) = args.next() {
                        tools = value
                            .split(',')
                            .map(|item| item.trim().to_lowercase())
                            .filter(|item| !item.is_empty())
                            .collect();
                    }
                }
                "--timeout-ms" => {
                    if let Some(value) = args.next().and_then(|value| value.parse().ok()) {
                        timeout_ms = value;
                    }
                }
                "--max-bytes" => {
                    if let Some(value) = args.next().and_then(|value| value.parse().ok()) {
                        max_bytes = value;
                    }
                }
                _ => {}
            }
        }
        Ok(Self {
            root,
            out,
            output_root,
            limit,
            per_ext,
            extensions,
            tools,
            timeout_ms,
            max_bytes,
        })
    }
}

fn config_path_from_args(args: &[String]) -> (PathBuf, bool) {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--config" {
            return iter
                .next()
                .map(|value| (PathBuf::from(value), true))
                .unwrap_or_else(|| {
                    (
                        PathBuf::from("evaluation/atom-evaluation.config.toml"),
                        true,
                    )
                });
        }
    }
    (
        PathBuf::from("evaluation/atom-evaluation.config.toml"),
        false,
    )
}

fn load_eval_config(path: &Path) -> io::Result<EvalConfig> {
    let text = fs::read_to_string(path)?;
    let mut config = EvalConfig::default();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match key.trim() {
            "evaluation_root" => config.root = Some(PathBuf::from(value)),
            "evaluation_output_root" => config.output_root = Some(PathBuf::from(value)),
            "evaluation_report_path" => config.out = Some(PathBuf::from(value)),
            _ => {}
        }
    }
    Ok(config)
}

struct ToolResult {
    tool: String,
    status: String,
    elapsed_ms: u128,
    output_path: Option<PathBuf>,
    error: Option<String>,
    metrics: MarkdownMetrics,
}

#[derive(Default)]
struct MarkdownMetrics {
    bytes: usize,
    headings: usize,
    tables: usize,
    images: usize,
    code_blocks: usize,
    list_items: usize,
    score: f64,
}

struct CaseResult {
    input: PathBuf,
    extension: String,
    results: Vec<ToolResult>,
    winner: Option<String>,
    judgment: String,
}

type StatusCounts = BTreeMap<String, usize>;
type ToolStatusCounts = BTreeMap<String, StatusCounts>;
type ExtensionToolStatusCounts = BTreeMap<String, ToolStatusCounts>;
type FailureReasonCounts = BTreeMap<String, StatusCounts>;

struct Summary {
    total_files: usize,
    by_extension: BTreeMap<String, usize>,
    tool_success: BTreeMap<String, usize>,
    tool_average_score: BTreeMap<String, f64>,
    status_by_extension: ExtensionToolStatusCounts,
    status_by_tool: ToolStatusCounts,
    failure_reasons: FailureReasonCounts,
    atom_wins: usize,
    superiority_claim: String,
}

fn select_files(
    root: &Path,
    limit: usize,
    per_ext: usize,
    extensions: &Option<Vec<String>>,
) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();
    let mut selected = Vec::new();
    let mut counts = HashMap::<String, usize>::new();
    for file in files {
        let Some(extension) = extension(&file) else {
            continue;
        };
        if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }
        if let Some(extensions) = extensions
            && !extensions.contains(&extension)
        {
            continue;
        }
        let count = counts.entry(extension).or_default();
        if *count >= per_ext {
            continue;
        }
        selected.push(file);
        *count += 1;
        if selected.len() >= limit {
            break;
        }
    }
    Ok(selected)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn evaluate_file(
    input: &Path,
    output_root: &Path,
    tools: &[String],
    max_bytes: u64,
    timeout_ms: u64,
) -> io::Result<CaseResult> {
    let extension = extension(input).unwrap_or_else(|| "unknown".to_string());
    if fs::metadata(input)?.len() > max_bytes {
        let mut results = vec![skipped_tool_result(
            "atom",
            "too_large",
            "file exceeds evaluator max bytes",
        )];
        results.extend(
            tools
                .iter()
                .filter(|tool| tool.as_str() != "atom")
                .map(|tool| {
                    skipped_tool_result(tool, "too_large", "file exceeds evaluator max bytes")
                }),
        );
        return Ok(CaseResult {
            input: input.to_path_buf(),
            extension,
            results,
            winner: None,
            judgment: "excluded: too_large".to_string(),
        });
    }
    let mut results = vec![run_atom(input, output_root)?];
    for tool in tools {
        if tool == "atom" {
            continue;
        }
        results.push(run_external_tool(tool, input, output_root, timeout_ms));
    }
    let winner = results
        .iter()
        .filter(|result| result.status == "ok")
        .max_by(|left, right| {
            left.metrics
                .score
                .partial_cmp(&right.metrics.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|result| result.tool.clone());
    let judgment = if results
        .iter()
        .filter(|result| result.status == "ok")
        .count()
        < 2
    {
        "not_proven: fewer than two tools succeeded".to_string()
    } else if winner.as_deref() == Some("atom") {
        "atom_best_by_heuristic_metrics".to_string()
    } else {
        "baseline_best_or_tied_by_heuristic_metrics".to_string()
    };
    Ok(CaseResult {
        input: input.to_path_buf(),
        extension,
        results,
        winner,
        judgment,
    })
}

fn skipped_tool_result(tool: &str, status: &str, error: &str) -> ToolResult {
    ToolResult {
        tool: tool.to_string(),
        status: status.to_string(),
        elapsed_ms: 0,
        output_path: None,
        error: Some(error.to_string()),
        metrics: MarkdownMetrics::default(),
    }
}

fn run_atom(input: &Path, output_root: &Path) -> io::Result<ToolResult> {
    let started = Instant::now();
    let output_path = output_path(output_root, "atom", input);
    let result = Converter::new()
        .with_options(ConversionOptions {
            flavor: Flavor::Gfm,
            ..ConversionOptions::default()
        })
        .convert_file(input);
    match result {
        Ok(result) => {
            write_output(&output_path, &result.markdown)?;
            let unsupported = result.markdown.contains("Unsupported input format:");
            Ok(ToolResult {
                tool: "atom".to_string(),
                status: if unsupported { "unsupported" } else { "ok" }.to_string(),
                elapsed_ms: started.elapsed().as_millis(),
                output_path: Some(output_path),
                error: unsupported.then(|| "format is not supported by atom pipeline".to_string()),
                metrics: if unsupported {
                    MarkdownMetrics::default()
                } else {
                    markdown_metrics(&result.markdown)
                },
            })
        }
        Err(error) => Ok(ToolResult {
            tool: "atom".to_string(),
            status: "error".to_string(),
            elapsed_ms: started.elapsed().as_millis(),
            output_path: None,
            error: Some(error.to_string()),
            metrics: MarkdownMetrics::default(),
        }),
    }
}

fn run_external_tool(tool: &str, input: &Path, output_root: &Path, timeout_ms: u64) -> ToolResult {
    let started = Instant::now();
    if timeout_ms == 0 {
        return ToolResult {
            tool: tool.to_string(),
            status: "timeout".to_string(),
            elapsed_ms: 0,
            output_path: None,
            error: Some("external tool timed out before execution".to_string()),
            metrics: MarkdownMetrics::default(),
        };
    }
    if !command_exists("docker") {
        return ToolResult {
            tool: tool.to_string(),
            status: "missing".to_string(),
            elapsed_ms: 0,
            output_path: None,
            error: Some("docker not found in PATH".to_string()),
            metrics: MarkdownMetrics::default(),
        };
    }
    let output_path = output_path(output_root, tool, input);
    let report_path = sidecar_report_path(&output_path);
    let output = run_external_tool_in_docker(tool, input, &output_path, &report_path);
    match output {
        Ok(output) if output.status.success() => match fs::read_to_string(&output_path) {
            Ok(markdown) => ToolResult {
                tool: tool.to_string(),
                status: "ok".to_string(),
                elapsed_ms: started.elapsed().as_millis(),
                output_path: Some(output_path),
                error: None,
                metrics: markdown_metrics(&markdown),
            },
            Err(error) => ToolResult {
                tool: tool.to_string(),
                status: "error".to_string(),
                elapsed_ms: started.elapsed().as_millis(),
                output_path: None,
                error: Some(format!("runner did not write markdown output: {error}")),
                metrics: MarkdownMetrics::default(),
            },
        },
        Ok(output) => ToolResult {
            tool: tool.to_string(),
            status: "error".to_string(),
            elapsed_ms: started.elapsed().as_millis(),
            output_path: None,
            error: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
            metrics: MarkdownMetrics::default(),
        },
        Err(error) => ToolResult {
            tool: tool.to_string(),
            status: "error".to_string(),
            elapsed_ms: started.elapsed().as_millis(),
            output_path: None,
            error: Some(error.to_string()),
            metrics: MarkdownMetrics::default(),
        },
    }
}

fn run_external_tool_in_docker(
    tool: &str,
    input: &Path,
    output_path: &Path,
    report_path: &Path,
) -> io::Result<std::process::Output> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let file_name = input.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "input path has no file name")
    })?;
    let output_dir = output_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output path has no parent"))?;
    fs::create_dir_all(output_dir)?;
    let input_mount_path = fs::canonicalize(parent)?;
    let output_mount_path = fs::canonicalize(output_dir)?;
    let mount = format!("{}:/input:ro", input_mount_path.display());
    let output_mount = format!("{}:/output", output_mount_path.display());
    let image = docker_image(tool);
    let mut command = Command::new("docker");
    command
        .arg("run")
        .arg("--rm")
        .arg("--network")
        .arg("none")
        .arg("-v")
        .arg(mount)
        .arg("-v")
        .arg(output_mount)
        .arg("-w")
        .arg("/input")
        .arg(image);
    command
        .arg(Path::new("/input").join(file_name))
        .arg(Path::new("/output").join(output_path.file_name().unwrap()))
        .arg(Path::new("/output").join(report_path.file_name().unwrap()));
    command.output()
}

fn docker_image(tool: &str) -> String {
    match tool {
        "pandoc" => env::var("ATOM_EVAL_PANDOC_IMAGE")
            .unwrap_or_else(|_| "atom-eval-pandoc:latest".to_string()),
        "markitdown" => env::var("ATOM_EVAL_MARKITDOWN_IMAGE")
            .unwrap_or_else(|_| "atom-eval-markitdown:latest".to_string()),
        "docling" => env::var("ATOM_EVAL_DOCLING_IMAGE")
            .unwrap_or_else(|_| "atom-eval-docling:latest".to_string()),
        "pymupdf4llm" => env::var("ATOM_EVAL_PYMUPDF4LLM_IMAGE")
            .unwrap_or_else(|_| "atom-eval-pymupdf4llm:latest".to_string()),
        "mammoth-js" => env::var("ATOM_EVAL_MAMMOTH_JS_IMAGE")
            .unwrap_or_else(|_| "atom-eval-mammoth-js:latest".to_string()),
        _ => tool.to_string(),
    }
}

fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn markdown_metrics(markdown: &str) -> MarkdownMetrics {
    let mut metrics = MarkdownMetrics {
        bytes: markdown.len(),
        ..MarkdownMetrics::default()
    };
    let mut in_code = false;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if !in_code {
                metrics.code_blocks += 1;
            }
            in_code = !in_code;
        } else if in_code {
            continue;
        } else if trimmed.starts_with('#') {
            metrics.headings += 1;
        } else if trimmed.starts_with('|') {
            metrics.tables += 1;
        } else if trimmed.starts_with("![") || trimmed.contains("<img ") {
            metrics.images += 1;
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            metrics.list_items += 1;
        }
    }
    metrics.score = if metrics.bytes == 0 {
        0.0
    } else {
        1.0 + metrics.headings as f64 * 2.0
            + metrics.tables as f64 * 2.0
            + metrics.images as f64 * 2.0
            + metrics.code_blocks as f64
            + metrics.list_items as f64 * 0.5
            + (metrics.bytes as f64).log10().min(6.0)
    };
    metrics
}

fn summarize(cases: &[CaseResult]) -> Summary {
    let mut by_extension = BTreeMap::new();
    let mut tool_success = BTreeMap::new();
    let mut tool_scores = BTreeMap::<String, (f64, usize)>::new();
    let mut status_by_extension = ExtensionToolStatusCounts::new();
    let mut status_by_tool = ToolStatusCounts::new();
    let mut failure_reasons = FailureReasonCounts::new();
    let mut atom_wins = 0;
    for case in cases {
        *by_extension.entry(case.extension.clone()).or_insert(0) += 1;
        if case.winner.as_deref() == Some("atom") {
            atom_wins += 1;
        }
        for result in &case.results {
            *status_by_extension
                .entry(case.extension.clone())
                .or_default()
                .entry(result.tool.clone())
                .or_default()
                .entry(result.status.clone())
                .or_default() += 1;
            *status_by_tool
                .entry(result.tool.clone())
                .or_default()
                .entry(result.status.clone())
                .or_default() += 1;
            if result.status == "ok" {
                *tool_success.entry(result.tool.clone()).or_insert(0) += 1;
                let entry = tool_scores.entry(result.tool.clone()).or_default();
                entry.0 += result.metrics.score;
                entry.1 += 1;
            } else {
                let reason = result
                    .error
                    .as_deref()
                    .filter(|error| !error.is_empty())
                    .unwrap_or(&result.status);
                *failure_reasons
                    .entry(result.tool.clone())
                    .or_default()
                    .entry(reason.to_string())
                    .or_default() += 1;
            }
        }
    }
    let tool_average_score = tool_scores
        .into_iter()
        .map(|(tool, (score, count))| (tool, score / count as f64))
        .collect();
    Summary {
        total_files: cases.len(),
        by_extension,
        tool_success,
        tool_average_score,
        status_by_extension,
        status_by_tool,
        failure_reasons,
        atom_wins,
        superiority_claim: "not_proven_without_human_review_or_ground_truth".to_string(),
    }
}

fn render_report(root: &Path, summary: &Summary, cases: &[CaseResult]) -> String {
    format!(
        concat!(
            "{{",
            "\"root\":\"{}\",",
            "\"summary\":{},",
            "\"cases\":[{}]",
            "}}\n"
        ),
        escape_json(&root.to_string_lossy()),
        render_summary(summary),
        cases.iter().map(render_case).collect::<Vec<_>>().join(",")
    )
}

fn render_review_index(summary: &Summary, cases: &[CaseResult]) -> String {
    let mut output = String::new();
    output.push_str("# Corpus Evaluation Review Index\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Total files: {}\n", summary.total_files));
    output.push_str(&format!("- atom wins: {}\n", summary.atom_wins));
    output.push_str(&format!(
        "- Superiority claim: `{}`\n\n",
        summary.superiority_claim
    ));
    output.push_str("## Cases\n\n");
    output.push_str("| Input | Winner | Judgment | Outputs |\n");
    output.push_str("| ---- | ---- | ---- | ---- |\n");
    for case in cases {
        let outputs = case
            .results
            .iter()
            .filter_map(|result| {
                result.output_path.as_ref().map(|path| {
                    format!("{}: `{}` ({})", result.tool, path.display(), result.status)
                })
            })
            .collect::<Vec<_>>()
            .join("<br>");
        output.push_str(&format!(
            "| `{}` | {} | `{}` | {} |\n",
            case.input.display(),
            case.winner.as_deref().unwrap_or("-"),
            case.judgment,
            outputs
        ));
    }
    output
}

fn render_summary(summary: &Summary) -> String {
    format!(
        concat!(
            "{{",
            "\"total_files\":{},",
            "\"by_extension\":{},",
            "\"tool_success\":{},",
            "\"tool_average_score\":{},",
            "\"status_by_extension\":{},",
            "\"status_by_tool\":{},",
            "\"failure_reasons\":{},",
            "\"atom_wins\":{},",
            "\"superiority_claim\":\"{}\"",
            "}}"
        ),
        summary.total_files,
        render_usize_map(&summary.by_extension),
        render_usize_map(&summary.tool_success),
        render_f64_map(&summary.tool_average_score),
        render_extension_tool_status_counts(&summary.status_by_extension),
        render_tool_status_counts(&summary.status_by_tool),
        render_tool_status_counts(&summary.failure_reasons),
        summary.atom_wins,
        escape_json(&summary.superiority_claim)
    )
}

fn render_case(case: &CaseResult) -> String {
    format!(
        concat!(
            "{{",
            "\"input\":\"{}\",",
            "\"extension\":\"{}\",",
            "\"winner\":{},",
            "\"judgment\":\"{}\",",
            "\"results\":[{}]",
            "}}"
        ),
        escape_json(&case.input.to_string_lossy()),
        escape_json(&case.extension),
        json_option(case.winner.as_deref()),
        escape_json(&case.judgment),
        case.results
            .iter()
            .map(render_tool_result)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_tool_result(result: &ToolResult) -> String {
    format!(
        concat!(
            "{{",
            "\"tool\":\"{}\",",
            "\"status\":\"{}\",",
            "\"elapsed_ms\":{},",
            "\"output_path\":{},",
            "\"error\":{},",
            "\"metrics\":{}",
            "}}"
        ),
        escape_json(&result.tool),
        escape_json(&result.status),
        result.elapsed_ms,
        json_option(
            result
                .output_path
                .as_ref()
                .map(|path| path.to_string_lossy())
                .as_deref()
        ),
        json_option(result.error.as_deref()),
        render_metrics(&result.metrics)
    )
}

fn render_metrics(metrics: &MarkdownMetrics) -> String {
    format!(
        concat!(
            "{{",
            "\"bytes\":{},",
            "\"headings\":{},",
            "\"tables\":{},",
            "\"images\":{},",
            "\"code_blocks\":{},",
            "\"list_items\":{},",
            "\"score\":{}",
            "}}"
        ),
        metrics.bytes,
        metrics.headings,
        metrics.tables,
        metrics.images,
        metrics.code_blocks,
        metrics.list_items,
        metrics.score
    )
}

fn render_usize_map(values: &BTreeMap<String, usize>) -> String {
    format!(
        "{{{}}}",
        values
            .iter()
            .map(|(key, value)| format!("\"{}\":{}", escape_json(key), value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_tool_status_counts(values: &ToolStatusCounts) -> String {
    format!(
        "{{{}}}",
        values
            .iter()
            .map(|(tool, statuses)| {
                format!("\"{}\":{}", escape_json(tool), render_usize_map(statuses))
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_extension_tool_status_counts(values: &ExtensionToolStatusCounts) -> String {
    format!(
        "{{{}}}",
        values
            .iter()
            .map(|(extension, tools)| {
                format!(
                    "\"{}\":{}",
                    escape_json(extension),
                    render_tool_status_counts(tools)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_f64_map(values: &BTreeMap<String, f64>) -> String {
    format!(
        "{{{}}}",
        values
            .iter()
            .map(|(key, value)| format!("\"{}\":{}", escape_json(key), value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn output_path(output_root: &Path, tool: &str, input: &Path) -> PathBuf {
    output_root
        .join(tool)
        .join(format!("{}.md", safe_name(&input.to_string_lossy())))
}

fn sidecar_report_path(output_path: &Path) -> PathBuf {
    output_path.with_extension("report.json")
}

fn write_output(path: &Path, markdown: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, markdown)
}

fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
}

fn json_option(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
