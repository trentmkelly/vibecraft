use std::collections::HashMap;

use crate::command_identifier_argument::CommandIdentifierModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSelectorArgumentModel {
    registry_key: CommandIdentifierModel,
    registry_lookup: HolderLookupModel,
}

impl ResourceSelectorArgumentModel {
    pub fn resource_selector(context: &CommandBuildContextModel, registry: &str) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry).expect("valid registry key");
        Self {
            registry_lookup: context.lookup_or_throw(&registry_key),
            registry_key,
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<Vec<HolderReferenceModel>, ResourceSelectorParseError> {
        let pattern = ensure_namespaced(&read_pattern(reader));
        let results = parse_with_registry_pattern(&pattern, &self.registry_lookup);
        if results.is_empty() {
            Err(ResourceSelectorParseError::NoMatches {
                selector: pattern,
                registry: self.registry_key.to_string(),
                cursor: reader.cursor(),
            })
        } else {
            Ok(results)
        }
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

    pub fn examples(&self) -> [&'static str; 3] {
        ["minecraft:*", "*:asset", "*"]
    }

    pub fn info_template(&self) -> InfoTemplateModel {
        InfoTemplateModel {
            registry_key: self.registry_key.clone(),
        }
    }
}

pub fn parse_with_registry(
    reader: &mut StringReaderModel,
    registry: &HolderLookupModel,
) -> Vec<HolderReferenceModel> {
    let pattern = ensure_namespaced(&read_pattern(reader));
    parse_with_registry_pattern(&pattern, registry)
}

pub fn get_selected_resources(
    context: &CommandContextModel,
    name: &str,
) -> Option<Vec<HolderReferenceModel>> {
    context.arguments.get(name).cloned()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBuildContextModel {
    registries: HashMap<CommandIdentifierModel, HolderLookupModel>,
}

impl CommandBuildContextModel {
    pub fn with_registry(mut self, registry: &str, elements: &[&str]) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry).expect("valid registry key");
        self.registries.insert(
            registry_key,
            HolderLookupModel::from_elements(registry, elements),
        );
        self
    }

    fn lookup_or_throw(&self, registry: &CommandIdentifierModel) -> HolderLookupModel {
        self.registries.get(registry).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, Vec<HolderReferenceModel>>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn with_selected_resources(
        mut self,
        name: impl Into<String>,
        values: Vec<HolderReferenceModel>,
    ) -> Self {
        self.arguments.insert(name.into(), values);
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
    pub fn with_registry(mut self, registry: &str, elements: &[&str]) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry).expect("valid registry key");
        self.registries.insert(
            registry_key,
            HolderLookupModel::from_elements(registry, elements),
        );
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
                suggest_resource(
                    lookup.list_elements().map(|holder| holder.key.identifier()),
                    remaining,
                )
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementSuggestionType {
    Elements,
    Tags,
    All,
}

impl ElementSuggestionType {
    fn should_suggest_elements(self) -> bool {
        matches!(self, Self::Elements | Self::All)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HolderLookupModel {
    elements: Vec<HolderReferenceModel>,
}

impl HolderLookupModel {
    fn from_elements(registry: &str, elements: &[&str]) -> Self {
        Self {
            elements: elements
                .iter()
                .map(|element| HolderReferenceModel {
                    key: ResourceKeyModel::create(registry, element),
                })
                .collect(),
        }
    }

    fn list_elements(&self) -> impl Iterator<Item = &HolderReferenceModel> {
        self.elements.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderReferenceModel {
    key: ResourceKeyModel,
}

impl HolderReferenceModel {
    pub fn key(&self) -> &ResourceKeyModel {
        &self.key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceKeyModel {
    registry: CommandIdentifierModel,
    identifier: CommandIdentifierModel,
}

impl ResourceKeyModel {
    fn create(registry: &str, identifier: &str) -> Self {
        Self {
            registry: CommandIdentifierModel::parse(registry).expect("valid registry key"),
            identifier: CommandIdentifierModel::parse(identifier).expect("valid element key"),
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
pub struct InfoTemplateModel {
    registry_key: CommandIdentifierModel,
}

impl InfoTemplateModel {
    pub fn serialize_to_network(&self) -> String {
        self.registry_key.to_string()
    }

    pub fn serialize_to_json_registry(&self) -> String {
        self.registry_key.to_string()
    }

    pub fn instantiate(&self, context: &CommandBuildContextModel) -> ResourceSelectorArgumentModel {
        ResourceSelectorArgumentModel::resource_selector(context, &self.registry_key.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceSelectorParseError {
    NoMatches {
        selector: String,
        registry: String,
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

fn parse_with_registry_pattern(
    pattern: &str,
    registry: &HolderLookupModel,
) -> Vec<HolderReferenceModel> {
    registry
        .list_elements()
        .filter(|element| wildcard_match(&element.key().identifier().to_string(), pattern))
        .cloned()
        .collect()
}

fn read_pattern(reader: &mut StringReaderModel) -> String {
    let start = reader.cursor();
    while reader.can_read() && is_allowed_pattern_character(reader.peek()) {
        reader.skip();
    }
    reader.slice(start, reader.cursor()).to_string()
}

fn is_allowed_pattern_character(character: char) -> bool {
    character.is_ascii_digit()
        || character.is_ascii_lowercase()
        || matches!(character, '_' | ':' | '/' | '.' | '-' | '*' | '?')
}

fn ensure_namespaced(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("minecraft:{input}")
    }
}

fn wildcard_match(input: &str, pattern: &str) -> bool {
    let input = input.as_bytes();
    let pattern = pattern.as_bytes();
    let mut table = vec![vec![false; pattern.len() + 1]; input.len() + 1];
    table[0][0] = true;

    for pattern_index in 1..=pattern.len() {
        if pattern[pattern_index - 1] == b'*' {
            table[0][pattern_index] = table[0][pattern_index - 1];
        }
    }

    for input_index in 1..=input.len() {
        for pattern_index in 1..=pattern.len() {
            table[input_index][pattern_index] = match pattern[pattern_index - 1] {
                b'*' => {
                    table[input_index][pattern_index - 1] || table[input_index - 1][pattern_index]
                }
                b'?' => table[input_index - 1][pattern_index - 1],
                literal => {
                    literal == input[input_index - 1] && table[input_index - 1][pattern_index - 1]
                }
            };
        }
    }

    table[input.len()][pattern.len()]
}

fn suggest_resource<'a>(
    values: impl Iterator<Item = &'a CommandIdentifierModel>,
    remaining: &str,
) -> Vec<String> {
    let contents = remaining.to_ascii_lowercase();
    values
        .filter(|id| {
            if contents.contains(':') {
                matches_sub_str(&contents, &id.to_string())
            } else {
                matches_sub_str(&contents, id.namespace()) || matches_sub_str(&contents, id.path())
            }
        })
        .map(ToString::to_string)
        .collect()
}

fn matches_sub_str(pattern: &str, input: &str) -> bool {
    let mut index = 0;
    loop {
        if input[index..].starts_with(pattern) {
            return true;
        }

        let Some(splitter_offset) = input[index..].find(['.', '_', '/']) else {
            return false;
        };
        index += splitter_offset + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_context() -> CommandBuildContextModel {
        CommandBuildContextModel::default().with_registry(
            "minecraft:configured_feature",
            &[
                "minecraft:asset",
                "minecraft:ancient_city",
                "minecraft:ore/diamond",
                "custom:asset",
                "custom:other",
            ],
        )
    }

    fn argument() -> ResourceSelectorArgumentModel {
        ResourceSelectorArgumentModel::resource_selector(
            &build_context(),
            "minecraft:configured_feature",
        )
    }

    fn parse(input: &str) -> Result<(Vec<String>, usize), ResourceSelectorParseError> {
        let mut reader = StringReaderModel::new(input);
        let values = argument()
            .parse(&mut reader)?
            .into_iter()
            .map(|holder| holder.key().identifier().to_string())
            .collect();
        Ok((values, reader.cursor()))
    }

    #[test]
    fn java_factory_examples_and_info_template_match_source() {
        let argument = argument();
        assert_eq!(argument.examples(), ["minecraft:*", "*:asset", "*"]);

        let template = argument.info_template();
        assert_eq!(
            template.serialize_to_network(),
            "minecraft:configured_feature"
        );
        assert_eq!(
            template.serialize_to_json_registry(),
            "minecraft:configured_feature"
        );

        let instantiated = template.instantiate(&build_context());
        assert_eq!(instantiated.info_template(), template);
    }

    #[test]
    fn java_parse_ensures_default_namespace_for_patterns_without_colon() {
        let (values, cursor) = parse("asset trailing").unwrap();

        assert_eq!(values, ["minecraft:asset"]);
        assert_eq!(cursor, "asset".len());

        let (values, cursor) = parse("*").unwrap();
        assert_eq!(
            values,
            [
                "minecraft:asset",
                "minecraft:ancient_city",
                "minecraft:ore/diamond"
            ]
        );
        assert_eq!(cursor, 1);
    }

    #[test]
    fn java_parse_keeps_explicit_namespaces_and_matches_full_identifier() {
        let (values, cursor) = parse("*:asset next").unwrap();
        assert_eq!(values, ["minecraft:asset", "custom:asset"]);
        assert_eq!(cursor, "*:asset".len());

        let (values, cursor) = parse("custom:*").unwrap();
        assert_eq!(values, ["custom:asset", "custom:other"]);
        assert_eq!(cursor, "custom:*".len());
    }

    #[test]
    fn java_wildcards_support_question_mark_and_are_case_sensitive() {
        let (values, cursor) = parse("ancient_cit?").unwrap();
        assert_eq!(values, ["minecraft:ancient_city"]);
        assert_eq!(cursor, "ancient_cit?".len());

        let mut reader = StringReaderModel::new("ASSET");
        assert_eq!(
            argument().parse(&mut reader),
            Err(ResourceSelectorParseError::NoMatches {
                selector: "minecraft:".to_string(),
                registry: "minecraft:configured_feature".to_string(),
                cursor: 0,
            })
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_read_pattern_stops_at_invalid_characters_without_resetting_on_no_match() {
        let mut reader = StringReaderModel::new("missing@rest");
        assert_eq!(
            argument().parse(&mut reader),
            Err(ResourceSelectorParseError::NoMatches {
                selector: "minecraft:missing".to_string(),
                registry: "minecraft:configured_feature".to_string(),
                cursor: "missing".len(),
            })
        );
        assert_eq!(reader.cursor(), "missing".len());
    }

    #[test]
    fn java_static_parse_returns_empty_collection_instead_of_throwing() {
        let mut reader = StringReaderModel::new("missing");
        let values = parse_with_registry(
            &mut reader,
            &build_context().lookup_or_throw(
                &CommandIdentifierModel::parse("minecraft:configured_feature").unwrap(),
            ),
        );

        assert!(values.is_empty());
        assert_eq!(reader.cursor(), "missing".len());
    }

    #[test]
    fn java_context_getter_returns_selected_collection_argument() {
        let selected = parse("*:asset").unwrap().0;
        let holders: Vec<_> = selected
            .iter()
            .map(|id| HolderReferenceModel {
                key: ResourceKeyModel::create("minecraft:configured_feature", id),
            })
            .collect();
        let context =
            CommandContextModel::default().with_selected_resources("targets", holders.clone());

        assert_eq!(get_selected_resources(&context, "targets"), Some(holders));
        assert_eq!(get_selected_resources(&context, "missing"), None);
    }

    #[test]
    fn java_suggestions_delegate_to_shared_registry_element_suggestions() {
        let source = SharedSuggestionProviderModel::default().with_registry(
            "minecraft:configured_feature",
            &["minecraft:asset", "minecraft:ancient_city", "custom:asset"],
        );
        let context =
            CommandContextModel::default().with_source(CommandSourceModel::Shared(source));
        let argument = argument();

        assert_eq!(
            argument.list_suggestions(&context, ""),
            ["minecraft:asset", "minecraft:ancient_city", "custom:asset"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "ancient_"),
            ["minecraft:ancient_city"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "custom:a"),
            ["custom:asset"]
        );

        let other_source = CommandContextModel::default().with_source(CommandSourceModel::Other);
        assert_eq!(
            argument.list_suggestions(&other_source, ""),
            Vec::<String>::new()
        );

        assert!(!ElementSuggestionType::Tags.should_suggest_elements());
        assert!(ElementSuggestionType::All.should_suggest_elements());
    }

    #[test]
    fn java_resource_keys_preserve_registry_and_identifier() {
        let holder = argument()
            .parse(&mut StringReaderModel::new("ore/*"))
            .unwrap()[0]
            .clone();

        assert_eq!(
            holder.key().registry().to_string(),
            "minecraft:configured_feature"
        );
        assert_eq!(
            holder.key().identifier().to_string(),
            "minecraft:ore/diamond"
        );
    }
}
