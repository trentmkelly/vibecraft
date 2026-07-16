use std::collections::HashMap;

use crate::chat_component::Component;
use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameModeArgumentModel;

impl GameModeArgumentModel {
    pub fn game_mode() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<GameTypeModel, GameModeParseError> {
        let game_type_string = reader.read_unquoted_string();
        GameTypeModel::by_name(&game_type_string, None).ok_or(GameModeParseError::Invalid {
            value: game_type_string,
        })
    }

    pub fn list_suggestions(
        &self,
        source: &CommandSourceModel,
        builder: &mut SuggestionsBuilderModel,
    ) -> Vec<SuggestionModel> {
        if source.shared_suggestion_provider {
            suggest(
                GameTypeModel::VALUES
                    .into_iter()
                    .map(|game_type| game_type.name().to_string()),
                builder,
            );
            builder.clone().build()
        } else {
            Vec::new()
        }
    }

    pub fn examples(&self) -> [&'static str; 2] {
        [
            GameTypeModel::Survival.name(),
            GameTypeModel::Creative.name(),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameTypeModel {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameTypeModel {
    pub const DEFAULT_MODE: Self = Self::Survival;
    pub const VALUES: [Self; 4] = [
        Self::Survival,
        Self::Creative,
        Self::Adventure,
        Self::Spectator,
    ];

    pub fn id(self) -> i32 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Adventure => "adventure",
            Self::Spectator => "spectator",
        }
    }

    pub fn serialized_name(self) -> &'static str {
        self.name()
    }

    pub fn short_display_key(self) -> String {
        format!("selectWorld.gameMode.{}", self.name())
    }

    pub fn long_display_key(self) -> String {
        format!("gameMode.{}", self.name())
    }

    pub fn short_display_name(self) -> Component {
        Component::translatable(self.short_display_key(), Vec::new())
    }

    pub fn long_display_name(self) -> Component {
        Component::translatable(self.long_display_key(), Vec::new())
    }

    pub fn by_id(id: i32) -> Self {
        Self::VALUES
            .into_iter()
            .find(|game_type| game_type.id() == id)
            .unwrap_or(Self::Survival)
    }

    pub fn by_name(name: &str, default_mode: Option<Self>) -> Option<Self> {
        Self::VALUES
            .into_iter()
            .find(|game_type| game_type.name() == name)
            .or(default_mode)
    }

    pub fn nullable_id(game_type: Option<Self>) -> i32 {
        game_type.map_or(-1, Self::id)
    }

    pub fn by_nullable_id(id: i32) -> Option<Self> {
        (id != -1).then(|| Self::by_id(id))
    }

    pub fn is_valid_id(id: i32) -> bool {
        Self::VALUES.iter().any(|game_type| game_type.id() == id)
    }

    pub fn is_block_placing_restricted(self) -> bool {
        matches!(self, Self::Adventure | Self::Spectator)
    }

    pub fn is_creative(self) -> bool {
        self == Self::Creative
    }

    pub fn is_survival(self) -> bool {
        matches!(self, Self::Survival | Self::Adventure)
    }

    pub fn update_player_abilities(self, abilities: &mut AbilitiesModel) {
        match self {
            Self::Creative => {
                abilities.mayfly = true;
                abilities.instabuild = true;
                abilities.invulnerable = true;
            }
            Self::Spectator => {
                abilities.mayfly = true;
                abilities.instabuild = false;
                abilities.invulnerable = true;
                abilities.flying = true;
            }
            Self::Survival | Self::Adventure => {
                abilities.mayfly = false;
                abilities.instabuild = false;
                abilities.invulnerable = false;
                abilities.flying = false;
            }
        }
        abilities.may_build = !self.is_block_placing_restricted();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbilitiesModel {
    pub mayfly: bool,
    pub instabuild: bool,
    pub invulnerable: bool,
    pub flying: bool,
    pub may_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

impl Default for AbilitiesModel {
    fn default() -> Self {
        Self {
            mayfly: false,
            instabuild: false,
            invulnerable: false,
            flying: false,
            may_build: true,
            flying_speed: 0.05,
            walking_speed: 0.1,
        }
    }
}

impl AbilitiesModel {
    pub fn get_flying_speed(&self) -> f32 {
        self.flying_speed
    }

    pub fn set_flying_speed(&mut self, value: f32) {
        self.flying_speed = value;
    }

    pub fn get_walking_speed(&self) -> f32 {
        self.walking_speed
    }

    pub fn set_walking_speed(&mut self, value: f32) {
        self.walking_speed = value;
    }

    pub fn pack(&self) -> AbilitiesPacked {
        AbilitiesPacked {
            invulnerable: self.invulnerable,
            flying: self.flying,
            may_fly: self.mayfly,
            instabuild: self.instabuild,
            may_build: self.may_build,
            flying_speed: self.flying_speed,
            walking_speed: self.walking_speed,
        }
    }

    pub fn apply(&mut self, packed: &AbilitiesPacked) {
        self.invulnerable = packed.invulnerable;
        self.flying = packed.flying;
        self.mayfly = packed.may_fly;
        self.instabuild = packed.instabuild;
        self.may_build = packed.may_build;
        self.flying_speed = packed.flying_speed;
        self.walking_speed = packed.walking_speed;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbilitiesPacked {
    pub invulnerable: bool,
    pub flying: bool,
    pub may_fly: bool,
    pub instabuild: bool,
    pub may_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

impl AbilitiesPacked {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(&serde_json::json!({
            "invulnerable": self.invulnerable,
            "flying": self.flying,
            "mayfly": self.may_fly,
            "instabuild": self.instabuild,
            "mayBuild": self.may_build,
            "flySpeed": self.flying_speed,
            "walkSpeed": self.walking_speed,
        }))
        .map_err(|error| error.to_string())
    }

    pub fn from_json(raw: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|error| format!("invalid Abilities.Packed JSON: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| "Abilities.Packed must be a JSON object".to_string())?;
        let bool_value = |name: &str, default: bool| {
            object.get(name).map_or(Ok(default), |value| {
                value
                    .as_bool()
                    .ok_or_else(|| format!("Abilities.Packed {name} must be boolean"))
            })
        };
        let float_value = |name: &str, default: f32| {
            object.get(name).map_or(Ok(default), |value| {
                value
                    .as_f64()
                    .filter(|value| value.is_finite())
                    .map(|value| value as f32)
                    .ok_or_else(|| format!("Abilities.Packed {name} must be finite number"))
            })
        };
        Ok(Self {
            invulnerable: bool_value("invulnerable", false)?,
            flying: bool_value("flying", false)?,
            may_fly: bool_value("mayfly", false)?,
            instabuild: bool_value("instabuild", false)?,
            may_build: bool_value("mayBuild", true)?,
            flying_speed: float_value("flySpeed", 0.05)?,
            walking_speed: float_value("walkSpeed", 0.1)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSourceModel {
    pub shared_suggestion_provider: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, GameTypeModel>,
}

impl CommandContextModel {
    pub fn with_game_mode(mut self, name: impl Into<String>, value: GameTypeModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_game_mode(context: &CommandContextModel, name: &str) -> Option<GameTypeModel> {
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

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && !self.peek().is_ascii_whitespace() {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameModeParseError {
    Invalid { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(GameTypeModel, usize), GameModeParseError> {
        let mut reader = StringReaderModel::new(input);
        let game_type = GameModeArgumentModel::game_mode().parse(&mut reader)?;
        Ok((game_type, reader.cursor()))
    }

    #[test]
    fn game_type_metadata_matches_java_enum() {
        assert_eq!(
            GameTypeModel::VALUES.map(|game_type| (game_type.id(), game_type.name())),
            [
                (0, "survival"),
                (1, "creative"),
                (2, "adventure"),
                (3, "spectator"),
            ]
        );
        assert_eq!(GameTypeModel::DEFAULT_MODE, GameTypeModel::Survival);
        assert_eq!(
            GameTypeModel::Creative.short_display_key(),
            "selectWorld.gameMode.creative"
        );
        assert_eq!(
            GameTypeModel::Spectator.long_display_key(),
            "gameMode.spectator"
        );
    }

    #[test]
    fn game_type_lookup_helpers_match_java_defaults() {
        assert_eq!(GameTypeModel::by_id(-99), GameTypeModel::Survival);
        assert_eq!(GameTypeModel::by_id(2), GameTypeModel::Adventure);
        assert_eq!(GameTypeModel::by_id(99), GameTypeModel::Survival);
        assert_eq!(
            GameTypeModel::by_name("creative", Some(GameTypeModel::Survival)),
            Some(GameTypeModel::Creative)
        );
        assert_eq!(
            GameTypeModel::by_name("missing", Some(GameTypeModel::Survival)),
            Some(GameTypeModel::Survival)
        );
        assert_eq!(GameTypeModel::by_name("missing", None), None);
        assert_eq!(GameTypeModel::nullable_id(None), -1);
        assert_eq!(
            GameTypeModel::nullable_id(Some(GameTypeModel::Spectator)),
            3
        );
        assert_eq!(GameTypeModel::by_nullable_id(-1), None);
        assert_eq!(
            GameTypeModel::by_nullable_id(42),
            Some(GameTypeModel::Survival)
        );
        assert!(GameTypeModel::is_valid_id(3));
        assert!(!GameTypeModel::is_valid_id(4));
        assert_eq!(GameTypeModel::Creative.serialized_name(), "creative");
        assert_eq!(
            GameTypeModel::Creative.short_display_name().get_string(),
            "selectWorld.gameMode.creative"
        );
        assert_eq!(
            GameTypeModel::Spectator.long_display_name().get_string(),
            "gameMode.spectator"
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn game_type_source_matches_java_26_1_2() {
        const JAVA_SOURCE: &str =
            vibecraft_java_source!("/net/minecraft/world/level/GameType.java");
        for fragment in [
            "SURVIVAL(0, \"survival\")",
            "CREATIVE(1, \"creative\")",
            "ADVENTURE(2, \"adventure\")",
            "SPECTATOR(3, \"spectator\")",
            "ByIdMap.continuous(GameType::getId, values(), ByIdMap.OutOfBoundsStrategy.ZERO)",
            "public Component getLongDisplayName()",
            "Component.translatable(\"gameMode.\" + name)",
            "public Component getShortDisplayName()",
            "Component.translatable(\"selectWorld.gameMode.\" + name)",
            "public void updatePlayerAbilities(final Abilities abilities)",
            "public boolean isBlockPlacingRestricted()",
            "public boolean isCreative()",
            "public boolean isSurvival()",
            "public static @Nullable GameType byNullableId(final int id)",
            "public static boolean isValidId(final int id)",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn game_type_ability_updates_match_java() {
        let mut creative = AbilitiesModel::default();
        GameTypeModel::Creative.update_player_abilities(&mut creative);
        assert_eq!(
            creative,
            AbilitiesModel {
                mayfly: true,
                instabuild: true,
                invulnerable: true,
                flying: false,
                may_build: true,
                flying_speed: 0.05,
                walking_speed: 0.1,
            }
        );

        let mut spectator = AbilitiesModel::default();
        GameTypeModel::Spectator.update_player_abilities(&mut spectator);
        assert_eq!(
            spectator,
            AbilitiesModel {
                mayfly: true,
                instabuild: false,
                invulnerable: true,
                flying: true,
                may_build: false,
                flying_speed: 0.05,
                walking_speed: 0.1,
            }
        );

        let mut adventure = AbilitiesModel {
            mayfly: true,
            instabuild: true,
            invulnerable: true,
            flying: true,
            may_build: true,
            flying_speed: 0.05,
            walking_speed: 0.1,
        };
        GameTypeModel::Adventure.update_player_abilities(&mut adventure);
        assert_eq!(
            adventure,
            AbilitiesModel {
                mayfly: false,
                instabuild: false,
                invulnerable: false,
                flying: false,
                may_build: false,
                flying_speed: 0.05,
                walking_speed: 0.1,
            }
        );
        assert!(GameTypeModel::Creative.is_creative());
        assert!(GameTypeModel::Adventure.is_survival());
        assert!(GameTypeModel::Spectator.is_block_placing_restricted());
    }

    #[test]
    fn abilities_pack_apply_and_codec_match_java_defaults() {
        let abilities = AbilitiesModel {
            invulnerable: true,
            flying: true,
            mayfly: true,
            instabuild: true,
            may_build: false,
            ..AbilitiesModel::default()
        };
        let mut abilities = abilities;
        abilities.set_flying_speed(0.2);
        abilities.set_walking_speed(0.15);
        assert_eq!(abilities.get_flying_speed(), 0.2);
        assert_eq!(abilities.get_walking_speed(), 0.15);
        let packed = abilities.pack();
        assert_eq!(packed.flying_speed, 0.2);
        assert_eq!(packed.walking_speed, 0.15);

        let encoded = packed.to_json().unwrap();
        assert_eq!(AbilitiesPacked::from_json(&encoded).unwrap(), packed);
        assert_eq!(
            AbilitiesPacked::from_json("{}").unwrap(),
            AbilitiesPacked {
                invulnerable: false,
                flying: false,
                may_fly: false,
                instabuild: false,
                may_build: true,
                flying_speed: 0.05,
                walking_speed: 0.1,
            }
        );
        assert!(AbilitiesPacked::from_json(r#"{"flySpeed":"fast"}"#).is_err());

        let mut applied = AbilitiesModel::default();
        applied.apply(&packed);
        assert_eq!(applied, abilities);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn abilities_source_matches_java_26_1_2() {
        const JAVA_SOURCE: &str =
            vibecraft_java_source!("/net/minecraft/world/entity/player/Abilities.java");
        for fragment in [
            "private static final boolean DEFAULT_INVULNERABLE = false",
            "private static final float DEFAULT_FLYING_SPEED = 0.05F",
            "public float getFlyingSpeed()",
            "public void setFlyingSpeed(final float value)",
            "public float getWalkingSpeed()",
            "public void setWalkingSpeed(final float value)",
            "public Abilities.Packed pack()",
            "public void apply(final Abilities.Packed packed)",
            "public record Packed(boolean invulnerable, boolean flying, boolean mayFly, boolean instabuild, boolean mayBuild, float flyingSpeed, float walkingSpeed)",
            "Codec.BOOL.fieldOf(\"mayBuild\").orElse(true)",
            "Codec.FLOAT.fieldOf(\"flySpeed\").orElse(0.05F)",
            "Codec.FLOAT.fieldOf(\"walkSpeed\").orElse(0.1F)",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn examples_are_survival_and_creative_names() {
        assert_eq!(
            GameModeArgumentModel::game_mode().examples(),
            ["survival", "creative"]
        );
    }

    #[test]
    fn get_game_mode_returns_typed_context_argument() {
        let context =
            CommandContextModel::default().with_game_mode("mode", GameTypeModel::Creative);

        assert_eq!(
            get_game_mode(&context, "mode"),
            Some(GameTypeModel::Creative)
        );
        assert_eq!(get_game_mode(&context, "missing"), None);
    }

    #[test]
    fn parse_uses_exact_by_name_lookup_and_advances_unquoted_token() {
        let (game_type, cursor) = parse("spectator tail").unwrap();

        assert_eq!(game_type, GameTypeModel::Spectator);
        assert_eq!(cursor, 9);
        assert_eq!(parse("creative"), Ok((GameTypeModel::Creative, 8)));
    }

    #[test]
    fn parse_rejects_unknown_case_mismatched_and_empty_values_without_cursor_reset() {
        let mut reader = StringReaderModel::new("Creative rest");
        assert_eq!(
            GameModeArgumentModel::game_mode().parse(&mut reader),
            Err(GameModeParseError::Invalid {
                value: "Creative".to_string(),
            })
        );
        assert_eq!(reader.cursor(), 8);

        assert_eq!(
            parse("missing"),
            Err(GameModeParseError::Invalid {
                value: "missing".to_string(),
            })
        );
        assert_eq!(
            parse(""),
            Err(GameModeParseError::Invalid {
                value: String::new(),
            })
        );
    }

    #[test]
    fn suggestions_are_source_gated_and_use_shared_matching() {
        let source = CommandSourceModel {
            shared_suggestion_provider: true,
        };
        let mut all = SuggestionsBuilderModel::new("");
        assert_eq!(
            GameModeArgumentModel::game_mode()
                .list_suggestions(&source, &mut all)
                .into_iter()
                .map(|suggestion| suggestion.value)
                .collect::<Vec<_>>(),
            vec!["survival", "creative", "adventure", "spectator"]
        );

        let mut creative = SuggestionsBuilderModel::new("c");
        assert_eq!(
            GameModeArgumentModel::game_mode().list_suggestions(&source, &mut creative),
            vec![SuggestionModel {
                value: "creative".to_string(),
                tooltip: None,
            }]
        );

        let non_shared_source = CommandSourceModel {
            shared_suggestion_provider: false,
        };
        let mut builder = SuggestionsBuilderModel::new("");
        assert!(GameModeArgumentModel::game_mode()
            .list_suggestions(&non_shared_source, &mut builder)
            .is_empty());
    }
}
