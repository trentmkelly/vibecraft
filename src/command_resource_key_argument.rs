use std::collections::HashMap;

use crate::command_identifier_argument::{
    CommandIdentifierModel, IdentifierArgumentParseError, StringReaderModel,
};

const CONFIGURED_FEATURE_REGISTRY: &str = "minecraft:worldgen/configured_feature";
const STRUCTURE_REGISTRY: &str = "minecraft:worldgen/structure";
const TEMPLATE_POOL_REGISTRY: &str = "minecraft:worldgen/template_pool";
const RECIPE_REGISTRY: &str = "minecraft:recipe";
const ADVANCEMENT_REGISTRY: &str = "minecraft:advancement";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceKeyArgumentModel {
    registry_key: CommandIdentifierModel,
}

impl ResourceKeyArgumentModel {
    pub fn key(registry_key: &str) -> Self {
        Self {
            registry_key: CommandIdentifierModel::parse(registry_key)
                .expect("valid registry key identifier"),
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<ResourceKeyModel, IdentifierArgumentParseError> {
        let resource_id = CommandIdentifierModel::read(reader)?;
        Ok(ResourceKeyModel::create(
            self.registry_key.clone(),
            resource_id,
        ))
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

pub fn get_registry_key(
    context: &CommandContextModel,
    name: &str,
    registry_key: &str,
    error_kind: ResourceKeyErrorKind,
) -> Result<ResourceKeyModel, ResourceKeyLookupError> {
    let expected_registry =
        CommandIdentifierModel::parse(registry_key).expect("valid registry key");
    let argument = context
        .arguments
        .get(name)
        .ok_or(ResourceKeyLookupError::MissingArgument)?
        .clone();

    if argument.is_for(&expected_registry) {
        Ok(argument)
    } else {
        Err(ResourceKeyLookupError::Dynamic {
            kind: error_kind,
            value: argument.identifier().clone(),
        })
    }
}

pub fn get_configured_feature(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceKeyLookupError> {
    resolve_key(
        context,
        name,
        CONFIGURED_FEATURE_REGISTRY,
        ResourceKeyErrorKind::Feature,
    )
}

pub fn get_structure(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceKeyLookupError> {
    resolve_key(
        context,
        name,
        STRUCTURE_REGISTRY,
        ResourceKeyErrorKind::Structure,
    )
}

pub fn get_structure_template_pool(
    context: &CommandContextModel,
    name: &str,
) -> Result<HolderReferenceModel, ResourceKeyLookupError> {
    resolve_key(
        context,
        name,
        TEMPLATE_POOL_REGISTRY,
        ResourceKeyErrorKind::TemplatePool,
    )
}

pub fn get_recipe(
    context: &CommandContextModel,
    name: &str,
) -> Result<RecipeHolderModel, ResourceKeyLookupError> {
    let key = get_registry_key(context, name, RECIPE_REGISTRY, ResourceKeyErrorKind::Recipe)?;
    let server = context.server()?;
    server
        .recipe_manager
        .by_key(&key)
        .ok_or_else(|| ResourceKeyLookupError::Dynamic {
            kind: ResourceKeyErrorKind::Recipe,
            value: key.identifier().clone(),
        })
}

pub fn get_advancement(
    context: &CommandContextModel,
    name: &str,
) -> Result<AdvancementHolderModel, ResourceKeyLookupError> {
    let key = get_registry_key(
        context,
        name,
        ADVANCEMENT_REGISTRY,
        ResourceKeyErrorKind::Advancement,
    )?;
    let server = context.server()?;
    server
        .advancements
        .get(key.identifier())
        .ok_or_else(|| ResourceKeyLookupError::Dynamic {
            kind: ResourceKeyErrorKind::Advancement,
            value: key.identifier().clone(),
        })
}

fn resolve_key(
    context: &CommandContextModel,
    name: &str,
    registry_key: &str,
    error_kind: ResourceKeyErrorKind,
) -> Result<HolderReferenceModel, ResourceKeyLookupError> {
    let key = get_registry_key(context, name, registry_key, error_kind)?;
    let server = context.server()?;
    server
        .registry_access
        .lookup_or_throw(key.registry())
        .get(&key)
        .ok_or_else(|| ResourceKeyLookupError::Dynamic {
            kind: error_kind,
            value: key.identifier().clone(),
        })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, ResourceKeyModel>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn with_resource_key(mut self, name: impl Into<String>, value: ResourceKeyModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }

    pub fn with_source(mut self, source: CommandSourceModel) -> Self {
        self.source = source;
        self
    }

    fn server(&self) -> Result<&ServerModel, ResourceKeyLookupError> {
        match &self.source {
            CommandSourceModel::Shared(provider) => Ok(&provider.server),
            CommandSourceModel::Other => Err(ResourceKeyLookupError::MissingServer),
        }
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
    server: ServerModel,
}

impl SharedSuggestionProviderModel {
    pub fn with_registry(mut self, registry: &str, elements: &[&str]) -> Self {
        self.server = self.server.with_registry(registry, elements);
        self
    }

    pub fn with_recipe(mut self, recipe: &str) -> Self {
        self.server = self.server.with_recipe(recipe);
        self
    }

    pub fn with_advancement(mut self, advancement: &str) -> Self {
        self.server = self.server.with_advancement(advancement);
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

        self.server
            .registry_access
            .registries
            .get(registry)
            .map(|registry| {
                suggest_resource(
                    registry
                        .elements
                        .iter()
                        .map(|holder| holder.key.identifier()),
                    remaining,
                )
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerModel {
    registry_access: RegistryAccessModel,
    recipe_manager: RecipeManagerModel,
    advancements: AdvancementManagerModel,
}

impl ServerModel {
    fn with_registry(mut self, registry: &str, elements: &[&str]) -> Self {
        self.registry_access = self.registry_access.with_registry(registry, elements);
        self
    }

    fn with_recipe(mut self, recipe: &str) -> Self {
        self.recipe_manager = self.recipe_manager.with_recipe(recipe);
        self
    }

    fn with_advancement(mut self, advancement: &str) -> Self {
        self.advancements = self.advancements.with_advancement(advancement);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryAccessModel {
    registries: HashMap<CommandIdentifierModel, RegistryModel>,
}

impl RegistryAccessModel {
    fn with_registry(mut self, registry: &str, elements: &[&str]) -> Self {
        let registry_key = CommandIdentifierModel::parse(registry).expect("valid registry key");
        let registry = RegistryModel {
            elements: elements
                .iter()
                .map(|element| HolderReferenceModel {
                    key: ResourceKeyModel::create(
                        registry_key.clone(),
                        CommandIdentifierModel::parse(element).expect("valid element key"),
                    ),
                    value_name: format!("holder:{element}"),
                })
                .collect(),
        };
        self.registries.insert(registry_key, registry);
        self
    }

    fn lookup_or_throw(&self, registry: &CommandIdentifierModel) -> RegistryModel {
        self.registries.get(registry).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryModel {
    elements: Vec<HolderReferenceModel>,
}

impl RegistryModel {
    fn get(&self, key: &ResourceKeyModel) -> Option<HolderReferenceModel> {
        self.elements
            .iter()
            .find(|holder| holder.key() == key)
            .cloned()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeManagerModel {
    recipes: HashMap<ResourceKeyModel, RecipeHolderModel>,
}

impl RecipeManagerModel {
    fn with_recipe(mut self, recipe: &str) -> Self {
        let key = ResourceKeyModel::create(
            CommandIdentifierModel::parse(RECIPE_REGISTRY).expect("valid registry key"),
            CommandIdentifierModel::parse(recipe).expect("valid recipe id"),
        );
        self.recipes.insert(
            key.clone(),
            RecipeHolderModel {
                key,
                value_name: format!("recipe:{recipe}"),
            },
        );
        self
    }

    fn by_key(&self, key: &ResourceKeyModel) -> Option<RecipeHolderModel> {
        self.recipes.get(key).cloned()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementManagerModel {
    advancements: HashMap<CommandIdentifierModel, AdvancementHolderModel>,
}

impl AdvancementManagerModel {
    fn with_advancement(mut self, advancement: &str) -> Self {
        let id = CommandIdentifierModel::parse(advancement).expect("valid advancement id");
        self.advancements.insert(
            id.clone(),
            AdvancementHolderModel {
                id,
                value_name: format!("advancement:{advancement}"),
            },
        );
        self
    }

    fn get(&self, id: &CommandIdentifierModel) -> Option<AdvancementHolderModel> {
        self.advancements.get(id).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderReferenceModel {
    key: ResourceKeyModel,
    value_name: String,
}

impl HolderReferenceModel {
    pub fn key(&self) -> &ResourceKeyModel {
        &self.key
    }

    pub fn value_name(&self) -> &str {
        &self.value_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeHolderModel {
    key: ResourceKeyModel,
    value_name: String,
}

impl RecipeHolderModel {
    pub fn key(&self) -> &ResourceKeyModel {
        &self.key
    }

    pub fn value_name(&self) -> &str {
        &self.value_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementHolderModel {
    id: CommandIdentifierModel,
    value_name: String,
}

impl AdvancementHolderModel {
    pub fn id(&self) -> &CommandIdentifierModel {
        &self.id
    }

    pub fn value_name(&self) -> &str {
        &self.value_name
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

    pub fn is_for(&self, registry: &CommandIdentifierModel) -> bool {
        &self.registry == registry
    }

    pub fn cast(&self, registry: &CommandIdentifierModel) -> Option<Self> {
        self.is_for(registry).then(|| self.clone())
    }

    pub fn identifier(&self) -> &CommandIdentifierModel {
        &self.identifier
    }

    pub fn registry(&self) -> &CommandIdentifierModel {
        &self.registry
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

    pub fn instantiate(&self) -> ResourceKeyArgumentModel {
        ResourceKeyArgumentModel::key(&self.registry_key.to_string())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKeyErrorKind {
    Feature,
    Structure,
    TemplatePool,
    Recipe,
    Advancement,
}

impl ResourceKeyErrorKind {
    pub fn translation_key(self) -> &'static str {
        match self {
            Self::Feature => "commands.place.feature.invalid",
            Self::Structure => "commands.place.structure.invalid",
            Self::TemplatePool => "commands.place.jigsaw.invalid",
            Self::Recipe => "recipe.notFound",
            Self::Advancement => "advancement.advancementNotFound",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKeyLookupError {
    MissingArgument,
    MissingServer,
    Dynamic {
        kind: ResourceKeyErrorKind,
        value: CommandIdentifierModel,
    },
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

fn source() -> CommandSourceModel {
    CommandSourceModel::Shared(
        SharedSuggestionProviderModel::default()
            .with_registry(
                CONFIGURED_FEATURE_REGISTRY,
                &["minecraft:ore/diamond", "custom:lake"],
            )
            .with_registry(STRUCTURE_REGISTRY, &["minecraft:village", "custom:tower"])
            .with_registry(
                TEMPLATE_POOL_REGISTRY,
                &["minecraft:village/plains/town_centers"],
            )
            .with_recipe("minecraft:bread")
            .with_advancement("minecraft:story/root"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource_key(registry: &str, id: &str) -> ResourceKeyModel {
        ResourceKeyModel::create(
            CommandIdentifierModel::parse(registry).unwrap(),
            CommandIdentifierModel::parse(id).unwrap(),
        )
    }

    fn context_with(name: &str, key: ResourceKeyModel) -> CommandContextModel {
        CommandContextModel::default()
            .with_source(source())
            .with_resource_key(name, key)
    }

    fn parse(input: &str) -> Result<(ResourceKeyModel, usize), IdentifierArgumentParseError> {
        let mut reader = StringReaderModel::new(input);
        let key = ResourceKeyArgumentModel::key(CONFIGURED_FEATURE_REGISTRY).parse(&mut reader)?;
        Ok((key, reader.cursor()))
    }

    #[test]
    fn java_factory_examples_and_info_template_match_source() {
        let argument = ResourceKeyArgumentModel::key(CONFIGURED_FEATURE_REGISTRY);
        assert_eq!(argument.examples(), ["foo", "foo:bar", "012"]);

        let template = argument.info_template();
        assert_eq!(template.serialize_to_network(), CONFIGURED_FEATURE_REGISTRY);
        assert_eq!(
            template.serialize_to_json_registry(),
            CONFIGURED_FEATURE_REGISTRY
        );
        assert_eq!(template.instantiate().info_template(), template);
    }

    #[test]
    fn java_parse_delegates_to_identifier_read_and_wraps_registry_key() {
        let (defaulted, defaulted_cursor) = parse("ore/diamond trailing").unwrap();
        assert_eq!(
            defaulted.registry().to_string(),
            CONFIGURED_FEATURE_REGISTRY
        );
        assert_eq!(defaulted.identifier().to_string(), "minecraft:ore/diamond");
        assert_eq!(defaulted_cursor, "ore/diamond".len());

        let (explicit, explicit_cursor) = parse("custom:lake").unwrap();
        assert_eq!(explicit.registry().to_string(), CONFIGURED_FEATURE_REGISTRY);
        assert_eq!(explicit.identifier().to_string(), "custom:lake");
        assert_eq!(explicit_cursor, "custom:lake".len());
    }

    #[test]
    fn java_invalid_identifier_parse_resets_cursor() {
        let mut reader = StringReaderModel::new("..:bad trailing");
        let error = ResourceKeyArgumentModel::key(CONFIGURED_FEATURE_REGISTRY).parse(&mut reader);

        assert_eq!(error, Err(IdentifierArgumentParseError::InvalidIdentifier));
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_get_registry_key_casts_by_registry_or_throws_given_exception() {
        let key = resource_key(CONFIGURED_FEATURE_REGISTRY, "minecraft:ore/diamond");
        let context = context_with("target", key.clone());

        assert_eq!(
            get_registry_key(
                &context,
                "target",
                CONFIGURED_FEATURE_REGISTRY,
                ResourceKeyErrorKind::Feature
            ),
            Ok(key.clone())
        );
        assert_eq!(
            key.cast(&CommandIdentifierModel::parse(CONFIGURED_FEATURE_REGISTRY).unwrap()),
            Some(key.clone())
        );
        assert_eq!(
            key.cast(&CommandIdentifierModel::parse(STRUCTURE_REGISTRY).unwrap()),
            None
        );

        assert_eq!(
            get_registry_key(
                &context,
                "target",
                STRUCTURE_REGISTRY,
                ResourceKeyErrorKind::Structure
            ),
            Err(ResourceKeyLookupError::Dynamic {
                kind: ResourceKeyErrorKind::Structure,
                value: CommandIdentifierModel::parse("minecraft:ore/diamond").unwrap(),
            })
        );
        assert_eq!(
            get_registry_key(
                &context,
                "missing",
                CONFIGURED_FEATURE_REGISTRY,
                ResourceKeyErrorKind::Feature
            ),
            Err(ResourceKeyLookupError::MissingArgument)
        );
    }

    #[test]
    fn java_resolve_key_helpers_read_server_registries() {
        let feature_context = context_with(
            "target",
            resource_key(CONFIGURED_FEATURE_REGISTRY, "minecraft:ore/diamond"),
        );
        let feature = get_configured_feature(&feature_context, "target").unwrap();
        assert_eq!(feature.value_name(), "holder:minecraft:ore/diamond");

        let structure_context =
            context_with("target", resource_key(STRUCTURE_REGISTRY, "custom:tower"));
        let structure = get_structure(&structure_context, "target").unwrap();
        assert_eq!(structure.value_name(), "holder:custom:tower");

        let pool_context = context_with(
            "target",
            resource_key(
                TEMPLATE_POOL_REGISTRY,
                "minecraft:village/plains/town_centers",
            ),
        );
        let pool = get_structure_template_pool(&pool_context, "target").unwrap();
        assert_eq!(
            pool.key().identifier().to_string(),
            "minecraft:village/plains/town_centers"
        );
    }

    #[test]
    fn java_resolve_key_helpers_reuse_exception_for_wrong_registry_and_missing_element() {
        let wrong_registry = context_with(
            "target",
            resource_key(STRUCTURE_REGISTRY, "minecraft:village"),
        );
        assert_eq!(
            get_configured_feature(&wrong_registry, "target"),
            Err(ResourceKeyLookupError::Dynamic {
                kind: ResourceKeyErrorKind::Feature,
                value: CommandIdentifierModel::parse("minecraft:village").unwrap(),
            })
        );

        let missing = context_with(
            "target",
            resource_key(CONFIGURED_FEATURE_REGISTRY, "minecraft:missing"),
        );
        assert_eq!(
            get_configured_feature(&missing, "target"),
            Err(ResourceKeyLookupError::Dynamic {
                kind: ResourceKeyErrorKind::Feature,
                value: CommandIdentifierModel::parse("minecraft:missing").unwrap(),
            })
        );

        assert_eq!(
            ResourceKeyErrorKind::Feature.translation_key(),
            "commands.place.feature.invalid"
        );
        assert_eq!(
            ResourceKeyErrorKind::Structure.translation_key(),
            "commands.place.structure.invalid"
        );
        assert_eq!(
            ResourceKeyErrorKind::TemplatePool.translation_key(),
            "commands.place.jigsaw.invalid"
        );
    }

    #[test]
    fn java_recipe_lookup_uses_recipe_manager_after_registry_cast() {
        let context = context_with("recipe", resource_key(RECIPE_REGISTRY, "minecraft:bread"));
        let recipe = get_recipe(&context, "recipe").unwrap();
        assert_eq!(recipe.key().identifier().to_string(), "minecraft:bread");
        assert_eq!(recipe.value_name(), "recipe:minecraft:bread");

        let wrong_registry = context_with(
            "recipe",
            resource_key(CONFIGURED_FEATURE_REGISTRY, "minecraft:bread"),
        );
        assert_eq!(
            get_recipe(&wrong_registry, "recipe"),
            Err(ResourceKeyLookupError::Dynamic {
                kind: ResourceKeyErrorKind::Recipe,
                value: CommandIdentifierModel::parse("minecraft:bread").unwrap(),
            })
        );

        let missing = context_with("recipe", resource_key(RECIPE_REGISTRY, "minecraft:missing"));
        assert_eq!(
            get_recipe(&missing, "recipe"),
            Err(ResourceKeyLookupError::Dynamic {
                kind: ResourceKeyErrorKind::Recipe,
                value: CommandIdentifierModel::parse("minecraft:missing").unwrap(),
            })
        );
        assert_eq!(
            ResourceKeyErrorKind::Recipe.translation_key(),
            "recipe.notFound"
        );
    }

    #[test]
    fn java_advancement_lookup_uses_advancement_manager_identifier_map() {
        let context = context_with(
            "advancement",
            resource_key(ADVANCEMENT_REGISTRY, "minecraft:story/root"),
        );
        let advancement = get_advancement(&context, "advancement").unwrap();
        assert_eq!(advancement.id().to_string(), "minecraft:story/root");
        assert_eq!(advancement.value_name(), "advancement:minecraft:story/root");

        let missing = context_with(
            "advancement",
            resource_key(ADVANCEMENT_REGISTRY, "minecraft:story/missing"),
        );
        assert_eq!(
            get_advancement(&missing, "advancement"),
            Err(ResourceKeyLookupError::Dynamic {
                kind: ResourceKeyErrorKind::Advancement,
                value: CommandIdentifierModel::parse("minecraft:story/missing").unwrap(),
            })
        );
        assert_eq!(
            ResourceKeyErrorKind::Advancement.translation_key(),
            "advancement.advancementNotFound"
        );
    }

    #[test]
    fn java_helpers_require_command_source_stack_server() {
        let context = CommandContextModel::default().with_resource_key(
            "target",
            resource_key(CONFIGURED_FEATURE_REGISTRY, "minecraft:ore/diamond"),
        );

        assert_eq!(
            get_configured_feature(&context, "target"),
            Err(ResourceKeyLookupError::MissingServer)
        );
    }

    #[test]
    fn java_suggestions_delegate_to_shared_registry_element_suggestions() {
        let context = CommandContextModel::default().with_source(source());
        let argument = ResourceKeyArgumentModel::key(CONFIGURED_FEATURE_REGISTRY);

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
