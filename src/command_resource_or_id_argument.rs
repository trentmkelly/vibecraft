use std::collections::HashMap;

use crate::command_identifier_argument::CommandIdentifierModel;
use crate::storage::nbt::{parse_snbt, Tag};

const LOOT_TABLE_REGISTRY: &str = "minecraft:loot_table";
const ITEM_MODIFIER_REGISTRY: &str = "minecraft:item_modifier";
const PREDICATE_REGISTRY: &str = "minecraft:predicate";
const DIALOG_REGISTRY: &str = "minecraft:dialog";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOrIdArgumentModel {
    registry_key: CommandIdentifierModel,
    element_lookup: Option<HolderLookupModel>,
    codec: InlineCodecModel,
}

impl ResourceOrIdArgumentModel {
    pub fn loot_table(context: &CommandBuildContextModel) -> Self {
        Self::new(context, LOOT_TABLE_REGISTRY, InlineCodecModel::LootTable)
    }

    pub fn loot_modifier(context: &CommandBuildContextModel) -> Self {
        Self::new(
            context,
            ITEM_MODIFIER_REGISTRY,
            InlineCodecModel::LootModifier,
        )
    }

    pub fn loot_predicate(context: &CommandBuildContextModel) -> Self {
        Self::new(context, PREDICATE_REGISTRY, InlineCodecModel::LootPredicate)
    }

    pub fn dialog(context: &CommandBuildContextModel) -> Self {
        Self::new(context, DIALOG_REGISTRY, InlineCodecModel::Dialog)
    }

    fn new(
        context: &CommandBuildContextModel,
        registry_key: &str,
        codec: InlineCodecModel,
    ) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry_key).expect("valid registry key");
        Self {
            element_lookup: context.lookup(&registry_key),
            registry_key,
            codec,
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<Option<HolderModel>, ResourceOrIdParseError> {
        let result = parse_result(reader)?;
        let Some(lookup) = &self.element_lookup else {
            return Ok(None);
        };
        result.parse(&self.registry_key, lookup, self.codec, reader.cursor())
    }

    pub fn list_suggestions(&self, context: &CommandContextModel, remaining: &str) -> Vec<String> {
        match &context.source {
            CommandSourceModel::Shared(provider) => provider.suggest_registry_elements(
                &self.registry_key,
                ElementSuggestionType::Elements,
                remaining,
            ),
            CommandSourceModel::Other => Vec::new(),
        }
    }

    pub fn examples(&self) -> [&'static str; 5] {
        ["foo", "foo:bar", "012", "{}", "true"]
    }
}

pub fn get_loot_table(context: &CommandContextModel, name: &str) -> Option<HolderModel> {
    get_resource(context, name)
}

pub fn get_loot_modifier(context: &CommandContextModel, name: &str) -> Option<HolderModel> {
    get_resource(context, name)
}

pub fn get_loot_predicate(context: &CommandContextModel, name: &str) -> Option<HolderModel> {
    get_resource(context, name)
}

pub fn get_dialog(context: &CommandContextModel, name: &str) -> Option<HolderModel> {
    get_resource(context, name)
}

