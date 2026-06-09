use crate::worldgen::{
    load_template_pool_registry, ParsedJigsawTemplatePool, JIGSAW_POOL_BOOTSTRAP_SOURCES,
    JIGSAW_STRUCTURE_START_POOLS,
};

const ANCIENT_CITY_STRUCTURE_PIECES_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/AncientCityStructurePieces.java"
);
const ANCIENT_CITY_STRUCTURE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/AncientCityStructurePools.java"
);
const BASTION_BRIDGE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionBridgePools.java"
);
const BASTION_HOGLIN_STABLE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionHoglinStablePools.java"
);
const BASTION_HOUSING_UNITS_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionHousingUnitsPools.java"
);
const BASTION_PIECES_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionPieces.java");
const BASTION_SHARED_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionSharedPools.java"
);
const BASTION_TREASURE_ROOM_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionTreasureRoomPools.java"
);
const DESERT_VILLAGE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/DesertVillagePools.java"
);
const PILLAGER_OUTPOST_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/PillagerOutpostPools.java"
);
const PLAIN_VILLAGE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/PlainVillagePools.java"
);
const POOLS_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/Pools.java");
const SAVANNA_VILLAGE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/SavannaVillagePools.java"
);
const SNOWY_VILLAGE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/SnowyVillagePools.java"
);
const TAIGA_VILLAGE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/TaigaVillagePools.java"
);
const TRAIL_RUINS_STRUCTURE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/TrailRuinsStructurePools.java"
);
const TRIAL_CHAMBERS_STRUCTURE_POOLS_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/TrialChambersStructurePools.java"
);
const VILLAGE_POOLS_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/VillagePools.java");

#[derive(Debug, Clone, Copy)]
struct JigsawPoolSourceAudit {
    source_file: &'static str,
    source: &'static str,
    line_count: usize,
    registrations: usize,
    single_elements: usize,
    list_elements: usize,
    empty_elements: usize,
    bootstrap_calls: usize,
    sentinels: &'static [&'static str],
}

