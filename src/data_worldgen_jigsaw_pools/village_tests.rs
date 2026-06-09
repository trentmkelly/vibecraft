use super::*;

fn load_vanilla_template_pools() -> crate::worldgen::ParsedTemplatePoolRegistry {
    load_template_pool_registry(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/template_pool",
    )
    .expect("vanilla template-pool registry should load")
}

#[test]
fn savanna_village_template_pool_json_ids_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let savanna_ids = registry
        .pools
        .keys()
        .filter(|id| id.starts_with("minecraft:village/savanna/"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        savanna_ids,
        vec![
            "minecraft:village/savanna/decor",
            "minecraft:village/savanna/houses",
            "minecraft:village/savanna/streets",
            "minecraft:village/savanna/terminators",
            "minecraft:village/savanna/town_centers",
            "minecraft:village/savanna/trees",
            "minecraft:village/savanna/villagers",
            "minecraft:village/savanna/zombie/decor",
            "minecraft:village/savanna/zombie/houses",
            "minecraft:village/savanna/zombie/streets",
            "minecraft:village/savanna/zombie/terminators",
            "minecraft:village/savanna/zombie/villagers",
        ]
    );
}

#[test]
fn savanna_village_town_street_and_terminator_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let town_centers = parsed_pool(&registry.pools, "minecraft:village/savanna/town_centers");
    assert_eq!(town_centers.fallback, "minecraft:empty");
    assert_eq!(town_centers.elements.len(), 8);
    assert_eq!(pool_weight_sum(town_centers), 459);
    assert_eq!(
        town_centers
            .elements
            .iter()
            .map(|entry| entry.weight)
            .collect::<Vec<_>>(),
        vec![100, 50, 150, 150, 2, 1, 3, 3]
    );
    assert_eq!(
        pool_entry_by_location(
            town_centers,
            "minecraft:village/savanna/zombie/town_centers/savanna_meeting_point_4",
        )
        .element
        .processors,
        vec!["minecraft:zombie_savanna".to_string()]
    );

    let streets = parsed_pool(&registry.pools, "minecraft:village/savanna/streets");
    assert_eq!(streets.fallback, "minecraft:village/savanna/terminators");
    assert_eq!(streets.elements.len(), 19);
    assert_eq!(pool_weight_sum(streets), 56);
    assert!(streets
        .elements
        .iter()
        .all(|entry| entry.element.processors == vec!["minecraft:street_savanna".to_string()]));
    assert!(streets
        .elements
        .iter()
        .all(|entry| entry.element.projection.as_deref() == Some("terrain_matching")));
    assert_eq!(
        pool_entry_by_location(streets, "minecraft:village/savanna/streets/straight_04").weight,
        7
    );

    let zombie_streets = parsed_pool(&registry.pools, "minecraft:village/savanna/zombie/streets");
    assert_eq!(
        zombie_streets.fallback,
        "minecraft:village/savanna/zombie/terminators"
    );
    assert_eq!(zombie_streets.elements.len(), 19);
    assert_eq!(pool_weight_sum(zombie_streets), 56);

    let terminators = parsed_pool(&registry.pools, "minecraft:village/savanna/terminators");
    assert_eq!(terminators.fallback, "minecraft:empty");
    assert_eq!(terminators.elements.len(), 5);
    assert_eq!(
        terminators.elements[4].element.location.as_deref(),
        Some("minecraft:village/savanna/terminators/terminator_05")
    );
    let zombie_terminators =
        parsed_pool(&registry.pools, "minecraft:village/savanna/zombie/terminators");
    assert_eq!(zombie_terminators.fallback, "minecraft:empty");
    assert_eq!(
        zombie_terminators.elements[4].element.location.as_deref(),
        Some("minecraft:village/savanna/zombie/terminators/terminator_05")
    );
}

