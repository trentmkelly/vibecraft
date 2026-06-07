#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::resources::{
    CLIENT_RESOURCE_PACK_FORMAT_MAJOR, CLIENT_RESOURCE_PACK_FORMAT_MINOR,
    SERVER_DATA_PACK_FORMAT_MAJOR, SERVER_DATA_PACK_FORMAT_MINOR,
};
use crate::world_version::{
    WorldVersionModel, CURRENT_DATA_SERIES, CURRENT_DATA_VERSION, CURRENT_PROTOCOL_VERSION,
};

pub const SNAPSHOT: bool = false;
pub const WORLD_VERSION: i32 = CURRENT_DATA_VERSION;
pub const SERIES: &str = CURRENT_DATA_SERIES;
pub const RELEASE_NETWORK_PROTOCOL_VERSION: i32 = CURRENT_PROTOCOL_VERSION;
pub const SNAPSHOT_NETWORK_PROTOCOL_VERSION: i32 = 307;
pub const SNBT_NAG_VERSION: i32 = 4763;
pub const SNAPSHOT_PROTOCOL_BIT: i32 = 30;
pub const CRASH_EAGERLY: bool = false;
pub const RESOURCE_PACK_FORMAT_MAJOR: u32 = CLIENT_RESOURCE_PACK_FORMAT_MAJOR;
pub const RESOURCE_PACK_FORMAT_MINOR: u32 = CLIENT_RESOURCE_PACK_FORMAT_MINOR;
pub const DATA_PACK_FORMAT_MAJOR: u32 = SERVER_DATA_PACK_FORMAT_MAJOR;
pub const DATA_PACK_FORMAT_MINOR: u32 = SERVER_DATA_PACK_FORMAT_MINOR;
pub const RPC_MANAGEMENT_SERVER_API_VERSION: &str = "2.0.0";
pub const LANGUAGE_FORMAT: i32 = 1;
pub const REPORT_FORMAT_VERSION: i32 = 1;
pub const DATA_VERSION_TAG: &str = "DataVersion";
pub const DEBUG_FLAG_PREFIX: &str = "MC_DEBUG_";
pub const FIX_TNT_DUPE: bool = false;
pub const FIX_SAND_DUPE: bool = false;
pub const DEFAULT_MINECRAFT_PORT: i32 = 25565;
pub const MAXIMUM_TICK_TIME_NANOS: i64 = 300_000_000;
pub const MAXIMUM_BLOCK_EXPLOSION_RESISTANCE: f32 = 3_600_000.0;
pub const USE_DEVONLY: bool = false;
pub const WORLD_RESOLUTION: i32 = 16;
pub const MAX_CHAT_LENGTH: i32 = 256;
pub const MAX_USER_INPUT_COMMAND_LENGTH: i32 = 32_500;
pub const MAX_FUNCTION_COMMAND_LENGTH: i32 = 2_000_000;
pub const MAX_PLAYER_NAME_LENGTH: i32 = 16;
pub const MAX_CHAINED_NEIGHBOR_UPDATES: i32 = 1_000_000;
pub const MAX_RENDER_DISTANCE: i32 = 32;
pub const MAX_CLOUD_DISTANCE: i32 = 128;
pub const ILLEGAL_FILE_CHARACTERS: &[char] = &[
    '/', '\n', '\r', '\t', '\0', '\u{000C}', '`', '?', '*', '\\', '<', '>', '|', '"', ':',
];
pub const TICKS_PER_SECOND: i32 = 20;
pub const MILLIS_PER_TICK: i32 = 50;
pub const TICKS_PER_MINUTE: i32 = 1_200;
pub const TICKS_PER_GAME_DAY: i32 = 24_000;
pub const DEFAULT_RANDOM_TICK_SPEED: i32 = 3;
pub const AVERAGE_GAME_TICKS_PER_RANDOM_TICK_PER_BLOCK: f32 = 1365.3334;
pub const AVERAGE_RANDOM_TICKS_PER_BLOCK_PER_MINUTE: f32 = 0.87890625;
pub const AVERAGE_RANDOM_TICKS_PER_BLOCK_PER_GAME_DAY: f32 = 17.578125;
pub const WORLD_ICON_SIZE: i32 = 64;

