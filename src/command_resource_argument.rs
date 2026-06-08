use std::collections::HashMap;

use crate::command_identifier_argument::{
    CommandIdentifierModel, IdentifierArgumentParseError, StringReaderModel,
};

const ATTRIBUTE_REGISTRY: &str = "minecraft:attribute";
const CONFIGURED_FEATURE_REGISTRY: &str = "minecraft:worldgen/configured_feature";
const STRUCTURE_REGISTRY: &str = "minecraft:worldgen/structure";
const ENTITY_TYPE_REGISTRY: &str = "minecraft:entity_type";
const MOB_EFFECT_REGISTRY: &str = "minecraft:mob_effect";
const ENCHANTMENT_REGISTRY: &str = "minecraft:enchantment";
const WORLD_CLOCK_REGISTRY: &str = "minecraft:world_clock";
const TIMELINE_REGISTRY: &str = "minecraft:timeline";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceArgumentModel {
    registry_key: CommandIdentifierModel,
    registry_lookup: HolderLookupModel,
}

impl ResourceArgumentModel {
    pub fn resource(context: &CommandBuildContextModel, registry_key: &str) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry_key)
            .expect("valid resource argument registry key");
        Self {
            registry_lookup: context.lookup_or_throw(&registry_key),
            registry_key,
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<HolderReferenceModel, ResourceArgumentParseError> {
        let resource_id =
            CommandIdentifierModel::read(reader).map_err(ResourceArgumentParseError::Identifier)?;
        let key = ResourceKeyModel::create(self.registry_key.clone(), resource_id.clone());
        self.registry_lookup
            .get(&key)
            .ok_or(ResourceArgumentParseError::UnknownResource {
                id: resource_id,
                registry: self.registry_key.clone(),
                cursor: reader.cursor(),
            })
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
        ["foo", "foo:bar", "012"]
    }

    pub fn info_template(&self) -> InfoTemplateModel {
        InfoTemplateModel {
            registry_key: self.registry_key.clone(),
        }
    }
}

pub fn get_resource(
    context: &CommandContextModel,
    name: &str,
    registry_key: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    let expected_registry =
        CommandIdentifierModel::parse(registry_key).expect("valid registry key");
    let argument = context
        .arguments
        .get(name)
        .ok_or(ResourceLookupError::MissingArgument)?
        .clone();
    let argument_key = argument.key();
    if argument_key.is_for(&expected_registry) {
        Ok(argument)
    } else {
        Err(ResourceLookupError::InvalidResourceType {
            id: Box::new(argument_key.identifier().clone()),
            actual_registry: Box::new(argument_key.registry().clone()),
            expected_registry: Box::new(expected_registry),
        })
    }
}

pub fn get_attribute(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, ATTRIBUTE_REGISTRY)
}

pub fn get_configured_feature(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, CONFIGURED_FEATURE_REGISTRY)
}

pub fn get_structure(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, STRUCTURE_REGISTRY)
}

pub fn get_entity_type(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, ENTITY_TYPE_REGISTRY)
}

pub fn get_summonable_entity_type(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    let result = get_resource(context, name, ENTITY_TYPE_REGISTRY)?;
    if result.value().can_summon {
        Ok(result)
    } else {
        Err(ResourceLookupError::NotSummonableEntity(
            result.key().identifier().to_string(),
        ))
    }
}

pub fn get_mob_effect(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, MOB_EFFECT_REGISTRY)
}

pub fn get_enchantment(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, ENCHANTMENT_REGISTRY)
}

pub fn get_clock(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, WORLD_CLOCK_REGISTRY)
}

