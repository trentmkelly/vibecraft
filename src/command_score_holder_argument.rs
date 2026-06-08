use std::collections::{BTreeMap, HashMap};

use crate::command_selector::{
    EntityRecord, EntityWithPosition, Selector, SelectorBase, SelectorError, Vec3,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreHolderArgumentModel {
    multiple: bool,
}

impl ScoreHolderArgumentModel {
    pub fn score_holder() -> Self {
        Self { multiple: false }
    }

    pub fn score_holders() -> Self {
        Self { multiple: true }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
        allow_selectors: bool,
    ) -> Result<ScoreHolderResultModel, ScoreHolderArgumentError> {
        if reader.can_read() && reader.peek() == '@' {
            if !allow_selectors {
                return Err(ScoreHolderArgumentError::SelectorsNotAllowed);
            }
            let token = reader.read_token();
            let selector = Selector::parse(&token).map_err(ScoreHolderArgumentError::Selector)?;
            if !self.multiple && selector.limit > 1 {
                return Err(ScoreHolderArgumentError::NotSingleEntity);
            }
            Ok(ScoreHolderResultModel::Selector(Box::new(selector)))
        } else {
            let text = reader.read_until_space();
            if text == "*" {
                Ok(ScoreHolderResultModel::Wildcard)
            } else if text.starts_with('#') {
                Ok(ScoreHolderResultModel::NameOnly(text))
            } else if is_uuid(&text) {
                Ok(ScoreHolderResultModel::UuidOrName(text))
            } else {
                Ok(ScoreHolderResultModel::PlayerNameOrNameOnly(text))
            }
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
        ["Player", "0123", "*", "@e"]
    }

    pub fn info_template(&self) -> ScoreHolderInfoTemplate {
        ScoreHolderInfoTemplate {
            multiple: self.multiple,
        }
    }
}

pub fn get_name(
    context: &CommandContextModel,
    name: &str,
) -> Result<ScoreHolderModel, ScoreHolderArgumentError> {
    get_names(context, name)?
        .into_iter()
        .next()
        .ok_or(ScoreHolderArgumentError::NoEntitiesFound)
}

pub fn get_names(
    context: &CommandContextModel,
    name: &str,
) -> Result<Vec<ScoreHolderModel>, ScoreHolderArgumentError> {
    get_names_with_wildcard(context, name, Vec::new)
}

pub fn get_names_with_default_wildcard(
    context: &CommandContextModel,
    name: &str,
) -> Result<Vec<ScoreHolderModel>, ScoreHolderArgumentError> {
    get_names_with_wildcard(context, name, || context.tracked_score_holders.clone())
}

pub fn get_names_with_wildcard(
    context: &CommandContextModel,
    name: &str,
    wildcard: impl FnOnce() -> Vec<ScoreHolderModel>,
) -> Result<Vec<ScoreHolderModel>, ScoreHolderArgumentError> {
    let result = context
        .arguments
        .get(name)
        .ok_or(ScoreHolderArgumentError::MissingArgument)?
        .get_names(context, wildcard)?;
    if result.is_empty() {
        Err(ScoreHolderArgumentError::NoEntitiesFound)
    } else {
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScoreHolderResultModel {
    Selector(Box<Selector>),
    Wildcard,
    NameOnly(String),
    UuidOrName(String),
    PlayerNameOrNameOnly(String),
}

impl ScoreHolderResultModel {
    fn get_names(
        &self,
        context: &CommandContextModel,
        wildcard: impl FnOnce() -> Vec<ScoreHolderModel>,
    ) -> Result<Vec<ScoreHolderModel>, ScoreHolderArgumentError> {
        match self {
            Self::Selector(selector) => {
                check_selector_permission(context, selector)?;
                let entities = selector
                    .select(
                        &context.entities,
                        context.source_position,
                        &context.source_level,
                        context.current_entity.as_deref(),
                    )
                    .into_iter()
                    .map(ScoreHolderModel::from_entity)
                    .collect::<Vec<_>>();
                if entities.is_empty() {
                    Err(ScoreHolderArgumentError::NoEntitiesFound)
                } else {
                    Ok(entities)
                }
            }
            Self::Wildcard => {
                let result = wildcard();
                if result.is_empty() {
                    Err(ScoreHolderArgumentError::NoResults)
                } else {
                    Ok(result)
                }
            }
            Self::NameOnly(name) => Ok(vec![ScoreHolderModel::name_only(name)]),
            Self::UuidOrName(text) => {
                let matches = context
                    .entities
                    .iter()
                    .filter(|entity| entity.entity.uuid == *text)
                    .map(|entity| ScoreHolderModel::from_entity(entity.entity.clone()))
                    .collect::<Vec<_>>();
                if matches.is_empty() {
                    Ok(vec![ScoreHolderModel::name_only(text)])
                } else {
                    Ok(matches)
                }
            }
            Self::PlayerNameOrNameOnly(text) => Ok(context
                .entities
                .iter()
                .find(|entity| entity.entity.player && entity.entity.name == *text)
                .map(|entity| vec![ScoreHolderModel::from_entity(entity.entity.clone())])
                .unwrap_or_else(|| vec![ScoreHolderModel::name_only(text)])),
        }
    }
}

fn check_selector_permission(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<(), ScoreHolderArgumentError> {
    if !matches!(selector.base, SelectorBase::Name(_)) && !context.can_use_selectors {
        Err(ScoreHolderArgumentError::SelectorsNotAllowed)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreHolderModel {
    name: String,
    uuid: Option<String>,
    entity: bool,
}

impl ScoreHolderModel {
    pub fn name_only(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            uuid: None,
            entity: false,
        }
    }

    fn from_entity(entity: EntityRecord) -> Self {
        Self {
            name: entity.name,
            uuid: Some(entity.uuid),
            entity: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandContextModel {
    arguments: HashMap<String, ScoreHolderResultModel>,
    entities: Vec<EntityWithPosition>,
    source_position: Vec3,
    source_level: String,
    current_entity: Option<String>,
    can_use_selectors: bool,
    tracked_score_holders: Vec<ScoreHolderModel>,
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
            tracked_score_holders: Vec::new(),
        }
    }
}

impl CommandContextModel {
    pub fn with_argument(mut self, name: impl Into<String>, value: ScoreHolderResultModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }

    pub fn with_entities(mut self, entities: Vec<EntityWithPosition>) -> Self {
        self.entities = entities;
        self
    }

    pub fn with_selector_permission(mut self, allowed: bool) -> Self {
        self.can_use_selectors = allowed;
        self
    }

    pub fn with_tracked_score_holders(mut self, holders: &[&str]) -> Self {
        self.tracked_score_holders = holders
            .iter()
            .map(|holder| ScoreHolderModel::name_only(*holder))
            .collect();
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
pub struct ScoreHolderInfoTemplate {
    multiple: bool,
}

impl ScoreHolderInfoTemplate {
    pub fn serialize_to_network(&self) -> u8 {
        u8::from(self.multiple)
    }

    pub fn deserialize_from_network(flags: u8) -> Self {
        Self {
            multiple: flags & 1 != 0,
        }
    }

    pub fn serialize_to_json_amount(&self) -> &'static str {
        if self.multiple {
            "multiple"
        } else {
            "single"
        }
    }

    pub fn instantiate(&self) -> ScoreHolderArgumentModel {
        ScoreHolderArgumentModel {
            multiple: self.multiple,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreHolderArgumentError {
    Selector(SelectorError),
    NotSingleEntity,
    SelectorsNotAllowed,
    NoEntitiesFound,
    NoResults,
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

    fn read_until_space(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && self.peek() != ' ' {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }

    fn read_token(&mut self) -> String {
        let start = self.cursor;
        if self.peek() == '@' {
            self.cursor += 1;
            if self.can_read() {
                self.cursor += 1;
            }
            if self.can_read() && self.peek() == '[' {
                let mut depth = 0usize;
                while self.can_read() {
                    let ch = self.peek();
                    match ch {
                        '[' => depth += 1,
                        ']' => {
                            depth = depth.saturating_sub(1);
                            self.cursor += 1;
                            if depth == 0 {
                                break;
                            }
                            continue;
                        }
                        _ => {}
                    }
                    self.cursor += 1;
                }
            }
            self.input[start..self.cursor].to_string()
        } else {
            self.read_until_space()
        }
    }
}

fn is_uuid(value: &str) -> bool {
    let parts = value.split('-').map(str::len).collect::<Vec<_>>();
    parts == [8, 4, 4, 4, 12]
        && value
            .chars()
            .filter(|ch| *ch != '-')
            .all(|ch| ch.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            },
            position: Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
        }
    }

    fn entity_in_level(
        name: &str,
        uuid: &str,
        player: bool,
        entity_type: &str,
        level: &str,
    ) -> EntityWithPosition {
        let mut entity = entity(name, uuid, player, entity_type);
        entity.entity.level = level.to_string();
        entity
    }

    #[test]
    fn java_factories_examples_and_info_flags_match_source() {
        let single = ScoreHolderArgumentModel::score_holder();
        let multiple = ScoreHolderArgumentModel::score_holders();
        assert_eq!(single.examples(), ["Player", "0123", "*", "@e"]);
        assert_eq!(single.info_template().serialize_to_network(), 0);
        assert_eq!(multiple.info_template().serialize_to_network(), 1);
        assert_eq!(single.info_template().serialize_to_json_amount(), "single");
        assert_eq!(
            multiple.info_template().serialize_to_json_amount(),
            "multiple"
        );
        assert_eq!(
            ScoreHolderInfoTemplate::deserialize_from_network(1)
                .instantiate()
                .info_template(),
            multiple.info_template()
        );
    }

    #[test]
    fn java_parse_selector_respects_multiple_and_selector_permissions() {
        let mut reader = StringReaderModel::new("@e");
        assert_eq!(
            ScoreHolderArgumentModel::score_holder().parse(&mut reader, true),
            Err(ScoreHolderArgumentError::NotSingleEntity)
        );
        assert_eq!(reader.cursor(), 2);

        let mut denied = StringReaderModel::new("@p");
        assert_eq!(
            ScoreHolderArgumentModel::score_holders().parse(&mut denied, false),
            Err(ScoreHolderArgumentError::SelectorsNotAllowed)
        );

        let mut allowed = StringReaderModel::new("@p rest");
        let result = ScoreHolderArgumentModel::score_holder()
            .parse(&mut allowed, true)
            .unwrap();
        assert!(matches!(result, ScoreHolderResultModel::Selector(_)));
        assert_eq!(allowed.cursor(), 2);
    }

    #[test]
    fn java_parse_plain_wildcard_hash_uuid_and_name_results() {
        assert_eq!(
            ScoreHolderArgumentModel::score_holders()
                .parse(&mut StringReaderModel::new("*"), true)
                .unwrap(),
            ScoreHolderResultModel::Wildcard
        );
        assert_eq!(
            ScoreHolderArgumentModel::score_holders()
                .parse(&mut StringReaderModel::new("#fake"), true)
                .unwrap(),
            ScoreHolderResultModel::NameOnly("#fake".to_string())
        );
        assert_eq!(
            ScoreHolderArgumentModel::score_holders()
                .parse(
                    &mut StringReaderModel::new("dd12be42-52a9-4a91-a8a1-11c01849e498"),
                    true,
                )
                .unwrap(),
            ScoreHolderResultModel::UuidOrName("dd12be42-52a9-4a91-a8a1-11c01849e498".to_string())
        );
        assert_eq!(
            ScoreHolderArgumentModel::score_holders()
                .parse(&mut StringReaderModel::new("Player tail"), true)
                .unwrap(),
            ScoreHolderResultModel::PlayerNameOrNameOnly("Player".to_string())
        );
    }

    #[test]
    fn java_get_names_and_get_name_reject_empty_results() {
        let context = CommandContextModel::default().with_argument(
            "holder",
            ScoreHolderResultModel::PlayerNameOrNameOnly("Alex".to_string()),
        );
        assert_eq!(
            get_name(&context, "holder").unwrap(),
            ScoreHolderModel::name_only("Alex")
        );
        let missing = CommandContextModel::default();
        assert_eq!(
            get_names(&missing, "holder"),
            Err(ScoreHolderArgumentError::MissingArgument)
        );
    }

    #[test]
    fn java_wildcard_uses_supplied_or_default_tracked_score_holders() {
        let context = CommandContextModel::default()
            .with_argument("targets", ScoreHolderResultModel::Wildcard)
            .with_tracked_score_holders(&["Alex", "#fake"]);
        assert_eq!(
            get_names_with_default_wildcard(&context, "targets")
                .unwrap()
                .iter()
                .map(|holder| holder.name().to_string())
                .collect::<Vec<_>>(),
            vec!["Alex", "#fake"]
        );
        assert_eq!(
            get_names_with_wildcard(&context, "targets", Vec::new),
            Err(ScoreHolderArgumentError::NoResults)
        );
    }

    #[test]
    fn java_name_and_uuid_results_resolve_players_entities_or_name_only() {
        let uuid = "dd12be42-52a9-4a91-a8a1-11c01849e498";
        let context = CommandContextModel::default()
            .with_entities(vec![
                entity("Alex", "u1", true, "minecraft:player"),
                entity_in_level("Zombie", uuid, false, "minecraft:zombie", "overworld"),
                entity_in_level("Pig", uuid, false, "minecraft:pig", "the_nether"),
            ])
            .with_argument(
                "name",
                ScoreHolderResultModel::PlayerNameOrNameOnly("Alex".to_string()),
            )
            .with_argument("uuid", ScoreHolderResultModel::UuidOrName(uuid.to_string()))
            .with_argument(
                "missing",
                ScoreHolderResultModel::PlayerNameOrNameOnly("Missing".to_string()),
            )
            .with_argument(
                "fake",
                ScoreHolderResultModel::NameOnly("#Alex".to_string()),
            );

        assert_eq!(
            get_name(&context, "name").unwrap().uuid,
            Some("u1".to_string())
        );
        assert_eq!(
            get_names(&context, "uuid")
                .unwrap()
                .iter()
                .map(|holder| holder.name().to_string())
                .collect::<Vec<_>>(),
            vec!["Zombie", "Pig"]
        );
        assert_eq!(
            get_name(&context, "missing").unwrap(),
            ScoreHolderModel::name_only("Missing")
        );
        assert_eq!(
            get_name(&context, "fake").unwrap(),
            ScoreHolderModel::name_only("#Alex")
        );
    }

    #[test]
    fn java_selector_result_finds_entities_and_rechecks_permission() {
        let selector = Selector::parse("@e").unwrap();
        let context = CommandContextModel::default()
            .with_entities(vec![
                entity("Alex", "u1", true, "minecraft:player"),
                entity("Zombie", "u2", false, "minecraft:zombie"),
            ])
            .with_argument(
                "targets",
                ScoreHolderResultModel::Selector(Box::new(selector.clone())),
            );
        assert_eq!(get_names(&context, "targets").unwrap().len(), 2);

        let denied = CommandContextModel::default()
            .with_selector_permission(false)
            .with_argument(
                "targets",
                ScoreHolderResultModel::Selector(Box::new(selector)),
            );
        assert_eq!(
            get_names(&denied, "targets"),
            Err(ScoreHolderArgumentError::SelectorsNotAllowed)
        );
    }

    #[test]
    fn java_suggestions_use_online_names_and_selector_permission() {
        let source = CommandSourceModel::Shared(
            SharedSuggestionProviderModel::new(true).with_online_players(&["Alex", "Steve"]),
        );
        assert_eq!(
            ScoreHolderArgumentModel::score_holders().list_suggestions(&source, "A"),
            vec!["Alex".to_string()]
        );
        assert_eq!(
            ScoreHolderArgumentModel::score_holders().list_suggestions(&source, "@"),
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
        assert!(ScoreHolderArgumentModel::score_holders()
            .list_suggestions(&source, "@")
            .is_empty());
        assert!(ScoreHolderArgumentModel::score_holders()
            .list_suggestions(&CommandSourceModel::Other, "")
            .is_empty());
    }
}
