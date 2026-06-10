use super::*;

fn damaged(item: &'static str, max: u32, damage: u32) -> ItemStack {
    let mut stack = ItemStack::new(item, 1);
    stack.set_component(ItemComponent::MaxDamage(max));
    stack.set_damage_value(damage);
    stack
}

fn grid_with(entries: &[(usize, ItemStack)]) -> Vec<ItemStack> {
    let mut grid = vec![ItemStack::empty(); 9];
    for (index, stack) in entries {
        grid[*index] = stack.clone();
    }
    grid
}

#[test]
fn repair_item_sums_remaining_durability_with_five_percent_bonus() {
    // remaining = (100-80) + (100-90) + 100*5/100 = 20 + 10 + 5 = 35
    // result damage = max(100 - 35, 0) = 65   (matches RepairItemRecipe.assemble)
    let grid = grid_with(&[
        (0, damaged("minecraft:diamond_pickaxe", 100, 80)),
        (4, damaged("minecraft:diamond_pickaxe", 100, 90)),
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid)
        .expect("two damaged matching tools should repair");
    assert_eq!(outcome.result.item_id(), "minecraft:diamond_pickaxe");
    assert_eq!(outcome.result.count(), 1);
    assert_eq!(outcome.result.max_damage(), 100);
    assert_eq!(outcome.result.damage_value(), 65);
    // Both single-count inputs are consumed.
    assert!(outcome.grid_after[0].is_empty());
    assert!(outcome.grid_after[4].is_empty());
}

#[test]
fn repair_item_carries_only_curse_enchantments() {
    let mut a = damaged("minecraft:diamond_pickaxe", 100, 80);
    a.set_component(ItemComponent::Enchantments(BTreeMap::from([
        ("minecraft:binding_curse".to_string(), 1),
        ("minecraft:efficiency".to_string(), 5),
    ])));
    let mut b = damaged("minecraft:diamond_pickaxe", 100, 90);
    b.set_component(ItemComponent::Enchantments(BTreeMap::from([(
        "minecraft:vanishing_curse".to_string(),
        1,
    )])));
    let grid = grid_with(&[(0, a), (1, b)]);
    let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).unwrap();
    match outcome.result.component("minecraft:enchantments") {
        Some(ItemComponent::Enchantments(map)) => {
            assert_eq!(map.get("minecraft:binding_curse"), Some(&1));
            assert_eq!(map.get("minecraft:vanishing_curse"), Some(&1));
            assert_eq!(map.get("minecraft:efficiency"), None, "non-curse dropped");
        }
        other => panic!("expected curse enchantments, got {other:?}"),
    }
}

fn banner(item: &'static str, pattern_count: usize) -> ItemStack {
    use crate::block_entity::BannerPatternLayer;
    use crate::map_state::DyeColor;
    let mut stack = ItemStack::new(item, 1);
    if pattern_count > 0 {
        let layers = (0..pattern_count)
            .map(|_| BannerPatternLayer {
                pattern: "minecraft:stripe_top".to_string(),
                color: DyeColor::White,
            })
            .collect();
        stack.set_component(ItemComponent::BannerPatterns(layers));
    }
    stack
}

