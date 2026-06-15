use super::super::*;
use super::*;

type RecipeKindCoverageCase = (
    &'static str,
    RecipeKind,
    usize,
    usize,
    Vec<Option<&'static str>>,
    bool,
    Option<ItemAmount>,
);

#[test]
fn every_recipe_kind_has_matches_assemble_remaining_and_unlock_coverage() {
    let mut cases = crafting_recipe_coverage_cases();
    cases.extend(cooking_recipe_coverage_cases());
    cases.extend(utility_recipe_coverage_cases());
    cases.extend(smithing_and_special_recipe_coverage_cases());

    let mut unlocks = PlayerRecipeUnlocks::default();
    let mut advancements = PlayerAdvancementSet::default();
    let advancement = crate::advancement_system::AdvancementDefinition::all_of(
        "minecraft:recipes/root",
        None,
        &["has_the_recipe"],
        crate::advancement_system::AdvancementRewards::default(),
        None,
    )
    .unwrap();

    for (
        index,
        (id, recipe, grid_width, grid_height, valid_input, should_match, expected_result),
    ) in cases.into_iter().enumerate()
    {
        assert_eq!(
            recipe.matches(grid_width, grid_height, &valid_input),
            should_match,
            "{id} valid-input match result"
        );
        let mut invalid_input = valid_input.clone();
        if let Some(first) = invalid_input.first_mut() {
            *first = Some("minecraft:bedrock");
        }
        assert!(
            !recipe.matches(grid_width, grid_height, &invalid_input),
            "{id} invalid-input should not match"
        );
        assert_eq!(recipe.assemble(), expected_result, "{id} assemble result");

        let remaining_input = valid_input
            .iter()
            .map(|item| item.map(CraftingStack::one))
            .collect::<Vec<_>>();
        let remaining = recipe.get_remaining_items(&remaining_input);
        assert_eq!(remaining.len(), remaining_input.len(), "{id} remaining len");
        if recipe.recipe_type() == "crafting" {
            let remainder_recipe = RecipeKind::Shapeless {
                ingredients: vec![IngredientSpec::Item("minecraft:water_bucket")],
                result: ItemAmount::one("minecraft:packed_ice"),
                category: crate::recipe_system::CraftingBookCategoryModel::Misc,
            };
            assert_eq!(
                remainder_recipe
                    .get_remaining_items(&[Some(CraftingStack::one("minecraft:water_bucket"))]),
                vec![Some(CraftingStack::one("minecraft:bucket"))]
            );
        } else {
            assert!(remaining.iter().all(Option::is_none), "{id} no remainders");
        }

        let recipe_definition = AdvancementRecipeDefinition {
            id: crate::registry::Identifier::parse(id).unwrap(),
            special: recipe.is_special(),
            show_notification: recipe.show_notification(),
        };
        let triggers = vec![RecipeUnlockedCriterion {
            advancement: advancement.id.clone(),
            criterion: "has_the_recipe".to_string(),
            recipe: recipe_definition.id.clone(),
        }];
        let events = unlocks.unlock_recipes(
            std::slice::from_ref(&recipe_definition),
            std::slice::from_ref(&advancement),
            &triggers,
            &mut advancements,
            1_700_000_000 + index as u64,
        );
        if recipe.is_special() {
            assert!(events.is_empty(), "{id} special recipes do not unlock");
            assert!(!unlocks.contains(&recipe_definition.id));
        } else {
            assert_eq!(events.len(), 1, "{id} unlock event");
            assert!(unlocks.contains(&recipe_definition.id));
            assert_eq!(events[0].recipe, recipe_definition.id);
            assert_eq!(events[0].show_notification, recipe.show_notification());
        }
    }
}

fn crafting_recipe_coverage_cases() -> Vec<RecipeKindCoverageCase> {
    vec![
        (
            "minecraft:crafting_table",
            RecipeKind::Shaped {
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
            },
            2,
            2,
            vec![
                Some("minecraft:oak_planks"),
                Some("minecraft:oak_planks"),
                Some("minecraft:oak_planks"),
                Some("minecraft:oak_planks"),
            ],
            true,
            Some(ItemAmount::one("minecraft:crafting_table")),
        ),
        (
            "minecraft:firework_star",
            RecipeKind::Shapeless {
                ingredients: vec![
                    IngredientSpec::Item("minecraft:gunpowder"),
                    IngredientSpec::AnyOf(vec!["minecraft:red_dye", "minecraft:blue_dye"]),
                ],
                result: ItemAmount::one("minecraft:firework_star"),
                category: crate::recipe_system::CraftingBookCategoryModel::Misc,
            },
            2,
            1,
            vec![Some("minecraft:blue_dye"), Some("minecraft:gunpowder")],
            true,
            Some(ItemAmount::one("minecraft:firework_star")),
        ),
    ]
}

