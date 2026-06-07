use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

pub const SCHEMA_VERSION: &str = "atom.llm-eval.v1";

#[derive(Debug)]
pub struct LlmEvalOptions {
    pub report_path: PathBuf,
    pub output_path: PathBuf,
    pub ollama_url: String,
    pub model: String,
    pub limit: usize,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct ReviewCandidate {
    pub input: String,
    pub priority: String,
    pub reasons: Vec<String>,
    pub output_paths: Vec<(String, PathBuf)>,
}

#[derive(Debug)]
pub struct EvaluationRequest {
    pub input: String,
    pub priority: String,
    pub reasons: Vec<String>,
    pub prompt_kind: String,
    pub prompt: String,
}

#[derive(Debug)]
pub struct EvaluationResult {
    pub request: EvaluationRequest,
    pub model: String,
    pub response: Option<String>,
    pub error: Option<String>,
}

pub fn run(options: LlmEvalOptions) -> io::Result<()> {
    let report = fs::read_to_string(&options.report_path)?;
    let mut candidates = parse_review_candidates(&report);
    if options.limit > 0 {
        candidates.truncate(options.limit);
    }
    let mut lines = Vec::new();
    for candidate in candidates {
        let request = build_request(candidate)?;
        if options.dry_run {
            lines.push(render_request_json(&request, &options.model));
        } else {
            let response = match call_ollama(&options.ollama_url, &options.model, &request.prompt) {
                Ok(response) => EvaluationResult {
                    request,
                    model: options.model.clone(),
                    response: Some(response),
                    error: None,
                },
                Err(error) => EvaluationResult {
                    request,
                    model: options.model.clone(),
                    response: None,
                    error: Some(error.to_string()),
                },
            };
            lines.push(render_result_json(&response));
        }
    }
    if let Some(parent) = options.output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&options.output_path, format!("{}\n", lines.join("\n")))
}

pub fn parse_review_candidates(report: &str) -> Vec<ReviewCandidate> {
    let Some(array) = extract_array_after_key(report, "\"review_candidates\"") else {
        return Vec::new();
    };
    split_top_level_objects(&array)
        .into_iter()
        .filter_map(|object| {
            let input = extract_string_value(&object, "input")?;
            let priority =
                extract_string_value(&object, "priority").unwrap_or_else(|| "medium".to_string());
            let reasons = extract_string_array(&object, "reasons");
            let output_paths = parse_output_paths(&object);
            Some(ReviewCandidate {
                input,
                priority,
                reasons,
                output_paths,
            })
        })
        .collect()
}

pub fn build_request(candidate: ReviewCandidate) -> io::Result<EvaluationRequest> {
    let markdowns = candidate
        .output_paths
        .iter()
        .filter_map(|(tool, path)| {
            fs::read_to_string(path)
                .ok()
                .map(|markdown| (tool.as_str(), markdown))
        })
        .collect::<Vec<_>>();
    let (prompt_kind, prompt) = if markdowns.len() >= 2 {
        (
            "pair_comparison".to_string(),
            pair_comparison_prompt(&candidate.input, &candidate.reasons, &markdowns),
        )
    } else {
        (
            "single_markdown_score".to_string(),
            single_markdown_score_prompt(&candidate.input, &candidate.reasons, &markdowns),
        )
    };
    Ok(EvaluationRequest {
        input: candidate.input,
        priority: candidate.priority,
        reasons: candidate.reasons,
        prompt_kind,
        prompt,
    })
}

pub fn single_markdown_score_prompt(
    input: &str,
    reasons: &[String],
    markdowns: &[(&str, String)],
) -> String {
    let mut prompt = base_prompt(input, reasons);
    prompt.push_str(
        "\nEvaluate the Markdown output on its own. Score each rubric item from 0 to 2 and return JSON only.\n",
    );
    append_markdowns(&mut prompt, markdowns);
    prompt
}

pub fn pair_comparison_prompt(
    input: &str,
    reasons: &[String],
    markdowns: &[(&str, String)],
) -> String {
    let mut prompt = base_prompt(input, reasons);
    prompt.push_str(
        "\nCompare the Markdown outputs. Pick the best tool only when evidence is clear. Return JSON only.\n",
    );
    append_markdowns(&mut prompt, markdowns);
    prompt
}

