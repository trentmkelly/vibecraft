use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest_2d_coordinates, suggest_coordinates, SuggestionModel, SuggestionProviderModel,
    SuggestionsBuilderModel, TextCoordinates,
};

pub const COORDINATES_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2Model {
    pub x: f64,
    pub y: f64,
}

impl Vec2Model {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPosModel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPosModel {
    fn containing(position: Vec3Model) -> Self {
        Self {
            x: floor_to_i32(position.x),
            y: floor_to_i32(position.y),
            z: floor_to_i32(position.z),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandSourceStackModel {
    position: Vec3Model,
    rotation: Vec2Model,
    anchor_position: Vec3Model,
}

impl CommandSourceStackModel {
    pub const fn new(position: Vec3Model, rotation: Vec2Model) -> Self {
        Self {
            position,
            rotation,
            anchor_position: position,
        }
    }

    pub const fn with_anchor_position(mut self, anchor_position: Vec3Model) -> Self {
        self.anchor_position = anchor_position;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinatesModel {
    World(WorldCoordinatesModel),
    Local(LocalCoordinatesModel),
}

impl CoordinatesModel {
    pub fn get_position(self, source: &CommandSourceStackModel) -> Vec3Model {
        match self {
            Self::World(coordinates) => coordinates.get_position(source),
            Self::Local(coordinates) => coordinates.get_position(source),
        }
    }

    pub fn get_rotation(self, source: &CommandSourceStackModel) -> Vec2Model {
        match self {
            Self::World(coordinates) => coordinates.get_rotation(source),
            Self::Local(coordinates) => coordinates.get_rotation(source),
        }
    }

    pub fn get_block_pos(self, source: &CommandSourceStackModel) -> BlockPosModel {
        BlockPosModel::containing(self.get_position(source))
    }

    pub fn is_x_relative(self) -> bool {
        match self {
            Self::World(coordinates) => coordinates.is_x_relative(),
            Self::Local(coordinates) => coordinates.is_x_relative(),
        }
    }

    pub fn is_y_relative(self) -> bool {
        match self {
            Self::World(coordinates) => coordinates.is_y_relative(),
            Self::Local(coordinates) => coordinates.is_y_relative(),
        }
    }

    pub fn is_z_relative(self) -> bool {
        match self {
            Self::World(coordinates) => coordinates.is_z_relative(),
            Self::Local(coordinates) => coordinates.is_z_relative(),
        }
    }
}

impl From<WorldCoordinatesModel> for CoordinatesModel {
    fn from(value: WorldCoordinatesModel) -> Self {
        Self::World(value)
    }
}

impl From<LocalCoordinatesModel> for CoordinatesModel {
    fn from(value: LocalCoordinatesModel) -> Self {
        Self::Local(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCoordinateModel {
    relative: bool,
    value: f64,
}

impl WorldCoordinateModel {
    pub const fn new(relative: bool, value: f64) -> Self {
        Self { relative, value }
    }

    pub fn get(self, original: f64) -> f64 {
        if self.relative {
            self.value + original
        } else {
            self.value
        }
    }

    pub fn is_relative(self) -> bool {
        self.relative
    }

    pub fn parse_double(
        reader: &mut StringReaderModel,
        center: bool,
    ) -> Result<Self, CoordinateParseError> {
        if reader.can_read() && reader.peek() == '^' {
            return Err(CoordinateParseError::MixedType {
                cursor: reader.cursor(),
            });
        }
        if !reader.can_read() {
            return Err(CoordinateParseError::ExpectedDouble {
                cursor: reader.cursor(),
            });
        }

        let relative = Self::is_relative_prefix(reader);
        let start = reader.cursor();
        let (value, number) = if reader.can_read() && reader.peek() != ' ' {
            (reader.read_double()?, reader.slice_from(start))
        } else {
            (0.0, String::new())
        };
        if relative && number.is_empty() {
            return Ok(Self::new(true, 0.0));
        }
        let centered = if !number.contains('.') && !relative && center {
            value + 0.5
        } else {
            value
        };
        Ok(Self::new(relative, centered))
    }

    pub fn parse_int(reader: &mut StringReaderModel) -> Result<Self, CoordinateParseError> {
        if reader.can_read() && reader.peek() == '^' {
            return Err(CoordinateParseError::MixedType {
                cursor: reader.cursor(),
            });
        }
        if !reader.can_read() {
            return Err(CoordinateParseError::ExpectedInt {
                cursor: reader.cursor(),
            });
        }

        let relative = Self::is_relative_prefix(reader);
        let value = if reader.can_read() && reader.peek() != ' ' {
            if relative {
                reader.read_double()?
            } else {
                f64::from(reader.read_int()?)
            }
        } else {
            0.0
        };
        Ok(Self::new(relative, value))
    }

    fn is_relative_prefix(reader: &mut StringReaderModel) -> bool {
        if reader.peek() == '~' {
            reader.skip();
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCoordinatesModel {
    x: WorldCoordinateModel,
    y: WorldCoordinateModel,
    z: WorldCoordinateModel,
}

impl WorldCoordinatesModel {
    pub const ZERO_ROTATION: Self = Self::absolute_rotation(Vec2Model::new(0.0, 0.0));

    pub const fn new(
        x: WorldCoordinateModel,
        y: WorldCoordinateModel,
        z: WorldCoordinateModel,
    ) -> Self {
        Self { x, y, z }
    }

    pub fn get_position(self, source: &CommandSourceStackModel) -> Vec3Model {
        Vec3Model::new(
            self.x.get(source.position.x),
            self.y.get(source.position.y),
            self.z.get(source.position.z),
        )
    }

    pub fn get_rotation(self, source: &CommandSourceStackModel) -> Vec2Model {
        Vec2Model::new(
            self.x.get(source.rotation.x) as f32 as f64,
            self.y.get(source.rotation.y) as f32 as f64,
        )
    }

    pub fn get_block_pos(self, source: &CommandSourceStackModel) -> BlockPosModel {
        BlockPosModel::containing(self.get_position(source))
    }

    pub fn is_x_relative(self) -> bool {
        self.x.is_relative()
    }

    pub fn is_y_relative(self) -> bool {
        self.y.is_relative()
    }

    pub fn is_z_relative(self) -> bool {
        self.z.is_relative()
    }

    pub fn parse_int(reader: &mut StringReaderModel) -> Result<Self, CoordinateParseError> {
        let start = reader.cursor();
        let x = WorldCoordinateModel::parse_int(reader)?;
        if reader.can_read() && reader.peek() == ' ' {
            reader.skip();
            let y = WorldCoordinateModel::parse_int(reader)?;
            if reader.can_read() && reader.peek() == ' ' {
                reader.skip();
                let z = WorldCoordinateModel::parse_int(reader)?;
                Ok(Self::new(x, y, z))
            } else {
                reader.set_cursor(start);
                Err(CoordinateParseError::Vec3Incomplete {
                    cursor: reader.cursor(),
                })
            }
        } else {
            reader.set_cursor(start);
            Err(CoordinateParseError::Vec3Incomplete {
                cursor: reader.cursor(),
            })
        }
    }

    pub fn parse_double(
        reader: &mut StringReaderModel,
        center_correct: bool,
    ) -> Result<Self, CoordinateParseError> {
        let start = reader.cursor();
        let x = WorldCoordinateModel::parse_double(reader, center_correct)?;
        if reader.can_read() && reader.peek() == ' ' {
            reader.skip();
            let y = WorldCoordinateModel::parse_double(reader, false)?;
            if reader.can_read() && reader.peek() == ' ' {
                reader.skip();
                let z = WorldCoordinateModel::parse_double(reader, center_correct)?;
                Ok(Self::new(x, y, z))
            } else {
                reader.set_cursor(start);
                Err(CoordinateParseError::Vec3Incomplete {
                    cursor: reader.cursor(),
                })
            }
        } else {
            reader.set_cursor(start);
            Err(CoordinateParseError::Vec3Incomplete {
                cursor: reader.cursor(),
            })
        }
    }

    pub const fn absolute(x: f64, y: f64, z: f64) -> Self {
        Self::new(
            WorldCoordinateModel::new(false, x),
            WorldCoordinateModel::new(false, y),
            WorldCoordinateModel::new(false, z),
        )
    }

    pub const fn absolute_rotation(rotation: Vec2Model) -> Self {
        Self::new(
            WorldCoordinateModel::new(false, rotation.x),
            WorldCoordinateModel::new(false, rotation.y),
            WorldCoordinateModel::new(true, 0.0),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalCoordinatesModel {
    left: f64,
    up: f64,
    forwards: f64,
}

impl LocalCoordinatesModel {
    pub const PREFIX_LOCAL_COORDINATE: char = '^';

    pub const fn new(left: f64, up: f64, forwards: f64) -> Self {
        Self { left, up, forwards }
    }

    pub fn get_position(self, source: &CommandSourceStackModel) -> Vec3Model {
        add_vec3(
            apply_local_coordinates_to_rotation(
                source.rotation,
                Vec3Model::new(self.left, self.up, self.forwards),
            ),
            source.anchor_position,
        )
    }

    pub fn get_rotation(self, _source: &CommandSourceStackModel) -> Vec2Model {
        Vec2Model::new(0.0, 0.0)
    }

    pub fn is_x_relative(self) -> bool {
        true
    }

    pub fn is_y_relative(self) -> bool {
        true
    }

    pub fn is_z_relative(self) -> bool {
        true
    }

    pub fn parse(reader: &mut StringReaderModel) -> Result<Self, CoordinateParseError> {
        let start = reader.cursor();
        let left = Self::read_double(reader, start)?;
        if reader.can_read() && reader.peek() == ' ' {
            reader.skip();
            let up = Self::read_double(reader, start)?;
            if reader.can_read() && reader.peek() == ' ' {
                reader.skip();
                let forwards = Self::read_double(reader, start)?;
                Ok(Self::new(left, up, forwards))
            } else {
                reader.set_cursor(start);
                Err(CoordinateParseError::Vec3Incomplete {
                    cursor: reader.cursor(),
                })
            }
        } else {
            reader.set_cursor(start);
            Err(CoordinateParseError::Vec3Incomplete {
                cursor: reader.cursor(),
            })
        }
    }

    fn read_double(
        reader: &mut StringReaderModel,
        start: usize,
    ) -> Result<f64, CoordinateParseError> {
        if !reader.can_read() {
            Err(CoordinateParseError::ExpectedDouble {
                cursor: reader.cursor(),
            })
        } else if reader.peek() != Self::PREFIX_LOCAL_COORDINATE {
            reader.set_cursor(start);
            Err(CoordinateParseError::MixedType {
                cursor: reader.cursor(),
            })
        } else {
            reader.skip();
            if reader.can_read() && reader.peek() != ' ' {
                reader.read_double()
            } else {
                Ok(0.0)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vec3ArgumentModel {
    center_correct: bool,
}

impl Vec3ArgumentModel {
    pub fn vec3() -> Self {
        Self {
            center_correct: true,
        }
    }

    pub fn vec3_with_center(center_correct: bool) -> Self {
        Self { center_correct }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<CoordinatesModel, CoordinateParseError> {
        if reader.can_read() && reader.peek() == LocalCoordinatesModel::PREFIX_LOCAL_COORDINATE {
            LocalCoordinatesModel::parse(reader).map(CoordinatesModel::from)
        } else {
            WorldCoordinatesModel::parse_double(reader, self.center_correct)
                .map(CoordinatesModel::from)
        }
    }

    pub fn list_suggestions(
        &self,
        source: Option<&SuggestionProviderModel>,
        builder: &mut SuggestionsBuilderModel,
    ) -> Vec<SuggestionModel> {
        let Some(source) = source else {
            return Vec::new();
        };
        let remainder = builder.remaining().to_string();
        let suggested_coordinates = if remainder.starts_with('^') {
            vec![TextCoordinates::default_local()]
        } else {
            source.get_absolute_coordinates()
        };
        suggest_coordinates(&remainder, &suggested_coordinates, builder, |candidate| {
            let mut reader = StringReaderModel::new(candidate);
            self.parse(&mut reader).is_ok()
        });
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 6] {
        [
            "0 0 0",
            "~ ~ ~",
            "^ ^ ^",
            "^1 ^ ^-5",
            "0.1 -0.5 .9",
            "~0.5 ~1 ~-5",
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vec2ArgumentModel {
    center_correct: bool,
}

impl Vec2ArgumentModel {
    pub fn vec2() -> Self {
        Self {
            center_correct: true,
        }
    }

    pub fn vec2_with_center(center_correct: bool) -> Self {
        Self { center_correct }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<CoordinatesModel, CoordinateParseError> {
        let start = reader.cursor();
        if !reader.can_read() {
            return Err(CoordinateParseError::Vec2Incomplete {
                cursor: reader.cursor(),
            });
        }

        let x = WorldCoordinateModel::parse_double(reader, self.center_correct)?;
        if reader.can_read() && reader.peek() == ' ' {
            reader.skip();
            let z = WorldCoordinateModel::parse_double(reader, self.center_correct)?;
            Ok(WorldCoordinatesModel::new(x, WorldCoordinateModel::new(true, 0.0), z).into())
        } else {
            reader.set_cursor(start);
            Err(CoordinateParseError::Vec2Incomplete {
                cursor: reader.cursor(),
            })
        }
    }

    pub fn list_suggestions(
        &self,
        source: Option<&SuggestionProviderModel>,
        builder: &mut SuggestionsBuilderModel,
    ) -> Vec<SuggestionModel> {
        let Some(source) = source else {
            return Vec::new();
        };
        let remainder = builder.remaining().to_string();
        let suggested_coordinates = if remainder.starts_with('^') {
            vec![TextCoordinates::default_local()]
        } else {
            source.get_absolute_coordinates()
        };
        suggest_2d_coordinates(&remainder, &suggested_coordinates, builder, |candidate| {
            let mut reader = StringReaderModel::new(candidate);
            self.parse(&mut reader).is_ok()
        });
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 4] {
        ["0 0", "~ ~", "0.1 -0.5", "~1 ~-2"]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotationArgumentModel;

impl RotationArgumentModel {
    pub fn rotation() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<WorldCoordinatesModel, CoordinateParseError> {
        let start = reader.cursor();
        if !reader.can_read() {
            return Err(CoordinateParseError::RotationIncomplete {
                cursor: reader.cursor(),
            });
        }

        let y = WorldCoordinateModel::parse_double(reader, false)?;
        if reader.can_read() && reader.peek() == ' ' {
            reader.skip();
            let x = WorldCoordinateModel::parse_double(reader, false)?;
            Ok(WorldCoordinatesModel::new(
                x,
                y,
                WorldCoordinateModel::new(true, 0.0),
            ))
        } else {
            reader.set_cursor(start);
            Err(CoordinateParseError::RotationIncomplete {
                cursor: reader.cursor(),
            })
        }
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["0 0", "~ ~", "~-5 ~5"]
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandContextModel {
    arguments: HashMap<String, CoordinatesModel>,
}

impl CommandContextModel {
    pub fn with_coordinates(
        mut self,
        name: impl Into<String>,
        value: impl Into<CoordinatesModel>,
    ) -> Self {
        self.arguments.insert(name.into(), value.into());
        self
    }
}

pub fn get_rotation(context: &CommandContextModel, name: &str) -> Option<CoordinatesModel> {
    context.arguments.get(name).copied()
}

pub fn get_coordinates(context: &CommandContextModel, name: &str) -> Option<CoordinatesModel> {
    context.arguments.get(name).copied()
}

pub fn get_vec3(
    context: &CommandContextModel,
    source: &CommandSourceStackModel,
    name: &str,
) -> Option<Vec3Model> {
    context
        .arguments
        .get(name)
        .map(|coordinates| coordinates.get_position(source))
}

pub fn get_vec2(
    context: &CommandContextModel,
    source: &CommandSourceStackModel,
    name: &str,
) -> Option<Vec2Model> {
    context.arguments.get(name).map(|coordinates| {
        let position = coordinates.get_position(source);
        Vec2Model::new(position.x as f32 as f64, position.z as f32 as f64)
    })
}

#[derive(Debug, Clone, PartialEq)]
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

    pub(crate) fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    pub(crate) fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    pub(crate) fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    pub(crate) fn skip(&mut self) {
        self.cursor += 1;
    }

    fn slice_from(&self, start: usize) -> String {
        self.input[start..self.cursor].to_string()
    }

    fn read_double(&mut self) -> Result<f64, CoordinateParseError> {
        let start = self.cursor;
        while self.can_read() && is_allowed_number(self.peek()) {
            self.skip();
        }
        let value = self.input[start..self.cursor].to_string();
        value
            .parse::<f64>()
            .map_err(|_| CoordinateParseError::InvalidDouble {
                cursor: start,
                value,
            })
    }

    fn read_int(&mut self) -> Result<i32, CoordinateParseError> {
        let start = self.cursor;
        while self.can_read() && is_allowed_number(self.peek()) {
            self.skip();
        }
        let value = self.input[start..self.cursor].to_string();
        if value.contains('.') {
            return Err(CoordinateParseError::InvalidInt {
                cursor: start,
                value,
            });
        }
        value
            .parse::<i32>()
            .map_err(|_| CoordinateParseError::InvalidInt {
                cursor: start,
                value,
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoordinateParseError {
    RotationIncomplete { cursor: usize },
    Vec2Incomplete { cursor: usize },
    Vec3Incomplete { cursor: usize },
    ExpectedDouble { cursor: usize },
    ExpectedInt { cursor: usize },
    MixedType { cursor: usize },
    InvalidDouble { cursor: usize, value: String },
    InvalidInt { cursor: usize, value: String },
}

fn is_allowed_number(value: char) -> bool {
    value.is_ascii_digit() || value == '-' || value == '.'
}

fn floor_to_i32(value: f64) -> i32 {
    value.floor() as i32
}

fn add_vec3(left: Vec3Model, right: Vec3Model) -> Vec3Model {
    Vec3Model::new(left.x + right.x, left.y + right.y, left.z + right.z)
}

fn cross(left: Vec3Model, right: Vec3Model) -> Vec3Model {
    Vec3Model::new(
        left.y * right.z - left.z * right.y,
        left.z * right.x - left.x * right.z,
        left.x * right.y - left.y * right.x,
    )
}

fn scale(value: Vec3Model, factor: f64) -> Vec3Model {
    Vec3Model::new(value.x * factor, value.y * factor, value.z * factor)
}

fn apply_local_coordinates_to_rotation(rotation: Vec2Model, direction: Vec3Model) -> Vec3Model {
    let y_cos = ((rotation.y as f32 + 90.0).to_radians()).cos();
    let y_sin = ((rotation.y as f32 + 90.0).to_radians()).sin();
    let x_cos = (-(rotation.x as f32).to_radians()).cos();
    let x_sin = (-(rotation.x as f32).to_radians()).sin();
    let x_cos_up = ((-(rotation.x as f32) + 90.0).to_radians()).cos();
    let x_sin_up = ((-(rotation.x as f32) + 90.0).to_radians()).sin();
    let forwards = Vec3Model::new(
        f64::from(y_cos * x_cos),
        f64::from(x_sin),
        f64::from(y_sin * x_cos),
    );
    let up = Vec3Model::new(
        f64::from(y_cos * x_cos_up),
        f64::from(x_sin_up),
        f64::from(y_sin * x_cos_up),
    );
    let left = scale(cross(forwards, up), -1.0);
    Vec3Model::new(
        forwards.x * direction.z + up.x * direction.y + left.x * direction.x,
        forwards.y * direction.z + up.y * direction.y + left.y * direction.x,
        forwards.z * direction.z + up.z * direction.y + left.z * direction.x,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> CommandSourceStackModel {
        CommandSourceStackModel::new(
            Vec3Model::new(10.25, 64.0, -4.75),
            Vec2Model::new(30.0, -90.0),
        )
    }

    fn parse_rotation(input: &str) -> Result<(WorldCoordinatesModel, usize), CoordinateParseError> {
        let mut reader = StringReaderModel::new(input);
        let value = RotationArgumentModel::rotation().parse(&mut reader)?;
        Ok((value, reader.cursor()))
    }

    #[test]
    fn java_rotation_factory_examples_and_context_getter_match_source() {
        const { assert!(COORDINATES_PACKAGE_NULL_MARKED) };
        assert_eq!(
            RotationArgumentModel::rotation().examples(),
            ["0 0", "~ ~", "~-5 ~5"]
        );
        let rotation = WorldCoordinatesModel::absolute_rotation(Vec2Model::new(1.0, 2.0));
        let context = CommandContextModel::default().with_coordinates("rot", rotation);

        assert_eq!(get_rotation(&context, "rot"), Some(rotation.into()));
        assert_eq!(get_rotation(&context, "missing"), None);
    }

    #[test]
    fn java_rotation_parses_y_then_x_and_uses_relative_z_zero() {
        let (coordinates, cursor) = parse_rotation("45 90 rest").unwrap_or_else(|error| {
            panic!("rotation should parse successfully: {error:?}");
        });

        assert_eq!(cursor, 5);
        assert_eq!(
            coordinates.get_rotation(&source()),
            Vec2Model::new(90.0, 45.0)
        );
        assert_eq!(coordinates.get_position(&source()).z, -4.75);
        assert!(!coordinates.is_x_relative());
        assert!(!coordinates.is_y_relative());
        assert!(coordinates.is_z_relative());
    }

    #[test]
    fn java_rotation_supports_relative_angles_against_source_rotation() {
        let (coordinates, cursor) = parse_rotation("~-5 ~5").unwrap_or_else(|error| {
            panic!("relative rotation should parse successfully: {error:?}");
        });

        assert_eq!(cursor, 6);
        assert_eq!(
            coordinates.get_rotation(&source()),
            Vec2Model::new(35.0, -95.0)
        );
        assert!(coordinates.is_x_relative());
        assert!(coordinates.is_y_relative());
        assert!(coordinates.is_z_relative());
    }

    #[test]
    fn java_rotation_reports_incomplete_and_resets_cursor_after_partial_read() {
        assert_eq!(
            parse_rotation(""),
            Err(CoordinateParseError::RotationIncomplete { cursor: 0 })
        );

        let mut reader = StringReaderModel::new("45");
        assert_eq!(
            RotationArgumentModel::rotation().parse(&mut reader),
            Err(CoordinateParseError::RotationIncomplete { cursor: 0 })
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_world_coordinate_double_matches_relative_center_and_mixed_rules() {
        let mut centered = StringReaderModel::new("1");
        assert_eq!(
            WorldCoordinateModel::parse_double(&mut centered, true),
            Ok(WorldCoordinateModel::new(false, 1.5))
        );

        let mut decimal = StringReaderModel::new("1.0");
        assert_eq!(
            WorldCoordinateModel::parse_double(&mut decimal, true),
            Ok(WorldCoordinateModel::new(false, 1.0))
        );

        let mut relative = StringReaderModel::new("~");
        assert_eq!(
            WorldCoordinateModel::parse_double(&mut relative, true),
            Ok(WorldCoordinateModel::new(true, 0.0))
        );

        let mut mixed = StringReaderModel::new("^1");
        assert_eq!(
            WorldCoordinateModel::parse_double(&mut mixed, true),
            Err(CoordinateParseError::MixedType { cursor: 0 })
        );
    }

    #[test]
    fn java_world_coordinate_int_uses_int_for_absolute_and_double_for_relative() {
        let mut absolute = StringReaderModel::new("-3");
        assert_eq!(
            WorldCoordinateModel::parse_int(&mut absolute),
            Ok(WorldCoordinateModel::new(false, -3.0))
        );

        let mut relative = StringReaderModel::new("~1.5");
        assert_eq!(
            WorldCoordinateModel::parse_int(&mut relative),
            Ok(WorldCoordinateModel::new(true, 1.5))
        );

        let mut bad_absolute = StringReaderModel::new("1.5");
        assert_eq!(
            WorldCoordinateModel::parse_int(&mut bad_absolute),
            Err(CoordinateParseError::InvalidInt {
                cursor: 0,
                value: "1.5".to_string()
            })
        );
    }

    #[test]
    fn java_world_coordinates_parse_complete_triplets_and_reset_on_incomplete() {
        let mut reader = StringReaderModel::new("1 2 3 tail");
        let coordinates =
            WorldCoordinatesModel::parse_double(&mut reader, true).unwrap_or_else(|error| {
                panic!("world coordinate triplet should parse successfully: {error:?}")
            });
        assert_eq!(reader.cursor(), 5);
        assert_eq!(
            coordinates.get_position(&source()),
            Vec3Model::new(1.5, 2.0, 3.5)
        );

        let mut incomplete = StringReaderModel::new("1 2");
        assert_eq!(
            WorldCoordinatesModel::parse_double(&mut incomplete, true),
            Err(CoordinateParseError::Vec3Incomplete { cursor: 0 })
        );
        assert_eq!(incomplete.cursor(), 0);
    }

    #[test]
    fn java_world_coordinates_parse_int_uses_integer_absolute_components() {
        let mut reader = StringReaderModel::new("1 ~2 -3 tail");
        let coordinates = WorldCoordinatesModel::parse_int(&mut reader).unwrap_or_else(|error| {
            panic!("integer world coordinate triplet should parse successfully: {error:?}")
        });

        assert_eq!(reader.cursor(), 7);
        assert_eq!(
            coordinates.get_position(&source()),
            Vec3Model::new(1.0, 66.0, -3.0)
        );
    }

    #[test]
    fn java_coordinates_default_block_pos_floors_resolved_position() {
        let coordinates = WorldCoordinatesModel::new(
            WorldCoordinateModel::new(false, -1.2),
            WorldCoordinateModel::new(false, 64.9),
            WorldCoordinateModel::new(true, -0.5),
        );

        assert_eq!(
            coordinates.get_block_pos(&source()),
            BlockPosModel {
                x: -2,
                y: 64,
                z: -6
            }
        );
    }

    #[test]
    fn java_world_coordinates_absolute_helpers_match_source_shape() {
        assert_eq!(
            WorldCoordinatesModel::absolute(1.0, 2.0, 3.0).get_position(&source()),
            Vec3Model::new(1.0, 2.0, 3.0)
        );
        assert_eq!(
            WorldCoordinatesModel::ZERO_ROTATION.get_rotation(&source()),
            Vec2Model::new(0.0, 0.0)
        );
        assert!(WorldCoordinatesModel::ZERO_ROTATION.is_z_relative());
    }

    #[test]
    fn java_local_coordinates_parse_requires_carets_and_resets_on_mixed_or_incomplete() {
        let mut reader = StringReaderModel::new("^1 ^ ^-5 tail");
        let coordinates = LocalCoordinatesModel::parse(&mut reader).unwrap_or_else(|error| {
            panic!("local coordinates should parse successfully: {error:?}")
        });

        assert_eq!(reader.cursor(), 8);
        assert_eq!(coordinates, LocalCoordinatesModel::new(1.0, 0.0, -5.0));
        assert_eq!(
            coordinates.get_rotation(&source()),
            Vec2Model::new(0.0, 0.0)
        );
        assert!(coordinates.is_x_relative());
        assert!(coordinates.is_y_relative());
        assert!(coordinates.is_z_relative());

        let mut mixed = StringReaderModel::new("^1 ~2 ^3");
        assert_eq!(
            LocalCoordinatesModel::parse(&mut mixed),
            Err(CoordinateParseError::MixedType { cursor: 0 })
        );
        assert_eq!(mixed.cursor(), 0);

        let mut incomplete = StringReaderModel::new("^1 ^2");
        assert_eq!(
            LocalCoordinatesModel::parse(&mut incomplete),
            Err(CoordinateParseError::Vec3Incomplete { cursor: 0 })
        );
        assert_eq!(incomplete.cursor(), 0);
    }

    #[test]
    fn java_local_coordinates_apply_rotation_from_anchor_position() {
        let source = CommandSourceStackModel::new(
            Vec3Model::new(99.0, 99.0, 99.0),
            Vec2Model::new(0.0, 0.0),
        )
        .with_anchor_position(Vec3Model::new(10.0, 64.0, -5.0));
        let forward = LocalCoordinatesModel::new(0.0, 0.0, 2.0);
        let up_left = LocalCoordinatesModel::new(1.0, 3.0, 0.0);

        assert_vec3_approx(
            forward.get_position(&source),
            Vec3Model::new(10.0, 64.0, -3.0),
        );
        assert_vec3_approx(
            up_left.get_position(&source),
            Vec3Model::new(11.0, 67.0, -5.0),
        );
    }

    #[test]
    fn java_vec3_argument_factories_examples_parse_and_getters_match_source() {
        assert_eq!(
            Vec3ArgumentModel::vec3().examples(),
            [
                "0 0 0",
                "~ ~ ~",
                "^ ^ ^",
                "^1 ^ ^-5",
                "0.1 -0.5 .9",
                "~0.5 ~1 ~-5"
            ]
        );

        let mut world_reader = StringReaderModel::new("1 2 3 rest");
        let world = Vec3ArgumentModel::vec3()
            .parse(&mut world_reader)
            .unwrap_or_else(|error| panic!("world vec3 should parse: {error:?}"));
        assert_eq!(world_reader.cursor(), 5);
        assert_eq!(world.get_position(&source()), Vec3Model::new(1.5, 2.0, 3.5));

        let mut no_center_reader = StringReaderModel::new("1 2 3");
        let no_center = Vec3ArgumentModel::vec3_with_center(false)
            .parse(&mut no_center_reader)
            .unwrap_or_else(|error| panic!("uncentered world vec3 should parse: {error:?}"));
        assert_eq!(
            no_center.get_position(&source()),
            Vec3Model::new(1.0, 2.0, 3.0)
        );

        let mut local_reader = StringReaderModel::new("^ ^ ^");
        let local = Vec3ArgumentModel::vec3()
            .parse(&mut local_reader)
            .unwrap_or_else(|error| panic!("local vec3 should parse: {error:?}"));
        assert!(matches!(local, CoordinatesModel::Local(_)));
        assert_eq!(local.get_rotation(&source()), Vec2Model::new(0.0, 0.0));
        assert!(local.is_x_relative());
        assert!(local.is_y_relative());
        assert!(local.is_z_relative());

        let context = CommandContextModel::default().with_coordinates("pos", world);
        assert_eq!(get_coordinates(&context, "pos"), Some(world));
        assert_eq!(
            get_vec3(&context, &source(), "pos"),
            Some(Vec3Model::new(1.5, 2.0, 3.5))
        );
        assert_eq!(
            world.get_block_pos(&source()),
            BlockPosModel { x: 1, y: 2, z: 3 }
        );
    }

    #[test]
    fn java_vec2_argument_factories_examples_parse_and_getter_match_source() {
        assert_eq!(
            Vec2ArgumentModel::vec2().examples(),
            ["0 0", "~ ~", "0.1 -0.5", "~1 ~-2"]
        );

        let mut reader = StringReaderModel::new("1 3 rest");
        let coordinates = Vec2ArgumentModel::vec2()
            .parse(&mut reader)
            .unwrap_or_else(|error| panic!("vec2 should parse: {error:?}"));

        assert_eq!(reader.cursor(), 3);
        assert_eq!(
            coordinates.get_position(&source()),
            Vec3Model::new(1.5, 64.0, 3.5)
        );
        let context = CommandContextModel::default().with_coordinates("pos", coordinates);
        assert_eq!(
            get_vec2(&context, &source(), "pos"),
            Some(Vec2Model::new(1.5, 3.5))
        );

        let mut no_center_reader = StringReaderModel::new("1 3");
        let no_center = Vec2ArgumentModel::vec2_with_center(false)
            .parse(&mut no_center_reader)
            .unwrap_or_else(|error| panic!("uncentered vec2 should parse: {error:?}"));
        assert_eq!(
            no_center.get_position(&source()),
            Vec3Model::new(1.0, 64.0, 3.0)
        );
    }

    #[test]
    fn java_vec2_rejects_local_input_and_resets_incomplete_cursor() {
        let mut local = StringReaderModel::new("^ ^");
        assert_eq!(
            Vec2ArgumentModel::vec2().parse(&mut local),
            Err(CoordinateParseError::MixedType { cursor: 0 })
        );

        let mut incomplete = StringReaderModel::new("1");
        assert_eq!(
            Vec2ArgumentModel::vec2().parse(&mut incomplete),
            Err(CoordinateParseError::Vec2Incomplete { cursor: 0 })
        );
        assert_eq!(incomplete.cursor(), 0);
    }

    #[test]
    fn java_vec_coordinate_suggestions_use_source_coordinates_and_validators() {
        let source = SuggestionProviderModel {
            online_players: Vec::new(),
        };
        let mut vec3_empty = SuggestionsBuilderModel::new("");
        assert_eq!(
            Vec3ArgumentModel::vec3().list_suggestions(Some(&source), &mut vec3_empty),
            vec![suggestion("~"), suggestion("~ ~"), suggestion("~ ~ ~")]
        );

        let mut vec3_local = SuggestionsBuilderModel::new("^");
        assert_eq!(
            Vec3ArgumentModel::vec3().list_suggestions(Some(&source), &mut vec3_local),
            vec![suggestion("^ ^"), suggestion("^ ^ ^")]
        );

        let mut vec2_empty = SuggestionsBuilderModel::new("");
        assert_eq!(
            Vec2ArgumentModel::vec2().list_suggestions(Some(&source), &mut vec2_empty),
            vec![suggestion("~"), suggestion("~ ~")]
        );

        let mut vec2_local = SuggestionsBuilderModel::new("^");
        assert!(Vec2ArgumentModel::vec2()
            .list_suggestions(Some(&source), &mut vec2_local)
            .is_empty());
        let mut non_shared = SuggestionsBuilderModel::new("");
        assert!(Vec3ArgumentModel::vec3()
            .list_suggestions(None, &mut non_shared)
            .is_empty());
    }

    fn suggestion(value: &str) -> SuggestionModel {
        SuggestionModel {
            value: value.to_string(),
            tooltip: None,
        }
    }

    fn assert_vec3_approx(actual: Vec3Model, expected: Vec3Model) {
        let epsilon = 1.0e-5;
        assert!((actual.x - expected.x).abs() < epsilon, "x: {actual:?}");
        assert!((actual.y - expected.y).abs() < epsilon, "y: {actual:?}");
        assert!((actual.z - expected.z).abs() < epsilon, "z: {actual:?}");
    }
}
