use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceRulePresetData {
    pub id: &'static str,
    pub rule: SurfaceRuleKind,
    pub blocks: &'static [&'static str],
    pub conditions: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRuleKind {
    OverworldLike {
        preliminary_surface_check: bool,
        bedrock_roof: bool,
        bedrock_floor: bool,
        deepslate: bool,
    },
    Nether,
    State(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceMaterialContext {
    pub seed: i64,
    pub random_algorithm: RandomAlgorithm,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub biome: &'static str,
    pub stone_depth_above: i32,
    pub stone_depth_below: i32,
    pub surface_depth: i32,
    pub preliminary_surface_y: i32,
    pub water_height: i32,
    pub temperature: f32,
    pub noise: f64,
    pub steep: bool,
    pub hole: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveSurface {
    Floor,
    Ceiling,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceConditionSource {
    Biome(&'static [&'static str]),
    NoiseThreshold {
        min: f64,
        max: f64,
    },
    VerticalGradient {
        random_name: &'static str,
        true_at_and_below: VerticalAnchor,
        false_at_and_above: VerticalAnchor,
    },
    YAbove {
        anchor: VerticalAnchor,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface: CaveSurface,
    },
    Not(&'static SurfaceConditionSource),
    Steep,
    Hole,
    AbovePreliminarySurface,
    Temperature,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceRuleSource {
    Bandlands,
    Block(&'static str),
    Sequence(&'static [SurfaceRuleSource]),
    Condition {
        condition: &'static SurfaceConditionSource,
        rule: &'static SurfaceRuleSource,
    },
}

/// Dynamic (owned) surface rule tree, suitable for JSON-loaded rules.
///
/// Unlike `SurfaceRuleSource` (which uses `&'static` references for hardcoded
/// presets), this type owns its data and can be constructed at runtime from the
/// `surface_rule` field in `worldgen/noise_settings/*.json`.
///
/// Mirrors Java's `SurfaceRules.RuleSource` hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub enum DynSurfaceRule {
    /// Badlands clay-band pattern.  Mirrors `SurfaceRules.Bandlands`.
    Bandlands,
    /// Place a specific block.  Mirrors `SurfaceRules.BlockRuleSource`.
    Block(String),
    /// Try each rule in order; return the first non-null result.
    Sequence(Vec<DynSurfaceRule>),
    /// Apply `rule` only when `condition` is true.
    Condition {
        condition: Box<DynSurfaceCondition>,
        rule: Box<DynSurfaceRule>,
    },
}

/// Dynamic (owned) surface condition, suitable for JSON-loaded rules.
///
/// Mirrors Java's `SurfaceRules.ConditionSource` hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub enum DynSurfaceCondition {
    /// True when the current biome is in the given list.
    Biome(Vec<String>),
    /// True when the specified noise at (x, 0, z) is in [min, max].
    NoiseThreshold {
        noise: String,
        min: f64,
        max: f64,
    },
    /// Probabilistic vertical gradient based on a named positional random.
    VerticalGradient {
        random_name: String,
        true_at_and_below: VerticalAnchor,
        false_at_and_above: VerticalAnchor,
    },
    /// True when `y + stoneDepthAbove (optionally) ≥ anchor + surfaceDepth * multiplier`.
    YAbove {
        anchor: VerticalAnchor,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    /// True when the block is at or above the adjusted water surface.
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    /// True when the stone-column depth is within the surface or ceiling threshold.
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface: CaveSurface,
    },
    Not(Box<DynSurfaceCondition>),
    Steep,
    Hole,
    AbovePreliminarySurface,
    Temperature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceRuleType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceConditionType {
    pub id: &'static str,
}

pub const SURFACE_RULE_TYPES: &[SurfaceRuleType] = &[
    SurfaceRuleType { id: "bandlands" },
    SurfaceRuleType { id: "block" },
    SurfaceRuleType { id: "sequence" },
    SurfaceRuleType { id: "condition" },
];

pub const SURFACE_CONDITION_TYPES: &[SurfaceConditionType] = &[
    SurfaceConditionType { id: "biome" },
    SurfaceConditionType {
        id: "noise_threshold",
    },
    SurfaceConditionType {
        id: "vertical_gradient",
    },
    SurfaceConditionType { id: "y_above" },
    SurfaceConditionType { id: "water" },
    SurfaceConditionType { id: "stone_depth" },
    SurfaceConditionType { id: "not" },
    SurfaceConditionType { id: "steep" },
    SurfaceConditionType { id: "hole" },
    SurfaceConditionType {
        id: "above_preliminary_surface",
    },
    SurfaceConditionType { id: "temperature" },
];

pub const OVERWORLD_SURFACE_BLOCKS: &[&str] = &[
    "minecraft:air",
    "minecraft:bedrock",
    "minecraft:white_terracotta",
    "minecraft:orange_terracotta",
    "minecraft:terracotta",
    "minecraft:red_sand",
    "minecraft:red_sandstone",
    "minecraft:stone",
    "minecraft:deepslate",
    "minecraft:dirt",
    "minecraft:podzol",
    "minecraft:coarse_dirt",
    "minecraft:mycelium",
    "minecraft:grass_block",
    "minecraft:calcite",
    "minecraft:gravel",
    "minecraft:sand",
    "minecraft:sandstone",
    "minecraft:packed_ice",
    "minecraft:snow_block",
    "minecraft:mud",
    "minecraft:powder_snow",
    "minecraft:ice",
    "minecraft:water",
];

pub const NETHER_SURFACE_BLOCKS: &[&str] = &[
    "minecraft:bedrock",
    "minecraft:gravel",
    "minecraft:lava",
    "minecraft:netherrack",
    "minecraft:soul_sand",
    "minecraft:soul_soil",
    "minecraft:basalt",
    "minecraft:blackstone",
    "minecraft:warped_wart_block",
    "minecraft:warped_nylium",
    "minecraft:nether_wart_block",
    "minecraft:crimson_nylium",
];

pub const OVERWORLD_SURFACE_CONDITIONS: &[&str] = &[
    "above_preliminary_surface",
    "bedrock_floor_vertical_gradient",
    "deepslate_vertical_gradient",
    "water",
    "stone_depth",
    "biome",
    "noise_threshold",
    "hole",
    "steep",
    "temperature",
];

pub const NETHER_SURFACE_CONDITIONS: &[&str] = &[
    "bedrock_floor_vertical_gradient",
    "bedrock_roof_vertical_gradient",
    "y_above",
    "hole",
    "noise_threshold",
    "biome",
    "stone_depth",
];

pub const BUILTIN_SURFACE_RULE_PRESETS: &[SurfaceRulePresetData] = &[
    SurfaceRulePresetData {
        id: "minecraft:overworld",
        rule: SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: true,
            bedrock_roof: false,
            bedrock_floor: true,
            deepslate: true,
        },
        blocks: OVERWORLD_SURFACE_BLOCKS,
        conditions: OVERWORLD_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:caves",
        rule: SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: true,
            bedrock_floor: true,
            deepslate: true,
        },
        blocks: OVERWORLD_SURFACE_BLOCKS,
        conditions: OVERWORLD_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:floating_islands",
        rule: SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: false,
            bedrock_floor: false,
            deepslate: true,
        },
        blocks: OVERWORLD_SURFACE_BLOCKS,
        conditions: OVERWORLD_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:nether",
        rule: SurfaceRuleKind::Nether,
        blocks: NETHER_SURFACE_BLOCKS,
        conditions: NETHER_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:end",
        rule: SurfaceRuleKind::State("minecraft:end_stone"),
        blocks: &["minecraft:end_stone"],
        conditions: &[],
    },
    SurfaceRulePresetData {
        id: "minecraft:air",
        rule: SurfaceRuleKind::State("minecraft:air"),
        blocks: &["minecraft:air"],
        conditions: &[],
    },
];