fn cooking_recipe_coverage_cases() -> Vec<RecipeKindCoverageCase> {
    vec![
        (
            "minecraft:iron_ingot_from_smelting_raw_iron",
            RecipeKind::Cooking {
                kind: CookingKind::Smelting,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: None,
                category: crate::recipe_system::CookingBookCategory::Misc,
            },
            1,
            1,
            vec![Some("minecraft:raw_iron")],
            true,
            Some(ItemAmount::one("minecraft:iron_ingot")),
        ),
        (
            "minecraft:iron_ingot_from_blasting_raw_iron",
            RecipeKind::Cooking {
                kind: CookingKind::Blasting,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: None,
                category: crate::recipe_system::CookingBookCategory::Misc,
            },
            1,
            1,
            vec![Some("minecraft:raw_iron")],
            true,
            Some(ItemAmount::one("minecraft:iron_ingot")),
        ),
        (
            "minecraft:cooked_beef_from_smoking",
            RecipeKind::Cooking {
                kind: CookingKind::Smoking,
                ingredient: IngredientSpec::Item("minecraft:beef"),
                result: ItemAmount::one("minecraft:cooked_beef"),
                experience_millis: 350,
                cooking_time: None,
                category: crate::recipe_system::CookingBookCategory::Misc,
            },
            1,
            1,
            vec![Some("minecraft:beef")],
            true,
            Some(ItemAmount::one("minecraft:cooked_beef")),
        ),
        (
            "minecraft:cooked_cod_from_campfire_cooking",
            RecipeKind::Cooking {
                kind: CookingKind::CampfireCooking,
                ingredient: IngredientSpec::Item("minecraft:cod"),
                result: ItemAmount::one("minecraft:cooked_cod"),
                experience_millis: 350,
                cooking_time: None,
                category: crate::recipe_system::CookingBookCategory::Misc,
            },
            1,
            1,
            vec![Some("minecraft:cod")],
            true,
            Some(ItemAmount::one("minecraft:cooked_cod")),
        ),
    ]
}

fn utility_recipe_coverage_cases() -> Vec<RecipeKindCoverageCase> {
    vec![
        (
            "minecraft:stone_slab_from_stone_stonecutting",
            RecipeKind::Stonecutting {
                ingredient: IngredientSpec::Item("minecraft:stone"),
                result: ItemAmount {
                    item: "minecraft:stone_slab",
                    count: 2,
                },
            },
            1,
            1,
            vec![Some("minecraft:stone")],
            true,
            Some(ItemAmount {
                item: "minecraft:stone_slab",
                count: 2,
            }),
        ),
        (
            "minecraft:filled_map_copy",
            RecipeKind::Transmute {
                input: IngredientSpec::Item("minecraft:filled_map"),
                material: IngredientSpec::Item("minecraft:map"),
                min_material_count: 1,
                max_material_count: 8,
                result: ItemAmount::one("minecraft:filled_map"),
                add_material_count_to_result: true,
            },
            2,
            1,
            vec![Some("minecraft:filled_map"), Some("minecraft:map")],
            true,
            Some(ItemAmount::one("minecraft:filled_map")),
        ),
        (
            "minecraft:tipped_arrow",
            RecipeKind::Imbue {
                source: IngredientSpec::Item("minecraft:lingering_potion"),
                material: IngredientSpec::Item("minecraft:arrow"),
                result: ItemAmount {
                    item: "minecraft:tipped_arrow",
                    count: 8,
                },
            },
            3,
            3,
            vec![
                Some("minecraft:arrow"),
                Some("minecraft:arrow"),
                Some("minecraft:arrow"),
                Some("minecraft:arrow"),
                Some("minecraft:lingering_potion"),
                Some("minecraft:arrow"),
                Some("minecraft:arrow"),
                Some("minecraft:arrow"),
                Some("minecraft:arrow"),
            ],
            true,
            Some(ItemAmount {
                item: "minecraft:tipped_arrow",
                count: 8,
            }),
        ),
    ]
}

