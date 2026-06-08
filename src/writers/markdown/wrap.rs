use super::inline::{contains_url_prefix, is_angle_wrapped_autolink};

pub(super) fn write_wrapped_text(text: &str, output: &mut String) {
    let mut line = String::new();
    for token in text.split_whitespace() {
        if line.is_empty() {
            push_wrapped_token(token, output, &mut line);
        } else if line.chars().count() + 1 + token.chars().count() <= 80 {
            line.push(' ');
            line.push_str(token);
        } else {
            output.push_str(line.trim_end());
            output.push('\n');
            line.clear();
            push_wrapped_token(token, output, &mut line);
        }
    }
    if !line.is_empty() {
        output.push_str(line.trim_end());
    }
}

fn push_wrapped_token(token: &str, output: &mut String, line: &mut String) {
    let token_len = token.chars().count();
    if token_len <= 80 || is_angle_wrapped_autolink(token) || contains_url_prefix(token) {
        line.push_str(token);
        return;
    }
    let mut line_len = line.chars().count();
    for character in token.chars() {
        if line_len >= 80 {
            output.push_str(line);
            output.push('\n');
            line.clear();
            line_len = 0;
        }
        line.push(character);
        line_len += 1;
    }
}

pub(super) fn write_wrapped_prefixed_text(
    text: &str,
    output: &mut String,
    prefix: &str,
    continuation: &str,
) {
    let mut line = prefix.to_string();
    for token in text.split_whitespace() {
        if line == prefix || line == continuation {
            line.push_str(token);
        } else if line.chars().count() + 1 + token.chars().count() <= 80
            || is_angle_wrapped_autolink(token)
            || contains_url_prefix(token)
        {
            line.push(' ');
            line.push_str(token);
        } else {
            output.push_str(line.trim_end());
            output.push('\n');
            line.clear();
            line.push_str(continuation);
            line.push_str(token);
        }
    }
    output.push_str(line.trim_end());
}
