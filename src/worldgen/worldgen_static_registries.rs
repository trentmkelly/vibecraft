use super::*;

pub const WORLDGEN_TYPE_REGISTRIES: &[WorldgenTypeRegistry] = &[
    WorldgenTypeRegistry {
        id: "minecraft:height_provider_type",
        entries: &[
            "minecraft:constant",
            "minecraft:uniform",
            "minecraft:biased_to_bottom",
            "minecraft:very_biased_to_bottom",
            "minecraft:trapezoid",
            "minecraft:weighted_list",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:block_predicate_type",
        entries: &[
            "minecraft:matching_blocks",
            "minecraft:matching_block_tag",
            "minecraft:matching_fluids",
            "minecraft:has_sturdy_face",
            "minecraft:solid",
            "minecraft:replaceable",
            "minecraft:would_survive",
            "minecraft:inside_world_bounds",
            "minecraft:any_of",
            "minecraft:all_of",
            "minecraft:not",
            "minecraft:true",
            "minecraft:unobstructed",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:placement_modifier_type",
        entries: &[
            "minecraft:block_predicate_filter",
            "minecraft:rarity_filter",
            "minecraft:surface_water_depth_filter",
            "minecraft:biome",
            "minecraft:count",
            "minecraft:noise_based_count",
            "minecraft:noise_threshold_count",
            "minecraft:count_on_every_layer",
            "minecraft:environment_scan",
            "minecraft:heightmap",
            "minecraft:height_range",
            "minecraft:in_square",
            "minecraft:random_offset",
            "minecraft:fixed_placement",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:trunk_placer_type",
        entries: &[
            "minecraft:straight_trunk_placer",
            "minecraft:forking_trunk_placer",
            "minecraft:giant_trunk_placer",
            "minecraft:mega_jungle_trunk_placer",
            "minecraft:dark_oak_trunk_placer",
            "minecraft:fancy_trunk_placer",
            "minecraft:bending_trunk_placer",
            "minecraft:upwards_branching_trunk_placer",
            "minecraft:cherry_trunk_placer",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:foliage_placer_type",
        entries: &[
            "minecraft:blob_foliage_placer",
            "minecraft:spruce_foliage_placer",
            "minecraft:pine_foliage_placer",
            "minecraft:acacia_foliage_placer",
            "minecraft:bush_foliage_placer",
            "minecraft:fancy_foliage_placer",
            "minecraft:jungle_foliage_placer",
            "minecraft:mega_pine_foliage_placer",
            "minecraft:dark_oak_foliage_placer",
            "minecraft:random_spread_foliage_placer",
            "minecraft:cherry_foliage_placer",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:block_state_provider_type",
        entries: &[
            "minecraft:simple_state_provider",
            "minecraft:weighted_state_provider",
            "minecraft:noise_threshold_provider",
            "minecraft:noise_provider",
            "minecraft:dual_noise_provider",
            "minecraft:rotated_block_provider",
            "minecraft:randomized_int_state_provider",
            "minecraft:rule_based_state_provider",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:tree_decorator_type",
        entries: &[
            "minecraft:trunk_vine",
            "minecraft:leave_vine",
            "minecraft:pale_moss",
            "minecraft:creaking_heart",
            "minecraft:cocoa",
            "minecraft:beehive",
            "minecraft:alter_ground",
            "minecraft:attached_to_leaves",
            "minecraft:place_on_ground",
            "minecraft:attached_to_logs",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:feature_size_type",
        entries: &[
            "minecraft:two_layers_feature_size",
            "minecraft:three_layers_feature_size",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:root_placer_type",
        entries: &["minecraft:mangrove_root_placer"],
    },
];

pub const FEATURE_BEHAVIOR_MODELS: &[FeatureBehaviorModel] = &[
    FeatureBehaviorModel { feature_type: "minecraft:tree", behavior: "validates roots/trunk/foliage/decorators, computes bounding box, updates leaves", success_condition: "at least one trunk placer node and all bounding-box updates succeed" },
    FeatureBehaviorModel { feature_type: "minecraft:vegetation_patch", behavior: "samples xz radius, validates replaceable ground, places depth columns, optionally places vegetation", success_condition: "at least one ground block or vegetation feature is placed" },
    FeatureBehaviorModel { feature_type: "minecraft:spring_feature", behavior: "requires valid block above, optional valid block below, exact adjacent rock and hole counts", success_condition: "rockCount == config.rockCount && holeCount == config.holeCount" },
    FeatureBehaviorModel { feature_type: "minecraft:ore", behavior: "builds sinusoidal vein ellipsoids, culls enclosed spheres, tests target states and discard chance", success_condition: "one or more ore blocks are placed" },
    FeatureBehaviorModel { feature_type: "minecraft:scattered_ore", behavior: "uses ore target tests with scattered attempts", success_condition: "one or more ore blocks are placed" },
    FeatureBehaviorModel { feature_type: "minecraft:disk", behavior: "fills xz disk columns while source blocks and vertical bounds match", success_condition: "one or more disk blocks are placed" },
    FeatureBehaviorModel { feature_type: "minecraft:lake", behavior: "carves an ellipsoid cavity, validates solid/fluid boundary constraints, fills fluid and barrier blocks", success_condition: "cavity validation succeeds" },
    FeatureBehaviorModel { feature_type: "minecraft:geode", behavior: "samples layered ellipsoid thresholds for filling/inner/alternate/budding states and cracks", success_condition: "all sampled positions are evaluated and set through geode layers" },
    FeatureBehaviorModel { feature_type: "minecraft:fossil", behavior: "selects fossil templates, offsets by random rotation and integrity, places fossil and overlay processors", success_condition: "template placement succeeds" },
    FeatureBehaviorModel { feature_type: "minecraft:monster_room", behavior: "validates solid floor/ceiling, counts wall openings, builds cobble shell, chests, and spawner", success_condition: "1 <= openingCount <= 5" },
];

pub const MONSTER_ROOM_BOUNDS: MonsterRoomBounds = MonsterRoomBounds {
    min_y: -1,
    max_y: 4,
    min_openings: 1,
    max_openings: 5,
};

pub const STRUCTURE_TYPES: &[&str] = &[
    "minecraft:buried_treasure",
    "minecraft:desert_pyramid",
    "minecraft:end_city",
    "minecraft:fortress",
    "minecraft:igloo",
    "minecraft:jigsaw",
    "minecraft:jungle_temple",
    "minecraft:mineshaft",
    "minecraft:nether_fossil",
    "minecraft:ocean_monument",
    "minecraft:ocean_ruin",
    "minecraft:ruined_portal",
    "minecraft:shipwreck",
    "minecraft:stronghold",
    "minecraft:swamp_hut",
    "minecraft:woodland_mansion",
];

pub const BUILTIN_STRUCTURES: &[&str] = &[
    "minecraft:pillager_outpost",
    "minecraft:mineshaft",
    "minecraft:mineshaft_mesa",
    "minecraft:mansion",
    "minecraft:jungle_pyramid",
    "minecraft:desert_pyramid",
    "minecraft:igloo",
    "minecraft:shipwreck",
    "minecraft:shipwreck_beached",
    "minecraft:swamp_hut",
    "minecraft:stronghold",
    "minecraft:monument",
    "minecraft:ocean_ruin_cold",
    "minecraft:ocean_ruin_warm",
    "minecraft:fortress",
    "minecraft:nether_fossil",
    "minecraft:end_city",
    "minecraft:buried_treasure",
    "minecraft:bastion_remnant",
    "minecraft:village_plains",
    "minecraft:village_desert",
    "minecraft:village_savanna",
    "minecraft:village_snowy",
    "minecraft:village_taiga",
    "minecraft:ruined_portal",
    "minecraft:ruined_portal_desert",
    "minecraft:ruined_portal_jungle",
    "minecraft:ruined_portal_swamp",
    "minecraft:ruined_portal_mountain",
    "minecraft:ruined_portal_ocean",
    "minecraft:ruined_portal_nether",
    "minecraft:ancient_city",
    "minecraft:trail_ruins",
    "minecraft:trial_chambers",
];

pub const BUILTIN_STRUCTURE_SETS: &[StructureSetEntry] = &[
    structure_set(
        "minecraft:villages",
        &[
            "minecraft:village_plains",
            "minecraft:village_desert",
            "minecraft:village_savanna",
            "minecraft:village_snowy",
            "minecraft:village_taiga",
        ],
        random_spread(34, 8, RandomSpreadType::Linear, 10387312),
    ),
    structure_set(
        "minecraft:desert_pyramids",
        &["minecraft:desert_pyramid"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357617),
    ),
    structure_set(
        "minecraft:igloos",
        &["minecraft:igloo"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357618),
    ),
    structure_set(
        "minecraft:jungle_temples",
        &["minecraft:jungle_pyramid"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357619),
    ),
    structure_set(
        "minecraft:swamp_huts",
        &["minecraft:swamp_hut"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357620),
    ),
    structure_set(
        "minecraft:pillager_outposts",
        &["minecraft:pillager_outpost"],
        random_spread(32, 8, RandomSpreadType::Linear, 165745296),
    ),
    structure_set(
        "minecraft:ancient_cities",
        &["minecraft:ancient_city"],
        random_spread(24, 8, RandomSpreadType::Linear, 20083232),
    ),
    structure_set(
        "minecraft:ocean_monuments",
        &["minecraft:monument"],
        random_spread(32, 5, RandomSpreadType::Triangular, 10387313),
    ),
    structure_set(
        "minecraft:woodland_mansions",
        &["minecraft:mansion"],
        random_spread(80, 20, RandomSpreadType::Triangular, 10387319),
    ),
    structure_set(
        "minecraft:buried_treasures",
        &["minecraft:buried_treasure"],
        random_spread(1, 0, RandomSpreadType::Linear, 0),
    ),
    structure_set(
        "minecraft:mineshafts",
        &["minecraft:mineshaft", "minecraft:mineshaft_mesa"],
        random_spread(1, 0, RandomSpreadType::Linear, 0),
    ),
    structure_set(
        "minecraft:ruined_portals",
        &[
            "minecraft:ruined_portal",
            "minecraft:ruined_portal_desert",
            "minecraft:ruined_portal_jungle",
            "minecraft:ruined_portal_swamp",
            "minecraft:ruined_portal_mountain",
            "minecraft:ruined_portal_ocean",
            "minecraft:ruined_portal_nether",
        ],
        random_spread(40, 15, RandomSpreadType::Linear, 34222645),
    ),
    structure_set(
        "minecraft:shipwrecks",
        &["minecraft:shipwreck", "minecraft:shipwreck_beached"],
        random_spread(24, 4, RandomSpreadType::Linear, 165745295),
    ),
    structure_set(
        "minecraft:ocean_ruins",
        &["minecraft:ocean_ruin_cold", "minecraft:ocean_ruin_warm"],
        random_spread(20, 8, RandomSpreadType::Linear, 14357621),
    ),
    structure_set(
        "minecraft:nether_complexes",
        &["minecraft:fortress", "minecraft:bastion_remnant"],
        random_spread(27, 4, RandomSpreadType::Linear, 30084232),
    ),
    structure_set(
        "minecraft:nether_fossils",
        &["minecraft:nether_fossil"],
        random_spread(2, 1, RandomSpreadType::Linear, 14357921),
    ),
    structure_set(
        "minecraft:end_cities",
        &["minecraft:end_city"],
        random_spread(20, 11, RandomSpreadType::Triangular, 10387313),
    ),
    structure_set(
        "minecraft:strongholds",
        &["minecraft:stronghold"],
        StructurePlacementKind::ConcentricRings {
            distance: 32,
            spread: 3,
            count: 128,
        },
    ),
    structure_set(
        "minecraft:trail_ruins",
        &["minecraft:trail_ruins"],
        random_spread(34, 8, RandomSpreadType::Linear, 83469867),
    ),
    structure_set(
        "minecraft:trial_chambers",
        &["minecraft:trial_chambers"],
        random_spread(34, 12, RandomSpreadType::Linear, 94251327),
    ),
];

pub const STRUCTURE_FAMILIES: &[StructureFamilyEntry] = &[
    structure_family(
        StructureFamily::Village,
        &[
            "minecraft:village_plains",
            "minecraft:village_desert",
            "minecraft:village_savanna",
            "minecraft:village_snowy",
            "minecraft:village_taiga",
        ],
    ),
    structure_family(StructureFamily::Stronghold, &["minecraft:stronghold"]),
    structure_family(
        StructureFamily::Mineshaft,
        &["minecraft:mineshaft", "minecraft:mineshaft_mesa"],
    ),
    structure_family(StructureFamily::OceanMonument, &["minecraft:monument"]),
    structure_family(StructureFamily::WoodlandMansion, &["minecraft:mansion"]),
    structure_family(StructureFamily::Bastion, &["minecraft:bastion_remnant"]),
    structure_family(StructureFamily::Fortress, &["minecraft:fortress"]),
    structure_family(StructureFamily::AncientCity, &["minecraft:ancient_city"]),
    structure_family(
        StructureFamily::TrialChambers,
        &["minecraft:trial_chambers"],
    ),
    structure_family(StructureFamily::EndCity, &["minecraft:end_city"]),
    structure_family(
        StructureFamily::RuinedPortal,
        &[
            "minecraft:ruined_portal",
            "minecraft:ruined_portal_desert",
            "minecraft:ruined_portal_jungle",
            "minecraft:ruined_portal_swamp",
            "minecraft:ruined_portal_mountain",
            "minecraft:ruined_portal_ocean",
            "minecraft:ruined_portal_nether",
        ],
    ),
    structure_family(
        StructureFamily::Shipwreck,
        &["minecraft:shipwreck", "minecraft:shipwreck_beached"],
    ),
    structure_family(
        StructureFamily::BuriedTreasure,
        &["minecraft:buried_treasure"],
    ),
    structure_family(StructureFamily::Igloo, &["minecraft:igloo"]),
    structure_family(StructureFamily::SwampHut, &["minecraft:swamp_hut"]),
    structure_family(
        StructureFamily::PillagerOutpost,
        &["minecraft:pillager_outpost"],
    ),
    structure_family(StructureFamily::TrailRuins, &["minecraft:trail_ruins"]),
    structure_family(StructureFamily::Fossil, &["minecraft:nether_fossil"]),
    structure_family(
        StructureFamily::DesertPyramid,
        &["minecraft:desert_pyramid"],
    ),
    structure_family(StructureFamily::JungleTemple, &["minecraft:jungle_pyramid"]),
    structure_family(
        StructureFamily::OceanRuins,
        &["minecraft:ocean_ruin_cold", "minecraft:ocean_ruin_warm"],
    ),
];

pub const STRUCTURE_POOL_ELEMENT_TYPES: &[&str] = &[
    "minecraft:single_pool_element",
    "minecraft:list_pool_element",
    "minecraft:feature_pool_element",
    "minecraft:empty_pool_element",
    "minecraft:legacy_single_pool_element",
];

pub const STRUCTURE_PROCESSOR_TYPES: &[&str] = &[
    "minecraft:block_ignore",
    "minecraft:block_rot",
    "minecraft:gravity",
    "minecraft:jigsaw_replacement",
    "minecraft:rule",
    "minecraft:nop",
    "minecraft:block_age",
    "minecraft:blackstone_replace",
    "minecraft:lava_submerged_block",
    "minecraft:protected_blocks",
    "minecraft:capped",
];

pub const STRUCTURE_RULE_TEST_TYPES: &[&str] = &[
    "minecraft:always_true",
    "minecraft:block_match",
    "minecraft:blockstate_match",
    "minecraft:tag_match",
    "minecraft:random_block_match",
    "minecraft:random_blockstate_match",
];

pub const STRUCTURE_POS_RULE_TEST_TYPES: &[&str] = &[
    "minecraft:always_true",
    "minecraft:linear_pos",
    "minecraft:axis_aligned_linear_pos",
];

pub const STRUCTURE_PIECE_TYPES: &[&str] = &[
    "mscorridor",
    "mscrossing",
    "msroom",
    "msstairs",
    "nebcr",
    "nebef",
    "nebs",
    "neccs",
    "nectb",
    "nece",
    "nescsc",
    "nesclt",
    "nesc",
    "nescrt",
    "necsr",
    "nemt",
    "nerc",
    "nesr",
    "nestart",
    "shcc",
    "shfc",
    "sh5c",
    "shlt",
    "shli",
    "shpr",
    "shph",
    "shrt",
    "shrc",
    "shsd",
    "shstart",
    "shs",
    "shssd",
    "tejp",
    "orp",
    "iglu",
    "rupo",
    "tesh",
    "tedp",
    "omb",
    "omcr",
    "omdxr",
    "omdxyr",
    "omdyr",
    "omdyzr",
    "omdzr",
    "omentry",
    "ompenthouse",
    "omsimple",
    "omsimplet",
    "omwr",
    "ecp",
    "wmp",
    "btp",
    "shipwreck",
    "nefos",
    "jigsaw",
];

pub const STRUCTURE_PROCESSOR_LISTS: &[&str] = &[
    "minecraft:empty",
    "minecraft:zombie_plains",
    "minecraft:zombie_savanna",
    "minecraft:zombie_snowy",
    "minecraft:zombie_taiga",
    "minecraft:zombie_desert",
    "minecraft:mossify_10_percent",
    "minecraft:mossify_20_percent",
    "minecraft:mossify_70_percent",
    "minecraft:street_plains",
    "minecraft:street_savanna",
    "minecraft:street_snowy_or_taiga",
    "minecraft:farm_plains",
    "minecraft:farm_savanna",
    "minecraft:farm_snowy",
    "minecraft:farm_taiga",
    "minecraft:farm_desert",
    "minecraft:outpost_rot",
    "minecraft:bottom_rampart",
    "minecraft:treasure_rooms",
    "minecraft:housing",
    "minecraft:side_wall_degradation",
    "minecraft:stable_degradation",
    "minecraft:bastion_generic_degradation",
    "minecraft:rampart_degradation",
    "minecraft:entrance_replacement",
    "minecraft:bridge",
    "minecraft:roof",
    "minecraft:high_wall",
    "minecraft:high_rampart",
    "minecraft:fossil_rot",
    "minecraft:fossil_coal",
    "minecraft:fossil_diamonds",
    "minecraft:ancient_city_start_degradation",
    "minecraft:ancient_city_generic_degradation",
    "minecraft:ancient_city_walls_degradation",
    "minecraft:trail_ruins_houses_archaeology",
    "minecraft:trail_ruins_roads_archaeology",
    "minecraft:trail_ruins_tower_top_archaeology",
    "minecraft:trial_chambers_copper_bulb_degradation",
];

pub const JIGSAW_POOL_BOOTSTRAP_SOURCES: &[JigsawPoolBootstrapSource] = &[
    JigsawPoolBootstrapSource {
        source_file: "AncientCityStructurePieces.java",
        registrations: 1,
    },
    JigsawPoolBootstrapSource {
        source_file: "AncientCityStructurePools.java",
        registrations: 6,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionBridgePools.java",
        registrations: 7,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionHoglinStablePools.java",
        registrations: 13,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionHousingUnitsPools.java",
        registrations: 15,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionPieces.java",
        registrations: 1,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionSharedPools.java",
        registrations: 4,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionTreasureRoomPools.java",
        registrations: 20,
    },
    JigsawPoolBootstrapSource {
        source_file: "DesertVillagePools.java",
        registrations: 12,
    },
    JigsawPoolBootstrapSource {
        source_file: "PillagerOutpostPools.java",
        registrations: 4,
    },
    JigsawPoolBootstrapSource {
        source_file: "PlainVillagePools.java",
        registrations: 17,
    },
    JigsawPoolBootstrapSource {
        source_file: "Pools.java",
        registrations: 2,
    },
    JigsawPoolBootstrapSource {
        source_file: "SavannaVillagePools.java",
        registrations: 12,
    },
    JigsawPoolBootstrapSource {
        source_file: "SnowyVillagePools.java",
        registrations: 11,
    },
    JigsawPoolBootstrapSource {
        source_file: "TaigaVillagePools.java",
        registrations: 10,
    },
    JigsawPoolBootstrapSource {
        source_file: "TrailRuinsStructurePools.java",
        registrations: 7,
    },
    JigsawPoolBootstrapSource {
        source_file: "TrialChambersStructurePools.java",
        registrations: 34,
    },
];

pub const JIGSAW_STRUCTURE_START_POOLS: &[JigsawStartPoolModel] = &[
    JigsawStartPoolModel {
        structure_family: "village/plains",
        source_file: "PlainVillagePools.java",
        pool: "minecraft:village/plains/town_centers",
    },
    JigsawStartPoolModel {
        structure_family: "village/desert",
        source_file: "DesertVillagePools.java",
        pool: "minecraft:village/desert/town_centers",
    },
    JigsawStartPoolModel {
        structure_family: "village/savanna",
        source_file: "SavannaVillagePools.java",
        pool: "minecraft:village/savanna/town_centers",
    },
    JigsawStartPoolModel {
        structure_family: "village/snowy",
        source_file: "SnowyVillagePools.java",
        pool: "minecraft:village/snowy/town_centers",
    },
    JigsawStartPoolModel {
        structure_family: "village/taiga",
        source_file: "TaigaVillagePools.java",
        pool: "minecraft:village/taiga/town_centers",
    },
    JigsawStartPoolModel {
        structure_family: "pillager_outpost",
        source_file: "PillagerOutpostPools.java",
        pool: "minecraft:pillager_outpost/base_plates",
    },
    JigsawStartPoolModel {
        structure_family: "bastion",
        source_file: "BastionPieces.java",
        pool: "minecraft:bastion/starts",
    },
    JigsawStartPoolModel {
        structure_family: "ancient_city",
        source_file: "AncientCityStructurePieces.java",
        pool: "minecraft:ancient_city/city_center",
    },
    JigsawStartPoolModel {
        structure_family: "trail_ruins",
        source_file: "TrailRuinsStructurePools.java",
        pool: "minecraft:trail_ruins/tower",
    },
    JigsawStartPoolModel {
        structure_family: "trial_chambers",
        source_file: "TrialChambersStructurePools.java",
        pool: "minecraft:trial_chambers/chamber/end",
    },
];

pub const BLENDING_CONSTANTS: BlendingConstants = BlendingConstants {
    height_blending_range_cells: 27,
    height_blending_range_chunks: 7,
    density_blending_range_cells: 2,
    density_blending_range_chunks: 2,
    old_chunk_xz_radius: 8,
    cell_width: 4,
    cell_height: 8,
    cell_ratio: 2,
};

pub const BLENDING_CELL_COLUMN_COUNT: usize = 16;
pub const BLENDING_NO_VALUE: f64 = f64::MAX;

pub const UPGRADE_DATA_MODEL: UpgradeDataModel = UpgradeDataModel {
    tag_indices: "Indices",
    tag_sides: "Sides",
    tag_neighbor_block_ticks: "neighbor_block_ticks",
    tag_neighbor_fluid_ticks: "neighbor_fluid_ticks",
    block_fixers: &["blacklist", "default", "chest", "leaves", "stem_block"],
    chunky_fixers: &["leaves"],
};

pub const BELOW_ZERO_RETROGEN_MODEL: BelowZeroRetrogenModel = BelowZeroRetrogenModel {
    target_status_field: "target_status",
    missing_bedrock_field: "missing_bedrock",
    upgrade_min_y: -64,
    upgrade_height: 64,
    max_generated_bedrock_y: 4,
    retained_biomes: &[
        "minecraft:lush_caves",
        "minecraft:dripstone_caves",
        "minecraft:deep_dark",
    ],
};

pub const SPAWN_SELECTION_CONSTANTS: SpawnSelectionConstants = SpawnSelectionConstants {
    initial_chunk_search_radius: 5,
    player_spawn_ticket_radius: 3,
    spawn_search_absolute_max_attempts: 1024,
    default_respawn_radius: 10,
    small_search_coprime_threshold: 16,
    large_search_coprime: 17,
};

const fn structure_family(
    family: StructureFamily,
    structures: &'static [&'static str],
) -> StructureFamilyEntry {
    StructureFamilyEntry { family, structures }
}

const fn structure_set(
    id: &'static str,
    structures: &'static [&'static str],
    placement: StructurePlacementKind,
) -> StructureSetEntry {
    StructureSetEntry {
        id,
        structures,
        placement,
    }
}

const fn random_spread(
    spacing: i32,
    separation: i32,
    spread_type: RandomSpreadType,
    salt: i32,
) -> StructurePlacementKind {
    StructurePlacementKind::RandomSpread {
        spacing,
        separation,
        salt,
        spread_type,
    }
}