fn smithing_and_special_recipe_coverage_cases() -> Vec<RecipeKindCoverageCase> {
    vec![
        (
            "minecraft:netherite_sword_smithing",
            RecipeKind::SmithingTransform {
                template: IngredientSpec::Item("minecraft:netherite_upgrade_smithing_template"),
                base: IngredientSpec::Item("minecraft:diamond_sword"),
                addition: IngredientSpec::Item("minecraft:netherite_ingot"),
                result: ItemAmount::one("minecraft:netherite_sword"),
            },
            3,
            1,
            vec![
                Some("minecraft:netherite_upgrade_smithing_template"),
                Some("minecraft:diamond_sword"),
                Some("minecraft:netherite_ingot"),
            ],
            true,
            Some(ItemAmount::one("minecraft:netherite_sword")),
        ),
        (
            "minecraft:spire_armor_trim_smithing",
            RecipeKind::SmithingTrim {
                template: IngredientSpec::Item("minecraft:spire_armor_trim_smithing_template"),
                base: IngredientSpec::Item("minecraft:iron_chestplate"),
                addition: IngredientSpec::Item("minecraft:amethyst_shard"),
                pattern: "minecraft:spire",
            },
            3,
            1,
            vec![
                Some("minecraft:spire_armor_trim_smithing_template"),
                Some("minecraft:iron_chestplate"),
                Some("minecraft:amethyst_shard"),
            ],
            true,
            None,
        ),
        (
            "minecraft:repair_item",
            RecipeKind::Special {
                kind: SpecialRecipeKind::RepairItem,
                result_hint: None,
            },
            2,
            1,
            vec![
                Some("minecraft:diamond_pickaxe"),
                Some("minecraft:diamond_pickaxe"),
            ],
            false,
            None,
        ),
    ]
}

#[test]
fn stonecutter_selectable_recipes_filter_all_outputs_for_input() {
    let recipes = vec![
        StonecutterSelection {
            recipe_id: "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting",
            input: IngredientSpec::Item("minecraft:smooth_stone"),
            result: ItemAmount {
                item: "minecraft:smooth_stone_slab",
                count: 2,
            },
        },
        StonecutterSelection {
            recipe_id: "minecraft:stone_slab_from_stone_stonecutting",
            input: IngredientSpec::Item("minecraft:stone"),
            result: ItemAmount {
                item: "minecraft:stone_slab",
                count: 2,
            },
        },
        StonecutterSelection {
            recipe_id: "minecraft:stone_bricks_from_stone_stonecutting",
            input: IngredientSpec::AnyOf(vec!["minecraft:stone"]),
            result: ItemAmount::one("minecraft:stone_bricks"),
        },
    ];

    let selected = stonecutter_recipes_for_input(&recipes, "minecraft:smooth_stone");
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].recipe_id,
        "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting"
    );
    assert_eq!(
        selected[0].result,
        ItemAmount {
            item: "minecraft:smooth_stone_slab",
            count: 2,
        }
    );

    let stone_outputs = stonecutter_recipes_for_input(&recipes, "minecraft:stone");
    assert_eq!(stone_outputs.len(), 2);
    assert!(stone_outputs
        .iter()
        .any(|recipe| recipe.recipe_id == "minecraft:stone_slab_from_stone_stonecutting"));
    assert!(stone_outputs
        .iter()
        .any(|recipe| recipe.recipe_id == "minecraft:stone_bricks_from_stone_stonecutting"));
}

#[test]
fn smithing_transform_preserves_original_components() {
    let base = SmithingComponentStack {
        item: "minecraft:diamond_sword",
        count: 1,
        custom_name: Some("Silk Edge"),
        enchantments: vec![
            EnchantmentComponent {
                id: "minecraft:sharpness",
                level: 5,
            },
            EnchantmentComponent {
                id: "minecraft:unbreaking",
                level: 3,
            },
        ],
        trim: None,
    };

    let result = smithing_transform_result("minecraft:netherite_sword", &base);
    assert_eq!(result.item, "minecraft:netherite_sword");
    assert_eq!(result.count, 1);
    assert_eq!(result.custom_name, Some("Silk Edge"));
    assert_eq!(result.enchantments, base.enchantments);
}

