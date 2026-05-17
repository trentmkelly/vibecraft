#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::item_properties::{item_definition, ItemComponent, ItemDefinition};

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    item_id: &'static str,
    count: i32,
    components: BTreeMap<&'static str, ItemComponent>,
    pop_time: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentPatch {
    Set(ItemComponent),
    Remove(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemStackValidationError {
    CountExceedsMax { count: i32, max: u32 },
    DamageableAndStackable { max_stack_size: u32 },
}

impl ItemStack {
    pub const EMPTY_ITEM: &'static str = "minecraft:air";

    pub fn empty() -> Self {
        Self {
            item_id: Self::EMPTY_ITEM,
            count: 0,
            components: BTreeMap::new(),
            pop_time: 0,
        }
    }

    pub fn new(item_id: &'static str, count: i32) -> Self {
        if item_id == Self::EMPTY_ITEM || count <= 0 {
            return Self::empty();
        }

        let definition = item_definition(item_id).unwrap_or_else(|| ItemDefinition::new(item_id));
        let components = definition
            .components
            .into_iter()
            .map(|component| (component.key(), component))
            .collect();
        Self {
            item_id,
            count,
            components,
            pop_time: 0,
        }
    }

    pub fn item_id(&self) -> &'static str {
        if self.is_empty() {
            Self::EMPTY_ITEM
        } else {
            self.item_id
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item_id == Self::EMPTY_ITEM || self.count <= 0
    }

    pub fn count(&self) -> i32 {
        if self.is_empty() {
            0
        } else {
            self.count
        }
    }

    pub fn set_count(&mut self, count: i32) {
        self.count = count;
    }

    pub fn grow(&mut self, amount: i32) {
        self.set_count(self.count() + amount);
    }

    pub fn shrink(&mut self, amount: i32) {
        self.grow(-amount);
    }

    pub fn limit_size(&mut self, max_stack_size: i32) {
        if !self.is_empty() && self.count > max_stack_size {
            self.count = max_stack_size;
        }
    }

    pub fn split(&mut self, amount: i32) -> Self {
        let real_amount = amount.min(self.count()).max(0);
        let result = self.copy_with_count(real_amount);
        self.shrink(real_amount);
        result
    }

    pub fn copy_and_clear(&mut self) -> Self {
        if self.is_empty() {
            Self::empty()
        } else {
            let result = self.clone();
            self.count = 0;
            result
        }
    }

    pub fn copy_with_count(&self, count: i32) -> Self {
        if self.is_empty() {
            Self::empty()
        } else {
            let mut copy = self.clone();
            copy.count = count;
            copy
        }
    }

    pub fn transmute_copy(&self, new_item: &'static str, new_count: i32) -> Self {
        if self.is_empty() {
            Self::empty()
        } else {
            let mut copy = self.clone();
            copy.item_id = new_item;
            copy.count = new_count;
            copy
        }
    }

    pub fn consume(&mut self, amount: i32, infinite_materials: bool) {
        if !infinite_materials {
            self.shrink(amount);
        }
    }

    pub fn consume_and_return(&mut self, amount: i32, infinite_materials: bool) -> Self {
        let split = self.copy_with_count(amount);
        self.consume(amount, infinite_materials);
        split
    }

    pub fn pop_time(&self) -> i32 {
        self.pop_time
    }

    pub fn set_pop_time(&mut self, pop_time: i32) {
        self.pop_time = pop_time;
    }

    pub fn component(&self, key: &'static str) -> Option<&ItemComponent> {
        if self.is_empty() {
            None
        } else {
            self.components.get(key)
        }
    }

    pub fn set_component(&mut self, component: ItemComponent) -> Option<ItemComponent> {
        self.components.insert(component.key(), component)
    }

    pub fn remove_component(&mut self, key: &'static str) -> Option<ItemComponent> {
        self.components.remove(key)
    }

    pub fn apply_components(&mut self, patch: &[ComponentPatch]) {
        for operation in patch {
            match operation {
                ComponentPatch::Set(component) => {
                    self.set_component(component.clone());
                }
                ComponentPatch::Remove(key) => {
                    self.remove_component(key);
                }
            }
        }
    }

    pub fn apply_components_and_validate(
        &mut self,
        patch: &[ComponentPatch],
    ) -> Result<(), ItemStackValidationError> {
        let old = self.components.clone();
        self.apply_components(patch);
        if let Err(err) = self.validate_strict() {
            self.components = old;
            Err(err)
        } else {
            Ok(())
        }
    }

    pub fn components_patch(&self) -> BTreeMap<&'static str, ItemComponent> {
        if self.is_empty() {
            BTreeMap::new()
        } else {
            self.components.clone()
        }
    }

    pub fn max_stack_size(&self) -> u32 {
        match self.component("minecraft:max_stack_size") {
            Some(ItemComponent::MaxStackSize(max)) => *max,
            _ => 1,
        }
    }

    pub fn max_damage(&self) -> u32 {
        match self.component("minecraft:max_damage") {
            Some(ItemComponent::MaxDamage(max)) => *max,
            _ => 0,
        }
    }

    pub fn is_damageable_item(&self) -> bool {
        self.component("minecraft:max_damage").is_some()
            && self.component("minecraft:damage").is_some()
            && self.component("minecraft:unbreakable").is_none()
    }

    pub fn damage_value(&self) -> u32 {
        match self.component("minecraft:damage") {
            Some(ItemComponent::Damage(value)) => (*value).min(self.max_damage()),
            _ => 0,
        }
    }

    pub fn set_damage_value(&mut self, value: u32) {
        let clamped = value.min(self.max_damage());
        self.set_component(ItemComponent::Damage(clamped));
    }

    pub fn is_damaged(&self) -> bool {
        self.is_damageable_item() && self.damage_value() > 0
    }

    pub fn is_broken(&self) -> bool {
        self.is_damageable_item() && self.damage_value() >= self.max_damage()
    }

    pub fn is_stackable(&self) -> bool {
        self.max_stack_size() > 1 && (!self.is_damageable_item() || !self.is_damaged())
    }

    pub fn validate_strict(&self) -> Result<(), ItemStackValidationError> {
        let max_stack_size = self.max_stack_size();
        if self.component("minecraft:max_damage").is_some() && max_stack_size > 1 {
            return Err(ItemStackValidationError::DamageableAndStackable { max_stack_size });
        }
        if self.count() > max_stack_size as i32 {
            return Err(ItemStackValidationError::CountExceedsMax {
                count: self.count(),
                max: max_stack_size,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_properties::{ItemUseAnimation, Rarity};

    #[test]
    fn empty_stack_rules_match_vanilla_air_or_non_positive_count() {
        assert!(ItemStack::empty().is_empty());
        assert!(ItemStack::new("minecraft:air", 64).is_empty());
        assert!(ItemStack::new("minecraft:stick", 0).is_empty());
        assert_eq!(
            ItemStack::new("minecraft:stick", 3).item_id(),
            "minecraft:stick"
        );
    }

    #[test]
    fn stack_counts_split_copy_clear_and_consume_like_item_stack() {
        let mut stack = ItemStack::new("minecraft:stick", 10);
        let split = stack.split(4);
        assert_eq!(split.count(), 4);
        assert_eq!(stack.count(), 6);

        let copy = stack.copy_with_count(2);
        assert_eq!(copy.count(), 2);
        assert_eq!(stack.count(), 6);

        let cleared = stack.copy_and_clear();
        assert_eq!(cleared.count(), 6);
        assert!(stack.is_empty());

        let mut pearl = ItemStack::new("minecraft:ender_pearl", 8);
        let returned = pearl.consume_and_return(3, false);
        assert_eq!(returned.count(), 3);
        assert_eq!(pearl.count(), 5);
        pearl.consume(4, true);
        assert_eq!(pearl.count(), 5);
    }

    #[test]
    fn item_stack_components_start_from_item_definition() {
        let bow = ItemStack::new("minecraft:bow", 1);
        assert_eq!(bow.max_stack_size(), 1);
        assert_eq!(bow.max_damage(), 384);
        assert_eq!(
            bow.component("minecraft:use_animation"),
            Some(&ItemComponent::UseAnimation(ItemUseAnimation::Bow))
        );

        let ominous = ItemStack::new("minecraft:ominous_bottle", 1);
        assert_eq!(
            ominous.component("minecraft:rarity"),
            Some(&ItemComponent::Rarity(Rarity::Uncommon))
        );
    }

    #[test]
    fn damage_state_clamps_and_controls_stackability() {
        let mut trident = ItemStack::new("minecraft:trident", 1);
        assert!(trident.is_damageable_item());
        assert!(!trident.is_stackable());
        trident.set_damage_value(99);
        assert_eq!(trident.damage_value(), 99);
        assert!(trident.is_damaged());
        assert!(!trident.is_stackable());
        trident.set_damage_value(999);
        assert_eq!(trident.damage_value(), 250);
        assert!(trident.is_broken());
    }

    #[test]
    fn component_patch_applies_sets_removes_and_rolls_back_invalid_changes() {
        let mut apple = ItemStack::new("minecraft:apple", 4);
        apple
            .apply_components_and_validate(&[ComponentPatch::Set(ItemComponent::MaxStackSize(16))])
            .unwrap();
        assert_eq!(apple.max_stack_size(), 16);
        apple.apply_components(&[ComponentPatch::Remove("minecraft:food")]);
        assert!(apple.component("minecraft:food").is_none());

        let err = apple
            .apply_components_and_validate(&[
                ComponentPatch::Set(ItemComponent::MaxDamage(5)),
                ComponentPatch::Set(ItemComponent::Damage(0)),
            ])
            .unwrap_err();
        assert_eq!(
            err,
            ItemStackValidationError::DamageableAndStackable { max_stack_size: 16 }
        );
        assert!(apple.component("minecraft:max_damage").is_none());
    }

    #[test]
    fn strict_validation_rejects_oversized_stacks() {
        let stack = ItemStack::new("minecraft:ender_pearl", 17);
        assert_eq!(
            stack.validate_strict(),
            Err(ItemStackValidationError::CountExceedsMax { count: 17, max: 16 })
        );
    }

    #[test]
    fn transmute_preserves_component_patch_and_changes_item_identity() {
        let potion = ItemStack::new("minecraft:potion", 1);
        let bottle = potion.transmute_copy("minecraft:glass_bottle", 1);
        assert_eq!(bottle.item_id(), "minecraft:glass_bottle");
        assert!(bottle.component("minecraft:potion_contents").is_some());
        assert_eq!(potion.item_id(), "minecraft:potion");
    }
}