#[test]
fn savanna_village_house_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let houses = parsed_pool(&registry.pools, "minecraft:village/savanna/houses");
    assert_eq!(houses.fallback, "minecraft:village/savanna/terminators");
    assert_eq!(houses.elements.len(), 32);
    assert_eq!(pool_weight_sum(houses), 81);
    let large_farm = pool_entry_by_location(
        houses,
        "minecraft:village/savanna/houses/savanna_large_farm_2",
    );
    assert_eq!(
        large_farm.element.processors,
        vec!["minecraft:farm_savanna".to_string()]
    );
    assert_eq!(large_farm.weight, 6);
    assert_eq!(
        houses.elements.last().unwrap().element.element_type,
        "minecraft:empty_pool_element"
    );
    assert_eq!(houses.elements.last().unwrap().weight, 5);

    let zombie_houses = parsed_pool(&registry.pools, "minecraft:village/savanna/zombie/houses");
    assert_eq!(
        zombie_houses.fallback,
        "minecraft:village/savanna/zombie/terminators"
    );
    assert_eq!(zombie_houses.elements.len(), 32);
    assert_eq!(pool_weight_sum(zombie_houses), 72);
    assert_eq!(
        pool_entry_by_location(
            zombie_houses,
            "minecraft:village/savanna/zombie/houses/savanna_large_farm_2",
        )
        .weight,
        4
    );
    assert_eq!(
        pool_entry_by_location(
            zombie_houses,
            "minecraft:village/savanna/houses/savanna_butchers_shop_1",
        )
        .element
        .processors,
        vec!["minecraft:zombie_savanna".to_string()]
    );
}

#[test]
fn savanna_village_decor_tree_and_villager_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let trees = parsed_pool(&registry.pools, "minecraft:village/savanna/trees");
    assert_eq!(trees.elements.len(), 1);
    assert_eq!(
        trees.elements[0].element.feature.as_deref(),
        Some("minecraft:acacia")
    );

    let decor = parsed_pool(&registry.pools, "minecraft:village/savanna/decor");
    assert_eq!(decor.fallback, "minecraft:empty");
    assert_eq!(pool_weight_sum(decor), 17);
    assert_eq!(
        decor.elements[1].element.feature.as_deref(),
        Some("minecraft:acacia")
    );
    assert_eq!(
        decor.elements[2].element.feature.as_deref(),
        Some("minecraft:pile_hay")
    );
    assert_eq!(
        decor.elements[3].element.feature.as_deref(),
        Some("minecraft:pile_melon")
    );
    assert_eq!(decor.elements.last().unwrap().weight, 4);

    let zombie_decor = parsed_pool(&registry.pools, "minecraft:village/savanna/zombie/decor");
    assert_eq!(
        zombie_decor.elements[0].element.processors,
        vec!["minecraft:zombie_savanna".to_string()]
    );
    assert_eq!(pool_weight_sum(zombie_decor), 17);

    let villagers = parsed_pool(&registry.pools, "minecraft:village/savanna/villagers");
    assert_eq!(pool_weight_sum(villagers), 12);
    assert_eq!(villagers.elements.len(), 3);
    let zombie_villagers =
        parsed_pool(&registry.pools, "minecraft:village/savanna/zombie/villagers");
    assert_eq!(pool_weight_sum(zombie_villagers), 11);
    assert_eq!(zombie_villagers.elements.len(), 2);
}

#[test]
fn snowy_village_template_pool_json_ids_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let snowy_ids = registry
        .pools
        .keys()
        .filter(|id| id.starts_with("minecraft:village/snowy/"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        snowy_ids,
        vec![
            "minecraft:village/snowy/decor",
            "minecraft:village/snowy/houses",
            "minecraft:village/snowy/streets",
            "minecraft:village/snowy/terminators",
            "minecraft:village/snowy/town_centers",
            "minecraft:village/snowy/trees",
            "minecraft:village/snowy/villagers",
            "minecraft:village/snowy/zombie/decor",
            "minecraft:village/snowy/zombie/houses",
            "minecraft:village/snowy/zombie/streets",
            "minecraft:village/snowy/zombie/villagers",
        ]
    );
}

#[test]
fn snowy_village_town_street_and_terminator_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let town_centers = parsed_pool(&registry.pools, "minecraft:village/snowy/town_centers");
    assert_eq!(town_centers.fallback, "minecraft:empty");
    assert_eq!(town_centers.elements.len(), 6);
    assert_eq!(pool_weight_sum(town_centers), 306);
    assert_eq!(
        town_centers
            .elements
            .iter()
            .map(|entry| entry.weight)
            .collect::<Vec<_>>(),
        vec![100, 50, 150, 2, 1, 3]
    );
    assert_eq!(
        pool_entry_by_location(
            town_centers,
            "minecraft:village/snowy/zombie/town_centers/snowy_meeting_point_3",
        )
        .element
        .processors,
        Vec::<String>::new()
    );

    let streets = parsed_pool(&registry.pools, "minecraft:village/snowy/streets");
    assert_eq!(streets.fallback, "minecraft:village/snowy/terminators");
    assert_eq!(streets.elements.len(), 16);
    assert_eq!(pool_weight_sum(streets), 47);
    assert!(streets.elements.iter().all(|entry| entry.element.processors
        == vec!["minecraft:street_snowy_or_taiga".to_string()]));
    assert!(streets
        .elements
        .iter()
        .all(|entry| entry.element.projection.as_deref() == Some("terrain_matching")));
    assert_eq!(
        pool_entry_by_location(streets, "minecraft:village/snowy/streets/straight_04").weight,
        7
    );

    let zombie_streets = parsed_pool(&registry.pools, "minecraft:village/snowy/zombie/streets");
    assert_eq!(
        zombie_streets.fallback,
        "minecraft:village/snowy/terminators"
    );
    assert_eq!(zombie_streets.elements.len(), 16);
    assert_eq!(pool_weight_sum(zombie_streets), 47);

    let terminators = parsed_pool(&registry.pools, "minecraft:village/snowy/terminators");
    assert_eq!(terminators.fallback, "minecraft:empty");
    assert_eq!(terminators.elements.len(), 4);
    assert!(terminators.elements.iter().all(|entry| entry.element.processors
        == vec!["minecraft:street_snowy_or_taiga".to_string()]));
}