#[test]
fn smithing_trim_applies_material_and_pattern_components() {
    let base = SmithingComponentStack::one("minecraft:iron_chestplate");
    let trimmed = smithing_trim_result(&base, Some("minecraft:amethyst"), "minecraft:spire")
        .expect("valid trim material should produce a trimmed copy");

    assert_eq!(trimmed.item, "minecraft:iron_chestplate");
    assert_eq!(trimmed.count, 1);
    assert_eq!(
        trimmed.trim,
        Some(ArmorTrimComponent {
            material: "minecraft:amethyst",
            pattern: "minecraft:spire",
        })
    );
    assert_eq!(
        smithing_trim_result(&trimmed, Some("minecraft:amethyst"), "minecraft:spire"),
        None
    );
    assert_eq!(smithing_trim_result(&base, None, "minecraft:spire"), None);
}

#[test]
fn placement_info_matches_vanilla_slot_index_contracts() {
    let single = PlacementInfo::create(IngredientSpec::Item("minecraft:stone"));
    assert_eq!(
        single.ingredients,
        vec![IngredientSpec::Item("minecraft:stone")]
    );
    assert_eq!(single.slots_to_ingredient_index, vec![0]);
    assert!(!single.is_impossible_to_place());

    let smithing_transform = PlacementInfo::create_from_optionals(vec![
        None,
        Some(IngredientSpec::Item("minecraft:diamond_sword")),
        Some(IngredientSpec::Item("minecraft:netherite_ingot")),
    ]);
    assert_eq!(
        smithing_transform.ingredients,
        vec![
            IngredientSpec::Item("minecraft:diamond_sword"),
            IngredientSpec::Item("minecraft:netherite_ingot"),
        ]
    );
    assert_eq!(
        smithing_transform.slots_to_ingredient_index,
        vec![PlacementInfo::EMPTY_SLOT, 0, 1]
    );

    let impossible = PlacementInfo::create_list(vec![
        IngredientSpec::Item("minecraft:stick"),
        IngredientSpec::Empty,
    ]);
    assert!(impossible.is_impossible_to_place());

    let simple = SimpleSmithingRecipeModel::new(true, smithing_transform.clone());
    assert_eq!(simple.group(), "");
    assert!(simple.show_notification);
    assert_eq!(simple.placement_info(), &smithing_transform);
}

#[test]
fn recipe_manager_indexes_by_type_key_and_matching_input() {
    let manager = RecipeManagerModel::new(vec![
        RecipeHolder {
            id: "minecraft:crafting_table",
            recipe: RecipeKind::Shaped {
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
            },
        },
        RecipeHolder {
            id: "minecraft:firework_star",
            recipe: RecipeKind::Shapeless {
                ingredients: vec![
                    IngredientSpec::Item("minecraft:gunpowder"),
                    IngredientSpec::AnyOf(vec!["minecraft:red_dye", "minecraft:blue_dye"]),
                ],
                result: ItemAmount::one("minecraft:firework_star"),
                category: crate::recipe_system::CraftingBookCategoryModel::Misc,
            },
        },
        RecipeHolder {
            id: "minecraft:iron_ingot_from_smelting_raw_iron",
            recipe: RecipeKind::Cooking {
                kind: CookingKind::Smelting,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: None,
                category: crate::recipe_system::CookingBookCategory::Misc,
            },
        },
        RecipeHolder {
            id: "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting",
            recipe: RecipeKind::Stonecutting {
                ingredient: IngredientSpec::Item("minecraft:smooth_stone"),
                result: ItemAmount {
                    item: "minecraft:smooth_stone_slab",
                    count: 2,
                },
            },
        },
    ]);

    assert_eq!(manager.recipe_map().values().len(), 4);
    assert_eq!(
        manager
            .recipe_map()
            .by_key("minecraft:crafting_table")
            .unwrap()
            .recipe
            .serializer(),
        "crafting_shaped"
    );
    assert_eq!(manager.recipe_map().by_type("crafting").len(), 2);
    assert_eq!(manager.recipe_map().by_type("smelting").len(), 1);

    let shaped = manager.recipe_map().get_recipe_for(
        "crafting",
        2,
        2,
        &[
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
        ],
    );
    assert_eq!(shaped.unwrap().id, "minecraft:crafting_table");
    assert!(manager
        .recipe_map()
        .get_recipe_for("crafting", 2, 2, &[None, None, None, None])
        .is_none());

    let cooking =
        manager
            .recipe_map()
            .get_recipe_for("smelting", 1, 1, &[Some("minecraft:raw_iron")]);
    assert_eq!(
        cooking.unwrap().id,
        "minecraft:iron_ingot_from_smelting_raw_iron"
    );
    assert_eq!(
        manager
            .property_set("minecraft:furnace_input")
            .accepted_items,
        vec!["minecraft:raw_iron"]
    );
    assert_eq!(manager.stonecutter_recipes().len(), 1);
}

