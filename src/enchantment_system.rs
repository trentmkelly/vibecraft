#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlotGroup {
    Any,
    Armor,
    Head,
    Chest,
    Legs,
    Feet,
    Hand,
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cost {
    pub base: i32,
    pub per_level: i32,
}

impl Cost {
    pub const fn constant(base: i32) -> Self {
        Self { base, per_level: 0 }
    }

    pub const fn dynamic(base: i32, per_level: i32) -> Self {
        Self { base, per_level }
    }

    pub fn calculate(self, level: i32) -> i32 {
        self.base + self.per_level * (level - 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnchantmentGroup {
    ArmorExclusive,
    BootsExclusive,
    WeaponDamageExclusive,
    MiningExclusive,
    BowExclusive,
    TridentExclusive,
    CrossbowExclusive,
    MaceExclusive,
    Curse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnchantmentEffectHook {
    DamageProtection,
    DamageImmunity,
    Damage,
    SmashDamagePerFallenBlock,
    Knockback,
    ArmorEffectiveness,
    PostAttack,
    PostPiercingAttack,
    HitBlock,
    ItemDamage,
    EquipmentDrops,
    LocationChanged,
    Tick,
    AmmoUse,
    ProjectilePiercing,
    ProjectileSpawned,
    ProjectileSpread,
    ProjectileCount,
    TridentReturnAcceleration,
    FishingTimeReduction,
    FishingLuckBonus,
    BlockExperience,
    MobExperience,
    RepairWithXp,
    Attributes,
    CrossbowChargeTime,
    CrossbowChargingSounds,
    TridentSound,
    PreventEquipmentDrop,
    PreventArmorChange,
    TridentSpinAttackStrength,
}

pub const ENCHANTMENT_EFFECT_COMPONENTS: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::DamageProtection,
    EnchantmentEffectHook::DamageImmunity,
    EnchantmentEffectHook::Damage,
    EnchantmentEffectHook::SmashDamagePerFallenBlock,
    EnchantmentEffectHook::Knockback,
    EnchantmentEffectHook::ArmorEffectiveness,
    EnchantmentEffectHook::PostAttack,
    EnchantmentEffectHook::PostPiercingAttack,
    EnchantmentEffectHook::HitBlock,
    EnchantmentEffectHook::ItemDamage,
    EnchantmentEffectHook::EquipmentDrops,
    EnchantmentEffectHook::LocationChanged,
    EnchantmentEffectHook::Tick,
    EnchantmentEffectHook::AmmoUse,
    EnchantmentEffectHook::ProjectilePiercing,
    EnchantmentEffectHook::ProjectileSpawned,
    EnchantmentEffectHook::ProjectileSpread,
    EnchantmentEffectHook::ProjectileCount,
    EnchantmentEffectHook::TridentReturnAcceleration,
    EnchantmentEffectHook::FishingTimeReduction,
    EnchantmentEffectHook::FishingLuckBonus,
    EnchantmentEffectHook::BlockExperience,
    EnchantmentEffectHook::MobExperience,
    EnchantmentEffectHook::RepairWithXp,
    EnchantmentEffectHook::Attributes,
    EnchantmentEffectHook::CrossbowChargeTime,
    EnchantmentEffectHook::CrossbowChargingSounds,
    EnchantmentEffectHook::TridentSound,
    EnchantmentEffectHook::PreventEquipmentDrop,
    EnchantmentEffectHook::PreventArmorChange,
    EnchantmentEffectHook::TridentSpinAttackStrength,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantmentDef {
    pub id: &'static str,
    pub supported_items: &'static str,
    pub primary_items: Option<&'static str>,
    pub weight: i32,
    pub max_level: i32,
    pub min_cost: Cost,
    pub max_cost: Cost,
    pub anvil_cost: i32,
    pub slots: &'static [EquipmentSlotGroup],
    pub groups: &'static [EnchantmentGroup],
    pub hooks: &'static [EnchantmentEffectHook],
}

const ARMOR: &[EquipmentSlotGroup] = &[EquipmentSlotGroup::Armor];
const FEET: &[EquipmentSlotGroup] = &[EquipmentSlotGroup::Feet];
const HEAD: &[EquipmentSlotGroup] = &[EquipmentSlotGroup::Head];
const HAND: &[EquipmentSlotGroup] = &[EquipmentSlotGroup::Hand];
const MAIN_HAND: &[EquipmentSlotGroup] = &[EquipmentSlotGroup::MainHand];
const ANY: &[EquipmentSlotGroup] = &[EquipmentSlotGroup::Any];
const ARMOR_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::ArmorExclusive];
const BOOTS_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::BootsExclusive];
const WEAPON_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::WeaponDamageExclusive];
const MINING_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::MiningExclusive];
const BOW_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::BowExclusive];
const TRIDENT_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::TridentExclusive];
const CROSSBOW_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::CrossbowExclusive];
const MACE_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::MaceExclusive];
const CURSE_GROUP: &[EnchantmentGroup] = &[EnchantmentGroup::Curse];

