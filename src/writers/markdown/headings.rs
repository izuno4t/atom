use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct HeadingState {
    pub(super) h1_written: bool,
    pub(super) last_level: u8,
    headings: BTreeMap<String, usize>,
}

pub(super) fn normalize_heading_level(level: u8, heading_state: &mut HeadingState) -> u8 {
    let mut level = level.clamp(1, 6);
    if !heading_state.h1_written && heading_state.last_level == 0 {
        level = 1;
        heading_state.h1_written = true;
    } else if level == 1 {
        if heading_state.h1_written {
            level = 2;
        } else {
            heading_state.h1_written = true;
        }
    }
    if heading_state.last_level > 0 && level > heading_state.last_level + 1 {
        level = heading_state.last_level + 1;
    }
    heading_state.last_level = level;
    level
}

pub(super) fn unique_heading_text(text: &str, heading_state: &mut HeadingState) -> String {
    let count = heading_state.headings.entry(text.to_string()).or_insert(0);
    *count += 1;
    if *count == 1 {
        text.to_string()
    } else {
        format!("{text} ({count})")
    }
}

pub(super) fn should_promote_first_paragraph(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty()
        && trimmed.chars().count() <= 80
        && !trimmed.starts_with("Unsupported input format:")
        && !trimmed.ends_with('。')
        && !trimmed.ends_with('.')
        && !trimmed.ends_with(',')
        && !trimmed.ends_with('、')
}

pub(super) fn normalize_heading_text(text: &str) -> String {
    text.trim_end_matches(|character| {
        matches!(
            character,
            '.' | ',' | ';' | ':' | '!' | '?' | '。' | '、' | '；' | '：' | '！' | '？'
        )
    })
    .to_string()
}

pub(super) fn split_long_heading(text: &str, level: u8) -> (String, Option<String>) {
    let available = 80usize.saturating_sub(level as usize + 1);
    if text.chars().count() <= available {
        return (text.to_string(), None);
    }
    let mut split_at = None;
    let mut count = 0;
    for (index, character) in text.char_indices() {
        count += 1;
        if character.is_whitespace() && count <= available {
            split_at = Some(index);
        }
        if count > available {
            break;
        }
    }
    let Some(index) = split_at else {
        return (text.chars().take(available).collect(), None);
    };
    let heading = text[..index].trim_end().to_string();
    let overflow = text[index..].trim().to_string();
    (heading, (!overflow.is_empty()).then_some(overflow))
}