#[test]
fn recipe_manager_reload_replaces_indexes_and_recipe_access_sets() {
    let mut manager = RecipeManagerModel::new(vec![RecipeHolder {
        id: "minecraft:iron_ingot_from_smelting_raw_iron",
        recipe: RecipeKind::Cooking {
            kind: CookingKind::Smelting,
            ingredient: IngredientSpec::Item("minecraft:raw_iron"),
            result: ItemAmount::one("minecraft:iron_ingot"),
            experience_millis: 700,
            cooking_time: None,
            category: crate::recipe_system::CookingBookCategory::Misc,
        },
    }]);

    assert!(manager
        .recipe_map()
        .by_key("minecraft:iron_ingot_from_smelting_raw_iron")
        .is_some());
    manager.reload(vec![RecipeHolder {
        id: "minecraft:netherite_sword_smithing",
        recipe: RecipeKind::SmithingTransform {
            template: IngredientSpec::Item("minecraft:netherite_upgrade_smithing_template"),
            base: IngredientSpec::Item("minecraft:diamond_sword"),
            addition: IngredientSpec::Item("minecraft:netherite_ingot"),
            result: ItemAmount::one("minecraft:netherite_sword"),
        },
    }]);

    assert!(manager
        .recipe_map()
        .by_key("minecraft:iron_ingot_from_smelting_raw_iron")
        .is_none());
    assert_eq!(manager.recipe_map().by_type("smithing").len(), 1);
    assert_eq!(
        manager
            .property_set("minecraft:smithing_template")
            .accepted_items,
        vec!["minecraft:netherite_upgrade_smithing_template"]
    );
    assert_eq!(
        manager
            .property_set("minecraft:smithing_base")
            .accepted_items,
        vec!["minecraft:diamond_sword"]
    );
    assert!(manager.stonecutter_recipes().is_empty());
}

#[test]
fn core_recipe_interface_methods_match_java_defaults() {
    let holder = RecipeHolder {
        id: "minecraft:crafting_table",
        recipe: RecipeKind::Shaped {
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
        },
    };
    assert_eq!(holder.get_id(), "minecraft:crafting_table");
    assert_eq!(holder.get_serializer(), "crafting_shaped");
    assert_eq!(holder.get_type(), "crafting");
    assert_eq!(
        holder.get_result_item(),
        Some(ItemAmount::one("minecraft:crafting_table"))
    );
    assert!(!holder.recipe.is_special());
    assert!(holder.recipe.show_notification());
    assert_eq!(holder.recipe.recipe_book_category(), "crafting_misc");

    let special = RecipeKind::Special {
        kind: SpecialRecipeKind::RepairItem,
        result_hint: None,
    };
    assert!(special.is_special());
    assert!(!special.show_notification());
    assert_eq!(special.recipe_book_category(), "crafting_misc");

    let normal = NormalCraftingRecipeModel {
        category: CraftingBookCategoryModel::Equipment,
        group: "tools",
        show_notification: true,
        placement_info: PlacementInfo::create(IngredientSpec::Item("minecraft:stick")),
    };
    assert_eq!(normal.recipe_book_category(), "crafting_equipment");
    assert_eq!(normal.group, "tools");
    assert!(!normal.is_incomplete());
    assert!(!normal.placement_info.is_impossible_to_place());

    let categories = [
        (
            CraftingBookCategoryModel::Building,
            "crafting_building_blocks",
        ),
        (CraftingBookCategoryModel::Equipment, "crafting_equipment"),
        (CraftingBookCategoryModel::Redstone, "crafting_redstone"),
        (CraftingBookCategoryModel::Misc, "crafting_misc"),
    ];
    for (category, expected) in categories {
        assert_eq!(category.recipe_book_category(), expected);
    }
}

