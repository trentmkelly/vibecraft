#![cfg_attr(
    not(vibecraft_has_decompiled_sources),
    allow(unused_imports)
)]

use super::*;
use crate::advancement_system::{
    PlayerAdvancementSet, PlayerRecipeUnlocks, RecipeDefinition as AdvancementRecipeDefinition,
    RecipeUnlockedCriterion,
};

mod registry_tests;

#[test]
fn recipe_book_settings_use_vanilla_stream_order_and_defaults() {
    let mut settings = RecipeBookSettings::default();
    assert_eq!(
        settings.stream_order(),
        [RecipeBookTypeSettings::default(); 4]
    );

    settings.set_open(RecipeBookType::Crafting, true);
    settings.set_filtering(RecipeBookType::BlastFurnace, true);

    assert_eq!(
        settings.stream_order(),
        [
            RecipeBookTypeSettings {
                open: true,
                filtering: false
            },
            RecipeBookTypeSettings::default(),
            RecipeBookTypeSettings {
                open: false,
                filtering: true
            },
            RecipeBookTypeSettings::default()
        ]
    );
    assert!(settings.get(RecipeBookType::Crafting).open);
    assert!(!settings.get(RecipeBookType::Smoker).open);
}

#[test]
fn recipe_book_add_flags_match_vanilla_bits() {
    let flags = RecipeBookEntryFlags::new(true, true);
    assert_eq!(flags.bits(), 3);
    assert!(flags.notification());
    assert!(flags.highlight());

    let highlight_only = RecipeBookEntryFlags::new(false, true);
    assert_eq!(highlight_only.bits(), 2);
    assert!(!highlight_only.notification());
    assert!(highlight_only.highlight());
}

#[test]
fn recipe_display_entries_require_explicit_crafting_requirements() {
    let entry = RecipeDisplayEntry {
        id: RecipeDisplayId(7),
        display: RecipeDisplay::ShapelessCrafting {
            ingredients: vec![SlotDisplay::Item("minecraft:oak_planks")],
            result: SlotDisplay::ItemStack {
                item: "minecraft:stick",
                count: 4,
            },
            crafting_station: SlotDisplay::Item("minecraft:crafting_table"),
        },
        group: Some(1),
        category: "crafting_misc",
        crafting_requirements: Some(vec![SlotDisplay::Composite(vec![
            SlotDisplay::Item("minecraft:oak_planks"),
            SlotDisplay::Item("minecraft:birch_planks"),
        ])]),
    };

    assert!(entry.can_craft(&["minecraft:birch_planks"]));
    assert!(!entry.can_craft(&["minecraft:cobblestone"]));

    let without_requirements = RecipeDisplayEntry {
        crafting_requirements: None,
        ..entry
    };
    assert!(!without_requirements.can_craft(&["minecraft:oak_planks"]));
}

#[test]
fn recipe_packets_carry_update_add_remove_and_settings_payloads() {
    let display_entry = RecipeDisplayEntry {
        id: RecipeDisplayId(3),
        display: RecipeDisplay::Stonecutter {
            ingredient: SlotDisplay::Item("minecraft:stone"),
            result: SlotDisplay::Item("minecraft:stone_slab"),
            crafting_station: SlotDisplay::Item("minecraft:stonecutter"),
        },
        group: None,
        category: "stonecutter",
        crafting_requirements: Some(vec![SlotDisplay::Item("minecraft:stone")]),
    };
    let add = ClientboundRecipeBookAddPacket {
        entries: vec![RecipeBookAddEntry {
            contents: display_entry,
            flags: RecipeBookEntryFlags::new(true, false),
        }],
        replace: true,
    };
    let remove = ClientboundRecipeBookRemovePacket {
        recipes: vec![RecipeDisplayId(3)],
    };
    let update = ClientboundUpdateRecipesPacket {
        item_sets: vec![RecipePropertySet {
            key: "minecraft:stonecutting",
            accepted_items: vec!["minecraft:stone"],
        }],
        stonecutter_recipes: vec![SelectableSingleInputRecipe {
            input: "minecraft:stone",
            recipe: Some("minecraft:stone_slab_from_stone_stonecutting"),
        }],
    };
    let settings = ClientboundRecipeBookSettingsPacket {
        settings: RecipeBookSettings::default(),
    };

    assert!(add.replace);
    assert_eq!(
        add.entries[0].flags.bits(),
        RecipeBookEntryFlags::NOTIFICATION
    );
    assert_eq!(remove.recipes, vec![RecipeDisplayId(3)]);
    assert_eq!(update.item_sets[0].accepted_items, vec!["minecraft:stone"]);
    assert_eq!(
        update.stonecutter_recipes[0].recipe,
        Some("minecraft:stone_slab_from_stone_stonecutting")
    );
    assert_eq!(
        settings.settings.stream_order(),
        [RecipeBookTypeSettings::default(); 4]
    );
}

