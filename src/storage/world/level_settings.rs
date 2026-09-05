//! The LevelSettings.parse fields shared by primary data and world summaries.
use super::*;

pub(super) struct ParsedLevelSettings {
    pub level_name: String,
    pub game_type: LevelGameType,
    pub difficulty: DifficultySettings,
    pub allow_commands: bool,
}

pub(super) fn parse(data: &[(String, Tag)]) -> ParsedLevelSettings {
    let field = |key: &str| {
        data.iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    };
    let game_type = field("GameType")
        .and_then(difficulty_settings::nbt_integer)
        .and_then(LevelGameType::from_id)
        .unwrap_or(LevelGameType::Survival);
    ParsedLevelSettings {
        level_name: compound_string(data, "LevelName").unwrap_or("").to_owned(),
        game_type,
        difficulty: field("difficulty_settings")
            .and_then(|tag| DifficultySettings::from_nbt(tag).ok())
            .unwrap_or_default(),
        allow_commands: field("allowCommands")
            .and_then(difficulty_settings::nbt_boolean)
            .unwrap_or(game_type == LevelGameType::Creative),
    }
}