pub fn get_timeline(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceLookupError> {
    get_resource(context, name, TIMELINE_REGISTRY)
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
                    .map(|(id, value)| HolderReferenceModel {
                        key: ResourceKeyModel::create(
                            registry_key.clone(),
                            CommandIdentifierModel::parse(id).expect("valid element id"),
                        ),
                        value: value.clone(),
                    })
                    .collect(),
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
    arguments: HashMap<String, HolderReferenceModel>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn with_holder(mut self, name: impl Into<String>, holder: HolderReferenceModel) -> Self {
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
                    lookup
                        .list_elements()
                        .map(|holder| holder.key().identifier()),
                    remaining,
                )
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HolderLookupModel {
    elements: Vec<HolderReferenceModel>,
}

impl HolderLookupModel {
    fn get(&self, key: &ResourceKeyModel) -> Option<HolderReferenceModel> {
        self.elements
            .iter()
            .find(|holder| holder.key() == key)
            .cloned()
    }

    fn list_elements(&self) -> impl Iterator<Item = &HolderReferenceModel> {
        self.elements.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderReferenceModel {
    key: ResourceKeyModel,
    value: ResourceValueModel,
}

impl HolderReferenceModel {
    pub fn key(&self) -> &ResourceKeyModel {
        &self.key
    }

    pub fn value(&self) -> &ResourceValueModel {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceValueModel {
    kind: ResourceValueKind,
    can_summon: bool,
}

impl ResourceValueModel {
    pub fn simple(kind: ResourceValueKind) -> Self {
        Self {
            kind,
            can_summon: true,
        }
    }

    pub fn entity_type(can_summon: bool) -> Self {
        Self {
            kind: ResourceValueKind::EntityType,
            can_summon,
        }
    }

    pub fn kind(&self) -> ResourceValueKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceValueKind {
    Attribute,
    ConfiguredFeature,
    Structure,
    EntityType,
    MobEffect,
    Enchantment,
    WorldClock,
    Timeline,
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

    pub fn instantiate(&self, context: &CommandBuildContextModel) -> ResourceArgumentModel {
        ResourceArgumentModel::resource(context, &self.registry_key.to_string())
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceArgumentParseError {
    Identifier(IdentifierArgumentParseError),
    UnknownResource {
        id: CommandIdentifierModel,
        registry: CommandIdentifierModel,
        cursor: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceLookupError {
    MissingArgument,
    InvalidResourceType {
        id: Box<CommandIdentifierModel>,
        actual_registry: Box<CommandIdentifierModel>,
        expected_registry: Box<CommandIdentifierModel>,
    },
    NotSummonableEntity(String),
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

fn build_context() -> CommandBuildContextModel {
    CommandBuildContextModel::default()
        .with_registry(
            ATTRIBUTE_REGISTRY,
            &[(
                "minecraft:generic.max_health",
                ResourceValueModel::simple(ResourceValueKind::Attribute),
            )],
        )
        .with_registry(
            CONFIGURED_FEATURE_REGISTRY,
            &[
                (
                    "minecraft:ore/diamond",
                    ResourceValueModel::simple(ResourceValueKind::ConfiguredFeature),
                ),
                (
                    "custom:lake",
                    ResourceValueModel::simple(ResourceValueKind::ConfiguredFeature),
                ),
            ],
        )
        .with_registry(
            STRUCTURE_REGISTRY,
            &[(
                "minecraft:village",
                ResourceValueModel::simple(ResourceValueKind::Structure),
            )],
        )
        .with_registry(
            ENTITY_TYPE_REGISTRY,
            &[
                ("minecraft:pig", ResourceValueModel::entity_type(true)),
                ("minecraft:player", ResourceValueModel::entity_type(false)),
            ],
        )
        .with_registry(
            MOB_EFFECT_REGISTRY,
            &[(
                "minecraft:speed",
                ResourceValueModel::simple(ResourceValueKind::MobEffect),
            )],
        )
        .with_registry(
            ENCHANTMENT_REGISTRY,
            &[(
                "minecraft:sharpness",
                ResourceValueModel::simple(ResourceValueKind::Enchantment),
            )],
        )
        .with_registry(
            WORLD_CLOCK_REGISTRY,
            &[(
                "minecraft:overworld",
                ResourceValueModel::simple(ResourceValueKind::WorldClock),
            )],
        )
        .with_registry(
            TIMELINE_REGISTRY,
            &[(
                "minecraft:story",
                ResourceValueModel::simple(ResourceValueKind::Timeline),
            )],
        )
}

fn shared_source() -> CommandSourceModel {
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

    fn argument(registry: &str) -> ResourceArgumentModel {
        ResourceArgumentModel::resource(&build_context(), registry)
    }

    fn parse(
        registry: &str,
        input: &str,
    ) -> Result<(HolderReferenceModel, usize), ResourceArgumentParseError> {
        let mut reader = StringReaderModel::new(input);
        let holder = argument(registry).parse(&mut reader)?;
        Ok((holder, reader.cursor()))
    }

    fn context_with(name: &str, holder: HolderReferenceModel) -> CommandContextModel {
        CommandContextModel::default()
            .with_source(shared_source())
            .with_holder(name, holder)
    }

    #[test]
    fn java_factory_examples_and_info_template_match_source() {
        let argument = argument(CONFIGURED_FEATURE_REGISTRY);
        assert_eq!(argument.examples(), ["foo", "foo:bar", "012"]);

        let template = argument.info_template();
        assert_eq!(template.serialize_to_network(), CONFIGURED_FEATURE_REGISTRY);
        assert_eq!(
            template.serialize_to_json_registry(),
            CONFIGURED_FEATURE_REGISTRY
        );
        assert_eq!(
            template.instantiate(&build_context()).info_template(),
            template
        );
    }

    #[test]
    fn java_parse_delegates_to_identifier_read_and_returns_holder_reference() {
        let (defaulted, cursor) =
            parse(CONFIGURED_FEATURE_REGISTRY, "ore/diamond trailing").unwrap();
        assert_eq!(
            defaulted.key().registry().to_string(),
            CONFIGURED_FEATURE_REGISTRY
        );
        assert_eq!(
            defaulted.key().identifier().to_string(),
            "minecraft:ore/diamond"
        );
        assert_eq!(
            defaulted.value().kind(),
            ResourceValueKind::ConfiguredFeature
        );
        assert_eq!(cursor, "ore/diamond".len());

        let (explicit, cursor) = parse(CONFIGURED_FEATURE_REGISTRY, "custom:lake").unwrap();
        assert_eq!(explicit.key().identifier().to_string(), "custom:lake");
        assert_eq!(cursor, "custom:lake".len());
    }

    #[test]
    fn java_parse_resets_cursor_only_for_identifier_syntax_errors() {
        let mut reader = StringReaderModel::new("..:bad trailing");
        let error = argument(CONFIGURED_FEATURE_REGISTRY).parse(&mut reader);

        assert_eq!(
            error,
            Err(ResourceArgumentParseError::Identifier(
                IdentifierArgumentParseError::InvalidIdentifier
            ))
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_parse_unknown_resource_uses_reader_context_after_successful_identifier() {
        let mut reader = StringReaderModel::new("missing@tail");
        let error = argument(CONFIGURED_FEATURE_REGISTRY).parse(&mut reader);

        assert_eq!(
            error,
            Err(ResourceArgumentParseError::UnknownResource {
                id: CommandIdentifierModel::parse("minecraft:missing").unwrap(),
                registry: CommandIdentifierModel::parse(CONFIGURED_FEATURE_REGISTRY).unwrap(),
                cursor: "missing".len(),
            })
        );
        assert_eq!(reader.cursor(), "missing".len());
    }

    #[test]
    fn java_get_resource_accepts_matching_registry_and_rejects_wrong_registry() {
        let holder = parse(CONFIGURED_FEATURE_REGISTRY, "ore/diamond").unwrap().0;
        let context = context_with("target", holder.clone());

        assert_eq!(
            get_resource(&context, "target", CONFIGURED_FEATURE_REGISTRY),
            Ok(holder)
        );
        assert_eq!(
            get_resource(&context, "target", STRUCTURE_REGISTRY),
            Err(ResourceLookupError::InvalidResourceType {
                id: Box::new(CommandIdentifierModel::parse("minecraft:ore/diamond").unwrap()),
                actual_registry: Box::new(
                    CommandIdentifierModel::parse(CONFIGURED_FEATURE_REGISTRY).unwrap()
                ),
                expected_registry: Box::new(
                    CommandIdentifierModel::parse(STRUCTURE_REGISTRY).unwrap()
                ),
            })
        );
        assert_eq!(
            get_resource(&context, "missing", CONFIGURED_FEATURE_REGISTRY),
            Err(ResourceLookupError::MissingArgument)
        );
    }

    #[test]
    fn java_typed_getters_delegate_to_get_resource_registry_checks() {
        type Getter =
            fn(&CommandContextModel, &str) -> Result<HolderReferenceModel, ResourceLookupError>;
        type GetterCase = (Getter, &'static str, &'static str, ResourceValueKind);

        let cases: [GetterCase; 8] = [
            (
                get_attribute,
                ATTRIBUTE_REGISTRY,
                "minecraft:generic.max_health",
                ResourceValueKind::Attribute,
            ),
            (
                get_configured_feature,
                CONFIGURED_FEATURE_REGISTRY,
                "minecraft:ore/diamond",
                ResourceValueKind::ConfiguredFeature,
            ),
            (
                get_structure,
                STRUCTURE_REGISTRY,
                "minecraft:village",
                ResourceValueKind::Structure,
            ),
            (
                get_entity_type,
                ENTITY_TYPE_REGISTRY,
                "minecraft:pig",
                ResourceValueKind::EntityType,
            ),
            (
                get_mob_effect,
                MOB_EFFECT_REGISTRY,
                "minecraft:speed",
                ResourceValueKind::MobEffect,
            ),
            (
                get_enchantment,
                ENCHANTMENT_REGISTRY,
                "minecraft:sharpness",
                ResourceValueKind::Enchantment,
            ),
            (
                get_clock,
                WORLD_CLOCK_REGISTRY,
                "minecraft:overworld",
                ResourceValueKind::WorldClock,
            ),
            (
                get_timeline,
                TIMELINE_REGISTRY,
                "minecraft:story",
                ResourceValueKind::Timeline,
            ),
        ];

        for (getter, registry, id, expected_kind) in cases {
            let holder = parse(registry, id).unwrap().0;
            let context = context_with("target", holder.clone());

            assert_eq!(
                getter(&context, "target").unwrap().value().kind(),
                expected_kind
            );
            assert_eq!(getter(&context, "target").unwrap(), holder);
        }
    }

    #[test]
    fn java_summonable_entity_type_checks_entity_type_summon_flag() {
        let pig = parse(ENTITY_TYPE_REGISTRY, "pig").unwrap().0;
        let pig_context = context_with("entity", pig.clone());
        assert_eq!(get_summonable_entity_type(&pig_context, "entity"), Ok(pig));

        let player = parse(ENTITY_TYPE_REGISTRY, "player").unwrap().0;
        let player_context = context_with("entity", player);
        assert_eq!(
            get_summonable_entity_type(&player_context, "entity"),
            Err(ResourceLookupError::NotSummonableEntity(
                "minecraft:player".to_string()
            ))
        );

        let feature = parse(CONFIGURED_FEATURE_REGISTRY, "ore/diamond").unwrap().0;
        let wrong_context = context_with("entity", feature);
        assert!(matches!(
            get_summonable_entity_type(&wrong_context, "entity"),
            Err(ResourceLookupError::InvalidResourceType { .. })
        ));
    }

    #[test]
    fn java_suggestions_delegate_to_shared_registry_element_suggestions() {
        let context = CommandContextModel::default().with_source(shared_source());
        let argument = argument(CONFIGURED_FEATURE_REGISTRY);

        assert_eq!(
            argument.list_suggestions(&context, ""),
            ["minecraft:ore/diamond", "custom:lake"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "ore/"),
            ["minecraft:ore/diamond"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "custom:l"),
            ["custom:lake"]
        );

        let other_source = CommandContextModel::default().with_source(CommandSourceModel::Other);
        assert_eq!(
            argument.list_suggestions(&other_source, ""),
            Vec::<String>::new()
        );

        assert!(!ElementSuggestionType::Tags.should_suggest_elements());
        assert!(ElementSuggestionType::All.should_suggest_elements());
    }
}
