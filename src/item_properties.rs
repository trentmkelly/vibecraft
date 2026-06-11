#![allow(dead_code)]

use crate::item_consumable_components::UseEffectsComponent;
use crate::item_custom_model_data_component::CustomModelData;
use crate::item_instrument_component::InstrumentComponent;
use crate::item_lore_component::ItemLoreComponent;
use crate::item_swing_animation_component::SwingAnimationComponent;
use crate::item_tooltip_components::TooltipDisplayComponent;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl Rarity {
    pub const VALUES: [Self; 4] = [Self::Common, Self::Uncommon, Self::Rare, Self::Epic];

    pub fn id(self) -> i32 {
        match self {
            Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::Epic => 3,
        }
    }

    pub fn by_id(id: i32) -> Self {
        Self::VALUES
            .iter()
            .copied()
            .find(|rarity| rarity.id() == id)
            .unwrap_or(Self::Common)
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Epic => "epic",
        }
    }

    pub fn by_serialized_name(name: &str) -> Option<Self> {
        Self::VALUES
            .iter()
            .copied()
            .find(|rarity| rarity.serialized_name() == name)
    }

    pub fn color(self) -> crate::chat_formatting::ChatFormatting {
        match self {
            Self::Common => crate::chat_formatting::ChatFormatting::White,
            Self::Uncommon => crate::chat_formatting::ChatFormatting::Yellow,
            Self::Rare => crate::chat_formatting::ChatFormatting::Aqua,
            Self::Epic => crate::chat_formatting::ChatFormatting::LightPurple,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDisplayContext {
    None,
    ThirdPersonLeftHand,
    ThirdPersonRightHand,
    FirstPersonLeftHand,
    FirstPersonRightHand,
    Head,
    Gui,
    Ground,
    Fixed,
    OnShelf,
}

impl ItemDisplayContext {
    pub const VALUES: [Self; 10] = [
        Self::None,
        Self::ThirdPersonLeftHand,
        Self::ThirdPersonRightHand,
        Self::FirstPersonLeftHand,
        Self::FirstPersonRightHand,
        Self::Head,
        Self::Gui,
        Self::Ground,
        Self::Fixed,
        Self::OnShelf,
    ];

    pub fn id(self) -> u8 {
        match self {
            Self::None => 0,
            Self::ThirdPersonLeftHand => 1,
            Self::ThirdPersonRightHand => 2,
            Self::FirstPersonLeftHand => 3,
            Self::FirstPersonRightHand => 4,
            Self::Head => 5,
            Self::Gui => 6,
            Self::Ground => 7,
            Self::Fixed => 8,
            Self::OnShelf => 9,
        }
    }

    pub fn by_id(id: i32) -> Self {
        if id < 0 {
            return Self::None;
        }
        let id = id as u8;
        Self::VALUES
            .iter()
            .copied()
            .find(|context| context.id() == id)
            .unwrap_or(Self::None)
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ThirdPersonLeftHand => "thirdperson_lefthand",
            Self::ThirdPersonRightHand => "thirdperson_righthand",
            Self::FirstPersonLeftHand => "firstperson_lefthand",
            Self::FirstPersonRightHand => "firstperson_righthand",
            Self::Head => "head",
            Self::Gui => "gui",
            Self::Ground => "ground",
            Self::Fixed => "fixed",
            Self::OnShelf => "on_shelf",
        }
    }

    pub fn by_serialized_name(name: &str) -> Option<Self> {
        Self::VALUES
            .iter()
            .copied()
            .find(|context| context.serialized_name() == name)
    }

    pub fn first_person(self) -> bool {
        matches!(self, Self::FirstPersonLeftHand | Self::FirstPersonRightHand)
    }

    pub fn left_hand(self) -> bool {
        matches!(self, Self::FirstPersonLeftHand | Self::ThirdPersonLeftHand)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingAnimationType {
    None,
    Whack,
    Stab,
}

impl SwingAnimationType {
    pub const VALUES: [Self; 3] = [Self::None, Self::Whack, Self::Stab];

    pub fn id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Whack => 1,
            Self::Stab => 2,
        }
    }

    pub fn by_id(id: i32) -> Self {
        Self::VALUES
            .iter()
            .copied()
            .find(|animation| animation.id() == id)
            .unwrap_or(Self::None)
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Whack => "whack",
            Self::Stab => "stab",
        }
    }

    pub fn by_serialized_name(name: &str) -> Option<Self> {
        Self::VALUES
            .iter()
            .copied()
            .find(|animation| animation.serialized_name() == name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipFlag {
    advanced: bool,
    creative: bool,
}

impl TooltipFlag {
    pub const NORMAL: Self = Self {
        advanced: false,
        creative: false,
    };
    pub const ADVANCED: Self = Self {
        advanced: true,
        creative: false,
    };

    pub fn new(advanced: bool, creative: bool) -> Self {
        Self { advanced, creative }
    }

    pub fn is_advanced(self) -> bool {
        self.advanced
    }

    pub fn is_creative(self) -> bool {
        self.creative
    }

    pub fn as_creative(self) -> Self {
        Self {
            advanced: self.advanced,
            creative: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UseCooldown {
    pub seconds: f32,
    pub cooldown_group: Option<&'static str>,
}

impl UseCooldown {
    pub fn new(seconds: f32) -> Self {
        Self {
            seconds,
            cooldown_group: None,
        }
    }

    pub fn with_group(seconds: f32, cooldown_group: &'static str) -> Self {
        Self {
            seconds,
            cooldown_group: Some(cooldown_group),
        }
    }

    pub fn ticks(self) -> i32 {
        (self.seconds * 20.0) as i32
    }
}

/// `net.minecraft.world.item.component.MapPostProcessing` — marks a cartography-table
/// result map for post-processing when the player takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapPostProcessing {
    Lock,
    Scale,
}

impl MapPostProcessing {
    /// Network id (`LOCK(0)`, `SCALE(1)`).
    pub fn id(self) -> i32 {
        match self {
            Self::Lock => 0,
            Self::Scale => 1,
        }
    }

    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::Scale,
            _ => Self::Lock,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemUseAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Trident,
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
    Bundle,
    Spear,
}

impl ItemUseAnimation {
    pub const VALUES: [Self; 12] = [
        Self::None,
        Self::Eat,
        Self::Drink,
        Self::Block,
        Self::Bow,
        Self::Trident,
        Self::Crossbow,
        Self::Spyglass,
        Self::TootHorn,
        Self::Brush,
        Self::Bundle,
        Self::Spear,
    ];

    pub fn id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Eat => 1,
            Self::Drink => 2,
            Self::Block => 3,
            Self::Bow => 4,
            Self::Trident => 5,
            Self::Crossbow => 6,
            Self::Spyglass => 7,
            Self::TootHorn => 8,
            Self::Brush => 9,
            Self::Bundle => 10,
            Self::Spear => 11,
        }
    }

    pub fn by_id(id: i32) -> Self {
        Self::VALUES
            .iter()
            .copied()
            .find(|animation| animation.id() == id)
            .unwrap_or(Self::None)
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Eat => "eat",
            Self::Drink => "drink",
            Self::Block => "block",
            Self::Bow => "bow",
            Self::Trident => "trident",
            Self::Crossbow => "crossbow",
            Self::Spyglass => "spyglass",
            Self::TootHorn => "toot_horn",
            Self::Brush => "brush",
            Self::Bundle => "bundle",
            Self::Spear => "spear",
        }
    }

    pub fn by_serialized_name(name: &str) -> Option<Self> {
        Self::VALUES
            .iter()
            .copied()
            .find(|animation| animation.serialized_name() == name)
    }

    pub fn has_custom_arm_transform(self) -> bool {
        matches!(self, Self::Eat | Self::Drink | Self::Spear)
    }
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

/// `minecraft:firework_explosion` (`FireworkExplosion`) — a single firework-star
/// burst: shape id, layered colours, fade colours, and the trail/twinkle flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkExplosion {
    pub shape: &'static str,
    pub colors: Vec<u32>,
    pub fade_colors: Vec<u32>,
    pub trail: bool,
    pub twinkle: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemComponent {
    MaxStackSize(u32),
    MaxDamage(u32),
    Damage(u32),
    Rarity(Rarity),
    Repairable(&'static str),
    UseAnimation(ItemUseAnimation),
    SwingAnimation(SwingAnimationComponent),
    UseEffects(UseEffectsComponent),
    UseCooldown(UseCooldown),
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
    TooltipDisplay(TooltipDisplayComponent),
    ItemLore(ItemLoreComponent),
    CustomModelData(CustomModelData),
    Instrument(InstrumentComponent),
    UseRemainder(&'static str),
    ItemModel(&'static str),
    ItemName(&'static str),
    /// `minecraft:banner_patterns` — ordered banner pattern layers (pattern + dye
    /// colour), e.g. the result of a loom application or an existing patterned banner.
    BannerPatterns(Vec<crate::block_entity::BannerPatternLayer>),
    /// `minecraft:map_post_processing` — cartography-table result marker (scale/lock).
    MapPostProcessing(MapPostProcessing),
    /// `minecraft:map_id` (`MapId`) — the saved-data id of a filled map, used to look
    /// up its `MapItemSavedData` (scale, decorations, …).
    MapId(i32),
    /// `minecraft:enchantments` — enchantments applied to a tool/armour (id → level).
    Enchantments(BTreeMap<String, i32>),
    /// `minecraft:stored_enchantments` — enchantments carried by an enchanted book
    /// (id → level); applied to an item only via the anvil.
    StoredEnchantments(BTreeMap<String, i32>),
    /// `minecraft:repair_cost` — accumulated anvil prior-work penalty.
    RepairCost(i32),
    /// `minecraft:trim` — an armour trim (`ArmorTrim`): the trim material + pattern ids.
    ArmorTrim {
        material: &'static str,
        pattern: &'static str,
    },
    WritableBookContent(Vec<String>),
    WrittenBookContent {
        title: String,
        author: String,
        generation: i32,
        pages: Vec<String>,
        resolved: bool,
    },
    /// `minecraft:dyed_color` (`DyedItemColor`) — the packed RGB tint applied to
    /// leather armour (and other dyeable items) by the crafting dye recipe.
    DyedColor(u32),
    /// `minecraft:base_color` (`DyeColor`) — the base dye colour of a banner or a
    /// shield (e.g. `"red"`), as carried over by the shield-decoration recipe.
    BaseColor(&'static str),
    /// `minecraft:potion_contents` (`PotionContents`) — the potion id a
    /// potion/tipped-arrow carries (modelled by id; full effect list is data-driven).
    PotionContents(&'static str),
    /// `minecraft:firework_explosion` (`FireworkExplosion`) — a firework star's burst.
    FireworkExplosion(FireworkExplosion),
    /// `minecraft:fireworks` (`Fireworks`) — a rocket's flight duration (1–3) and the
    /// list of star explosions it carries.
    Fireworks {
        flight_duration: u8,
        explosions: Vec<FireworkExplosion>,
    },
    /// `minecraft:pot_decorations` (`PotDecorations`) — the four sherd/brick ids on a
    /// decorated pot's back/left/right/front faces.
    PotDecorations {
        back: &'static str,
        left: &'static str,
        right: &'static str,
        front: &'static str,
    },
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
    pub cooldown_group: Option<&'static str>,
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
                ItemComponent::TooltipDisplay(TooltipDisplayComponent::default_component()),
                ItemComponent::ItemLore(ItemLoreComponent::empty()),
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
        self.set(ItemComponent::UseCooldown(UseCooldown::new(seconds)));
        self
    }

    pub fn use_cooldown_group(mut self, seconds: f32, cooldown_group: &'static str) -> Self {
        self.set(ItemComponent::UseCooldown(UseCooldown::with_group(
            seconds,
            cooldown_group,
        )));
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

    pub fn writable_book_content(mut self, pages: Vec<String>) -> Self {
        self.set(ItemComponent::WritableBookContent(pages));
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
            cooldown_group: None,
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
                ItemComponent::UseCooldown(cooldown) => {
                    effective.cooldown_seconds = Some(cooldown.seconds);
                    effective.cooldown_group = cooldown.cooldown_group;
                }
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
                | ItemComponent::SwingAnimation(_)
                | ItemComponent::UseEffects(_)
                | ItemComponent::TooltipDisplay(_)
                | ItemComponent::ItemLore(_)
                | ItemComponent::CustomModelData(_)
                | ItemComponent::Instrument(_)
                | ItemComponent::UseRemainder(_)
                | ItemComponent::ItemModel(_)
                | ItemComponent::ItemName(_)
                | ItemComponent::BannerPatterns(_)
                | ItemComponent::MapPostProcessing(_)
                | ItemComponent::Enchantments(_)
                | ItemComponent::StoredEnchantments(_)
                | ItemComponent::RepairCost(_)
                | ItemComponent::ArmorTrim { .. }
                | ItemComponent::WritableBookContent(_)
                | ItemComponent::WrittenBookContent { .. }
                | ItemComponent::MapId(_)
                | ItemComponent::DyedColor(_)
                | ItemComponent::BaseColor(_)
                | ItemComponent::PotionContents(_)
                | ItemComponent::FireworkExplosion(_)
                | ItemComponent::Fireworks { .. }
                | ItemComponent::PotDecorations { .. }
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
            Self::SwingAnimation(_) => "minecraft:swing_animation",
            Self::UseEffects(_) => "minecraft:use_effects",
            Self::UseCooldown(_) => "minecraft:use_cooldown",
            Self::Food { .. } => "minecraft:food",
            Self::Equippable { .. } => "minecraft:equippable",
            Self::Enchantable(_) => "minecraft:enchantable",
            Self::Tooltip(_) => "minecraft:enchantment_glint_override",
            Self::TooltipDisplay(_) => "minecraft:tooltip_display",
            Self::ItemLore(_) => "minecraft:lore",
            Self::CustomModelData(_) => "minecraft:custom_model_data",
            Self::Instrument(_) => "minecraft:instrument",
            Self::UseRemainder(_) => "minecraft:use_remainder",
            Self::ItemModel(_) => "minecraft:item_model",
            Self::ItemName(_) => "minecraft:item_name",
            Self::BannerPatterns(_) => "minecraft:banner_patterns",
            Self::MapPostProcessing(_) => "minecraft:map_post_processing",
            Self::Enchantments(_) => "minecraft:enchantments",
            Self::StoredEnchantments(_) => "minecraft:stored_enchantments",
            Self::RepairCost(_) => "minecraft:repair_cost",
            Self::ArmorTrim { .. } => "minecraft:trim",
            Self::WritableBookContent(_) => "minecraft:writable_book_content",
            Self::WrittenBookContent { .. } => "minecraft:written_book_content",
            Self::MapId(_) => "minecraft:map_id",
            Self::DyedColor(_) => "minecraft:dyed_color",
            Self::BaseColor(_) => "minecraft:base_color",
            Self::PotionContents(_) => "minecraft:potion_contents",
            Self::FireworkExplosion(_) => "minecraft:firework_explosion",
            Self::Fireworks { .. } => "minecraft:fireworks",
            Self::PotDecorations { .. } => "minecraft:pot_decorations",
            Self::Custom(name) => name,
        }
    }
}

pub fn representative_item_definitions() -> Vec<ItemDefinition> {
    let mut definitions = vec![
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
        ItemDefinition::new("minecraft:writable_book")
            .stacks_to(1)
            .writable_book_content(Vec::new()),
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
        // Items.java: FLINT_AND_STEEL = registerItem("flint_and_steel",
        // FlintAndSteelItem::new, new Item.Properties().durability(64)).
        ItemDefinition::new("minecraft:flint_and_steel").durability(64),
        ItemDefinition::new("minecraft:bundle")
            .stacks_to(1)
            .use_animation(ItemUseAnimation::Bundle)
            .custom("minecraft:bundle_contents"),
        ItemDefinition::new("minecraft:ominous_bottle")
            .rarity(Rarity::Uncommon)
            .drink()
            .custom("minecraft:ominous_bottle_amplifier"),
        ItemDefinition::new("minecraft:trial_key"),
    ];
    definitions.extend(tool_material_item_definitions());
    definitions
}

/// Items.java registers AxeItem/ShovelItem/HoeItem through
/// `ToolMaterial.applyToolProperties`, whose common path applies the material
/// durability (WOOD 59, STONE 131, COPPER 190, IRON 250, GOLD 32,
/// DIAMOND 1561, NETHERITE 2031).
fn tool_material_item_definitions() -> Vec<ItemDefinition> {
    vec![
        ItemDefinition::new("minecraft:wooden_axe").durability(59),
        ItemDefinition::new("minecraft:stone_axe").durability(131),
        ItemDefinition::new("minecraft:copper_axe").durability(190),
        ItemDefinition::new("minecraft:iron_axe").durability(250),
        ItemDefinition::new("minecraft:golden_axe").durability(32),
        ItemDefinition::new("minecraft:diamond_axe").durability(1561),
        ItemDefinition::new("minecraft:netherite_axe").durability(2031),
        ItemDefinition::new("minecraft:wooden_shovel").durability(59),
        ItemDefinition::new("minecraft:stone_shovel").durability(131),
        ItemDefinition::new("minecraft:copper_shovel").durability(190),
        ItemDefinition::new("minecraft:iron_shovel").durability(250),
        ItemDefinition::new("minecraft:golden_shovel").durability(32),
        ItemDefinition::new("minecraft:diamond_shovel").durability(1561),
        ItemDefinition::new("minecraft:netherite_shovel").durability(2031),
        ItemDefinition::new("minecraft:wooden_hoe").durability(59),
        ItemDefinition::new("minecraft:stone_hoe").durability(131),
        ItemDefinition::new("minecraft:copper_hoe").durability(190),
        ItemDefinition::new("minecraft:iron_hoe").durability(250),
        ItemDefinition::new("minecraft:golden_hoe").durability(32),
        ItemDefinition::new("minecraft:diamond_hoe").durability(1561),
        ItemDefinition::new("minecraft:netherite_hoe").durability(2031),
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

    const ITEM_DISPLAY_CONTEXT_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/world/item/ItemDisplayContext.java");
    const ITEM_USE_ANIMATION_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/world/item/ItemUseAnimation.java");
    const RARITY_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/world/item/Rarity.java");
    const SWING_ANIMATION_TYPE_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/world/item/SwingAnimationType.java");
    const TOOLTIP_FLAG_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/world/item/TooltipFlag.java");
    const USE_COOLDOWN_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/UseCooldown.java"
    );

    #[test]
    fn item_display_context_matches_java_ids_names_and_hand_helpers() {
        assert!(ITEM_DISPLAY_CONTEXT_JAVA.contains("NONE(0, \"none\")"));
        assert!(
            ITEM_DISPLAY_CONTEXT_JAVA.contains("ON_SHELF(9, \"on_shelf\")")
        );
        assert!(ITEM_DISPLAY_CONTEXT_JAVA
            .contains("ByIdMap.OutOfBoundsStrategy.ZERO"));
        assert!(ITEM_DISPLAY_CONTEXT_JAVA.contains(
            "return this == FIRST_PERSON_LEFT_HAND || this == FIRST_PERSON_RIGHT_HAND;"
        ));
        assert!(ITEM_DISPLAY_CONTEXT_JAVA
            .contains("return this == FIRST_PERSON_LEFT_HAND || this == THIRD_PERSON_LEFT_HAND;"));

        let names: Vec<_> = ItemDisplayContext::VALUES
            .iter()
            .map(|context| context.serialized_name())
            .collect();
        assert_eq!(
            names,
            vec![
                "none",
                "thirdperson_lefthand",
                "thirdperson_righthand",
                "firstperson_lefthand",
                "firstperson_righthand",
                "head",
                "gui",
                "ground",
                "fixed",
                "on_shelf",
            ]
        );
        assert_eq!(ItemDisplayContext::None.id(), 0);
        assert_eq!(ItemDisplayContext::OnShelf.id(), 9);
        assert_eq!(ItemDisplayContext::by_id(-1), ItemDisplayContext::None);
        assert_eq!(ItemDisplayContext::by_id(99), ItemDisplayContext::None);
        assert_eq!(
            ItemDisplayContext::by_serialized_name("firstperson_lefthand"),
            Some(ItemDisplayContext::FirstPersonLeftHand)
        );
        assert!(ItemDisplayContext::FirstPersonLeftHand.first_person());
        assert!(ItemDisplayContext::FirstPersonRightHand.first_person());
        assert!(!ItemDisplayContext::ThirdPersonRightHand.first_person());
        assert!(ItemDisplayContext::FirstPersonLeftHand.left_hand());
        assert!(ItemDisplayContext::ThirdPersonLeftHand.left_hand());
        assert!(!ItemDisplayContext::FirstPersonRightHand.left_hand());
    }

    #[test]
    fn item_use_animation_matches_java_ids_names_and_custom_arm_transform() {
        assert!(ITEM_USE_ANIMATION_JAVA.contains("NONE(0, \"none\")"));
        assert!(ITEM_USE_ANIMATION_JAVA.contains("TOOT_HORN(8, \"toot_horn\")"));
        assert!(ITEM_USE_ANIMATION_JAVA.contains("SPEAR(11, \"spear\", true)"));
        assert!(ITEM_USE_ANIMATION_JAVA.contains("ByIdMap.OutOfBoundsStrategy.ZERO"));
        assert!(ITEM_USE_ANIMATION_JAVA.contains("hasCustomArmTransform"));

        let names: Vec<_> = ItemUseAnimation::VALUES
            .iter()
            .map(|animation| animation.serialized_name())
            .collect();
        assert_eq!(
            names,
            vec![
                "none",
                "eat",
                "drink",
                "block",
                "bow",
                "trident",
                "crossbow",
                "spyglass",
                "toot_horn",
                "brush",
                "bundle",
                "spear",
            ]
        );
        assert_eq!(ItemUseAnimation::None.id(), 0);
        assert_eq!(ItemUseAnimation::Spear.id(), 11);
        assert_eq!(ItemUseAnimation::by_id(-1), ItemUseAnimation::None);
        assert_eq!(ItemUseAnimation::by_id(99), ItemUseAnimation::None);
        assert_eq!(
            ItemUseAnimation::by_serialized_name("toot_horn"),
            Some(ItemUseAnimation::TootHorn)
        );
        assert!(ItemUseAnimation::Eat.has_custom_arm_transform());
        assert!(ItemUseAnimation::Drink.has_custom_arm_transform());
        assert!(ItemUseAnimation::Spear.has_custom_arm_transform());
        assert!(!ItemUseAnimation::Bow.has_custom_arm_transform());
        assert!(!ItemUseAnimation::Trident.has_custom_arm_transform());
    }

    #[test]
    fn rarity_matches_java_ids_names_colors_and_zero_fallback() {
        assert!(RARITY_JAVA.contains("COMMON(0, \"common\", ChatFormatting.WHITE)"));
        assert!(RARITY_JAVA.contains("EPIC(3, \"epic\", ChatFormatting.LIGHT_PURPLE)"));
        assert!(RARITY_JAVA.contains("ByIdMap.OutOfBoundsStrategy.ZERO"));
        assert!(RARITY_JAVA.contains("public ChatFormatting color()"));

        assert_eq!(Rarity::VALUES.len(), 4);
        assert_eq!(Rarity::Common.id(), 0);
        assert_eq!(Rarity::Epic.id(), 3);
        assert_eq!(Rarity::by_id(-1), Rarity::Common);
        assert_eq!(Rarity::by_id(99), Rarity::Common);
        assert_eq!(Rarity::Uncommon.serialized_name(), "uncommon");
        assert_eq!(Rarity::by_serialized_name("rare"), Some(Rarity::Rare));
        assert_eq!(
            Rarity::Common.color(),
            crate::chat_formatting::ChatFormatting::White
        );
        assert_eq!(
            Rarity::Uncommon.color(),
            crate::chat_formatting::ChatFormatting::Yellow
        );
        assert_eq!(
            Rarity::Rare.color(),
            crate::chat_formatting::ChatFormatting::Aqua
        );
        assert_eq!(
            Rarity::Epic.color(),
            crate::chat_formatting::ChatFormatting::LightPurple
        );
    }

    #[test]
    fn swing_animation_type_matches_java_ids_names_and_zero_fallback() {
        assert!(SWING_ANIMATION_TYPE_JAVA.contains("NONE(0, \"none\")"));
        assert!(SWING_ANIMATION_TYPE_JAVA.contains("STAB(2, \"stab\")"));
        assert!(SWING_ANIMATION_TYPE_JAVA.contains("ByIdMap.OutOfBoundsStrategy.ZERO"));
        assert!(SWING_ANIMATION_TYPE_JAVA.contains("StreamCodec<ByteBuf, SwingAnimationType>"));

        assert_eq!(SwingAnimationType::VALUES.len(), 3);
        assert_eq!(SwingAnimationType::None.id(), 0);
        assert_eq!(SwingAnimationType::Whack.id(), 1);
        assert_eq!(SwingAnimationType::Stab.id(), 2);
        assert_eq!(SwingAnimationType::by_id(-1), SwingAnimationType::None);
        assert_eq!(SwingAnimationType::by_id(99), SwingAnimationType::None);
        assert_eq!(SwingAnimationType::Whack.serialized_name(), "whack");
        assert_eq!(
            SwingAnimationType::by_serialized_name("stab"),
            Some(SwingAnimationType::Stab)
        );
    }

    #[test]
    fn tooltip_flag_defaults_and_creative_copy_match_java() {
        assert!(TOOLTIP_FLAG_JAVA
            .contains("TooltipFlag.Default NORMAL = new TooltipFlag.Default(false, false);"));
        assert!(TOOLTIP_FLAG_JAVA
            .contains("TooltipFlag.Default ADVANCED = new TooltipFlag.Default(true, false);"));
        assert!(TOOLTIP_FLAG_JAVA.contains("return new TooltipFlag.Default(this.advanced, true);"));

        assert!(!TooltipFlag::NORMAL.is_advanced());
        assert!(!TooltipFlag::NORMAL.is_creative());
        assert!(TooltipFlag::ADVANCED.is_advanced());
        assert!(!TooltipFlag::ADVANCED.is_creative());

        let creative_normal = TooltipFlag::NORMAL.as_creative();
        assert!(!creative_normal.is_advanced());
        assert!(creative_normal.is_creative());

        let creative_advanced = TooltipFlag::ADVANCED.as_creative();
        assert!(creative_advanced.is_advanced());
        assert!(creative_advanced.is_creative());
        assert_eq!(
            TooltipFlag::new(true, true),
            TooltipFlag::ADVANCED.as_creative()
        );
    }

    #[test]
    fn use_cooldown_seconds_group_and_ticks_match_java() {
        for sentinel in [
            "public record UseCooldown(float seconds, Optional<Identifier> cooldownGroup)",
            "ExtraCodecs.POSITIVE_FLOAT.fieldOf(\"seconds\").forGetter(UseCooldown::seconds)",
            "Identifier.CODEC.optionalFieldOf(\"cooldown_group\").forGetter(UseCooldown::cooldownGroup)",
            "public UseCooldown(final float seconds)",
            "return (int)(this.seconds * 20.0F);",
            "player.getCooldowns().addCooldown(stack, this.ticks());",
        ] {
            assert!(
                USE_COOLDOWN_JAVA.contains(sentinel),
                "missing UseCooldown sentinel {sentinel}"
            );
        }

        let default = UseCooldown::new(1.25);
        assert_eq!(default.seconds, 1.25);
        assert_eq!(default.cooldown_group, None);
        assert_eq!(default.ticks(), 25);

        let grouped = UseCooldown::with_group(0.5, "minecraft:throwables");
        assert_eq!(grouped.seconds, 0.5);
        assert_eq!(grouped.cooldown_group, Some("minecraft:throwables"));
        assert_eq!(grouped.ticks(), 10);

        let pearl = item_definition("minecraft:ender_pearl").unwrap().effective();
        assert_eq!(pearl.cooldown_seconds, Some(1.0));
        assert_eq!(pearl.cooldown_group, None);
    }

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
        assert!(!ids.windows(2).all(|pair| pair[0] < pair[1]));
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
        assert!(book.components.contains_key("minecraft:lore"));
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
        assert_eq!(
            ItemComponent::ItemLore(crate::item_lore_component::ItemLoreComponent::empty()).key(),
            "minecraft:lore"
        );
        assert_eq!(
            ItemComponent::Instrument(crate::item_instrument_component::InstrumentComponent::new(
                "minecraft:ponder_goat_horn"
            ))
            .key(),
            "minecraft:instrument"
        );
    }
}