pub const DEBUG_FLAG_NAMES: &[&str] = &[
    "OPEN_INCOMPATIBLE_WORLDS",
    "ALLOW_LOW_SIM_DISTANCE",
    "HOTKEYS",
    "UI_NARRATION",
    "SHUFFLE_UI_RENDERING_ORDER",
    "SHUFFLE_MODELS",
    "RENDER_UI_LAYERING_RECTANGLES",
    "PATHFINDING",
    "SHOW_LOCAL_SERVER_ENTITY_HIT_BOXES",
    "SHAPES",
    "NEIGHBORSUPDATE",
    "EXPERIMENTAL_REDSTONEWIRE_UPDATE_ORDER",
    "STRUCTURES",
    "GAME_EVENT_LISTENERS",
    "DUMP_TEXTURE_ATLAS",
    "STRUCTURE_EDIT_MODE",
    "SAVE_STRUCTURES_AS_SNBT",
    "SYNCHRONOUS_GL_LOGS",
    "VERBOSE_SERVER_EVENTS",
    "NAMED_RUNNABLES",
    "GOAL_SELECTOR",
    "VILLAGE_SECTIONS",
    "BRAIN",
    "POI",
    "BEES",
    "RAIDS",
    "BLOCK_BREAK",
    "MONITOR_TICK_TIMES",
    "KEEP_JIGSAW_BLOCKS_DURING_STRUCTURE_GEN",
    "DONT_SAVE_WORLD",
    "LARGE_DRIPSTONE",
    "CARVERS",
    "ORE_VEINS",
    "SCULK_CATALYST",
    "BYPASS_REALMS_VERSION_CHECK",
    "SOCIAL_INTERACTIONS",
    "CHAT_DISABLED",
    "VALIDATE_RESOURCE_PATH_CASE",
    "UNLOCK_ALL_TRADES",
    "BREEZE_MOB",
    "TRIAL_SPAWNER_DETECTS_SHEEP_AS_PLAYERS",
    "VAULT_DETECTS_SHEEP_AS_PLAYERS",
    "FORCE_ONBOARDING_SCREEN",
    "CURSOR_POS",
    "DEFAULT_SKIN_OVERRIDE",
    "PANORAMA_SCREENSHOT",
    "CHASE_COMMAND",
    "VERBOSE_COMMAND_ERRORS",
    "DEV_COMMANDS",
    "ACTIVE_TEXT_AREAS",
    "PREFER_WAYLAND",
    "IGNORE_LOCAL_MOB_CAP",
    "DISABLE_LIQUID_SPREADING",
    "AQUIFERS",
    "JFR_PROFILING_ENABLE_LEVEL_LOADING",
    "ENTITY_BLOCK_INTERSECTION",
    "GENERATE_SQUARE_TERRAIN_WITHOUT_NOISE",
    "ONLY_GENERATE_HALF_THE_WORLD",
    "DISABLE_FLUID_GENERATION",
    "DISABLE_AQUIFERS",
    "DISABLE_SURFACE",
    "DISABLE_CARVERS",
    "DISABLE_STRUCTURES",
    "DISABLE_FEATURES",
    "DISABLE_ORE_VEINS",
    "DISABLE_BLENDING",
    "DISABLE_BELOW_ZERO_RETROGENERATION",
    "SUBTITLES",
    "COMMAND_STACK_TRACES",
    "WORLD_RECREATE",
    "SHOW_SERVER_DEBUG_VALUES",
    "FEATURE_COUNT",
    "FORCE_TELEMETRY",
    "DONT_SEND_TELEMETRY_TO_BACKEND",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NettyLeakDetectionLevel {
    Disabled,
}

pub const NETTY_LEAK_DETECTION: NettyLeakDetectionLevel = NettyLeakDetectionLevel::Disabled;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedConstantsRuntime {
    pub debug_enabled: bool,
    pub debug_print_properties: bool,
    pub debug_flags: BTreeMap<&'static str, bool>,
    pub debug_fake_latency_ms: i32,
    pub debug_fake_jitter_ms: i32,
    pub check_data_fixer_schema: bool,
    pub is_running_in_ide: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedConstantsInitError {
    pub message: String,
}

impl SharedConstantsRuntime {
    pub fn from_system_properties(
        properties: &BTreeMap<String, String>,
    ) -> Result<Self, SharedConstantsInitError> {
        let debug_enabled = boolean_property(properties, &prefix_debug_flag_name("ENABLED"));
        let debug_print_properties =
            boolean_property(properties, &prefix_debug_flag_name("PRINT_PROPERTIES"));
        let mut debug_flags = BTreeMap::new();
        for name in DEBUG_FLAG_NAMES {
            debug_flags.insert(*name, debug_flag(properties, debug_enabled, name));
        }

        Ok(Self {
            debug_enabled,
            debug_print_properties,
            debug_flags,
            debug_fake_latency_ms: debug_int_value(properties, debug_enabled, "FAKE_LATENCY_MS")?,
            debug_fake_jitter_ms: debug_int_value(properties, debug_enabled, "FAKE_JITTER_MS")?,
            check_data_fixer_schema: true,
            is_running_in_ide: false,
        })
    }

    pub fn debug_generate_square_terrain_without_noise(&self) -> bool {
        self.debug_flag("GENERATE_SQUARE_TERRAIN_WITHOUT_NOISE")
    }

    pub fn debug_only_generate_half_the_world(&self) -> bool {
        self.debug_flag("ONLY_GENERATE_HALF_THE_WORLD")
    }

    pub fn command_stack_traces(&self) -> bool {
        self.debug_flag("COMMAND_STACK_TRACES")
    }

    pub fn debug_flag(&self, name: &str) -> bool {
        self.debug_flags.get(name).copied().unwrap_or(false)
    }
}

pub fn prefix_debug_flag_name(name: &str) -> String {
    format!("{DEBUG_FLAG_PREFIX}{name}")
}

pub fn boolean_property(properties: &BTreeMap<String, String>, name: &str) -> bool {
    properties
        .get(name)
        .is_some_and(|value| value.is_empty() || value.eq_ignore_ascii_case("true"))
}

pub fn debug_flag(properties: &BTreeMap<String, String>, debug_enabled: bool, name: &str) -> bool {
    debug_enabled && boolean_property(properties, &prefix_debug_flag_name(name))
}

pub fn debug_int_value(
    properties: &BTreeMap<String, String>,
    debug_enabled: bool,
    name: &str,
) -> Result<i32, SharedConstantsInitError> {
    if !debug_enabled {
        return Ok(0);
    }

    properties
        .get(&prefix_debug_flag_name(name))
        .map_or(Ok(0), |value| {
            value
                .parse::<i32>()
                .map_err(|error| SharedConstantsInitError {
                    message: error.to_string(),
                })
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPosModel {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosModel {
    pub fn min_block_x(self) -> i32 {
        self.x << 4
    }

    pub fn min_block_z(self) -> i32 {
        self.z << 4
    }
}

pub fn get_protocol_version() -> i32 {
    RELEASE_NETWORK_PROTOCOL_VERSION
}

pub fn debug_void_terrain(pos: ChunkPosModel, runtime: &SharedConstantsRuntime) -> bool {
    let pos_x = pos.min_block_x();
    let pos_z = pos.min_block_z();
    if runtime.debug_only_generate_half_the_world() {
        pos_z < 0
    } else {
        runtime.debug_generate_square_terrain_without_noise()
            && (!(0..=8192).contains(&pos_x) || !(0..=1024).contains(&pos_z))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedConstantsVersionState {
    current_version_token: Option<usize>,
    current_version: Option<WorldVersionModel>,
}

impl SharedConstantsVersionState {
    pub fn new() -> Self {
        Self {
            current_version_token: None,
            current_version: None,
        }
    }

    pub fn set_version(
        &mut self,
        token: usize,
        version: WorldVersionModel,
    ) -> Result<(), &'static str> {
        match self.current_version_token {
            None => {
                self.current_version_token = Some(token);
                self.current_version = Some(version);
                Ok(())
            }
            Some(existing) if existing == token => Ok(()),
            Some(_) => Err("Cannot override the current game version!"),
        }
    }

    pub fn try_detect_version(&mut self, detected: WorldVersionModel) {
        if self.current_version.is_none() {
            self.current_version_token = Some(usize::MAX);
            self.current_version = Some(detected);
        }
    }

    pub fn get_current_version(&self) -> Result<&WorldVersionModel, &'static str> {
        self.current_version.as_ref().ok_or("Game version not set")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedConstantsStaticInitAction {
    SetResourceLeakDetectorLevel(NettyLeakDetectionLevel),
    SetCommandSyntaxExceptionStackTraces(bool),
    InstallBrigadierExceptions,
}

pub fn static_init_actions(
    runtime: &SharedConstantsRuntime,
) -> Vec<SharedConstantsStaticInitAction> {
    vec![
        SharedConstantsStaticInitAction::SetResourceLeakDetectorLevel(NETTY_LEAK_DETECTION),
        SharedConstantsStaticInitAction::SetCommandSyntaxExceptionStackTraces(
            runtime.command_stack_traces(),
        ),
        SharedConstantsStaticInitAction::InstallBrigadierExceptions,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_version::WorldVersionModel;

    fn properties(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn shared_constants_version_and_format_constants_match_java() {
        assert_eq!(WORLD_VERSION, 4790);
        assert_eq!(SERIES, "main");
        assert_eq!(RELEASE_NETWORK_PROTOCOL_VERSION, 775);
        assert_eq!(SNAPSHOT_NETWORK_PROTOCOL_VERSION, 307);
        assert_eq!(SNBT_NAG_VERSION, 4763);
        assert_eq!(SNAPSHOT_PROTOCOL_BIT, 30);
        assert_eq!(RESOURCE_PACK_FORMAT_MAJOR, 84);
        assert_eq!(RESOURCE_PACK_FORMAT_MINOR, 0);
        assert_eq!(DATA_PACK_FORMAT_MAJOR, 101);
        assert_eq!(DATA_PACK_FORMAT_MINOR, 1);
        assert_eq!(RPC_MANAGEMENT_SERVER_API_VERSION, "2.0.0");
        assert_eq!(LANGUAGE_FORMAT, 1);
        assert_eq!(REPORT_FORMAT_VERSION, 1);
        assert_eq!(DATA_VERSION_TAG, "DataVersion");
        assert_eq!(DEBUG_FLAG_PREFIX, "MC_DEBUG_");
    }

    #[test]
    fn shared_constants_boolean_constants_match_java() {
        assert_eq!(
            (
                SNAPSHOT,
                CRASH_EAGERLY,
                FIX_TNT_DUPE,
                FIX_SAND_DUPE,
                USE_DEVONLY
            ),
            (false, false, false, false, false)
        );
    }

    #[test]
    fn shared_constants_limits_and_tick_constants_match_java() {
        assert_eq!(DEFAULT_MINECRAFT_PORT, 25565);
        assert_eq!(MAXIMUM_TICK_TIME_NANOS, 300_000_000);
        assert_eq!(MAXIMUM_BLOCK_EXPLOSION_RESISTANCE, 3_600_000.0);
        assert_eq!(WORLD_RESOLUTION, 16);
        assert_eq!(MAX_CHAT_LENGTH, 256);
        assert_eq!(MAX_USER_INPUT_COMMAND_LENGTH, 32_500);
        assert_eq!(MAX_FUNCTION_COMMAND_LENGTH, 2_000_000);
        assert_eq!(MAX_PLAYER_NAME_LENGTH, 16);
        assert_eq!(MAX_CHAINED_NEIGHBOR_UPDATES, 1_000_000);
        assert_eq!(MAX_RENDER_DISTANCE, 32);
        assert_eq!(MAX_CLOUD_DISTANCE, 128);
        assert_eq!(TICKS_PER_SECOND, 20);
        assert_eq!(MILLIS_PER_TICK, 50);
        assert_eq!(TICKS_PER_MINUTE, 1_200);
        assert_eq!(TICKS_PER_GAME_DAY, 24_000);
        assert_eq!(DEFAULT_RANDOM_TICK_SPEED, 3);
        assert_eq!(AVERAGE_GAME_TICKS_PER_RANDOM_TICK_PER_BLOCK, 1365.3334);
        assert_eq!(AVERAGE_RANDOM_TICKS_PER_BLOCK_PER_MINUTE, 0.87890625);
        assert_eq!(AVERAGE_RANDOM_TICKS_PER_BLOCK_PER_GAME_DAY, 17.578125);
        assert_eq!(WORLD_ICON_SIZE, 64);
    }

    #[test]
    fn illegal_file_characters_match_java_order() {
        assert_eq!(
            ILLEGAL_FILE_CHARACTERS,
            &[
                '/', '\n', '\r', '\t', '\0', '\u{000C}', '`', '?', '*', '\\', '<', '>', '|', '"',
                ':',
            ]
        );
    }

    #[test]
    fn debug_properties_match_java_system_property_rules() {
        let props = properties(&[
            ("MC_DEBUG_ENABLED", ""),
            ("MC_DEBUG_PRINT_PROPERTIES", "true"),
            ("MC_DEBUG_HOTKEYS", "TrUe"),
            ("MC_DEBUG_SHAPES", "false"),
            ("MC_DEBUG_FAKE_LATENCY_MS", "42"),
        ]);
        let runtime =
            SharedConstantsRuntime::from_system_properties(&props).map_err(|err| err.message);

        assert_eq!(
            runtime.map(|runtime| (
                runtime.debug_enabled,
                runtime.debug_print_properties,
                runtime.debug_flag("HOTKEYS"),
                runtime.debug_flag("SHAPES"),
                runtime.debug_flag("OPEN_INCOMPATIBLE_WORLDS"),
                runtime.debug_fake_latency_ms,
                runtime.debug_fake_jitter_ms,
            )),
            Ok((true, true, true, false, false, 42, 0))
        );
    }

    #[test]
    fn debug_flags_are_all_disabled_when_debug_enabled_is_absent() {
        let props = properties(&[("MC_DEBUG_HOTKEYS", ""), ("MC_DEBUG_FAKE_LATENCY_MS", "42")]);
        let runtime =
            SharedConstantsRuntime::from_system_properties(&props).map_err(|err| err.message);

        assert_eq!(
            runtime.map(|runtime| (
                runtime.debug_enabled,
                runtime.debug_flag("HOTKEYS"),
                runtime.debug_fake_latency_ms,
            )),
            Ok((false, false, 0))
        );
    }

    #[test]
    fn debug_int_value_throws_when_enabled_property_is_invalid() {
        let props = properties(&[
            ("MC_DEBUG_ENABLED", ""),
            ("MC_DEBUG_FAKE_LATENCY_MS", "NaN"),
        ]);
        let error =
            SharedConstantsRuntime::from_system_properties(&props).map_err(|err| err.message);

        assert!(error.is_err());
    }

    #[test]
    fn debug_void_terrain_matches_java_branch_order() {
        let mut props = properties(&[
            ("MC_DEBUG_ENABLED", ""),
            ("MC_DEBUG_GENERATE_SQUARE_TERRAIN_WITHOUT_NOISE", ""),
        ]);
        assert_eq!(
            SharedConstantsRuntime::from_system_properties(&props)
                .map_err(|err| err.message)
                .map(|runtime| (
                    debug_void_terrain(ChunkPosModel { x: 513, z: 0 }, &runtime),
                    debug_void_terrain(ChunkPosModel { x: -1, z: 0 }, &runtime),
                    debug_void_terrain(ChunkPosModel { x: 0, z: 65 }, &runtime),
                    debug_void_terrain(ChunkPosModel { x: 512, z: 64 }, &runtime),
                )),
            Ok((true, true, true, false))
        );

        props.insert(
            "MC_DEBUG_ONLY_GENERATE_HALF_THE_WORLD".to_string(),
            String::new(),
        );
        assert_eq!(
            SharedConstantsRuntime::from_system_properties(&props)
                .map_err(|err| err.message)
                .map(|runtime| (
                    debug_void_terrain(ChunkPosModel { x: 600, z: -1 }, &runtime),
                    debug_void_terrain(ChunkPosModel { x: 600, z: 0 }, &runtime),
                )),
            Ok((true, false))
        );
    }

    #[test]
    fn version_state_matches_java_reference_override_rules() {
        let mut state = SharedConstantsVersionState::new();
        assert_eq!(state.get_current_version(), Err("Game version not set"));

        let version = WorldVersionModel::current_26_1_2();
        assert!(state.set_version(1, version.clone()).is_ok());
        assert!(state.set_version(1, version.clone()).is_ok());
        assert_eq!(
            state.set_version(2, version),
            Err("Cannot override the current game version!")
        );
        assert_eq!(
            state
                .get_current_version()
                .map(|version| version.id.as_str()),
            Ok("26.1.2")
        );
    }

    #[test]
    fn try_detect_version_only_sets_missing_current_version() {
        let mut state = SharedConstantsVersionState::new();
        state.try_detect_version(WorldVersionModel::current_26_1_2());
        assert_eq!(
            state
                .get_current_version()
                .map(|version| version.id.as_str()),
            Ok("26.1.2")
        );

        let other = crate::world_version::create_built_in_world_version("other", "Other", true);
        state.try_detect_version(other);
        assert_eq!(
            state
                .get_current_version()
                .map(|version| version.id.as_str()),
            Ok("26.1.2")
        );
    }

    #[test]
    fn static_init_actions_match_java_static_block() {
        let props = properties(&[
            ("MC_DEBUG_ENABLED", ""),
            ("MC_DEBUG_COMMAND_STACK_TRACES", ""),
        ]);
        assert_eq!(
            SharedConstantsRuntime::from_system_properties(&props)
                .map_err(|err| err.message)
                .map(|runtime| static_init_actions(&runtime)),
            Ok(vec![
                SharedConstantsStaticInitAction::SetResourceLeakDetectorLevel(
                    NettyLeakDetectionLevel::Disabled,
                ),
                SharedConstantsStaticInitAction::SetCommandSyntaxExceptionStackTraces(true),
                SharedConstantsStaticInitAction::InstallBrigadierExceptions,
            ])
        );
    }
}