#[test]
fn snowy_village_house_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let houses = parsed_pool(&registry.pools, "minecraft:village/snowy/houses");
    assert_eq!(houses.fallback, "minecraft:village/snowy/terminators");
    assert_eq!(houses.elements.len(), 31);
    assert_eq!(pool_weight_sum(houses), 68);
    let snowy_farm = pool_entry_by_location(houses, "minecraft:village/snowy/houses/snowy_farm_2");
    assert_eq!(
        snowy_farm.element.processors,
        vec!["minecraft:farm_snowy".to_string()]
    );
    assert_eq!(snowy_farm.weight, 3);
    assert_eq!(
        houses.elements.last().unwrap().element.element_type,
        "minecraft:empty_pool_element"
    );
    assert_eq!(houses.elements.last().unwrap().weight, 6);

    let zombie_houses = parsed_pool(&registry.pools, "minecraft:village/snowy/zombie/houses");
    assert_eq!(
        zombie_houses.fallback,
        "minecraft:village/snowy/terminators"
    );
    assert_eq!(zombie_houses.elements.len(), 31);
    assert_eq!(pool_weight_sum(zombie_houses), 65);
    assert_eq!(
        pool_entry_by_location(
            zombie_houses,
            "minecraft:village/snowy/zombie/houses/snowy_medium_house_3",
        )
        .weight,
        1
    );
    assert_eq!(
        pool_entry_by_location(
            zombie_houses,
            "minecraft:village/snowy/houses/snowy_butchers_shop_1",
        )
        .element
        .processors,
        vec!["minecraft:zombie_snowy".to_string()]
    );
}

#[test]
fn snowy_village_decor_tree_and_villager_pools_match_java_bootstrap() {
    let registry = load_vanilla_template_pools();
    let trees = parsed_pool(&registry.pools, "minecraft:village/snowy/trees");
    assert_eq!(trees.elements.len(), 1);
    assert_eq!(
        trees.elements[0].element.feature.as_deref(),
        Some("minecraft:spruce")
    );

    let decor = parsed_pool(&registry.pools, "minecraft:village/snowy/decor");
    assert_eq!(decor.fallback, "minecraft:empty");
    assert_eq!(pool_weight_sum(decor), 27);
    assert_eq!(
        decor.elements[3].element.feature.as_deref(),
        Some("minecraft:spruce")
    );
    assert_eq!(
        decor.elements[4].element.feature.as_deref(),
        Some("minecraft:pile_snow")
    );
    assert_eq!(
        decor.elements[5].element.feature.as_deref(),
        Some("minecraft:pile_ice")
    );
    assert_eq!(decor.elements.last().unwrap().weight, 9);

    let zombie_decor = parsed_pool(&registry.pools, "minecraft:village/snowy/zombie/decor");
    assert_eq!(
        zombie_decor.elements[0].element.processors,
        vec!["minecraft:zombie_snowy".to_string()]
    );
    assert_eq!(pool_weight_sum(zombie_decor), 22);
    assert_eq!(
        zombie_decor.elements[5].element.feature.as_deref(),
        Some("minecraft:pile_ice")
    );

    let villagers = parsed_pool(&registry.pools, "minecraft:village/snowy/villagers");
    assert_eq!(pool_weight_sum(villagers), 12);
    assert_eq!(villagers.elements.len(), 3);
    let zombie_villagers =
        parsed_pool(&registry.pools, "minecraft:village/snowy/zombie/villagers");
    assert_eq!(pool_weight_sum(zombie_villagers), 11);
    assert_eq!(zombie_villagers.elements.len(), 2);
}
