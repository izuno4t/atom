pub(crate) fn relationship_target(rels_xml: &str, id: &str) -> Option<String> {
    let marker = format!("Id=\"{id}\"");
    let start = rels_xml.find(&marker)?;
    let rest = &rels_xml[start..];
    let attr = "Target=\"";
    let value_start = rest.find(attr)? + attr.len();
    let value_end = rest[value_start..].find('"')?;
    Some(rest[value_start..value_start + value_end].to_string())
}

pub(crate) fn extract_attr_values_for_tag(input: &str, tag: &str, attr: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = input;
    let marker = format!("<{tag}");
    while let Some(start) = rest.find(&marker) {
        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            break;
        };
        if let Some(value) = attr_value(&after[..=end], attr) {
            values.push(value);
        }
        rest = &after[end + 1..];
    }
    values
}

pub(crate) fn attr_value(input: &str, name: &str) -> Option<String> {
    let pattern = format!("{name}=\"");
    let value_start = input.find(&pattern)? + pattern.len();
    let value_end = input[value_start..].find('"')?;
    Some(input[value_start..value_start + value_end].to_string())
}
