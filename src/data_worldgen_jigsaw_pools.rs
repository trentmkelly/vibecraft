use crate::worldgen::{JIGSAW_POOL_BOOTSTRAP_SOURCES, JIGSAW_STRUCTURE_START_POOLS};

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
    fn ancient_city_and_bastion_start_pool_metadata_matches_java() {
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
    }
}
