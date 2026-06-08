use std::collections::BTreeSet;

use crate::command_coordinate_arguments::{
    get_coordinates, BlockPosModel, CommandContextModel, CommandSourceStackModel,
    CoordinateParseError, CoordinatesModel, LocalCoordinatesModel, StringReaderModel, Vec2Model,
    Vec3Model, WorldCoordinateModel, WorldCoordinatesModel,
};
use crate::command_shared_suggestion_provider::{
    suggest_2d_coordinates, suggest_coordinates, SuggestionModel, SuggestionProviderModel,
    SuggestionsBuilderModel, TextCoordinates,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnPosModel {
    pub x: i32,
    pub z: i32,
}

impl ColumnPosModel {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn to_chunk_pos(self) -> (i32, i32) {
        (
            block_to_section_coord(self.x),
            block_to_section_coord(self.z),
        )
    }

    pub fn to_long(self) -> i64 {
        Self::as_long(self.x, self.z)
    }

    pub fn as_long(x: i32, z: i32) -> i64 {
        (i64::from(x) & 0xffff_ffff) | ((i64::from(z) & 0xffff_ffff) << 32)
    }

    pub fn get_x(pos: i64) -> i32 {
        pos as i32
    }

    pub fn get_z(pos: i64) -> i32 {
        ((pos as u64 >> 32) & 0xffff_ffff) as i32
    }

    pub fn java_to_string(self) -> String {
        format!("[{}, {}]", self.x, self.z)
    }

    pub fn java_hash_code(self) -> i32 {
        chunk_pos_hash(self.x, self.z)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPosArgumentModel;

impl BlockPosArgumentModel {
    pub fn block_pos() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<CoordinatesModel, PositionArgumentError> {
        if reader.can_read() && reader.peek() == LocalCoordinatesModel::PREFIX_LOCAL_COORDINATE {
            LocalCoordinatesModel::parse(reader)
                .map(CoordinatesModel::from)
                .map_err(PositionArgumentError::Coordinate)
        } else {
            WorldCoordinatesModel::parse_int(reader)
                .map(CoordinatesModel::from)
                .map_err(PositionArgumentError::Coordinate)
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
            source.get_relevant_coordinates()
        };
        suggest_coordinates(&remainder, &suggested_coordinates, builder, |candidate| {
            let mut reader = StringReaderModel::new(candidate);
            self.parse(&mut reader).is_ok()
        });
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 5] {
        ["0 0 0", "~ ~ ~", "^ ^ ^", "^1 ^ ^-5", "~0.5 ~1 ~-5"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnPosArgumentModel;

impl ColumnPosArgumentModel {
    pub fn column_pos() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<CoordinatesModel, PositionArgumentError> {
        let start = reader.cursor();
        if !reader.can_read() {
            return Err(PositionArgumentError::ColumnPosIncomplete {
                cursor: reader.cursor(),
            });
        }

        let x = WorldCoordinateModel::parse_int(reader)?;
        if reader.can_read() && reader.peek() == ' ' {
            reader.skip();
            let z = WorldCoordinateModel::parse_int(reader)?;
            Ok(WorldCoordinatesModel::new(x, WorldCoordinateModel::new(true, 0.0), z).into())
        } else {
            reader.set_cursor(start);
            Err(PositionArgumentError::ColumnPosIncomplete {
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
            source.get_relevant_coordinates()
        };
        suggest_2d_coordinates(&remainder, &suggested_coordinates, builder, |candidate| {
            let mut reader = StringReaderModel::new(candidate);
            self.parse(&mut reader).is_ok()
        });
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 5] {
        ["0 0", "~ ~", "~1 ~-2", "^ ^", "^-1 ^0"]
    }
}

pub fn get_block_pos(
    context: &CommandContextModel,
    source: &CommandSourceStackModel,
    name: &str,
) -> Option<BlockPosModel> {
    get_coordinates(context, name).map(|coordinates| coordinates.get_block_pos(source))
}

pub fn get_loaded_block_pos(
    context: &CommandContextModel,
    level: &ServerLevelModel,
    source: &CommandSourceStackModel,
    name: &str,
) -> Result<BlockPosModel, PositionArgumentError> {
    let pos = get_block_pos(context, source, name).ok_or(PositionArgumentError::MissingArgument)?;
    if !level.has_chunk_at(pos) {
        Err(PositionArgumentError::PositionNotLoaded)
    } else if !level.is_in_world_bounds(pos) {
        Err(PositionArgumentError::PositionOutOfWorld)
    } else {
        Ok(pos)
    }
}

pub fn get_spawnable_pos(
    context: &CommandContextModel,
    source: &CommandSourceStackModel,
    name: &str,
) -> Result<BlockPosModel, PositionArgumentError> {
    let pos = get_block_pos(context, source, name).ok_or(PositionArgumentError::MissingArgument)?;
    if is_in_spawnable_bounds(pos) {
        Ok(pos)
    } else {
        Err(PositionArgumentError::PositionOutOfBounds)
    }
}

pub fn get_column_pos(
    context: &CommandContextModel,
    source: &CommandSourceStackModel,
    name: &str,
) -> Option<ColumnPosModel> {
    get_block_pos(context, source, name).map(|pos| ColumnPosModel::new(pos.x, pos.z))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLevelModel {
    min_y: i32,
    height: i32,
    loaded_chunks: BTreeSet<(i32, i32)>,
}

impl ServerLevelModel {
    pub fn new(
        min_y: i32,
        height: i32,
        loaded_chunks: impl IntoIterator<Item = (i32, i32)>,
    ) -> Self {
        Self {
            min_y,
            height,
            loaded_chunks: loaded_chunks.into_iter().collect(),
        }
    }

    fn max_y(&self) -> i32 {
        self.min_y + self.height - 1
    }

    fn has_chunk_at(&self, pos: BlockPosModel) -> bool {
        self.loaded_chunks
            .contains(&(block_to_section_coord(pos.x), block_to_section_coord(pos.z)))
    }

    fn is_in_world_bounds(&self, pos: BlockPosModel) -> bool {
        pos.y >= self.min_y && pos.y <= self.max_y() && is_in_world_bounds_horizontal(pos)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionArgumentError {
    Coordinate(CoordinateParseError),
    ColumnPosIncomplete { cursor: usize },
    MissingArgument,
    PositionNotLoaded,
    PositionOutOfWorld,
    PositionOutOfBounds,
}

impl From<CoordinateParseError> for PositionArgumentError {
    fn from(value: CoordinateParseError) -> Self {
        Self::Coordinate(value)
    }
}

fn block_to_section_coord(value: i32) -> i32 {
    value >> 4
}

fn chunk_pos_hash(x: i32, z: i32) -> i32 {
    let x_transform = 1_664_525i32.wrapping_mul(x).wrapping_add(1_013_904_223);
    let z_transform = 1_664_525i32
        .wrapping_mul(z ^ -559_038_737)
        .wrapping_add(1_013_904_223);
    x_transform ^ z_transform
}

fn is_in_world_bounds_horizontal(pos: BlockPosModel) -> bool {
    pos.x >= -30_000_000 && pos.z >= -30_000_000 && pos.x < 30_000_000 && pos.z < 30_000_000
}

fn is_in_spawnable_bounds(pos: BlockPosModel) -> bool {
    pos.y >= -20_000_000 && pos.y < 20_000_000 && is_in_world_bounds_horizontal(pos)
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

    #[test]
    fn java_block_pos_argument_parse_examples_getters_and_suggestions_match_source() {
        assert_eq!(
            BlockPosArgumentModel::block_pos().examples(),
            ["0 0 0", "~ ~ ~", "^ ^ ^", "^1 ^ ^-5", "~0.5 ~1 ~-5"]
        );

        let mut world_reader = StringReaderModel::new("1 ~2 -3 tail");
        let world = BlockPosArgumentModel::block_pos()
            .parse(&mut world_reader)
            .unwrap_or_else(|error| panic!("block pos should parse world ints: {error:?}"));
        assert_eq!(world_reader.cursor(), 7);
        let context = CommandContextModel::default().with_coordinates("pos", world);
        assert_eq!(
            get_block_pos(&context, &source(), "pos"),
            Some(BlockPosModel { x: 1, y: 66, z: -3 })
        );

        let mut local_reader = StringReaderModel::new("^ ^ ^");
        let local = BlockPosArgumentModel::block_pos()
            .parse(&mut local_reader)
            .unwrap_or_else(|error| panic!("block pos should parse local coordinates: {error:?}"));
        assert!(matches!(local, CoordinatesModel::Local(_)));

        let source = SuggestionProviderModel {
            online_players: Vec::new(),
        };
        let mut empty = SuggestionsBuilderModel::new("");
        assert_eq!(
            BlockPosArgumentModel::block_pos().list_suggestions(Some(&source), &mut empty),
            vec![suggestion("~"), suggestion("~ ~"), suggestion("~ ~ ~")]
        );
        let mut local_suggestions = SuggestionsBuilderModel::new("^");
        assert_eq!(
            BlockPosArgumentModel::block_pos()
                .list_suggestions(Some(&source), &mut local_suggestions),
            vec![suggestion("^ ^"), suggestion("^ ^ ^")]
        );
    }

    #[test]
    fn java_loaded_and_spawnable_block_pos_validation_match_error_order() {
        let source = source();
        let level = ServerLevelModel::new(-64, 384, [(0, 0), (1_875_000, 0)]);
        let loaded = CommandContextModel::default()
            .with_coordinates("pos", WorldCoordinatesModel::absolute(1.0, 64.0, 2.0));
        assert_eq!(
            get_loaded_block_pos(&loaded, &level, &source, "pos"),
            Ok(BlockPosModel { x: 1, y: 64, z: 2 })
        );
        assert_eq!(
            get_spawnable_pos(&loaded, &source, "pos"),
            Ok(BlockPosModel { x: 1, y: 64, z: 2 })
        );

        let unloaded = CommandContextModel::default()
            .with_coordinates("pos", WorldCoordinatesModel::absolute(32.0, 64.0, 0.0));
        assert_eq!(
            get_loaded_block_pos(&unloaded, &level, &source, "pos"),
            Err(PositionArgumentError::PositionNotLoaded)
        );

        let out_of_world_y = CommandContextModel::default()
            .with_coordinates("pos", WorldCoordinatesModel::absolute(1.0, 400.0, 2.0));
        assert_eq!(
            get_loaded_block_pos(&out_of_world_y, &level, &source, "pos"),
            Err(PositionArgumentError::PositionOutOfWorld)
        );

        let out_of_world_horizontal = CommandContextModel::default().with_coordinates(
            "pos",
            WorldCoordinatesModel::absolute(30_000_000.0, 64.0, 0.0),
        );
        assert_eq!(
            get_loaded_block_pos(&out_of_world_horizontal, &level, &source, "pos"),
            Err(PositionArgumentError::PositionOutOfWorld)
        );

        let unloaded_and_out_of_world = CommandContextModel::default().with_coordinates(
            "pos",
            WorldCoordinatesModel::absolute(-30_000_001.0, 64.0, 0.0),
        );
        assert_eq!(
            get_loaded_block_pos(&unloaded_and_out_of_world, &level, &source, "pos"),
            Err(PositionArgumentError::PositionNotLoaded)
        );

        let out_of_spawnable = CommandContextModel::default().with_coordinates(
            "pos",
            WorldCoordinatesModel::absolute(1.0, 20_000_000.0, 2.0),
        );
        assert_eq!(
            get_spawnable_pos(&out_of_spawnable, &source, "pos"),
            Err(PositionArgumentError::PositionOutOfBounds)
        );
    }

    #[test]
    fn java_column_pos_argument_parse_getter_examples_and_suggestions_match_source() {
        assert_eq!(
            ColumnPosArgumentModel::column_pos().examples(),
            ["0 0", "~ ~", "~1 ~-2", "^ ^", "^-1 ^0"]
        );

        let mut reader = StringReaderModel::new("1 ~-2 tail");
        let coordinates = ColumnPosArgumentModel::column_pos()
            .parse(&mut reader)
            .unwrap_or_else(|error| panic!("column pos should parse world ints: {error:?}"));
        assert_eq!(reader.cursor(), 5);
        let context = CommandContextModel::default().with_coordinates("column", coordinates);
        assert_eq!(
            get_column_pos(&context, &source(), "column"),
            Some(ColumnPosModel::new(1, -7))
        );

        let column = ColumnPosModel::new(-1, 32);
        assert_eq!(column.to_chunk_pos(), (-1, 2));
        assert_eq!(ColumnPosModel::get_x(column.to_long()), -1);
        assert_eq!(ColumnPosModel::get_z(column.to_long()), 32);
        assert_eq!(column.java_to_string(), "[-1, 32]");
        assert_eq!(column.java_hash_code(), chunk_pos_hash(-1, 32));

        let mut incomplete = StringReaderModel::new("1");
        assert_eq!(
            ColumnPosArgumentModel::column_pos().parse(&mut incomplete),
            Err(PositionArgumentError::ColumnPosIncomplete { cursor: 0 })
        );
        assert_eq!(incomplete.cursor(), 0);

        let mut local = StringReaderModel::new("^ ^");
        assert_eq!(
            ColumnPosArgumentModel::column_pos().parse(&mut local),
            Err(PositionArgumentError::Coordinate(
                CoordinateParseError::MixedType { cursor: 0 }
            ))
        );

        let source = SuggestionProviderModel {
            online_players: Vec::new(),
        };
        let mut empty = SuggestionsBuilderModel::new("");
        assert_eq!(
            ColumnPosArgumentModel::column_pos().list_suggestions(Some(&source), &mut empty),
            vec![suggestion("~"), suggestion("~ ~")]
        );
        let mut local_suggestions = SuggestionsBuilderModel::new("^");
        assert!(ColumnPosArgumentModel::column_pos()
            .list_suggestions(Some(&source), &mut local_suggestions)
            .is_empty());
    }

    fn suggestion(value: &str) -> SuggestionModel {
        SuggestionModel {
            value: value.to_string(),
            tooltip: None,
        }
    }
}
