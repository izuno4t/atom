#[derive(Clone)]
struct PdfOxideWord {
    text: String,
    x0: f32,
    x1: f32,
    top: f32,
}

struct PdfOxideRow {
    words: Vec<PdfOxideWord>,
    top: f32,
    is_paragraph: bool,
}

pub(super) fn extract_pdf_oxide_form_words_document(
    document: &pdf_oxide::PdfDocument,
    page_count: usize,
) -> pdf_oxide::error::Result<String> {
    let mut chunks = Vec::new();
    let mut previous_schema = None;
    for page_index in 0..page_count {
        let words = extract_pdf_oxide_form_words_page(document, page_index)?;
        if words.is_empty() {
            continue;
        }
        let (content, schema) =
            extract_form_content_from_pdf_oxide_words(words, previous_schema.as_ref());
        if let Some(schema) = schema {
            previous_schema = Some(schema);
        }
        if !content.trim().is_empty() {
            chunks.push(content);
        }
    }
    Ok(chunks.join("\n\n"))
}

fn extract_pdf_oxide_form_words_page(
    document: &pdf_oxide::PdfDocument,
    page_index: usize,
) -> pdf_oxide::error::Result<Vec<PdfOxideWord>> {
    let words = document
        .extract_words(page_index)?
        .into_iter()
        .filter_map(|word| pdf_oxide_word(word.text, word.bbox.x, word.bbox.width, word.bbox.y))
        .collect::<Vec<_>>();
    Ok(words)
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

fn extract_form_content_from_pdf_oxide_words(
    words: Vec<PdfOxideWord>,
    previous_schema: Option<&PdfTableSchema>,
) -> (String, Option<PdfTableSchema>) {
    if words.is_empty() {
        return (String::new(), None);
    }

    let rows_by_y = group_words_by_y(words);
    let page_width = rows_by_y
        .values()
        .flat_map(|row| row.iter().map(|word| word.x1))
        .fold(612.0_f32, f32::max);
    let row_info = rows_by_y
        .into_values()
        .filter_map(|row_words| build_pdf_oxide_row(row_words, page_width))
        .filter(|row| !is_pdf_oxide_rule_row(row))
        .collect::<Vec<_>>();
    if let Some((content, schema)) = render_multiline_table(&row_info, previous_schema) {
        return (content, Some(schema));
    }
    (render_plain_rows(&row_info), None)
}

fn render_plain_rows(row_info: &[PdfOxideRow]) -> String {
    let mut rows = row_info.iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| right.top.total_cmp(&left.top));
    rows.into_iter()
        .map(|row| {
            row.words
                .iter()
                .map(|word| word.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !is_pdf_oxide_rule_text(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_pdf_oxide_rule_row(row: &PdfOxideRow) -> bool {
    let text = row
        .words
        .iter()
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>()
        .join("");
    is_pdf_oxide_rule_text(&text)
}

fn is_pdf_oxide_rule_text(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.len() >= 3
        && trimmed
            .chars()
            .all(|character| matches!(character, '-' | '|' | ':' | ' '))
        && trimmed.chars().any(|character| character == '-')
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
    let top = row_words
        .iter()
        .map(|word| word.top)
        .fold(f32::INFINITY, f32::min);
    Some(PdfOxideRow {
        words: row_words,
        top,
        is_paragraph: line_width > page_width * 0.55 && combined_text.len() > 60,
    })
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

    #[test]
    fn renders_multiline_cells_as_table_from_column_geometry() {
        let words = vec![
            word("No", 50.0, 10.0, 100.0),
            word("Question", 85.0, 48.0, 100.0),
            word("Owner", 350.0, 35.0, 100.0),
            word("Answer", 400.0, 42.0, 100.0),
            word("Why", 85.0, 25.0, 80.0),
            word("test?", 112.0, 35.0, 80.0),
            word("Because", 400.0, 60.0, 80.0),
            word("1", 72.0, 7.0, 60.0),
            word("quality", 85.0, 45.0, 60.0),
            word("Team", 350.0, 32.0, 60.0),
            word("matters.", 400.0, 55.0, 60.0),
            word("How", 85.0, 28.0, 40.0),
            word("often?", 115.0, 45.0, 40.0),
            word("Every", 400.0, 38.0, 40.0),
            word("2", 72.0, 7.0, 20.0),
            word("release", 85.0, 50.0, 20.0),
            word("Ops", 350.0, 24.0, 20.0),
            word("cycle.", 400.0, 42.0, 20.0),
        ];

        let (actual, _) = extract_form_content_from_pdf_oxide_words(words, None);

        assert!(actual.starts_with("| No"));
        assert!(actual.contains("Question"));
        assert!(
            actual.contains("| 1   | Why test? quality  | Team  | Because matters. |"),
            "{actual}"
        );
        assert!(
            actual.contains("| 2   | How often? release | Ops   | Every cycle.     |"),
            "{actual}"
        );
    }

    fn word(text: &str, x: f32, width: f32, y: f32) -> PdfOxideWord {
        PdfOxideWord {
            text: text.to_string(),
            x0: x,
            x1: x + width,
            top: y,
        }
    }
}
mod table;

use table::{PdfTableSchema, render_multiline_table};