fn base_prompt(input: &str, reasons: &[String]) -> String {
    format!(
        concat!(
            "You are evaluating document-to-Markdown quality for atom.\n",
            "Input path: {}\n",
            "Review reasons: {}\n",
            "Rubric items: title_heading, text_paragraph, list, table, figure_image, ",
            "caption, footnote, formula_value, reading_order, warnings.\n",
            "Return this JSON shape: {{",
            "\"schema_version\":\"{}\",",
            "\"scores\":{{\"title_heading\":0,\"text_paragraph\":0,\"list\":0,",
            "\"table\":0,\"figure_image\":0,\"caption\":0,\"footnote\":0,",
            "\"formula_value\":0,\"reading_order\":0,\"warnings\":0}},",
            "\"winner\":null,\"confidence\":\"low|medium|high\",",
            "\"findings\":[\"short evidence-based finding\"],",
            "\"fixture_candidates\":[\"minimal reproducible issue\"]",
            "}}.\n"
        ),
        input,
        reasons.join(","),
        SCHEMA_VERSION
    )
}

fn append_markdowns(prompt: &mut String, markdowns: &[(&str, String)]) {
    if markdowns.is_empty() {
        prompt.push_str("\nNo Markdown output was available.\n");
        return;
    }
    for (tool, markdown) in markdowns {
        prompt.push_str(&format!(
            "\n--- BEGIN MARKDOWN tool={} ---\n{}\n--- END MARKDOWN tool={} ---\n",
            tool,
            truncate_markdown(markdown),
            tool
        ));
    }
}

fn truncate_markdown(markdown: &str) -> String {
    const MAX_CHARS: usize = 30_000;
    if markdown.chars().count() <= MAX_CHARS {
        return markdown.to_string();
    }
    let mut truncated = markdown.chars().take(MAX_CHARS).collect::<String>();
    truncated.push_str("\n\n[truncated by atom-llm-eval]\n");
    truncated
}

fn call_ollama(ollama_url: &str, model: &str, prompt: &str) -> io::Result<String> {
    let (host, port) = parse_local_http_url(ollama_url)?;
    let body = format!(
        concat!(
            "{{",
            "\"model\":\"{}\",",
            "\"stream\":false,",
            "\"messages\":[{{\"role\":\"user\",\"content\":\"{}\"}}]",
            "}}"
        ),
        escape_json(model),
        escape_json(prompt)
    );
    let request = format!(
        concat!(
            "POST /api/chat HTTP/1.1\r\n",
            "Host: {}:{}\r\n",
            "Content-Type: application/json\r\n",
            "Content-Length: {}\r\n",
            "Connection: close\r\n",
            "\r\n",
            "{}"
        ),
        host,
        port,
        body.len(),
        body
    );
    let mut stream = TcpStream::connect((host.as_str(), port))?;
    stream.set_read_timeout(Some(Duration::from_secs(300)))?;
    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
        return Err(io::Error::other(first_response_line(&response)));
    }
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or_default();
    extract_nested_message_content(body)
        .ok_or_else(|| io::Error::other("Ollama response did not contain message.content"))
}

fn parse_local_http_url(url: &str) -> io::Result<(String, u16)> {
    let stripped = url.strip_prefix("http://").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "only local http:// Ollama URLs are supported",
        )
    })?;
    let host_port = stripped
        .trim_end_matches('/')
        .split('/')
        .next()
        .unwrap_or(stripped);
    let (host, port) = host_port.split_once(':').unwrap_or((host_port, "11434"));
    if host != "127.0.0.1" && host != "localhost" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "atom-llm-eval only sends prompts to localhost Ollama",
        ));
    }
    Ok((host.to_string(), port.parse().unwrap_or(11434)))
}

fn first_response_line(response: &str) -> String {
    response
        .lines()
        .next()
        .unwrap_or("empty HTTP response")
        .to_string()
}

fn extract_nested_message_content(body: &str) -> Option<String> {
    let message_index = body.find("\"message\"")?;
    extract_string_value(&body[message_index..], "content")
}