#[test]
fn recipe_display_model_covers_all_vanilla_display_and_slot_variants() {
    let mut settings = RecipeBookSettings::default();
    settings.set_open(RecipeBookType::Furnace, true);
    settings.set_filtering(RecipeBookType::Smoker, true);

    let displays = [
        RecipeDisplay::ShapedCrafting {
            width: 2,
            height: 2,
            ingredients: vec![SlotDisplay::Item("minecraft:planks")],
            result: SlotDisplay::ItemStack {
                item: "minecraft:crafting_table",
                count: 1,
            },
            crafting_station: SlotDisplay::Empty,
        },
        RecipeDisplay::Furnace {
            ingredient: SlotDisplay::Tag("minecraft:logs"),
            fuel: SlotDisplay::AnyFuel,
            result: SlotDisplay::Item("minecraft:charcoal"),
            crafting_station: SlotDisplay::Item("minecraft:furnace"),
            duration_ticks: 200,
            experience: 1,
        },
        RecipeDisplay::Smithing {
            template: SlotDisplay::SmithingTrim {
                base: Box::new(SlotDisplay::Item(
                    "minecraft:netherite_upgrade_smithing_template",
                )),
            },
            base: SlotDisplay::OnlyWithComponent {
                component: "minecraft:damage",
            },
            addition: SlotDisplay::WithAnyPotion,
            result: SlotDisplay::Dyed {
                base: Box::new(SlotDisplay::Item("minecraft:leather_chestplate")),
                color: 0x33_66_99,
            },
            crafting_station: SlotDisplay::WithRemainder {
                input: Box::new(SlotDisplay::Item("minecraft:water_bucket")),
                remainder: Box::new(SlotDisplay::Item("minecraft:bucket")),
            },
        },
    ];

    assert_eq!(displays.len(), 3);
    assert!(settings.get(RecipeBookType::Furnace).open);
    assert!(settings.get(RecipeBookType::Smoker).filtering);
}

