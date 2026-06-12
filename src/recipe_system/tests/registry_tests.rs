use super::*;

fn ids(entries: &[RegistryEntry]) -> Vec<&'static str> {
    entries.iter().map(|entry| entry.id).collect()
}

#[test]
fn recipe_registries_match_vanilla_bootstrap_order() {
    assert_eq!(
        ids(RECIPE_TYPES),
        vec![
            "crafting",
            "smelting",
            "blasting",
            "smoking",
            "campfire_cooking",
            "stonecutting",
            "smithing"
        ]
    );
    assert_eq!(
        ids(RECIPE_SERIALIZERS),
        vec![
            "crafting_shaped",
            "crafting_shapeless",
            "crafting_dye",
            "crafting_imbue",
            "crafting_transmute",
            "crafting_decorated_pot",
            "crafting_special_bookcloning",
            "crafting_special_mapextending",
            "crafting_special_firework_rocket",
            "crafting_special_firework_star",
            "crafting_special_firework_star_fade",
            "crafting_special_bannerduplicate",
            "crafting_special_shielddecoration",
            "crafting_special_repairitem",
            "smelting",
            "blasting",
            "smoking",
            "campfire_cooking",
            "stonecutting",
            "smithing_transform",
            "smithing_trim"
        ]
    );
    assert_eq!(
        ids(RECIPE_DISPLAY_TYPES),
        vec![
            "crafting_shapeless",
            "crafting_shaped",
            "furnace",
            "stonecutter",
            "smithing"
        ]
    );
    assert_eq!(
        ids(SLOT_DISPLAY_TYPES),
        vec![
            "empty",
            "any_fuel",
            "with_any_potion",
            "only_with_component",
            "item",
            "item_stack",
            "tag",
            "dyed",
            "smithing_trim",
            "with_remainder",
            "composite"
        ]
    );
    assert_eq!(
        ids(RECIPE_BOOK_CATEGORIES),
        vec![
            "crafting_building_blocks",
            "crafting_redstone",
            "crafting_equipment",
            "crafting_misc",
            "furnace_food",
            "furnace_blocks",
            "furnace_misc",
            "blast_furnace_blocks",
            "blast_furnace_misc",
            "smoker_food",
            "stonecutter",
            "smithing",
            "campfire"
        ]
    );
}

fn assert_recipe_ids(recipe: RecipeKind, recipe_type: &str, serializer: &str) {
    assert_eq!(recipe.recipe_type(), recipe_type);
    assert_eq!(recipe.serializer(), serializer);
}

#[test]
fn normal_crafting_recipe_kind_ids_match_vanilla_registries() {
    let item = IngredientSpec::Item("minecraft:stone");
    let result = ItemAmount::one("minecraft:stone");

    assert_recipe_ids(
        RecipeKind::Shaped {
            width: 1,
            height: 1,
            pattern: vec![Some(item.clone())],
            result: result.clone(),
            category: CraftingBookCategoryModel::Misc,
        },
        "crafting",
        "crafting_shaped",
    );
    assert_recipe_ids(
        RecipeKind::Shapeless {
            ingredients: vec![item.clone()],
            result: result.clone(),
            category: CraftingBookCategoryModel::Misc,
        },
        "crafting",
        "crafting_shapeless",
    );
    for (recipe, serializer) in [
        (
            RecipeKind::Transmute {
                input: item.clone(),
                material: item.clone(),
                min_material_count: 1,
                max_material_count: 1,
                result: result.clone(),
                add_material_count_to_result: false,
            },
            "crafting_transmute",
        ),
        (
            RecipeKind::Imbue {
                source: item.clone(),
                material: item.clone(),
                result: result.clone(),
            },
            "crafting_imbue",
        ),
    ] {
        assert_recipe_ids(recipe, "crafting", serializer);
    }
}

#[test]
fn special_crafting_recipe_kind_ids_match_vanilla_registries() {
    for (kind, serializer) in [
        (
            SpecialRecipeKind::BookCloning,
            "crafting_special_bookcloning",
        ),
        (
            SpecialRecipeKind::MapExtending,
            "crafting_special_mapextending",
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
            SpecialRecipeKind::BannerDuplicate,
            "crafting_special_bannerduplicate",
        ),
        (
            SpecialRecipeKind::ShieldDecoration,
            "crafting_special_shielddecoration",
        ),
        (SpecialRecipeKind::RepairItem, "crafting_special_repairitem"),
        (SpecialRecipeKind::DyedItem, "crafting_dye"),
        (SpecialRecipeKind::DecoratedPot, "crafting_decorated_pot"),
    ] {
        assert_recipe_ids(
            RecipeKind::Special {
                kind,
                result_hint: None,
            },
            "crafting",
            serializer,
        );
    }
}

