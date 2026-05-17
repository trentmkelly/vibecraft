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
