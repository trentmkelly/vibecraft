#![allow(dead_code)]

use std::fmt;

pub const CHAT_FORMATTING_PREFIX_CODE: char = '§';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChatFormatting {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChatFormattingData {
    enum_name: &'static str,
    code: char,
    is_format: bool,
    id: i32,
    color: Option<u32>,
}

impl ChatFormatting {
    pub const VALUES: [Self; 22] = [
        Self::Black,
        Self::DarkBlue,
        Self::DarkGreen,
        Self::DarkAqua,
        Self::DarkRed,
        Self::DarkPurple,
        Self::Gold,
        Self::Gray,
        Self::DarkGray,
        Self::Blue,
        Self::Green,
        Self::Aqua,
        Self::Red,
        Self::LightPurple,
        Self::Yellow,
        Self::White,
        Self::Obfuscated,
        Self::Bold,
        Self::Strikethrough,
        Self::Underline,
        Self::Italic,
        Self::Reset,
    ];

    fn data(self) -> ChatFormattingData {
        match self {
            Self::Black => data("BLACK", '0', false, 0, Some(0)),
            Self::DarkBlue => data("DARK_BLUE", '1', false, 1, Some(170)),
            Self::DarkGreen => data("DARK_GREEN", '2', false, 2, Some(43_520)),
            Self::DarkAqua => data("DARK_AQUA", '3', false, 3, Some(43_690)),
            Self::DarkRed => data("DARK_RED", '4', false, 4, Some(11_141_120)),
            Self::DarkPurple => data("DARK_PURPLE", '5', false, 5, Some(11_141_290)),
            Self::Gold => data("GOLD", '6', false, 6, Some(16_755_200)),
            Self::Gray => data("GRAY", '7', false, 7, Some(11_184_810)),
            Self::DarkGray => data("DARK_GRAY", '8', false, 8, Some(5_592_405)),
            Self::Blue => data("BLUE", '9', false, 9, Some(5_592_575)),
            Self::Green => data("GREEN", 'a', false, 10, Some(5_635_925)),
            Self::Aqua => data("AQUA", 'b', false, 11, Some(5_636_095)),
            Self::Red => data("RED", 'c', false, 12, Some(16_733_525)),
            Self::LightPurple => data("LIGHT_PURPLE", 'd', false, 13, Some(16_733_695)),
            Self::Yellow => data("YELLOW", 'e', false, 14, Some(16_777_045)),
            Self::White => data("WHITE", 'f', false, 15, Some(16_777_215)),
            Self::Obfuscated => data("OBFUSCATED", 'k', true, -1, None),
            Self::Bold => data("BOLD", 'l', true, -1, None),
            Self::Strikethrough => data("STRIKETHROUGH", 'm', true, -1, None),
            Self::Underline => data("UNDERLINE", 'n', true, -1, None),
            Self::Italic => data("ITALIC", 'o', true, -1, None),
            Self::Reset => data("RESET", 'r', false, -1, None),
        }
    }

    pub fn get_char(self) -> char {
        self.data().code
    }

    pub fn get_id(self) -> i32 {
        self.data().id
    }

    pub fn is_format(self) -> bool {
        self.data().is_format
    }

    pub fn is_color(self) -> bool {
        !self.is_format() && self != Self::Reset
    }

    pub fn get_color(self) -> Option<u32> {
        self.data().color
    }

    pub fn get_name(self) -> String {
        self.data().enum_name.to_ascii_lowercase()
    }

    pub fn get_serialized_name(self) -> String {
        self.get_name()
    }

    pub fn get_by_name(name: Option<&str>) -> Option<Self> {
        let cleaned = clean_name(name?);
        Self::VALUES
            .iter()
            .copied()
            .find(|format| clean_name(format.data().enum_name) == cleaned)
    }

    pub fn get_by_id(id: i32) -> Option<Self> {
        if id < 0 {
            return Some(Self::Reset);
        }

        Self::VALUES
            .iter()
            .copied()
            .find(|format| format.get_id() == id)
    }

    pub fn get_by_code(code: char) -> Option<Self> {
        let sanitized = code.to_ascii_lowercase();
        Self::VALUES
            .iter()
            .copied()
            .find(|format| format.get_char() == sanitized)
    }

    pub fn get_names(get_colors: bool, get_formats: bool) -> Vec<String> {
        Self::VALUES
            .iter()
            .copied()
            .filter(|format| {
                (!format.is_color() || get_colors) && (!format.is_format() || get_formats)
            })
            .map(Self::get_name)
            .collect()
    }

    pub fn validate_color_codec(self) -> Result<Self, String> {
        if self.is_format() {
            Err(format!("Formatting was not a valid color: {self}"))
        } else {
            Ok(self)
        }
    }
}

impl fmt::Display for ChatFormatting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{CHAT_FORMATTING_PREFIX_CODE}{}",
            self.get_char()
        )
    }
}

fn data(
    enum_name: &'static str,
    code: char,
    is_format: bool,
    id: i32,
    color: Option<u32>,
) -> ChatFormattingData {
    ChatFormattingData {
        enum_name,
        code,
        is_format,
        id,
        color,
    }
}

pub fn clean_name(name: &str) -> String {
    name.chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_ascii_lowercase())
        .collect()
}

pub fn strip_formatting(input: Option<&str>) -> Option<String> {
    input.map(strip_formatting_str)
}

