use std::collections::{BTreeMap, HashMap};

use crate::command_selector::{
    EntityRecord, EntityWithPosition, Selector, SelectorBase, SelectorError, Vec3,
};
use crate::player_access::NameAndId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfileArgumentModel;

impl GameProfileArgumentModel {
    pub fn game_profile() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
        allow_selectors: bool,
    ) -> Result<GameProfileResultModel, GameProfileArgumentError> {
        if reader.can_read() && reader.peek() == '@' {
            if !allow_selectors {
                return Err(GameProfileArgumentError::SelectorsNotAllowed);
            }
            let token = reader.read_argument_token();
            let selector = Selector::parse(&token).map_err(GameProfileArgumentError::Selector)?;
            if selector.includes_entities {
                return Err(GameProfileArgumentError::OnlyPlayersAllowed);
            }
            Ok(GameProfileResultModel::Selector(Box::new(selector)))
        } else {
            Ok(GameProfileResultModel::Name(reader.read_until_space()))
        }
    }

    pub fn list_suggestions(&self, source: &CommandSourceModel, remaining: &str) -> Vec<String> {
        let CommandSourceModel::Shared(provider) = source else {
            return Vec::new();
        };

        let mut suggestions = provider.online_player_names.clone();
        if provider.can_use_selectors {
            suggestions.extend(
                ["@p", "@a", "@r", "@s", "@e", "@n"]
                    .iter()
                    .map(ToString::to_string),
            );
        }
        suggestions
            .into_iter()
            .filter(|suggestion| suggestion.starts_with(remaining))
            .collect()
    }

    pub fn examples(&self) -> [&'static str; 4] {
        [
            "Player",
            "0123",
            "dd12be42-52a9-4a91-a8a1-11c01849e498",
            "@e",
        ]
    }
}

