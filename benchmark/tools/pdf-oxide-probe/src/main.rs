use std::env;
use std::path::PathBuf;
use std::time::Instant;

use pdf_oxide::converters::ConversionOptions;
use pdf_oxide::PdfDocument;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let (mode, path) = match args.as_slice() {
        [path] => ("extract-text".to_string(), PathBuf::from(path)),
        [mode, path] => (mode.to_string(), PathBuf::from(path)),
        _ => panic!("usage: pdf-oxide-probe [extract-text|plain-text|markdown|form-words] <pdf>"),
    };

    let started = Instant::now();
    let result = run(&mode, &path);
    let elapsed_ms = started.elapsed().as_millis();
    match result {
        Ok((pages, chars)) => {
            println!(
                "path\tstatus\telapsed_ms\tpages\tchars\terror\n{}\tok\t{}\t{}\t{}\t",
                path.display(),
                elapsed_ms,
                pages,
                chars
            );
        }
        Err(error) => {
            println!(
                "path\tstatus\telapsed_ms\tpages\tchars\terror\n{}\terror\t{}\t0\t0\t{}",
                path.display(),
                elapsed_ms,
                error.replace('\t', "\\t").replace('\n', "\\n")
            );
        }
    }
}

fn run(mode: &str, path: &PathBuf) -> Result<(usize, usize), String> {
    let doc = PdfDocument::open(path).map_err(|error| error.to_string())?;
    let pages = doc.page_count().map_err(|error| error.to_string())?;
    let text = match mode {
        "extract-text" => {
            let mut text = String::new();
            for page in 0..pages {
                if page > 0 {
                    text.push('\n');
                }
                text.push_str(&doc.extract_text(page).map_err(|error| error.to_string())?);
            }
            text
        }
        "plain-text" => doc
            .to_plain_text_all(&ConversionOptions::default())
            .map_err(|error| error.to_string())?,
        "markdown" => doc
            .to_markdown_all(&ConversionOptions::default())
            .map_err(|error| error.to_string())?,
        "form-words" => form_words_all(&doc, pages).map_err(|error| error.to_string())?,
        other => return Err(format!("unknown mode: {other}")),
    };
    Ok((pages, text.chars().count()))
}

#[derive(Clone)]
struct ProbeWord {
    text: String,
    x0: f32,
    x1: f32,
    top: f32,
}

fn form_words_all(doc: &PdfDocument, pages: usize) -> pdf_oxide::error::Result<String> {
    let mut chunks = Vec::new();
    for page_index in 0..pages {
        let words = doc
            .extract_words(page_index)?
            .into_iter()
            .map(|word| ProbeWord {
                text: word.text,
                x0: word.bbox.x,
                x1: word.bbox.x + word.bbox.width,
                top: word.bbox.y,
            })
            .collect::<Vec<_>>();
        if let Some(content) = extract_form_content_from_words(words) {
            if !content.trim().is_empty() {
                chunks.push(content);
            }
        } else {
            chunks.push(doc.extract_text(page_index)?);
        }
    }
    Ok(chunks.join("\n\n"))
}

