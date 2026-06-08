use std::collections::HashMap;

use crate::command_identifier_argument::{CommandIdentifierModel, IdentifierArgumentParseError};

const ENTITY_TYPE_REGISTRY: &str = "minecraft:entity_type";
const CONFIGURED_FEATURE_REGISTRY: &str = "minecraft:worldgen/configured_feature";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOrTagArgumentModel {
    registry_key: CommandIdentifierModel,
    registry_lookup: HolderLookupModel,
}

impl ResourceOrTagArgumentModel {
    pub fn resource_or_tag(context: &CommandBuildContextModel, registry_key: &str) -> Self {
        let registry_key =
            CommandIdentifierModel::parse(registry_key).expect("valid registry key identifier");
        Self {
            registry_lookup: context.lookup_or_throw(&registry_key),
            registry_key,
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<ResourceOrTagResultModel, ResourceOrTagParseError> {
        if reader.can_read() && reader.peek() == '#' {
            let cursor = reader.cursor();
            reader.skip();
            match read_identifier(reader) {
                Ok(tag_id) => {
                    let tag_key = TagKeyModel::create(self.registry_key.clone(), tag_id.clone());
                    self.registry_lookup
                        .get_tag(&tag_key)
                        .map(ResourceOrTagResultModel::tag)
                        .ok_or(ResourceOrTagParseError::UnknownTag {
                            id: tag_id,
                            registry: self.registry_key.clone(),
                            cursor: reader.cursor(),
                        })
                        .inspect_err(|_error| reader.set_cursor(cursor))
                }
                Err(error) => {
                    reader.set_cursor(cursor);
                    Err(ResourceOrTagParseError::Identifier(error))
                }
            }
        } else {
            let resource_id =
                read_identifier(reader).map_err(ResourceOrTagParseError::Identifier)?;
            let resource_key =
                ResourceKeyModel::create(self.registry_key.clone(), resource_id.clone());
            self.registry_lookup
                .get_resource(&resource_key)
                .map(ResourceOrTagResultModel::resource)
                .ok_or(ResourceOrTagParseError::UnknownResource {
                    id: resource_id,
                    registry: self.registry_key.clone(),
                    cursor: reader.cursor(),
                })
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

pub fn get_resource_or_tag(
    context: &CommandContextModel,
    name: &str,
    registry_key: &str,
) -> Result<ResourceOrTagResultModel, ResourceOrTagLookupError> {
    let expected_registry =
        CommandIdentifierModel::parse(registry_key).expect("valid registry key");
    let argument = context
        .arguments
        .get(name)
        .ok_or(ResourceOrTagLookupError::MissingArgument)?
        .clone();
    argument
        .cast(&expected_registry)
        .ok_or_else(|| match argument.unwrap() {
            Ok(holder) => ResourceOrTagLookupError::InvalidResourceType {
                id: Box::new(holder.key().identifier().clone()),
                actual_registry: Box::new(holder.key().registry().clone()),
                expected_registry: Box::new(expected_registry),
            },
            Err(tag) => ResourceOrTagLookupError::InvalidTagType {
                id: Box::new(tag.key().location().clone()),
                actual_registry: Box::new(tag.key().registry().clone()),
                expected_registry: Box::new(expected_registry),
            },
        })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBuildContextModel {
    registries: HashMap<CommandIdentifierModel, HolderLookupModel>,
}

impl CommandBuildContextModel {
    pub fn with_registry(mut self, registry: &str, elements: &[(&str, &[&str])]) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry).expect("valid registry key");
        let holders: Vec<_> = elements
            .iter()
            .map(|(id, tags)| HolderReferenceModel {
                key: ResourceKeyModel::create(
                    registry_key.clone(),
                    CommandIdentifierModel::parse(id).expect("valid element id"),
                ),
                tags: tags
                    .iter()
                    .map(|tag| {
                        TagKeyModel::create(
                            registry_key.clone(),
                            CommandIdentifierModel::parse(tag).expect("valid tag id"),
                        )
                    })
                    .collect(),
            })
            .collect();

        let mut tag_members = HashMap::<TagKeyModel, Vec<HolderReferenceModel>>::new();
        for holder in &holders {
            for tag in &holder.tags {
                tag_members
                    .entry(tag.clone())
                    .or_default()
                    .push(holder.clone());
            }
        }

        let tags = tag_members
            .into_iter()
            .map(|(key, values)| HolderSetNamedModel { key, values })
            .collect();
        self.registries.insert(
            registry_key,
            HolderLookupModel {
                elements: holders,
                tags,
            },
        );
        self
    }

    fn lookup_or_throw(&self, registry: &CommandIdentifierModel) -> HolderLookupModel {
        self.registries.get(registry).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, ResourceOrTagResultModel>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn with_result(
        mut self,
        name: impl Into<String>,
        result: ResourceOrTagResultModel,
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
    registries: HashMap<CommandIdentifierModel, HolderLookupModel>,
}

impl SharedSuggestionProviderModel {
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
                registry.tags.iter().map(|tag| tag.key().location()),
                remaining,
                SuggestionPrefix::Tag,
            ));
        }
        if element_type.should_suggest_elements() {
            suggestions.extend(suggest_resource(
                registry
                    .elements
                    .iter()
                    .map(|holder| holder.key().identifier()),
                remaining,
                SuggestionPrefix::Element,
            ));
        }
        suggestions
    }
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HolderLookupModel {
    elements: Vec<HolderReferenceModel>,
    tags: Vec<HolderSetNamedModel>,
}

impl HolderLookupModel {
    fn get_resource(&self, key: &ResourceKeyModel) -> Option<HolderReferenceModel> {
        self.elements
            .iter()
            .find(|holder| holder.key() == key)
            .cloned()
    }

    fn get_tag(&self, key: &TagKeyModel) -> Option<HolderSetNamedModel> {
        self.tags.iter().find(|tag| tag.key() == key).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderReferenceModel {
    key: ResourceKeyModel,
    tags: Vec<TagKeyModel>,
}

impl HolderReferenceModel {
    pub fn key(&self) -> &ResourceKeyModel {
        &self.key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSetNamedModel {
    key: TagKeyModel,
    values: Vec<HolderReferenceModel>,
}

impl HolderSetNamedModel {
    pub fn key(&self) -> &TagKeyModel {
        &self.key
    }

    fn contains(&self, holder: &HolderReferenceModel) -> bool {
        self.values.contains(holder)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOrTagResultModel {
    Resource(HolderReferenceModel),
    Tag(HolderSetNamedModel),
}

impl ResourceOrTagResultModel {
    fn resource(holder: HolderReferenceModel) -> Self {
        Self::Resource(holder)
    }

    fn tag(tag: HolderSetNamedModel) -> Self {
        Self::Tag(tag)
    }

    pub fn unwrap(&self) -> Result<HolderReferenceModel, HolderSetNamedModel> {
        match self {
            Self::Resource(holder) => Ok(holder.clone()),
            Self::Tag(tag) => Err(tag.clone()),
        }
    }

    pub fn cast(&self, registry_key: &CommandIdentifierModel) -> Option<Self> {
        match self {
            Self::Resource(holder) => holder
                .key()
                .is_for(registry_key)
                .then(|| Self::Resource(holder.clone())),
            Self::Tag(tag) => tag
                .key()
                .is_for(registry_key)
                .then(|| Self::Tag(tag.clone())),
        }
    }

    pub fn test(&self, holder: &HolderReferenceModel) -> bool {
        match self {
            Self::Resource(value) => holder == value,
            Self::Tag(tag) => tag.contains(holder),
        }
    }

    pub fn as_printable(&self) -> String {
        match self {
            Self::Resource(holder) => holder.key().identifier().to_string(),
            Self::Tag(tag) => format!("#{}", tag.key().location()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    fn is_for(&self, registry: &CommandIdentifierModel) -> bool {
        &self.registry == registry
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
    fn create(registry: CommandIdentifierModel, location: CommandIdentifierModel) -> Self {
        Self { registry, location }
    }

    fn is_for(&self, registry: &CommandIdentifierModel) -> bool {
        &self.registry == registry
    }

    pub fn registry(&self) -> &CommandIdentifierModel {
        &self.registry
    }

    pub fn location(&self) -> &CommandIdentifierModel {
        &self.location
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

    pub fn instantiate(&self, context: &CommandBuildContextModel) -> ResourceOrTagArgumentModel {
        ResourceOrTagArgumentModel::resource_or_tag(context, &self.registry_key.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOrTagParseError {
    Identifier(IdentifierArgumentParseError),
    UnknownResource {
        id: CommandIdentifierModel,
        registry: CommandIdentifierModel,
        cursor: usize,
    },
    UnknownTag {
        id: CommandIdentifierModel,
        registry: CommandIdentifierModel,
        cursor: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOrTagLookupError {
    MissingArgument,
    InvalidResourceType {
        id: Box<CommandIdentifierModel>,
        actual_registry: Box<CommandIdentifierModel>,
        expected_registry: Box<CommandIdentifierModel>,
    },
    InvalidTagType {
        id: Box<CommandIdentifierModel>,
        actual_registry: Box<CommandIdentifierModel>,
        expected_registry: Box<CommandIdentifierModel>,
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

fn build_context() -> CommandBuildContextModel {
    CommandBuildContextModel::default().with_registry(
        ENTITY_TYPE_REGISTRY,
        &[
            ("minecraft:skeleton", &["skeletons", "undead"]),
            ("minecraft:zombie", &["undead"]),
            ("custom:skeleton", &["skeletons"]),
        ],
    )
}

fn suggestion_source() -> CommandSourceModel {
    let context = build_context();
    let mut provider = SharedSuggestionProviderModel::default();
    for (registry, lookup) in context.registries {
        provider.registries.insert(registry, lookup);
    }
    CommandSourceModel::Shared(provider)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argument() -> ResourceOrTagArgumentModel {
        ResourceOrTagArgumentModel::resource_or_tag(&build_context(), ENTITY_TYPE_REGISTRY)
    }

    fn parse(input: &str) -> Result<(ResourceOrTagResultModel, usize), ResourceOrTagParseError> {
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
        assert_eq!(
            template.instantiate(&build_context()).info_template(),
            template
        );
    }

    #[test]
    fn java_parse_resource_resolves_existing_holder_reference() {
        let (result, cursor) = parse("skeleton trailing").unwrap();
        let holder = result.unwrap().unwrap();

        assert_eq!(holder.key().registry().to_string(), ENTITY_TYPE_REGISTRY);
        assert_eq!(holder.key().identifier().to_string(), "minecraft:skeleton");
        assert_eq!(result.as_printable(), "minecraft:skeleton");
        assert_eq!(cursor, "skeleton".len());
    }

    #[test]
    fn java_parse_tag_resolves_existing_named_holder_set() {
        let (result, cursor) = parse("#skeletons trailing").unwrap();
        let tag = result.unwrap().unwrap_err();

        assert_eq!(tag.key().registry().to_string(), ENTITY_TYPE_REGISTRY);
        assert_eq!(tag.key().location().to_string(), "minecraft:skeletons");
        assert_eq!(result.as_printable(), "#minecraft:skeletons");
        assert_eq!(cursor, "#skeletons".len());
    }

    #[test]
    fn java_parse_unknown_resource_preserves_identifier_cursor() {
        let mut reader = StringReaderModel::new("missing@tail");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(ResourceOrTagParseError::UnknownResource {
                id: CommandIdentifierModel::parse("minecraft:missing").unwrap(),
                registry: CommandIdentifierModel::parse(ENTITY_TYPE_REGISTRY).unwrap(),
                cursor: "missing".len(),
            })
        );
        assert_eq!(reader.cursor(), "missing".len());
    }

    #[test]
    fn java_parse_unknown_tag_and_tag_identifier_errors_reset_to_hash() {
        let mut unknown = StringReaderModel::new("#missing@tail");
        let unknown_error = argument().parse(&mut unknown);
        assert_eq!(
            unknown_error,
            Err(ResourceOrTagParseError::UnknownTag {
                id: CommandIdentifierModel::parse("minecraft:missing").unwrap(),
                registry: CommandIdentifierModel::parse(ENTITY_TYPE_REGISTRY).unwrap(),
                cursor: "#missing".len(),
            })
        );
        assert_eq!(unknown.cursor(), 0);

        let mut invalid = StringReaderModel::new("#..:bad trailing");
        let invalid_error = argument().parse(&mut invalid);
        assert_eq!(
            invalid_error,
            Err(ResourceOrTagParseError::Identifier(
                IdentifierArgumentParseError::InvalidIdentifier
            ))
        );
        assert_eq!(invalid.cursor(), 0);
    }

    #[test]
    fn java_resource_identifier_errors_reset_to_original_cursor() {
        let mut reader = StringReaderModel::new("..:bad trailing");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(ResourceOrTagParseError::Identifier(
                IdentifierArgumentParseError::InvalidIdentifier
            ))
        );
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
    fn java_context_getter_returns_cast_result_or_type_specific_exception() {
        let (resource, _) = parse("skeleton").unwrap();
        let (tag, _) = parse("#skeletons").unwrap();
        let resource_context =
            CommandContextModel::default().with_result("target", resource.clone());
        let tag_context = CommandContextModel::default().with_result("target", tag.clone());

        assert_eq!(
            get_resource_or_tag(&resource_context, "target", ENTITY_TYPE_REGISTRY),
            Ok(resource)
        );
        assert_eq!(
            get_resource_or_tag(&resource_context, "target", CONFIGURED_FEATURE_REGISTRY),
            Err(ResourceOrTagLookupError::InvalidResourceType {
                id: Box::new(CommandIdentifierModel::parse("minecraft:skeleton").unwrap()),
                actual_registry: Box::new(
                    CommandIdentifierModel::parse(ENTITY_TYPE_REGISTRY).unwrap()
                ),
                expected_registry: Box::new(
                    CommandIdentifierModel::parse(CONFIGURED_FEATURE_REGISTRY).unwrap()
                ),
            })
        );
        assert_eq!(
            get_resource_or_tag(&tag_context, "target", CONFIGURED_FEATURE_REGISTRY),
            Err(ResourceOrTagLookupError::InvalidTagType {
                id: Box::new(CommandIdentifierModel::parse("minecraft:skeletons").unwrap()),
                actual_registry: Box::new(
                    CommandIdentifierModel::parse(ENTITY_TYPE_REGISTRY).unwrap()
                ),
                expected_registry: Box::new(
                    CommandIdentifierModel::parse(CONFIGURED_FEATURE_REGISTRY).unwrap()
                ),
            })
        );
        assert_eq!(
            get_resource_or_tag(&tag_context, "missing", ENTITY_TYPE_REGISTRY),
            Err(ResourceOrTagLookupError::MissingArgument)
        );
    }

    #[test]
    fn java_result_test_uses_holder_equality_or_named_tag_membership() {
        let (resource, _) = parse("skeleton").unwrap();
        let (tag, _) = parse("#skeletons").unwrap();
        let skeleton = argument()
            .registry_lookup
            .get_resource(&ResourceKeyModel::create(
                CommandIdentifierModel::parse(ENTITY_TYPE_REGISTRY).unwrap(),
                CommandIdentifierModel::parse("minecraft:skeleton").unwrap(),
            ))
            .unwrap();
        let zombie = argument()
            .registry_lookup
            .get_resource(&ResourceKeyModel::create(
                CommandIdentifierModel::parse(ENTITY_TYPE_REGISTRY).unwrap(),
                CommandIdentifierModel::parse("minecraft:zombie").unwrap(),
            ))
            .unwrap();

        assert!(resource.test(&skeleton));
        assert!(!resource.test(&zombie));
        assert!(tag.test(&skeleton));
        assert!(!tag.test(&zombie));
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
