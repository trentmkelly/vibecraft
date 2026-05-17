#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemUseAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Spear,
    Crossbow,
    Spyglass,
    Brush,
    Bundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Head,
    Chest,
    Legs,
    Feet,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TooltipBehavior {
    Normal,
    Hidden,
    WithComponent(&'static str),
    GlintOverride(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemComponent {
    MaxStackSize(u32),
    MaxDamage(u32),
    Damage(u32),
    Rarity(Rarity),
    Repairable(&'static str),
    UseAnimation(ItemUseAnimation),
    UseCooldown(f32),
    Food {
        nutrition: i32,
        saturation: f32,
        can_always_eat: bool,
    },
    Equippable {
        slot: EquipmentSlot,
        swappable: bool,
        damage_on_hurt: bool,
    },
    Enchantable(u32),
    Tooltip(TooltipBehavior),
    UseRemainder(&'static str),
    ItemModel(&'static str),
    ItemName(&'static str),
    Custom(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemDefinition {
    pub registry_id: &'static str,
    pub components: Vec<ItemComponent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveItemProperties {
    pub registry_id: &'static str,
    pub max_stack_size: u32,
    pub rarity: Rarity,
    pub durability: Option<u32>,
    pub repairable: Option<&'static str>,
    pub use_animation: ItemUseAnimation,
    pub cooldown_seconds: Option<f32>,
    pub food: Option<(i32, f32, bool)>,
    pub equipment_slot: Option<EquipmentSlot>,
    pub equippable_swappable: bool,
    pub enchantability: Option<u32>,
    pub tooltip: TooltipBehavior,
    pub components: BTreeMap<&'static str, ItemComponent>,
}

impl ItemDefinition {
    pub fn new(registry_id: &'static str) -> Self {
        Self {
            registry_id,
            components: vec![
                ItemComponent::MaxStackSize(64),
                ItemComponent::Rarity(Rarity::Common),
                ItemComponent::UseAnimation(ItemUseAnimation::None),
                ItemComponent::Tooltip(TooltipBehavior::Normal),
                ItemComponent::ItemName(registry_id),
                ItemComponent::ItemModel(registry_id),
            ],
        }
    }

    pub fn stacks_to(mut self, max: u32) -> Self {
        self.set(ItemComponent::MaxStackSize(max));
        self
    }

    pub fn durability(mut self, max_damage: u32) -> Self {
        self.set(ItemComponent::MaxDamage(max_damage));
        self.set(ItemComponent::Damage(0));
        self.set(ItemComponent::MaxStackSize(1));
        self
    }

    pub fn rarity(mut self, rarity: Rarity) -> Self {
        self.set(ItemComponent::Rarity(rarity));
        self
    }

    pub fn repairable(mut self, ingredient: &'static str) -> Self {
        self.set(ItemComponent::Repairable(ingredient));
        self
    }

    pub fn use_animation(mut self, animation: ItemUseAnimation) -> Self {
        self.set(ItemComponent::UseAnimation(animation));
        self
    }

    pub fn use_cooldown(mut self, seconds: f32) -> Self {
        self.set(ItemComponent::UseCooldown(seconds));
        self
    }

    pub fn food(mut self, nutrition: i32, saturation: f32, can_always_eat: bool) -> Self {
        self.set(ItemComponent::Food {
            nutrition,
            saturation,
            can_always_eat,
        });
        self.set(ItemComponent::UseAnimation(ItemUseAnimation::Eat));
        self
    }

    pub fn drink(mut self) -> Self {
        self.set(ItemComponent::UseAnimation(ItemUseAnimation::Drink));
        self
    }

    pub fn equippable(
        mut self,
        slot: EquipmentSlot,
        swappable: bool,
        damage_on_hurt: bool,
    ) -> Self {
        self.set(ItemComponent::Equippable {
            slot,
            swappable,
            damage_on_hurt,
        });
        self
    }

    pub fn enchantable(mut self, value: u32) -> Self {
        self.set(ItemComponent::Enchantable(value));
        self
    }

    pub fn tooltip(mut self, tooltip: TooltipBehavior) -> Self {
        self.set(ItemComponent::Tooltip(tooltip));
        self
    }

    pub fn use_remainder(mut self, item: &'static str) -> Self {
        self.set(ItemComponent::UseRemainder(item));
        self
    }

    pub fn custom(mut self, name: &'static str) -> Self {
        self.set(ItemComponent::Custom(name));
        self
    }

    pub fn effective(&self) -> EffectiveItemProperties {
        let mut effective = EffectiveItemProperties {
            registry_id: self.registry_id,
            max_stack_size: 64,
            rarity: Rarity::Common,
            durability: None,
            repairable: None,
            use_animation: ItemUseAnimation::None,
            cooldown_seconds: None,
            food: None,
            equipment_slot: None,
            equippable_swappable: true,
            enchantability: None,
            tooltip: TooltipBehavior::Normal,
            components: BTreeMap::new(),
        };

        for component in &self.components {
            effective
                .components
                .insert(component.key(), component.clone());
            match component {
                ItemComponent::MaxStackSize(max) => effective.max_stack_size = *max,
                ItemComponent::MaxDamage(max) => effective.durability = Some(*max),
                ItemComponent::Rarity(rarity) => effective.rarity = *rarity,
                ItemComponent::Repairable(ingredient) => effective.repairable = Some(*ingredient),
                ItemComponent::UseAnimation(animation) => effective.use_animation = *animation,
                ItemComponent::UseCooldown(seconds) => effective.cooldown_seconds = Some(*seconds),
                ItemComponent::Food {
                    nutrition,
                    saturation,
                    can_always_eat,
                } => effective.food = Some((*nutrition, *saturation, *can_always_eat)),
                ItemComponent::Equippable {
                    slot, swappable, ..
                } => {
                    effective.equipment_slot = Some(*slot);
                    effective.equippable_swappable = *swappable;
                }
                ItemComponent::Enchantable(value) => effective.enchantability = Some(*value),
                ItemComponent::Tooltip(tooltip) => effective.tooltip = tooltip.clone(),
                ItemComponent::Damage(_)
                | ItemComponent::UseRemainder(_)
                | ItemComponent::ItemModel(_)
                | ItemComponent::ItemName(_)
                | ItemComponent::Custom(_) => {}
            }
        }

        effective
    }

    fn set(&mut self, component: ItemComponent) {
        let key = component.key();
        self.components.retain(|existing| existing.key() != key);
        self.components.push(component);
    }
}

impl ItemComponent {
    pub fn key(&self) -> &'static str {
        match self {
            Self::MaxStackSize(_) => "minecraft:max_stack_size",
            Self::MaxDamage(_) => "minecraft:max_damage",
            Self::Damage(_) => "minecraft:damage",
            Self::Rarity(_) => "minecraft:rarity",
            Self::Repairable(_) => "minecraft:repairable",
            Self::UseAnimation(_) => "minecraft:use_animation",
            Self::UseCooldown(_) => "minecraft:use_cooldown",
            Self::Food { .. } => "minecraft:food",
            Self::Equippable { .. } => "minecraft:equippable",
            Self::Enchantable(_) => "minecraft:enchantable",
            Self::Tooltip(_) => "minecraft:tooltip_display",
            Self::UseRemainder(_) => "minecraft:use_remainder",
            Self::ItemModel(_) => "minecraft:item_model",
            Self::ItemName(_) => "minecraft:item_name",
            Self::Custom(name) => name,
        }
    }
}

pub fn representative_item_definitions() -> Vec<ItemDefinition> {
    vec![
        ItemDefinition::new("minecraft:stick"),
        ItemDefinition::new("minecraft:apple").food(4, 2.4, false),
        ItemDefinition::new("minecraft:bow")
            .durability(384)
            .enchantable(1)
            .use_animation(ItemUseAnimation::Bow),
        ItemDefinition::new("minecraft:crossbow")
            .stacks_to(1)
            .durability(465)
            .enchantable(1)
            .use_animation(ItemUseAnimation::Crossbow)
            .custom("minecraft:charged_projectiles"),
        ItemDefinition::new("minecraft:diamond_sword")
            .durability(1561)
            .repairable("#minecraft:diamond_tool_materials")
            .enchantable(10)
            .custom("minecraft:attribute_modifiers")
            .custom("minecraft:weapon"),
        ItemDefinition::new("minecraft:diamond_helmet")
            .durability(363)
            .repairable("#minecraft:repairs_diamond_armor")
            .enchantable(10)
            .equippable(EquipmentSlot::Head, true, true)
            .custom("minecraft:attribute_modifiers"),
        ItemDefinition::new("minecraft:elytra")
            .durability(432)
            .rarity(Rarity::Epic)
            .repairable("minecraft:phantom_membrane")
            .equippable(EquipmentSlot::Chest, true, false)
            .custom("minecraft:glider"),
        ItemDefinition::new("minecraft:saddle")
            .stacks_to(1)
            .equippable(EquipmentSlot::Body, true, false),
        ItemDefinition::new("minecraft:ender_pearl")
            .stacks_to(16)
            .use_cooldown(1.0),
        ItemDefinition::new("minecraft:potion")
            .stacks_to(1)
            .drink()
            .use_remainder("minecraft:glass_bottle")
            .custom("minecraft:potion_contents"),
        ItemDefinition::new("minecraft:written_book")
            .stacks_to(16)
            .tooltip(TooltipBehavior::GlintOverride(true)),
        ItemDefinition::new("minecraft:mace")
            .rarity(Rarity::Epic)
            .durability(500)
            .repairable("minecraft:breeze_rod")
            .enchantable(15)
            .custom("minecraft:tool")
            .custom("minecraft:weapon"),
        ItemDefinition::new("minecraft:shield")
            .durability(336)
            .repairable("#minecraft:wooden_tool_materials")
            .equippable(EquipmentSlot::OffHand, false, true)
            .use_animation(ItemUseAnimation::Block)
            .custom("minecraft:banner_patterns"),
        ItemDefinition::new("minecraft:trident")
            .rarity(Rarity::Rare)
            .durability(250)
            .enchantable(1)
            .use_animation(ItemUseAnimation::Spear)
            .custom("minecraft:tool")
            .custom("minecraft:weapon"),
        ItemDefinition::new("minecraft:brush")
            .durability(64)
            .use_animation(ItemUseAnimation::Brush),
        ItemDefinition::new("minecraft:bundle")
            .stacks_to(1)
            .use_animation(ItemUseAnimation::Bundle)
            .custom("minecraft:bundle_contents"),
        ItemDefinition::new("minecraft:ominous_bottle")
            .rarity(Rarity::Uncommon)
            .drink()
            .custom("minecraft:ominous_bottle_amplifier"),
        ItemDefinition::new("minecraft:trial_key"),
    ]
}

pub fn item_definition(registry_id: &str) -> Option<ItemDefinition> {
    representative_item_definitions()
        .into_iter()
        .find(|definition| definition.registry_id == registry_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durability_sets_damage_zero_and_unstackable_like_item_properties() {
        let props = ItemDefinition::new("minecraft:bow")
            .durability(384)
            .enchantable(1)
            .effective();
        assert_eq!(props.max_stack_size, 1);
        assert_eq!(props.durability, Some(384));
        assert_eq!(props.enchantability, Some(1));
        assert!(props.components.contains_key("minecraft:damage"));
    }

    #[test]
    fn representative_registry_ids_cover_core_component_shapes() {
        let ids: Vec<_> = representative_item_definitions()
            .into_iter()
            .map(|definition| definition.registry_id)
            .collect();
        assert!(ids.contains(&"minecraft:stick"));
        assert!(ids.contains(&"minecraft:diamond_sword"));
        assert!(ids.contains(&"minecraft:elytra"));
        assert!(ids.contains(&"minecraft:ominous_bottle"));
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]) == false);
    }

    #[test]
    fn food_cooldown_and_use_remainder_components_resolve_effective_behavior() {
        let apple = item_definition("minecraft:apple").unwrap().effective();
        assert_eq!(apple.food, Some((4, 2.4, false)));
        assert_eq!(apple.use_animation, ItemUseAnimation::Eat);

        let pearl = item_definition("minecraft:ender_pearl")
            .unwrap()
            .effective();
        assert_eq!(pearl.max_stack_size, 16);
        assert_eq!(pearl.cooldown_seconds, Some(1.0));

        let potion = item_definition("minecraft:potion").unwrap().effective();
        assert_eq!(potion.max_stack_size, 1);
        assert_eq!(potion.use_animation, ItemUseAnimation::Drink);
        assert!(potion.components.contains_key("minecraft:use_remainder"));
    }

    #[test]
    fn equipment_slots_swappability_and_damage_on_hurt_match_representative_items() {
        let helmet = item_definition("minecraft:diamond_helmet")
            .unwrap()
            .effective();
        assert_eq!(helmet.equipment_slot, Some(EquipmentSlot::Head));
        assert!(helmet.equippable_swappable);
        assert_eq!(helmet.repairable, Some("#minecraft:repairs_diamond_armor"));

        let shield = item_definition("minecraft:shield").unwrap().effective();
        assert_eq!(shield.equipment_slot, Some(EquipmentSlot::OffHand));
        assert!(!shield.equippable_swappable);

        let elytra = item_definition("minecraft:elytra").unwrap().effective();
        assert_eq!(elytra.equipment_slot, Some(EquipmentSlot::Chest));
        assert_eq!(elytra.rarity, Rarity::Epic);
        assert!(elytra.components.contains_key("minecraft:glider"));
    }

    #[test]
    fn rarity_enchantability_repairability_and_tooltip_components_are_exposed() {
        let mace = item_definition("minecraft:mace").unwrap().effective();
        assert_eq!(mace.rarity, Rarity::Epic);
        assert_eq!(mace.durability, Some(500));
        assert_eq!(mace.repairable, Some("minecraft:breeze_rod"));
        assert_eq!(mace.enchantability, Some(15));

        let book = item_definition("minecraft:written_book")
            .unwrap()
            .effective();
        assert_eq!(book.max_stack_size, 16);
        assert_eq!(book.tooltip, TooltipBehavior::GlintOverride(true));
    }

    #[test]
    fn vanilla_component_keys_are_stable() {
        assert_eq!(
            ItemComponent::MaxStackSize(16).key(),
            "minecraft:max_stack_size"
        );
        assert_eq!(ItemComponent::MaxDamage(250).key(), "minecraft:max_damage");
        assert_eq!(
            ItemComponent::Equippable {
                slot: EquipmentSlot::Chest,
                swappable: true,
                damage_on_hurt: false
            }
            .key(),
            "minecraft:equippable"
        );
    }
}