#[test]
fn cooking_recipe_kind_ids_match_vanilla_registries() {
    let item = IngredientSpec::Item("minecraft:stone");
    let result = ItemAmount::one("minecraft:stone");

    for (kind, recipe_type) in [
        (CookingKind::Smelting, "smelting"),
        (CookingKind::Blasting, "blasting"),
        (CookingKind::Smoking, "smoking"),
        (CookingKind::CampfireCooking, "campfire_cooking"),
    ] {
        assert_recipe_ids(
            RecipeKind::Cooking {
                kind,
                ingredient: item.clone(),
                result: result.clone(),
                experience_millis: 0,
                cooking_time: None,
                category: CookingBookCategory::Misc,
            },
            recipe_type,
            recipe_type,
        );
    }
}

#[test]
fn single_input_and_smithing_recipe_kind_ids_match_vanilla_registries() {
    let item = IngredientSpec::Item("minecraft:stone");
    let result = ItemAmount::one("minecraft:stone");

    assert_recipe_ids(
        RecipeKind::Stonecutting {
            ingredient: item.clone(),
            result: result.clone(),
        },
        "stonecutting",
        "stonecutting",
    );
    assert_recipe_ids(
        RecipeKind::SmithingTransform {
            template: item.clone(),
            base: item.clone(),
            addition: item.clone(),
            result,
        },
        "smithing",
        "smithing_transform",
    );
    assert_recipe_ids(
        RecipeKind::SmithingTrim {
            template: item.clone(),
            base: item.clone(),
            addition: item,
            pattern: "minecraft:sentry",
        },
        "smithing",
        "smithing_trim",
    );
}

#[test]
fn recipe_holder_identity_matches_java_resource_key_semantics() {
    let crafting = RecipeHolder {
        id: "minecraft:shared_id",
        recipe: RecipeKind::Shaped {
            width: 1,
            height: 1,
            pattern: vec![Some(IngredientSpec::Item("minecraft:stone"))],
            result: ItemAmount::one("minecraft:stone_button"),
            category: CraftingBookCategoryModel::Misc,
        },
    };
    let cooking = RecipeHolder {
        id: "minecraft:shared_id",
        recipe: RecipeKind::Cooking {
            kind: CookingKind::Smelting,
            ingredient: IngredientSpec::Item("minecraft:raw_iron"),
            result: ItemAmount::one("minecraft:iron_ingot"),
            experience_millis: 700,
            cooking_time: None,
            category: CookingBookCategory::Misc,
        },
    };
    let different_id = RecipeHolder {
        id: "minecraft:different_id",
        recipe: crafting.recipe.clone(),
    };

    assert_eq!(crafting, cooking);
    assert_ne!(crafting, different_id);
    assert_eq!(
        crafting.to_string(),
        "ResourceKey[minecraft:recipe / minecraft:shared_id]"
    );

    let mut holders = std::collections::HashSet::new();
    holders.insert(crafting);
    holders.insert(cooking);
    holders.insert(different_id);
    assert_eq!(holders.len(), 2);
}

#[test]
fn recipe_map_values_and_type_filters_keep_java_builder_order() {
    let recipes = RecipeMap::create(vec![
        RecipeHolder {
            id: "minecraft:crafting_table",
            recipe: RecipeKind::Shaped {
                width: 1,
                height: 1,
                pattern: vec![Some(IngredientSpec::Item("minecraft:oak_planks"))],
                result: ItemAmount::one("minecraft:crafting_table"),
                category: CraftingBookCategoryModel::Misc,
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
                category: CookingBookCategory::Misc,
            },
        },
    ]);

    assert_eq!(
        recipes
            .values()
            .iter()
            .map(|holder| holder.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:crafting_table",
            "minecraft:iron_ingot_from_smelting_raw_iron"
        ]
    );
    assert_eq!(
        recipes
            .by_type("crafting")
            .iter()
            .map(|holder| holder.id)
            .collect::<Vec<_>>(),
        vec!["minecraft:crafting_table"]
    );
}
