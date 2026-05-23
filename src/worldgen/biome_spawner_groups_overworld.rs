use super::*;

pub const PLAINS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: PLAINS_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: PLAINS_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const FOREST_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: FOREST_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const BIRCH_FOREST_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: BIRCH_FOREST_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const FLOWER_FOREST_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: FLOWER_FOREST_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const MUSHROOM_FIELDS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: MUSHROOM_FIELDS_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const BADLANDS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: BADLANDS_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const WOODED_BADLANDS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: WOODED_BADLANDS_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const MEADOW_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: MEADOW_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const CHERRY_GROVE_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: CHERRY_GROVE_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const PALE_GARDEN_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const GROVE_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: GROVE_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const SNOWY_SLOPES_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: SNOWY_SLOPES_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const FROZEN_PEAKS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: GOAT_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const STONY_PEAKS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const WINDSWEPT_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: WINDSWEPT_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const DESERT_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: DESERT_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: DESERT_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const SAVANNA_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: SAVANNA_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: PLAINS_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const SAVANNA_PLATEAU_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: SAVANNA_PLATEAU_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: PLAINS_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const TAIGA_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: TAIGA_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const OLD_GROWTH_PINE_TAIGA_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: TAIGA_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: OLD_GROWTH_PINE_TAIGA_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const SNOWY_PLAINS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: SNOWY_PLAINS_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: SNOWY_PLAINS_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const JUNGLE_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: JUNGLE_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: JUNGLE_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const SPARSE_JUNGLE_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: SPARSE_JUNGLE_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const BAMBOO_JUNGLE_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: BAMBOO_JUNGLE_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: JUNGLE_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const SWAMP_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: SWAMP_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: SWAMP_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const MANGROVE_SWAMP_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: MANGROVE_SWAMP_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: SWAMP_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: MANGROVE_SWAMP_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const LUSH_CAVES_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: LUSH_CAVES_AXOLOTL_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: MANGROVE_SWAMP_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const DRIPSTONE_CAVES_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: DRIPSTONE_CAVES_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

