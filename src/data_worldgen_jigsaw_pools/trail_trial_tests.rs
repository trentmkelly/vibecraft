use super::*;

fn load_vanilla_template_pools() -> crate::worldgen::ParsedTemplatePoolRegistry {
    load_template_pool_registry("../decompiled-server-26.1.2/data/minecraft/worldgen/template_pool")
        .expect("vanilla template-pool registry should load")
}

fn assert_single_processor_pool(
    pool: &ParsedJigsawTemplatePool,
    element_count: usize,
    processor: &str,
) {
    assert_eq!(pool.fallback, "minecraft:empty");
    assert_eq!(pool.elements.len(), element_count);
    assert_eq!(pool_weight_sum(pool), element_count as i32);
    assert!(pool.elements.iter().all(|entry| entry.weight == 1));
    assert!(pool.elements.iter().all(|entry| {
        entry.element.element_type == "minecraft:single_pool_element"
            && entry.element.processors.len() == 1
            && entry.element.processors[0] == processor
            && entry.element.projection.as_deref() == Some("rigid")
    }));
}

#[test]
fn trail_ruins_template_pool_json_ids_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let trail_ruins_ids = registry
        .pools
        .keys()
        .filter(|id| id.starts_with("minecraft:trail_ruins/"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        trail_ruins_ids,
        vec![
            "minecraft:trail_ruins/buildings",
            "minecraft:trail_ruins/buildings/grouped",
            "minecraft:trail_ruins/decor",
            "minecraft:trail_ruins/roads",
            "minecraft:trail_ruins/tower",
            "minecraft:trail_ruins/tower/additions",
            "minecraft:trail_ruins/tower/tower_top",
        ]
    );
}

#[test]
fn trail_ruins_tower_and_road_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let tower = parsed_pool(&registry.pools, "minecraft:trail_ruins/tower");
    assert_single_processor_pool(tower, 5, "minecraft:trail_ruins_houses_archaeology");
    assert_eq!(
        tower.elements[0].element.location.as_deref(),
        Some("minecraft:trail_ruins/tower/tower_1")
    );
    assert_eq!(
        tower.elements[4].element.location.as_deref(),
        Some("minecraft:trail_ruins/tower/tower_5")
    );

    let tower_top = parsed_pool(&registry.pools, "minecraft:trail_ruins/tower/tower_top");
    assert_single_processor_pool(tower_top, 5, "minecraft:trail_ruins_tower_top_archaeology");
    assert_eq!(
        tower_top.elements[4].element.location.as_deref(),
        Some("minecraft:trail_ruins/tower/tower_top_5")
    );

    let additions = parsed_pool(&registry.pools, "minecraft:trail_ruins/tower/additions");
    assert_single_processor_pool(additions, 25, "minecraft:trail_ruins_houses_archaeology");
    assert_eq!(
        additions.elements[0].element.location.as_deref(),
        Some("minecraft:trail_ruins/tower/hall_1")
    );
    assert_eq!(
        additions.elements[24].element.location.as_deref(),
        Some("minecraft:trail_ruins/tower/stable_5")
    );

    let roads = parsed_pool(&registry.pools, "minecraft:trail_ruins/roads");
    assert_single_processor_pool(roads, 7, "minecraft:trail_ruins_roads_archaeology");
    assert_eq!(
        roads.elements[0].element.location.as_deref(),
        Some("minecraft:trail_ruins/roads/long_road_end")
    );
    assert_eq!(
        roads.elements[6].element.location.as_deref(),
        Some("minecraft:trail_ruins/roads/road_spacer_1")
    );
}

