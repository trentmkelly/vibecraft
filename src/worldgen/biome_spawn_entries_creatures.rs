use super::*;

pub const PLAINS_AMBIENT_SPAWNS: &[MobSpawnerDataModel] = &[MobSpawnerDataModel {
    entity_type: "minecraft:bat",
    weight: 10,
    min_count: 8,
    max_count: 8,
}];

pub const PLAINS_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:horse",
        weight: 5,
        min_count: 2,
        max_count: 6,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:donkey",
        weight: 1,
        min_count: 1,
        max_count: 3,
    },
];

pub const BEACH_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[MobSpawnerDataModel {
    entity_type: "minecraft:turtle",
    weight: 5,
    min_count: 2,
    max_count: 5,
}];

pub const MUSHROOM_FIELDS_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[MobSpawnerDataModel {
    entity_type: "minecraft:mooshroom",
    weight: 8,
    min_count: 4,
    max_count: 8,
}];

pub const DEEP_FROZEN_OCEAN_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[MobSpawnerDataModel {
    entity_type: "minecraft:polar_bear",
    weight: 1,
    min_count: 1,
    max_count: 2,
}];

pub const BADLANDS_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:armadillo",
        weight: 6,
        min_count: 1,
        max_count: 2,
    },
];

pub const WOODED_BADLANDS_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:armadillo",
        weight: 6,
        min_count: 1,
        max_count: 2,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:wolf",
        weight: 2,
        min_count: 4,
        max_count: 8,
    },
];

pub const MEADOW_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:donkey",
        weight: 1,
        min_count: 1,
        max_count: 2,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 2,
        min_count: 2,
        max_count: 6,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 2,
        min_count: 2,
        max_count: 4,
    },
];

pub const CHERRY_GROVE_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 1,
        min_count: 1,
        max_count: 2,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 2,
        min_count: 2,
        max_count: 6,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 2,
        min_count: 2,
        max_count: 4,
    },
];

pub const GROVE_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:wolf",
        weight: 1,
        min_count: 1,
        max_count: 1,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 8,
        min_count: 2,
        max_count: 3,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:fox",
        weight: 4,
        min_count: 2,
        max_count: 4,
    },
];

pub const SNOWY_SLOPES_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 4,
        min_count: 2,
        max_count: 3,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:goat",
        weight: 5,
        min_count: 1,
        max_count: 3,
    },
];

pub const GOAT_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[MobSpawnerDataModel {
    entity_type: "minecraft:goat",
    weight: 5,
    min_count: 1,
    max_count: 3,
}];

pub const WINDSWEPT_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:llama",
        weight: 5,
        min_count: 4,
        max_count: 6,
    },
];

pub const DESERT_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 12,
        min_count: 2,
        max_count: 3,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:camel",
        weight: 1,
        min_count: 1,
        max_count: 1,
    },
];

pub const SAVANNA_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:horse",
        weight: 1,
        min_count: 2,
        max_count: 6,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:donkey",
        weight: 1,
        min_count: 1,
        max_count: 1,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:armadillo",
        weight: 10,
        min_count: 2,
        max_count: 3,
    },
];

pub const SAVANNA_PLATEAU_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:horse",
        weight: 1,
        min_count: 2,
        max_count: 6,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:donkey",
        weight: 1,
        min_count: 1,
        max_count: 1,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:armadillo",
        weight: 10,
        min_count: 2,
        max_count: 3,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:llama",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:wolf",
        weight: 8,
        min_count: 4,
        max_count: 8,
    },
];

pub const TAIGA_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:wolf",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 4,
        min_count: 2,
        max_count: 3,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:fox",
        weight: 8,
        min_count: 2,
        max_count: 4,
    },
];

pub const SNOWY_PLAINS_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 10,
        min_count: 2,
        max_count: 3,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:polar_bear",
        weight: 1,
        min_count: 1,
        max_count: 2,
    },
];

pub const JUNGLE_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:parrot",
        weight: 40,
        min_count: 1,
        max_count: 2,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:panda",
        weight: 1,
        min_count: 1,
        max_count: 2,
    },
];

pub const SPARSE_JUNGLE_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:wolf",
        weight: 8,
        min_count: 2,
        max_count: 4,
    },
];

pub const BAMBOO_JUNGLE_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:parrot",
        weight: 40,
        min_count: 1,
        max_count: 2,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:panda",
        weight: 80,
        min_count: 1,
        max_count: 2,
    },
];

pub const SWAMP_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:frog",
        weight: 10,
        min_count: 2,
        max_count: 5,
    },
];

pub const MANGROVE_SWAMP_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[MobSpawnerDataModel {
    entity_type: "minecraft:frog",
    weight: 10,
    min_count: 2,
    max_count: 5,
}];

pub const FOREST_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:wolf",
        weight: 5,
        min_count: 4,
        max_count: 4,
    },
];

pub const BIRCH_FOREST_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
];

pub const FLOWER_FOREST_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:rabbit",
        weight: 4,
        min_count: 2,
        max_count: 3,
    },
];