fn extract_form_content_from_words(words: Vec<ProbeWord>) -> Option<String> {
    if words.is_empty() {
        return None;
    }

    let y_tolerance = 5.0_f32;
    let mut rows_by_y = std::collections::BTreeMap::<i32, Vec<ProbeWord>>::new();
    for word in words {
        let y_key = (word.top / y_tolerance).round() as i32 * y_tolerance as i32;
        rows_by_y.entry(y_key).or_default().push(word);
    }

    let page_width = rows_by_y
        .values()
        .flat_map(|row| row.iter().map(|word| word.x1))
        .fold(612.0_f32, f32::max);

    let mut row_info = Vec::new();
    for (y_key, mut row_words) in rows_by_y {
        row_words.sort_by(|a, b| a.x0.total_cmp(&b.x0));
        if row_words.is_empty() {
            continue;
        }
        let first_x0 = row_words[0].x0;
        let last_x1 = row_words[row_words.len() - 1].x1;
        let line_width = last_x1 - first_x0;
        let combined_text = row_words
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let mut x_groups = Vec::<f32>::new();
        for x in row_words.iter().map(|word| word.x0) {
            if x_groups.last().is_none_or(|last| x - *last > 50.0) {
                x_groups.push(x);
            }
        }
        let is_paragraph = line_width > page_width * 0.55 && combined_text.len() > 60;
        let has_partial_numbering = row_words
            .first()
            .is_some_and(|word| is_partial_numbering(word.text.trim()));

        row_info.push(RowInfo {
            y_key,
            words: row_words,
            text: combined_text,
            x_groups,
            is_paragraph,
            has_partial_numbering,
            is_table_row: false,
        });
    }

    let mut all_table_x_positions = Vec::new();
    for info in &row_info {
        if info.x_groups.len() >= 3 && !info.is_paragraph {
            all_table_x_positions.extend(info.x_groups.iter().copied());
        }
    }
    if all_table_x_positions.is_empty() {
        return None;
    }
    all_table_x_positions.sort_by(f32::total_cmp);

    let gaps = all_table_x_positions
        .windows(2)
        .filter_map(|pair| {
            let gap = pair[1] - pair[0];
            (gap > 5.0).then_some(gap)
        })
        .collect::<Vec<_>>();
    let adaptive_tolerance = if gaps.len() >= 3 {
        let mut sorted_gaps = gaps;
        sorted_gaps.sort_by(f32::total_cmp);
        sorted_gaps[(sorted_gaps.len() as f32 * 0.70) as usize].clamp(25.0, 50.0)
    } else {
        35.0
    };

    let mut global_columns = Vec::<f32>::new();
    for x in all_table_x_positions {
        if global_columns
            .last()
            .is_none_or(|last| x - *last > adaptive_tolerance)
        {
            global_columns.push(x);
        }
    }
    if global_columns.len() <= 1 {
        return None;
    }
    let content_width = global_columns[global_columns.len() - 1] - global_columns[0];
    let avg_col_width = content_width / global_columns.len() as f32;
    if avg_col_width < 30.0 {
        return None;
    }
    let columns_per_inch = global_columns.len() as f32 / (content_width / 72.0);
    if columns_per_inch > 10.0 {
        return None;
    }
    let adaptive_max_columns = (20.0 * (page_width / 612.0)).max(15.0) as usize;
    if global_columns.len() > adaptive_max_columns {
        return None;
    }

    for info in &mut row_info {
        if info.is_paragraph || info.has_partial_numbering {
            info.is_table_row = false;
            continue;
        }
        let mut aligned_columns = std::collections::BTreeSet::new();
        for word in &info.words {
            for (col_idx, col_x) in global_columns.iter().enumerate() {
                if (word.x0 - *col_x).abs() < 40.0 {
                    aligned_columns.insert(col_idx);
                    break;
                }
            }
        }
        info.is_table_row = aligned_columns.len() >= 2;
    }

    let mut table_regions = Vec::<(usize, usize)>::new();
    let mut idx = 0;
    while idx < row_info.len() {
        if row_info[idx].is_table_row {
            let start = idx;
            while idx < row_info.len() && row_info[idx].is_table_row {
                idx += 1;
            }
            table_regions.push((start, idx));
        } else {
            idx += 1;
        }
    }
    let total_table_rows = table_regions
        .iter()
        .map(|(start, end)| end - start)
        .sum::<usize>();
    if !row_info.is_empty() && total_table_rows as f32 / (row_info.len() as f32) < 0.2 {
        return None;
    }

    let num_cols = global_columns.len();
    let mut result_lines = Vec::new();
    let mut idx = 0;
    while idx < row_info.len() {
        let table_region = table_regions
            .iter()
            .find(|(start, _)| *start == idx)
            .copied();
        if let Some((start, end)) = table_region {
            let table_data = (start..end)
                .map(|table_idx| extract_cells(&row_info[table_idx], &global_columns))
                .collect::<Vec<_>>();
            let col_widths = (0..num_cols)
                .map(|col| {
                    table_data
                        .iter()
                        .map(|row| row[col].len())
                        .max()
                        .unwrap_or(0)
                        .max(3)
                })
                .collect::<Vec<_>>();
            if let Some(header) = table_data.first() {
                result_lines.push(format_table_row(header, &col_widths));
                result_lines.push(format!(
                    "| {} |",
                    col_widths
                        .iter()
                        .map(|width| "-".repeat(*width))
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
                for row in table_data.iter().skip(1) {
                    result_lines.push(format_table_row(row, &col_widths));
                }
            }
            idx = end;
        } else {
            result_lines.push(row_info[idx].text.clone());
            idx += 1;
        }
    }

    Some(result_lines.join("\n"))
}

struct RowInfo {
    #[allow(dead_code)]
    y_key: i32,
    words: Vec<ProbeWord>,
    text: String,
    x_groups: Vec<f32>,
    is_paragraph: bool,
    has_partial_numbering: bool,
    is_table_row: bool,
}

fn extract_cells(info: &RowInfo, global_columns: &[f32]) -> Vec<String> {
    let num_cols = global_columns.len();
    let mut cells = vec![String::new(); num_cols];
    for word in &info.words {
        let mut assigned_col = num_cols - 1;
        for col_idx in 0..num_cols - 1 {
            let col_end = global_columns[col_idx + 1];
            if word.x0 < col_end - 20.0 {
                assigned_col = col_idx;
                break;
            }
        }
        if !cells[assigned_col].is_empty() {
            cells[assigned_col].push(' ');
        }
        cells[assigned_col].push_str(&word.text);
    }
    cells
}

fn format_table_row(row: &[String], col_widths: &[usize]) -> String {
    format!(
        "| {} |",
        row.iter()
            .enumerate()
            .map(|(idx, cell)| format!("{cell:<width$}", width = col_widths[idx]))
            .collect::<Vec<_>>()
            .join(" | ")
    )
}

fn is_partial_numbering(text: &str) -> bool {
    let Some(rest) = text.strip_prefix('.') else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|character| character.is_ascii_digit())
}
