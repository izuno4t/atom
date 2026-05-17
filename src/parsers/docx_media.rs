use super::docx_relationships::{extract_attr_values_for_tag, relationship_target};

pub(crate) fn extract_image_target(input: &str, rels_xml: &str) -> Option<String> {
    extract_image_relationship_id(input)
        .map(|id| relationship_target(rels_xml, &id).unwrap_or_else(|| format!("media/{id}.png")))
}

fn extract_image_relationship_id(input: &str) -> Option<String> {
    extract_attr_values_for_tag(input, "a:blip", "r:embed")
        .into_iter()
        .next()
        .or_else(|| {
            extract_attr_values_for_tag(input, "a:blip", "r:link")
                .into_iter()
                .next()
        })
        .or_else(|| {
            extract_attr_values_for_tag(input, "v:imagedata", "r:id")
                .into_iter()
                .next()
        })
}
