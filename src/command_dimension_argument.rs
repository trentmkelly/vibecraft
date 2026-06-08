use std::collections::HashMap;

use crate::command_identifier_argument::{
    CommandIdentifierModel, IdentifierArgumentParseError, StringReaderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionArgumentModel;

impl DimensionArgumentModel {
    pub fn dimension() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<CommandIdentifierModel, IdentifierArgumentParseError> {
        CommandIdentifierModel::read(reader)
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["minecraft:overworld", "minecraft:the_nether"]
    }

    pub fn list_suggestions(&self, context: &CommandContextModel, remaining: &str) -> Vec<String> {
        match &context.source {
            CommandSourceModel::Shared(provider) => suggest_resource(
                provider.levels.iter().map(ResourceKeyModel::identifier),
                remaining,
            ),
            CommandSourceModel::Other => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, CommandIdentifierModel>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn with_dimension_argument(
        mut self,
        name: impl Into<String>,
        value: CommandIdentifierModel,
    ) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }

    pub fn with_source(mut self, source: CommandSourceModel) -> Self {
        self.source = source;
        self
    }
}

pub fn get_dimension(
    context: &CommandContextModel,
    name: &str,
) -> Result<ServerLevelModel, DimensionLookupError> {
    let location = context
        .arguments
        .get(name)
        .ok_or(DimensionLookupError::MissingArgument)?
        .clone();
    let key = ResourceKeyModel::create(
        CommandIdentifierModel::parse("minecraft:dimension").expect("valid registry key"),
        location.clone(),
    );
    match &context.source {
        CommandSourceModel::Shared(provider) => provider
            .server
            .get_level(&key)
            .ok_or(DimensionLookupError::InvalidDimension(location)),
        CommandSourceModel::Other => Err(DimensionLookupError::MissingServer),
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
    levels: Vec<ResourceKeyModel>,
    server: ServerModel,
}

impl SharedSuggestionProviderModel {
    pub fn with_level(mut self, name: &str) -> Self {
        let identifier = CommandIdentifierModel::parse(name).expect("valid level identifier");
        let key = ResourceKeyModel::create(
            CommandIdentifierModel::parse("minecraft:dimension").expect("valid registry key"),
            identifier,
        );
        self.server.levels.insert(
            key.clone(),
            ServerLevelModel {
                key: key.clone(),
                display_name: format!("level:{name}"),
            },
        );
        self.levels.push(key);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerModel {
    levels: HashMap<ResourceKeyModel, ServerLevelModel>,
}

impl ServerModel {
    fn get_level(&self, key: &ResourceKeyModel) -> Option<ServerLevelModel> {
        self.levels.get(key).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLevelModel {
    key: ResourceKeyModel,
    display_name: String,
}

impl ServerLevelModel {
    pub fn key(&self) -> &ResourceKeyModel {
        &self.key
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
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

    pub fn identifier(&self) -> &CommandIdentifierModel {
        &self.identifier
    }

    pub fn registry(&self) -> &CommandIdentifierModel {
        &self.registry
    }

    pub fn registry_key(&self) -> Self {
        Self::create(
            CommandIdentifierModel::parse("minecraft:root").expect("valid root registry"),
            self.registry.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DimensionLookupError {
    MissingArgument,
    MissingServer,
    InvalidDimension(CommandIdentifierModel),
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

fn suggestion_provider(levels: &[&str]) -> SharedSuggestionProviderModel {
    levels.iter().fold(
        SharedSuggestionProviderModel::default(),
        |provider, level| provider.with_level(level),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(CommandIdentifierModel, usize), IdentifierArgumentParseError> {
        let mut reader = StringReaderModel::new(input);
        let identifier = DimensionArgumentModel::dimension().parse(&mut reader)?;
        Ok((identifier, reader.cursor()))
    }

    #[test]
    fn java_dimension_factory_examples_and_resource_key_shape_match_source() {
        let argument = DimensionArgumentModel::dimension();
        assert_eq!(
            argument.examples(),
            ["minecraft:overworld", "minecraft:the_nether"]
        );

        let key = ResourceKeyModel::create(
            CommandIdentifierModel::parse("minecraft:dimension").unwrap(),
            CommandIdentifierModel::parse("minecraft:overworld").unwrap(),
        );
        assert_eq!(key.registry().to_string(), "minecraft:dimension");
        assert_eq!(key.identifier().to_string(), "minecraft:overworld");
        assert_eq!(key.registry_key().registry().to_string(), "minecraft:root");
        assert_eq!(
            key.registry_key().identifier().to_string(),
            "minecraft:dimension"
        );
    }

    #[test]
    fn java_parse_delegates_to_identifier_read() {
        let (defaulted, defaulted_cursor) = parse("overworld trailing").unwrap();
        assert_eq!(defaulted.to_string(), "minecraft:overworld");
        assert_eq!(defaulted_cursor, "overworld".len());

        let (explicit, explicit_cursor) = parse("custom:moon/rest").unwrap();
        assert_eq!(explicit.namespace(), "custom");
        assert_eq!(explicit.path(), "moon/rest");
        assert_eq!(explicit_cursor, "custom:moon/rest".len());

        let (empty, empty_cursor) = parse("NotAllowed").unwrap();
        assert_eq!(empty.to_string(), "minecraft:");
        assert_eq!(empty_cursor, 0);
    }

    #[test]
    fn java_invalid_identifier_parse_resets_cursor() {
        let mut reader = StringReaderModel::new("..:bad trailing");
        let error = DimensionArgumentModel::dimension().parse(&mut reader);

        assert_eq!(error, Err(IdentifierArgumentParseError::InvalidIdentifier));
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_get_dimension_wraps_identifier_in_dimension_resource_key() {
        let source = CommandSourceModel::Shared(suggestion_provider(&[
            "minecraft:overworld",
            "minecraft:the_nether",
        ]));
        let context = CommandContextModel::default()
            .with_source(source)
            .with_dimension_argument(
                "dimension",
                CommandIdentifierModel::parse("minecraft:the_nether").unwrap(),
            );

        let level = get_dimension(&context, "dimension").unwrap();
        assert_eq!(level.key().registry().to_string(), "minecraft:dimension");
        assert_eq!(level.key().identifier().to_string(), "minecraft:the_nether");
        assert_eq!(level.display_name(), "level:minecraft:the_nether");
    }

    #[test]
    fn java_get_dimension_rejects_missing_or_unknown_dimensions() {
        let source = CommandSourceModel::Shared(suggestion_provider(&["minecraft:overworld"]));
        let missing_argument = CommandContextModel::default().with_source(source.clone());
        assert_eq!(
            get_dimension(&missing_argument, "dimension"),
            Err(DimensionLookupError::MissingArgument)
        );

        let unknown_level = CommandContextModel::default()
            .with_source(source)
            .with_dimension_argument(
                "dimension",
                CommandIdentifierModel::parse("minecraft:missing").unwrap(),
            );
        assert_eq!(
            get_dimension(&unknown_level, "dimension"),
            Err(DimensionLookupError::InvalidDimension(
                CommandIdentifierModel::parse("minecraft:missing").unwrap()
            ))
        );

        let without_server = CommandContextModel::default().with_dimension_argument(
            "dimension",
            CommandIdentifierModel::parse("minecraft:overworld").unwrap(),
        );
        assert_eq!(
            get_dimension(&without_server, "dimension"),
            Err(DimensionLookupError::MissingServer)
        );
    }

    #[test]
    fn java_level_suggestions_require_shared_suggestion_provider_source() {
        let context = CommandContextModel::default().with_source(CommandSourceModel::Shared(
            suggestion_provider(&["minecraft:overworld", "minecraft:the_nether"]),
        ));
        let argument = DimensionArgumentModel::dimension();

        assert_eq!(
            argument.list_suggestions(&context, ""),
            ["minecraft:overworld", "minecraft:the_nether"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "over"),
            ["minecraft:overworld"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "the_"),
            ["minecraft:the_nether"]
        );
        assert_eq!(
            argument.list_suggestions(&context, "minecraft:t"),
            ["minecraft:the_nether"]
        );

        let non_shared = CommandContextModel::default().with_source(CommandSourceModel::Other);
        assert_eq!(
            argument.list_suggestions(&non_shared, ""),
            Vec::<String>::new()
        );
    }
}