fn extract_array_after_key(input: &str, key: &str) -> Option<String> {
    let start = input.find(key)?;
    let array_start = input[start..].find('[')? + start;
    extract_balanced(input, array_start, '[', ']')
}

fn extract_balanced(input: &str, start: usize, open: char, close: char) -> Option<String> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in input[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        if character == '"' {
            in_string = true;
        } else if character == open {
            depth += 1;
        } else if character == close {
            depth -= 1;
            if depth == 0 {
                return Some(input[start + 1..start + index].to_string());
            }
        }
    }
    None
}

fn split_top_level_objects(input: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut object_start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        if character == '"' {
            in_string = true;
        } else if character == '{' {
            if depth == 0 {
                object_start = Some(index);
            }
            depth += 1;
        } else if character == '}' {
            depth -= 1;
            if depth == 0
                && let Some(start) = object_start.take()
            {
                objects.push(input[start..=index].to_string());
            }
        }
    }
    objects
}

fn parse_output_paths(object: &str) -> Vec<(String, PathBuf)> {
    let Some(array) = extract_array_after_key(object, "\"output_paths\"") else {
        return Vec::new();
    };
    split_top_level_objects(&array)
        .into_iter()
        .filter_map(|entry| {
            Some((
                extract_string_value(&entry, "tool")?,
                PathBuf::from(extract_string_value(&entry, "path")?),
            ))
        })
        .collect()
}

fn extract_string_array(object: &str, key: &str) -> Vec<String> {
    let Some(array) = extract_array_after_key(object, &format!("\"{key}\"")) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut index = 0usize;
    while let Some(start) = array[index..].find('"') {
        let start = index + start + 1;
        if let Some((value, end)) = read_json_string(&array, start) {
            values.push(value);
            index = end;
        } else {
            break;
        }
    }
    values
}

fn extract_string_value(object: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = object.find(&marker)? + marker.len();
    read_json_string(object, start).map(|(value, _)| value)
}

fn read_json_string(input: &str, start: usize) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (offset, character) in input[start..].char_indices() {
        if escaped {
            value.push(match character {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some((value, start + offset + 1));
        } else {
            value.push(character);
        }
    }
    None
}

fn render_request_json(request: &EvaluationRequest, model: &str) -> String {
    format!(
        concat!(
            "{{",
            "\"schema_version\":\"{}\",",
            "\"model\":\"{}\",",
            "\"input\":\"{}\",",
            "\"priority\":\"{}\",",
            "\"reasons\":[{}],",
            "\"prompt_kind\":\"{}\",",
            "\"prompt\":\"{}\"",
            "}}"
        ),
        SCHEMA_VERSION,
        escape_json(model),
        escape_json(&request.input),
        escape_json(&request.priority),
        render_string_array(&request.reasons),
        escape_json(&request.prompt_kind),
        escape_json(&request.prompt)
    )
}

fn render_result_json(result: &EvaluationResult) -> String {
    format!(
        concat!(
            "{{",
            "\"schema_version\":\"{}\",",
            "\"model\":\"{}\",",
            "\"input\":\"{}\",",
            "\"priority\":\"{}\",",
            "\"reasons\":[{}],",
            "\"prompt_kind\":\"{}\",",
            "\"response\":{},",
            "\"error\":{}",
            "}}"
        ),
        SCHEMA_VERSION,
        escape_json(&result.model),
        escape_json(&result.request.input),
        escape_json(&result.request.priority),
        render_string_array(&result.request.reasons),
        escape_json(&result.request.prompt_kind),
        json_option(result.response.as_deref()),
        json_option(result.error.as_deref())
    )
}

fn render_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_review_candidates_from_report_summary() {
        let report = r#"{"summary":{"review_candidates":[{"input":"a.pdf","priority":"high","reasons":["atom_failed_but_baseline_succeeded"],"output_paths":[{"tool":"atom","path":"out/atom/a.md"},{"tool":"markitdown","path":"out/markitdown/a.md"}]}]}}"#;

        let candidates = parse_review_candidates(report);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].input, "a.pdf");
        assert_eq!(candidates[0].priority, "high");
        assert_eq!(candidates[0].output_paths.len(), 2);
    }

    #[test]
    fn rejects_non_local_ollama_url() {
        let error = parse_local_http_url("http://example.com:11434").unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
