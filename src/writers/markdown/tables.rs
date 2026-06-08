use crate::{Flavor, TableRow, escape_html};

pub(super) fn write_table(rows: &[TableRow], flavor: Flavor, output: &mut String) {
    let requires_html = rows.iter().flat_map(|row| &row.cells).any(|cell| {
        cell.rowspan > 1 || cell.colspan > 1 || cell.image.is_some() || cell.text.contains('\n')
    }) || matches!(flavor, Flavor::CommonMark | Flavor::HedgeDoc);
    if requires_html {
        write_html_table(rows, output);
        return;
    }

    if rows.is_empty() {
        return;
    }
    let header = &rows[0];
    output.push('|');
    for cell in &header.cells {
        output.push(' ');
        output.push_str(&cell.text);
        output.push_str(" |");
    }
    output.push('\n');
    output.push('|');
    for _ in &header.cells {
        output.push_str(" --- |");
    }
    output.push('\n');
    for row in rows.iter().skip(1) {
        output.push('|');
        for cell in &row.cells {
            output.push(' ');
            output.push_str(&cell.text);
            output.push_str(" |");
        }
        output.push('\n');
    }
    output.push('\n');
}

fn write_html_table(rows: &[TableRow], output: &mut String) {
    output.push_str("<table>\n");
    for row in rows {
        output.push_str("<tr>");
        for cell in &row.cells {
            output.push_str("<td");
            if cell.rowspan > 1 {
                output.push_str(&format!(" rowspan=\"{}\"", cell.rowspan));
            }
            if cell.colspan > 1 {
                output.push_str(&format!(" colspan=\"{}\"", cell.colspan));
            }
            output.push('>');
            if let Some(path) = &cell.image {
                output.push_str("<img src=\"");
                output.push_str(&escape_html(path));
                output.push_str("\" alt=\"");
                output.push_str(&escape_html(&cell.text));
                output.push_str("\">");
            } else {
                output.push_str(&escape_html(&cell.text));
            }
            output.push_str("</td>");
        }
        output.push_str("</tr>\n");
    }
    output.push_str("</table>\n\n");
}