pub fn get_game_profiles(
    context: &CommandContextModel,
    name: &str,
) -> Result<Vec<NameAndId>, GameProfileArgumentError> {
    context
        .arguments
        .get(name)
        .ok_or(GameProfileArgumentError::MissingArgument)?
        .get_names(context)
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameProfileResultModel {
    Name(String),
    Selector(Box<Selector>),
}

impl GameProfileResultModel {
    fn get_names(
        &self,
        context: &CommandContextModel,
    ) -> Result<Vec<NameAndId>, GameProfileArgumentError> {
        match self {
            Self::Name(name) => context
                .name_to_id_cache
                .get(name)
                .cloned()
                .map(|profile| vec![profile])
                .ok_or(GameProfileArgumentError::UnknownPlayer),
            Self::Selector(selector) => {
                check_selector_permission(context, selector)?;
                let players = selector
                    .select(
                        &context.entities,
                        context.source_position,
                        &context.source_level,
                        context.current_entity.as_deref(),
                    )
                    .into_iter()
                    .filter(|entity| entity.player)
                    .map(|entity| NameAndId {
                        uuid: entity.uuid,
                        name: entity.name,
                    })
                    .collect::<Vec<_>>();
                if players.is_empty() {
                    Err(GameProfileArgumentError::NoPlayersFound)
                } else {
                    Ok(players)
                }
            }
        }
    }
}

fn check_selector_permission(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<(), GameProfileArgumentError> {
    if !matches!(selector.base, SelectorBase::Name(_)) && !context.can_use_selectors {
        Err(GameProfileArgumentError::SelectorsNotAllowed)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandContextModel {
    arguments: HashMap<String, GameProfileResultModel>,
    name_to_id_cache: HashMap<String, NameAndId>,
    entities: Vec<EntityWithPosition>,
    source_position: Vec3,
    source_level: String,
    current_entity: Option<String>,
    can_use_selectors: bool,
}

impl Default for CommandContextModel {
    fn default() -> Self {
        Self {
            arguments: HashMap::new(),
            name_to_id_cache: HashMap::new(),
            entities: Vec::new(),
            source_position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            source_level: "overworld".to_string(),
            current_entity: None,
            can_use_selectors: true,
        }
    }
}

impl CommandContextModel {
    pub fn with_argument(
        mut self,
        name: impl Into<String>,
        result: GameProfileResultModel,
    ) -> Self {
        self.arguments.insert(name.into(), result);
        self
    }

    pub fn with_cached_profile(mut self, profile: NameAndId) -> Self {
        self.name_to_id_cache.insert(profile.name.clone(), profile);
        self
    }

    pub fn with_entities(mut self, entities: Vec<EntityWithPosition>) -> Self {
        self.entities = entities;
        self
    }

    pub fn with_current_entity(mut self, uuid: impl Into<String>) -> Self {
        self.current_entity = Some(uuid.into());
        self
    }

    pub fn with_selector_permission(mut self, allowed: bool) -> Self {
        self.can_use_selectors = allowed;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CommandSourceModel {
    Shared(SharedSuggestionProviderModel),
    #[default]
    Other,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SharedSuggestionProviderModel {
    online_player_names: Vec<String>,
    can_use_selectors: bool,
}

impl SharedSuggestionProviderModel {
    pub fn new(can_use_selectors: bool) -> Self {
        Self {
            online_player_names: Vec::new(),
            can_use_selectors,
        }
    }

    pub fn with_online_players(mut self, names: &[&str]) -> Self {
        self.online_player_names = names.iter().map(|name| (*name).to_string()).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameProfileArgumentError {
    Selector(SelectorError),
    OnlyPlayersAllowed,
    SelectorsNotAllowed,
    UnknownPlayer,
    NoPlayersFound,
    MissingArgument,
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

    fn read_argument_token(&mut self) -> String {
        let start = self.cursor;
        let mut bracket_depth = 0usize;
        while self.cursor < self.input.len() {
            let ch = self.input.as_bytes()[self.cursor] as char;
            match ch {
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                ch if ch.is_ascii_whitespace() && bracket_depth == 0 => break,
                _ => {}
            }
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }

    fn read_until_space(&mut self) -> String {
        let start = self.cursor;
        while self.cursor < self.input.len() && self.peek() != ' ' {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(name: &str, uuid: &str) -> NameAndId {
        NameAndId {
            name: name.to_string(),
            uuid: uuid.to_string(),
        }
    }

    fn player(name: &str, uuid: &str) -> EntityWithPosition {
        entity(name, uuid, true, "minecraft:player")
    }

    fn entity(name: &str, uuid: &str, player: bool, entity_type: &str) -> EntityWithPosition {
        EntityWithPosition {
            entity: EntityRecord {
                name: name.to_string(),
                uuid: uuid.to_string(),
                player,
                entity_type: entity_type.to_string(),
                level: "overworld".to_string(),
                team: None,
                tags: Vec::new(),
                scores: BTreeMap::new(),
                nbt: BTreeMap::new(),
                predicates: Vec::new(),
                gamemode: None,
                experience_level: 0,
                x_rotation: 0.0,
                y_rotation: 0.0,
                advancements: BTreeMap::new(),
            },
            position: Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
        }
    }

    #[test]
    fn java_factory_examples_and_raw_name_parse_match_source() {
        let argument = GameProfileArgumentModel::game_profile();
        assert_eq!(
            argument.examples(),
            [
                "Player",
                "0123",
                "dd12be42-52a9-4a91-a8a1-11c01849e498",
                "@e"
            ]
        );

        let mut reader = StringReaderModel::new("Player trailing");
        let result = argument
            .parse(&mut reader, true)
            .unwrap_or_else(|error| panic!("unexpected parse error: {error:?}"));

        assert_eq!(result, GameProfileResultModel::Name("Player".to_string()));
        assert_eq!(reader.cursor(), "Player".len());
    }

    #[test]
    fn java_name_result_resolves_against_name_to_id_cache_lazily() {
        let result = GameProfileResultModel::Name("Alex".to_string());
        let context = CommandContextModel::default()
            .with_argument("target", result)
            .with_cached_profile(profile("Alex", "u1"));

        assert_eq!(
            get_game_profiles(&context, "target"),
            Ok(vec![profile("Alex", "u1")])
        );

        let missing = CommandContextModel::default().with_argument(
            "target",
            GameProfileResultModel::Name("Missing".to_string()),
        );
        assert_eq!(
            get_game_profiles(&missing, "target"),
            Err(GameProfileArgumentError::UnknownPlayer)
        );
    }

    #[test]
    fn java_selector_parse_requires_player_selector() {
        let argument = GameProfileArgumentModel::game_profile();

        let mut entities = StringReaderModel::new("@e");
        assert_eq!(
            argument.parse(&mut entities, true),
            Err(GameProfileArgumentError::OnlyPlayersAllowed)
        );

        let mut denied = StringReaderModel::new("@a");
        assert_eq!(
            argument.parse(&mut denied, false),
            Err(GameProfileArgumentError::SelectorsNotAllowed)
        );

        let mut players = StringReaderModel::new("@a next");
        let result = argument
            .parse(&mut players, true)
            .unwrap_or_else(|error| panic!("unexpected parse error: {error:?}"));
        assert!(matches!(result, GameProfileResultModel::Selector(_)));
        assert_eq!(players.cursor(), 2);
    }

    #[test]
    fn java_selector_result_returns_player_name_and_id_values() {
        let selector = GameProfileResultModel::Selector(Box::new(
            Selector::parse("@a")
                .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
        ));
        let context = CommandContextModel::default()
            .with_argument("targets", selector)
            .with_entities(vec![
                player("Alex", "u1"),
                player("Steve", "u2"),
                entity("Zombie", "u3", false, "minecraft:zombie"),
            ]);

        assert_eq!(
            get_game_profiles(&context, "targets"),
            Ok(vec![profile("Alex", "u1"), profile("Steve", "u2")])
        );
    }

    #[test]
    fn java_selector_result_errors_for_empty_or_permission_denied_players() {
        let selector = Selector::parse("@a")
            .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}"));
        let empty = CommandContextModel::default().with_argument(
            "targets",
            GameProfileResultModel::Selector(Box::new(selector.clone())),
        );
        assert_eq!(
            get_game_profiles(&empty, "targets"),
            Err(GameProfileArgumentError::NoPlayersFound)
        );

        let denied = CommandContextModel::default()
            .with_selector_permission(false)
            .with_argument(
                "targets",
                GameProfileResultModel::Selector(Box::new(selector)),
            );
        assert_eq!(
            get_game_profiles(&denied, "targets"),
            Err(GameProfileArgumentError::SelectorsNotAllowed)
        );
    }

    #[test]
    fn java_self_selector_uses_current_source_player() {
        let selector = GameProfileResultModel::Selector(Box::new(
            Selector::parse("@s")
                .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
        ));
        let context = CommandContextModel::default()
            .with_argument("target", selector)
            .with_current_entity("u1")
            .with_entities(vec![player("Alex", "u1"), player("Steve", "u2")]);

        assert_eq!(
            get_game_profiles(&context, "target"),
            Ok(vec![profile("Alex", "u1")])
        );
    }

    #[test]
    fn java_suggestions_use_shared_online_names_and_selector_permission() {
        let source = CommandSourceModel::Shared(
            SharedSuggestionProviderModel::new(true).with_online_players(&["Alex", "Steve"]),
        );
        assert_eq!(
            GameProfileArgumentModel::game_profile().list_suggestions(&source, "A"),
            vec!["Alex".to_string()]
        );
        assert_eq!(
            GameProfileArgumentModel::game_profile().list_suggestions(&source, "@"),
            vec![
                "@p".to_string(),
                "@a".to_string(),
                "@r".to_string(),
                "@s".to_string(),
                "@e".to_string(),
                "@n".to_string()
            ]
        );

        let source = CommandSourceModel::Shared(
            SharedSuggestionProviderModel::new(false).with_online_players(&["Alex"]),
        );
        assert!(GameProfileArgumentModel::game_profile()
            .list_suggestions(&source, "@")
            .is_empty());
        assert!(GameProfileArgumentModel::game_profile()
            .list_suggestions(&CommandSourceModel::Other, "")
            .is_empty());
    }
}
