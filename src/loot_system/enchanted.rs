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
    /// `RegistryCodecs.homogeneousList`: an id, a `#tag` or a list of ids.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, String> {
        let identifier = |text: &str| {
            crate::registry::Identifier::parse(text)
                .map(|id| id.to_string())
                .map_err(|err| format!("{text}: {err}"))
        };
        match value {
            serde_json::Value::String(text) => match text.strip_prefix('#') {
                Some(tag) => Ok(Self::Tag(identifier(tag)?)),
                None => Ok(Self::Ids(vec![identifier(text)?])),
            },
            serde_json::Value::Array(ids) => ids
                .iter()
                .map(|id| {
                    id.as_str()
                        .ok_or_else(|| format!("not an id: {id}"))
                        .and_then(identifier)
                })
                .collect::<Result<_, _>>()
                .map(Self::Ids),
            other => Err(format!("not an id, tag or list: {other}")),
        }
    }

    /// Whether `id` (an entry of `registry`, e.g. `"item"` or `"enchantment"`) is in
    /// the set; tags resolve through [`LootContext::tag_members`].
    pub fn contains_in(&self, registry: &str, id: &str, context: &LootContext) -> bool {
        match self {
            Self::Ids(ids) => ids.iter().any(|candidate| candidate == id),
            Self::Tag(tag) => context
                .tag_members(registry, tag)
                .is_some_and(|members| members.iter().any(|member| member == id)),
        }
    }

    /// `HolderSet.stream()`: the members in set order (tag order for a tag). An
    /// unknown tag is empty.
    pub fn members(&self, registry: &str, context: &LootContext) -> Vec<String> {
        match self {
            Self::Ids(ids) => ids.clone(),
            Self::Tag(tag) => context
                .tag_members(registry, tag)
                .map(<[String]>::to_vec)
                .unwrap_or_default(),
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
                .is_some_and(|tool| items.contains_in("item", tool, context))
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
                    .is_none_or(|set| set.contains_in("enchantment", id, context))
                    && bound.min_level.is_none_or(|min| *level >= min)
                    && bound.max_level.is_none_or(|max| *level <= max)
            })
        })
    }
}

impl ToolPredicate {
    /// `ItemPredicate.test(stack)`: the stack's item against `items` and its
    /// `minecraft:enchantments` component against the enchantment requirements.
    pub fn matches_stack(&self, stack: &LootStack, context: &LootContext) -> bool {
        if let Some(items) = &self.items {
            if !items.contains_in("item", &stack.item, context) {
                return false;
            }
        }
        let held = stack.item_enchantments(super::stack_components::ENCHANTMENTS_COMPONENT);
        self.enchantments.iter().all(|bound| {
            held.iter().any(|(id, level)| {
                bound
                    .enchantments
                    .as_ref()
                    .is_none_or(|set| set.contains_in("enchantment", id, context))
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
