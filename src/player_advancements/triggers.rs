//! Condition matching for the criterion triggers the live server can raise.
//!
//! Java decodes each criterion's `conditions` into a `CriterionTriggerInstance`; here the
//! raw JSON is matched directly for the triggers whose events exist server-side:
//!
//! * `minecraft:tick` / `minecraft:impossible` (`PlayerTrigger`, `ImpossibleTrigger`),
//! * `minecraft:recipe_unlocked` (`RecipeUnlockedTrigger.TriggerInstance.matches`),
//! * `minecraft:inventory_changed` (`InventoryChangeTrigger.TriggerInstance.matches`).
//!
//! Conditions that need a predicate this module cannot evaluate (a `player` entity
//! predicate, item `components`/`predicates`, ...) never match instead of matching
//! wrongly: the criterion simply stays unobtained.
//! TODO(advancement-predicates): decode `ContextAwarePredicate`, `ItemPredicate`
//! component matchers and the `location`/`entity`/`damage` predicates so the triggers
//! that need them (`location`, `player_killed_entity`, `placed_block`, ...) can fire.

use serde_json::Value;

use crate::advancement_system::CriterionSpec;
use crate::item_stack::ItemStack;
use crate::player_inventory::PlayerInventory;
use crate::registry::Identifier;

/// Registry lookups a condition needs beyond the event itself.
pub trait TriggerEnvironment {
    /// Whether `item` (`minecraft:stone`) is in the item tag `tag` (`minecraft:planks`).
    fn item_in_tag(&self, item: &str, tag: &str) -> bool;
}

/// Tag membership from the built-in tag tables ([`crate::item_tags`]).
#[derive(Debug, Clone, Copy, Default)]
pub struct BuiltinItemTags;

impl TriggerEnvironment for BuiltinItemTags {
    fn item_in_tag(&self, item: &str, tag: &str) -> bool {
        crate::item_tags::item_in_tag(item, tag)
    }
}

/// `SimpleCriterionTrigger.SimpleInstance.player()`: only an absent (or empty) player
/// predicate is known to pass.
fn player_predicate_passes(conditions: &Value) -> bool {
    match conditions.get("player") {
        None => true,
        Some(Value::Array(predicates)) => predicates.is_empty(),
        Some(_) => false,
    }
}

/// `PlayerTrigger.trigger(player)`: `t -> true` once the player predicate passes.
pub fn player_trigger_matches(spec: &CriterionSpec) -> bool {
    player_predicate_passes(&spec.conditions)
}

/// `RecipeUnlockedTrigger.TriggerInstance.matches`: `recipe.id() == this.recipe`.
pub fn recipe_unlocked_matches(spec: &CriterionSpec, recipe: &Identifier) -> bool {
    player_predicate_passes(&spec.conditions)
        && spec
            .conditions
            .get("recipe")
            .and_then(Value::as_str)
            .and_then(|id| Identifier::parse(id).ok())
            .is_some_and(|expected| expected == *recipe)
}

/// `InventoryChangeTrigger.trigger` + `TriggerInstance.matches`.
pub fn inventory_changed_matches(
    spec: &CriterionSpec,
    inventory: &PlayerInventory,
    changed: &ItemStack,
    environment: &dyn TriggerEnvironment,
) -> bool {
    if !player_predicate_passes(&spec.conditions) {
        return false;
    }
    let counts = SlotCounts::of(inventory);
    let Some(slots_match) = slots_match(spec.conditions.get("slots"), counts) else {
        return false;
    };
    if !slots_match {
        return false;
    }
    let predicates = match spec.conditions.get("items") {
        None => &[][..],
        Some(Value::Array(items)) => items.as_slice(),
        Some(_) => return false,
    };
    match predicates {
        [] => true,
        [single] => {
            !changed.is_empty()
                && item_predicate_test(single, changed, environment) == Some(true)
        }
        _ => all_predicates_found(predicates, inventory, environment),
    }
}