#[test]
fn trail_ruins_building_and_decor_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    for (id, element_count) in [
        ("minecraft:trail_ruins/buildings", 15),
        ("minecraft:trail_ruins/buildings/grouped", 20),
        ("minecraft:trail_ruins/decor", 7),
    ] {
        assert_single_processor_pool(
            parsed_pool(&registry.pools, id),
            element_count,
            "minecraft:trail_ruins_houses_archaeology",
        );
    }

    let buildings = parsed_pool(&registry.pools, "minecraft:trail_ruins/buildings");
    assert_eq!(
        buildings.elements[0].element.location.as_deref(),
        Some("minecraft:trail_ruins/buildings/group_hall_1")
    );
    assert_eq!(
        buildings.elements[14].element.location.as_deref(),
        Some("minecraft:trail_ruins/buildings/one_room_5")
    );

    let grouped = parsed_pool(&registry.pools, "minecraft:trail_ruins/buildings/grouped");
    assert_eq!(
        grouped.elements[0].element.location.as_deref(),
        Some("minecraft:trail_ruins/buildings/group_full_1")
    );
    assert_eq!(
        grouped.elements[19].element.location.as_deref(),
        Some("minecraft:trail_ruins/buildings/group_room_5")
    );

    let decor = parsed_pool(&registry.pools, "minecraft:trail_ruins/decor");
    assert_eq!(
        decor.elements[6].element.location.as_deref(),
        Some("minecraft:trail_ruins/decor/decor_7")
    );
}

const TRIAL_CHAMBERS_POOL_SUMMARIES: &[(&str, &str, usize, i32)] = &[
    ("minecraft:trial_chambers/atrium", "minecraft:empty", 7, 7),
    (
        "minecraft:trial_chambers/chamber/addon",
        "minecraft:empty",
        10,
        10,
    ),
    (
        "minecraft:trial_chambers/chamber/assembly",
        "minecraft:empty",
        21,
        39,
    ),
    (
        "minecraft:trial_chambers/chamber/end",
        "minecraft:empty",
        2,
        2,
    ),
    (
        "minecraft:trial_chambers/chamber/entrance_cap",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/chamber/eruption",
        "minecraft:empty",
        10,
        10,
    ),
    (
        "minecraft:trial_chambers/chamber/pedestal",
        "minecraft:empty",
        14,
        22,
    ),
    (
        "minecraft:trial_chambers/chamber/slanted",
        "minecraft:empty",
        13,
        13,
    ),
    (
        "minecraft:trial_chambers/chambers/end",
        "minecraft:trial_chambers/hallway/fallback",
        4,
        4,
    ),
    (
        "minecraft:trial_chambers/chests/contents/supply",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/chests/supply",
        "minecraft:empty",
        1,
        1,
    ),
    ("minecraft:trial_chambers/corridor", "minecraft:empty", 9, 9),
    (
        "minecraft:trial_chambers/corridor/slices",
        "minecraft:empty",
        8,
        14,
    ),
    (
        "minecraft:trial_chambers/corridors/addon/lower",
        "minecraft:empty",
        6,
        14,
    ),
    (
        "minecraft:trial_chambers/corridors/addon/middle",
        "minecraft:empty",
        3,
        11,
    ),
    (
        "minecraft:trial_chambers/corridors/addon/middle_upper",
        "minecraft:empty",
        6,
        12,
    ),
    ("minecraft:trial_chambers/decor", "minecraft:empty", 12, 45),
    (
        "minecraft:trial_chambers/decor/bed",
        "minecraft:empty",
        16,
        46,
    ),
    (
        "minecraft:trial_chambers/decor/chamber",
        "minecraft:empty",
        2,
        5,
    ),
    (
        "minecraft:trial_chambers/decor/disposal",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/dispensers/chamber",
        "minecraft:empty",
        4,
        4,
    ),
    ("minecraft:trial_chambers/entrance", "minecraft:empty", 3, 3),
    (
        "minecraft:trial_chambers/hallway",
        "minecraft:trial_chambers/hallway/fallback",
        30,
        1231,
    ),
    (
        "minecraft:trial_chambers/hallway/fallback",
        "minecraft:empty",
        4,
        4,
    ),
    (
        "minecraft:trial_chambers/reward/all",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/reward/contents/default",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/reward/ominous_vault",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/all",
        "minecraft:empty",
        3,
        3,
    ),
    (
        "minecraft:trial_chambers/spawner/breeze",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/contents/breeze",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/melee",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/melee/husk",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/melee/spider",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/melee/zombie",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/ranged",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/ranged/poison_skeleton",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/ranged/skeleton",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/ranged/stray",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/slow_ranged",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/slow_ranged/poison_skeleton",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/slow_ranged/skeleton",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/slow_ranged/stray",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/small_melee",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/small_melee/baby_zombie",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/small_melee/cave_spider",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/small_melee/silverfish",
        "minecraft:empty",
        1,
        1,
    ),
    (
        "minecraft:trial_chambers/spawner/small_melee/slime",
        "minecraft:empty",
        1,
        1,
    ),
];

