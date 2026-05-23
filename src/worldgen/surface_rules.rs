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

