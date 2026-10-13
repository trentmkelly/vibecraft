//! Typed access to the component values loot functions edit on a [`LootStack`].
//!
//! [`LootStack::components`] maps a component id to a canonical text form:
//!
//! * `minecraft:enchantments` / `minecraft:stored_enchantments` (`ItemEnchantments`):
//!   `id:level` pairs joined by `,`, sorted by enchantment id;
//! * `minecraft:damage`: the decimal damage value;
//! * `minecraft:block_state` (`BlockItemStateProperties`): `name=value` pairs joined
//!   by `,`, sorted by property name.
//!
//! The helpers here read and write those forms with the semantics of the Java
//! `ItemStack` / `EnchantmentHelper` methods the loot functions call.

use std::collections::BTreeMap;

use serde_json::{Map, Number, Value};

use super::reference_data::item_loot_properties;
use super::LootStack;

/// Sparse `ItemContainerContents` slots or ordered `ChargedProjectiles` stacks.
/// Empty slots are retained because `ContainerComponentManipulator.modifyItems`
/// transforms every slot in place before rebuilding the component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentsSlots(pub Vec<Option<LootStack>>);

impl ContentsSlots {
    /// Decodes the canonical JSON stored for a container component.
    pub fn decode(text: &str, component: super::item_functions::ContentsComponent) -> Self {
        let Ok(Value::Array(values)) = serde_json::from_str::<Value>(text) else {
            return Self(Vec::new());
        };
        match component {
            super::item_functions::ContentsComponent::Container => {
                let max_slot = values
                    .iter()
                    .filter_map(|value| value.get("slot").and_then(Value::as_u64))
                    .filter(|slot| *slot < 256)
                    .max();
                let Some(max_slot) = max_slot else {
                    return Self(Vec::new());
                };
                let mut slots = vec![None; max_slot as usize + 1];
                for value in values {
                    let Some(slot) = value.get("slot").and_then(Value::as_u64) else {
                        continue;
                    };
                    if slot >= 256 {
                        continue;
                    }
                    if let Some(stack) = value.get("item").and_then(decode_stack_value) {
                        slots[slot as usize] = Some(stack);
                    }
                }
                Self(slots)
            }
            super::item_functions::ContentsComponent::ChargedProjectiles => Self(
                values
                    .iter()
                    .filter_map(decode_stack_value)
                    .map(Some)
                    .collect(),
            ),
        }
    }

    /// Encodes the component's vanilla JSON codec shape.
    pub fn encode(
        slots: &[Option<LootStack>],
        component: super::item_functions::ContentsComponent,
    ) -> String {
        let value = match component {
            super::item_functions::ContentsComponent::Container => {
                let end = slots
                    .iter()
                    .rposition(|stack| stack.as_ref().is_some_and(|stack| !stack.is_empty()))
                    .map_or(0, |index| index + 1);
                Value::Array(
                    slots[..end]
                        .iter()
                        .enumerate()
                        .filter_map(|(slot, stack)| {
                            stack.as_ref().filter(|stack| !stack.is_empty()).map(|stack| {
                                serde_json::json!({"slot": slot, "item": encode_stack_value(stack)})
                            })
                        })
                        .collect(),
                )
            }
            super::item_functions::ContentsComponent::ChargedProjectiles => Value::Array(
                slots
                    .iter()
                    .filter_map(Option::as_ref)
                    .filter(|stack| !stack.is_empty())
                    .map(encode_stack_value)
                    .collect(),
            ),
        };
        value.to_string()
    }
}

fn decode_stack_value(value: &Value) -> Option<LootStack> {
    let item = value.get("id")?.as_str()?;
    let item = crate::registry::Identifier::parse(item).ok()?.to_string();
    let count = value.get("count").and_then(Value::as_i64).unwrap_or(1) as i32;
    let mut stack = LootStack::new(item, count);
    if let Some(Value::Object(components)) = value.get("components") {
        stack.components = components
            .iter()
            .filter_map(|(id, value)| {
                let id = crate::registry::Identifier::parse(id).ok()?.to_string();
                Some((id.clone(), decode_component_value(&id, value)))
            })
            .collect();
    }
    Some(stack)
}