fn get_resource(context: &CommandContextModel, name: &str) -> Option<HolderModel> {
    context.arguments.get(name).cloned()
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceOrIdResultModel {
    Reference(CommandIdentifierModel),
    Inline(Tag),
}

impl ResourceOrIdResultModel {
    fn parse(
        self,
        registry_key: &CommandIdentifierModel,
        lookup: &HolderLookupModel,
        codec: InlineCodecModel,
        cursor: usize,
    ) -> Result<Option<HolderModel>, ResourceOrIdParseError> {
        match self {
            Self::Reference(id) => {
                let key = ResourceKeyModel::create(registry_key.clone(), id.clone());
                lookup
                    .get(&key)
                    .map(Some)
                    .ok_or(ResourceOrIdParseError::NoSuchElement {
                        id,
                        registry: registry_key.clone(),
                        cursor,
                    })
            }
            Self::Inline(tag) => codec
                .parse(tag)
                .map(|value| Some(HolderModel::direct(value)))
                .map_err(|message| ResourceOrIdParseError::FailedToParse { message, cursor }),
        }
    }
}

fn parse_result(
    reader: &mut StringReaderModel,
) -> Result<ResourceOrIdResultModel, ResourceOrIdParseError> {
    let start = reader.cursor();
    let token = reader.read_value_token();
    if is_inline_token(&token) {
        let tag = parse_snbt(&token).map_err(|error| ResourceOrIdParseError::FailedToParse {
            message: error.to_string(),
            cursor: start,
        })?;
        return Ok(ResourceOrIdResultModel::Inline(tag));
    }
    CommandIdentifierModel::parse(&token)
        .map(ResourceOrIdResultModel::Reference)
        .map_err(|error| ResourceOrIdParseError::Identifier {
            error,
            cursor: start,
        })
}

fn is_inline_token(token: &str) -> bool {
    token.starts_with('{')
        || token.starts_with('[')
        || token.starts_with('"')
        || token.starts_with('\'')
        || token.eq_ignore_ascii_case("true")
        || token.eq_ignore_ascii_case("false")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineCodecModel {
    LootTable,
    LootModifier,
    LootPredicate,
    Dialog,
}

impl InlineCodecModel {
    fn parse(self, tag: Tag) -> Result<ResourceValueModel, String> {
        match (self, tag) {
            (Self::LootTable, Tag::Compound(fields)) => Ok(ResourceValueModel::LootTable {
                pools: int_field(&fields, "pools").unwrap_or(0),
            }),
            (Self::LootModifier, Tag::Compound(fields)) => Ok(ResourceValueModel::LootModifier {
                function: string_field(&fields, "function")
                    .unwrap_or_else(|| "minecraft:set_count".to_string()),
            }),
            (Self::LootPredicate, Tag::Compound(fields)) => Ok(ResourceValueModel::LootPredicate {
                condition: string_field(&fields, "condition")
                    .unwrap_or_else(|| "minecraft:survives_explosion".to_string()),
            }),
            (Self::Dialog, Tag::Compound(fields)) => Ok(ResourceValueModel::Dialog {
                title: string_field(&fields, "title").unwrap_or_default(),
            }),
            (_, tag) => Err(format!(
                "inline value must be a compound, got tag id {}",
                tag.id()
            )),
        }
    }
}

fn int_field(fields: &[(String, Tag)], name: &str) -> Option<i32> {
    fields
        .iter()
        .find_map(|(key, value)| match (key == name, value) {
            (true, Tag::Int(value)) => Some(*value),
            _ => None,
        })
}

fn string_field(fields: &[(String, Tag)], name: &str) -> Option<String> {
    fields
        .iter()
        .find_map(|(key, value)| match (key == name, value) {
            (true, Tag::String(value)) => Some(value.clone()),
            _ => None,
        })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBuildContextModel {
    registries: HashMap<CommandIdentifierModel, HolderLookupModel>,
}

impl CommandBuildContextModel {
    pub fn with_registry(
        mut self,
        registry: &str,
        elements: &[(&str, ResourceValueModel)],
    ) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry).expect("valid registry key");
        self.registries.insert(
            registry_key.clone(),
            HolderLookupModel {
                elements: elements
                    .iter()
                    .map(|(id, value)| {
                        HolderModel::reference(
                            ResourceKeyModel::create(
                                registry_key.clone(),
                                CommandIdentifierModel::parse(id).expect("valid element id"),
                            ),
                            value.clone(),
                        )
                    })
                    .collect(),
            },
        );
        self
    }

    fn lookup(&self, registry: &CommandIdentifierModel) -> Option<HolderLookupModel> {
        self.registries.get(registry).cloned()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, HolderModel>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn with_holder(mut self, name: impl Into<String>, holder: HolderModel) -> Self {
        self.arguments.insert(name.into(), holder);
        self
    }

    pub fn with_source(mut self, source: CommandSourceModel) -> Self {
        self.source = source;
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
    registries: HashMap<CommandIdentifierModel, HolderLookupModel>,
}

impl SharedSuggestionProviderModel {
    pub fn with_registry(
        mut self,
        registry: &str,
        elements: &[(&str, ResourceValueModel)],
    ) -> Self {
        let context = CommandBuildContextModel::default().with_registry(registry, elements);
        self.registries = context.registries;
        self
    }

    fn suggest_registry_elements(
        &self,
        registry: &CommandIdentifierModel,
        element_type: ElementSuggestionType,
        remaining: &str,
    ) -> Vec<String> {
        if !element_type.should_suggest_elements() {
            return Vec::new();
        }
        self.registries
            .get(registry)
            .map(|lookup| {
                lookup
                    .elements
                    .iter()
                    .filter_map(|holder| holder.key.as_ref())
                    .map(|key| key.identifier.to_string())
                    .filter(|id| id.starts_with(remaining))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementSuggestionType {
    Elements,
}

impl ElementSuggestionType {
    fn should_suggest_elements(self) -> bool {
        matches!(self, Self::Elements)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HolderLookupModel {
    elements: Vec<HolderModel>,
}

impl HolderLookupModel {
    fn get(&self, key: &ResourceKeyModel) -> Option<HolderModel> {
        self.elements
            .iter()
            .find(|holder| holder.key.as_ref() == Some(key))
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderModel {
    key: Option<ResourceKeyModel>,
    value: ResourceValueModel,
}

impl HolderModel {
    fn reference(key: ResourceKeyModel, value: ResourceValueModel) -> Self {
        Self {
            key: Some(key),
            value,
        }
    }

    fn direct(value: ResourceValueModel) -> Self {
        Self { key: None, value }
    }

    pub fn key(&self) -> Option<&ResourceKeyModel> {
        self.key.as_ref()
    }

    pub fn value(&self) -> &ResourceValueModel {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceValueModel {
    LootTable { pools: i32 },
    LootModifier { function: String },
    LootPredicate { condition: String },
    Dialog { title: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceKeyModel {
    registry: CommandIdentifierModel,
    identifier: CommandIdentifierModel,
}

impl ResourceKeyModel {
    fn create(registry: CommandIdentifierModel, identifier: CommandIdentifierModel) -> Self {
        Self {
            registry,
            identifier,
        }
    }

    pub fn registry(&self) -> &CommandIdentifierModel {
        &self.registry
    }

    pub fn identifier(&self) -> &CommandIdentifierModel {
        &self.identifier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOrIdParseError {
    Identifier {
        error: crate::command_identifier_argument::IdentifierArgumentParseError,
        cursor: usize,
    },
    FailedToParse {
        message: String,
        cursor: usize,
    },
    NoSuchElement {
        id: CommandIdentifierModel,
        registry: CommandIdentifierModel,
        cursor: usize,
    },
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

    fn read_value_token(&mut self) -> String {
        let start = self.cursor;
        if self.cursor >= self.input.len() {
            return String::new();
        }
        if matches!(self.input.as_bytes()[self.cursor] as char, '{' | '[') {
            let open = self.input.as_bytes()[self.cursor] as char;
            let close = if open == '{' { '}' } else { ']' };
            let mut depth = 0usize;
            let mut quote = None;
            while self.cursor < self.input.len() {
                let ch = self.input.as_bytes()[self.cursor] as char;
                if let Some(active) = quote {
                    if ch == active {
                        quote = None;
                    }
                } else {
                    match ch {
                        '"' | '\'' => quote = Some(ch),
                        value if value == open => depth += 1,
                        value if value == close => {
                            depth = depth.saturating_sub(1);
                            self.cursor += 1;
                            if depth == 0 {
                                break;
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
                self.cursor += 1;
            }
        } else {
            while self.cursor < self.input.len()
                && !(self.input.as_bytes()[self.cursor] as char).is_ascii_whitespace()
            {
                self.cursor += 1;
            }
        }
        self.input[start..self.cursor].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> CommandIdentifierModel {
        CommandIdentifierModel::parse(value).unwrap()
    }

    #[test]
    fn java_factories_examples_and_context_getters_match_source() {
        let context = CommandBuildContextModel::default();
        assert_eq!(
            ResourceOrIdArgumentModel::loot_table(&context).examples(),
            ["foo", "foo:bar", "012", "{}", "true"]
        );
        let holder = HolderModel::direct(ResourceValueModel::Dialog {
            title: "Notice".to_string(),
        });
        let modifier = HolderModel::direct(ResourceValueModel::LootModifier {
            function: "minecraft:set_count".to_string(),
        });
        let predicate = HolderModel::direct(ResourceValueModel::LootPredicate {
            condition: "minecraft:survives_explosion".to_string(),
        });
        let command_context = CommandContextModel::default()
            .with_holder("dialog", holder.clone())
            .with_holder("modifier", modifier.clone())
            .with_holder("predicate", predicate.clone());
        assert_eq!(get_dialog(&command_context, "dialog"), Some(holder));
        assert_eq!(
            get_loot_modifier(&command_context, "modifier"),
            Some(modifier)
        );
        assert_eq!(
            get_loot_predicate(&command_context, "predicate"),
            Some(predicate)
        );
        assert_eq!(get_loot_table(&command_context, "missing"), None);
    }

    #[test]
    fn java_reference_result_resolves_holder_from_registry_lookup() {
        let context = CommandBuildContextModel::default().with_registry(
            LOOT_TABLE_REGISTRY,
            &[(
                "minecraft:chests/simple",
                ResourceValueModel::LootTable { pools: 1 },
            )],
        );
        let argument = ResourceOrIdArgumentModel::loot_table(&context);
        let mut reader = StringReaderModel::new("minecraft:chests/simple trailing");
        let holder = argument.parse(&mut reader).unwrap().unwrap();

        assert_eq!(holder.key().unwrap().registry(), &id(LOOT_TABLE_REGISTRY));
        assert_eq!(
            holder.key().unwrap().identifier(),
            &id("minecraft:chests/simple")
        );
        assert_eq!(holder.value(), &ResourceValueModel::LootTable { pools: 1 });
        assert_eq!(reader.cursor(), "minecraft:chests/simple".len());
    }

    #[test]
    fn java_missing_registry_lookup_returns_null_holder() {
        let argument = ResourceOrIdArgumentModel::dialog(&CommandBuildContextModel::default());
        let mut reader = StringReaderModel::new("minecraft:test");

        assert_eq!(argument.parse(&mut reader), Ok(None));
    }

    #[test]
    fn java_unknown_reference_reports_id_registry_and_reader_cursor() {
        let context = CommandBuildContextModel::default().with_registry(LOOT_TABLE_REGISTRY, &[]);
        let argument = ResourceOrIdArgumentModel::loot_table(&context);
        let mut reader = StringReaderModel::new("minecraft:missing");

        assert_eq!(
            argument.parse(&mut reader),
            Err(ResourceOrIdParseError::NoSuchElement {
                id: id("minecraft:missing"),
                registry: id(LOOT_TABLE_REGISTRY),
                cursor: "minecraft:missing".len()
            })
        );
    }

    #[test]
    fn java_inline_result_uses_codec_to_make_direct_holder() {
        let context = CommandBuildContextModel::default().with_registry(DIALOG_REGISTRY, &[]);
        let argument = ResourceOrIdArgumentModel::dialog(&context);
        let mut reader = StringReaderModel::new("{title:\"Hello\"} rest");
        let holder = argument.parse(&mut reader).unwrap().unwrap();

        assert_eq!(holder.key(), None);
        assert_eq!(
            holder.value(),
            &ResourceValueModel::Dialog {
                title: "Hello".to_string()
            }
        );
        assert_eq!(reader.cursor(), "{title:\"Hello\"}".len());
    }

    #[test]
    fn java_inline_parse_wraps_codec_failures() {
        let context = CommandBuildContextModel::default().with_registry(PREDICATE_REGISTRY, &[]);
        let argument = ResourceOrIdArgumentModel::loot_predicate(&context);
        let mut reader = StringReaderModel::new("true");

        assert!(matches!(
            argument.parse(&mut reader),
            Err(ResourceOrIdParseError::FailedToParse { cursor: 4, .. })
        ));
    }

    #[test]
    fn java_concrete_inline_codecs_cover_loot_modifier_and_predicate() {
        let context = CommandBuildContextModel::default()
            .with_registry(ITEM_MODIFIER_REGISTRY, &[])
            .with_registry(PREDICATE_REGISTRY, &[]);

        let mut modifier_reader = StringReaderModel::new("{function:\"minecraft:set_name\"}");
        assert_eq!(
            ResourceOrIdArgumentModel::loot_modifier(&context)
                .parse(&mut modifier_reader)
                .unwrap()
                .unwrap()
                .value(),
            &ResourceValueModel::LootModifier {
                function: "minecraft:set_name".to_string()
            }
        );

        let mut predicate_reader =
            StringReaderModel::new("{condition:\"minecraft:random_chance\"}");
        assert_eq!(
            ResourceOrIdArgumentModel::loot_predicate(&context)
                .parse(&mut predicate_reader)
                .unwrap()
                .unwrap()
                .value(),
            &ResourceValueModel::LootPredicate {
                condition: "minecraft:random_chance".to_string()
            }
        );
    }

    #[test]
    fn java_suggestions_delegate_to_shared_provider_elements() {
        let source =
            CommandSourceModel::Shared(SharedSuggestionProviderModel::default().with_registry(
                LOOT_TABLE_REGISTRY,
                &[
                    (
                        "minecraft:chests/simple",
                        ResourceValueModel::LootTable { pools: 1 },
                    ),
                    (
                        "minecraft:gameplay/fishing",
                        ResourceValueModel::LootTable { pools: 2 },
                    ),
                ],
            ));
        let context = CommandContextModel::default().with_source(source);
        let argument = ResourceOrIdArgumentModel::loot_table(
            &CommandBuildContextModel::default().with_registry(LOOT_TABLE_REGISTRY, &[]),
        );

        assert_eq!(
            argument.list_suggestions(&context, "minecraft:chests"),
            vec!["minecraft:chests/simple".to_string()]
        );
        assert!(argument
            .list_suggestions(&CommandContextModel::default(), "minecraft:")
            .is_empty());
    }
}
