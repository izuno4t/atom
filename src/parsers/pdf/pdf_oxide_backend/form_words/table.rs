use super::PdfOxideRow;

pub(super) fn table_regions(row_info: &[PdfOxideRow]) -> Vec<(usize, usize)> {
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
    table_regions
}

pub(super) fn has_enough_table_rows(
    row_info: &[PdfOxideRow],
    table_regions: &[(usize, usize)],
) -> bool {
    let total_table_rows = table_regions
        .iter()
        .map(|(start, end)| end - start)
        .sum::<usize>();
    row_info.is_empty() || total_table_rows as f32 / (row_info.len() as f32) >= 0.2
}

pub(super) fn render_rows_with_tables(
    row_info: &[PdfOxideRow],
    table_regions: &[(usize, usize)],
    global_columns: &[f32],
) -> String {
    let mut result_lines = Vec::new();
    let mut idx = 0;
    while idx < row_info.len() {
        if let Some((_, end)) = table_regions
            .iter()
            .find(|(start, _)| *start == idx)
            .copied()
        {
            render_table_region(row_info, idx, end, global_columns, &mut result_lines);
            idx = end;
        } else {
            result_lines.push(row_info[idx].text.clone());
            idx += 1;
        }
    }
    result_lines.join("\n")
}

fn render_table_region(
    row_info: &[PdfOxideRow],
    start: usize,
    end: usize,
    global_columns: &[f32],
    result_lines: &mut Vec<String>,
) {
    let table_data = (start..end)
        .map(|table_idx| extract_pdf_oxide_cells(&row_info[table_idx], global_columns))
        .collect::<Vec<_>>();
    let col_widths = (0..global_columns.len())
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
}

fn extract_pdf_oxide_cells(info: &PdfOxideRow, global_columns: &[f32]) -> Vec<String> {
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