#[test]
fn crafting_recipe_default_remaining_items_uses_item_remainders() {
    let remaining = default_crafting_remaining_items(&[
        Some(CraftingStack::one("minecraft:water_bucket")),
        Some(CraftingStack::one("minecraft:potion")),
        Some(CraftingStack::one("minecraft:oak_planks")),
        None,
    ]);
    assert_eq!(
        remaining,
        vec![
            Some(CraftingStack::one("minecraft:bucket")),
            Some(CraftingStack::one("minecraft:glass_bottle")),
            None,
            None,
        ]
    );
}

#[test]
fn recipe_json_loader_decodes_representative_vanilla_files() {
    // Use an empty tag map for these unit tests; they exercise recipe parsing
    // for ingredient types that are direct item IDs, not tag references.
    let no_tags = ItemTagMap::default();

    assert_representative_core_recipe_json_decodes(&no_tags);
    assert_representative_special_recipe_json_decodes(&no_tags);
    assert_invalid_special_recipe_json_is_rejected(&no_tags);
}

fn assert_representative_core_recipe_json_decodes(no_tags: &ItemTagMap) {
    let shaped = load_recipe_json(
        "minecraft:crafting_table",
        include_str!(
            "../../../vanilla-data/data/minecraft/recipe/crafting_table.json"
        ),
        no_tags,
    )
    .expect("crafting table recipe should decode");
    assert_eq!(shaped.recipe.serializer(), "crafting_shaped");
    assert_eq!(shaped.recipe.recipe_type(), "crafting");

    let smelting = load_recipe_json(
        "minecraft:iron_ingot_from_smelting_raw_iron",
        include_str!(
            "../../../vanilla-data/data/minecraft/recipe/iron_ingot_from_smelting_raw_iron.json"
        ),
        no_tags,
    )
    .expect("smelting recipe should decode");
    assert_eq!(smelting.recipe.serializer(), "smelting");
    assert_eq!(smelting.recipe.cooking_time(), Some(200));

    let stonecutting = load_recipe_json(
        "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting",
        include_str!(
            "../../../vanilla-data/data/minecraft/recipe/smooth_stone_slab_from_smooth_stone_stonecutting.json"
        ),
        no_tags,
    )
    .expect("stonecutting recipe should decode");
    assert_eq!(stonecutting.recipe.serializer(), "stonecutting");
    assert_eq!(
        stonecutting.recipe.assemble(),
        Some(ItemAmount {
            item: "minecraft:smooth_stone_slab",
            count: 2,
        })
    );
}

fn assert_representative_special_recipe_json_decodes(no_tags: &ItemTagMap) {
    for (id, raw, serializer) in [
        (
            "minecraft:white_banner_duplicate",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/white_banner_duplicate.json"
            ),
            "crafting_special_bannerduplicate",
        ),
        (
            "minecraft:book_cloning",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/book_cloning.json"
            ),
            "crafting_special_bookcloning",
        ),
        (
            "minecraft:decorated_pot",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/decorated_pot.json"
            ),
            "crafting_decorated_pot",
        ),
        (
            "minecraft:leather_helmet_dyed",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/leather_helmet_dyed.json"
            ),
            "crafting_dye",
        ),
        (
            "minecraft:firework_rocket",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/firework_rocket.json"
            ),
            "crafting_special_firework_rocket",
        ),
        (
            "minecraft:firework_star",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/firework_star.json"
            ),
            "crafting_special_firework_star",
        ),
        (
            "minecraft:firework_star_fade",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/firework_star_fade.json"
            ),
            "crafting_special_firework_star_fade",
        ),
        (
            "minecraft:map_extending",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/map_extending.json"
            ),
            "crafting_special_mapextending",
        ),
        (
            "minecraft:repair_item",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/repair_item.json"
            ),
            "crafting_special_repairitem",
        ),
        (
            "minecraft:shield_decoration",
            include_str!(
                "../../../vanilla-data/data/minecraft/recipe/shield_decoration.json"
            ),
            "crafting_special_shielddecoration",
        ),
    ] {
        let recipe =
            load_recipe_json(id, raw, no_tags).expect("special recipe JSON should decode");
        assert_eq!(recipe.recipe.serializer(), serializer);
    }
}

