use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UuidArgumentModel;

impl UuidArgumentModel {
    pub fn uuid() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<UuidModel, UuidParseError> {
        let remaining = reader.remaining();
        let maybe_uuid = leading_allowed_uuid_characters(remaining);
        if !maybe_uuid.is_empty() {
            if let Some(result) = UuidModel::from_java_string(maybe_uuid) {
                reader.set_cursor(reader.cursor() + maybe_uuid.len());
                return Ok(result);
            }
        }

        Err(UuidParseError::InvalidUuid)
    }

    pub fn examples(&self) -> [&'static str; 1] {
        ["dd12be42-52a9-4a91-a8a1-11c01849e498"]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UuidModel {
    most_significant_bits: u64,
    least_significant_bits: u64,
}

impl UuidModel {
    pub fn from_java_string(input: &str) -> Option<Self> {
        let bytes = input.as_bytes();
        if bytes.len() != 36
            || bytes[8] != b'-'
            || bytes[13] != b'-'
            || bytes[18] != b'-'
            || bytes[23] != b'-'
        {
            return None;
        }

        let mut value = 0_u128;
        for (index, byte) in bytes.iter().copied().enumerate() {
            if matches!(index, 8 | 13 | 18 | 23) {
                continue;
            }
            let digit = hex_digit(byte)?;
            value = (value << 4) | u128::from(digit);
        }

        Some(Self {
            most_significant_bits: (value >> 64) as u64,
            least_significant_bits: value as u64,
        })
    }

    pub fn to_java_string(self) -> String {
        let value = (u128::from(self.most_significant_bits) << 64)
            | u128::from(self.least_significant_bits);
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (value >> 96) as u32,
            ((value >> 80) & 0xFFFF) as u16,
            ((value >> 64) & 0xFFFF) as u16,
            ((value >> 48) & 0xFFFF) as u16,
            value & 0x0000_FFFF_FFFF_FFFF
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, UuidModel>,
}

impl CommandContextModel {
    pub fn with_uuid(mut self, name: impl Into<String>, value: UuidModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_uuid(context: &CommandContextModel, name: &str) -> Option<UuidModel> {
    context.arguments.get(name).copied()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReaderModel {
    input: String,
    cursor: usize,
}

impl StringReaderModel {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn remaining(&self) -> &str {
        &self.input[self.cursor..]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UuidParseError {
    InvalidUuid,
}

fn leading_allowed_uuid_characters(input: &str) -> &str {
    let end = input
        .bytes()
        .position(|byte| !is_allowed_uuid_character(byte))
        .unwrap_or(input.len());
    &input[..end]
}

fn is_allowed_uuid_character(byte: u8) -> bool {
    matches!(byte, b'-' | b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "dd12be42-52a9-4a91-a8a1-11c01849e498";

    fn parse(input: &str) -> Result<(UuidModel, usize), UuidParseError> {
        let mut reader = StringReaderModel::new(input);
        let uuid = UuidArgumentModel::uuid().parse(&mut reader)?;
        Ok((uuid, reader.cursor()))
    }

    #[test]
    fn examples_match_java_examples() {
        assert_eq!(UuidArgumentModel::uuid().examples(), [EXAMPLE]);
    }

    #[test]
    fn get_uuid_returns_typed_context_argument() {
        let uuid = UuidModel::from_java_string(EXAMPLE).unwrap();
        let context = CommandContextModel::default().with_uuid("id", uuid);

        assert_eq!(get_uuid(&context, "id"), Some(uuid));
        assert_eq!(get_uuid(&context, "missing"), None);
    }

    #[test]
    fn parse_valid_uuid_advances_by_matched_uuid_length() {
        let (uuid, cursor) = parse("dd12be42-52a9-4a91-a8a1-11c01849e498 rest").unwrap();

        assert_eq!(uuid.to_java_string(), EXAMPLE);
        assert_eq!(cursor, 36);
    }

    #[test]
    fn parse_uppercase_uuid_matches_java_normalized_uuid_string() {
        let (uuid, cursor) = parse("DD12BE42-52A9-4A91-A8A1-11C01849E498!").unwrap();

        assert_eq!(uuid.to_java_string(), EXAMPLE);
        assert_eq!(cursor, 36);
    }

    #[test]
    fn leading_allowed_pattern_stops_before_non_uuid_characters() {
        let (uuid, cursor) = parse("dd12be42-52a9-4a91-a8a1-11c01849e498xyz").unwrap();

        assert_eq!(uuid.to_java_string(), EXAMPLE);
        assert_eq!(cursor, 36);
    }

    #[test]
    fn invalid_uuid_preserves_cursor() {
        let mut reader = StringReaderModel::new("dd12be42-52a9-4a91-a8a1-11c01849e498-tail");

        assert_eq!(
            UuidArgumentModel::uuid().parse(&mut reader),
            Err(UuidParseError::InvalidUuid)
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn missing_allowed_prefix_and_malformed_uuid_are_invalid() {
        assert_eq!(parse("not-a-uuid"), Err(UuidParseError::InvalidUuid));
        assert_eq!(parse(""), Err(UuidParseError::InvalidUuid));
        assert_eq!(
            parse("dd12be4252a94a91a8a111c01849e498"),
            Err(UuidParseError::InvalidUuid)
        );
    }

    #[test]
    fn uuid_model_splits_most_and_least_significant_bits_like_java_uuid() {
        let uuid = UuidModel::from_java_string(EXAMPLE).unwrap();

        assert_eq!(uuid.most_significant_bits, 0xdd12_be42_52a9_4a91);
        assert_eq!(uuid.least_significant_bits, 0xa8a1_11c0_1849_e498);
    }
}