#[test]
fn trial_chambers_template_pool_json_ids_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let trial_chambers_ids = registry
        .pools
        .keys()
        .filter(|id| id.starts_with("minecraft:trial_chambers/"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        trial_chambers_ids,
        TRIAL_CHAMBERS_POOL_SUMMARIES
            .iter()
            .map(|(id, _, _, _)| *id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn trial_chambers_template_pool_summaries_match_extracted_json() {
    let registry = load_vanilla_template_pools();
    for (id, fallback, element_count, weight_sum) in TRIAL_CHAMBERS_POOL_SUMMARIES {
        let pool = parsed_pool(&registry.pools, id);
        assert_eq!(&pool.fallback, fallback, "fallback mismatch for {id}");
        assert_eq!(
            pool.elements.len(),
            *element_count,
            "element count mismatch for {id}"
        );
        assert_eq!(
            pool_weight_sum(pool),
            *weight_sum,
            "weight sum mismatch for {id}"
        );
        assert!(pool.elements.iter().all(|entry| {
            entry.element.element_type == "minecraft:empty_pool_element"
                || entry.element.projection.as_deref() == Some("rigid")
        }));
    }
}

#[test]
fn trial_chambers_representative_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let start = parsed_pool(&registry.pools, "minecraft:trial_chambers/chamber/end");
    assert_eq!(
        start.elements[0].element.location.as_deref(),
        Some("minecraft:trial_chambers/corridor/end_1")
    );
    assert_eq!(
        start.elements[0].element.processors,
        vec!["minecraft:trial_chambers_copper_bulb_degradation".to_string()]
    );

    let hallway = parsed_pool(&registry.pools, "minecraft:trial_chambers/hallway");
    assert_eq!(
        pool_entry_by_location(hallway, "minecraft:trial_chambers/chamber/chamber_8").weight,
        150
    );
    assert_eq!(
        pool_entry_by_location(hallway, "minecraft:trial_chambers/hallway/rubble_chamber").weight,
        10
    );

    let lower_addons = parsed_pool(
        &registry.pools,
        "minecraft:trial_chambers/corridors/addon/lower",
    );
    assert_eq!(
        lower_addons.elements[0].element.element_type,
        "minecraft:empty_pool_element"
    );
    assert_eq!(lower_addons.elements[0].weight, 8);

    let decor = parsed_pool(&registry.pools, "minecraft:trial_chambers/decor");
    assert_eq!(decor.elements[0].weight, 22);
    assert_eq!(
        pool_entry_by_location(decor, "minecraft:trial_chambers/decor/undecorated_pot").weight,
        10
    );

    let spawners = parsed_pool(&registry.pools, "minecraft:trial_chambers/spawner/all");
    assert_eq!(
        spawners
            .elements
            .iter()
            .filter_map(|entry| entry.element.location.as_deref())
            .collect::<Vec<_>>(),
        vec![
            "minecraft:trial_chambers/spawner/connectors/ranged",
            "minecraft:trial_chambers/spawner/connectors/melee",
            "minecraft:trial_chambers/spawner/connectors/small_melee",
        ]
    );
}