pub fn strip_formatting_str(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character == CHAT_FORMATTING_PREFIX_CODE
            && chars.peek().copied().is_some_and(is_strip_formatting_code)
        {
            chars.next();
            continue;
        }
        output.push(character);
    }
    output
}

fn is_strip_formatting_code(code: char) -> bool {
    matches!(code.to_ascii_lowercase(), '0'..='9' | 'a'..='f' | 'k'..='o' | 'r')
}

#[cfg(test)]
mod tests {
    use super::{clean_name, strip_formatting, strip_formatting_str, ChatFormatting};

    #[test]
    fn chat_formatting_constants_match_java_table() {
        assert_eq!(ChatFormatting::VALUES.len(), 22);
        assert_eq!(ChatFormatting::Black.get_char(), '0');
        assert_eq!(ChatFormatting::Black.get_id(), 0);
        assert_eq!(ChatFormatting::Black.get_color(), Some(0));
        assert!(ChatFormatting::Black.is_color());
        assert!(!ChatFormatting::Black.is_format());

        assert_eq!(ChatFormatting::DarkPurple.get_color(), Some(11_141_290));
        assert_eq!(ChatFormatting::White.get_id(), 15);
        assert_eq!(ChatFormatting::White.get_color(), Some(16_777_215));
        assert_eq!(ChatFormatting::Obfuscated.get_char(), 'k');
        assert!(ChatFormatting::Obfuscated.is_format());
        assert!(!ChatFormatting::Obfuscated.is_color());
        assert_eq!(ChatFormatting::Reset.get_id(), -1);
        assert!(!ChatFormatting::Reset.is_color());
        assert!(!ChatFormatting::Reset.is_format());
    }

    #[test]
    fn chat_formatting_names_and_display_match_java() {
        assert_eq!(ChatFormatting::DarkBlue.get_name(), "dark_blue");
        assert_eq!(
            ChatFormatting::LightPurple.get_serialized_name(),
            "light_purple"
        );
        assert_eq!(ChatFormatting::Gold.to_string(), "§6");
        assert_eq!(ChatFormatting::Bold.to_string(), "§l");
    }

    #[test]
    fn chat_formatting_clean_name_and_lookups_match_java() {
        assert_eq!(clean_name("DARK_BLUE"), "darkblue");
        assert_eq!(clean_name("dark-blue 1!"), "darkblue");
        assert_eq!(
            ChatFormatting::get_by_name(Some("dark-blue")),
            Some(ChatFormatting::DarkBlue)
        );
        assert_eq!(
            ChatFormatting::get_by_name(Some("LIGHT_PURPLE")),
            Some(ChatFormatting::LightPurple)
        );
        assert_eq!(ChatFormatting::get_by_name(None), None);
        assert_eq!(ChatFormatting::get_by_name(Some("missing")), None);

        assert_eq!(ChatFormatting::get_by_id(-99), Some(ChatFormatting::Reset));
        assert_eq!(ChatFormatting::get_by_id(0), Some(ChatFormatting::Black));
        assert_eq!(ChatFormatting::get_by_id(15), Some(ChatFormatting::White));
        assert_eq!(ChatFormatting::get_by_id(16), None);

        assert_eq!(
            ChatFormatting::get_by_code('A'),
            Some(ChatFormatting::Green)
        );
        assert_eq!(
            ChatFormatting::get_by_code('r'),
            Some(ChatFormatting::Reset)
        );
        assert_eq!(
            ChatFormatting::get_by_code('R'),
            Some(ChatFormatting::Reset)
        );
        assert_eq!(ChatFormatting::get_by_code('x'), None);
    }

    #[test]
    fn chat_formatting_get_names_match_java_filters() {
        assert_eq!(ChatFormatting::get_names(false, false), vec!["reset"]);
        assert_eq!(
            ChatFormatting::get_names(true, false),
            vec![
                "black",
                "dark_blue",
                "dark_green",
                "dark_aqua",
                "dark_red",
                "dark_purple",
                "gold",
                "gray",
                "dark_gray",
                "blue",
                "green",
                "aqua",
                "red",
                "light_purple",
                "yellow",
                "white",
                "reset",
            ]
        );
        assert_eq!(
            ChatFormatting::get_names(false, true),
            vec![
                "obfuscated",
                "bold",
                "strikethrough",
                "underline",
                "italic",
                "reset",
            ]
        );
    }

    #[test]
    fn chat_formatting_strip_formatting_matches_java_pattern() {
        assert_eq!(strip_formatting(None), None);
        assert_eq!(
            strip_formatting(Some("A§0B§fC§Kx§Oy§Rz")),
            Some("ABCxyz".to_string())
        );
        assert_eq!(strip_formatting_str("A§xB§ C§"), "A§xB§ C§");
        assert_eq!(strip_formatting_str("A§gB§pC"), "A§gB§pC");
    }

    #[test]
    fn chat_formatting_color_codec_validation_matches_java() {
        assert_eq!(
            ChatFormatting::Red.validate_color_codec(),
            Ok(ChatFormatting::Red)
        );
        assert_eq!(
            ChatFormatting::Reset.validate_color_codec(),
            Ok(ChatFormatting::Reset)
        );
        assert_eq!(
            ChatFormatting::Bold.validate_color_codec(),
            Err("Formatting was not a valid color: §l".to_string())
        );
    }
}