fn encode_stack_value(stack: &LootStack) -> Value {
    let mut value = Map::new();
    value.insert("id".to_string(), Value::String(stack.item.clone()));
    if stack.count != 1 {
        value.insert(
            "count".to_string(),
            Value::Number(Number::from(stack.count)),
        );
    }
    if !stack.components.is_empty() {
        let components = stack
            .components
            .iter()
            .map(|(id, text)| {
                let value = encode_component_value(id, text);
                (id.clone(), value)
            })
            .collect();
        value.insert("components".to_string(), Value::Object(components));
    }
    Value::Object(value)
}

/// `DataComponents.ENCHANTMENTS`.
pub const ENCHANTMENTS_COMPONENT: &str = "minecraft:enchantments";
/// `DataComponents.STORED_ENCHANTMENTS`.
pub const STORED_ENCHANTMENTS_COMPONENT: &str = "minecraft:stored_enchantments";
/// `DataComponents.DAMAGE`.
pub const DAMAGE_COMPONENT: &str = "minecraft:damage";
/// `DataComponents.BLOCK_STATE`.
pub const BLOCK_STATE_COMPONENT: &str = "minecraft:block_state";

/// `Items.BOOK`.
pub const BOOK: &str = "minecraft:book";
/// `Items.ENCHANTED_BOOK`.
pub const ENCHANTED_BOOK: &str = "minecraft:enchanted_book";

/// `ItemEnchantments.Mutable.set/upgrade` caps levels at 255.
const MAX_STORED_ENCHANTMENT_LEVEL: i32 = 255;

/// Parses the canonical `ItemEnchantments` text.
fn parse_enchantments(text: &str) -> BTreeMap<String, i32> {
    text.split(',')
        .filter_map(|entry| {
            let (id, level) = entry.rsplit_once(':')?;
            Some((id.to_string(), level.parse().ok()?))
        })
        .collect()
}

/// Formats `ItemEnchantments` in the canonical text form.
fn format_enchantments(enchantments: &BTreeMap<String, i32>) -> String {
    enchantments
        .iter()
        .map(|(id, level)| format!("{id}:{level}"))
        .collect::<Vec<_>>()
        .join(",")
}

impl LootStack {
    /// `EnchantmentHelper.getComponentType`: enchanted books store their enchantments
    /// in `stored_enchantments`, every other item in `enchantments`.
    pub fn enchantments_component_key(&self) -> &'static str {
        if self.item == ENCHANTED_BOOK {
            STORED_ENCHANTMENTS_COMPONENT
        } else {
            ENCHANTMENTS_COMPONENT
        }
    }

    /// Whether the stack holds `component`: set on the stack, or present in the item's
    /// default components (`ItemStack.has`). Items the registry does not know behave
    /// like plain items with the common default components.
    fn holds_enchantment_component(&self, component: &str) -> bool {
        if self.components.contains_key(component) {
            return true;
        }
        match item_loot_properties(&self.item) {
            Some(properties) if component == STORED_ENCHANTMENTS_COMPONENT => {
                properties.can_hold_stored_enchantments
            }
            Some(properties) => properties.can_hold_enchantments,
            None => component == ENCHANTMENTS_COMPONENT,
        }
    }

    /// The stack's `ItemEnchantments` for `component`, empty when unset.
    pub fn item_enchantments(&self, component: &str) -> BTreeMap<String, i32> {
        self.components
            .get(component)
            .map(|text| parse_enchantments(text))
            .unwrap_or_default()
    }

    /// `EnchantmentHelper.updateEnchantments`: runs `update` on a mutable copy of the
    /// enchantments and stores the result. Like Java it does nothing when the stack
    /// cannot hold the component.
    pub fn update_enchantments(&mut self, update: impl FnOnce(&mut BTreeMap<String, i32>)) {
        let component = self.enchantments_component_key();
        if !self.holds_enchantment_component(component) {
            return;
        }
        let mut enchantments = self.item_enchantments(component);
        update(&mut enchantments);
        self.components
            .insert(component.to_string(), format_enchantments(&enchantments));
    }

    /// `ItemStack.enchant`: `ItemEnchantments.Mutable.upgrade` (keeps the higher level).
    pub fn enchant(&mut self, enchantment: &str, level: i32) {
        self.update_enchantments(|enchantments| {
            upgrade_enchantment(enchantments, enchantment, level);
        });
    }

    /// `ItemStack.getMaxDamage`: 0 for items without `max_damage`.
    pub fn max_damage(&self) -> i32 {
        item_loot_properties(&self.item)
            .and_then(|properties| properties.max_damage)
            .unwrap_or(0)
    }

    /// `ItemStack.isDamageableItem`: has `max_damage` and `damage` and is not
    /// `unbreakable`.
    pub fn is_damageable_item(&self) -> bool {
        self.max_damage() > 0 && !self.components.contains_key("minecraft:unbreakable")
    }

    /// `ItemStack.getDamageValue`: the `damage` component clamped to `[0, max_damage]`.
    pub fn damage_value(&self) -> i32 {
        let stored = self
            .components
            .get(DAMAGE_COMPONENT)
            .and_then(|text| text.parse::<i32>().ok())
            .unwrap_or(0);
        stored.clamp(0, self.max_damage())
    }

    /// `ItemStack.setDamageValue`: stores the value clamped to `[0, max_damage]`.
    pub fn set_damage_value(&mut self, damage: i32) {
        let clamped = damage.clamp(0, self.max_damage());
        self.components
            .insert(DAMAGE_COMPONENT.to_string(), clamped.to_string());
    }

    /// `ItemStack.getMaxStackSize` (`MAX_STACK_SIZE`, defaulting to 1 like Java's
    /// `getOrDefault`); unknown items use the common default of 64.
    pub fn max_stack_size(&self) -> i32 {
        self.components
            .get("minecraft:max_stack_size")
            .and_then(|value| value.parse::<i32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or_else(|| {
                item_loot_properties(&self.item).map_or(64, |properties| properties.max_stack_size)
            })
    }
}

