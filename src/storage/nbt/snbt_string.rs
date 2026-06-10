#![allow(dead_code)]

pub fn quote_snbt_key(key: &str) -> String {
    if !key.eq_ignore_ascii_case("true")
        && !key.eq_ignore_ascii_case("false")
        && is_unquoted_snbt_key(key)
    {
        key.to_string()
    } else {
        quote_and_escape_snbt_string(key)
    }
}

pub fn quote_and_escape_snbt_string(value: &str) -> String {
    let mut out = String::new();
    out.push(' ');
    let mut quote = None;
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' | '\'' => {
                if quote.is_none() {
                    quote = Some(if ch == '"' { '\'' } else { '"' });
                }
                if quote == Some(ch) {
                    out.push('\\');
                }
                out.push(ch);
            }
            _ => {
                if let Some(escaped) = escape_control_character(ch) {
                    out.push('\\');
                    out.push_str(&escaped);
                } else {
                    out.push(ch);
                }
            }
        }
    }
    let quote = quote.unwrap_or('"');
    out.replace_range(0..1, &quote.to_string());
    out.push(quote);
    out
}

pub fn escape_snbt_string_without_quotes(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' | '\'' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => {
                if let Some(escaped) = escape_control_character(ch) {
                    out.push('\\');
                    out.push_str(&escaped);
                } else {
                    out.push(ch);
                }
            }
        }
    }
    out
}

fn is_unquoted_snbt_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || matches!(first, '.' | '_'))
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '+' | '-'))
}

fn escape_control_character(ch: char) -> Option<String> {
    match ch {
        '\u{0008}' => Some("b".to_string()),
        '\t' => Some("t".to_string()),
        '\n' => Some("n".to_string()),
        '\u{000C}' => Some("f".to_string()),
        '\r' => Some("r".to_string()),
        ch if ch < ' ' => Some(format!("x{:02X}", ch as u32)),
        _ => None,
    }
}
