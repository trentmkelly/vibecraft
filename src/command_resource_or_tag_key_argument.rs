use std::collections::{HashMap, HashSet};

use crate::command_identifier_argument::{CommandIdentifierModel, IdentifierArgumentParseError};

const ENTITY_TYPE_REGISTRY: &str = "minecraft:entity_type";
const CONFIGURED_FEATURE_REGISTRY: &str = "minecraft:worldgen/configured_feature";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOrTagKeyArgumentModel {
    registry_key: CommandIdentifierModel,
}

impl ResourceOrTagKeyArgumentModel {
    pub fn resource_or_tag_key(registry_key: &str) -> Self {
        Self {
            registry_key: CommandIdentifierModel::parse(registry_key)
                .expect("valid registry key identifier"),
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<ResourceOrTagKeyResultModel, IdentifierArgumentParseError> {
        if reader.can_read() && reader.peek() == '#' {
            let cursor = reader.cursor();
            reader.skip();
            match read_identifier(reader) {
                Ok(tag_id) => Ok(ResourceOrTagKeyResultModel::tag(TagKeyModel::create(
                    self.registry_key.clone(),
                    tag_id,
                ))),
                Err(error) => {
                    reader.set_cursor(cursor);
                    Err(error)
                }
            }
        } else {
            let resource_id = read_identifier(reader)?;
            Ok(ResourceOrTagKeyResultModel::resource(
                ResourceKeyModel::create(self.registry_key.clone(), resource_id),
            ))
        }
    }

    pub fn list_suggestions(&self, context: &CommandContextModel, remaining: &str) -> Vec<String> {
        match &context.source {
            CommandSourceModel::Shared(provider) => provider.suggest_registry_elements(
                &self.registry_key,
                ElementSuggestionType::All,
                remaining,
            ),
            CommandSourceModel::Other => Vec::new(),
        }
    }

    pub fn examples(&self) -> [&'static str; 5] {
        [
            "foo",
            "foo:bar",
            "012",
            "#skeletons",
            "#minecraft:skeletons",
        ]
    }

    pub fn info_template(&self) -> InfoTemplateModel {
        InfoTemplateModel {
            registry_key: self.registry_key.clone(),
        }
    }
}

pub fn get_resource_or_tag_key(
    context: &CommandContextModel,
    name: &str,
    registry_key: &str,
    exception_name: &'static str,
) -> Result<ResourceOrTagKeyResultModel, ResourceOrTagKeyLookupError> {
    let expected_registry =
        CommandIdentifierModel::parse(registry_key).expect("valid registry key");
    let argument = context
        .arguments
        .get(name)
        .ok_or(ResourceOrTagKeyLookupError::MissingArgument)?
        .clone();
    argument
        .cast(&expected_registry)
        .ok_or(ResourceOrTagKeyLookupError::Dynamic {
            exception_name,
            printable: argument.as_printable(),
        })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, ResourceOrTagKeyResultModel>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn with_result(
        mut self,
        name: impl Into<String>,
        result: ResourceOrTagKeyResultModel,
    ) -> Self {
        self.arguments.insert(name.into(), result);
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
    registries: HashMap<CommandIdentifierModel, RegistrySuggestionModel>,
}

impl SharedSuggestionProviderModel {
    pub fn with_registry(mut self, registry: &str, elements: &[&str], tags: &[&str]) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry).expect("valid registry key");
        self.registries.insert(
            registry_key,
            RegistrySuggestionModel {
                elements: elements
                    .iter()
                    .map(|id| CommandIdentifierModel::parse(id).expect("valid element id"))
                    .collect(),
                tags: tags
                    .iter()
                    .map(|id| CommandIdentifierModel::parse(id).expect("valid tag id"))
                    .collect(),
            },
        );
        self
    }

    fn suggest_registry_elements(
        &self,
        registry: &CommandIdentifierModel,
        element_type: ElementSuggestionType,
        remaining: &str,
    ) -> Vec<String> {
        let Some(registry) = self.registries.get(registry) else {
            return Vec::new();
        };

        let mut suggestions = Vec::new();
        if element_type.should_suggest_tags() {
            suggestions.extend(suggest_resource(
                registry.tags.iter(),
                remaining,
                SuggestionPrefix::Tag,
            ));
        }
        if element_type.should_suggest_elements() {
            suggestions.extend(suggest_resource(
                registry.elements.iter(),
                remaining,
                SuggestionPrefix::Element,
            ));
        }
        suggestions
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrySuggestionModel {
    elements: Vec<CommandIdentifierModel>,
    tags: Vec<CommandIdentifierModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementSuggestionType {
    Elements,
    Tags,
    All,
}

impl ElementSuggestionType {
    fn should_suggest_tags(self) -> bool {
        matches!(self, Self::Tags | Self::All)
    }

    fn should_suggest_elements(self) -> bool {
        matches!(self, Self::Elements | Self::All)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOrTagKeyResultModel {
    Resource(ResourceKeyModel),
    Tag(TagKeyModel),
}

impl ResourceOrTagKeyResultModel {
    fn resource(key: ResourceKeyModel) -> Self {
        Self::Resource(key)
    }

    fn tag(key: TagKeyModel) -> Self {
        Self::Tag(key)
    }

    pub fn unwrap(&self) -> Result<ResourceKeyModel, TagKeyModel> {
        match self {
            Self::Resource(key) => Ok(key.clone()),
            Self::Tag(key) => Err(key.clone()),
        }
    }

    pub fn cast(&self, registry_key: &CommandIdentifierModel) -> Option<Self> {
        match self {
            Self::Resource(key) => key.cast(registry_key).map(Self::Resource),
            Self::Tag(key) => key.cast(registry_key).map(Self::Tag),
        }
    }

    pub fn test(&self, holder: &HolderModel) -> bool {
        match self {
            Self::Resource(key) => holder.is_key(key),
            Self::Tag(key) => holder.is_tag(key),
        }
    }

    pub fn as_printable(&self) -> String {
        match self {
            Self::Resource(key) => key.identifier().to_string(),
            Self::Tag(key) => format!("#{}", key.location()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKeyModel {
    registry: CommandIdentifierModel,
    identifier: CommandIdentifierModel,
}

impl ResourceKeyModel {
    pub fn create(registry: CommandIdentifierModel, identifier: CommandIdentifierModel) -> Self {
        Self {
            registry,
            identifier,
        }
    }

    pub fn cast(&self, registry: &CommandIdentifierModel) -> Option<Self> {
        (&self.registry == registry).then(|| self.clone())
    }

    pub fn registry(&self) -> &CommandIdentifierModel {
        &self.registry
    }

    pub fn identifier(&self) -> &CommandIdentifierModel {
        &self.identifier
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagKeyModel {
    registry: CommandIdentifierModel,
    location: CommandIdentifierModel,
}

impl TagKeyModel {
    pub fn create(registry: CommandIdentifierModel, location: CommandIdentifierModel) -> Self {
        Self { registry, location }
    }

    pub fn cast(&self, registry: &CommandIdentifierModel) -> Option<Self> {
        (&self.registry == registry).then(|| self.clone())
    }

    pub fn registry(&self) -> &CommandIdentifierModel {
        &self.registry
    }

    pub fn location(&self) -> &CommandIdentifierModel {
        &self.location
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderModel {
    key: ResourceKeyModel,
    tags: HashSet<TagKeyModel>,
}

impl HolderModel {
    pub fn new(registry: &str, identifier: &str, tags: &[&str]) -> Self {
        let registry = CommandIdentifierModel::parse(registry).expect("valid registry id");
        Self {
            key: ResourceKeyModel::create(
                registry.clone(),
                CommandIdentifierModel::parse(identifier).expect("valid holder id"),
            ),
            tags: tags
                .iter()
                .map(|tag| {
                    TagKeyModel::create(
                        registry.clone(),
                        CommandIdentifierModel::parse(tag).expect("valid tag id"),
                    )
                })
                .collect(),
        }
    }

    fn is_key(&self, key: &ResourceKeyModel) -> bool {
        &self.key == key
    }

    fn is_tag(&self, key: &TagKeyModel) -> bool {
        self.tags.contains(key)
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

    pub fn instantiate(&self) -> ResourceOrTagKeyArgumentModel {
        ResourceOrTagKeyArgumentModel::resource_or_tag_key(&self.registry_key.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOrTagKeyLookupError {
    MissingArgument,
    Dynamic {
        exception_name: &'static str,
        printable: String,
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

fn read_identifier(
    reader: &mut StringReaderModel,
) -> Result<CommandIdentifierModel, IdentifierArgumentParseError> {
    let start = reader.cursor();
    while reader.can_read() && is_allowed_in_identifier(reader.peek()) {
        reader.skip();
    }
    let raw = reader.slice(start, reader.cursor()).to_string();
    match CommandIdentifierModel::parse(&raw) {
        Ok(identifier) => Ok(identifier),
        Err(_error) => {
            reader.set_cursor(start);
            Err(IdentifierArgumentParseError::InvalidIdentifier)
        }
    }
}

fn is_allowed_in_identifier(value: char) -> bool {
    value.is_ascii_digit()
        || value.is_ascii_lowercase()
        || matches!(value, '_' | ':' | '/' | '.' | '-')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SuggestionPrefix {
    Element,
    Tag,
}

fn suggest_resource<'a>(
    values: impl Iterator<Item = &'a CommandIdentifierModel>,
    remaining: &str,
    prefix: SuggestionPrefix,
) -> Vec<String> {
    let contents = remaining.to_ascii_lowercase();
    values
        .filter(|id| match prefix {
            SuggestionPrefix::Element => resource_matches(&contents, id),
            SuggestionPrefix::Tag => tag_matches(&contents, id),
        })
        .map(|id| match prefix {
            SuggestionPrefix::Element => id.to_string(),
            SuggestionPrefix::Tag => format!("#{id}"),
        })
        .collect()
}

fn tag_matches(contents: &str, id: &CommandIdentifierModel) -> bool {
    if contents.is_empty() {
        return true;
    }
    let common_len = common_prefix_len(contents, "#");
    common_len > 0 && resource_matches(&contents[common_len..], id)
}

fn resource_matches(contents: &str, id: &CommandIdentifierModel) -> bool {
    if contents.contains(':') {
        matches_sub_str(contents, &id.to_string())
    } else {
        matches_sub_str(contents, id.namespace()) || matches_sub_str(contents, id.path())
    }
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.bytes()
        .zip(right.bytes())
        .take_while(|(left, right)| left == right)
        .count()
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

fn suggestion_source() -> CommandSourceModel {
    CommandSourceModel::Shared(SharedSuggestionProviderModel::default().with_registry(
        ENTITY_TYPE_REGISTRY,
        &["minecraft:skeleton", "minecraft:zombie", "custom:skeleton"],
        &["minecraft:skeletons", "minecraft:undead"],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argument() -> ResourceOrTagKeyArgumentModel {
        ResourceOrTagKeyArgumentModel::resource_or_tag_key(ENTITY_TYPE_REGISTRY)
    }

    fn parse(
        input: &str,
    ) -> Result<(ResourceOrTagKeyResultModel, usize), IdentifierArgumentParseError> {
        let mut reader = StringReaderModel::new(input);
        let result = argument().parse(&mut reader)?;
        Ok((result, reader.cursor()))
    }

    #[test]
    fn java_factory_examples_and_info_template_match_source() {
        let argument = argument();
        assert_eq!(
            argument.examples(),
            [
                "foo",
                "foo:bar",
                "012",
                "#skeletons",
                "#minecraft:skeletons"
            ]
        );

        let template = argument.info_template();
        assert_eq!(template.serialize_to_network(), ENTITY_TYPE_REGISTRY);
        assert_eq!(template.serialize_to_json_registry(), ENTITY_TYPE_REGISTRY);
        assert_eq!(template.instantiate().info_template(), template);
    }

    #[test]
    fn java_parse_resource_key_uses_identifier_read_and_default_namespace() {
        let (result, cursor) = parse("skeleton trailing").unwrap();
        let key = result.unwrap().unwrap();

        assert_eq!(key.registry().to_string(), ENTITY_TYPE_REGISTRY);
        assert_eq!(key.identifier().to_string(), "minecraft:skeleton");
        assert_eq!(result.as_printable(), "minecraft:skeleton");
        assert_eq!(cursor, "skeleton".len());
    }

    #[test]
    fn java_parse_tag_key_consumes_hash_and_defaults_namespace() {
        let (result, cursor) = parse("#skeletons trailing").unwrap();
        let tag = result.unwrap().unwrap_err();

        assert_eq!(tag.registry().to_string(), ENTITY_TYPE_REGISTRY);
        assert_eq!(tag.location().to_string(), "minecraft:skeletons");
        assert_eq!(result.as_printable(), "#minecraft:skeletons");
        assert_eq!(cursor, "#skeletons".len());
    }

    #[test]
    fn java_tag_parse_resets_cursor_to_hash_on_identifier_syntax_error() {
        let mut reader = StringReaderModel::new("#..:bad trailing");
        let error = argument().parse(&mut reader);

        assert_eq!(error, Err(IdentifierArgumentParseError::InvalidIdentifier));
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_resource_parse_resets_cursor_on_identifier_syntax_error() {
        let mut reader = StringReaderModel::new("..:bad trailing");
        let error = argument().parse(&mut reader);

        assert_eq!(error, Err(IdentifierArgumentParseError::InvalidIdentifier));
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_result_cast_preserves_matching_registry_and_rejects_mismatches() {
        let (resource, _) = parse("skeleton").unwrap();
        let (tag, _) = parse("#skeletons").unwrap();
        let entity_registry = CommandIdentifierModel::parse(ENTITY_TYPE_REGISTRY).unwrap();
        let feature_registry = CommandIdentifierModel::parse(CONFIGURED_FEATURE_REGISTRY).unwrap();

        assert_eq!(resource.cast(&entity_registry), Some(resource.clone()));
        assert_eq!(tag.cast(&entity_registry), Some(tag.clone()));
        assert_eq!(resource.cast(&feature_registry), None);
        assert_eq!(tag.cast(&feature_registry), None);
    }

    #[test]
    fn java_context_getter_returns_cast_result_or_dynamic_exception() {
        let (resource, _) = parse("skeleton").unwrap();
        let context = CommandContextModel::default().with_result("target", resource.clone());

        assert_eq!(
            get_resource_or_tag_key(&context, "target", ENTITY_TYPE_REGISTRY, "wrong.type"),
            Ok(resource)
        );
        assert_eq!(
            get_resource_or_tag_key(
                &context,
                "target",
                CONFIGURED_FEATURE_REGISTRY,
                "wrong.type"
            ),
            Err(ResourceOrTagKeyLookupError::Dynamic {
                exception_name: "wrong.type",
                printable: "minecraft:skeleton".to_string(),
            })
        );
        assert_eq!(
            get_resource_or_tag_key(&context, "missing", ENTITY_TYPE_REGISTRY, "wrong.type"),
            Err(ResourceOrTagKeyLookupError::MissingArgument)
        );
    }

    #[test]
    fn java_result_test_uses_holder_key_or_tag_membership() {
        let (resource, _) = parse("skeleton").unwrap();
        let (tag, _) = parse("#skeletons").unwrap();
        let skeleton = HolderModel::new(ENTITY_TYPE_REGISTRY, "minecraft:skeleton", &["skeletons"]);
        let zombie = HolderModel::new(ENTITY_TYPE_REGISTRY, "minecraft:zombie", &["skeletons"]);
        let pig = HolderModel::new(ENTITY_TYPE_REGISTRY, "minecraft:pig", &[]);

        assert!(resource.test(&skeleton));
        assert!(!resource.test(&zombie));
        assert!(tag.test(&skeleton));
        assert!(tag.test(&zombie));
        assert!(!tag.test(&pig));
    }

    #[test]
    fn java_suggestions_delegate_to_shared_registry_all_suggestions() {
        let context = CommandContextModel::default().with_source(suggestion_source());
        let argument = argument();

        assert_eq!(
            argument.list_suggestions(&context, ""),
            [
                "#minecraft:skeletons",
                "#minecraft:undead",
                "minecraft:skeleton",
                "minecraft:zombie",
                "custom:skeleton"
            ]
        );
        assert_eq!(
            argument.list_suggestions(&context, "#skel"),
            ["#minecraft:skeletons"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "custom:s"),
            ["custom:skeleton"]
        );

        let other = CommandContextModel::default().with_source(CommandSourceModel::Other);
        assert_eq!(argument.list_suggestions(&other, ""), Vec::<String>::new());

        assert!(ElementSuggestionType::All.should_suggest_tags());
        assert!(ElementSuggestionType::All.should_suggest_elements());
        assert!(!ElementSuggestionType::Elements.should_suggest_tags());
        assert!(!ElementSuggestionType::Tags.should_suggest_elements());
    }
}
