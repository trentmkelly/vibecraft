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

const fn ench(
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
) -> EnchantmentDef {
    EnchantmentDef {
        id,
        supported_items,
        primary_items: None,
        weight,
        max_level,
        min_cost,
        max_cost,
        anvil_cost,
        slots,
        groups,
        hooks,
    }
}

pub const ENCHANTMENTS: &[EnchantmentDef] = &[
    ench(
        "minecraft:protection",
        "#minecraft:armor_enchantable",
        10,
        4,
        Cost::dynamic(1, 11),
        Cost::dynamic(12, 11),
        1,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_PROTECTION,
    ),
    ench(
        "minecraft:fire_protection",
        "#minecraft:armor_enchantable",
        5,
        4,
        Cost::dynamic(10, 8),
        Cost::dynamic(18, 8),
        2,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_ATTR,
    ),
    ench(
        "minecraft:feather_falling",
        "#minecraft:foot_armor_enchantable",
        5,
        4,
        Cost::dynamic(5, 6),
        Cost::dynamic(11, 6),
        2,
        FEET,
        EMPTY_GROUPS,
        DAMAGE_PROTECTION,
    ),
    ench(
        "minecraft:blast_protection",
        "#minecraft:armor_enchantable",
        2,
        4,
        Cost::dynamic(5, 8),
        Cost::dynamic(13, 8),
        4,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_ATTR,
    ),
    ench(
        "minecraft:projectile_protection",
        "#minecraft:armor_enchantable",
        5,
        4,
        Cost::dynamic(3, 6),
        Cost::dynamic(9, 6),
        2,
        ARMOR,
        ARMOR_GROUP,
        DAMAGE_PROTECTION,
    ),
    ench(
        "minecraft:respiration",
        "#minecraft:head_armor_enchantable",
        2,
        3,
        Cost::dynamic(10, 10),
        Cost::dynamic(40, 10),
        4,
        HEAD,
        EMPTY_GROUPS,
        EMPTY_HOOKS,
    ),
    ench(
        "minecraft:aqua_affinity",
        "#minecraft:head_armor_enchantable",
        2,
        1,
        Cost::constant(1),
        Cost::constant(41),
        4,
        HEAD,
        EMPTY_GROUPS,
        EMPTY_HOOKS,
    ),
    ench(
        "minecraft:thorns",
        "#minecraft:armor_enchantable",
        1,
        3,
        Cost::dynamic(10, 20),
        Cost::dynamic(60, 20),
        8,
        ARMOR,
        EMPTY_GROUPS,
        DAMAGE_POST,
    ),
    ench(
        "minecraft:depth_strider",
        "#minecraft:foot_armor_enchantable",
        2,
        3,
        Cost::dynamic(10, 10),
        Cost::dynamic(25, 10),
        4,
        FEET,
        BOOTS_GROUP,
        MOVEMENT,
    ),
    ench(
        "minecraft:frost_walker",
        "#minecraft:foot_armor_enchantable",
        2,
        2,
        Cost::dynamic(10, 10),
        Cost::dynamic(25, 10),
        4,
        FEET,
        BOOTS_GROUP,
        MOVEMENT,
    ),
    ench(
        "minecraft:binding_curse",
        "#minecraft:equippable_enchantable",
        1,
        1,
        Cost::constant(25),
        Cost::constant(50),
        8,
        ANY,
        CURSE_GROUP,
        PREVENT_DROP,
    ),
    ench(
        "minecraft:soul_speed",
        "#minecraft:foot_armor_enchantable",
        1,
        3,
        Cost::dynamic(10, 10),
        Cost::dynamic(25, 10),
        8,
        FEET,
        EMPTY_GROUPS,
        MOVEMENT,
    ),
    ench(
        "minecraft:swift_sneak",
        "#minecraft:leg_armor_enchantable",
        1,
        3,
        Cost::dynamic(25, 25),
        Cost::dynamic(75, 25),
        8,
        &[EquipmentSlotGroup::Legs],
        EMPTY_GROUPS,
        MOVEMENT,
    ),
    ench(
        "minecraft:sharpness",
        "#minecraft:weapon_enchantable",
        10,
        5,
        Cost::dynamic(1, 11),
        Cost::dynamic(21, 11),
        1,
        MAIN_HAND,
        WEAPON_GROUP,
        DAMAGE,
    ),
    ench(
        "minecraft:smite",
        "#minecraft:weapon_enchantable",
        5,
        5,
        Cost::dynamic(5, 8),
        Cost::dynamic(25, 8),
        2,
        MAIN_HAND,
        WEAPON_GROUP,
        DAMAGE,
    ),
    ench(
        "minecraft:bane_of_arthropods",
        "#minecraft:weapon_enchantable",
        5,
        5,
        Cost::dynamic(5, 8),
        Cost::dynamic(25, 8),
        2,
        MAIN_HAND,
        WEAPON_GROUP,
        DAMAGE_POST,
    ),
    ench(
        "minecraft:knockback",
        "#minecraft:weapon_enchantable",
        5,
        2,
        Cost::dynamic(5, 20),
        Cost::dynamic(55, 20),
        2,
        MAIN_HAND,
        EMPTY_GROUPS,
        KNOCKBACK,
    ),
    ench(
        "minecraft:fire_aspect",
        "#minecraft:weapon_enchantable",
        2,
        2,
        Cost::dynamic(10, 20),
        Cost::dynamic(60, 20),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE_POST,
    ),
    ench(
        "minecraft:looting",
        "#minecraft:weapon_enchantable",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        LOOT,
    ),
    ench(
        "minecraft:sweeping_edge",
        "#minecraft:sword_enchantable",
        2,
        3,
        Cost::dynamic(5, 9),
        Cost::dynamic(20, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE,
    ),
    ench(
        "minecraft:efficiency",
        "#minecraft:mining_enchantable",
        10,
        5,
        Cost::dynamic(1, 10),
        Cost::dynamic(51, 10),
        1,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::Attributes],
    ),
    ench(
        "minecraft:silk_touch",
        "#minecraft:mining_enchantable",
        1,
        1,
        Cost::constant(15),
        Cost::constant(65),
        8,
        MAIN_HAND,
        MINING_GROUP,
        &[EnchantmentEffectHook::BlockExperience],
    ),
    ench(
        "minecraft:unbreaking",
        "#minecraft:durability_enchantable",
        5,
        3,
        Cost::dynamic(5, 8),
        Cost::dynamic(55, 8),
        2,
        ANY,
        EMPTY_GROUPS,
        ITEM_DAMAGE,
    ),
    ench(
        "minecraft:fortune",
        "#minecraft:mining_enchantable",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        MINING_GROUP,
        LOOT,
    ),
    ench(
        "minecraft:power",
        "#minecraft:bow_enchantable",
        10,
        5,
        Cost::dynamic(1, 10),
        Cost::dynamic(16, 10),
        1,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE,
    ),
    ench(
        "minecraft:punch",
        "#minecraft:bow_enchantable",
        2,
        2,
        Cost::dynamic(12, 20),
        Cost::dynamic(37, 20),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        KNOCKBACK,
    ),
    ench(
        "minecraft:flame",
        "#minecraft:bow_enchantable",
        2,
        1,
        Cost::constant(20),
        Cost::constant(50),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE_POST,
    ),
    ench(
        "minecraft:infinity",
        "#minecraft:bow_enchantable",
        1,
        1,
        Cost::constant(20),
        Cost::constant(50),
        8,
        MAIN_HAND,
        BOW_GROUP,
        &[EnchantmentEffectHook::AmmoUse],
    ),
    ench(
        "minecraft:luck_of_the_sea",
        "#minecraft:fishing_enchantable",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::FishingLuckBonus],
    ),
    ench(
        "minecraft:lure",
        "#minecraft:fishing_enchantable",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::FishingTimeReduction],
    ),
    ench(
        "minecraft:loyalty",
        "#minecraft:trident_enchantable",
        5,
        3,
        Cost::dynamic(5, 7),
        Cost::dynamic(50, 7),
        2,
        MAIN_HAND,
        TRIDENT_GROUP,
        TRIDENT,
    ),
    ench(
        "minecraft:impaling",
        "#minecraft:trident_enchantable",
        2,
        5,
        Cost::dynamic(1, 8),
        Cost::dynamic(21, 8),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        DAMAGE,
    ),
    ench(
        "minecraft:riptide",
        "#minecraft:trident_enchantable",
        2,
        3,
        Cost::dynamic(10, 7),
        Cost::dynamic(25, 7),
        4,
        MAIN_HAND,
        TRIDENT_GROUP,
        TRIDENT,
    ),
    ench(
        "minecraft:channeling",
        "#minecraft:trident_enchantable",
        1,
        1,
        Cost::constant(25),
        Cost::constant(50),
        8,
        MAIN_HAND,
        TRIDENT_GROUP,
        DAMAGE_POST,
    ),
    ench(
        "minecraft:multishot",
        "#minecraft:crossbow_enchantable",
        2,
        1,
        Cost::constant(20),
        Cost::constant(50),
        4,
        MAIN_HAND,
        CROSSBOW_GROUP,
        PROJECTILE,
    ),
    ench(
        "minecraft:quick_charge",
        "#minecraft:crossbow_enchantable",
        5,
        3,
        Cost::dynamic(12, 20),
        Cost::dynamic(50, 20),
        2,
        MAIN_HAND,
        EMPTY_GROUPS,
        CROSSBOW,
    ),
    ench(
        "minecraft:piercing",
        "#minecraft:crossbow_enchantable",
        10,
        4,
        Cost::dynamic(1, 10),
        Cost::dynamic(50, 10),
        1,
        MAIN_HAND,
        CROSSBOW_GROUP,
        &[EnchantmentEffectHook::ProjectilePiercing],
    ),
    ench(
        "minecraft:density",
        "#minecraft:mace_enchantable",
        5,
        5,
        Cost::dynamic(5, 8),
        Cost::dynamic(25, 8),
        2,
        MAIN_HAND,
        MACE_GROUP,
        &[EnchantmentEffectHook::SmashDamagePerFallenBlock],
    ),
    ench(
        "minecraft:breach",
        "#minecraft:mace_enchantable",
        2,
        4,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        MACE_GROUP,
        &[EnchantmentEffectHook::ArmorEffectiveness],
    ),
    ench(
        "minecraft:wind_burst",
        "#minecraft:mace_enchantable",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        MACE_GROUP,
        &[EnchantmentEffectHook::PostAttack],
    ),
    ench(
        "minecraft:lunge",
        "#minecraft:weapon_enchantable",
        2,
        3,
        Cost::dynamic(15, 9),
        Cost::dynamic(65, 9),
        4,
        MAIN_HAND,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::Attributes],
    ),
    ench(
        "minecraft:mending",
        "#minecraft:durability_enchantable",
        2,
        1,
        Cost::constant(25),
        Cost::constant(75),
        4,
        ANY,
        EMPTY_GROUPS,
        &[EnchantmentEffectHook::RepairWithXp],
    ),
    ench(
        "minecraft:vanishing_curse",
        "#minecraft:vanishing_enchantable",
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

/// Probability that Unbreaking prevents durability damage.
/// Armor: 60% + (40% / (level + 1)), Tools: 100% / (level + 1)
pub fn unbreaking_durability_skip_chance(level: i32, is_armor: bool) -> f32 {
    if level <= 0 {
        return 0.0;
    }
    if is_armor {
        1.0 - 1.0 / (level + 1) as f32
    } else {
        1.0 - 1.0 / (level + 1) as f32
    }
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
mod tests {
    use super::*;

    #[test]
    fn enchantment_registry_and_effect_component_surface_match_decompiled_keys() {
        assert_eq!(ENCHANTMENTS.len(), 43);
        assert_eq!(ENCHANTMENTS[0].id, "minecraft:protection");
        assert_eq!(ENCHANTMENTS.last().unwrap().id, "minecraft:vanishing_curse");
        assert_eq!(ENCHANTMENT_EFFECT_COMPONENTS.len(), 31);
        assert!(ENCHANTMENT_EFFECT_COMPONENTS.contains(&EnchantmentEffectHook::DamageProtection));
        assert!(ENCHANTMENT_EFFECT_COMPONENTS.contains(&EnchantmentEffectHook::RepairWithXp));
        assert_eq!(PROVIDER_TYPES.len(), 3);
        assert_eq!(VANILLA_PROVIDERS.len(), 7);
    }

    #[test]
    fn costs_slots_weights_and_compatibility_follow_enchantment_definition_contracts() {
        let protection = enchantment("protection").unwrap();
        assert_eq!(protection.weight, 10);
        assert_eq!(protection.max_level, 4);
        assert_eq!(protection.min_cost.calculate(3), 23);
        assert_eq!(protection.max_cost.calculate(3), 34);

        let fire = enchantment("fire_protection").unwrap();
        let blast = enchantment("blast_protection").unwrap();
        assert!(!are_compatible(fire, blast));
        assert!(are_compatible(fire, enchantment("thorns").unwrap()));
        assert!(!are_compatible(
            enchantment("sharpness").unwrap(),
            enchantment("smite").unwrap()
        ));
        assert!(!are_compatible(
            enchantment("silk_touch").unwrap(),
            enchantment("fortune").unwrap()
        ));
    }

    #[test]
    fn representative_damage_mining_movement_loot_and_post_attack_hooks_are_modeled() {
        assert_eq!(damage_bonus("minecraft:sharpness", 5, None), 3.0);
        assert_eq!(damage_bonus("minecraft:smite", 2, Some("undead")), 5.0);
        assert_eq!(damage_bonus("minecraft:impaling", 3, Some("aquatic")), 7.5);
        assert_eq!(
            protection_bonus("minecraft:feather_falling", 4, "is_fall"),
            12.0
        );
        assert_eq!(
            protection_bonus("minecraft:projectile_protection", 4, "is_projectile"),
            8.0
        );

        assert!(enchantment("efficiency")
            .unwrap()
            .hooks
            .contains(&EnchantmentEffectHook::Attributes));
        assert!(enchantment("fortune")
            .unwrap()
            .hooks
            .contains(&EnchantmentEffectHook::BlockExperience));
        assert!(enchantment("depth_strider")
            .unwrap()
            .hooks
            .contains(&EnchantmentEffectHook::LocationChanged));
        assert!(enchantment("thorns")
            .unwrap()
            .hooks
            .contains(&EnchantmentEffectHook::PostAttack));
        assert!(enchantment("unbreaking")
            .unwrap()
            .hooks
            .contains(&EnchantmentEffectHook::ItemDamage));
    }

    #[test]
    fn protection_damage_reduction_caps_at_80_percent() {
        // Prot I = 4% reduction
        let dmg = protection_damage_reduction(1, 10.0);
        assert!((dmg - 9.6).abs() < 0.01);

        // Prot IV full armor (max per piece is 20, total realistic max 80%)
        let dmg = protection_damage_reduction(20, 10.0);
        assert!((dmg - 2.0).abs() < 0.01);

        // Over cap (100 total) clamped to 80%
        let dmg = protection_damage_reduction(25, 10.0);
        assert!((dmg - 2.0).abs() < 0.01);
    }

    #[test]
    fn sharpness_smite_bane_and_knockback_bonuses_match_vanilla_formulas() {
        // Sharpness I: 0.5*1 + 0.5 = 1.0
        assert!((sharpness_bonus(1) - 1.0).abs() < 0.001);
        // Sharpness V: 0.5*5 + 0.5 = 3.0
        assert!((sharpness_bonus(5) - 3.0).abs() < 0.001);
        assert_eq!(sharpness_bonus(0), 0.0);

        // Smite V on undead: 5 * 2.5 = 12.5
        assert!((smite_bonus(5, true) - 12.5).abs() < 0.001);
        // Smite V on non-undead: 0
        assert_eq!(smite_bonus(5, false), 0.0);

        // Bane IV on arthropod: 4 * 2.5 = 10.0
        assert!((bane_of_arthropods_bonus(4, true) - 10.0).abs() < 0.001);

        // Knockback II: 2 * 3 = 6 blocks
        assert!((knockback_bonus_blocks(2) - 6.0).abs() < 0.001);
    }

    #[test]
    fn efficiency_speed_bonus_matches_vanilla_formula() {
        // Efficiency I: 1*1+1 = 2.0 added speed
        assert!((efficiency_speed_bonus(1) - 2.0).abs() < 0.001);
        // Efficiency V: 5*5+1 = 26.0
        assert!((efficiency_speed_bonus(5) - 26.0).abs() < 0.001);
        assert_eq!(efficiency_speed_bonus(0), 0.0);
    }

    #[test]
    fn sweeping_edge_ratio_matches_vanilla_formula() {
        // Level 1: 1/(1+1) = 0.5
        assert!((sweeping_edge_ratio(1) - 0.5).abs() < 0.001);
        // Level 3: 3/(3+1) = 0.75
        assert!((sweeping_edge_ratio(3) - 0.75).abs() < 0.001);
    }

    #[test]
    fn feather_falling_reduces_fall_damage_per_level() {
        // Level 4: 12*4=48% reduction
        let dmg = feather_falling_damage_reduction(4, 10.0);
        assert!((dmg - 5.2).abs() < 0.01);
        // Level 1: 12% reduction
        let dmg = feather_falling_damage_reduction(1, 10.0);
        assert!((dmg - 8.8).abs() < 0.01);
    }

    #[test]
    fn mending_power_impaling_depth_strider_follow_vanilla() {
        assert_eq!(mending_repair_from_xp(5), 10);
        assert!((power_arrow_bonus(1, 6.0) - 6.0).abs() < 0.001); // 6 * (0.5+0.5) = 6
        assert!((impaling_bonus(3, true) - 7.5).abs() < 0.001); // 3*2.5 = 7.5
        assert_eq!(impaling_bonus(3, false), 0.0);
        assert!((depth_strider_speed_factor(3) - 1.0).abs() < 0.001);
        assert!((depth_strider_speed_factor(1) - 0.333).abs() < 0.01);
    }

    #[test]
    fn thorns_and_fortune_and_infinity_match_vanilla_behavior() {
        assert!((thorns_activation_chance(1) - 0.15).abs() < 0.001);
        assert!((thorns_activation_chance(4) - 0.60).abs() < 0.001);
        assert_eq!(fortune_extra_drops(3, 0.99), 3); // 0.99 * 4 = 3
        assert_eq!(fortune_extra_drops(3, 0.0), 0);
        assert!(infinity_prevents_consumption(true, true));
        assert!(!infinity_prevents_consumption(false, true));
        assert!(!infinity_prevents_consumption(true, false));
        assert!(curse_of_vanishing_destroys_on_death(true));
        assert!(!curse_of_vanishing_destroys_on_death(false));
    }

    #[test]
    fn fire_aspect_frost_walker_soul_speed_swift_sneak_match_vanilla() {
        // FireAspect II = 8 seconds on fire
        assert_eq!(fire_aspect_seconds_on_fire(2), 8);
        assert_eq!(fire_aspect_seconds_on_fire(1), 4);
        assert_eq!(fire_aspect_seconds_on_fire(0), 0);
        // FrostWalker II: radius 4 blocks
        assert_eq!(frost_walker_radius(2), 4);
        assert_eq!(frost_walker_radius(1), 3);
        // SoulSpeed III: 0.09 attribute bonus
        assert!((soul_speed_attribute_bonus(3) - 0.09).abs() < 0.001);
        // SwiftSneak III: 0.45 speed modifier
        assert!((swift_sneak_speed_modifier(3) - 0.45).abs() < 0.001);
    }

    #[test]
    fn channeling_riptide_multishot_quickcharge_binding_breach_wind_burst_match_vanilla() {
        // Channeling only fires in thunderstorm with open sky
        assert!(channeling_can_strike(1, true, true));
        assert!(!channeling_can_strike(1, false, true));
        assert!(!channeling_can_strike(1, true, false));
        assert!(!channeling_can_strike(0, true, true));
        // Riptide I: power = 0.9
        assert!((riptide_thrust_power(1) - 0.9).abs() < 0.001);
        // Riptide III: power = 1.5
        assert!((riptide_thrust_power(3) - 1.5).abs() < 0.001);
        // MultiShot always gives 2 extra (3 total)
        assert_eq!(multishot_extra_projectiles(), 2);
        // QuickCharge III reduces 15 ticks
        assert_eq!(quick_charge_use_ticks_reduction(3), 15);
        // BindingCurse: survival cannot remove, creative can
        assert!(!binding_curse_can_remove(true, false));
        assert!(binding_curse_can_remove(true, true));
        assert!(binding_curse_can_remove(false, false));
        // Breach IV: 60% armor reduction
        assert!((breach_armor_reduction_fraction(4) - 0.6).abs() < 0.001);
        // WindBurst III: 3.0 blocks knockback
        assert!((wind_burst_knockback_blocks(3) - 3.0).abs() < 0.001);
    }

    #[test]
    fn loyalty_enables_return_at_any_level() {
        assert!(loyalty_enables_return(1));
        assert!(loyalty_enables_return(3));
        assert!(!loyalty_enables_return(0));
    }

    #[test]
    fn providers_cover_spawn_raid_and_loot_selection_paths() {
        assert_eq!(
            VANILLA_PROVIDERS[0].provider_type,
            ProviderType::ByCostWithDifficulty
        );
        assert_eq!(VANILLA_PROVIDERS[0].min_cost, 5);
        assert_eq!(VANILLA_PROVIDERS[0].max_cost, 17);
        assert_eq!(VANILLA_PROVIDERS[1].enchantment, Some("minecraft:piercing"));
        assert_eq!(
            VANILLA_PROVIDERS[6].enchantment,
            Some("minecraft:silk_touch")
        );
    }
}