/// `ItemEnchantments.Mutable.upgrade`.
pub fn upgrade_enchantment(
    enchantments: &mut BTreeMap<String, i32>,
    enchantment: &str,
    level: i32,
) {
    if level > 0 {
        let level = level.min(MAX_STORED_ENCHANTMENT_LEVEL);
        let entry = enchantments.entry(enchantment.to_string()).or_insert(level);
        *entry = (*entry).max(level);
    }
}

/// `ItemEnchantments.Mutable.set`: a level of 0 or less removes the enchantment.
pub fn set_enchantment(enchantments: &mut BTreeMap<String, i32>, enchantment: &str, level: i32) {
    if level <= 0 {
        enchantments.remove(enchantment);
    } else {
        enchantments.insert(
            enchantment.to_string(),
            level.min(MAX_STORED_ENCHANTMENT_LEVEL),
        );
    }
}

/// Convert Java component JSON into the canonical forms used by typed item
/// mutation helpers. Generic string/object components keep their existing form.
fn decode_component_value(id: &str, value: &Value) -> String {
    if matches!(id, ENCHANTMENTS_COMPONENT | STORED_ENCHANTMENTS_COMPONENT) {
        if let Some(entries) = value.as_object() {
            let enchantments = entries
                .iter()
                .filter_map(|(id, level)| {
                    let id = crate::registry::Identifier::parse(id).ok()?.to_string();
                    Some((id, i32::try_from(level.as_i64()?).ok()?))
                })
                .collect();
            return format_enchantments(&enchantments);
        }
    } else if id == BLOCK_STATE_COMPONENT {
        if let Some(entries) = value.as_object() {
            return entries
                .iter()
                .filter_map(|(key, value)| Some(format!("{key}={}", value.as_str()?)))
                .collect::<Vec<_>>()
                .join(",");
        }
    }
    super::world_view::canonical_component_value(id, value)
}

/// Inverse of the canonical forms edited by `ItemEnchantments` and
/// `BlockItemStateProperties` helpers. Their Java codecs require maps, even
/// though the Rust component carrier stores sorted compact text internally.
fn encode_component_value(id: &str, text: &str) -> Value {
    if matches!(id, ENCHANTMENTS_COMPONENT | STORED_ENCHANTMENTS_COMPONENT) {
        // `set_components` may already carry the Java JSON form.
        if let Ok(Value::Object(entries)) = serde_json::from_str(text) {
            return Value::Object(entries);
        }
        return Value::Object(
            parse_enchantments(text)
                .into_iter()
                .map(|(id, level)| (id, Value::Number(Number::from(level))))
                .collect(),
        );
    }
    if id == BLOCK_STATE_COMPONENT {
        if let Ok(Value::Object(entries)) = serde_json::from_str(text) {
            return Value::Object(entries);
        }
        let entries = text
            .split(',')
            .filter_map(|entry| {
                let (key, value) = entry.split_once('=')?;
                Some((key.to_string(), Value::String(value.to_string())))
            })
            .collect();
        return Value::Object(entries);
    }
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_string()))
}