const DAMAGE_PROTECTION: &[EnchantmentEffectHook] = &[EnchantmentEffectHook::DamageProtection];
const DAMAGE_ATTR: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::DamageProtection,
    EnchantmentEffectHook::Attributes,
];
const DAMAGE: &[EnchantmentEffectHook] = &[EnchantmentEffectHook::Damage];
const DAMAGE_POST: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::Damage,
    EnchantmentEffectHook::PostAttack,
];
const KNOCKBACK: &[EnchantmentEffectHook] = &[EnchantmentEffectHook::Knockback];
const ITEM_DAMAGE: &[EnchantmentEffectHook] = &[EnchantmentEffectHook::ItemDamage];
const LOOT: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::BlockExperience,
    EnchantmentEffectHook::MobExperience,
];
const MOVEMENT: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::Attributes,
    EnchantmentEffectHook::LocationChanged,
];
const PROJECTILE: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::ProjectileCount,
    EnchantmentEffectHook::ProjectileSpread,
    EnchantmentEffectHook::ProjectilePiercing,
    EnchantmentEffectHook::ProjectileSpawned,
];
const CROSSBOW: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::CrossbowChargeTime,
    EnchantmentEffectHook::CrossbowChargingSounds,
    EnchantmentEffectHook::ProjectileCount,
    EnchantmentEffectHook::ProjectilePiercing,
];
const TRIDENT: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::TridentReturnAcceleration,
    EnchantmentEffectHook::TridentSpinAttackStrength,
    EnchantmentEffectHook::TridentSound,
];
const PREVENT_DROP: &[EnchantmentEffectHook] = &[
    EnchantmentEffectHook::PreventEquipmentDrop,
    EnchantmentEffectHook::PreventArmorChange,
];
const EMPTY_HOOKS: &[EnchantmentEffectHook] = &[];
const EMPTY_GROUPS: &[EnchantmentGroup] = &[];

struct EnchantmentSpec {
    id: &'static str,
    supported_items: &'static str,
    weight: i32,
    max_level: i32,
    min_cost: Cost,
    max_cost: Cost,
    anvil_cost: i32,
    slots: &'static [EquipmentSlotGroup],
    groups: &'static [EnchantmentGroup],
    hooks: &'static [EnchantmentEffectHook],
}

macro_rules! ench {
    (
        $id:expr,
        $supported_items:expr,
        $weight:expr,
        $max_level:expr,
        $min_cost:expr,
        $max_cost:expr,
        $anvil_cost:expr,
        $slots:expr,
        $groups:expr,
        $hooks:expr $(,)?
    ) => {
        enchantment_from_spec(EnchantmentSpec {
            id: $id,
            supported_items: $supported_items,
            weight: $weight,
            max_level: $max_level,
            min_cost: $min_cost,
            max_cost: $max_cost,
            anvil_cost: $anvil_cost,
            slots: $slots,
            groups: $groups,
            hooks: $hooks,
        })
    };
}

