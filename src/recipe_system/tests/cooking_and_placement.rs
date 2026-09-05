use super::*;

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
