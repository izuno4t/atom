use super::{PdfOxideRow, PdfOxideWord};

#[derive(Clone)]
struct PdfTableColumn {
    left_boundary: f32,
    right_boundary: f32,
}

#[derive(Clone)]
pub(super) struct PdfTableSchema {
    columns: Vec<PdfTableColumn>,
    header: Vec<String>,
}

pub(super) fn render_multiline_table(
    row_info: &[PdfOxideRow],
    previous_schema: Option<&PdfTableSchema>,
) -> Option<(String, PdfTableSchema)> {
    let detected_schema = detect_schema(row_info);
    let (schema, header_idx, output_schema) = if let Some(schema) = previous_schema {
        let output_schema = detected_schema
            .as_ref()
            .map(|(_, schema)| schema.clone())
            .unwrap_or_else(|| schema.clone());
        (schema, None, output_schema)
    } else {
        let (idx, schema) = detected_schema.as_ref()?;
        (schema, Some(*idx), schema.clone())
    };
    let header_top = header_idx.map(|idx| row_info[idx].top);
    let anchors = data_row_anchors(row_info, &schema.columns, header_idx, header_top);
    if anchors.len() < 2 {
        return None;
    }

    let mut table_data = Vec::with_capacity(anchors.len() + 1);
    table_data.push(schema.header.clone());
    table_data.extend((0..anchors.len()).map(|_| vec![String::new(); schema.columns.len()]));

    let mut body_rows = row_info
        .iter()
        .enumerate()
        .filter(|(idx, row)| Some(*idx) != header_idx && header_top.is_none_or(|top| row.top < top))
        .collect::<Vec<_>>();
    body_rows.sort_by(|(_, left), (_, right)| right.top.total_cmp(&left.top));

    for (_, row) in body_rows {
        let anchor_idx = row_anchor_index(row.top, &anchors)?;
        let cells = split_words_into_cells(&row.words, &schema.columns);
        append_cells(&mut table_data[anchor_idx + 1], cells);
    }

    if table_data
        .iter()
        .skip(1)
        .any(|row| row.iter().filter(|cell| !cell.is_empty()).count() < 2)
    {
        return None;
    }

    Some((render_table_data(&table_data), output_schema))
}

fn detect_schema(row_info: &[PdfOxideRow]) -> Option<(usize, PdfTableSchema)> {
    let (header_idx, header) = row_info
        .iter()
        .enumerate()
        .filter(|(_, row)| !row.is_paragraph && row.words.len() >= 3)
        .max_by(|(_, left), (_, right)| left.top.total_cmp(&right.top))?;
    let columns = infer_columns_from_header(header)?;
    let header = split_words_into_cells(&header.words, &columns);
    Some((header_idx, PdfTableSchema { columns, header }))
}

fn infer_columns_from_header(header: &PdfOxideRow) -> Option<Vec<PdfTableColumn>> {
    let words = header
        .words
        .iter()
        .filter(|word| !word.text.trim().is_empty())
        .collect::<Vec<_>>();
    if words.len() < 3 {
        return None;
    }

    let mut columns = Vec::with_capacity(words.len());
    for (idx, word) in words.iter().enumerate() {
        let left_boundary = if idx == 0 {
            f32::NEG_INFINITY
        } else {
            (words[idx - 1].x1 + word.x0) / 2.0
        };
        let right_boundary = if idx + 1 == words.len() {
            f32::INFINITY
        } else {
            (word.x1 + words[idx + 1].x0) / 2.0
        };
        columns.push(PdfTableColumn {
            left_boundary,
            right_boundary,
        });
    }
    Some(columns)
}

fn data_row_anchors(
    rows: &[PdfOxideRow],
    columns: &[PdfTableColumn],
    header_idx: Option<usize>,
    header_top: Option<f32>,
) -> Vec<f32> {
    let mut anchors = rows
        .iter()
        .enumerate()
        .filter_map(|(idx, row)| {
            if Some(idx) == header_idx || header_top.is_some_and(|top| row.top >= top) {
                return None;
            }
            let cells = split_words_into_cells(&row.words, columns);
            (!cells.first().is_none_or(String::is_empty)).then_some(row.top)
        })
        .collect::<Vec<_>>();
    anchors.sort_by(|left, right| right.total_cmp(left));
    anchors
}

fn row_anchor_index(row_top: f32, anchors: &[f32]) -> Option<usize> {
    if anchors.is_empty() {
        return None;
    }
    for idx in 0..anchors.len() - 1 {
        let boundary = (anchors[idx] + anchors[idx + 1]) / 2.0;
        if row_top > boundary {
            return Some(idx);
        }
    }
    Some(anchors.len() - 1)
}

fn split_words_into_cells(words: &[PdfOxideWord], columns: &[PdfTableColumn]) -> Vec<String> {
    let mut cells = vec![String::new(); columns.len()];
    for word in words {
        let col_idx = columns
            .iter()
            .position(|column| word.x0 >= column.left_boundary && word.x0 < column.right_boundary)
            .unwrap_or(columns.len() - 1);
        if !cells[col_idx].is_empty() {
            cells[col_idx].push(' ');
        }
        cells[col_idx].push_str(word.text.trim());
    }
    cells
}

fn append_cells(target: &mut [String], cells: Vec<String>) {
    for (target_cell, cell) in target.iter_mut().zip(cells) {
        if cell.is_empty() {
            continue;
        }
        if !target_cell.is_empty() {
            target_cell.push(' ');
        }
        target_cell.push_str(&cell);
    }
}

fn render_table_data(table_data: &[Vec<String>]) -> String {
    let col_count = table_data.first().map(Vec::len).unwrap_or(0);
    let col_widths = (0..col_count)
        .map(|col| {
            table_data
                .iter()
                .map(|row| row[col].len())
                .max()
                .unwrap_or(0)
                .max(3)
        })
        .collect::<Vec<_>>();
    let mut result_lines = Vec::new();
    if let Some(header) = table_data.first() {
        result_lines.push(format_pdf_table_row(header, &col_widths));
        result_lines.push(format!(
            "| {} |",
            col_widths
                .iter()
                .map(|width| "-".repeat(*width))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
        for row in table_data.iter().skip(1) {
            result_lines.push(format_pdf_table_row(row, &col_widths));
        }
    }
    result_lines.join("\n")
}

fn format_pdf_table_row(row: &[String], col_widths: &[usize]) -> String {
    format!(
        "| {} |",
        row.iter()
            .enumerate()
            .map(|(idx, cell)| format!("{cell:<width$}", width = col_widths[idx]))
            .collect::<Vec<_>>()
            .join(" | ")
    )
}