const fn enchantment_from_spec(spec: EnchantmentSpec) -> EnchantmentDef {
    EnchantmentDef {
        id: spec.id,
        supported_items: spec.supported_items,
        primary_items: None,
        weight: spec.weight,
        max_level: spec.max_level,
        min_cost: spec.min_cost,
        max_cost: spec.max_cost,
        anvil_cost: spec.anvil_cost,
        slots: spec.slots,
        groups: spec.groups,
        hooks: spec.hooks,
    }
}

pub const ENCHANTMENTS: &[EnchantmentDef] = &[
    ench!(
        "minecraft:protection",
        "#minecraft:enchantable/armor",
        10,
        4,
        Cost::dynamic(1, 11),
        Cost::dynamic(12, 11),
        1,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_PROTECTION,
    ),
    ench!(
        "minecraft:fire_protection",
        "#minecraft:enchantable/armor",
        5,
        4,
        Cost::dynamic(10, 8),
        Cost::dynamic(18, 8),
        2,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_ATTR,
    ),
    ench!(
        "minecraft:feather_falling",
        "#minecraft:enchantable/foot_armor",
        5,
        4,
        Cost::dynamic(5, 6),
        Cost::dynamic(11, 6),
        2,
        FEET,
        EMPTY_GROUPS,
        DAMAGE_PROTECTION,
    ),
    ench!(
        "minecraft:blast_protection",
        "#minecraft:enchantable/armor",
        2,
        4,
        Cost::dynamic(5, 8),
        Cost::dynamic(13, 8),
        4,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_ATTR,
    ),
    ench!(
        "minecraft:projectile_protection",
        "#minecraft:enchantable/armor",
        5,
        4,
        Cost::dynamic(3, 6),
        Cost::dynamic(9, 6),
        2,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_PROTECTION,
    ),
    ench!(
        "minecraft:respiration",
        "#minecraft:enchantable/head_armor",
        2,
        3,
        Cost::dynamic(10, 10),
        Cost::dynamic(40, 10),
        4,
        HEAD,
        EMPTY_GROUPS,
        EMPTY_HOOKS,
    ),
    ench!(
        "minecraft:aqua_affinity",
        "#minecraft:enchantable/head_armor",
        2,
        1,
        Cost::constant(1),
        Cost::constant(41),
        4,
        HEAD,
        EMPTY_GROUPS,
        EMPTY_HOOKS,
    ),
    ench!(
        "minecraft:thorns",
        "#minecraft:enchantable/armor",
        1,
        3,
        Cost::dynamic(10, 20),
        Cost::dynamic(60, 20),
        8,
        ARMOR,
        EMPTY_GROUPS,
        DAMAGE_POST,
    ),
    ench!(
        "minecraft:depth_strider",
        "#minecraft:enchantable/foot_armor",
        2,
        3,
        Cost::dynamic(10, 10),
        Cost::dynamic(25, 10),
        4,
        FEET,
        BOOTS_GROUP,
        MOVEMENT,
    ),
    ench!(
        "minecraft:frost_walker",
        "#minecraft:enchantable/foot_armor",
        2,
        2,
        Cost::dynamic(10, 10),
        Cost::dynamic(25, 10),
        4,
        FEET,
        BOOTS_GROUP,
        MOVEMENT,
    ),
    ench!(
        "minecraft:binding_curse",
        "#minecraft:enchantable/equippable",
        1,
        1,
        Cost::constant(25),
        Cost::constant(50),
        8,
        ANY,
        CURSE_GROUP,
        PREVENT_DROP,
    ),
    ench!(
        "minecraft:soul_speed",
        "#minecraft:enchantable/foot_armor",
        1,
        3,
        Cost::dynamic(10, 10),
        Cost::dynamic(25, 10),
        8,
        FEET,
        EMPTY_GROUPS,
        MOVEMENT,
    ),
    ench!(
        "minecraft:swift_sneak",
        "#minecraft:enchantable/leg_armor",
        1,
        3,
        Cost::dynamic(25, 25),
        Cost::dynamic(75, 25),
        8,
        &[EquipmentSlotGroup::Legs],
        EMPTY_GROUPS,
        MOVEMENT,
    ),
    ench!(
        "minecraft:sharpness",
        "#minecraft:enchantable/sharp_weapon",
        10,
        5,
        Cost::dynamic(1, 11),
        Cost::dynamic(21, 11),
        1,
        MAIN_HAND,
        WEAPON_GROUP,
        DAMAGE,
    ),
    ench!(
        "minecraft:smite",
        "#minecraft:enchantable/weapon",
        5,
        5,
        Cost::dynamic(5, 8),
        Cost::dynamic(25, 8),
        2,
        MAIN_HAND,
        WEAPON_GROUP,
        DAMAGE,
    ),
    ench!(
        "minecraft:bane_of_arthropods",
        "#minecraft:enchantable/weapon",
        5,
        5,
        Cost::dynamic(5, 8),
        Cost::dynamic(25, 8),
        2,
        MAIN_HAND,
        WEAPON_GROUP,
        DAMAGE_POST,
    ),
    ench!(
        "minecraft:knockback",
        "#minecraft:enchantable/melee_weapon",
        5,
        2,
        Cost::dynamic(5, 20),
        Cost::dynamic(55, 20),
        2,
        MAIN_HAND,
        EMPTY_GROUPS,
        KNOCKBACK,
    ),
    ench!(
        "minecraft:fire_aspect",
        "#minecraft:enchantable/fire_aspect",
        2,
        2,
        Cost::dynamic(10, 20),
        Cost::dynamic(60, 20),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE_POST,
    ),
    ench!(
        "minecraft:looting",
        "#minecraft:enchantable/melee_weapon",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        LOOT,
    ),
    ench!(
        "minecraft:sweeping_edge",
        "#minecraft:enchantable/sweeping",
        2,
        3,
        Cost::dynamic(5, 9),
        Cost::dynamic(20, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE,
    ),
    ench!(
        "minecraft:efficiency",
        "#minecraft:enchantable/mining",
        10,
        5,
        Cost::dynamic(1, 10),
        Cost::dynamic(51, 10),
        1,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::Attributes],
    ),
    ench!(
        "minecraft:silk_touch",
        "#minecraft:enchantable/mining_loot",
        1,
        1,
        Cost::constant(15),
        Cost::constant(65),
        8,
        MAIN_HAND,
        MINING_GROUP,
        &[EnchantmentEffectHook::BlockExperience],
    ),
    ench!(
        "minecraft:unbreaking",
        "#minecraft:enchantable/durability",
        5,
        3,
        Cost::dynamic(5, 8),
        Cost::dynamic(55, 8),
        2,
        ANY,
        EMPTY_GROUPS,
        ITEM_DAMAGE,
    ),
    ench!(
        "minecraft:fortune",
        "#minecraft:enchantable/mining_loot",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        MINING_GROUP,
        LOOT,
    ),
    ench!(
        "minecraft:power",
        "#minecraft:enchantable/bow",
        10,
        5,
        Cost::dynamic(1, 10),
        Cost::dynamic(16, 10),
        1,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE,
    ),
    ench!(
        "minecraft:punch",
        "#minecraft:enchantable/bow",
        2,
        2,
        Cost::dynamic(12, 20),
        Cost::dynamic(37, 20),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        KNOCKBACK,
    ),
    ench!(
        "minecraft:flame",
        "#minecraft:enchantable/bow",
        2,
        1,
        Cost::constant(20),
        Cost::constant(50),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE_POST,
    ),
    ench!(
        "minecraft:infinity",
        "#minecraft:enchantable/bow",
        1,
        1,
        Cost::constant(20),
        Cost::constant(50),
        8,
        MAIN_HAND,
        BOW_GROUP,
        &[EnchantmentEffectHook::AmmoUse],
    ),
    ench!(
        "minecraft:luck_of_the_sea",
        "#minecraft:enchantable/fishing",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::FishingLuckBonus],
    ),
    ench!(
        "minecraft:lure",
        "#minecraft:enchantable/fishing",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::FishingTimeReduction],
    ),
    ench!(
        "minecraft:loyalty",
        "#minecraft:enchantable/trident",
        5,
        3,
        Cost::dynamic(5, 7),
        Cost::dynamic(50, 7),
        2,
        MAIN_HAND,
        TRIDENT_GROUP,
        TRIDENT,
    ),
    ench!(
        "minecraft:impaling",
        "#minecraft:enchantable/trident",
        2,
        5,
        Cost::dynamic(1, 8),
        Cost::dynamic(21, 8),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE,
    ),
    ench!(
        "minecraft:riptide",
        "#minecraft:enchantable/trident",
        2,
        3,
        Cost::dynamic(10, 7),
        Cost::dynamic(25, 7),
        4,
        MAIN_HAND,
        TRIDENT_GROUP,
        TRIDENT,
    ),
    ench!(
        "minecraft:channeling",
        "#minecraft:enchantable/trident",
        1,
        1,
        Cost::constant(25),
        Cost::constant(50),
        8,
        MAIN_HAND,
        TRIDENT_GROUP,
        DAMAGE_POST,
    ),
    ench!(
        "minecraft:multishot",
        "#minecraft:enchantable/crossbow",
        2,
        1,
        Cost::constant(20),
        Cost::constant(50),
        4,
        MAIN_HAND,
        CROSSBOW_GROUP,
        PROJECTILE,
    ),
    ench!(
        "minecraft:quick_charge",
        "#minecraft:enchantable/crossbow",
        5,
        3,
        Cost::dynamic(12, 20),
        Cost::dynamic(50, 20),
        2,
        MAIN_HAND,
        EMPTY_GROUPS,
        CROSSBOW,
    ),
    ench!(
        "minecraft:piercing",
        "#minecraft:enchantable/crossbow",
        10,
        4,
        Cost::dynamic(1, 10),
        Cost::dynamic(50, 10),
        1,
        MAIN_HAND,
        CROSSBOW_GROUP,
        &[EnchantmentEffectHook::ProjectilePiercing],
    ),
    ench!(
        "minecraft:density",
        "#minecraft:enchantable/mace",
        5,
        5,
        Cost::dynamic(5, 8),
        Cost::dynamic(25, 8),
        2,
        MAIN_HAND,
        MACE_GROUP,
        &[EnchantmentEffectHook::SmashDamagePerFallenBlock],
    ),
    ench!(
        "minecraft:breach",
        "#minecraft:enchantable/mace",
        2,
        4,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        MACE_GROUP,
        &[EnchantmentEffectHook::ArmorEffectiveness],
    ),
    ench!(
        "minecraft:wind_burst",
        "#minecraft:enchantable/mace",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        MACE_GROUP,
        &[EnchantmentEffectHook::PostAttack],
    ),
    ench!(
        "minecraft:lunge",
        "#minecraft:enchantable/lunge",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::Attributes],
    ),
    ench!(
        "minecraft:mending",
        "#minecraft:enchantable/durability",
        2,
        1,
        Cost::constant(25),
        Cost::constant(75),
        4,
        ANY,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::RepairWithXp],
    ),
    ench!(
        "minecraft:vanishing_curse",
        "#minecraft:enchantable/vanishing",
        1,
        1,
        Cost::constant(25),
        Cost::constant(50),
        8,
        ANY,
        CURSE_GROUP,
        &[EnchantmentEffectHook::PreventEquipmentDrop],
    ),
];

