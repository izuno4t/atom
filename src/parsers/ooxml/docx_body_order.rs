pub(crate) enum DocxBlock {
    Paragraph(String),
    Table(String),
}

pub(crate) fn extract_body_blocks(xml: &str) -> Vec<DocxBlock> {
    let body = body_content(xml);
    let mut blocks = Vec::new();
    let mut rest = body;

    loop {
        let paragraph_start = find_tag_start(rest, "w:p", 0);
        let table_start = find_tag_start(rest, "w:tbl", 0);
        let Some((kind, start)) = earliest_block(paragraph_start, table_start) else {
            break;
        };
        let after = &rest[start..];
        let Some((content, consumed)) = (match kind {
            "p" => extract_balanced_block(after, "w:p", "</w:p>"),
            "tbl" => extract_balanced_block(after, "w:tbl", "</w:tbl>"),
            _ => None,
        }) else {
            break;
        };
        match kind {
            "p" => blocks.push(DocxBlock::Paragraph(content)),
            "tbl" => blocks.push(DocxBlock::Table(content)),
            _ => {}
        }
        rest = &after[consumed..];
    }

    blocks
}

fn body_content(xml: &str) -> &str {
    let Some(start) = find_tag_start(xml, "w:body", 0) else {
        return xml;
    };
    let after_start = &xml[start..];
    let Some(open_end) = after_start.find('>') else {
        return xml;
    };
    let body_start = start + open_end + 1;
    let Some(body_end) = xml[body_start..].find("</w:body>") else {
        return &xml[body_start..];
    };
    &xml[body_start..body_start + body_end]
}

fn earliest_block(
    paragraph_start: Option<usize>,
    table_start: Option<usize>,
) -> Option<(&'static str, usize)> {
    match (paragraph_start, table_start) {
        (Some(paragraph), Some(table)) if paragraph <= table => Some(("p", paragraph)),
        (Some(_), Some(table)) => Some(("tbl", table)),
        (Some(paragraph), None) => Some(("p", paragraph)),
        (None, Some(table)) => Some(("tbl", table)),
        (None, None) => None,
    }
}

fn extract_balanced_block(input: &str, tag: &str, close: &str) -> Option<(String, usize)> {
    let open_end = input.find('>')?;
    if input[..open_end].trim_end().ends_with('/') {
        return Some((String::new(), open_end + 1));
    }
    let body_start = open_end + 1;
    let mut depth = 1usize;
    let mut index = body_start;

    while index < input.len() {
        let next_open = find_tag_start(input, tag, index);
        let next_close = input[index..].find(close).map(|offset| index + offset);
        match (next_open, next_close) {
            (Some(open), Some(close_start)) if open < close_start => {
                depth += 1;
                index = open + tag.len() + 1;
            }
            (_, Some(close_start)) => {
                depth -= 1;
                if depth == 0 {
                    return Some((
                        input[body_start..close_start].to_string(),
                        close_start + close.len(),
                    ));
                }
                index = close_start + close.len();
            }
            _ => return None,
        }
    }

    None
}

fn find_tag_start(input: &str, tag: &str, from: usize) -> Option<usize> {
    let marker = format!("<{tag}");
    let mut search_from = from;
    while let Some(offset) = input[search_from..].find(&marker) {
        let start = search_from + offset;
        let after = start + marker.len();
        let boundary = input[after..].chars().next();
        if boundary.is_some_and(|character| {
            character.is_whitespace() || character == '>' || character == '/'
        }) {
            return Some(start);
        }
        search_from = after;
    }
    None
}
