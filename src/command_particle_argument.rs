use std::collections::HashMap;

use crate::command_identifier_argument::{CommandIdentifierModel, IdentifierArgumentParseError};

const PARTICLE_TYPE_REGISTRY: &str = "minecraft:particle_type";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleArgumentModel {
    registries: CommandBuildContextModel,
}

impl ParticleArgumentModel {
    pub fn particle(context: &CommandBuildContextModel) -> Self {
        Self {
            registries: context.clone(),
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<ParticleOptionsModel, ParticleParseError> {
        read_particle(reader, &self.registries)
    }

    pub fn list_suggestions(&self, remaining: &str) -> Vec<String> {
        let particles = self
            .registries
            .lookup_or_throw(&CommandIdentifierModel::parse(PARTICLE_TYPE_REGISTRY).unwrap());
        suggest_resource(particles.list_element_ids(), remaining)
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["foo", "foo:bar", "particle{foo:bar}"]
    }
}

pub fn get_particle(context: &CommandContextModel, name: &str) -> Option<ParticleOptionsModel> {
    context.arguments.get(name).cloned()
}

pub fn read_particle(
    reader: &mut StringReaderModel,
    registries: &CommandBuildContextModel,
) -> Result<ParticleOptionsModel, ParticleParseError> {
    let particle_type = read_particle_type(
        reader,
        &registries
            .lookup_or_throw(&CommandIdentifierModel::parse(PARTICLE_TYPE_REGISTRY).unwrap()),
    )?;
    read_particle_options(reader, &particle_type)
}

fn read_particle_type(
    reader: &mut StringReaderModel,
    particles: &HolderLookupModel,
) -> Result<ParticleTypeModel, ParticleParseError> {
    let id = read_identifier(reader).map_err(ParticleParseError::Identifier)?;
    let key = ResourceKeyModel::create(
        CommandIdentifierModel::parse(PARTICLE_TYPE_REGISTRY).unwrap(),
        id.clone(),
    );
    particles
        .get(&key)
        .map(|holder| holder.value().clone())
        .ok_or(ParticleParseError::UnknownParticle {
            id,
            cursor: reader.cursor(),
        })
}

fn read_particle_options(
    reader: &mut StringReaderModel,
    particle_type: &ParticleTypeModel,
) -> Result<ParticleOptionsModel, ParticleParseError> {
    let extra_data = if reader.can_read() && reader.peek() == '{' {
        parse_compound_argument(reader).map_err(ParticleParseError::TagParser)?
    } else {
        ParticlePayloadModel::EmptyMap
    };
    particle_type
        .decode(extra_data)
        .map_err(ParticleParseError::InvalidOptions)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, ParticleOptionsModel>,
}

impl CommandContextModel {
    pub fn with_particle(mut self, name: impl Into<String>, value: ParticleOptionsModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBuildContextModel {
    registries: HashMap<CommandIdentifierModel, HolderLookupModel>,
}

impl CommandBuildContextModel {
    pub fn with_particle_types(mut self, particles: &[ParticleTypeModel]) -> Self {
        let registry_key = CommandIdentifierModel::parse(PARTICLE_TYPE_REGISTRY).unwrap();
        self.registries.insert(
            registry_key.clone(),
            HolderLookupModel {
                entries: particles
                    .iter()
                    .map(|particle| HolderReferenceModel {
                        key: ResourceKeyModel::create(registry_key.clone(), particle.id.clone()),
                        value: particle.clone(),
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
pub struct HolderLookupModel {
    entries: Vec<HolderReferenceModel>,
}

impl HolderLookupModel {
    fn get(&self, key: &ResourceKeyModel) -> Option<HolderReferenceModel> {
        self.entries
            .iter()
            .find(|holder| holder.key() == key)
            .cloned()
    }

    fn list_element_ids(&self) -> impl Iterator<Item = &CommandIdentifierModel> {
        self.entries.iter().map(|holder| holder.key().identifier())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderReferenceModel {
    key: ResourceKeyModel,
    value: ParticleTypeModel,
}

impl HolderReferenceModel {
    fn key(&self) -> &ResourceKeyModel {
        &self.key
    }

    fn value(&self) -> &ParticleTypeModel {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleTypeModel {
    id: CommandIdentifierModel,
    option_rule: ParticleOptionRule,
}

impl ParticleTypeModel {
    pub fn simple(id: &str) -> Self {
        Self {
            id: CommandIdentifierModel::parse(id).expect("valid particle id"),
            option_rule: ParticleOptionRule::NoFields,
        }
    }

    pub fn block_state(id: &str, required_key: &str) -> Self {
        Self {
            id: CommandIdentifierModel::parse(id).expect("valid particle id"),
            option_rule: ParticleOptionRule::RequiresField(required_key.to_string()),
        }
    }

    fn decode(
        &self,
        payload: ParticlePayloadModel,
    ) -> Result<ParticleOptionsModel, ParticleOptionDecodeError> {
        match (&self.option_rule, &payload) {
            (ParticleOptionRule::NoFields, ParticlePayloadModel::EmptyMap) => {
                Ok(ParticleOptionsModel {
                    particle_id: self.id.clone(),
                    payload,
                })
            }
            (ParticleOptionRule::NoFields, ParticlePayloadModel::Compound { fields })
                if fields.is_empty() =>
            {
                Ok(ParticleOptionsModel {
                    particle_id: self.id.clone(),
                    payload,
                })
            }
            (ParticleOptionRule::NoFields, ParticlePayloadModel::Compound { .. }) => {
                Err(ParticleOptionDecodeError::UnexpectedFields)
            }
            (
                ParticleOptionRule::RequiresField(required),
                ParticlePayloadModel::Compound { fields },
            ) if fields.iter().any(|field| field == required) => Ok(ParticleOptionsModel {
                particle_id: self.id.clone(),
                payload,
            }),
            (ParticleOptionRule::RequiresField(required), _payload) => {
                Err(ParticleOptionDecodeError::MissingField(required.clone()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParticleOptionRule {
    NoFields,
    RequiresField(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleOptionsModel {
    particle_id: CommandIdentifierModel,
    payload: ParticlePayloadModel,
}

impl ParticleOptionsModel {
    pub fn particle_id(&self) -> &CommandIdentifierModel {
        &self.particle_id
    }

    pub fn payload(&self) -> &ParticlePayloadModel {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticlePayloadModel {
    EmptyMap,
    Compound { fields: Vec<String> },
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

    fn identifier(&self) -> &CommandIdentifierModel {
        &self.identifier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticleParseError {
    Identifier(IdentifierArgumentParseError),
    UnknownParticle {
        id: CommandIdentifierModel,
        cursor: usize,
    },
    TagParser(TagParseError),
    InvalidOptions(ParticleOptionDecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagParseError {
    ExpectedOpeningBrace,
    UnclosedCompound,
    InvalidFieldName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticleOptionDecodeError {
    UnexpectedFields,
    MissingField(String),
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

fn parse_compound_argument(
    reader: &mut StringReaderModel,
) -> Result<ParticlePayloadModel, TagParseError> {
    if !reader.can_read() || reader.peek() != '{' {
        return Err(TagParseError::ExpectedOpeningBrace);
    }

    reader.skip();
    let mut fields = Vec::new();
    loop {
        if !reader.can_read() {
            return Err(TagParseError::UnclosedCompound);
        }
        if reader.peek() == '}' {
            reader.skip();
            return Ok(ParticlePayloadModel::Compound { fields });
        }

        let field_start = reader.cursor();
        while reader.can_read() && is_allowed_field_name(reader.peek()) {
            reader.skip();
        }
        if reader.cursor() == field_start {
            return Err(TagParseError::InvalidFieldName);
        }
        let field = reader.slice(field_start, reader.cursor()).to_string();

        while reader.can_read() && reader.peek().is_ascii_whitespace() {
            reader.skip();
        }
        if !reader.can_read() || reader.peek() != ':' {
            return Err(TagParseError::InvalidFieldName);
        }
        reader.skip();
        skip_value(reader)?;
        fields.push(field);

        while reader.can_read() && reader.peek().is_ascii_whitespace() {
            reader.skip();
        }
        if reader.can_read() && reader.peek() == ',' {
            reader.skip();
        }
    }
}

fn is_allowed_field_name(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.' | '/')
}

fn skip_value(reader: &mut StringReaderModel) -> Result<(), TagParseError> {
    while reader.can_read() && reader.peek().is_ascii_whitespace() {
        reader.skip();
    }
    if !reader.can_read() {
        return Err(TagParseError::UnclosedCompound);
    }

    if reader.peek() == '{' {
        let mut depth = 0;
        while reader.can_read() {
            match reader.peek() {
                '{' => {
                    depth += 1;
                    reader.skip();
                }
                '}' => {
                    depth -= 1;
                    reader.skip();
                    if depth == 0 {
                        break;
                    }
                }
                _ => reader.skip(),
            }
        }
        if depth != 0 {
            return Err(TagParseError::UnclosedCompound);
        }
        return Ok(());
    }

    while reader.can_read() && !matches!(reader.peek(), ',' | '}') {
        reader.skip();
    }
    Ok(())
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
    CommandBuildContextModel::default().with_particle_types(&[
        ParticleTypeModel::simple("minecraft:smoke"),
        ParticleTypeModel::block_state("minecraft:block", "Name"),
        ParticleTypeModel::simple("custom:spark"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argument() -> ParticleArgumentModel {
        ParticleArgumentModel::particle(&build_context())
    }

    fn parse(input: &str) -> Result<(ParticleOptionsModel, usize), ParticleParseError> {
        let mut reader = StringReaderModel::new(input);
        let options = argument().parse(&mut reader)?;
        Ok((options, reader.cursor()))
    }

    #[test]
    fn java_factory_examples_and_context_getter_match_source() {
        let argument = argument();
        assert_eq!(argument.examples(), ["foo", "foo:bar", "particle{foo:bar}"]);

        let parsed = parse("smoke").unwrap().0;
        let context = CommandContextModel::default().with_particle("particle", parsed.clone());
        assert_eq!(get_particle(&context, "particle"), Some(parsed));
        assert_eq!(get_particle(&context, "missing"), None);
    }

    #[test]
    fn java_parse_reads_particle_type_and_uses_empty_map_without_options() {
        let (options, cursor) = parse("smoke trailing").unwrap();

        assert_eq!(options.particle_id().to_string(), "minecraft:smoke");
        assert_eq!(options.payload(), &ParticlePayloadModel::EmptyMap);
        assert_eq!(cursor, "smoke".len());
    }

    #[test]
    fn java_parse_accepts_explicit_namespace_and_compound_options() {
        let input = "minecraft:block{Name:\"minecraft:stone\"} rest";
        let (options, cursor) = parse(input).unwrap();

        assert_eq!(options.particle_id().to_string(), "minecraft:block");
        assert_eq!(
            options.payload(),
            &ParticlePayloadModel::Compound {
                fields: vec!["Name".to_string()]
            }
        );
        assert_eq!(cursor, "minecraft:block{Name:\"minecraft:stone\"}".len());
    }

    #[test]
    fn java_unknown_particle_reports_identifier_and_post_identifier_cursor() {
        let mut reader = StringReaderModel::new("missing@tail");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(ParticleParseError::UnknownParticle {
                id: CommandIdentifierModel::parse("minecraft:missing").unwrap(),
                cursor: "missing".len(),
            })
        );
        assert_eq!(reader.cursor(), "missing".len());
    }

    #[test]
    fn java_identifier_errors_reset_cursor_before_particle_lookup() {
        let mut reader = StringReaderModel::new("..:bad trailing");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(ParticleParseError::Identifier(
                IdentifierArgumentParseError::InvalidIdentifier
            ))
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_tag_parser_errors_are_reported_before_codec_decode() {
        let mut reader = StringReaderModel::new("block{Name:\"minecraft:stone\"");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(ParticleParseError::TagParser(
                TagParseError::UnclosedCompound
            ))
        );
    }

    #[test]
    fn java_particle_codec_rejects_invalid_or_missing_options() {
        assert_eq!(
            parse("smoke{foo:bar}").map(|value| value.0),
            Err(ParticleParseError::InvalidOptions(
                ParticleOptionDecodeError::UnexpectedFields
            ))
        );
        assert_eq!(
            parse("block").map(|value| value.0),
            Err(ParticleParseError::InvalidOptions(
                ParticleOptionDecodeError::MissingField("Name".to_string())
            ))
        );
    }

    #[test]
    fn java_static_read_particle_uses_given_registry_provider() {
        let mut reader = StringReaderModel::new("custom:spark");
        let options = read_particle(&mut reader, &build_context()).unwrap();

        assert_eq!(options.particle_id().to_string(), "custom:spark");
        assert_eq!(reader.cursor(), "custom:spark".len());
    }

    #[test]
    fn java_suggestions_list_particle_registry_element_ids() {
        let argument = argument();

        assert_eq!(
            argument.list_suggestions(""),
            ["minecraft:smoke", "minecraft:block", "custom:spark"]
        );
        assert_eq!(argument.list_suggestions("blo"), ["minecraft:block"]);
        assert_eq!(argument.list_suggestions("custom:s"), ["custom:spark"]);
    }
}