/// The `items.size() != 1` branch: every predicate must match some non-empty slot.
fn all_predicates_found(
    predicates: &[Value],
    inventory: &PlayerInventory,
    environment: &dyn TriggerEnvironment,
) -> bool {
    let mut remaining: Vec<&Value> = predicates.iter().collect();
    for slot in 0..inventory.container_size() {
        if remaining.is_empty() {
            return true;
        }
        let stack = inventory.get(slot);
        if !stack.is_empty() {
            remaining.retain(|predicate| {
                item_predicate_test(predicate, stack, environment) != Some(true)
            });
        }
    }
    remaining.is_empty()
}

/// Slot occupancy counts computed by `InventoryChangeTrigger.trigger`.
#[derive(Clone, Copy)]
struct SlotCounts {
    full: i32,
    empty: i32,
    occupied: i32,
}

impl SlotCounts {
    fn of(inventory: &PlayerInventory) -> Self {
        let mut counts = Self {
            full: 0,
            empty: 0,
            occupied: 0,
        };
        for slot in 0..inventory.container_size() {
            let stack = inventory.get(slot);
            if stack.is_empty() {
                counts.empty += 1;
            } else {
                counts.occupied += 1;
                if stack.count() >= stack.max_stack_size() as i32 {
                    counts.full += 1;
                }
            }
        }
        counts
    }
}

/// `Slots.matches`; `None` when the JSON is not a valid `Slots` object.
fn slots_match(slots: Option<&Value>, counts: SlotCounts) -> Option<bool> {
    let Some(slots) = slots else {
        return Some(true);
    };
    let slots = slots.as_object()?;
    let bound = |key: &str, value: i32| match slots.get(key) {
        None => Some(true),
        Some(bounds) => IntBounds::parse(bounds).map(|bounds| bounds.matches(value)),
    };
    Some(bound("full", counts.full)? && bound("empty", counts.empty)? && bound("occupied", counts.occupied)?)
}

/// `MinMaxBounds.Ints`: a bare number is an exact match, an object has `min`/`max`.
struct IntBounds {
    min: Option<i64>,
    max: Option<i64>,
}

impl IntBounds {
    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::Number(number) => {
                let exact = number.as_i64()?;
                Some(Self {
                    min: Some(exact),
                    max: Some(exact),
                })
            }
            Value::Object(object) => {
                let read = |key: &str| match object.get(key) {
                    None => Some(None),
                    Some(bound) => bound.as_i64().map(Some),
                };
                Some(Self {
                    min: read("min")?,
                    max: read("max")?,
                })
            }
            _ => None,
        }
    }

    fn matches(&self, value: i32) -> bool {
        let value = i64::from(value);
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
}

/// `ItemPredicate.test(stack)` for the `items` and `count` fields. `None` means the
/// predicate uses a field this module cannot evaluate (`components`, `predicates`).
pub(super) fn item_predicate_test(
    predicate: &Value,
    stack: &ItemStack,
    environment: &dyn TriggerEnvironment,
) -> Option<bool> {
    let predicate = predicate.as_object()?;
    for unsupported in ["components", "predicates"] {
        match predicate.get(unsupported) {
            None => {}
            Some(Value::Object(fields)) if fields.is_empty() => {}
            Some(_) => return None,
        }
    }
    if let Some(items) = predicate.get("items") {
        if !item_set_contains(items, stack.item_id(), environment)? {
            return Some(false);
        }
    }
    if let Some(count) = predicate.get("count") {
        if !IntBounds::parse(count)?.matches(stack.count()) {
            return Some(false);
        }
    }
    Some(true)
}

/// `RegistryCodecs.homogeneousList` for items: one id, a `#tag`, or a list of ids.
fn item_set_contains(
    items: &Value,
    item: &str,
    environment: &dyn TriggerEnvironment,
) -> Option<bool> {
    match items {
        Value::String(entry) => Some(entry_matches(entry, item, environment)),
        Value::Array(entries) => {
            let mut found = false;
            for entry in entries {
                found |= entry_matches(entry.as_str()?, item, environment);
            }
            Some(found)
        }
        _ => None,
    }
}

fn entry_matches(entry: &str, item: &str, environment: &dyn TriggerEnvironment) -> bool {
    match entry.strip_prefix('#') {
        Some(tag) => environment.item_in_tag(item, tag),
        None => entry == item,
    }
}