const JIGSAW_POOL_SOURCES: &[JigsawPoolSourceAudit] = &[
    JigsawPoolSourceAudit {
        source_file: "AncientCityStructurePieces.java",
        source: ANCIENT_CITY_STRUCTURE_PIECES_JAVA,
        line_count: 35,
        registrations: 1,
        single_elements: 3,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 1,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"ancient_city/city_center\");",
            "ProcessorLists.ANCIENT_CITY_START_DEGRADATION",
            "StructurePoolElement.single(\"ancient_city/city_center/city_center_3\", ancientCityStartDegradation)",
            "AncientCityStructurePools.bootstrap(context);",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "AncientCityStructurePools.java",
        source: ANCIENT_CITY_STRUCTURE_POOLS_JAVA,
        line_count: 156,
        registrations: 6,
        single_elements: 61,
        list_elements: 2,
        empty_elements: 2,
        bootstrap_calls: 0,
        sentinels: &[
            "CavePlacements.SCULK_PATCH_ANCIENT_CITY",
            "ProcessorLists.ANCIENT_CITY_GENERIC_DEGRADATION",
            "ProcessorLists.ANCIENT_CITY_WALLS_DEGRADATION",
            "StructurePoolElement.list(\n                        ImmutableList.of(",
            "StructurePoolElement.single(\"ancient_city/structures/ice_box_1\")",
            "Pools.register(\n         context,\n         \"ancient_city/city/entrance\"",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "BastionBridgePools.java",
        source: BASTION_BRIDGE_POOLS_JAVA,
        line_count: 100,
        registrations: 7,
        single_elements: 12,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 0,
        sentinels: &[
            "ProcessorLists.ENTRANCE_REPLACEMENT",
            "ProcessorLists.BRIDGE",
            "Pools.register(\n         context,\n         \"bastion/bridge/connectors\"",
            "StructurePoolElement.single(\"bastion/bridge/connectors/back_bridge_bottom\", bastionGenericDegradation)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "BastionHoglinStablePools.java",
        source: BASTION_HOGLIN_STABLE_POOLS_JAVA,
        line_count: 199,
        registrations: 13,
        single_elements: 53,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 0,
        sentinels: &[
            "ProcessorLists.STABLE_DEGRADATION",
            "Pools.register(\n         context,\n         \"bastion/hoglin_stable/starting_pieces\"",
            "StructurePoolElement.single(\"bastion/hoglin_stable/large_stables/inner_4\", stableDegradation)",
            "StructurePoolElement.single(\"bastion/hoglin_stable/ramparts/ramparts_3\", stableDegradation)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "BastionHousingUnitsPools.java",
        source: BASTION_HOUSING_UNITS_POOLS_JAVA,
        line_count: 182,
        registrations: 15,
        single_elements: 31,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 0,
        sentinels: &[
            "ProcessorLists.HOUSING",
            "Pools.register(\n         context,\n         \"bastion/units/center_pieces\"",
            "StructurePoolElement.single(\"bastion/units/walls/wall_base\", housing)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "BastionPieces.java",
        source: BASTION_PIECES_JAVA,
        line_count: 40,
        registrations: 1,
        single_elements: 4,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 5,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"bastion/starts\");",
            "StructurePoolElement.single(\"bastion/treasure/big_air_full\", bastionGenericDegradation)",
            "BastionTreasureRoomPools.bootstrap(context);",
            "BastionSharedPools.bootstrap(context);",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "BastionSharedPools.java",
        source: BASTION_SHARED_POOLS_JAVA,
        line_count: 61,
        registrations: 4,
        single_elements: 11,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 0,
        sentinels: &[
            "Holder<StructureTemplatePool> empty = pools.getOrThrow(Pools.EMPTY);",
            "Pools.register(\n         context,\n         \"bastion/blocks/gold\"",
            "StructurePoolElement.single(\"bastion/blocks/air\")",
            "StructurePoolElement.single(\"bastion/mobs/sword_piglin\")",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "BastionTreasureRoomPools.java",
        source: BASTION_TREASURE_ROOM_POOLS_JAVA,
        line_count: 280,
        registrations: 20,
        single_elements: 65,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 0,
        sentinels: &[
            "ProcessorLists.TREASURE_ROOMS",
            "ProcessorLists.HIGH_RAMPART",
            "Pools.register(\n         context,\n         \"bastion/treasure/bases\"",
            "StructurePoolElement.single(\"bastion/treasure/connectors/center_to_wall_middle\", treasureRooms)",
            "StructurePoolElement.single(\"bastion/treasure/extensions/empty\", treasureRooms)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "DesertVillagePools.java",
        source: DESERT_VILLAGE_POOLS_JAVA,
        line_count: 253,
        registrations: 12,
        single_elements: 0,
        list_elements: 0,
        empty_elements: 4,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"village/desert/town_centers\");",
            "Holder<PlacedFeature> patchCactusVillage = placedFeatures.getOrThrow(VillagePlacements.PATCH_CACTUS_VILLAGE);",
            "Holder<StructureProcessorList> zombieDesert = processorLists.getOrThrow(ProcessorLists.ZOMBIE_DESERT);",
            "Holder<StructureTemplatePool> terminators = pools.getOrThrow(TERMINATORS_KEY);",
            "Pair.of(StructurePoolElement.legacy(\"village/desert/zombie/town_centers/desert_meeting_point_3\", zombieDesert), 1)",
            "Pair.of(StructurePoolElement.legacy(\"village/desert/houses/desert_large_farm_1\", farmDesert), 11)",
            "Pair.of(StructurePoolElement.legacy(\"village/desert/houses/desert_large_farm_1\", zombieDesert), 7)",
            "Pair.of(StructurePoolElement.feature(patchCactusVillage), 4)",
            "Pair.of(StructurePoolElement.legacy(\"village/desert/camel_spawn\"), 1)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "PillagerOutpostPools.java",
        source: PILLAGER_OUTPOST_POOLS_JAVA,
        line_count: 74,
        registrations: 4,
        single_elements: 0,
        list_elements: 1,
        empty_elements: 1,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"pillager_outpost/base_plates\");",
            "Holder<StructureProcessorList> outpostRot = processorLists.getOrThrow(ProcessorLists.OUTPOST_ROT);",
            "Holder<StructureTemplatePool> empty = pools.getOrThrow(Pools.EMPTY);",
            "StructurePoolElement.legacy(\"pillager_outpost/base_plate\")",
            "StructurePoolElement.legacy(\"pillager_outpost/watchtower_overgrown\", outpostRot)",
            "Pools.register(\n         context,\n         \"pillager_outpost/feature_plates\"",
            "Pair.of(StructurePoolElement.legacy(\"pillager_outpost/feature_cage_with_allays\"), 1)",
            "Pair.of(StructurePoolElement.empty(), 6)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "PlainVillagePools.java",
        source: PLAIN_VILLAGE_POOLS_JAVA,
        line_count: 359,
        registrations: 17,
        single_elements: 0,
        list_elements: 0,
        empty_elements: 6,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"village/plains/town_centers\");",
            "Holder<PlacedFeature> oakVillage = placedFeatures.getOrThrow(VillagePlacements.OAK_VILLAGE);",
            "Holder<StructureProcessorList> mossify10Percent = processorLists.getOrThrow(ProcessorLists.MOSSIFY_10_PERCENT);",
            "Holder<StructureProcessorList> zombiePlains = processorLists.getOrThrow(ProcessorLists.ZOMBIE_PLAINS);",
            "Holder<StructureProcessorList> streetPlains = processorLists.getOrThrow(ProcessorLists.STREET_PLAINS);",
            "Pair.of(StructurePoolElement.legacy(\"village/plains/town_centers/plains_fountain_01\", mossify20Percent), 50)",
            "Pair.of(StructurePoolElement.legacy(\"village/plains/houses/plains_large_farm_1\", farmPlains), 4)",
            "Pair.of(StructurePoolElement.feature(flowerPlainVillage), 1)",
            "Pair.of(StructurePoolElement.legacy(\"village/common/animals/cat_jellie\"), 1)",
            "Pair.of(StructurePoolElement.legacy(\"village/common/well_bottom\"), 1)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "Pools.java",
        source: POOLS_JAVA,
        line_count: 41,
        registrations: 2,
        single_elements: 0,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 6,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> EMPTY = createKey(\"empty\");",
            "return ResourceKey.create(Registries.TEMPLATE_POOL, location);",
            "return createKey(Identifier.withDefaultNamespace(name));",
            "return createKey(Identifier.parse(name));",
            "context.register(createKey(name), pool);",
            "Holder<StructureTemplatePool> empty = pools.getOrThrow(EMPTY);",
            "context.register(EMPTY, new StructureTemplatePool(empty, ImmutableList.of(), StructureTemplatePool.Projection.RIGID));",
            "BastionPieces.bootstrap(context);",
            "PillagerOutpostPools.bootstrap(context);",
            "VillagePools.bootstrap(context);",
            "AncientCityStructurePieces.bootstrap(context);",
            "TrailRuinsStructurePools.bootstrap(context);",
            "TrialChambersStructurePools.bootstrap(context);",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "VillagePools.java",
        source: VILLAGE_POOLS_JAVA,
        line_count: 13,
        registrations: 0,
        single_elements: 0,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 5,
        sentinels: &[
            "PlainVillagePools.bootstrap(context);",
            "SnowyVillagePools.bootstrap(context);",
            "SavannaVillagePools.bootstrap(context);",
            "DesertVillagePools.bootstrap(context);",
            "TaigaVillagePools.bootstrap(context);",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "SavannaVillagePools.java",
        source: SAVANNA_VILLAGE_POOLS_JAVA,
        line_count: 289,
        registrations: 12,
        single_elements: 0,
        list_elements: 0,
        empty_elements: 4,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"village/savanna/town_centers\");",
            "Holder<PlacedFeature> acaciaVillage = placedFeatures.getOrThrow(VillagePlacements.ACACIA_VILLAGE);",
            "Holder<PlacedFeature> pileMelonVillage = placedFeatures.getOrThrow(VillagePlacements.PILE_MELON_VILLAGE);",
            "Holder<StructureProcessorList> zombieSavanna = processorLists.getOrThrow(ProcessorLists.ZOMBIE_SAVANNA);",
            "Holder<StructureProcessorList> streetSavanna = processorLists.getOrThrow(ProcessorLists.STREET_SAVANNA);",
            "Holder<StructureProcessorList> farmSavanna = processorLists.getOrThrow(ProcessorLists.FARM_SAVANNA);",
            "Holder<StructureTemplatePool> zombieTerminators = pools.getOrThrow(ZOMBIE_TERMINATORS_KEY);",
            "Pair.of(StructurePoolElement.legacy(\"village/savanna/town_centers/savanna_meeting_point_3\"), 150)",
            "Pair.of(StructurePoolElement.legacy(\"village/savanna/zombie/town_centers/savanna_meeting_point_4\", zombieSavanna), 3)",
            "Pair.of(StructurePoolElement.legacy(\"village/savanna/streets/straight_04\", streetSavanna), 7)",
            "Pair.of(StructurePoolElement.legacy(\"village/savanna/houses/savanna_large_farm_2\", farmSavanna), 6)",
            "Pair.of(StructurePoolElement.legacy(\"village/savanna/zombie/houses/savanna_large_farm_2\", zombieSavanna), 4)",
            "Pair.of(StructurePoolElement.feature(pileMelonVillage), 1)",
            "Pair.of(StructurePoolElement.legacy(\"village/savanna/zombie/villagers/unemployed\"), 10)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "SnowyVillagePools.java",
        source: SNOWY_VILLAGE_POOLS_JAVA,
        line_count: 266,
        registrations: 11,
        single_elements: 0,
        list_elements: 0,
        empty_elements: 4,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"village/snowy/town_centers\");",
            "Holder<PlacedFeature> spruceVillage = placedFeatures.getOrThrow(VillagePlacements.SPRUCE_VILLAGE);",
            "Holder<PlacedFeature> pileIceVillage = placedFeatures.getOrThrow(VillagePlacements.PILE_ICE_VILLAGE);",
            "Holder<StructureProcessorList> streetSnowyOrTaiga = processorLists.getOrThrow(ProcessorLists.STREET_SNOWY_OR_TAIGA);",
            "Holder<StructureProcessorList> farmSnowy = processorLists.getOrThrow(ProcessorLists.FARM_SNOWY);",
            "Holder<StructureProcessorList> zombieSnowy = processorLists.getOrThrow(ProcessorLists.ZOMBIE_SNOWY);",
            "Pair.of(StructurePoolElement.legacy(\"village/snowy/town_centers/snowy_meeting_point_3\"), 150)",
            "Pair.of(StructurePoolElement.legacy(\"village/snowy/zombie/town_centers/snowy_meeting_point_3\"), 3)",
            "Pair.of(StructurePoolElement.legacy(\"village/snowy/streets/straight_04\", streetSnowyOrTaiga), 7)",
            "Pair.of(StructurePoolElement.legacy(\"village/snowy/houses/snowy_farm_2\", farmSnowy), 3)",
            "Pair.of(StructurePoolElement.legacy(\"village/snowy/zombie/houses/snowy_medium_house_3\", zombieSnowy), 1)",
            "Pair.of(StructurePoolElement.feature(pileIceVillage), 4)",
            "Pair.of(StructurePoolElement.legacy(\"village/snowy/zombie/villagers/unemployed\"), 10)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "TaigaVillagePools.java",
        source: TAIGA_VILLAGE_POOLS_JAVA,
        line_count: 264,
        registrations: 10,
        single_elements: 0,
        list_elements: 0,
        empty_elements: 4,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"village/taiga/town_centers\");",
            "Holder<PlacedFeature> pineVillage = placedFeatures.getOrThrow(VillagePlacements.PINE_VILLAGE);",
            "Holder<PlacedFeature> patchBerryBushVillage = placedFeatures.getOrThrow(VillagePlacements.PATCH_BERRY_BUSH_VILLAGE);",
            "Holder<StructureProcessorList> mossify10Percent = processorLists.getOrThrow(ProcessorLists.MOSSIFY_10_PERCENT);",
            "Holder<StructureProcessorList> zombieTaiga = processorLists.getOrThrow(ProcessorLists.ZOMBIE_TAIGA);",
            "Holder<StructureProcessorList> streetSnowyOrTaiga = processorLists.getOrThrow(ProcessorLists.STREET_SNOWY_OR_TAIGA);",
            "Holder<StructureProcessorList> farmTaiga = processorLists.getOrThrow(ProcessorLists.FARM_TAIGA);",
            "Pair.of(StructurePoolElement.legacy(\"village/taiga/town_centers/taiga_meeting_point_1\", mossify10Percent), 49)",
            "Pair.of(StructurePoolElement.legacy(\"village/taiga/zombie/town_centers/taiga_meeting_point_2\", zombieTaiga), 1)",
            "Pair.of(StructurePoolElement.legacy(\"village/taiga/streets/straight_05\", streetSnowyOrTaiga), 7)",
            "Pair.of(StructurePoolElement.legacy(\"village/taiga/houses/taiga_large_farm_2\", farmTaiga), 6)",
            "Pair.of(StructurePoolElement.legacy(\"village/taiga/zombie/houses/taiga_large_farm_2\", zombieTaiga), 6)",
            "Pair.of(StructurePoolElement.feature(patchBerryBushVillage), 1)",
            "Pair.of(StructurePoolElement.legacy(\"village/taiga/zombie/villagers/unemployed\"), 10)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "TrailRuinsStructurePools.java",
        source: TRAIL_RUINS_STRUCTURE_POOLS_JAVA,
        line_count: 177,
        registrations: 7,
        single_elements: 84,
        list_elements: 0,
        empty_elements: 0,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"trail_ruins/tower\");",
            "Holder<StructureProcessorList> housesArchyProcessor = processorLists.getOrThrow(ProcessorLists.TRAIL_RUINS_HOUSES_ARCHAEOLOGY);",
            "Holder<StructureProcessorList> roadsArchyProcessor = processorLists.getOrThrow(ProcessorLists.TRAIL_RUINS_ROADS_ARCHAEOLOGY);",
            "Holder<StructureProcessorList> towerTopArchyProcessor = processorLists.getOrThrow(ProcessorLists.TRAIL_RUINS_TOWER_TOP_ARCHAEOLOGY);",
            "Pair.of(StructurePoolElement.single(\"trail_ruins/tower/tower_5\", housesArchyProcessor), 1)",
            "Pair.of(StructurePoolElement.single(\"trail_ruins/tower/tower_top_5\", towerTopArchyProcessor), 1)",
            "Pair.of(StructurePoolElement.single(\"trail_ruins/tower/stable_5\", housesArchyProcessor), 1)",
            "Pair.of(StructurePoolElement.single(\"trail_ruins/roads/road_spacer_1\", roadsArchyProcessor), 1)",
            "Pair.of(StructurePoolElement.single(\"trail_ruins/buildings/group_room_5\", housesArchyProcessor), 1)",
            "Pair.of(StructurePoolElement.single(\"trail_ruins/decor/decor_7\", housesArchyProcessor), 1)",
        ],
    },
    JigsawPoolSourceAudit {
        source_file: "TrialChambersStructurePools.java",
        source: TRIAL_CHAMBERS_STRUCTURE_POOLS_JAVA,
        line_count: 561,
        registrations: 34,
        single_elements: 194,
        list_elements: 0,
        empty_elements: 6,
        bootstrap_calls: 0,
        sentinels: &[
            "public static final ResourceKey<StructureTemplatePool> START = Pools.createKey(\"trial_chambers/chamber/end\");",
            "public static final ResourceKey<StructureTemplatePool> HALLWAY_FALLBACK = Pools.createKey(\"trial_chambers/hallway/fallback\");",
            "PoolAliasBinding.randomGroup(",
            "PoolAliasBinding.random(\n            spawner(\"contents/melee\")",
            "PoolAliasBinding.random(\n            spawner(\"contents/small_melee\")",
            "Holder<StructureProcessorList> trialChambersCopperBulbDegradation = processorLists.getOrThrow(ProcessorLists.TRIAL_CHAMBERS_COPPER_BULB_DEGRADATION);",
            "Pair.of(StructurePoolElement.single(\"trial_chambers/chamber/assembly/cover_7\"), 5)",
            "Pair.of(StructurePoolElement.single(\"trial_chambers/chamber/chamber_8\", trialChambersCopperBulbDegradation), 150)",
            "Pair.of(StructurePoolElement.empty(), 22)",
            "Pools.register(\n         context,\n         \"trial_chambers/spawner/all\"",
            "PoolAliasBindings.registerTargetsAsPools(context, empty, ALIAS_BINDINGS);",
        ],
    },
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

fn rust_registration_count(source_file: &str) -> usize {
    JIGSAW_POOL_BOOTSTRAP_SOURCES
        .iter()
        .find(|source| source.source_file == source_file)
        .map(|source| source.registrations)
        .unwrap_or_else(|| panic!("missing Rust jigsaw bootstrap source: {source_file}"))
}

fn parsed_pool<'a>(
    pools: &'a std::collections::BTreeMap<String, ParsedJigsawTemplatePool>,
    id: &str,
) -> &'a ParsedJigsawTemplatePool {
    pools
        .get(id)
        .unwrap_or_else(|| panic!("missing parsed template pool: {id}"))
}

fn pool_weight_sum(pool: &ParsedJigsawTemplatePool) -> i32 {
    pool.elements.iter().map(|entry| entry.weight).sum()
}

fn pool_entry_by_location<'a>(
    pool: &'a ParsedJigsawTemplatePool,
    location: &str,
) -> &'a crate::worldgen::ParsedJigsawTemplatePoolEntry {
    pool.elements
        .iter()
        .find(|entry| entry.element.location.as_deref() == Some(location))
        .unwrap_or_else(|| panic!("missing template pool element location: {location}"))
}

#[cfg(test)]
mod village_tests;

#[cfg(test)]
mod trail_trial_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jigsaw_pool_bootstrap_sources_match_java_counts() {
        for source in JIGSAW_POOL_SOURCES {
            assert_source_contains_all(source.source, source.sentinels);
            assert_eq!(
                source.source.lines().count(),
                source.line_count,
                "line-count drift for {}",
                source.source_file
            );
            assert_eq!(
                count_occurrences(source.source, "Pools.register(")
                    + count_occurrences(source.source, "context.register("),
                source.registrations,
                "registration count drift for {}",
                source.source_file
            );
            assert_eq!(
                rust_registration_count(source.source_file),
                source.registrations,
                "Rust bootstrap metadata drift for {}",
                source.source_file
            );
            assert_eq!(
                count_occurrences(source.source, "StructurePoolElement.single("),
                source.single_elements,
                "single element count drift for {}",
                source.source_file
            );
            assert_eq!(
                count_occurrences(source.source, "StructurePoolElement.list("),
                source.list_elements,
                "list element count drift for {}",
                source.source_file
            );
            assert_eq!(
                count_occurrences(source.source, "StructurePoolElement.empty()"),
                source.empty_elements,
                "empty element count drift for {}",
                source.source_file
            );
            assert_eq!(
                count_occurrences(source.source, ".bootstrap(context)"),
                source.bootstrap_calls,
                "chained bootstrap call count drift for {}",
                source.source_file
            );
        }
    }

    #[test]
    fn jigsaw_start_pool_metadata_matches_java_bootstraps() {
        let ancient_city_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "AncientCityStructurePieces.java")
            .expect("missing ancient city start pool metadata");
        assert_eq!(ancient_city_start.structure_family, "ancient_city");
        assert_eq!(
            ancient_city_start.pool,
            "minecraft:ancient_city/city_center"
        );

        let bastion_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "BastionPieces.java")
            .expect("missing bastion start pool metadata");
        assert_eq!(bastion_start.structure_family, "bastion");
        assert_eq!(bastion_start.pool, "minecraft:bastion/starts");

        let desert_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "DesertVillagePools.java")
            .expect("missing desert village start pool metadata");
        assert_eq!(desert_start.structure_family, "village/desert");
        assert_eq!(desert_start.pool, "minecraft:village/desert/town_centers");

        let outpost_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "PillagerOutpostPools.java")
            .expect("missing pillager outpost start pool metadata");
        assert_eq!(outpost_start.structure_family, "pillager_outpost");
        assert_eq!(outpost_start.pool, "minecraft:pillager_outpost/base_plates");

        let plain_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "PlainVillagePools.java")
            .expect("missing plains village start pool metadata");
        assert_eq!(plain_start.structure_family, "village/plains");
        assert_eq!(plain_start.pool, "minecraft:village/plains/town_centers");

        let savanna_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "SavannaVillagePools.java")
            .expect("missing savanna village start pool metadata");
        assert_eq!(savanna_start.structure_family, "village/savanna");
        assert_eq!(savanna_start.pool, "minecraft:village/savanna/town_centers");

        let snowy_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "SnowyVillagePools.java")
            .expect("missing snowy village start pool metadata");
        assert_eq!(snowy_start.structure_family, "village/snowy");
        assert_eq!(snowy_start.pool, "minecraft:village/snowy/town_centers");

        let taiga_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "TaigaVillagePools.java")
            .expect("missing taiga village start pool metadata");
        assert_eq!(taiga_start.structure_family, "village/taiga");
        assert_eq!(taiga_start.pool, "minecraft:village/taiga/town_centers");

        let trail_ruins_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "TrailRuinsStructurePools.java")
            .expect("missing trail ruins start pool metadata");
        assert_eq!(trail_ruins_start.structure_family, "trail_ruins");
        assert_eq!(trail_ruins_start.pool, "minecraft:trail_ruins/tower");

        let trial_chambers_start = JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .find(|pool| pool.source_file == "TrialChambersStructurePools.java")
            .expect("missing trial chambers start pool metadata");
        assert_eq!(trial_chambers_start.structure_family, "trial_chambers");
        assert_eq!(
            trial_chambers_start.pool,
            "minecraft:trial_chambers/chamber/end"
        );
    }

    fn load_vanilla_template_pools() -> crate::worldgen::ParsedTemplatePoolRegistry {
        load_template_pool_registry(
            "../decompiled-server-26.1.2/data/minecraft/worldgen/template_pool",
        )
        .expect("vanilla template-pool registry should load")
    }

    #[test]
    fn root_pools_bootstrap_empty_pool_and_chained_sources_match_java() {
        let registry = load_vanilla_template_pools();
        let empty = parsed_pool(&registry.pools, "minecraft:empty");
        assert_eq!(empty.fallback, "minecraft:empty");
        assert!(empty.elements.is_empty());

        assert_eq!(
            JIGSAW_POOL_BOOTSTRAP_SOURCES
                .iter()
                .find(|source| source.source_file == "Pools.java")
                .map(|source| source.registrations),
            Some(2)
        );
        for chained_source in [
            "BastionPieces.java",
            "PillagerOutpostPools.java",
            "VillagePools.java",
            "AncientCityStructurePieces.java",
            "TrailRuinsStructurePools.java",
            "TrialChambersStructurePools.java",
        ] {
            assert!(
                POOLS_JAVA.contains(&format!(
                    "{}.bootstrap(context);",
                    chained_source.trim_end_matches(".java")
                )),
                "Pools.java is missing chained bootstrap for {chained_source}"
            );
        }
    }

    #[test]
    fn desert_village_template_pool_json_ids_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let desert_ids = registry
            .pools
            .keys()
            .filter(|id| id.starts_with("minecraft:village/desert/"))
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            desert_ids,
            vec![
                "minecraft:village/desert/camel",
                "minecraft:village/desert/decor",
                "minecraft:village/desert/houses",
                "minecraft:village/desert/streets",
                "minecraft:village/desert/terminators",
                "minecraft:village/desert/town_centers",
                "minecraft:village/desert/villagers",
                "minecraft:village/desert/zombie/decor",
                "minecraft:village/desert/zombie/houses",
                "minecraft:village/desert/zombie/streets",
                "minecraft:village/desert/zombie/terminators",
                "minecraft:village/desert/zombie/villagers",
            ]
        );
    }

    #[test]
    fn desert_village_town_and_street_pools_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let town_centers = parsed_pool(&registry.pools, "minecraft:village/desert/town_centers");
        assert_eq!(town_centers.fallback, "minecraft:empty");
        assert_eq!(town_centers.elements.len(), 6);
        assert_eq!(
            town_centers
                .elements
                .iter()
                .map(|entry| entry.weight)
                .collect::<Vec<_>>(),
            vec![98, 98, 49, 2, 2, 1]
        );
        assert_eq!(
            town_centers.elements[3].element.location.as_deref(),
            Some("minecraft:village/desert/zombie/town_centers/desert_meeting_point_1")
        );
        assert_eq!(
            town_centers.elements[3].element.processors,
            vec!["minecraft:zombie_desert".to_string()]
        );

        let streets = parsed_pool(&registry.pools, "minecraft:village/desert/streets");
        assert_eq!(streets.fallback, "minecraft:village/desert/terminators");
        assert_eq!(streets.elements.len(), 11);
        assert_eq!(pool_weight_sum(streets), 35);
        assert!(streets
            .elements
            .iter()
            .all(|entry| entry.element.projection.as_deref() == Some("terrain_matching")));
    }

    #[test]
    fn desert_village_house_pools_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let houses = parsed_pool(&registry.pools, "minecraft:village/desert/houses");
        assert_eq!(houses.fallback, "minecraft:village/desert/terminators");
        assert_eq!(houses.elements.len(), 29);
        assert_eq!(pool_weight_sum(houses), 72);
        let large_farm = pool_entry_by_location(
            houses,
            "minecraft:village/desert/houses/desert_large_farm_1",
        );
        assert_eq!(
            large_farm.element.processors,
            vec!["minecraft:farm_desert".to_string()]
        );
        assert_eq!(large_farm.weight, 11);
        assert_eq!(
            houses.elements.last().unwrap().element.element_type,
            "minecraft:empty_pool_element"
        );
        assert_eq!(houses.elements.last().unwrap().weight, 5);

        let zombie_houses = parsed_pool(&registry.pools, "minecraft:village/desert/zombie/houses");
        assert_eq!(
            zombie_houses.fallback,
            "minecraft:village/desert/zombie/terminators"
        );
        assert_eq!(zombie_houses.elements.len(), 29);
        assert_eq!(pool_weight_sum(zombie_houses), 68);
        let zombie_large_farm = pool_entry_by_location(
            zombie_houses,
            "minecraft:village/desert/houses/desert_large_farm_1",
        );
        assert_eq!(zombie_large_farm.weight, 7);
        assert_eq!(
            zombie_large_farm.element.processors,
            vec!["minecraft:zombie_desert".to_string()]
        );
    }

    #[test]
    fn desert_village_decor_camel_and_villager_pools_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let decor = parsed_pool(&registry.pools, "minecraft:village/desert/decor");
        assert_eq!(decor.fallback, "minecraft:empty");
        assert_eq!(pool_weight_sum(decor), 28);
        assert_eq!(
            decor.elements[1].element.feature.as_deref(),
            Some("minecraft:patch_cactus")
        );
        assert_eq!(
            decor.elements[2].element.feature.as_deref(),
            Some("minecraft:pile_hay")
        );

        let camel = parsed_pool(&registry.pools, "minecraft:village/desert/camel");
        assert_eq!(camel.elements.len(), 1);
        assert_eq!(camel.elements[0].weight, 1);
        assert_eq!(
            camel.elements[0].element.location.as_deref(),
            Some("minecraft:village/desert/camel_spawn")
        );

        let zombie_villagers =
            parsed_pool(&registry.pools, "minecraft:village/desert/zombie/villagers");
        assert_eq!(pool_weight_sum(zombie_villagers), 11);
        assert_eq!(zombie_villagers.elements.len(), 2);
    }

    #[test]
    fn pillager_outpost_template_pool_json_ids_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let outpost_ids = registry
            .pools
            .keys()
            .filter(|id| id.starts_with("minecraft:pillager_outpost/"))
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            outpost_ids,
            vec![
                "minecraft:pillager_outpost/base_plates",
                "minecraft:pillager_outpost/feature_plates",
                "minecraft:pillager_outpost/features",
                "minecraft:pillager_outpost/towers",
            ]
        );
    }

    #[test]
    fn pillager_outpost_base_tower_and_feature_pools_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let base_plates = parsed_pool(&registry.pools, "minecraft:pillager_outpost/base_plates");
        assert_eq!(base_plates.fallback, "minecraft:empty");
        assert_eq!(base_plates.elements.len(), 1);
        assert_eq!(base_plates.elements[0].weight, 1);
        assert_eq!(
            base_plates.elements[0].element.location.as_deref(),
            Some("minecraft:pillager_outpost/base_plate")
        );
        assert_eq!(
            base_plates.elements[0].element.projection.as_deref(),
            Some("rigid")
        );

        let towers = parsed_pool(&registry.pools, "minecraft:pillager_outpost/towers");
        assert_eq!(towers.fallback, "minecraft:empty");
        assert_eq!(towers.elements.len(), 1);
        assert_eq!(towers.elements[0].weight, 1);
        assert_eq!(
            towers.elements[0].element.element_type,
            "minecraft:list_pool_element"
        );
        assert_eq!(towers.elements[0].element.children.len(), 2);
        assert_eq!(
            towers.elements[0].element.children[0].location.as_deref(),
            Some("minecraft:pillager_outpost/watchtower")
        );
        assert_eq!(
            towers.elements[0].element.children[1].location.as_deref(),
            Some("minecraft:pillager_outpost/watchtower_overgrown")
        );
        assert_eq!(
            towers.elements[0].element.children[1].processors,
            vec!["minecraft:outpost_rot".to_string()]
        );

        let feature_plates =
            parsed_pool(&registry.pools, "minecraft:pillager_outpost/feature_plates");
        assert_eq!(feature_plates.fallback, "minecraft:empty");
        assert_eq!(feature_plates.elements.len(), 1);
        assert_eq!(
            feature_plates.elements[0].element.location.as_deref(),
            Some("minecraft:pillager_outpost/feature_plate")
        );
        assert_eq!(
            feature_plates.elements[0].element.projection.as_deref(),
            Some("terrain_matching")
        );
    }

    #[test]
    fn pillager_outpost_features_pool_matches_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let features = parsed_pool(&registry.pools, "minecraft:pillager_outpost/features");
        assert_eq!(features.fallback, "minecraft:empty");
        assert_eq!(features.elements.len(), 8);
        assert_eq!(pool_weight_sum(features), 13);
        assert_eq!(
            features
                .elements
                .iter()
                .filter_map(|entry| entry.element.location.as_deref())
                .collect::<Vec<_>>(),
            vec![
                "minecraft:pillager_outpost/feature_cage1",
                "minecraft:pillager_outpost/feature_cage2",
                "minecraft:pillager_outpost/feature_cage_with_allays",
                "minecraft:pillager_outpost/feature_logs",
                "minecraft:pillager_outpost/feature_tent1",
                "minecraft:pillager_outpost/feature_tent2",
                "minecraft:pillager_outpost/feature_targets",
            ]
        );
        assert_eq!(
            features.elements.last().unwrap().element.element_type,
            "minecraft:empty_pool_element"
        );
        assert_eq!(features.elements.last().unwrap().weight, 6);
    }

    #[test]
    fn plains_village_template_pool_json_ids_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let plains_ids = registry
            .pools
            .keys()
            .filter(|id| id.starts_with("minecraft:village/plains/"))
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            plains_ids,
            vec![
                "minecraft:village/plains/decor",
                "minecraft:village/plains/houses",
                "minecraft:village/plains/streets",
                "minecraft:village/plains/terminators",
                "minecraft:village/plains/town_centers",
                "minecraft:village/plains/trees",
                "minecraft:village/plains/villagers",
                "minecraft:village/plains/zombie/decor",
                "minecraft:village/plains/zombie/houses",
                "minecraft:village/plains/zombie/streets",
                "minecraft:village/plains/zombie/villagers",
            ]
        );

        let common_ids = registry
            .pools
            .keys()
            .filter(|id| id.starts_with("minecraft:village/common/"))
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            common_ids,
            vec![
                "minecraft:village/common/animals",
                "minecraft:village/common/butcher_animals",
                "minecraft:village/common/cats",
                "minecraft:village/common/iron_golem",
                "minecraft:village/common/sheep",
                "minecraft:village/common/well_bottoms",
            ]
        );
    }

    #[test]
    fn plains_village_town_and_street_pools_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let town_centers = parsed_pool(&registry.pools, "minecraft:village/plains/town_centers");
        assert_eq!(town_centers.fallback, "minecraft:empty");
        assert_eq!(town_centers.elements.len(), 8);
        assert_eq!(pool_weight_sum(town_centers), 204);
        assert_eq!(
            town_centers
                .elements
                .iter()
                .map(|entry| entry.weight)
                .collect::<Vec<_>>(),
            vec![50, 50, 50, 50, 1, 1, 1, 1]
        );
        assert_eq!(
            pool_entry_by_location(
                town_centers,
                "minecraft:village/plains/town_centers/plains_fountain_01"
            )
            .element
            .processors,
            vec!["minecraft:mossify_20_percent".to_string()]
        );
        assert_eq!(
            pool_entry_by_location(
                town_centers,
                "minecraft:village/plains/zombie/town_centers/plains_fountain_01"
            )
            .element
            .processors,
            vec!["minecraft:zombie_plains".to_string()]
        );

        let streets = parsed_pool(&registry.pools, "minecraft:village/plains/streets");
        assert_eq!(streets.fallback, "minecraft:village/plains/terminators");
        assert_eq!(streets.elements.len(), 16);
        assert_eq!(pool_weight_sum(streets), 49);
        assert!(streets
            .elements
            .iter()
            .all(|entry| entry.element.processors == vec!["minecraft:street_plains".to_string()]));
        assert!(streets
            .elements
            .iter()
            .all(|entry| entry.element.projection.as_deref() == Some("terrain_matching")));

        let zombie_streets =
            parsed_pool(&registry.pools, "minecraft:village/plains/zombie/streets");
        assert_eq!(zombie_streets.elements.len(), 16);
        assert_eq!(pool_weight_sum(zombie_streets), 49);
    }

    #[test]
    fn plains_village_house_pools_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let houses = parsed_pool(&registry.pools, "minecraft:village/plains/houses");
        assert_eq!(houses.fallback, "minecraft:village/plains/terminators");
        assert_eq!(houses.elements.len(), 37);
        assert_eq!(pool_weight_sum(houses), 87);
        assert_eq!(
            pool_entry_by_location(
                houses,
                "minecraft:village/plains/houses/plains_small_house_1"
            )
            .element
            .processors,
            vec!["minecraft:mossify_10_percent".to_string()]
        );
        let large_farm = pool_entry_by_location(
            houses,
            "minecraft:village/plains/houses/plains_large_farm_1",
        );
        assert_eq!(
            large_farm.element.processors,
            vec!["minecraft:farm_plains".to_string()]
        );
        assert_eq!(large_farm.weight, 4);
        assert_eq!(
            pool_entry_by_location(
                houses,
                "minecraft:village/plains/houses/plains_meeting_point_4"
            )
            .element
            .processors,
            vec!["minecraft:mossify_70_percent".to_string()]
        );
        assert_eq!(
            houses.elements.last().unwrap().element.element_type,
            "minecraft:empty_pool_element"
        );
        assert_eq!(houses.elements.last().unwrap().weight, 10);

        let zombie_houses = parsed_pool(&registry.pools, "minecraft:village/plains/zombie/houses");
        assert_eq!(
            zombie_houses.fallback,
            "minecraft:village/plains/terminators"
        );
        assert_eq!(zombie_houses.elements.len(), 36);
        assert_eq!(pool_weight_sum(zombie_houses), 83);
        assert_eq!(
            pool_entry_by_location(
                zombie_houses,
                "minecraft:village/plains/zombie/houses/plains_small_house_1"
            )
            .element
            .processors,
            vec!["minecraft:zombie_plains".to_string()]
        );
        assert_eq!(zombie_houses.elements.last().unwrap().weight, 10);
    }

    #[test]
    fn plains_village_decor_tree_villager_and_common_pools_match_java_bootstrap() {
        let registry = load_vanilla_template_pools();
        let trees = parsed_pool(&registry.pools, "minecraft:village/plains/trees");
        assert_eq!(trees.elements.len(), 1);
        assert_eq!(
            trees.elements[0].element.feature.as_deref(),
            Some("minecraft:oak")
        );

        let decor = parsed_pool(&registry.pools, "minecraft:village/plains/decor");
        assert_eq!(pool_weight_sum(decor), 7);
        assert_eq!(
            decor.elements[1].element.feature.as_deref(),
            Some("minecraft:oak")
        );
        assert_eq!(
            decor.elements[2].element.feature.as_deref(),
            Some("minecraft:flower_plain")
        );
        assert_eq!(
            decor.elements[3].element.feature.as_deref(),
            Some("minecraft:pile_hay")
        );
        assert_eq!(decor.elements.last().unwrap().weight, 2);

        let villagers = parsed_pool(&registry.pools, "minecraft:village/plains/villagers");
        assert_eq!(pool_weight_sum(villagers), 12);
        assert_eq!(villagers.elements.len(), 3);
        let zombie_villagers =
            parsed_pool(&registry.pools, "minecraft:village/plains/zombie/villagers");
        assert_eq!(pool_weight_sum(zombie_villagers), 11);
        assert_eq!(zombie_villagers.elements.len(), 2);

        let animals = parsed_pool(&registry.pools, "minecraft:village/common/animals");
        assert_eq!(animals.elements.len(), 10);
        assert_eq!(pool_weight_sum(animals), 26);
        assert_eq!(
            pool_entry_by_location(animals, "minecraft:village/common/animals/cows_1").weight,
            7
        );
        assert_eq!(animals.elements.last().unwrap().weight, 5);

        let cats = parsed_pool(&registry.pools, "minecraft:village/common/cats");
        assert_eq!(cats.elements.len(), 11);
        assert_eq!(pool_weight_sum(cats), 13);
        assert_eq!(
            pool_entry_by_location(cats, "minecraft:village/common/animals/cat_jellie").weight,
            1
        );

        let iron_golem = parsed_pool(&registry.pools, "minecraft:village/common/iron_golem");
        assert_eq!(
            iron_golem.elements[0].element.location.as_deref(),
            Some("minecraft:village/common/iron_golem")
        );
        let well_bottoms = parsed_pool(&registry.pools, "minecraft:village/common/well_bottoms");
        assert_eq!(
            well_bottoms.elements[0].element.location.as_deref(),
            Some("minecraft:village/common/well_bottom")
        );
    }
}
