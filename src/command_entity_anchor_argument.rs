use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityAnchorArgumentModel;

impl EntityAnchorArgumentModel {
    pub fn anchor() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<EntityAnchorModel, EntityAnchorParseError> {
        let start = reader.cursor();
        let name = reader.read_unquoted_string();
        EntityAnchorModel::get_by_name(&name).ok_or_else(|| {
            reader.set_cursor(start);
            EntityAnchorParseError::Invalid { name }
        })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(
            EntityAnchorModel::names().into_iter().map(str::to_string),
            builder,
        );
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["eyes", "feet"]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityAnchorModel {
    Feet,
    Eyes,
}

impl EntityAnchorModel {
    pub fn get_by_name(name: &str) -> Option<Self> {
        match name {
            "feet" => Some(Self::Feet),
            "eyes" => Some(Self::Eyes),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Feet => "feet",
            Self::Eyes => "eyes",
        }
    }

    pub fn names() -> [&'static str; 2] {
        [Self::Feet.name(), Self::Eyes.name()]
    }

    pub fn apply_entity(self, entity: &EntityModel) -> Vec3Model {
        self.transform(entity.position, entity.eye_height)
    }

    pub fn apply_source(self, source: &CommandSourceStackModel) -> Vec3Model {
        match &source.entity {
            Some(entity) => self.transform(source.position, entity.eye_height),
            None => source.position,
        }
    }

    fn transform(self, position: Vec3Model, eye_height: f64) -> Vec3Model {
        match self {
            Self::Feet => position,
            Self::Eyes => Vec3Model {
                y: position.y + eye_height,
                ..position
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, EntityAnchorModel>,
}

impl CommandContextModel {
    pub fn with_anchor(mut self, name: impl Into<String>, value: EntityAnchorModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_anchor(context: &CommandContextModel, name: &str) -> Option<EntityAnchorModel> {
    context.arguments.get(name).copied()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3Model {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3Model {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityModel {
    position: Vec3Model,
    eye_height: f64,
}

impl EntityModel {
    pub const fn new(position: Vec3Model, eye_height: f64) -> Self {
        Self {
            position,
            eye_height,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSourceStackModel {
    position: Vec3Model,
    entity: Option<EntityModel>,
}

impl CommandSourceStackModel {
    pub const fn new(position: Vec3Model, entity: Option<EntityModel>) -> Self {
        Self { position, entity }
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

    fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && !self.peek().is_ascii_whitespace() {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityAnchorParseError {
    Invalid { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(EntityAnchorModel, usize), EntityAnchorParseError> {
        let mut reader = StringReaderModel::new(input);
        let anchor = EntityAnchorArgumentModel::anchor().parse(&mut reader)?;
        Ok((anchor, reader.cursor()))
    }

    #[test]
    fn examples_match_java_examples() {
        assert_eq!(
            EntityAnchorArgumentModel::anchor().examples(),
            ["eyes", "feet"]
        );
    }

    #[test]
    fn get_anchor_returns_typed_context_argument() {
        let context = CommandContextModel::default().with_anchor("anchor", EntityAnchorModel::Eyes);

        assert_eq!(
            get_anchor(&context, "anchor"),
            Some(EntityAnchorModel::Eyes)
        );
        assert_eq!(get_anchor(&context, "missing"), None);
    }

    #[test]
    fn parse_accepts_exact_anchor_names_and_advances_past_unquoted_token() {
        let (anchor, cursor) = parse("eyes rest").unwrap();

        assert_eq!(anchor, EntityAnchorModel::Eyes);
        assert_eq!(cursor, 4);
        assert_eq!(parse("feet"), Ok((EntityAnchorModel::Feet, 4)));
    }

    #[test]
    fn parse_is_case_sensitive_and_resets_cursor_on_invalid_anchor() {
        let mut reader = StringReaderModel::new("Eyes rest");

        assert_eq!(
            EntityAnchorArgumentModel::anchor().parse(&mut reader),
            Err(EntityAnchorParseError::Invalid {
                name: "Eyes".to_string(),
            })
        );
        assert_eq!(reader.cursor(), 0);
        assert_eq!(
            parse(""),
            Err(EntityAnchorParseError::Invalid {
                name: String::new(),
            })
        );
    }

    #[test]
    fn anchor_lookup_matches_java_by_name_map() {
        assert_eq!(
            EntityAnchorModel::get_by_name("feet"),
            Some(EntityAnchorModel::Feet)
        );
        assert_eq!(
            EntityAnchorModel::get_by_name("eyes"),
            Some(EntityAnchorModel::Eyes)
        );
        assert_eq!(EntityAnchorModel::get_by_name("eye"), None);
    }

    #[test]
    fn apply_entity_uses_position_for_feet_and_position_plus_eye_height_for_eyes() {
        let entity = EntityModel::new(Vec3Model::new(1.0, 64.0, -2.0), 1.62);

        assert_eq!(
            EntityAnchorModel::Feet.apply_entity(&entity),
            Vec3Model::new(1.0, 64.0, -2.0)
        );
        assert_eq!(
            EntityAnchorModel::Eyes.apply_entity(&entity),
            Vec3Model::new(1.0, 65.62, -2.0)
        );
    }

    #[test]
    fn apply_source_uses_source_position_and_optional_entity_eye_height() {
        let entity = EntityModel::new(Vec3Model::new(99.0, 0.0, 99.0), 0.9);
        let source_with_entity =
            CommandSourceStackModel::new(Vec3Model::new(5.0, 10.0, 15.0), Some(entity));
        let source_without_entity =
            CommandSourceStackModel::new(Vec3Model::new(5.0, 10.0, 15.0), None);

        assert_eq!(
            EntityAnchorModel::Eyes.apply_source(&source_with_entity),
            Vec3Model::new(5.0, 10.9, 15.0)
        );
        assert_eq!(
            EntityAnchorModel::Eyes.apply_source(&source_without_entity),
            Vec3Model::new(5.0, 10.0, 15.0)
        );
        assert_eq!(
            EntityAnchorModel::Feet.apply_source(&source_with_entity),
            Vec3Model::new(5.0, 10.0, 15.0)
        );
    }

    #[test]
    fn list_suggestions_exposes_anchor_names_and_shared_filtering() {
        let mut all = SuggestionsBuilderModel::new("");
        let mut values = EntityAnchorArgumentModel::anchor()
            .list_suggestions(&mut all)
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect::<Vec<_>>();
        values.sort();
        assert_eq!(values, vec!["eyes", "feet"]);

        let mut eyes = SuggestionsBuilderModel::new("ey");
        assert_eq!(
            EntityAnchorArgumentModel::anchor().list_suggestions(&mut eyes),
            vec![SuggestionModel {
                value: "eyes".to_string(),
                tooltip: None,
            }]
        );
    }
}
