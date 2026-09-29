//! Rename tables extracted from the Java source of `EntityTheRenameningFix` (generated data).

/// `EntityTheRenameningFix.RENAMED_IDS`.
pub const RENAMED_IDS: &[(&str, &str)] = &[
    (
        "minecraft:commandblock_minecart",
        "minecraft:command_block_minecart",
    ),
    ("minecraft:ender_crystal", "minecraft:end_crystal"),
    ("minecraft:snowman", "minecraft:snow_golem"),
    ("minecraft:evocation_illager", "minecraft:evoker"),
    ("minecraft:evocation_fangs", "minecraft:evoker_fangs"),
    ("minecraft:illusion_illager", "minecraft:illusioner"),
    ("minecraft:vindication_illager", "minecraft:vindicator"),
    ("minecraft:villager_golem", "minecraft:iron_golem"),
    ("minecraft:xp_orb", "minecraft:experience_orb"),
    ("minecraft:xp_bottle", "minecraft:experience_bottle"),
    ("minecraft:eye_of_ender_signal", "minecraft:eye_of_ender"),
    ("minecraft:fireworks_rocket", "minecraft:firework_rocket"),
];

/// `EntityTheRenameningFix.RENAMED_BLOCKS`.
pub const RENAMED_BLOCKS: &[(&str, &str)] = &[
    ("minecraft:portal", "minecraft:nether_portal"),
    ("minecraft:oak_bark", "minecraft:oak_wood"),
    ("minecraft:spruce_bark", "minecraft:spruce_wood"),
    ("minecraft:birch_bark", "minecraft:birch_wood"),
    ("minecraft:jungle_bark", "minecraft:jungle_wood"),
    ("minecraft:acacia_bark", "minecraft:acacia_wood"),
    ("minecraft:dark_oak_bark", "minecraft:dark_oak_wood"),
    ("minecraft:stripped_oak_bark", "minecraft:stripped_oak_wood"),
    (
        "minecraft:stripped_spruce_bark",
        "minecraft:stripped_spruce_wood",
    ),
    (
        "minecraft:stripped_birch_bark",
        "minecraft:stripped_birch_wood",
    ),
    (
        "minecraft:stripped_jungle_bark",
        "minecraft:stripped_jungle_wood",
    ),
    (
        "minecraft:stripped_acacia_bark",
        "minecraft:stripped_acacia_wood",
    ),
    (
        "minecraft:stripped_dark_oak_bark",
        "minecraft:stripped_dark_oak_wood",
    ),
    ("minecraft:mob_spawner", "minecraft:spawner"),
];

/// The entries `RENAMED_ITEMS` adds to `RENAMED_BLOCKS`.
pub const RENAMED_ITEMS_EXTRA: &[(&str, &str)] = &[
    ("minecraft:clownfish", "minecraft:tropical_fish"),
    (
        "minecraft:chorus_fruit_popped",
        "minecraft:popped_chorus_fruit",
    ),
    (
        "minecraft:evocation_illager_spawn_egg",
        "minecraft:evoker_spawn_egg",
    ),
    (
        "minecraft:vindication_illager_spawn_egg",
        "minecraft:vindicator_spawn_egg",
    ),
];