pub fn enchantment(id: &str) -> Option<&'static EnchantmentDef> {
    let id = if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    };
    ENCHANTMENTS.iter().find(|enchantment| enchantment.id == id)
}

/// Whether an enchantment is in `EnchantmentTags.CURSE` (binding/vanishing curse),
/// modelled by the `Curse` exclusivity group.
mod enchanting;
pub use enchanting::*;


pub fn are_compatible(left: &EnchantmentDef, right: &EnchantmentDef) -> bool {
    left.id != right.id
        && !left
            .groups
            .iter()
            .any(|group| *group != EnchantmentGroup::Curse && right.groups.contains(group))
}

pub fn damage_bonus(id: &str, level: i32, target_family: Option<&str>) -> f32 {
    match id {
        "minecraft:sharpness" => 0.5 * level as f32 + 0.5,
        "minecraft:smite" if target_family == Some("undead") => 2.5 * level as f32,
        "minecraft:bane_of_arthropods" if target_family == Some("arthropod") => 2.5 * level as f32,
        "minecraft:power" => 0.5 * level as f32 + 0.5,
        "minecraft:impaling" if target_family == Some("aquatic") => 2.5 * level as f32,
        _ => 0.0,
    }
}

pub fn protection_bonus(id: &str, level: i32, damage_tag: &str) -> f32 {
    match id {
        "minecraft:protection" if damage_tag != "bypasses_invulnerability" => level as f32,
        "minecraft:fire_protection" if damage_tag == "is_fire" => 2.0 * level as f32,
        "minecraft:feather_falling" if damage_tag == "is_fall" => 3.0 * level as f32,
        "minecraft:blast_protection" if damage_tag == "is_explosion" => 2.0 * level as f32,
        "minecraft:projectile_protection" if damage_tag == "is_projectile" => 2.0 * level as f32,
        _ => 0.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    ByCost,
    ByCostWithDifficulty,
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantmentProviderDef {
    pub id: &'static str,
    pub provider_type: ProviderType,
    pub enchantment: Option<&'static str>,
    pub min_cost: i32,
    pub max_cost: i32,
}

pub const PROVIDER_TYPES: &[ProviderType] = &[
    ProviderType::ByCost,
    ProviderType::ByCostWithDifficulty,
    ProviderType::Single,
];

pub const VANILLA_PROVIDERS: &[EnchantmentProviderDef] = &[
    EnchantmentProviderDef {
        id: "minecraft:mob_spawn_equipment",
        provider_type: ProviderType::ByCostWithDifficulty,
        enchantment: None,
        min_cost: 5,
        max_cost: 17,
    },
    EnchantmentProviderDef {
        id: "minecraft:pillager_spawn_crossbow",
        provider_type: ProviderType::Single,
        enchantment: Some("minecraft:piercing"),
        min_cost: 1,
        max_cost: 1,
    },
    EnchantmentProviderDef {
        id: "minecraft:raid/pillager_post_wave_3",
        provider_type: ProviderType::Single,
        enchantment: Some("minecraft:quick_charge"),
        min_cost: 1,
        max_cost: 1,
    },
    EnchantmentProviderDef {
        id: "minecraft:raid/pillager_post_wave_5",
        provider_type: ProviderType::Single,
        enchantment: Some("minecraft:quick_charge"),
        min_cost: 2,
        max_cost: 2,
    },
    EnchantmentProviderDef {
        id: "minecraft:raid/vindicator",
        provider_type: ProviderType::Single,
        enchantment: Some("minecraft:sharpness"),
        min_cost: 1,
        max_cost: 1,
    },
    EnchantmentProviderDef {
        id: "minecraft:raid/vindicator_post_wave_5",
        provider_type: ProviderType::Single,
        enchantment: Some("minecraft:sharpness"),
        min_cost: 2,
        max_cost: 2,
    },
    EnchantmentProviderDef {
        id: "minecraft:enderman_loot_drop",
        provider_type: ProviderType::Single,
        enchantment: Some("minecraft:silk_touch"),
        min_cost: 1,
        max_cost: 1,
    },
];

/// Compute the damage reduction provided by Protection enchantments.
/// Protection I reduces by 4% per level per piece; max 80% total reduction.
/// Source: vanilla `ProtectionEnchantment.getDamageAfterMagicAbsorb()`
pub fn protection_damage_reduction(total_protection_level: i32, damage: f32) -> f32 {
    let reduction = (total_protection_level * 4).min(80);
    damage * (1.0 - reduction as f32 / 100.0)
}

/// Extra damage from Sharpness (multiplied by 0.5 * level + 0.5).
pub fn sharpness_bonus(level: i32) -> f32 {
    if level <= 0 {
        return 0.0;
    }
    level as f32 * 0.5 + 0.5
}

/// Extra damage from Smite — applies only against undead mobs.
pub fn smite_bonus(level: i32, target_is_undead: bool) -> f32 {
    if !target_is_undead || level <= 0 {
        return 0.0;
    }
    level as f32 * 2.5
}

/// Extra damage from Bane of Arthropods — applies only against arthropods.
pub fn bane_of_arthropods_bonus(level: i32, target_is_arthropod: bool) -> f32 {
    if !target_is_arthropod || level <= 0 {
        return 0.0;
    }
    level as f32 * 2.5
}

/// Extra knockback from Knockback enchantment (added to sprint-knockback).
/// Each level adds 3 blocks of knockback range.
pub fn knockback_bonus_blocks(level: i32) -> f32 {
    level as f32 * 3.0
}

/// Extra projectile knockback from Punch.
pub fn punch_knockback_bonus_blocks(level: i32) -> f32 {
    level as f32 * 3.0
}

/// Probability that Unbreaking prevents durability damage.
/// Current durability handling applies the same modeled skip chance for armor
/// and tools; callers keep passing the item category for Java parity surfaces.
pub fn unbreaking_durability_skip_chance(level: i32, _is_armor: bool) -> f32 {
    if level <= 0 {
        return 0.0;
    }
    1.0 - 1.0 / (level + 1) as f32
}

/// Break speed multiplier from Efficiency.
/// speed += efficiency_level ^ 2 + 1 (added to tool speed when speed > 1.0)
pub fn efficiency_speed_bonus(level: i32) -> f32 {
    if level <= 0 {
        return 0.0;
    }
    (level * level + 1) as f32
}

/// Thorns reflection damage (1-4 HP, random per hit).
/// Level determines the chance of activation: 15% * level.
pub fn thorns_activation_chance(level: i32) -> f32 {
    (level as f32 * 0.15).min(1.0)
}

/// Thorns damage dealt to attacker (1-4 HP range).
pub const THORNS_MIN_DAMAGE: f32 = 1.0;
pub const THORNS_MAX_DAMAGE: f32 = 4.0;

/// Sweeping Edge damage multiplier.
/// sweepDamage = baseDamage * level / (level + 1)
pub fn sweeping_edge_ratio(level: i32) -> f32 {
    level as f32 / (level as f32 + 1.0)
}

/// Fall damage reduction from Feather Falling.
/// Reduces by 12% per level (max 48% at level 4).
pub fn feather_falling_damage_reduction(level: i32, damage: f32) -> f32 {
    let reduction = (level * 12).min(80);
    damage * (1.0 - reduction as f32 / 100.0)
}

/// Looting/Fortune extra drop count (0 or more extra items).
/// For Fortune, the extra drops follow: 0 to level additional items (uniform).
/// For Looting, similar behavior but applies to mob drops.
pub fn fortune_extra_drops(level: i32, random_0_to_1: f64) -> i32 {
    if level <= 0 {
        return 0;
    }
    // Vanilla: random.nextInt(level + 1) extra drops
    (random_0_to_1 * (level + 1) as f64) as i32
}

/// Respiration extra underwater breathing time (ticks per level added to 300 base).
pub fn respiration_bonus_ticks(level: i32) -> i32 {
    level * 15 * 20 // 15 seconds per level at 20 ticks/second
}

/// Aqua Affinity removes the underwater mining speed penalty.
pub fn aqua_affinity_removes_underwater_penalty(level: i32) -> bool {
    level >= 1
}

/// Whether the Infinity enchantment prevents arrow consumption.
pub fn infinity_prevents_consumption(has_infinity: bool, has_arrow: bool) -> bool {
    has_infinity && has_arrow
}

/// Whether Curse of Vanishing destroys the item on death.
pub fn curse_of_vanishing_destroys_on_death(has_curse: bool) -> bool {
    has_curse
}

/// Mending repair amount: XP orbs absorbed → durability repaired (2 durability per XP).
pub fn mending_repair_from_xp(xp_absorbed: i32) -> i32 {
    xp_absorbed * 2
}

/// Power enchantment bonus damage for bows (base * (0.5 * level + 0.5) + 0.5).
pub fn power_arrow_bonus(level: i32, base_damage: f32) -> f32 {
    if level <= 0 {
        return 0.0;
    }
    base_damage * (0.5 * level as f32 + 0.5)
}

/// Impaling bonus damage: applies to mobs in water or rain.
pub fn impaling_bonus(level: i32, target_in_water_or_rain: bool) -> f32 {
    if !target_in_water_or_rain || level <= 0 {
        return 0.0;
    }
    level as f32 * 2.5
}

/// Depth Strider underwater movement speed multiplier.
/// Level 3 = full water movement speed (multiplies by 1/3 reduction per level).
pub fn depth_strider_speed_factor(level: i32) -> f32 {
    (level as f32 / 3.0).min(1.0)
}

/// Density mace enchantment: extra damage per block fallen before hitting.
pub fn density_smash_bonus_per_block(level: i32, blocks_fallen: f32) -> f32 {
    if level <= 0 {
        return 0.0;
    }
    level as f32 * 0.5 * blocks_fallen
}

/// FireAspect/Flame: seconds the target is set on fire (FireAspect = level * 4; Flame = 5).
pub fn fire_aspect_seconds_on_fire(level: i32) -> i32 {
    level * 4
}

pub fn flame_seconds_on_fire(level: i32) -> i32 {
    if level >= 1 {
        5
    } else {
        0
    }
}

/// FrostWalker: maximum radius in blocks around the player for freezing water.
/// Vanilla: blocks within distance level + 2 are checked.
pub fn frost_walker_radius(level: i32) -> i32 {
    level + 2
}

/// SoulSpeed: movement speed attribute bonus (multiplied onto base speed).
/// Vanilla attribute modifier value = 0.03 * amplifier.
pub fn soul_speed_attribute_bonus(level: i32) -> f64 {
    0.03 * level as f64
}

/// SwiftSneak: sneak speed modifier added per level.
/// Vanilla: add 0.15 per level to sneak speed modifier attribute.
pub fn swift_sneak_speed_modifier(level: i32) -> f64 {
    0.15 * level as f64
}

/// Loyalty: returns whether the thrown trident will return to the owner.
/// Any level >= 1 enables return; level affects return speed.
pub fn loyalty_enables_return(level: i32) -> bool {
    level >= 1
}

/// Channeling: returns whether the trident can summon a lightning bolt on hit.
/// Requires target exposed to open sky and weather is currently thundering.
pub fn channeling_can_strike(level: i32, target_in_open_air: bool, thundering: bool) -> bool {
    level >= 1 && target_in_open_air && thundering
}

/// Riptide: thrust velocity boost when throwing the trident in rain or water.
/// Vanilla linear scaling: 0.6 + 0.3 × level.
pub fn riptide_thrust_power(level: i32) -> f32 {
    0.6 + 0.3 * level as f32
}

/// MultiShot: extra projectiles fired beyond the first (always 2 extra = 3 total).
pub fn multishot_extra_projectiles() -> i32 {
    2
}

/// Piercing: total entities an arrow can pierce, including the first hit.
pub fn piercing_entity_limit(level: i32) -> i32 {
    (level + 1).max(1)
}

/// QuickCharge: tick reduction in crossbow charging time per level (vanilla: 5 ticks/level).
pub fn quick_charge_use_ticks_reduction(level: i32) -> i32 {
    5 * level
}

/// BindingCurse: whether the item can be removed from the armor slot.
/// Items with Binding Curse cannot be unequipped in survival; creative/spectator bypass.
pub fn binding_curse_can_remove(has_curse: bool, non_survival_mode: bool) -> bool {
    !has_curse || non_survival_mode
}

/// Breach: fraction of armor value to ignore when computing mace damage.
/// Vanilla: 0.15 per level, capped at 1.0.
pub fn breach_armor_reduction_fraction(level: i32) -> f32 {
    (0.15 * level as f32).min(1.0)
}

/// WindBurst: extra upward/knockback velocity applied on hit per level.
pub fn wind_burst_knockback_blocks(level: i32) -> f32 {
    level as f32
}

#[cfg(test)]
mod tests;
