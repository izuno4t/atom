pub(super) fn is_probably_human_text(text: &str) -> bool {
    if text
        .chars()
        .filter(|character| *character == '\u{fffd}')
        .count()
        >= 2
    {
        return false;
    }
    let char_count = text.chars().count();
    if char_count == 0 {
        return false;
    }
    let control_count = text
        .chars()
        .filter(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        .count();
    if control_count * 4 > char_count {
        return false;
    }
    if char_count > 4000 && !text.chars().any(|character| character.is_whitespace()) {
        return false;
    }
    true
}

pub(super) fn is_pdf_repeated_noise_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.chars().all(|character| character.is_ascii_digit()) && trimmed.chars().count() <= 4 {
        return true;
    }
    if trimmed.contains('©') || trimmed.to_ascii_lowercase().contains("copyright") {
        return true;
    }
    looks_like_pdf_date_footer(trimmed)
}

fn looks_like_pdf_date_footer(text: &str) -> bool {
    let mut parts = text.split_whitespace();
    let Some(date) = parts.next() else {
        return false;
    };
    let Some(time) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && date.len() == 10
        && date.chars().nth(4) == Some('/')
        && date.chars().nth(7) == Some('/')
        && time.len() == 5
        && time.chars().nth(2) == Some(':')
        && date
            .chars()
            .filter(|character| *character != '/')
            .all(|character| character.is_ascii_digit())
        && time
            .chars()
            .filter(|character| *character != ':')
            .all(|character| character.is_ascii_digit())
}
