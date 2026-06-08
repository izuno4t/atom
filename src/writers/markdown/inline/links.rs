pub(super) fn repair_split_url_tokens<'a>(tokens: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut repaired = Vec::<String>::new();
    for token in tokens {
        if let Some(previous) = repaired.last_mut()
            && should_join_split_url(previous, token)
        {
            previous.push_str(token);
            continue;
        }
        repaired.push(token.to_string());
    }
    repaired
}

pub(super) fn angle_bracket_bare_link(token: &str) -> String {
    let (prefix, core, suffix) = split_surrounding_punctuation(token);
    if is_angle_wrapped_autolink(core) {
        return token.to_string();
    }
    if core.starts_with("http://")
        || core.starts_with("https://")
        || core.starts_with("www.")
        || looks_like_email(core)
    {
        format!("{prefix}<{core}>{suffix}")
    } else if contains_url_prefix(token) {
        escape_inline_angle_brackets(token)
    } else if let Some(token) = angle_bracket_embedded_email(token) {
        token
    } else {
        escape_markdown_text_markers(&escape_inline_angle_brackets(token))
    }
}

pub(super) fn angle_bracket_embedded_urls(text: &str) -> String {
    let mut output = String::new();
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let Some(relative_start) = find_next_embedded_url_start(rest) else {
            output.push_str(rest);
            break;
        };
        let start = index + relative_start;
        output.push_str(&text[index..start]);
        if start > 0 && text[..start].ends_with('<') {
            output.push_str(&text[start..start + url_prefix_len(&text[start..])]);
            index = start + url_prefix_len(&text[start..]);
            continue;
        }
        let end = find_embedded_url_end(text, start);
        output.push('<');
        output.push_str(&text[start..end]);
        output.push('>');
        index = end;
    }
    output
}

fn should_join_split_url(previous: &str, next: &str) -> bool {
    let next_core = next.strip_suffix('>').unwrap_or(next);
    contains_url_prefix(previous)
        && next_core.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(
                    character,
                    '/' | '.' | '_' | '-' | '~' | '?' | '#' | '&' | '=' | '%' | ':'
                )
        })
        && (previous.ends_with('/') || previous.ends_with(".h") || previous.ends_with('-'))
}

pub(in crate::writers::markdown) fn contains_url_prefix(text: &str) -> bool {
    text.contains("http://") || text.contains("https://") || text.contains("www.")
}

fn angle_bracket_embedded_email(token: &str) -> Option<String> {
    let at = token.find('@')?;
    let start = token[..at]
        .char_indices()
        .rev()
        .find(|(_, character)| !is_email_local_character(*character))
        .map(|(index, character)| index + character.len_utf8())
        .unwrap_or(0);
    let end = token[at + 1..]
        .char_indices()
        .find(|(_, character)| !is_email_domain_character(*character))
        .map(|(index, _)| at + 1 + index)
        .unwrap_or(token.len());
    let candidate = &token[start..end];
    if !looks_like_email(candidate) {
        return None;
    }

    let mut wrapped = String::new();
    wrapped.push_str(&escape_markdown_text_markers(
        &escape_inline_angle_brackets(&token[..start]),
    ));
    wrapped.push('<');
    wrapped.push_str(candidate);
    wrapped.push('>');
    wrapped.push_str(&escape_markdown_text_markers(
        &escape_inline_angle_brackets(&token[end..]),
    ));
    Some(wrapped)
}

pub(in crate::writers::markdown) fn is_angle_wrapped_autolink(token: &str) -> bool {
    let Some(inner) = token
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
    else {
        return false;
    };
    inner.starts_with("http://")
        || inner.starts_with("https://")
        || inner.starts_with("www.")
        || looks_like_email(inner)
}

fn escape_markdown_text_markers(token: &str) -> String {
    token
        .replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('`', "\\`")
}

fn escape_inline_angle_brackets(token: &str) -> String {
    if !(token.contains('<') || token.contains('>')) {
        return token.to_string();
    }
    token.replace('<', "&lt;").replace('>', "&gt;")
}

fn find_next_embedded_url_start(text: &str) -> Option<usize> {
    ["http://", "https://", "www."]
        .iter()
        .filter_map(|prefix| text.find(prefix))
        .min()
}

fn url_prefix_len(text: &str) -> usize {
    if text.starts_with("https://") {
        "https://".len()
    } else if text.starts_with("http://") {
        "http://".len()
    } else {
        "www.".len()
    }
}

fn find_embedded_url_end(text: &str, start: usize) -> usize {
    for (offset, character) in text[start..].char_indices() {
        if character.is_whitespace()
            || matches!(
                character,
                '<' | '>' | '"' | '\'' | '“' | '”' | '‘' | '’' | ')' | '）'
            )
            || !character.is_ascii()
        {
            return start + offset;
        }
    }
    text.len()
}

fn split_surrounding_punctuation(token: &str) -> (&str, &str, &str) {
    let prefix_len = token
        .char_indices()
        .find(|(_, character)| character.is_alphanumeric() || *character == 'h')
        .map(|(index, _)| index)
        .unwrap_or(0);
    let mut suffix_start = token.len();
    for (index, character) in token.char_indices().rev() {
        if character.is_alphanumeric() || matches!(character, '/' | '-') {
            suffix_start = index + character.len_utf8();
            break;
        }
    }
    if prefix_len >= suffix_start {
        return ("", token, "");
    }
    (
        &token[..prefix_len],
        &token[prefix_len..suffix_start],
        &token[suffix_start..],
    )
}

fn looks_like_email(token: &str) -> bool {
    let Some((local, domain)) = token.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && domain
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
}

fn is_email_local_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '%' | '+' | '-')
}

fn is_email_domain_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '-')
}
