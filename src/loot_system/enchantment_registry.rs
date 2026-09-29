//! The data-driven enchantment registry loot functions select from.
//!
//! Java's `Enchantment` records come from `data/<ns>/enchantment/*.json` (a data pack
//! registry). `enchant_with_levels` and `enchant_randomly` read the definition fields
//! `supported_items`, `primary_items`, `weight`, `max_level`, `min_cost`, `max_cost` and
//! `exclusive_set`, so [`EnchantmentRegistry`] loads exactly those from the pack; item
//! and enchantment tags resolve through the loot context's
//! [`RegistryTags`](super::RegistryTags).

use std::collections::BTreeMap;

use serde_json::Value;

use super::{HolderSet, LootContext};
use crate::enchantment_system::EnchantmentCandidate;
use crate::registry_pipeline::resources::{FileToIdConverter, ResourceManager};

/// `Enchantment.Cost`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantmentCost {
    pub base: i32,
    pub per_level_above_first: i32,
}

impl EnchantmentCost {
    /// `Enchantment.Cost.calculate`.
    pub fn calculate(self, level: i32) -> i32 {
        self.base + self.per_level_above_first * (level - 1)
    }

    fn from_json(value: &Value) -> Result<Self, String> {
        let field = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_i64)
                .and_then(|number| i32::try_from(number).ok())
                .ok_or_else(|| format!("cost is missing integer field {key}"))
        };
        Ok(Self {
            base: field("base")?,
            per_level_above_first: field("per_level_above_first")?,
        })
    }
}

/// The parts of an `Enchantment` record enchantment selection reads.
#[derive(Debug, Clone, PartialEq)]
pub struct EnchantmentDefinition {
    pub id: String,
    /// `EnchantmentDefinition.supportedItems`.
    pub supported_items: HolderSet,
    /// `EnchantmentDefinition.primaryItems`.
    pub primary_items: Option<HolderSet>,
    pub weight: i32,
    pub max_level: i32,
    pub min_cost: EnchantmentCost,
    pub max_cost: EnchantmentCost,
    /// `Enchantment.exclusiveSet` (empty when absent).
    pub exclusive_set: Option<HolderSet>,
}

impl EnchantmentDefinition {
    fn from_json(id: &str, json: &Value) -> Result<Self, String> {
        let set = |key: &str| json.get(key).map(HolderSet::from_json).transpose();
        let int = |key: &str| {
            json.get(key)
                .and_then(Value::as_i64)
                .and_then(|number| i32::try_from(number).ok())
                .ok_or_else(|| format!("{id}: missing integer field {key}"))
        };
        let cost = |key: &str| {
            EnchantmentCost::from_json(
                json.get(key)
                    .ok_or_else(|| format!("{id}: missing field {key}"))?,
            )
        };
        Ok(Self {
            id: id.to_string(),
            supported_items: set("supported_items")?
                .ok_or_else(|| format!("{id}: missing field supported_items"))?,
            primary_items: set("primary_items")?,
            weight: int("weight")?,
            max_level: int("max_level")?,
            min_cost: cost("min_cost")?,
            max_cost: cost("max_cost")?,
            exclusive_set: set("exclusive_set")?,
        })
    }

    /// `Enchantment.isSupportedItem` / `canEnchant`: the item is in `supported_items`.
    pub fn is_supported_item(&self, item: &str, context: &LootContext) -> bool {
        self.supported_items.contains_in("item", item, context)
    }

    /// `Enchantment.isPrimaryItem`.
    pub fn is_primary_item(&self, item: &str, context: &LootContext) -> bool {
        self.is_supported_item(item, context)
            && self
                .primary_items
                .as_ref()
                .is_none_or(|primary| primary.contains_in("item", item, context))
    }
}

/// The loaded `enchantment` registry, iterated in id order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnchantmentRegistry {
    definitions: BTreeMap<String, EnchantmentDefinition>,
}

impl EnchantmentRegistry {
    /// Loads every `enchantment/*.json` of `manager` (the highest-priority copy per id).
    /// Documents that fail to decode are reported in `problems` and skipped, like Java's
    /// registry loader.
    pub fn load(manager: &ResourceManager, problems: &mut Vec<String>) -> Self {
        let converter = FileToIdConverter::json("enchantment");
        let mut registry = Self::default();
        for (location, resource) in manager.list_matching_resources(&converter) {
            let Ok(id) = converter.file_to_id(&location) else {
                continue;
            };
            let decoded = resource
                .read_to_string()
                .map_err(|err| err.to_string())
                .and_then(|raw| serde_json::from_str::<Value>(&raw).map_err(|err| err.to_string()))
                .and_then(|json| EnchantmentDefinition::from_json(&id.to_string(), &json));
            match decoded {
                Ok(definition) => {
                    registry.definitions.insert(definition.id.clone(), definition);
                }
                Err(err) => problems.push(format!("Couldn't parse enchantment {id}: {err}")),
            }
        }
        registry
    }

    /// The definition of `id`.
    pub fn get(&self, id: &str) -> Option<&EnchantmentDefinition> {
        self.definitions.get(id)
    }

    /// Every registered id in registry (sorted) order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.definitions.keys().map(String::as_str)
    }

    /// `Enchantment.areCompatible`: distinct, and neither's exclusive set holds the other.
    pub fn are_compatible(&self, first: &str, second: &str, context: &LootContext) -> bool {
        let excludes = |owner: &str, other: &str| {
            self.get(owner)
                .and_then(|definition| definition.exclusive_set.as_ref())
                .is_some_and(|set| set.contains_in("enchantment", other, context))
        };
        first != second && !excludes(first, second) && !excludes(second, first)
    }

    /// `EnchantmentHelper.getAvailableEnchantmentResults`: each enchantment of `source`
    /// that is a primary item for `item` (or `item` is a plain book), at the highest
    /// level whose `[minCost, maxCost]` range contains `value`.
    pub fn available_results(
        &self,
        value: i32,
        item: &str,
        source: &[String],
        context: &LootContext,
    ) -> Vec<EnchantmentCandidate> {
        let is_book = item == "minecraft:book";
        let mut results = Vec::new();
        for id in source {
            let Some(definition) = self.get(id) else {
                continue;
            };
            if !(is_book || definition.is_primary_item(item, context)) {
                continue;
            }
            // `getMinLevel()` is 1.
            for level in (1..=definition.max_level).rev() {
                if value >= definition.min_cost.calculate(level)
                    && value <= definition.max_cost.calculate(level)
                {
                    results.push(EnchantmentCandidate {
                        id: id.clone(),
                        level,
                        weight: definition.weight,
                    });
                    break;
                }
            }
        }
        results
    }
}
