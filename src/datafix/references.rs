//! Type references, the Rust port of `net.minecraft.util.datafix.fixes.References`.
//!
//! In Java a `TypeReference` is a named handle (`DSL.TypeReference`) that a
//! [`Schema`](super::schema::Schema) maps to a type template. Here it is the
//! reference id string (for example `"item_stack"`), which is what identifies
//! the reference in Java (`TypeReference.typeName()`).

/// Handle of a data type known to the DataFixer (Java `DSL.TypeReference`).
pub type TypeReference = &'static str;

pub const LEVEL: TypeReference = "level";
pub const LIGHTWEIGHT_LEVEL: TypeReference = "lightweight_level";
pub const PLAYER: TypeReference = "player";
pub const CHUNK: TypeReference = "chunk";
pub const HOTBAR: TypeReference = "hotbar";
pub const OPTIONS: TypeReference = "options";
pub const STRUCTURE: TypeReference = "structure";
pub const STATS: TypeReference = "stats";
pub const SAVED_DATA_COMMAND_STORAGE: TypeReference = "saved_data/command_storage";
pub const SAVED_DATA_CUSTOM_BOSS_EVENTS: TypeReference = "saved_data/custom_boss_events";
pub const SAVED_DATA_ENDER_DRAGON_FIGHT: TypeReference = "saved_data/ender_dragon_fight";
pub const SAVED_DATA_GAME_RULES: TypeReference = "saved_data/game_rules";
pub const SAVED_DATA_TICKETS: TypeReference = "saved_data/tickets";
pub const SAVED_DATA_MAP_DATA: TypeReference = "saved_data/map_data";
pub const SAVED_DATA_MAP_INDEX: TypeReference = "saved_data/idcounts";
pub const SAVED_DATA_RAIDS: TypeReference = "saved_data/raids";
pub const SAVED_DATA_RANDOM_SEQUENCES: TypeReference = "saved_data/random_sequences";
pub const SAVED_DATA_SCHEDULED_EVENTS: TypeReference = "saved_data/scheduled_events";
pub const SAVED_DATA_SCOREBOARD: TypeReference = "saved_data/scoreboard";
pub const SAVED_DATA_STOPWATCHES: TypeReference = "saved_data/stopwatches";
pub const SAVED_DATA_STRUCTURE_FEATURE_INDICES: TypeReference =
    "saved_data/structure_feature_indices";
pub const SAVED_DATA_WANDERING_TRADER: TypeReference = "saved_data/wandering_trader";
pub const SAVED_DATA_WEATHER: TypeReference = "saved_data/weather";
pub const SAVED_DATA_WORLD_BORDER: TypeReference = "saved_data/world_border";
pub const SAVED_DATA_WORLD_CLOCKS: TypeReference = "saved_data/world_clocks";
pub const SAVED_DATA_WORLD_GEN_SETTINGS: TypeReference = "saved_data/world_gen_settings";
pub const ADVANCEMENTS: TypeReference = "advancements";
pub const POI_CHUNK: TypeReference = "poi_chunk";
pub const ENTITY_CHUNK: TypeReference = "entity_chunk";
pub const DEBUG_PROFILE: TypeReference = "debug_profile";
pub const BLOCK_ENTITY: TypeReference = "block_entity";
pub const ITEM_STACK: TypeReference = "item_stack";
pub const BLOCK_STATE: TypeReference = "block_state";
pub const FLAT_BLOCK_STATE: TypeReference = "flat_block_state";
pub const DATA_COMPONENTS: TypeReference = "data_components";
pub const VILLAGER_TRADE: TypeReference = "villager_trade";
pub const PARTICLE: TypeReference = "particle";
pub const TEXT_COMPONENT: TypeReference = "text_component";
pub const ENTITY_EQUIPMENT: TypeReference = "entity_equipment";
pub const ENTITY_NAME: TypeReference = "entity_name";
pub const ENTITY_TREE: TypeReference = "entity_tree";
pub const ENTITY: TypeReference = "entity";
pub const BLOCK_NAME: TypeReference = "block_name";
pub const ITEM_NAME: TypeReference = "item_name";
pub const GAME_EVENT_NAME: TypeReference = "game_event_name";
pub const UNTAGGED_SPAWNER: TypeReference = "untagged_spawner";
pub const STRUCTURE_FEATURE: TypeReference = "structure_feature";
pub const OBJECTIVE: TypeReference = "objective";
pub const TEAM: TypeReference = "team";
pub const RECIPE: TypeReference = "recipe";
pub const BIOME: TypeReference = "biome";
pub const MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST: TypeReference =
    "multi_noise_biome_source_parameter_list";
pub const WORLD_GEN_SETTINGS: TypeReference = "world_gen_settings";

