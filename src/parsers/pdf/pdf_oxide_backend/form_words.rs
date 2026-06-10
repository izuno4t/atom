#[derive(Clone)]
struct PdfOxideWord {
    text: String,
    x0: f32,
    x1: f32,
    top: f32,
}

struct PdfOxideRow {
    words: Vec<PdfOxideWord>,
    text: String,
    x_groups: Vec<f32>,
    is_paragraph: bool,
    has_partial_numbering: bool,
    is_table_row: bool,
}

pub(super) fn extract_pdf_oxide_form_words_page(
    document: &pdf_oxide::PdfDocument,
    page_index: usize,
) -> pdf_oxide::error::Result<Option<String>> {
    let words = document
        .extract_words(page_index)?
        .into_iter()
        .filter_map(|word| pdf_oxide_word(word.text, word.bbox.x, word.bbox.width, word.bbox.y))
        .collect::<Vec<_>>();
    Ok(extract_form_content_from_pdf_oxide_words(words))
}

fn pdf_oxide_word(text: String, x: f32, width: f32, y: f32) -> Option<PdfOxideWord> {
    let x1 = x + width;
    (x.is_finite() && x1.is_finite() && y.is_finite()).then_some(PdfOxideWord {
        text,
        x0: x,
        x1,
        top: y,
    })
}

fn extract_form_content_from_pdf_oxide_words(words: Vec<PdfOxideWord>) -> Option<String> {
    if words.is_empty() {
        return None;
    }

    let rows_by_y = group_words_by_y(words);
    let page_width = rows_by_y
        .values()
        .flat_map(|row| row.iter().map(|word| word.x1))
        .fold(612.0_f32, f32::max);
    let mut row_info = rows_by_y
        .into_values()
        .filter_map(|row_words| build_pdf_oxide_row(row_words, page_width))
        .collect::<Vec<_>>();
    let global_columns = infer_global_columns(&row_info, page_width)?;
    mark_table_rows(&mut row_info, &global_columns);
    let table_regions = table_regions(&row_info);
    if !has_enough_table_rows(&row_info, &table_regions) {
        return None;
    }
    Some(render_rows_with_tables(
        &row_info,
        &table_regions,
        &global_columns,
    ))
}

fn group_words_by_y(
    words: Vec<PdfOxideWord>,
) -> std::collections::BTreeMap<i32, Vec<PdfOxideWord>> {
    let y_tolerance = 5.0_f32;
    let mut rows_by_y = std::collections::BTreeMap::<i32, Vec<PdfOxideWord>>::new();
    for word in words {
        let y_key = (word.top / y_tolerance).round() as i32 * y_tolerance as i32;
        rows_by_y.entry(y_key).or_default().push(word);
    }
    rows_by_y
}

fn build_pdf_oxide_row(mut row_words: Vec<PdfOxideWord>, page_width: f32) -> Option<PdfOxideRow> {
    row_words.sort_by(|a, b| a.x0.total_cmp(&b.x0));
    if row_words.is_empty() {
        return None;
    }

    let first_x0 = row_words[0].x0;
    let last_x1 = row_words[row_words.len() - 1].x1;
    let line_width = last_x1 - first_x0;
    let combined_text = row_words
        .iter()
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let x_groups = row_x_groups(&row_words);
    Some(PdfOxideRow {
        has_partial_numbering: row_words
            .first()
            .is_some_and(|word| is_pdf_partial_numbering(word.text.trim())),
        words: row_words,
        text: combined_text.clone(),
        x_groups,
        is_paragraph: line_width > page_width * 0.55 && combined_text.len() > 60,
        is_table_row: false,
    })
}

fn row_x_groups(row_words: &[PdfOxideWord]) -> Vec<f32> {
    let mut x_groups = Vec::<f32>::new();
    for x in row_words.iter().map(|word| word.x0) {
        if x_groups.last().is_none_or(|last| x - *last > 50.0) {
            x_groups.push(x);
        }
    }
    x_groups
}

fn infer_global_columns(row_info: &[PdfOxideRow], page_width: f32) -> Option<Vec<f32>> {
    let mut all_table_x_positions = Vec::new();
    for info in row_info {
        if info.x_groups.len() >= 3 && !info.is_paragraph {
            all_table_x_positions.extend(info.x_groups.iter().copied());
        }
    }
    if all_table_x_positions.is_empty() {
        return None;
    }
    all_table_x_positions.sort_by(f32::total_cmp);

    let adaptive_tolerance = adaptive_column_tolerance(&all_table_x_positions);
    let mut global_columns = Vec::<f32>::new();
    for x in all_table_x_positions {
        if global_columns
            .last()
            .is_none_or(|last| x - *last > adaptive_tolerance)
        {
            global_columns.push(x);
        }
    }
    valid_global_columns(global_columns, page_width)
}

fn adaptive_column_tolerance(all_table_x_positions: &[f32]) -> f32 {
    let gaps = all_table_x_positions
        .windows(2)
        .filter_map(|pair| {
            let gap = pair[1] - pair[0];
            (gap > 5.0).then_some(gap)
        })
        .collect::<Vec<_>>();
    if gaps.len() >= 3 {
        let mut sorted_gaps = gaps;
        sorted_gaps.sort_by(f32::total_cmp);
        sorted_gaps[(sorted_gaps.len() as f32 * 0.70) as usize].clamp(25.0, 50.0)
    } else {
        35.0
    }
}

fn valid_global_columns(global_columns: Vec<f32>, page_width: f32) -> Option<Vec<f32>> {
    if global_columns.len() <= 1 {
        return None;
    }
    let content_width = global_columns[global_columns.len() - 1] - global_columns[0];
    let avg_col_width = content_width / global_columns.len() as f32;
    let columns_per_inch = global_columns.len() as f32 / (content_width / 72.0);
    let adaptive_max_columns = (20.0 * (page_width / 612.0)).max(15.0) as usize;
    (avg_col_width >= 30.0
        && columns_per_inch <= 10.0
        && global_columns.len() <= adaptive_max_columns)
        .then_some(global_columns)
}

fn mark_table_rows(row_info: &mut [PdfOxideRow], global_columns: &[f32]) {
    for info in row_info {
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
}

fn is_pdf_partial_numbering(text: &str) -> bool {
    let Some(rest) = text.strip_prefix('.') else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_words_with_non_finite_coordinates() {
        assert!(pdf_oxide_word("ok".to_string(), 10.0, 10.0, 10.0).is_some());
        assert!(pdf_oxide_word("bad".to_string(), f32::NAN, 10.0, 10.0).is_none());
        assert!(pdf_oxide_word("bad".to_string(), 10.0, f32::INFINITY, 10.0).is_none());
        assert!(pdf_oxide_word("bad".to_string(), 10.0, 10.0, f32::NEG_INFINITY).is_none());
    }
}
mod table;

use table::{has_enough_table_rows, render_rows_with_tables, table_regions};