#[test]
fn shaped_and_shapeless_recipes_match_vanilla_grid_rules() {
    let shaped = RecipeKind::Shaped {
        width: 2,
        height: 2,
        pattern: vec![
            Some(IngredientSpec::Item("minecraft:oak_planks")),
            Some(IngredientSpec::Item("minecraft:oak_planks")),
            Some(IngredientSpec::Item("minecraft:oak_planks")),
            Some(IngredientSpec::Item("minecraft:oak_planks")),
        ],
        result: ItemAmount::one("minecraft:crafting_table"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    let grid = vec![
        None,
        Some("minecraft:oak_planks"),
        Some("minecraft:oak_planks"),
        None,
        Some("minecraft:oak_planks"),
        Some("minecraft:oak_planks"),
        None,
        None,
        None,
    ];

    assert!(shaped.matches(3, 3, &grid));
    assert_eq!(shaped.serializer(), "crafting_shaped");
    assert_eq!(
        shaped.assemble(),
        Some(ItemAmount::one("minecraft:crafting_table"))
    );

    let asymmetric = RecipeKind::Shaped {
        width: 2,
        height: 1,
        pattern: vec![
            Some(IngredientSpec::Item("minecraft:stick")),
            Some(IngredientSpec::Item("minecraft:coal")),
        ],
        result: ItemAmount::one("minecraft:torch"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    assert!(asymmetric.matches(2, 1, &[Some("minecraft:stick"), Some("minecraft:coal")]));
    // 26.1.2 `ShapedRecipePattern.matches` tries the horizontally-mirrored pattern
    // for a non-symmetrical recipe, so the reversed layout also matches.
    assert!(asymmetric.matches(2, 1, &[Some("minecraft:coal"), Some("minecraft:stick")]));
    // A different item in a pattern slot still fails (mirroring is not a wildcard).
    assert!(!asymmetric.matches(2, 1, &[Some("minecraft:stick"), Some("minecraft:stick")]));

    let shapeless = RecipeKind::Shapeless {
        ingredients: vec![
            IngredientSpec::Item("minecraft:gunpowder"),
            IngredientSpec::AnyOf(vec!["minecraft:red_dye", "minecraft:blue_dye"]),
        ],
        result: ItemAmount::one("minecraft:firework_star"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    assert!(shapeless.matches(
        2,
        2,
        &[
            Some("minecraft:blue_dye"),
            None,
            Some("minecraft:gunpowder"),
            None
        ]
    ));
    assert!(!shapeless.matches(
        2,
        2,
        &[
            Some("minecraft:blue_dye"),
            Some("minecraft:gunpowder"),
            Some("minecraft:paper"),
            None
        ]
    ));

    // Overlapping ingredients: a broad ingredient (any plank) + a narrow one (oak)
    // against an oak + birch grid. A greedy first-match would assign the broad
    // ingredient to oak and then fail the oak-only ingredient; backtracking (like
    // `StackedContents.canCraft`) finds broad→birch, oak→oak.
    let overlapping = RecipeKind::Shapeless {
        ingredients: vec![
            IngredientSpec::AnyOf(vec!["minecraft:oak_planks", "minecraft:birch_planks"]),
            IngredientSpec::Item("minecraft:oak_planks"),
        ],
        result: ItemAmount::one("minecraft:stick"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    assert!(overlapping.matches(
        2,
        1,
        &[Some("minecraft:oak_planks"), Some("minecraft:birch_planks")]
    ));
    // Two birch cannot satisfy the oak-only ingredient.
    assert!(!overlapping.matches(
        2,
        1,
        &[
            Some("minecraft:birch_planks"),
            Some("minecraft:birch_planks")
        ]
    ));
}

fn assert_transmute_recipe_matches_java_crafting_rules() {
    let transmute = RecipeKind::Transmute {
        input: IngredientSpec::Item("minecraft:filled_map"),
        material: IngredientSpec::Item("minecraft:map"),
        min_material_count: 1,
        max_material_count: 8,
        result: ItemAmount::one("minecraft:filled_map"),
        add_material_count_to_result: true,
    };

    assert_eq!(transmute.serializer(), "crafting_transmute");
    assert!(transmute.matches(
        3,
        3,
        &[
            Some("minecraft:filled_map"),
            Some("minecraft:map"),
            Some("minecraft:map"),
            None,
            None,
            None,
            None,
            None,
            None,
        ]
    ));
    assert!(!transmute.matches(
        3,
        3,
        &[
            Some("minecraft:filled_map"),
            Some("minecraft:filled_map"),
            Some("minecraft:map"),
            None,
            None,
            None,
            None,
            None,
            None,
        ]
    ));

    let source = ComponentCraftingStack {
        item: "minecraft:filled_map",
        count: 1,
        potion_contents: None,
        custom_name: Some("Base map"),
        ..ComponentCraftingStack::one("minecraft:filled_map")
    };
    let copied = transmute_result(&ItemAmount::one("minecraft:filled_map"), &source, 2, true);
    assert_eq!(copied.item, "minecraft:filled_map");
    assert_eq!(copied.count, 3);
    assert_eq!(copied.custom_name, Some("Base map"));
}

fn assert_imbue_recipe_matches_java_crafting_rules() {
    let imbue = RecipeKind::Imbue {
        source: IngredientSpec::Item("minecraft:lingering_potion"),
        material: IngredientSpec::Item("minecraft:arrow"),
        result: ItemAmount {
            item: "minecraft:tipped_arrow",
            count: 8,
        },
    };
    assert_eq!(imbue.serializer(), "crafting_imbue");
    assert!(imbue.matches(
        3,
        3,
        &[
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:lingering_potion"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
        ]
    ));
    assert!(!imbue.matches(
        3,
        3,
        &[
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:potion"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
            Some("minecraft:arrow"),
        ]
    ));

    let potion = ComponentCraftingStack {
        item: "minecraft:lingering_potion",
        count: 1,
        potion_contents: Some("minecraft:strong_harming"),
        custom_name: Some("Splashy"),
        ..ComponentCraftingStack::one("minecraft:lingering_potion")
    };
    let arrows = imbue_result(
        &ItemAmount {
            item: "minecraft:tipped_arrow",
            count: 8,
        },
        &potion,
    );
    assert_eq!(arrows.item, "minecraft:tipped_arrow");
    assert_eq!(arrows.count, 8);
    assert_eq!(arrows.potion_contents, Some("minecraft:strong_harming"));
    assert_eq!(arrows.custom_name, None);
}

#[test]
fn transmute_and_imbue_recipes_match_java_crafting_rules() {
    assert_transmute_recipe_matches_java_crafting_rules();
    assert_imbue_recipe_matches_java_crafting_rules();
}

fn assert_banner_book_and_pot_special_recipes_produce_vanilla_components() {
    let mut patterned_banner = ComponentCraftingStack::one("minecraft:white_banner");
    patterned_banner.banner_color = Some("white");
    patterned_banner.banner_patterns = 2;
    let mut blank_banner = ComponentCraftingStack::one("minecraft:white_banner");
    blank_banner.banner_color = Some("white");
    let (banner_copy, banner_remainders) = banner_duplicate_result(
        "minecraft:white_banner",
        &[Some(patterned_banner.clone()), Some(blank_banner)],
    )
    .expect("one patterned and one blank banner of the same color should copy");
    assert_eq!(banner_copy.banner_patterns, 2);
    assert_eq!(
        banner_remainders[0].as_ref().unwrap().item,
        "minecraft:white_banner"
    );
    assert!(banner_duplicate_result(
        "minecraft:white_banner",
        &[
            Some(patterned_banner.clone()),
            Some(patterned_banner.clone())
        ]
    )
    .is_none());

    let mut written = ComponentCraftingStack::one("minecraft:written_book");
    written.written_book_generation = Some(0);
    let (books, book_remainders) = book_cloning_result(
        "minecraft:written_book",
        &[
            Some(written.clone()),
            Some(ComponentCraftingStack::one("minecraft:writable_book")),
            Some(ComponentCraftingStack::one("minecraft:writable_book")),
        ],
        0,
        1,
    )
    .expect("generation 0 written book plus blanks should clone");
    assert_eq!(books.count, 2);
    assert_eq!(books.written_book_generation, Some(1));
    assert_eq!(
        book_remainders[0].as_ref().unwrap().item,
        "minecraft:written_book"
    );
    written.written_book_generation = Some(2);
    assert!(book_cloning_result(
        "minecraft:written_book",
        &[
            Some(written),
            Some(ComponentCraftingStack::one("minecraft:writable_book"))
        ],
        0,
        1
    )
    .is_none());

    let pot = decorated_pot_result(&[
        None,
        Some(ComponentCraftingStack::one(
            "minecraft:angler_pottery_sherd",
        )),
        None,
        Some(ComponentCraftingStack::one("minecraft:brick")),
        None,
        Some(ComponentCraftingStack::one(
            "minecraft:archer_pottery_sherd",
        )),
        None,
        Some(ComponentCraftingStack::one(
            "minecraft:arms_up_pottery_sherd",
        )),
        None,
    ])
    .expect("decorated pot uses back, left, right, front slots");
    assert_eq!(
        pot.pot_decorations,
        Some(PotDecorationsModel {
            back: "minecraft:angler_pottery_sherd",
            left: "minecraft:brick",
            right: "minecraft:archer_pottery_sherd",
            front: "minecraft:arms_up_pottery_sherd",
        })
    );
}

fn assert_dye_firework_and_map_special_recipes_produce_vanilla_components() {
    let mut leather = ComponentCraftingStack::one("minecraft:leather_helmet");
    leather.dyed_color = Some(0x0000FF);
    let mut red_dye = ComponentCraftingStack::one("minecraft:red_dye");
    red_dye.dye_color = Some(0xFF0000);
    let dyed = dye_result("minecraft:leather_helmet", &[Some(leather), Some(red_dye)])
        .expect("leather target plus dye should produce dyed target");
    assert_eq!(dyed.dyed_color, Some(0x7F007F));

    let mut star = ComponentCraftingStack::one("minecraft:firework_star");
    star.firework_explosion = Some(FireworkExplosionModel {
        shape: "star",
        colors: vec![0xFF0000],
        fade_colors: Vec::new(),
        trail: true,
        twinkle: false,
    });
    let rocket = firework_rocket_result(
        "minecraft:firework_rocket",
        3,
        &[
            Some(ComponentCraftingStack::one("minecraft:paper")),
            Some(ComponentCraftingStack::one("minecraft:gunpowder")),
            Some(ComponentCraftingStack::one("minecraft:gunpowder")),
            Some(star.clone()),
        ],
    )
    .expect("paper plus one to three gunpowder and optional stars should craft rockets");
    assert_eq!(rocket.count, 3);
    assert_eq!(rocket.fireworks.as_ref().unwrap().flight_duration, 2);
    assert_eq!(
        rocket.fireworks.as_ref().unwrap().explosions,
        vec![star.firework_explosion.clone().unwrap()]
    );

    let mut blue_dye = ComponentCraftingStack::one("minecraft:blue_dye");
    blue_dye.dye_color = Some(0x0000FF);
    let crafted_star = firework_star_result(&[
        Some(ComponentCraftingStack::one("minecraft:gunpowder")),
        Some(blue_dye.clone()),
        Some(ComponentCraftingStack::one("minecraft:gold_nugget")),
        Some(ComponentCraftingStack::one("minecraft:diamond")),
        Some(ComponentCraftingStack::one("minecraft:glowstone_dust")),
    ])
    .expect("gunpowder plus dye with optional shape/trail/twinkle should craft a star");
    let explosion = crafted_star.firework_explosion.as_ref().unwrap();
    assert_eq!(explosion.shape, "star");
    assert_eq!(explosion.colors, vec![0x0000FF]);
    assert!(explosion.trail);
    assert!(explosion.twinkle);

    let faded = firework_star_fade_result(&[Some(crafted_star), Some(blue_dye)])
        .expect("firework star plus dyes should set fade colors");
    assert_eq!(
        faded.firework_explosion.unwrap().fade_colors,
        vec![0x0000FF]
    );

    let mut map = ComponentCraftingStack::one("minecraft:filled_map");
    map.map_scale = Some(3);
    let mut map_grid = vec![Some(ComponentCraftingStack::one("minecraft:paper")); 9];
    map_grid[4] = Some(map);
    let extended =
        map_extending_result(&map_grid).expect("scale < 4 non-exploration map should extend");
    assert!(extended.map_post_processing_scale);
}

fn assert_repair_and_shield_special_recipes_produce_vanilla_components() {
    let mut first_pick = ComponentCraftingStack::one("minecraft:diamond_pickaxe");
    first_pick.max_damage = Some(100);
    first_pick.damage = Some(80);
    first_pick.curses.push(EnchantmentComponent {
        id: "minecraft:binding_curse",
        level: 1,
    });
    let mut second_pick = ComponentCraftingStack::one("minecraft:diamond_pickaxe");
    second_pick.max_damage = Some(100);
    second_pick.damage = Some(90);
    second_pick.curses.push(EnchantmentComponent {
        id: "minecraft:vanishing_curse",
        level: 1,
    });
    let repaired = repair_item_result(&[Some(first_pick), Some(second_pick)])
        .expect("two same damaged single-count items should repair");
    assert_eq!(repaired.damage, Some(65));
    assert_eq!(repaired.curses.len(), 2);

    let mut shield_banner = ComponentCraftingStack::one("minecraft:red_banner");
    shield_banner.banner_color = Some("red");
    shield_banner.banner_patterns = 3;
    let decorated_shield = shield_decoration_result(&[
        Some(shield_banner),
        Some(ComponentCraftingStack::one("minecraft:shield")),
    ])
    .expect("pattern banner plus clear shield should decorate shield");
    assert_eq!(decorated_shield.banner_patterns, 3);
    assert_eq!(decorated_shield.base_color, Some("red"));
}

#[test]
fn special_crafting_recipes_produce_vanilla_components() {
    assert_banner_book_and_pot_special_recipes_produce_vanilla_components();
    assert_dye_firework_and_map_special_recipes_produce_vanilla_components();
    assert_repair_and_shield_special_recipes_produce_vanilla_components();
}

#[test]
fn cooking_stonecutting_and_smithing_recipes_match_single_input_contracts() {
    for (kind, expected_time) in [
        (CookingKind::Smelting, 200),
        (CookingKind::Blasting, 100),
        (CookingKind::Smoking, 100),
        (CookingKind::CampfireCooking, 100),
    ] {
        let recipe = RecipeKind::Cooking {
            kind,
            ingredient: IngredientSpec::Item("minecraft:raw_iron"),
            result: ItemAmount::one("minecraft:iron_ingot"),
            experience_millis: 700,
            cooking_time: None,
            category: crate::recipe_system::CookingBookCategory::Misc,
        };
        assert!(recipe.matches(1, 1, &[Some("minecraft:raw_iron")]));
        assert_eq!(recipe.cooking_time(), Some(expected_time));
        assert_eq!(recipe.cooking_experience_millis(), Some(700));
        assert_eq!(
            recipe.single_item_input(),
            Some(&IngredientSpec::Item("minecraft:raw_iron"))
        );
        assert_eq!(
            recipe.single_item_result(),
            Some(&ItemAmount::one("minecraft:iron_ingot"))
        );
        assert_eq!(recipe.serializer(), kind.serializer());
    }

    let stonecutting = RecipeKind::Stonecutting {
        ingredient: IngredientSpec::Item("minecraft:stone"),
        result: ItemAmount {
            item: "minecraft:stone_slab",
            count: 2,
        },
    };
    assert!(stonecutting.matches(1, 1, &[Some("minecraft:stone")]));
    assert!(!stonecutting.matches(1, 2, &[Some("minecraft:stone"), Some("minecraft:stone")]));
    assert_eq!(
        stonecutting.single_item_input(),
        Some(&IngredientSpec::Item("minecraft:stone"))
    );
    assert_eq!(
        stonecutting.single_item_result(),
        Some(&ItemAmount {
            item: "minecraft:stone_slab",
            count: 2,
        })
    );

    let transform = RecipeKind::SmithingTransform {
        template: IngredientSpec::Item("minecraft:netherite_upgrade_smithing_template"),
        base: IngredientSpec::Item("minecraft:diamond_sword"),
        addition: IngredientSpec::Item("minecraft:netherite_ingot"),
        result: ItemAmount::one("minecraft:netherite_sword"),
    };
    let trim = RecipeKind::SmithingTrim {
        template: IngredientSpec::Item("minecraft:spire_armor_trim_smithing_template"),
        base: IngredientSpec::Item("minecraft:iron_chestplate"),
        addition: IngredientSpec::Item("minecraft:amethyst_shard"),
        pattern: "minecraft:spire",
    };

    assert!(transform.matches(
        3,
        1,
        &[
            Some("minecraft:netherite_upgrade_smithing_template"),
            Some("minecraft:diamond_sword"),
            Some("minecraft:netherite_ingot")
        ]
    ));
    assert_eq!(transform.serializer(), "smithing_transform");
    assert_eq!(transform.recipe_type(), "smithing");
    assert_eq!(transform.recipe_book_category(), "smithing");
    assert_eq!(transform.smithing_is_incomplete(), Some(false));
    assert!(trim.matches(
        3,
        1,
        &[
            Some("minecraft:spire_armor_trim_smithing_template"),
            Some("minecraft:iron_chestplate"),
            Some("minecraft:amethyst_shard")
        ]
    ));
    assert_eq!(trim.serializer(), "smithing_trim");
    assert_eq!(trim.recipe_type(), "smithing");
    assert_eq!(trim.recipe_book_category(), "smithing");
    assert_eq!(trim.smithing_is_incomplete(), Some(false));
    assert_eq!(trim.assemble(), None);

    let incomplete_transform = RecipeKind::SmithingTransform {
        template: IngredientSpec::Empty,
        base: IngredientSpec::Item("minecraft:diamond_sword"),
        addition: IngredientSpec::Item("minecraft:netherite_ingot"),
        result: ItemAmount::one("minecraft:netherite_sword"),
    };
    assert_eq!(
        incomplete_transform
            .smithing_placement_info()
            .unwrap()
            .slots_to_ingredient_index,
        vec![PlacementInfo::EMPTY_SLOT, 0, 1]
    );
}

#[test]
fn fuel_values_match_vanilla_burn_time_defaults() {
    let fuels = FuelValues::vanilla();
    assert_eq!(fuels.burn_duration(Some("minecraft:lava_bucket")), 20_000);
    assert_eq!(fuels.burn_duration(Some("minecraft:coal_block")), 16_000);
    assert_eq!(fuels.burn_duration(Some("minecraft:blaze_rod")), 2_400);
    assert_eq!(fuels.burn_duration(Some("minecraft:coal")), 1_600);
    assert_eq!(fuels.burn_duration(Some("minecraft:charcoal")), 1_600);
    assert_eq!(fuels.burn_duration(Some("minecraft:oak_log")), 300);
    assert_eq!(fuels.burn_duration(Some("minecraft:oak_planks")), 300);
    assert_eq!(fuels.burn_duration(Some("minecraft:oak_slab")), 150);
    assert_eq!(fuels.burn_duration(Some("minecraft:oak_hanging_sign")), 800);
    assert_eq!(fuels.burn_duration(Some("minecraft:oak_boat")), 1_200);
    assert_eq!(fuels.burn_duration(Some("minecraft:white_wool")), 100);
    assert_eq!(fuels.burn_duration(Some("minecraft:white_carpet")), 67);
    assert_eq!(
        fuels.burn_duration(Some("minecraft:dried_kelp_block")),
        4_001
    );
    assert_eq!(fuels.burn_duration(Some("minecraft:bamboo")), 50);
    assert_eq!(fuels.burn_duration(None), 0);
    assert_eq!(fuels.burn_duration(Some("minecraft:diamond")), 0);
    assert!(fuels.is_fuel("minecraft:stick"));
    assert!(!fuels.is_fuel("minecraft:bucket"));
    let fuel_items = fuels.fuel_items().collect::<Vec<_>>();
    assert!(fuel_items.contains(&"minecraft:lava_bucket"));
    assert!(fuel_items.contains(&"minecraft:stick"));
    assert!(!fuel_items.contains(&"minecraft:bucket"));

    let faster = FuelValues::vanilla_with_base_unit(100);
    assert_eq!(faster.burn_duration(Some("minecraft:coal")), 800);
    assert_eq!(faster.burn_duration(Some("minecraft:white_carpet")), 34);
}

#[test]
fn cooking_recipe_experience_and_fuel_interaction_follow_furnace_rules() {
    let recipe = RecipeKind::Cooking {
        kind: CookingKind::Smelting,
        ingredient: IngredientSpec::Item("minecraft:raw_iron"),
        result: ItemAmount::one("minecraft:iron_ingot"),
        experience_millis: 700,
        cooking_time: None,
        category: crate::recipe_system::CookingBookCategory::Misc,
    };
    let fuels = FuelValues::vanilla();

    assert_eq!(recipe.cooking_time(), Some(200));
    assert!(recipe.matches(1, 1, &[Some("minecraft:raw_iron")]));
    assert_eq!(fuels.burn_duration(Some("minecraft:coal")), 1_600);
    assert_eq!(fuels.burn_duration(Some("minecraft:stick")), 100);

    let usage = FurnaceRecipeUsage {
        recipe_id: "minecraft:iron_ingot_from_smelting_raw_iron",
        times_used: 3,
        experience_millis: 700,
    };
    assert_eq!(furnace_experience_to_award(&usage, 0.05), 3);
    assert_eq!(furnace_experience_to_award(&usage, 0.95), 2);
    assert_eq!(
        furnace_experience_to_award(
            &FurnaceRecipeUsage {
                times_used: 0,
                ..usage
            },
            0.0
        ),
        0
    );
}

#[cfg(vibecraft_has_decompiled_sources)]
mod recipe_kind_tests;

#[test]
fn cooking_book_category_drives_recipe_book_group() {
    use crate::recipe_system::CookingBookCategory;

    assert_eq!(
        CookingBookCategory::from_id(Some("food")),
        CookingBookCategory::Food
    );
    assert_eq!(
        CookingBookCategory::from_id(Some("blocks")),
        CookingBookCategory::Blocks
    );
    assert_eq!(
        CookingBookCategory::from_id(Some("misc")),
        CookingBookCategory::Misc
    );
    assert_eq!(
        CookingBookCategory::from_id(None),
        CookingBookCategory::Misc
    );

    let cooking = |kind, category| RecipeKind::Cooking {
        kind,
        ingredient: IngredientSpec::Item("minecraft:raw_iron"),
        result: ItemAmount::one("minecraft:iron_ingot"),
        experience_millis: 700,
        cooking_time: None,
        category,
    };

    // SmeltingRecipe.recipeBookCategory: FOOD/BLOCKS/MISC -> the matching group.
    assert_eq!(
        cooking(CookingKind::Smelting, CookingBookCategory::Food).recipe_book_category(),
        "furnace_food"
    );
    assert_eq!(
        cooking(CookingKind::Smelting, CookingBookCategory::Blocks).recipe_book_category(),
        "furnace_blocks"
    );
    assert_eq!(
        cooking(CookingKind::Smelting, CookingBookCategory::Misc).recipe_book_category(),
        "furnace_misc"
    );
    // BlastingRecipe: BLOCKS -> blocks, FOOD/MISC -> misc.
    assert_eq!(
        cooking(CookingKind::Blasting, CookingBookCategory::Blocks).recipe_book_category(),
        "blast_furnace_blocks"
    );
    assert_eq!(
        cooking(CookingKind::Blasting, CookingBookCategory::Food).recipe_book_category(),
        "blast_furnace_misc"
    );
    // Smoker is always food; campfire is always campfire.
    assert_eq!(
        cooking(CookingKind::Smoking, CookingBookCategory::Misc).recipe_book_category(),
        "smoker_food"
    );
    assert_eq!(
        cooking(CookingKind::CampfireCooking, CookingBookCategory::Misc).recipe_book_category(),
        "campfire"
    );
}

#[test]
fn recipe_is_incomplete_when_a_required_ingredient_is_empty() {
    // A complete shapeless recipe.
    let complete = RecipeKind::Shapeless {
        ingredients: vec![IngredientSpec::Item("minecraft:stick")],
        result: ItemAmount::one("minecraft:torch"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    assert!(!complete.is_incomplete());

    // An empty ingredient makes it incomplete.
    let incomplete = RecipeKind::Shapeless {
        ingredients: vec![
            IngredientSpec::Item("minecraft:stick"),
            IngredientSpec::Empty,
        ],
        result: ItemAmount::one("minecraft:torch"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    assert!(incomplete.is_incomplete());

    // Smithing: only the base is required (template/addition optional).
    let smithing_ok = RecipeKind::SmithingTrim {
        template: IngredientSpec::Empty,
        base: IngredientSpec::Item("minecraft:diamond_chestplate"),
        addition: IngredientSpec::Empty,
        pattern: "minecraft:sentry",
    };
    assert!(
        !smithing_ok.is_incomplete(),
        "empty template/addition are allowed"
    );
    let smithing_bad = RecipeKind::SmithingTrim {
        template: IngredientSpec::Item("minecraft:sentry_armor_trim_smithing_template"),
        base: IngredientSpec::Empty,
        addition: IngredientSpec::Item("minecraft:copper_ingot"),
        pattern: "minecraft:sentry",
    };
    assert!(smithing_bad.is_incomplete(), "empty base is incomplete");

    // Special recipes are never incomplete.
    assert!(!RecipeKind::Special {
        kind: crate::recipe_system::SpecialRecipeKind::RepairItem,
        result_hint: None,
    }
    .is_incomplete());
}

#[test]
fn placement_info_is_built_per_recipe_type_like_java() {
    // Shapeless: create(list) -> one slot per ingredient, in order.
    let shapeless = RecipeKind::Shapeless {
        ingredients: vec![
            IngredientSpec::Item("minecraft:stick"),
            IngredientSpec::Item("minecraft:coal"),
        ],
        result: ItemAmount::one("minecraft:torch"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    let p = shapeless.placement_info();
    assert_eq!(p.ingredients.len(), 2);
    assert_eq!(p.slots_to_ingredient_index, vec![0, 1]);
    assert!(!p.is_impossible_to_place());

    // Cooking: single ingredient.
    let cooking = RecipeKind::Cooking {
        kind: CookingKind::Smelting,
        ingredient: IngredientSpec::Item("minecraft:raw_iron"),
        result: ItemAmount::one("minecraft:iron_ingot"),
        experience_millis: 700,
        cooking_time: None,
        category: crate::recipe_system::CookingBookCategory::Misc,
    };
    assert_eq!(cooking.placement_info().slots_to_ingredient_index, vec![0]);

    // Imbue: 3x3 ring of material around a centre source (= ImbueRecipe.createPlacementInfo).
    let imbue = RecipeKind::Imbue {
        source: IngredientSpec::Item("minecraft:lingering_potion"),
        material: IngredientSpec::Item("minecraft:arrow"),
        result: ItemAmount {
            item: "minecraft:tipped_arrow",
            count: 8,
        },
    };
    assert_eq!(imbue.placement_info().slots_to_ingredient_index.len(), 9);

    // Special (CustomRecipe) -> NOT_PLACEABLE.
    let special = RecipeKind::Special {
        kind: crate::recipe_system::SpecialRecipeKind::RepairItem,
        result_hint: None,
    };
    assert!(special.placement_info().is_impossible_to_place());

    // A shapeless recipe with an empty ingredient is impossible to place.
    let broken = RecipeKind::Shapeless {
        ingredients: vec![IngredientSpec::Empty],
        result: ItemAmount::one("minecraft:torch"),
        category: crate::recipe_system::CraftingBookCategoryModel::Misc,
    };
    assert!(broken.placement_info().is_impossible_to_place());
}

#[test]
fn crafting_book_category_drives_shaped_shapeless_recipe_book_group() {
    use crate::recipe_system::CraftingBookCategoryModel as C;
    assert_eq!(C::from_id(Some("building")), C::Building);
    assert_eq!(C::from_id(Some("redstone")), C::Redstone);
    assert_eq!(C::from_id(Some("equipment")), C::Equipment);
    assert_eq!(C::from_id(None), C::Misc);

    let shaped = |category| RecipeKind::Shaped {
        width: 1,
        height: 1,
        pattern: vec![Some(IngredientSpec::Item("minecraft:redstone"))],
        result: ItemAmount::one("minecraft:redstone_block"),
        category,
    };
    assert_eq!(
        shaped(C::Building).recipe_book_category(),
        "crafting_building_blocks"
    );
    assert_eq!(
        shaped(C::Redstone).recipe_book_category(),
        "crafting_redstone"
    );
    assert_eq!(
        shaped(C::Equipment).recipe_book_category(),
        "crafting_equipment"
    );
    assert_eq!(shaped(C::Misc).recipe_book_category(), "crafting_misc");

    let shapeless = RecipeKind::Shapeless {
        ingredients: vec![IngredientSpec::Item("minecraft:stick")],
        result: ItemAmount::one("minecraft:torch"),
        category: C::Building,
    };
    assert_eq!(shapeless.recipe_book_category(), "crafting_building_blocks");
}