#[test]
fn shield_decoration_copies_banner_patterns_and_base_color() {
    let hint = ItemAmount::one("minecraft:shield");
    let grid = grid_with(&[
        (0, banner("minecraft:red_banner", 3)),
        (1, ItemStack::new("minecraft:shield", 1)),
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::ShieldDecoration, Some(&hint), &grid)
        .expect("banner + clear shield should decorate");
    assert_eq!(outcome.result.item_id(), "minecraft:shield");
    assert_eq!(
        outcome.result.component("minecraft:base_color"),
        Some(&ItemComponent::BaseColor("red"))
    );
    match outcome.result.component("minecraft:banner_patterns") {
        Some(ItemComponent::BannerPatterns(layers)) => assert_eq!(layers.len(), 3),
        other => panic!("expected 3 pattern layers, got {other:?}"),
    }
    // A shield already carrying patterns is rejected (not a clear target).
    let grid = grid_with(&[
        (0, banner("minecraft:red_banner", 3)),
        (1, banner("minecraft:shield", 1)),
    ]);
    assert!(
        special_crafting_result(SpecialRecipeKind::ShieldDecoration, Some(&hint), &grid).is_none()
    );
}

#[test]
fn banner_duplicate_copies_patterns_and_keeps_source() {
    let hint = ItemAmount::one("minecraft:red_banner");
    let grid = grid_with(&[
        (0, banner("minecraft:red_banner", 4)), // source
        (1, banner("minecraft:red_banner", 0)), // blank target
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::BannerDuplicate, Some(&hint), &grid)
        .expect("patterned + blank same-colour banner should duplicate");
    match outcome.result.component("minecraft:banner_patterns") {
        Some(ItemComponent::BannerPatterns(layers)) => assert_eq!(layers.len(), 4),
        other => panic!("expected copied patterns, got {other:?}"),
    }
    // The patterned source banner is left behind; the blank one is consumed.
    assert_eq!(banner_pattern_count(&outcome.grid_after[0]), 4);
    assert!(outcome.grid_after[1].is_empty());

    // A mismatched recipe colour does not match these inputs.
    let other_hint = ItemAmount::one("minecraft:blue_banner");
    assert!(
        special_crafting_result(SpecialRecipeKind::BannerDuplicate, Some(&other_hint), &grid)
            .is_none()
    );
}

fn written_book(generation: i32) -> ItemStack {
    let mut stack = ItemStack::new("minecraft:written_book", 1);
    stack.set_component(ItemComponent::WrittenBookContent {
        title: "Tale".to_string(),
        author: "Steve".to_string(),
        generation,
        pages: vec!["page".to_string()],
        resolved: true,
    });
    stack
}

#[test]
fn book_cloning_copies_at_next_generation_and_keeps_source() {
    let hint = ItemAmount::one("minecraft:written_book");
    let grid = grid_with(&[
        (0, written_book(0)),
        (1, ItemStack::new("minecraft:writable_book", 1)),
        (2, ItemStack::new("minecraft:writable_book", 1)),
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::BookCloning, Some(&hint), &grid)
        .expect("written book + 2 writable books should clone");
    assert_eq!(outcome.result.item_id(), "minecraft:written_book");
    assert_eq!(outcome.result.count(), 2, "one copy per writable book");
    assert_eq!(written_book_generation(&outcome.result), Some(1));
    // Source written book retained, writable books consumed.
    assert_eq!(outcome.grid_after[0].item_id(), "minecraft:written_book");
    assert!(outcome.grid_after[1].is_empty());
    assert!(outcome.grid_after[2].is_empty());

    // A generation-2 source is out of the allowed 0..=1 range.
    let grid = grid_with(&[
        (0, written_book(2)),
        (1, ItemStack::new("minecraft:writable_book", 1)),
    ]);
    assert!(special_crafting_result(SpecialRecipeKind::BookCloning, Some(&hint), &grid).is_none());
}

#[test]
fn decorated_pot_records_four_faces() {
    let hint = ItemAmount::one("minecraft:decorated_pot");
    let grid = grid_with(&[
        (1, ItemStack::new("minecraft:brick", 1)),
        (3, ItemStack::new("minecraft:angler_pottery_sherd", 1)),
        (5, ItemStack::new("minecraft:brick", 1)),
        (7, ItemStack::new("minecraft:skull_pottery_sherd", 1)),
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::DecoratedPot, Some(&hint), &grid)
        .expect("four cardinal pot ingredients should craft a decorated pot");
    assert_eq!(outcome.result.item_id(), "minecraft:decorated_pot");
    assert_eq!(
        outcome.result.component("minecraft:pot_decorations"),
        Some(&ItemComponent::PotDecorations {
            back: "minecraft:brick",
            left: "minecraft:angler_pottery_sherd",
            right: "minecraft:brick",
            front: "minecraft:skull_pottery_sherd",
        })
    );

    // An ingredient off the cardinal slots is rejected.
    let grid = grid_with(&[
        (0, ItemStack::new("minecraft:brick", 1)),
        (3, ItemStack::new("minecraft:brick", 1)),
        (5, ItemStack::new("minecraft:brick", 1)),
        (7, ItemStack::new("minecraft:brick", 1)),
    ]);
    assert!(special_crafting_result(SpecialRecipeKind::DecoratedPot, Some(&hint), &grid).is_none());
}

#[test]
fn dyed_item_blends_dye_into_dyed_color() {
    // A single red dye on an uncoloured leather helmet yields exactly the dye's
    // texture-diffuse colour (DyeColor.RED = 11546150).
    let hint = ItemAmount::one("minecraft:leather_helmet");
    let grid = grid_with(&[
        (0, ItemStack::new("minecraft:leather_helmet", 1)),
        (1, ItemStack::new("minecraft:red_dye", 1)),
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::DyedItem, Some(&hint), &grid)
        .expect("leather + dye should dye");
    assert_eq!(
        outcome.result.component("minecraft:dyed_color"),
        Some(&ItemComponent::DyedColor(11546150))
    );
}

#[test]
fn firework_rocket_flight_duration_is_gunpowder_count() {
    let hint = ItemAmount {
        item: "minecraft:firework_rocket",
        count: 3,
    };
    let grid = grid_with(&[
        (0, ItemStack::new("minecraft:paper", 1)),
        (1, ItemStack::new("minecraft:gunpowder", 1)),
        (2, ItemStack::new("minecraft:gunpowder", 1)),
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::FireworkRocket, Some(&hint), &grid)
        .expect("paper + gunpowder should craft a rocket");
    assert_eq!(outcome.result.count(), 3);
    assert_eq!(
        outcome.result.component("minecraft:fireworks"),
        Some(&ItemComponent::Fireworks {
            flight_duration: 2,
            explosions: Vec::new(),
        })
    );
    // Four gunpowder exceeds the flight-duration cap.
    let grid = grid_with(&[
        (0, ItemStack::new("minecraft:paper", 1)),
        (1, ItemStack::new("minecraft:gunpowder", 1)),
        (2, ItemStack::new("minecraft:gunpowder", 1)),
        (3, ItemStack::new("minecraft:gunpowder", 1)),
        (4, ItemStack::new("minecraft:gunpowder", 1)),
    ]);
    assert!(
        special_crafting_result(SpecialRecipeKind::FireworkRocket, Some(&hint), &grid).is_none()
    );
}

#[test]
fn firework_star_assembles_shape_color_and_modifiers() {
    use crate::item_properties::FireworkExplosion;
    let hint = ItemAmount::one("minecraft:firework_star");
    let grid = grid_with(&[
        (0, ItemStack::new("minecraft:gunpowder", 1)),
        (1, ItemStack::new("minecraft:red_dye", 1)),
        (2, ItemStack::new("minecraft:fire_charge", 1)), // large_ball
        (3, ItemStack::new("minecraft:glowstone_dust", 1)), // twinkle
    ]);
    let outcome = special_crafting_result(SpecialRecipeKind::FireworkStar, Some(&hint), &grid)
        .expect("gunpowder + dye should craft a star");
    assert_eq!(
        outcome.result.component("minecraft:firework_explosion"),
        Some(&ItemComponent::FireworkExplosion(FireworkExplosion {
            shape: "large_ball",
            colors: vec![11743532], // DyeColor.RED firework colour
            fade_colors: Vec::new(),
            trail: false,
            twinkle: true,
        }))
    );
}

#[test]
fn firework_star_fade_adds_fade_colors() {
    use crate::item_properties::FireworkExplosion;
    let hint = ItemAmount::one("minecraft:firework_star");
    let mut star = ItemStack::new("minecraft:firework_star", 1);
    star.set_component(ItemComponent::FireworkExplosion(FireworkExplosion {
        shape: "small_ball",
        colors: vec![11546150],
        fade_colors: Vec::new(),
        trail: false,
        twinkle: false,
    }));
    let grid = grid_with(&[(0, star), (1, ItemStack::new("minecraft:blue_dye", 1))]);
    let outcome = special_crafting_result(SpecialRecipeKind::FireworkStarFade, Some(&hint), &grid)
        .expect("star + dye should add fade colours");
    match outcome.result.component("minecraft:firework_explosion") {
        Some(ItemComponent::FireworkExplosion(explosion)) => {
            assert_eq!(explosion.fade_colors, vec![2437522]); // DyeColor.BLUE firework
            assert_eq!(explosion.colors, vec![11546150], "original colours kept");
        }
        other => panic!("expected explosion, got {other:?}"),
    }
}

#[test]
fn map_extending_requires_scalable_non_exploration_map_and_paper_ring() {
    let mut store = MapDataStore::default();
    store.insert(
        7,
        MapCraftingData {
            scale: 2,
            exploration_map: false,
        },
    );

    let map_grid = |center: ItemStack| {
        let mut grid = vec![ItemStack::new("minecraft:paper", 1); 9];
        grid[4] = center;
        grid
    };
    let filled_map = |id: i32| {
        let mut stack = ItemStack::new("minecraft:filled_map", 1);
        stack.set_component(ItemComponent::MapId(id));
        stack
    };

    // Extendable map -> filled_map + MAP_POST_PROCESSING=SCALE, keeping the id.
    let outcome = map_extending_result(&map_grid(filled_map(7)), &store)
        .expect("scale-2 non-exploration map surrounded by paper should extend");
    assert_eq!(outcome.result.item_id(), "minecraft:filled_map");
    assert_eq!(
        outcome.result.component("minecraft:map_id"),
        Some(&ItemComponent::MapId(7))
    );
    assert_eq!(
        outcome.result.component("minecraft:map_post_processing"),
        Some(&ItemComponent::MapPostProcessing(
            crate::item_properties::MapPostProcessing::Scale
        ))
    );

    // No saved-data entry for the map id -> no match.
    let mut empty = MapDataStore::default();
    empty.insert(
        99,
        MapCraftingData {
            scale: 0,
            exploration_map: false,
        },
    );
    assert!(map_extending_result(&map_grid(filled_map(7)), &empty).is_none());

    // A non-paper ring item breaks the pattern.
    let mut grid = map_grid(filled_map(7));
    grid[0] = ItemStack::new("minecraft:stick", 1);
    assert!(map_extending_result(&grid, &store).is_none());
}

#[test]
fn repair_item_rejects_mismatched_or_multi_count_inputs() {
    // Different items.
    let grid = grid_with(&[
        (0, damaged("minecraft:diamond_pickaxe", 100, 80)),
        (1, damaged("minecraft:iron_pickaxe", 100, 80)),
    ]);
    assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());

    // A stack of two is rejected.
    let mut pair = damaged("minecraft:diamond_pickaxe", 100, 80);
    pair.set_count(2);
    let grid = grid_with(&[
        (0, pair),
        (1, damaged("minecraft:diamond_pickaxe", 100, 80)),
    ]);
    assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());

    // A non-damageable item is rejected.
    let grid = grid_with(&[
        (0, ItemStack::new("minecraft:stone", 1)),
        (1, ItemStack::new("minecraft:stone", 1)),
    ]);
    assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());
}
