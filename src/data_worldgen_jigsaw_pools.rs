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
    fn ancient_city_bastion_desert_and_outpost_start_pool_metadata_matches_java() {
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
    }

    fn load_vanilla_template_pools() -> crate::worldgen::ParsedTemplatePoolRegistry {
        load_template_pool_registry(
            "../decompiled-server-26.1.2/data/minecraft/worldgen/template_pool",
        )
        .expect("vanilla template-pool registry should load")
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
}
