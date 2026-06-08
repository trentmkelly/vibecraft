use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveCriteriaArgumentModel {
    registry: CriteriaRegistryModel,
}

impl ObjectiveCriteriaArgumentModel {
    pub fn criteria() -> Self {
        Self {
            registry: CriteriaRegistryModel::java_subset(),
        }
    }

    pub fn with_registry(registry: CriteriaRegistryModel) -> Self {
        Self { registry }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<ObjectiveCriteriaModel, ObjectiveCriteriaParseError> {
        let start = reader.cursor();
        while reader.can_read() && reader.peek() != ' ' {
            reader.skip();
        }

        let id = reader.slice(start, reader.cursor()).to_string();
        self.registry.by_name(&id).ok_or_else(|| {
            reader.set_cursor(start);
            ObjectiveCriteriaParseError::InvalidValue { value: id }
        })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        let mut ids = self.registry.custom_criteria_names();
        for stat_type in &self.registry.stat_types {
            for value in &stat_type.values {
                ids.push(Self::get_name(stat_type, value));
            }
        }
        suggest(ids, builder);
        builder.clone().build()
    }

    pub fn get_name(stat_type: &StatTypeModel, value: &StatValueModel) -> String {
        StatModel::build_name(stat_type, value)
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["foo", "foo.bar.baz", "minecraft:foo"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, ObjectiveCriteriaModel>,
}

impl CommandContextModel {
    pub fn with_criteria(mut self, name: impl Into<String>, value: ObjectiveCriteriaModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_criteria(context: &CommandContextModel, name: &str) -> Option<ObjectiveCriteriaModel> {
    context.arguments.get(name).cloned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriteriaRegistryModel {
    custom_criteria: Vec<ObjectiveCriteriaModel>,
    criteria_cache: HashMap<String, ObjectiveCriteriaModel>,
    stat_types: Vec<StatTypeModel>,
}

impl CriteriaRegistryModel {
    pub fn java_subset() -> Self {
        let mut registry = Self {
            custom_criteria: Vec::new(),
            criteria_cache: HashMap::new(),
            stat_types: Vec::new(),
        };
        for criteria in ObjectiveCriteriaModel::java_custom_criteria() {
            registry.register_custom(criteria);
        }
        registry
    }

    pub fn with_stat_type(mut self, stat_type: StatTypeModel) -> Self {
        self.stat_types.push(stat_type);
        self
    }

    pub fn by_name(&self, name: &str) -> Option<ObjectiveCriteriaModel> {
        if let Some(criteria) = self.criteria_cache.get(name) {
            return Some(criteria.clone());
        }

        let colon = name.find(':')?;
        let stat_type_key = IdentifierModel::by_separator(&name[..colon], '.');
        let value_key = IdentifierModel::by_separator(&name[colon + 1..], '.');
        self.stat_types
            .iter()
            .find(|stat_type| stat_type.registry_key == stat_type_key)
            .and_then(|stat_type| stat_type.get(&value_key))
    }

    pub fn custom_criteria_names(&self) -> Vec<String> {
        self.custom_criteria
            .iter()
            .map(|criteria| criteria.name.clone())
            .collect()
    }

    fn register_custom(&mut self, criteria: ObjectiveCriteriaModel) {
        self.criteria_cache
            .insert(criteria.name.clone(), criteria.clone());
        self.custom_criteria.push(criteria);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveCriteriaModel {
    name: String,
    read_only: bool,
    render_type: RenderTypeModel,
}

impl ObjectiveCriteriaModel {
    pub fn java_custom_criteria() -> Vec<Self> {
        let mut criteria = vec![
            Self::custom("dummy"),
            Self::custom("trigger"),
            Self::custom("deathCount"),
            Self::custom("playerKillCount"),
            Self::custom("totalKillCount"),
            Self::read_only("health", RenderTypeModel::Hearts),
            Self::read_only("food", RenderTypeModel::Integer),
            Self::read_only("air", RenderTypeModel::Integer),
            Self::read_only("armor", RenderTypeModel::Integer),
            Self::read_only("xp", RenderTypeModel::Integer),
            Self::read_only("level", RenderTypeModel::Integer),
        ];
        for prefix in ["teamkill", "killedByTeam"] {
            for color in [
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
            ] {
                criteria.push(Self::custom(format!("{prefix}.{color}")));
            }
        }
        criteria
    }

    pub fn custom(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            read_only: false,
            render_type: RenderTypeModel::Integer,
        }
    }

    pub fn read_only(name: impl Into<String>, render_type: RenderTypeModel) -> Self {
        Self {
            name: name.into(),
            read_only: true,
            render_type,
        }
    }

    pub fn stat(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            read_only: false,
            render_type: RenderTypeModel::Integer,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn default_render_type(&self) -> RenderTypeModel {
        self.render_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTypeModel {
    Integer,
    Hearts,
}

impl RenderTypeModel {
    pub fn id(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Hearts => "hearts",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatTypeModel {
    registry_key: IdentifierModel,
    values: Vec<StatValueModel>,
}

impl StatTypeModel {
    pub fn new(registry_key: IdentifierModel, values: Vec<StatValueModel>) -> Self {
        Self {
            registry_key,
            values,
        }
    }

    fn get(&self, value_key: &IdentifierModel) -> Option<ObjectiveCriteriaModel> {
        self.values
            .iter()
            .find(|value| &value.key == value_key)
            .map(|value| ObjectiveCriteriaModel::stat(StatModel::build_name(self, value)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatValueModel {
    key: IdentifierModel,
}

impl StatValueModel {
    pub fn new(key: IdentifierModel) -> Self {
        Self { key }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatModel;

impl StatModel {
    pub fn build_name(stat_type: &StatTypeModel, value: &StatValueModel) -> String {
        format!(
            "{}:{}",
            stat_type.registry_key.to_stat_key(),
            value.key.to_stat_key()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentifierModel {
    namespace: String,
    path: String,
}

impl IdentifierModel {
    pub fn parse(value: &str) -> Self {
        Self::by_separator(value, ':')
    }

    pub fn by_separator(value: &str, separator: char) -> Self {
        if let Some(separator_index) = value.find(separator) {
            let path = &value[separator_index + 1..];
            if separator_index == 0 {
                Self::new("minecraft", path)
            } else {
                Self::new(&value[..separator_index], path)
            }
        } else {
            Self::new("minecraft", value)
        }
    }

    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn to_stat_key(&self) -> String {
        self.to_string().replace(':', ".")
    }
}

impl std::fmt::Display for IdentifierModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.path)
    }
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

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.input[start..end]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectiveCriteriaParseError {
    InvalidValue { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_registry() -> CriteriaRegistryModel {
        CriteriaRegistryModel::java_subset().with_stat_type(StatTypeModel::new(
            IdentifierModel::parse("minecraft:mined"),
            vec![
                StatValueModel::new(IdentifierModel::parse("minecraft:stone")),
                StatValueModel::new(IdentifierModel::parse("minecraft:custom/path")),
            ],
        ))
    }

    fn suggestion_values(suggestions: Vec<SuggestionModel>) -> Vec<String> {
        suggestions
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect()
    }

    #[test]
    fn java_factory_examples_and_context_lookup_match_source() {
        let argument = ObjectiveCriteriaArgumentModel::criteria();
        assert_eq!(argument.examples(), ["foo", "foo.bar.baz", "minecraft:foo"]);

        let criteria = ObjectiveCriteriaModel::custom("dummy");
        let context = CommandContextModel::default().with_criteria("criteria", criteria.clone());
        assert_eq!(get_criteria(&context, "criteria"), Some(criteria));
        assert_eq!(get_criteria(&context, "missing"), None);
    }

    #[test]
    fn java_custom_criteria_metadata_matches_source_shape() {
        let registry = CriteriaRegistryModel::java_subset();
        let names = registry.custom_criteria_names();

        assert_eq!(names.len(), 43);
        assert!(names.contains(&"dummy".to_string()));
        assert!(names.contains(&"health".to_string()));
        assert!(names.contains(&"teamkill.dark_purple".to_string()));
        assert!(names.contains(&"killedByTeam.white".to_string()));

        let health = registry.by_name("health").unwrap();
        assert!(health.is_read_only());
        assert_eq!(health.default_render_type(), RenderTypeModel::Hearts);
        assert_eq!(RenderTypeModel::Integer.id(), "integer");
        assert_eq!(RenderTypeModel::Hearts.id(), "hearts");
    }

    #[test]
    fn java_parse_reads_until_space_and_resolves_custom_criteria() {
        let argument = ObjectiveCriteriaArgumentModel::with_registry(sample_registry());
        let mut reader = StringReaderModel::new("teamkill.dark_blue rest");

        let parsed = argument.parse(&mut reader).unwrap();

        assert_eq!(parsed.name(), "teamkill.dark_blue");
        assert_eq!(reader.cursor(), "teamkill.dark_blue".len());
    }

    #[test]
    fn java_parse_resolves_stat_criteria_by_separator_and_build_name() {
        let argument = ObjectiveCriteriaArgumentModel::with_registry(sample_registry());
        let mut reader = StringReaderModel::new("minecraft.mined:minecraft.stone rest");

        let parsed = argument.parse(&mut reader).unwrap();

        assert_eq!(parsed.name(), "minecraft.mined:minecraft.stone");
        assert_eq!(reader.cursor(), "minecraft.mined:minecraft.stone".len());
    }

    #[test]
    fn java_invalid_criteria_resets_reader_cursor() {
        let argument = ObjectiveCriteriaArgumentModel::with_registry(sample_registry());
        let mut reader = StringReaderModel::new("missing.criteria rest");

        assert_eq!(
            argument.parse(&mut reader),
            Err(ObjectiveCriteriaParseError::InvalidValue {
                value: "missing.criteria".to_string()
            })
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_empty_token_is_invalid_and_keeps_cursor_at_start() {
        let argument = ObjectiveCriteriaArgumentModel::with_registry(sample_registry());
        let mut reader = StringReaderModel::new("");

        assert_eq!(
            argument.parse(&mut reader),
            Err(ObjectiveCriteriaParseError::InvalidValue {
                value: String::new()
            })
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_get_name_matches_stat_build_name_location_to_key_conversion() {
        let stat_type = StatTypeModel::new(
            IdentifierModel::parse("minecraft:custom"),
            vec![StatValueModel::new(IdentifierModel::parse(
                "minecraft:leave_game",
            ))],
        );
        let value = &stat_type.values[0];

        assert_eq!(
            ObjectiveCriteriaArgumentModel::get_name(&stat_type, value),
            "minecraft.custom:minecraft.leave_game"
        );
    }

    #[test]
    fn java_suggestions_include_custom_criteria_then_stat_entries() {
        let argument = ObjectiveCriteriaArgumentModel::with_registry(sample_registry());
        let mut builder = SuggestionsBuilderModel::new("minecraft.mined");

        assert_eq!(
            suggestion_values(argument.list_suggestions(&mut builder)),
            vec![
                "minecraft.mined:minecraft.stone".to_string(),
                "minecraft.mined:minecraft.custom/path".to_string(),
            ]
        );
    }
}
