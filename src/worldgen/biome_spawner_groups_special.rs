use super::*;

pub const DEEP_DARK_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
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
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const NETHER_WASTES_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: NETHER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: NETHER_WASTES_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const CRIMSON_FOREST_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: NETHER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: CRIMSON_FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const WARPED_FOREST_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: NETHER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: WARPED_FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const SOUL_SAND_VALLEY_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: NETHER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: SOUL_SAND_VALLEY_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const BASALT_DELTAS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: NETHER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: BASALT_DELTAS_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const END_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
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
        entries: END_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const THE_VOID_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: &[],
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
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: &[],
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

pub const WARPED_FOREST_SPAWN_COSTS: &[MobSpawnCostModel] = &[MobSpawnCostModel {
    entity_type: "minecraft:enderman",
    energy_budget: 0.12,
    charge: 1.0,
}];

pub const SOUL_SAND_VALLEY_SPAWN_COSTS: &[MobSpawnCostModel] = &[
    MobSpawnCostModel {
        entity_type: "minecraft:enderman",
        energy_budget: 0.15,
        charge: 0.7,
    },
    MobSpawnCostModel {
        entity_type: "minecraft:ghast",
        energy_budget: 0.15,
        charge: 0.7,
    },
    MobSpawnCostModel {
        entity_type: "minecraft:skeleton",
        energy_budget: 0.15,
        charge: 0.7,
    },
    MobSpawnCostModel {
        entity_type: "minecraft:strider",
        energy_budget: 0.15,
        charge: 0.7,
    },
];

pub const BEACH_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: BEACH_CREATURE_SPAWNS,
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

pub const RIVER_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: RIVER_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: RIVER_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: RIVER_WATER_CREATURE_SPAWNS,
    },
];

pub const FROZEN_RIVER_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: FROZEN_RIVER_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: FROZEN_RIVER_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: FROZEN_RIVER_WATER_CREATURE_SPAWNS,
    },
];

pub const OCEAN_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: OCEAN_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: OCEAN_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: OCEAN_WATER_CREATURE_SPAWNS,
    },
];

pub const COLD_OCEAN_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: OCEAN_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: COLD_OCEAN_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: COLD_OCEAN_WATER_CREATURE_SPAWNS,
    },
];

pub const LUKEWARM_OCEAN_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: OCEAN_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: LUKEWARM_OCEAN_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: LUKEWARM_OCEAN_WATER_CREATURE_SPAWNS,
    },
];

pub const DEEP_LUKEWARM_OCEAN_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: OCEAN_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: DEEP_LUKEWARM_OCEAN_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: DEEP_LUKEWARM_OCEAN_WATER_CREATURE_SPAWNS,
    },
];

pub const DEEP_FROZEN_OCEAN_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: DEEP_FROZEN_OCEAN_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: OCEAN_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: DEEP_FROZEN_OCEAN_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: DEEP_FROZEN_OCEAN_WATER_CREATURE_SPAWNS,
    },
];

pub const WARM_OCEAN_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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
        entries: WARM_OCEAN_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: WARM_OCEAN_WATER_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: WARM_OCEAN_WATER_CREATURE_SPAWNS,
    },
];

pub const SNOWY_BEACH_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
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

pub const OVERWORLD_COMMON_CARVERS: &[&str] = &[
    "minecraft:cave",
    "minecraft:cave_extra_underground",
    "minecraft:canyon",
];

pub const NETHER_COMMON_CARVERS: &[&str] = &["minecraft:nether_cave"];
