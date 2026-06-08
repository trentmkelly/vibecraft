use std::collections::{BTreeMap, HashMap};

use crate::command_selector::{
    EntityRecord, EntityWithPosition, Selector, SelectorBase, SelectorError, Vec3,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityArgumentModel {
    single: bool,
    players_only: bool,
}

impl EntityArgumentModel {
    pub fn entity() -> Self {
        Self::new(true, false)
    }

    pub fn entities() -> Self {
        Self::new(false, false)
    }

    pub fn player() -> Self {
        Self::new(true, true)
    }

    pub fn players() -> Self {
        Self::new(false, true)
    }

    fn new(single: bool, players_only: bool) -> Self {
        Self {
            single,
            players_only,
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
        allow_selectors: bool,
    ) -> Result<Selector, EntityArgumentError> {
        let token = reader.read_argument_token();
        if token.starts_with('@') && !allow_selectors {
            return Err(EntityArgumentError::SelectorsNotAllowed);
        }
        let selector = Selector::parse(&token).map_err(EntityArgumentError::Selector)?;
        if selector.limit > 1 && self.single {
            reader.set_cursor(0);
            return Err(if self.players_only {
                EntityArgumentError::NotSinglePlayer
            } else {
                EntityArgumentError::NotSingleEntity
            });
        }
        if selector.includes_entities
            && self.players_only
            && !selector.current_entity
            && !matches!(selector.base, SelectorBase::Name(_))
        {
            reader.set_cursor(0);
            return Err(EntityArgumentError::OnlyPlayersAllowed);
        }
        Ok(selector)
    }

    pub fn list_suggestions(&self, source: &CommandSourceModel, remaining: &str) -> Vec<String> {
        let CommandSourceModel::Shared(provider) = source else {
            return Vec::new();
        };

        let mut suggestions = if self.players_only {
            provider.online_player_names.clone()
        } else {
            provider
                .online_player_names
                .iter()
                .chain(provider.selected_entities.iter())
                .cloned()
                .collect::<Vec<_>>()
        };

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

    pub fn examples(&self) -> [&'static str; 5] {
        [
            "Player",
            "0123",
            "@e",
            "@e[type=foo]",
            "dd12be42-52a9-4a91-a8a1-11c01849e498",
        ]
    }

    pub fn info_template(&self) -> EntityArgumentInfoTemplate {
        EntityArgumentInfoTemplate {
            single: self.single,
            players_only: self.players_only,
        }
    }
}

pub fn get_entity(
    context: &CommandContextModel,
    name: &str,
) -> Result<EntityRecord, EntityArgumentError> {
    find_single_entity(context, argument(context, name)?)
}

pub fn get_entities(
    context: &CommandContextModel,
    name: &str,
) -> Result<Vec<EntityRecord>, EntityArgumentError> {
    let result = get_optional_entities(context, name)?;
    if result.is_empty() {
        Err(EntityArgumentError::NoEntitiesFound)
    } else {
        Ok(result)
    }
}

pub fn get_optional_entities(
    context: &CommandContextModel,
    name: &str,
) -> Result<Vec<EntityRecord>, EntityArgumentError> {
    find_entities(context, argument(context, name)?)
}

pub fn get_player(
    context: &CommandContextModel,
    name: &str,
) -> Result<EntityRecord, EntityArgumentError> {
    find_single_player(context, argument(context, name)?)
}

pub fn get_players(
    context: &CommandContextModel,
    name: &str,
) -> Result<Vec<EntityRecord>, EntityArgumentError> {
    let result = get_optional_players(context, name)?;
    if result.is_empty() {
        Err(EntityArgumentError::NoPlayersFound)
    } else {
        Ok(result)
    }
}

pub fn get_optional_players(
    context: &CommandContextModel,
    name: &str,
) -> Result<Vec<EntityRecord>, EntityArgumentError> {
    find_players(context, argument(context, name)?)
}

fn argument<'a>(
    context: &'a CommandContextModel,
    name: &str,
) -> Result<&'a Selector, EntityArgumentError> {
    context
        .arguments
        .get(name)
        .ok_or(EntityArgumentError::MissingArgument)
}

fn find_single_entity(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<EntityRecord, EntityArgumentError> {
    let entities = find_entities(context, selector)?;
    match entities.len() {
        0 => Err(EntityArgumentError::NoEntitiesFound),
        1 => Ok(entities[0].clone()),
        _ => Err(EntityArgumentError::NotSingleEntity),
    }
}

fn find_single_player(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<EntityRecord, EntityArgumentError> {
    let players = find_players(context, selector)?;
    if players.len() == 1 {
        Ok(players[0].clone())
    } else {
        Err(EntityArgumentError::NoPlayersFound)
    }
}

fn find_entities(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<Vec<EntityRecord>, EntityArgumentError> {
    check_selector_permission(context, selector)?;
    if !selector.includes_entities {
        return find_players(context, selector);
    }
    Ok(selector.select(
        &context.entities,
        context.source_position,
        &context.source_level,
        context.current_entity.as_deref(),
    ))
}

fn find_players(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<Vec<EntityRecord>, EntityArgumentError> {
    check_selector_permission(context, selector)?;
    Ok(selector
        .select(
            &context.entities,
            context.source_position,
            &context.source_level,
            context.current_entity.as_deref(),
        )
        .into_iter()
        .filter(|entity| entity.player)
        .collect())
}

fn check_selector_permission(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<(), EntityArgumentError> {
    if !matches!(
        selector.base,
        crate::command_selector::SelectorBase::Name(_)
    ) && !context.can_use_selectors
    {
        Err(EntityArgumentError::SelectorsNotAllowed)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandContextModel {
    arguments: HashMap<String, Selector>,
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
    pub fn with_argument(mut self, name: impl Into<String>, selector: Selector) -> Self {
        self.arguments.insert(name.into(), selector);
        self
    }

    pub fn with_entities(mut self, entities: Vec<EntityWithPosition>) -> Self {
        self.entities = entities;
        self
    }

    pub fn with_source_position(mut self, position: Vec3) -> Self {
        self.source_position = position;
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
    selected_entities: Vec<String>,
    can_use_selectors: bool,
}

impl SharedSuggestionProviderModel {
    pub fn new(can_use_selectors: bool) -> Self {
        Self {
            online_player_names: Vec::new(),
            selected_entities: Vec::new(),
            can_use_selectors,
        }
    }

    pub fn with_online_players(mut self, names: &[&str]) -> Self {
        self.online_player_names = names.iter().map(|name| (*name).to_string()).collect();
        self
    }

    pub fn with_selected_entities(mut self, names: &[&str]) -> Self {
        self.selected_entities = names.iter().map(|name| (*name).to_string()).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityArgumentInfoTemplate {
    single: bool,
    players_only: bool,
}

impl EntityArgumentInfoTemplate {
    pub fn serialize_to_network(&self) -> u8 {
        u8::from(self.single) | (u8::from(self.players_only) << 1)
    }

    pub fn deserialize_from_network(flags: u8) -> Self {
        Self {
            single: flags & 1 != 0,
            players_only: flags & 2 != 0,
        }
    }

    pub fn serialize_to_json(&self) -> (&'static str, &'static str) {
        (
            if self.single { "single" } else { "multiple" },
            if self.players_only {
                "players"
            } else {
                "entities"
            },
        )
    }

    pub fn instantiate(&self) -> EntityArgumentModel {
        EntityArgumentModel::new(self.single, self.players_only)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityArgumentError {
    Selector(SelectorError),
    NotSingleEntity,
    NotSinglePlayer,
    OnlyPlayersAllowed,
    NoEntitiesFound,
    NoPlayersFound,
    SelectorsNotAllowed,
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

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn read_argument_token(&mut self) -> String {
        while self.cursor < self.input.len()
            && self.input.as_bytes()[self.cursor].is_ascii_whitespace()
        {
            self.cursor += 1;
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(
        name: &str,
        uuid: &str,
        player: bool,
        entity_type: &str,
        x: f64,
    ) -> EntityWithPosition {
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
            position: Vec3 { x, y: 64.0, z: 0.0 },
        }
    }

    #[test]
    fn java_factories_examples_and_info_flags_match_source() {
        let entity = EntityArgumentModel::entity();
        let entities = EntityArgumentModel::entities();
        let player = EntityArgumentModel::player();
        let players = EntityArgumentModel::players();

        assert_eq!(
            entity.examples(),
            [
                "Player",
                "0123",
                "@e",
                "@e[type=foo]",
                "dd12be42-52a9-4a91-a8a1-11c01849e498"
            ]
        );
        assert_eq!(entity.info_template().serialize_to_network(), 1);
        assert_eq!(entities.info_template().serialize_to_network(), 0);
        assert_eq!(player.info_template().serialize_to_network(), 3);
        assert_eq!(players.info_template().serialize_to_network(), 2);
        assert_eq!(
            player.info_template().serialize_to_json(),
            ("single", "players")
        );
        assert_eq!(
            EntityArgumentInfoTemplate::deserialize_from_network(3)
                .instantiate()
                .info_template(),
            player.info_template()
        );
    }

    #[test]
    fn java_parse_applies_single_and_players_only_constraints() {
        let mut reader = StringReaderModel::new("@e");
        let error = EntityArgumentModel::entity().parse(&mut reader, true);
        assert_eq!(error, Err(EntityArgumentError::NotSingleEntity));
        assert_eq!(reader.cursor(), 0);

        let mut reader = StringReaderModel::new("@e");
        let error = EntityArgumentModel::player().parse(&mut reader, true);
        assert_eq!(error, Err(EntityArgumentError::NotSinglePlayer));
        assert_eq!(reader.cursor(), 0);

        let mut reader = StringReaderModel::new("@s rest");
        let selector = EntityArgumentModel::player()
            .parse(&mut reader, true)
            .unwrap_or_else(|error| panic!("unexpected parse error: {error:?}"));
        assert!(selector.current_entity);
        assert_eq!(reader.cursor(), 2);
    }

    #[test]
    fn java_parse_respects_source_selector_permission() {
        let mut denied = StringReaderModel::new("@p");
        assert_eq!(
            EntityArgumentModel::players().parse(&mut denied, false),
            Err(EntityArgumentError::SelectorsNotAllowed)
        );

        let mut name = StringReaderModel::new("Player next");
        let selector = EntityArgumentModel::player()
            .parse(&mut name, false)
            .unwrap_or_else(|error| panic!("unexpected parse error: {error:?}"));
        assert_eq!(selector.limit, 1);
        assert_eq!(name.cursor(), "Player".len());
    }

    #[test]
    fn java_getters_delegate_to_selector_find_methods_and_empty_errors() {
        let entities = vec![
            record("Alex", "u1", true, "minecraft:player", 1.0),
            record("Zombie", "u2", false, "minecraft:zombie", 2.0),
        ];
        let context = CommandContextModel::default()
            .with_entities(entities)
            .with_source_position(Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            })
            .with_argument(
                "one",
                Selector::parse("Alex")
                    .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
            )
            .with_argument(
                "many",
                Selector::parse("@e")
                    .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
            )
            .with_argument(
                "missing",
                Selector::parse("Missing")
                    .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
            );

        assert_eq!(
            get_entity(&context, "one")
                .unwrap_or_else(|error| panic!("unexpected getter error: {error:?}"))
                .name,
            "Alex"
        );
        assert_eq!(
            get_entity(&context, "many"),
            Err(EntityArgumentError::NotSingleEntity)
        );
        assert_eq!(
            get_entities(&context, "many")
                .unwrap_or_else(|error| panic!("unexpected getter error: {error:?}"))
                .len(),
            2
        );
        assert_eq!(
            get_entities(&context, "missing"),
            Err(EntityArgumentError::NoEntitiesFound)
        );
    }

    #[test]
    fn java_player_getters_filter_entities_and_require_non_empty_results() {
        let context = CommandContextModel::default()
            .with_entities(vec![
                record("Alex", "u1", true, "minecraft:player", 1.0),
                record("Zombie", "u2", false, "minecraft:zombie", 2.0),
            ])
            .with_current_entity("u1")
            .with_argument(
                "players",
                Selector::parse("@a")
                    .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
            )
            .with_argument(
                "self",
                Selector::parse("@s")
                    .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
            )
            .with_argument(
                "zombie",
                Selector::parse("Zombie")
                    .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
            );

        assert_eq!(
            get_player(&context, "players")
                .unwrap_or_else(|error| panic!("unexpected getter error: {error:?}"))
                .name,
            "Alex"
        );
        assert_eq!(
            get_players(&context, "players")
                .unwrap_or_else(|error| panic!("unexpected getter error: {error:?}"))
                .len(),
            1
        );
        assert_eq!(
            get_player(&context, "self")
                .unwrap_or_else(|error| panic!("unexpected getter error: {error:?}"))
                .name,
            "Alex"
        );
        assert_eq!(
            get_players(&context, "zombie"),
            Err(EntityArgumentError::NoPlayersFound)
        );
    }

    #[test]
    fn java_find_methods_recheck_selector_permissions() {
        let context = CommandContextModel::default()
            .with_selector_permission(false)
            .with_argument(
                "players",
                Selector::parse("@a")
                    .unwrap_or_else(|error| panic!("unexpected selector error: {error:?}")),
            );

        assert_eq!(
            get_optional_players(&context, "players"),
            Err(EntityArgumentError::SelectorsNotAllowed)
        );
    }

    #[test]
    fn java_suggestions_use_shared_provider_names_entities_and_selector_permission() {
        let provider = SharedSuggestionProviderModel::new(true)
            .with_online_players(&["Alex", "Steve"])
            .with_selected_entities(&["Zombie"]);
        let source = CommandSourceModel::Shared(provider);

        assert_eq!(
            EntityArgumentModel::players().list_suggestions(&source, "A"),
            vec!["Alex".to_string()]
        );
        assert_eq!(
            EntityArgumentModel::entities().list_suggestions(&source, "Z"),
            vec!["Zombie".to_string()]
        );
        assert_eq!(
            EntityArgumentModel::entities().list_suggestions(&source, "@"),
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
        assert!(EntityArgumentModel::entities()
            .list_suggestions(&source, "@")
            .is_empty());
        assert!(EntityArgumentModel::entities()
            .list_suggestions(&CommandSourceModel::Other, "")
            .is_empty());
    }
}
