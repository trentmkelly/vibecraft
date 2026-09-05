//! Java LevelSettings.DifficultySettings's required NBT record fields.
use super::LevelDifficulty;
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifficultySettings {
    pub difficulty: LevelDifficulty,
    pub hardcore: bool,
    pub locked: bool,
}

impl Default for DifficultySettings {
    fn default() -> Self {
        Self {
            difficulty: LevelDifficulty::Normal,
            hardcore: false,
            locked: false,
        }
    }
}

impl DifficultySettings {
    pub fn to_nbt(self) -> Tag {
        Tag::Compound(vec![
            (
                "difficulty".to_owned(),
                Tag::String(self.difficulty.serialized_name().to_owned()),
            ),
            ("hardcore".to_owned(), Tag::Byte(i8::from(self.hardcore))),
            ("locked".to_owned(), Tag::Byte(i8::from(self.locked))),
        ])
    }

    /// All fields are required. LevelSettings.parse defaults the entire record
    /// if any one field is missing or malformed, rather than mixing defaults.
    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let Tag::Compound(fields) = tag else {
            return Err("difficulty_settings must be a compound".to_owned());
        };
        let get = |key: &str| {
            fields
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value)
        };
        let Some(Tag::String(difficulty)) = get("difficulty") else {
            return Err("difficulty must be a serialized name".to_owned());
        };
        Ok(Self {
            difficulty: LevelDifficulty::from_name(difficulty)
                .ok_or_else(|| format!("unknown difficulty: {difficulty}"))?,
            hardcore: get("hardcore")
                .and_then(nbt_boolean)
                .ok_or_else(|| "hardcore must be numeric".to_owned())?,
            locked: get("locked")
                .and_then(nbt_boolean)
                .ok_or_else(|| "locked must be numeric".to_owned())?,
        })
    }
}

impl LevelDifficulty {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "peaceful" => Some(Self::Peaceful),
            "easy" => Some(Self::Easy),
            "normal" => Some(Self::Normal),
            "hard" => Some(Self::Hard),
            _ => None,
        }
    }
}

/// DynamicOps.getBooleanValue uses Number.byteValue, not a general nonzero test.
/// Float/double narrowing goes through int before byte in Java.
pub(super) fn nbt_boolean(tag: &Tag) -> Option<bool> {
    let byte = match tag {
        Tag::Byte(value) => *value,
        Tag::Short(value) => *value as i8,
        Tag::Int(value) => *value as i8,
        Tag::Long(value) => *value as i8,
        Tag::Float(value) => *value as i32 as i8,
        Tag::Double(value) => *value as i32 as i8,
        _ => return None,
    };
    Some(byte != 0)
}

/// Dynamic.asInt delegates to Number.intValue for all numeric NBT tags.
pub(super) fn nbt_integer(tag: &Tag) -> Option<i32> {
    match tag {
        Tag::Byte(value) => Some(i32::from(*value)),
        Tag::Short(value) => Some(i32::from(*value)),
        Tag::Int(value) => Some(*value),
        Tag::Long(value) => Some(*value as i32),
        Tag::Float(value) => Some(*value as i32),
        Tag::Double(value) => Some(*value as i32),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
