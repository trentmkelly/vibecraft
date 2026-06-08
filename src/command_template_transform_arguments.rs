use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateMirrorArgumentModel;

impl TemplateMirrorArgumentModel {
    pub fn template_mirror() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<MirrorModel, EnumParseError> {
        let id = reader.read_unquoted_string();
        MirrorModel::by_id(&id).ok_or(EnumParseError::InvalidValue { value: id })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(MirrorModel::ids().into_iter().map(str::to_string), builder);
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["none", "left_right"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateRotationArgumentModel;

impl TemplateRotationArgumentModel {
    pub fn template_rotation() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<RotationModel, EnumParseError> {
        let id = reader.read_unquoted_string();
        RotationModel::by_id(&id).ok_or(EnumParseError::InvalidValue { value: id })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(
            RotationModel::ids().into_iter().map(str::to_string),
            builder,
        );
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["none", "clockwise_90"]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirrorModel {
    None,
    LeftRight,
    FrontBack,
}

impl MirrorModel {
    pub const VALUES: [Self; 3] = [Self::None, Self::LeftRight, Self::FrontBack];

    pub fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LeftRight => "left_right",
            Self::FrontBack => "front_back",
        }
    }

    pub fn symbol_key(self) -> String {
        format!("mirror.{}", self.id())
    }

    pub fn rotation_group(self) -> &'static str {
        match self {
            Self::None => "identity",
            Self::LeftRight => "invert_z",
            Self::FrontBack => "invert_x",
        }
    }

    pub fn by_id(id: &str) -> Option<Self> {
        Self::VALUES.into_iter().find(|mirror| mirror.id() == id)
    }

    pub fn ids() -> [&'static str; 3] {
        [Self::None.id(), Self::LeftRight.id(), Self::FrontBack.id()]
    }

    pub fn mirror_rotation(self, rotation: i32, steps: i32) -> i32 {
        let half_steps = steps / 2;
        let corrected_rotation = if rotation > half_steps {
            rotation - steps
        } else {
            rotation
        };
        match self {
            Self::LeftRight => (half_steps - corrected_rotation + steps) % steps,
            Self::FrontBack => (steps - corrected_rotation) % steps,
            Self::None => rotation,
        }
    }

    pub fn rotation_for_direction(self, direction: DirectionModel) -> RotationModel {
        let axis = direction.axis();
        if (self == Self::LeftRight && axis == AxisModel::Z)
            || (self == Self::FrontBack && axis == AxisModel::X)
        {
            RotationModel::Clockwise180
        } else {
            RotationModel::None
        }
    }

    pub fn mirror_direction(self, direction: DirectionModel) -> DirectionModel {
        if (self == Self::FrontBack && direction.axis() == AxisModel::X)
            || (self == Self::LeftRight && direction.axis() == AxisModel::Z)
        {
            direction.opposite()
        } else {
            direction
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotationModel {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl RotationModel {
    pub const VALUES: [Self; 4] = [
        Self::None,
        Self::Clockwise90,
        Self::Clockwise180,
        Self::Counterclockwise90,
    ];

    pub fn index(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Counterclockwise90 => 3,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }

    pub fn rotation_group(self) -> &'static str {
        match self {
            Self::None => "identity",
            Self::Clockwise90 => "rot_90_y_neg",
            Self::Clockwise180 => "rot_180_face_xz",
            Self::Counterclockwise90 => "rot_90_y_pos",
        }
    }

    pub fn by_id(id: &str) -> Option<Self> {
        Self::VALUES
            .into_iter()
            .find(|rotation| rotation.id() == id)
    }

    pub fn by_index_wrapped(index: i32) -> Self {
        Self::VALUES[index.rem_euclid(Self::VALUES.len() as i32) as usize]
    }

    pub fn ids() -> [&'static str; 4] {
        [
            Self::None.id(),
            Self::Clockwise90.id(),
            Self::Clockwise180.id(),
            Self::Counterclockwise90.id(),
        ]
    }

    pub fn get_rotated(self, rotation: Self) -> Self {
        match rotation {
            Self::Clockwise90 => match self {
                Self::None => Self::Clockwise90,
                Self::Clockwise90 => Self::Clockwise180,
                Self::Clockwise180 => Self::Counterclockwise90,
                Self::Counterclockwise90 => Self::None,
            },
            Self::Clockwise180 => match self {
                Self::None => Self::Clockwise180,
                Self::Clockwise90 => Self::Counterclockwise90,
                Self::Clockwise180 => Self::None,
                Self::Counterclockwise90 => Self::Clockwise90,
            },
            Self::Counterclockwise90 => match self {
                Self::None => Self::Counterclockwise90,
                Self::Clockwise90 => Self::None,
                Self::Clockwise180 => Self::Clockwise90,
                Self::Counterclockwise90 => Self::Clockwise180,
            },
            Self::None => self,
        }
    }

    pub fn rotate_direction(self, direction: DirectionModel) -> DirectionModel {
        if direction.axis() == AxisModel::Y {
            return direction;
        }
        match self {
            Self::Clockwise90 => direction.clockwise(),
            Self::Clockwise180 => direction.opposite(),
            Self::Counterclockwise90 => direction.counterclockwise(),
            Self::None => direction,
        }
    }

    pub fn rotate_steps(self, rotation: i32, steps: i32) -> i32 {
        match self {
            Self::Clockwise90 => (rotation + steps / 4) % steps,
            Self::Clockwise180 => (rotation + steps / 2) % steps,
            Self::Counterclockwise90 => (rotation + steps * 3 / 4) % steps,
            Self::None => rotation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionModel {
    North,
    South,
    West,
    East,
    Up,
    Down,
}

impl DirectionModel {
    pub fn axis(self) -> AxisModel {
        match self {
            Self::East | Self::West => AxisModel::X,
            Self::Up | Self::Down => AxisModel::Y,
            Self::North | Self::South => AxisModel::Z,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }

    pub fn clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::Up | Self::Down => self,
        }
    }

    pub fn counterclockwise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            Self::Up | Self::Down => self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisModel {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    mirrors: HashMap<String, MirrorModel>,
    rotations: HashMap<String, RotationModel>,
}

impl CommandContextModel {
    pub fn with_mirror(mut self, name: impl Into<String>, value: MirrorModel) -> Self {
        self.mirrors.insert(name.into(), value);
        self
    }

    pub fn with_rotation(mut self, name: impl Into<String>, value: RotationModel) -> Self {
        self.rotations.insert(name.into(), value);
        self
    }
}

pub fn get_mirror(context: &CommandContextModel, name: &str) -> Option<MirrorModel> {
    context.mirrors.get(name).copied()
}

pub fn get_rotation(context: &CommandContextModel, name: &str) -> Option<RotationModel> {
    context.rotations.get(name).copied()
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

    fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && !self.peek().is_ascii_whitespace() {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumParseError {
    InvalidValue { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_mirror(input: &str) -> Result<(MirrorModel, usize), EnumParseError> {
        let mut reader = StringReaderModel::new(input);
        let mirror = TemplateMirrorArgumentModel::template_mirror().parse(&mut reader)?;
        Ok((mirror, reader.cursor()))
    }

    fn parse_rotation(input: &str) -> Result<(RotationModel, usize), EnumParseError> {
        let mut reader = StringReaderModel::new(input);
        let rotation = TemplateRotationArgumentModel::template_rotation().parse(&mut reader)?;
        Ok((rotation, reader.cursor()))
    }

    #[test]
    fn mirror_argument_uses_string_representable_parse_examples_and_suggestions() {
        assert_eq!(
            TemplateMirrorArgumentModel::template_mirror().examples(),
            ["none", "left_right"]
        );
        assert_eq!(
            parse_mirror("front_back tail"),
            Ok((MirrorModel::FrontBack, 10))
        );
        assert_eq!(
            parse_mirror("FRONT_BACK"),
            Err(EnumParseError::InvalidValue {
                value: "FRONT_BACK".to_string(),
            })
        );

        let mut builder = SuggestionsBuilderModel::new("f");
        assert_eq!(
            TemplateMirrorArgumentModel::template_mirror().list_suggestions(&mut builder),
            vec![SuggestionModel {
                value: "front_back".to_string(),
                tooltip: None,
            }]
        );
    }

    #[test]
    fn rotation_argument_uses_string_representable_parse_examples_and_suggestions() {
        assert_eq!(
            TemplateRotationArgumentModel::template_rotation().examples(),
            ["none", "clockwise_90"]
        );
        assert_eq!(
            parse_rotation("counterclockwise_90 tail"),
            Ok((RotationModel::Counterclockwise90, 19))
        );
        assert_eq!(
            parse_rotation("clockwise_180"),
            Err(EnumParseError::InvalidValue {
                value: "clockwise_180".to_string(),
            })
        );

        let mut builder = SuggestionsBuilderModel::new("c");
        assert_eq!(
            TemplateRotationArgumentModel::template_rotation().list_suggestions(&mut builder),
            vec![
                SuggestionModel {
                    value: "clockwise_90".to_string(),
                    tooltip: None,
                },
                SuggestionModel {
                    value: "counterclockwise_90".to_string(),
                    tooltip: None,
                }
            ]
        );
    }

    #[test]
    fn context_accessors_return_typed_template_arguments() {
        let context = CommandContextModel::default()
            .with_mirror("mirror", MirrorModel::LeftRight)
            .with_rotation("rotation", RotationModel::Clockwise90);

        assert_eq!(get_mirror(&context, "mirror"), Some(MirrorModel::LeftRight));
        assert_eq!(
            get_rotation(&context, "rotation"),
            Some(RotationModel::Clockwise90)
        );
        assert_eq!(get_mirror(&context, "missing"), None);
        assert_eq!(get_rotation(&context, "missing"), None);
    }

    #[test]
    fn mirror_metadata_and_rotation_index_math_match_java() {
        assert_eq!(
            MirrorModel::VALUES.map(|mirror| (mirror.id(), mirror.rotation_group())),
            [
                ("none", "identity"),
                ("left_right", "invert_z"),
                ("front_back", "invert_x"),
            ]
        );
        assert_eq!(MirrorModel::FrontBack.symbol_key(), "mirror.front_back");
        assert_eq!(MirrorModel::None.mirror_rotation(3, 16), 3);
        assert_eq!(MirrorModel::LeftRight.mirror_rotation(3, 16), 5);
        assert_eq!(MirrorModel::FrontBack.mirror_rotation(3, 16), 13);
        assert_eq!(MirrorModel::LeftRight.mirror_rotation(9, 16), 15);
    }

    #[test]
    fn mirror_direction_and_result_rotation_match_java_axis_checks() {
        assert_eq!(
            MirrorModel::LeftRight.rotation_for_direction(DirectionModel::North),
            RotationModel::Clockwise180
        );
        assert_eq!(
            MirrorModel::LeftRight.rotation_for_direction(DirectionModel::East),
            RotationModel::None
        );
        assert_eq!(
            MirrorModel::FrontBack.rotation_for_direction(DirectionModel::East),
            RotationModel::Clockwise180
        );
        assert_eq!(
            MirrorModel::LeftRight.mirror_direction(DirectionModel::North),
            DirectionModel::South
        );
        assert_eq!(
            MirrorModel::FrontBack.mirror_direction(DirectionModel::East),
            DirectionModel::West
        );
        assert_eq!(
            MirrorModel::FrontBack.mirror_direction(DirectionModel::North),
            DirectionModel::North
        );
    }

    #[test]
    fn rotation_metadata_by_id_wrap_and_composition_match_java() {
        assert_eq!(
            RotationModel::VALUES
                .map(|rotation| { (rotation.index(), rotation.id(), rotation.rotation_group()) }),
            [
                (0, "none", "identity"),
                (1, "clockwise_90", "rot_90_y_neg"),
                (2, "180", "rot_180_face_xz"),
                (3, "counterclockwise_90", "rot_90_y_pos"),
            ]
        );
        assert_eq!(
            RotationModel::by_index_wrapped(-1),
            RotationModel::Counterclockwise90
        );
        assert_eq!(RotationModel::by_index_wrapped(4), RotationModel::None);
        assert_eq!(
            RotationModel::Clockwise90.get_rotated(RotationModel::Clockwise180),
            RotationModel::Counterclockwise90
        );
        assert_eq!(
            RotationModel::Counterclockwise90.get_rotated(RotationModel::Clockwise90),
            RotationModel::None
        );
        assert_eq!(
            RotationModel::Clockwise180.get_rotated(RotationModel::Counterclockwise90),
            RotationModel::Clockwise90
        );
    }

    #[test]
    fn rotation_direction_and_step_math_match_java() {
        assert_eq!(
            RotationModel::Clockwise90.rotate_direction(DirectionModel::North),
            DirectionModel::East
        );
        assert_eq!(
            RotationModel::Clockwise180.rotate_direction(DirectionModel::East),
            DirectionModel::West
        );
        assert_eq!(
            RotationModel::Counterclockwise90.rotate_direction(DirectionModel::South),
            DirectionModel::East
        );
        assert_eq!(
            RotationModel::Clockwise90.rotate_direction(DirectionModel::Up),
            DirectionModel::Up
        );
        assert_eq!(RotationModel::Clockwise90.rotate_steps(2, 16), 6);
        assert_eq!(RotationModel::Clockwise180.rotate_steps(2, 16), 10);
        assert_eq!(RotationModel::Counterclockwise90.rotate_steps(2, 16), 14);
    }
}