fn assert_invalid_special_recipe_json_is_rejected(no_tags: &ItemTagMap) {
    assert!(
        load_recipe_json(
            "minecraft:bad_book_cloning",
            r#"{"type":"minecraft:crafting_special_bookcloning","source":"minecraft:written_book","result":{"id":"minecraft:written_book"}}"#,
            no_tags,
        )
        .is_err(),
        "book cloning JSON must decode its material field"
    );
    assert!(
        load_recipe_json(
            "minecraft:bad_firework_star",
            r##"{"type":"minecraft:crafting_special_firework_star","shapes":{"huge":"minecraft:stone"},"trail":"minecraft:diamond","twinkle":"minecraft:glowstone_dust","fuel":"minecraft:gunpowder","dye":"#minecraft:dyes","result":{"id":"minecraft:firework_star"}}"##,
            no_tags,
        )
        .is_err(),
        "firework star JSON must reject unknown shape keys"
    );
}

#[test]
fn recipe_manager_loads_all_vanilla_recipe_json_files() {
    let recipe_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    let manager = load_recipe_directory(&recipe_dir).expect("vanilla recipe directory should load");
    assert_eq!(manager.recipe_map().values().len(), 1515);
    assert_eq!(manager.recipe_map().by_type("crafting").len(), 1094);
    assert_eq!(manager.recipe_map().by_type("smelting").len(), 73);
    assert_eq!(manager.recipe_map().by_type("stonecutting").len(), 275);
    assert_eq!(manager.stonecutter_recipes().len(), 275);
}

#[test]
fn recipe_manager_loads_vanilla_inventory_recipe_unlocks_from_advancements() {
    let recipe_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    let manager = load_recipe_directory(&recipe_dir).expect("vanilla recipe directory should load");

    let oak_log_unlocks = manager.recipes_unlocked_by_item("minecraft:oak_log");
    assert!(
        oak_log_unlocks.contains(&"minecraft:oak_planks"),
        "minecraft:oak_log should unlock minecraft:oak_planks; got {oak_log_unlocks:?}"
    );

    let stripped_oak_unlocks = manager.recipes_unlocked_by_item("minecraft:stripped_oak_wood");
    assert!(
        stripped_oak_unlocks.contains(&"minecraft:oak_planks"),
        "minecraft:stripped_oak_wood should unlock minecraft:oak_planks via #minecraft:oak_logs"
    );

    assert!(
        !manager
            .recipes_unlocked_by_item("minecraft:cobblestone")
            .contains(&"minecraft:oak_planks"),
        "non-log items must not unlock oak planks"
    );

    assert!(
        manager
            .initially_unlocked_recipes()
            .contains(&"minecraft:crafting_table"),
        "minecraft:crafting_table is unlocked by the Java recipe advancement's minecraft:tick criterion"
    );
}

#[test]
fn special_recipe_kinds_cover_checklist_families_and_serializer_names() {
    let special = [
        (SpecialRecipeKind::Transmute, "crafting_transmute"),
        (SpecialRecipeKind::MapCloning, "crafting_special_mapcloning"),
        (
            SpecialRecipeKind::MapExtending,
            "crafting_special_mapextending",
        ),
        (
            SpecialRecipeKind::BannerDuplicate,
            "crafting_special_bannerduplicate",
        ),
        (
            SpecialRecipeKind::ShieldDecoration,
            "crafting_special_shielddecoration",
        ),
        (
            SpecialRecipeKind::FireworkRocket,
            "crafting_special_firework_rocket",
        ),
        (
            SpecialRecipeKind::FireworkStar,
            "crafting_special_firework_star",
        ),
        (
            SpecialRecipeKind::FireworkStarFade,
            "crafting_special_firework_star_fade",
        ),
        (
            SpecialRecipeKind::SuspiciousStew,
            "crafting_special_suspiciousstew",
        ),
        (
            SpecialRecipeKind::BookCloning,
            "crafting_special_bookcloning",
        ),
        (SpecialRecipeKind::RepairItem, "crafting_special_repairitem"),
        (SpecialRecipeKind::DyedItem, "crafting_dye"),
        (SpecialRecipeKind::DecoratedPot, "crafting_decorated_pot"),
        (SpecialRecipeKind::Imbue, "crafting_imbue"),
    ];

    for (kind, serializer) in special {
        let recipe = RecipeKind::Special {
            kind,
            result_hint: None,
        };
        assert_eq!(recipe.serializer(), serializer);
        assert!(!recipe.matches(3, 3, &[None; 9]));
        assert_eq!(recipe.assemble(), None);
    }
}

