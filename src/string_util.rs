//! String helpers matching Minecraft's `StringUtil` utility.

#![allow(dead_code)]

fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

fn truncate_utf16_units(value: &str, limit: usize) -> String {
    let mut result = String::new();
    let mut used = 0;
    for character in value.chars() {
        let width = character.len_utf16();
        if used + width > limit {
            break;
        }
        result.push(character);
        used += width;
    }
    result
}

fn is_color_code(character: char) -> bool {
    matches!(
        character,
        '0'..='9' | 'a'..='f' | 'A'..='F' | 'k'..='o' | 'K'..='O' | 'r' | 'R'
    )
}

pub fn format_tick_duration(ticks: i32, tickrate: f32) -> String {
    let mut seconds = (ticks as f32 / tickrate).floor() as i32;
    let mut minutes = seconds / 60;
    seconds %= 60;
    let hours = minutes / 60;
    minutes %= 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

pub fn strip_color(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut characters = input.chars();
    while let Some(character) = characters.next() {
        if character == '\u{00a7}' {
            if let Some(next) = characters.next() {
                if is_color_code(next) {
                    continue;
                }
                result.push(character);
                result.push(next);
                continue;
            }
        }
        result.push(character);
    }
    result
}

pub fn is_null_or_empty(value: Option<&str>) -> bool {
    value.is_none_or(str::is_empty)
}

pub fn truncate_string_if_necessary(
    value: &str,
    max_length: usize,
    add_dot_dot_dot_if_truncated: bool,
) -> String {
    if utf16_len(value) <= max_length {
        value.to_string()
    } else if add_dot_dot_dot_if_truncated && max_length > 3 {
        format!("{}...", truncate_utf16_units(value, max_length - 3))
    } else {
        truncate_utf16_units(value, max_length)
    }
}

pub fn line_count(value: &str) -> usize {
    if value.is_empty() {
        return 0;
    }
    let characters: Vec<char> = value.chars().collect();
    let mut count = 1;
    let mut index = 0;
    while index < characters.len() {
        if characters[index] == '\r' && characters.get(index + 1) == Some(&'\n') {
            count += 1;
            index += 2;
        } else if characters[index] == '\u{000b}' {
            count += 1;
            index += 1;
        } else {
            index += 1;
        }
    }
    count
}

pub fn ends_with_new_line(value: &str) -> bool {
    value.ends_with("\r\n") || value.ends_with('\u{000b}')
}

pub fn trim_chat_message(message: &str) -> String {
    truncate_string_if_necessary(message, 256, false)
}

pub fn is_allowed_chat_character(character: i32) -> bool {
    character != 167 && character >= 32 && character != 127
}

pub fn is_valid_player_name(name: &str) -> bool {
    utf16_len(name) <= 16
        && name
            .chars()
            .all(|character| {
                let codepoint = character as i32;
                codepoint > 32 && codepoint < 127
            })
}

pub fn filter_text(input: &str, multiline: bool) -> String {
    input
        .chars()
        .filter(|character| {
            is_allowed_chat_character(*character as i32) || (multiline && *character == '\n')
        })
        .collect()
}

fn is_space_char(character: char) -> bool {
    matches!(
        character,
        '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
    )
}

pub fn is_whitespace(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32)
        .is_some_and(|character| character.is_whitespace() || is_space_char(character))
}

pub fn is_blank(value: Option<&str>) -> bool {
    value.is_none_or(|value| value.chars().all(|character| is_whitespace(character as i32)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn string_util_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/StringUtil.java");
        assert_eq!(JAVA.lines().count(), 95);
        for fragment in [
            "private static final Pattern STRIP_COLOR_PATTERN",
            "private static final Pattern LINE_PATTERN",
            "public static String formatTickDuration",
            "public static String stripColor",
            "public static boolean isNullOrEmpty",
            "public static int lineCount",
            "public static boolean endsWithNewLine",
            "public static String filterText",
            "public static boolean isValidPlayerName",
        ] {
            assert!(JAVA.contains(fragment), "missing StringUtil source fragment: {fragment}");
        }
    }

    #[test]
    fn string_util_matches_vanilla_ascii_and_control_semantics() {
        assert_eq!(format_tick_duration(3_725, 20.0), "03:06");
        assert_eq!(format_tick_duration(72_000, 20.0), "01:00:00");
        assert_eq!(strip_color("§aGreen §Lbold§r"), "Green bold");
        assert_eq!(strip_color("§x"), "§x");
        assert!(is_null_or_empty(None));
        assert!(is_null_or_empty(Some("")));
        assert_eq!(truncate_string_if_necessary("abcdef", 4, true), "a...");
        assert_eq!(truncate_string_if_necessary("abcdef", 4, false), "abcd");
        assert_eq!(line_count("a\r\nb\u{000b}c"), 3);
        assert!(ends_with_new_line("a\r\n"));
        assert!(ends_with_new_line("a\u{000b}"));
        assert!(!ends_with_new_line("a\n"));
        assert_eq!(trim_chat_message(&"a".repeat(257)).len(), 256);
        assert!(is_valid_player_name("Player_1"));
        assert!(!is_valid_player_name("name with space"));
        assert!(!is_valid_player_name("é"));
        assert_eq!(filter_text("ok\u{0001}\n", false), "ok");
        assert_eq!(filter_text("ok\u{0001}\n", true), "ok\n");
        assert!(is_whitespace(' ' as i32));
        assert!(is_whitespace('\u{00a0}' as i32));
        assert!(is_blank(Some(" \t")));
        assert!(is_blank(None));
        assert!(!is_blank(Some(" x ")));
    }
}