/// Every reference paired with its Java `References` field name, used by fixture
/// tooling that addresses references the way the Java sources do.
pub const JAVA_FIELD_NAMES: &[(&str, TypeReference)] = &[
    ("LEVEL", LEVEL),
    ("LIGHTWEIGHT_LEVEL", LIGHTWEIGHT_LEVEL),
    ("PLAYER", PLAYER),
    ("CHUNK", CHUNK),
    ("HOTBAR", HOTBAR),
    ("OPTIONS", OPTIONS),
    ("STRUCTURE", STRUCTURE),
    ("STATS", STATS),
    ("SAVED_DATA_COMMAND_STORAGE", SAVED_DATA_COMMAND_STORAGE),
    (
        "SAVED_DATA_CUSTOM_BOSS_EVENTS",
        SAVED_DATA_CUSTOM_BOSS_EVENTS,
    ),
    (
        "SAVED_DATA_ENDER_DRAGON_FIGHT",
        SAVED_DATA_ENDER_DRAGON_FIGHT,
    ),
    ("SAVED_DATA_GAME_RULES", SAVED_DATA_GAME_RULES),
    ("SAVED_DATA_TICKETS", SAVED_DATA_TICKETS),
    ("SAVED_DATA_MAP_DATA", SAVED_DATA_MAP_DATA),
    ("SAVED_DATA_MAP_INDEX", SAVED_DATA_MAP_INDEX),
    ("SAVED_DATA_RAIDS", SAVED_DATA_RAIDS),
    ("SAVED_DATA_RANDOM_SEQUENCES", SAVED_DATA_RANDOM_SEQUENCES),
    ("SAVED_DATA_SCHEDULED_EVENTS", SAVED_DATA_SCHEDULED_EVENTS),
    ("SAVED_DATA_SCOREBOARD", SAVED_DATA_SCOREBOARD),
    ("SAVED_DATA_STOPWATCHES", SAVED_DATA_STOPWATCHES),
    (
        "SAVED_DATA_STRUCTURE_FEATURE_INDICES",
        SAVED_DATA_STRUCTURE_FEATURE_INDICES,
    ),
    ("SAVED_DATA_WANDERING_TRADER", SAVED_DATA_WANDERING_TRADER),
    ("SAVED_DATA_WEATHER", SAVED_DATA_WEATHER),
    ("SAVED_DATA_WORLD_BORDER", SAVED_DATA_WORLD_BORDER),
    ("SAVED_DATA_WORLD_CLOCKS", SAVED_DATA_WORLD_CLOCKS),
    (
        "SAVED_DATA_WORLD_GEN_SETTINGS",
        SAVED_DATA_WORLD_GEN_SETTINGS,
    ),
    ("ADVANCEMENTS", ADVANCEMENTS),
    ("POI_CHUNK", POI_CHUNK),
    ("ENTITY_CHUNK", ENTITY_CHUNK),
    ("DEBUG_PROFILE", DEBUG_PROFILE),
    ("BLOCK_ENTITY", BLOCK_ENTITY),
    ("ITEM_STACK", ITEM_STACK),
    ("BLOCK_STATE", BLOCK_STATE),
    ("FLAT_BLOCK_STATE", FLAT_BLOCK_STATE),
    ("DATA_COMPONENTS", DATA_COMPONENTS),
    ("VILLAGER_TRADE", VILLAGER_TRADE),
    ("PARTICLE", PARTICLE),
    ("TEXT_COMPONENT", TEXT_COMPONENT),
    ("ENTITY_EQUIPMENT", ENTITY_EQUIPMENT),
    ("ENTITY_NAME", ENTITY_NAME),
    ("ENTITY_TREE", ENTITY_TREE),
    ("ENTITY", ENTITY),
    ("BLOCK_NAME", BLOCK_NAME),
    ("ITEM_NAME", ITEM_NAME),
    ("GAME_EVENT_NAME", GAME_EVENT_NAME),
    ("UNTAGGED_SPAWNER", UNTAGGED_SPAWNER),
    ("STRUCTURE_FEATURE", STRUCTURE_FEATURE),
    ("OBJECTIVE", OBJECTIVE),
    ("TEAM", TEAM),
    ("RECIPE", RECIPE),
    ("BIOME", BIOME),
    (
        "MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST",
        MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST,
    ),
    ("WORLD_GEN_SETTINGS", WORLD_GEN_SETTINGS),
];

/// Resolves a Java `References` field name (for example `ITEM_STACK`) to its reference.
pub fn by_java_field_name(name: &str) -> Option<TypeReference> {
    JAVA_FIELD_NAMES
        .iter()
        .find(|(field, _)| *field == name)
        .map(|(_, reference)| *reference)
}