#[test]
fn item_tag_map_resolves_tags_recursively() {
    // Load the real vanilla tag data so we test the full resolution chain:
    //   minecraft:logs → minecraft:logs_that_burn → minecraft:oak_logs → minecraft:oak_log
    let tag_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data/data/minecraft/tags/item");
    let tags = load_item_tag_directory(&tag_dir);

    // A simple leaf tag: oak_logs resolves to the four concrete oak log variants.
    let oak_logs = tags.resolve("minecraft:oak_logs");
    assert!(
        oak_logs.contains(&"minecraft:oak_log"),
        "minecraft:oak_logs should contain minecraft:oak_log; got {oak_logs:?}"
    );
    assert!(
        oak_logs.contains(&"minecraft:oak_wood"),
        "minecraft:oak_logs should contain minecraft:oak_wood"
    );
    assert!(
        oak_logs.contains(&"minecraft:stripped_oak_log"),
        "minecraft:oak_logs should contain minecraft:stripped_oak_log"
    );
    assert!(
        oak_logs.contains(&"minecraft:stripped_oak_wood"),
        "minecraft:oak_logs should contain minecraft:stripped_oak_wood"
    );

    // A multi-level tag: logs → logs_that_burn → oak_logs → oak_log
    let logs = tags.resolve("minecraft:logs");
    assert!(
        logs.contains(&"minecraft:oak_log"),
        "minecraft:logs should contain minecraft:oak_log via recursive resolution; got {logs:?}"
    );
    assert!(
        logs.contains(&"minecraft:spruce_log"),
        "minecraft:logs should contain minecraft:spruce_log"
    );

    // An unknown tag should return an empty slice without panicking.
    let unknown = tags.resolve("minecraft:does_not_exist");
    assert!(
        unknown.is_empty(),
        "unknown tag should resolve to empty slice"
    );
}

#[test]
fn recipe_manager_loads_vanilla_recipes_with_tag_ingredients() {
    // This is the integration test that proves crafting actually works end-to-end.
    // load_recipe_directory loads tags automatically from ../tags/item/ relative
    // to the recipe directory.
    let recipe_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    let manager = load_recipe_directory(&recipe_dir).expect("vanilla recipe directory should load");

    // oak_planks: type=crafting_shapeless, ingredient=#minecraft:oak_logs
    // Any oak log variant in slot 0 of a 1×1 grid should match.
    let oak_planks_from_log =
        manager
            .recipe_map()
            .get_recipe_for("crafting", 1, 1, &[Some("minecraft:oak_log")]);
    assert!(
        oak_planks_from_log.is_some(),
        "should find oak_planks recipe for minecraft:oak_log (tag #minecraft:oak_logs)"
    );
    assert_eq!(
        oak_planks_from_log.unwrap().recipe.assemble().unwrap().item,
        "minecraft:oak_planks"
    );

    // Stripped oak log is also in #minecraft:oak_logs — it should match too.
    let oak_planks_from_stripped = manager.recipe_map().get_recipe_for(
        "crafting",
        1,
        1,
        &[Some("minecraft:stripped_oak_log")],
    );
    assert!(
        oak_planks_from_stripped.is_some(),
        "should find oak_planks recipe for minecraft:stripped_oak_log"
    );

    // crafting_table: shaped 2×2, each slot = minecraft:oak_planks (direct item ID)
    let crafting_table = manager.recipe_map().get_recipe_for(
        "crafting",
        2,
        2,
        &[
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
        ],
    );
    assert!(
        crafting_table.is_some(),
        "should find crafting_table recipe for 2×2 oak_planks"
    );
    assert_eq!(crafting_table.unwrap().id, "minecraft:crafting_table");
}
