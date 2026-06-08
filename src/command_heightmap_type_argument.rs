use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightmapTypeArgumentModel;

impl HeightmapTypeArgumentModel {
    pub fn heightmap() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<HeightmapTypeModel, HeightmapTypeParseError> {
        let id = reader.read_unquoted_string();
        HeightmapTypeModel::from_lowercase_id(&id)
            .filter(|heightmap| heightmap.keep_after_worldgen())
            .ok_or(HeightmapTypeParseError::InvalidValue { value: id })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(Self::ids().into_iter().map(str::to_string), builder);
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["world_surface", "ocean_floor"]
    }

    pub fn ids() -> [&'static str; 4] {
        [
            HeightmapTypeModel::WorldSurface.lower_case_id(),
            HeightmapTypeModel::OceanFloor.lower_case_id(),
            HeightmapTypeModel::MotionBlocking.lower_case_id(),
            HeightmapTypeModel::MotionBlockingNoLeaves.lower_case_id(),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapTypeModel {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

impl HeightmapTypeModel {
    pub const VALUES: [Self; 6] = [
        Self::WorldSurfaceWg,
        Self::WorldSurface,
        Self::OceanFloorWg,
        Self::OceanFloor,
        Self::MotionBlocking,
        Self::MotionBlockingNoLeaves,
    ];

    pub fn serialization_key(self) -> &'static str {
        match self {
            Self::WorldSurfaceWg => "WORLD_SURFACE_WG",
            Self::WorldSurface => "WORLD_SURFACE",
            Self::OceanFloorWg => "OCEAN_FLOOR_WG",
            Self::OceanFloor => "OCEAN_FLOOR",
            Self::MotionBlocking => "MOTION_BLOCKING",
            Self::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
        }
    }

    pub fn lower_case_id(self) -> &'static str {
        match self {
            Self::WorldSurfaceWg => "world_surface_wg",
            Self::WorldSurface => "world_surface",
            Self::OceanFloorWg => "ocean_floor_wg",
            Self::OceanFloor => "ocean_floor",
            Self::MotionBlocking => "motion_blocking",
            Self::MotionBlockingNoLeaves => "motion_blocking_no_leaves",
        }
    }

    pub fn usage(self) -> HeightmapUsageModel {
        match self {
            Self::WorldSurfaceWg | Self::OceanFloorWg => HeightmapUsageModel::Worldgen,
            Self::OceanFloor => HeightmapUsageModel::LiveWorld,
            Self::WorldSurface | Self::MotionBlocking | Self::MotionBlockingNoLeaves => {
                HeightmapUsageModel::Client
            }
        }
    }

    pub fn keep_after_worldgen(self) -> bool {
        self.usage() != HeightmapUsageModel::Worldgen
    }

    pub fn send_to_client(self) -> bool {
        self.usage() == HeightmapUsageModel::Client
    }

    pub fn kept_types() -> Vec<Self> {
        Self::VALUES
            .into_iter()
            .filter(|heightmap| heightmap.keep_after_worldgen())
            .collect()
    }

    fn from_lowercase_id(id: &str) -> Option<Self> {
        Self::VALUES
            .into_iter()
            .find(|heightmap| heightmap.lower_case_id() == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapUsageModel {
    Worldgen,
    LiveWorld,
    Client,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, HeightmapTypeModel>,
}

impl CommandContextModel {
    pub fn with_heightmap(mut self, name: impl Into<String>, value: HeightmapTypeModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_heightmap(context: &CommandContextModel, name: &str) -> Option<HeightmapTypeModel> {
    context.arguments.get(name).copied()
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
pub enum HeightmapTypeParseError {
    InvalidValue { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(HeightmapTypeModel, usize), HeightmapTypeParseError> {
        let mut reader = StringReaderModel::new(input);
        let heightmap = HeightmapTypeArgumentModel::heightmap().parse(&mut reader)?;
        Ok((heightmap, reader.cursor()))
    }

    #[test]
    fn java_heightmap_type_metadata_matches_source_enum() {
        assert_eq!(
            HeightmapTypeModel::VALUES.map(HeightmapTypeModel::serialization_key),
            [
                "WORLD_SURFACE_WG",
                "WORLD_SURFACE",
                "OCEAN_FLOOR_WG",
                "OCEAN_FLOOR",
                "MOTION_BLOCKING",
                "MOTION_BLOCKING_NO_LEAVES",
            ]
        );
        assert!(!HeightmapTypeModel::WorldSurfaceWg.keep_after_worldgen());
        assert!(!HeightmapTypeModel::OceanFloorWg.keep_after_worldgen());
        assert!(HeightmapTypeModel::OceanFloor.keep_after_worldgen());
        assert!(!HeightmapTypeModel::OceanFloor.send_to_client());
        assert!(HeightmapTypeModel::WorldSurface.send_to_client());
        assert!(HeightmapTypeModel::MotionBlocking.send_to_client());
        assert_eq!(
            HeightmapTypeModel::kept_types(),
            vec![
                HeightmapTypeModel::WorldSurface,
                HeightmapTypeModel::OceanFloor,
                HeightmapTypeModel::MotionBlocking,
                HeightmapTypeModel::MotionBlockingNoLeaves,
            ]
        );
    }

    #[test]
    fn examples_are_first_two_kept_types_after_lowercase_conversion() {
        assert_eq!(
            HeightmapTypeArgumentModel::heightmap().examples(),
            ["world_surface", "ocean_floor"]
        );
    }

    #[test]
    fn get_heightmap_returns_typed_context_argument() {
        let context = CommandContextModel::default()
            .with_heightmap("heightmap", HeightmapTypeModel::MotionBlocking);

        assert_eq!(
            get_heightmap(&context, "heightmap"),
            Some(HeightmapTypeModel::MotionBlocking)
        );
        assert_eq!(get_heightmap(&context, "missing"), None);
    }

    #[test]
    fn parse_accepts_lowercase_kept_heightmap_ids_and_advances_cursor() {
        let (heightmap, cursor) = parse("motion_blocking_no_leaves rest").unwrap();

        assert_eq!(heightmap, HeightmapTypeModel::MotionBlockingNoLeaves);
        assert_eq!(cursor, 25);
        assert_eq!(
            parse("ocean_floor"),
            Ok((HeightmapTypeModel::OceanFloor, 11))
        );
    }

    #[test]
    fn parse_rejects_worldgen_only_and_case_mismatched_values_without_cursor_reset() {
        let mut reader = StringReaderModel::new("world_surface_wg rest");
        assert_eq!(
            HeightmapTypeArgumentModel::heightmap().parse(&mut reader),
            Err(HeightmapTypeParseError::InvalidValue {
                value: "world_surface_wg".to_string(),
            })
        );
        assert_eq!(reader.cursor(), 16);

        let mut upper = StringReaderModel::new("WORLD_SURFACE rest");
        assert_eq!(
            HeightmapTypeArgumentModel::heightmap().parse(&mut upper),
            Err(HeightmapTypeParseError::InvalidValue {
                value: "WORLD_SURFACE".to_string(),
            })
        );
        assert_eq!(upper.cursor(), 13);
    }

    #[test]
    fn parse_rejects_unknown_and_empty_values() {
        assert_eq!(
            parse("heightmap"),
            Err(HeightmapTypeParseError::InvalidValue {
                value: "heightmap".to_string(),
            })
        );
        assert_eq!(
            parse(""),
            Err(HeightmapTypeParseError::InvalidValue {
                value: String::new(),
            })
        );
    }

    #[test]
    fn suggestions_use_kept_lowercase_ids_and_shared_matching() {
        let mut all = SuggestionsBuilderModel::new("");
        assert_eq!(
            HeightmapTypeArgumentModel::heightmap()
                .list_suggestions(&mut all)
                .into_iter()
                .map(|suggestion| suggestion.value)
                .collect::<Vec<_>>(),
            HeightmapTypeArgumentModel::ids()
        );

        let mut motion = SuggestionsBuilderModel::new("motion");
        assert_eq!(
            HeightmapTypeArgumentModel::heightmap().list_suggestions(&mut motion),
            vec![
                SuggestionModel {
                    value: "motion_blocking".to_string(),
                    tooltip: None,
                },
                SuggestionModel {
                    value: "motion_blocking_no_leaves".to_string(),
                    tooltip: None,
                }
            ]
        );
    }
}
