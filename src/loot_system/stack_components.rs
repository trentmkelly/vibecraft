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

use super::reference_data::item_loot_properties;
use super::LootStack;

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
        item_loot_properties(&self.item).map_or(64, |properties| properties.max_stack_size)
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
