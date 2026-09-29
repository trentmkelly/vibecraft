//! Enchantment-aware loot parts: `LevelBasedValue`, the `match_tool` item predicate
//! subset, and the context's view of the tool/attacker enchantment levels.
//!
//! Java reads enchantment levels from the `TOOL` item stack (`table_bonus`,
//! `apply_bonus`, `match_tool`) or from the attacking entity
//! (`enchanted_count_increase`, `random_chance_with_enchanted_bonus`). The loot
//! request carries the levels that matter to vanilla tables as scalars
//! (`silk_touch`, `fortune_level`, `looting_level`), and everything else through
//! [`LootContext::enchantment_levels`]; [`LootContext::enchantment_level_of`] unifies
//! them behind one lookup.

use super::*;

/// `LevelBasedValue`: a number computed from an enchantment level.
#[derive(Debug, Clone, PartialEq)]
pub enum LevelBasedValue {
    Constant(f32),
    Clamped {
        value: Box<LevelBasedValue>,
        min: f32,
        max: f32,
    },
    Fraction {
        numerator: Box<LevelBasedValue>,
        denominator: Box<LevelBasedValue>,
    },
    LevelsSquared(f32),
    Linear {
        base: f32,
        per_level_above_first: f32,
    },
    Exponent {
        base: Box<LevelBasedValue>,
        power: Box<LevelBasedValue>,
    },
    Lookup {
        values: Vec<f32>,
        fallback: Box<LevelBasedValue>,
    },
}

impl LevelBasedValue {
    /// `LevelBasedValue.calculate`.
    pub fn calculate(&self, level: i32) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Clamped { value, min, max } => value.calculate(level).clamp(*min, *max),
            Self::Fraction {
                numerator,
                denominator,
            } => {
                let denominator = denominator.calculate(level);
                if denominator == 0.0 {
                    0.0
                } else {
                    numerator.calculate(level) / denominator
                }
            }
            Self::LevelsSquared(added) => (level * level) as f32 + added,
            Self::Linear {
                base,
                per_level_above_first,
            } => base + per_level_above_first * (level - 1) as f32,
            Self::Exponent { base, power } => {
                (base.calculate(level) as f64).powf(power.calculate(level) as f64) as f32
            }
            Self::Lookup { values, fallback } => match usize::try_from(level - 1) {
                Ok(index) if index < values.len() => values[index],
                _ => fallback.calculate(level),
            },
        }
    }
}

/// A registry `HolderSet`: explicit ids or a tag.
#[derive(Debug, Clone, PartialEq)]
pub enum HolderSet {
    Ids(Vec<String>),
    Tag(String),
}

impl HolderSet {
    /// Whether `id` is in the set; tags resolve through the context's tag map.
    pub fn contains(&self, id: &str, context: &LootContext) -> bool {
        match self {
            Self::Ids(ids) => ids.iter().any(|candidate| candidate == id),
            Self::Tag(tag) => context
                .tags
                .get(tag)
                .is_some_and(|members| members.iter().any(|member| member == id)),
        }
    }
}

/// One `EnchantmentPredicate` of an `ItemEnchantmentsPredicate`: some enchantment of
/// the stack must be in `enchantments` (any when absent) with a level in range.
#[derive(Debug, Clone, PartialEq)]
pub struct EnchantmentBound {
    pub enchantments: Option<HolderSet>,
    pub min_level: Option<i32>,
    pub max_level: Option<i32>,
}

/// The `ItemPredicate` subset `match_tool` decodes: an item set and enchantment
/// requirements (`predicates: {"minecraft:enchantments": [...]}`).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolPredicate {
    pub items: Option<HolderSet>,
    pub enchantments: Vec<EnchantmentBound>,
}

impl ToolPredicate {
    /// `MatchTool.test`: the context's tool against the predicate.
    pub fn matches(&self, context: &LootContext) -> bool {
        if let Some(items) = &self.items {
            if !context
                .tool
                .as_deref()
                .is_some_and(|tool| items.contains(tool, context))
            {
                return false;
            }
        }
        let held = context.tool_enchantments();
        self.enchantments.iter().all(|bound| {
            held.iter().any(|(id, level)| {
                bound
                    .enchantments
                    .as_ref()
                    .is_none_or(|set| set.contains(id, context))
                    && bound.min_level.is_none_or(|min| *level >= min)
                    && bound.max_level.is_none_or(|max| *level <= max)
            })
        })
    }
}

impl LootContext {
    /// The tool's present enchantments: the request-level scalars plus
    /// [`Self::enchantment_levels`].
    pub fn tool_enchantments(&self) -> Vec<(String, i32)> {
        let mut held: Vec<(String, i32)> = self
            .enchantment_levels
            .iter()
            .map(|(id, level)| (id.clone(), *level))
            .collect();
        let scalars = [
            (
                "minecraft:silk_touch",
                i32::from(
                    self.entity_properties
                        .get("silk_touch")
                        .is_some_and(|v| v == "true"),
                ),
            ),
            ("minecraft:fortune", self.fortune_level),
            ("minecraft:looting", self.looting_level),
        ];
        held.extend(
            scalars
                .into_iter()
                .filter(|(_, level)| *level > 0)
                .map(|(id, level)| (id.to_string(), level)),
        );
        held
    }

    /// `EnchantmentHelper.getItemEnchantmentLevel` / `getEnchantmentLevel`: the level
    /// of `enchantment` on the tool or attacker, 0 when absent.
    pub fn enchantment_level_of(&self, enchantment: &str) -> i32 {
        self.tool_enchantments()
            .iter()
            .filter(|(id, _)| id == enchantment)
            .map(|(_, level)| *level)
            .max()
            .unwrap_or(0)
    }
}
