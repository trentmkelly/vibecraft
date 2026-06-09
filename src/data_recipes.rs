use std::collections::BTreeMap;

const CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/recipes/CustomCraftingRecipeBuilder.java"
);
const RECIPE_BUILDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeBuilder.java");
const RECIPE_CATEGORY_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeCategory.java");
const RECIPE_OUTPUT_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeOutput.java");
const RECIPE_PROVIDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeProvider.java");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataRecipeCategory {
    BuildingBlocks,
    Decorations,
    Redstone,
    Transportation,
    Tools,
    Combat,
    Food,
    Brewing,
    Misc,
}

impl DataRecipeCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::BuildingBlocks => "building_blocks",
            Self::Decorations => "decorations",
            Self::Redstone => "redstone",
            Self::Transportation => "transportation",
            Self::Tools => "tools",
            Self::Combat => "combat",
            Self::Food => "food",
            Self::Brewing => "brewing",
            Self::Misc => "misc",
        }
    }

    fn crafting_book_category(self) -> &'static str {
        match self {
            Self::BuildingBlocks => "building",
            Self::Tools | Self::Combat => "equipment",
            Self::Redstone => "redstone",
            Self::Decorations | Self::Transportation | Self::Food | Self::Brewing | Self::Misc => {
                "misc"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CraftingCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CraftingBookInfoModel {
    category: &'static str,
    group: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomCraftingRecipeBuilderModel {
    category: DataRecipeCategory,
    group: Option<String>,
    advancement_criteria: BTreeMap<String, String>,
    factory_name: &'static str,
}

impl CustomCraftingRecipeBuilderModel {
    fn custom_crafting(category: DataRecipeCategory, factory_name: &'static str) -> Self {
        Self {
            category,
            group: None,
            advancement_criteria: BTreeMap::new(),
            factory_name,
        }
    }

    fn unlocked_by(mut self, name: &str, criterion: &str) -> Self {
        self.advancement_criteria
            .insert(name.to_string(), criterion.to_string());
        self
    }

    fn group(mut self, group: Option<&str>) -> Self {
        self.group = group.map(str::to_string);
        self
    }

    fn save_by_name(&self, name: &str) -> CustomCraftingSaveModel {
        self.save_by_key(&format!("minecraft:{name}"))
    }

    fn save_by_key(&self, id: &str) -> CustomCraftingSaveModel {
        CustomCraftingSaveModel {
            id: id.to_string(),
            common_info: create_crafting_common_info(true),
            book_info: create_crafting_book_info(self.category, self.group.as_deref()),
            factory_name: self.factory_name,
            advancement_criteria: self.advancement_criteria.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomCraftingSaveModel {
    id: String,
    common_info: CraftingCommonInfoModel,
    book_info: CraftingBookInfoModel,
    factory_name: &'static str,
    advancement_criteria: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AcceptedRecipeModel {
    id: String,
    recipe: String,
    advancement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecipeOutputModel {
    accepted: Vec<AcceptedRecipeModel>,
    advancement_builder: String,
    root_advancement_included: bool,
}

impl RecipeOutputModel {
    fn new(advancement_builder: &str) -> Self {
        Self {
            accepted: Vec::new(),
            advancement_builder: advancement_builder.to_string(),
            root_advancement_included: false,
        }
    }

    fn accept(&mut self, id: &str, recipe: &str, advancement: Option<&str>) {
        self.accepted.push(AcceptedRecipeModel {
            id: id.to_string(),
            recipe: recipe.to_string(),
            advancement: advancement.map(str::to_string),
        });
    }

    fn advancement(&self) -> &str {
        &self.advancement_builder
    }

    fn include_root_advancement(&mut self) {
        self.root_advancement_included = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecipeProviderPlanKind {
    Shaped,
    Shapeless,
    Cooking(&'static str),
    SmithingTransform,
    SmithingTrim,
    Stonecutting,
    Special(&'static str),
    Transmute,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecipeProviderPlan {
    kind: RecipeProviderPlanKind,
    category: DataRecipeCategory,
    result: String,
    count: u32,
    ingredients: Vec<String>,
    patterns: Vec<&'static str>,
    group: Option<String>,
    unlocks: Vec<String>,
    id: String,
}

impl RecipeProviderPlan {
    fn shaped(category: DataRecipeCategory, result: &str, count: u32, id: &str) -> Self {
        Self {
            kind: RecipeProviderPlanKind::Shaped,
            category,
            result: result.to_string(),
            count,
            ingredients: Vec::new(),
            patterns: Vec::new(),
            group: None,
            unlocks: Vec::new(),
            id: id.to_string(),
        }
    }

    fn shapeless(category: DataRecipeCategory, result: &str, count: u32, id: &str) -> Self {
        Self {
            kind: RecipeProviderPlanKind::Shapeless,
            category,
            result: result.to_string(),
            count,
            ingredients: Vec::new(),
            patterns: Vec::new(),
            group: None,
            unlocks: Vec::new(),
            id: id.to_string(),
        }
    }

    fn ingredient(mut self, ingredient: &str) -> Self {
        self.ingredients.push(ingredient.to_string());
        self
    }

    fn ingredient_count(mut self, ingredient: &str, count: u32) -> Self {
        for _ in 0..count {
            self.ingredients.push(ingredient.to_string());
        }
        self
    }

    fn pattern(mut self, pattern: &'static str) -> Self {
        self.patterns.push(pattern);
        self
    }

    fn group(mut self, group: Option<&str>) -> Self {
        self.group = group.map(str::to_string);
        self
    }

    fn unlock(mut self, unlock: &str) -> Self {
        self.unlocks.push(unlock.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecipeProviderRunnerModel {
    seen_recipe_ids: Vec<String>,
    saved_recipes: Vec<String>,
    saved_advancements: Vec<String>,
}

impl RecipeProviderRunnerModel {
    fn new() -> Self {
        Self {
            seen_recipe_ids: Vec::new(),
            saved_recipes: Vec::new(),
            saved_advancements: Vec::new(),
        }
    }

    fn accept(
        &mut self,
        recipe_id: &str,
        recipe_payload: &str,
        advancement_id: Option<&str>,
    ) -> Result<(), String> {
        if self.seen_recipe_ids.iter().any(|seen| seen == recipe_id) {
            return Err(format!("Duplicate recipe {recipe_id}"));
        }
        self.seen_recipe_ids.push(recipe_id.to_string());
        self.saved_recipes
            .push(format!("recipe:{recipe_id}:{recipe_payload}"));
        if let Some(advancement_id) = advancement_id {
            self.saved_advancements
                .push(format!("advancement:{advancement_id}"));
        }
        Ok(())
    }

    fn advancement_parent(&self) -> &'static str {
        "recipes/root"
    }

    fn include_root_advancement(&mut self) {
        self.saved_advancements
            .push("advancement:recipes/root:impossible".to_string());
    }
}

fn provider_get_item_name(item: &str) -> String {
    item.strip_prefix("minecraft:").unwrap_or(item).to_string()
}

fn provider_get_has_name(item: &str) -> String {
    format!("has_{}", provider_get_item_name(item))
}

fn provider_get_conversion_recipe_name(product: &str, material: &str) -> String {
    format!(
        "{}_from_{}",
        provider_get_item_name(product),
        provider_get_item_name(material)
    )
}

fn provider_get_smelting_recipe_name(product: &str) -> String {
    format!("{}_from_smelting", provider_get_item_name(product))
}

fn provider_get_blasting_recipe_name(product: &str) -> String {
    format!("{}_from_blasting", provider_get_item_name(product))
}

fn provider_one_to_one_conversion(
    product: &str,
    resource: &str,
    group: Option<&str>,
    product_count: u32,
) -> RecipeProviderPlan {
    RecipeProviderPlan::shapeless(
        DataRecipeCategory::Misc,
        product,
        product_count,
        &provider_get_conversion_recipe_name(product, resource),
    )
    .ingredient(resource)
    .group(group)
    .unlock(&provider_get_has_name(resource))
}

fn provider_ore_cooking(
    recipe_kind: &'static str,
    smeltables: &[&str],
    result: &str,
    group: &str,
    suffix: &'static str,
) -> Vec<RecipeProviderPlan> {
    smeltables
        .iter()
        .map(|item| RecipeProviderPlan {
            kind: RecipeProviderPlanKind::Cooking(recipe_kind),
            category: DataRecipeCategory::Misc,
            result: result.to_string(),
            count: 1,
            ingredients: vec![(*item).to_string()],
            patterns: Vec::new(),
            group: Some(group.to_string()),
            unlocks: vec![provider_get_has_name(item)],
            id: format!(
                "{}{suffix}_{}",
                provider_get_item_name(result),
                provider_get_item_name(item)
            ),
        })
        .collect()
}

fn provider_two_by_two_packer(
    category: DataRecipeCategory,
    result: &str,
    ingredient: &str,
) -> RecipeProviderPlan {
    RecipeProviderPlan::shaped(category, result, 1, &provider_get_item_name(result))
        .ingredient(ingredient)
        .pattern("##")
        .pattern("##")
        .unlock(&provider_get_has_name(ingredient))
}

fn provider_three_by_three_packer(
    category: DataRecipeCategory,
    result: &str,
    ingredient: &str,
    unlocked_by: Option<&str>,
) -> RecipeProviderPlan {
    RecipeProviderPlan::shapeless(category, result, 1, &provider_get_item_name(result))
        .ingredient_count(ingredient, 9)
        .unlock(unlocked_by.unwrap_or(&provider_get_has_name(ingredient)))
}

fn provider_wood_from_logs(result: &str, log: &str) -> RecipeProviderPlan {
    RecipeProviderPlan::shaped(DataRecipeCategory::BuildingBlocks, result, 3, result)
        .ingredient(log)
        .pattern("##")
        .pattern("##")
        .group(Some("bark"))
        .unlock("has_log")
}

fn provider_wooden_boat(result: &str, planks: &str) -> RecipeProviderPlan {
    RecipeProviderPlan::shaped(DataRecipeCategory::Transportation, result, 1, result)
        .ingredient(planks)
        .pattern("# #")
        .pattern("###")
        .group(Some("boat"))
        .unlock("in_water")
}

fn provider_chest_boat(chest_boat: &str, boat: &str) -> RecipeProviderPlan {
    RecipeProviderPlan::shapeless(
        DataRecipeCategory::Transportation,
        chest_boat,
        1,
        chest_boat,
    )
    .ingredient("minecraft:chest")
    .ingredient(boat)
    .group(Some("chest_boat"))
    .unlock("has_boat")
}

fn provider_fence_builder(result: &str, base: &str) -> RecipeProviderPlan {
    let (count, stick) = if result == "minecraft:nether_brick_fence" {
        (6, "minecraft:nether_brick")
    } else {
        (3, "minecraft:stick")
    };
    RecipeProviderPlan::shaped(DataRecipeCategory::Decorations, result, count, result)
        .ingredient(base)
        .ingredient(stick)
        .pattern("W#W")
        .pattern("W#W")
}

fn provider_hanging_sign(result: &str, ingredient: &str) -> RecipeProviderPlan {
    RecipeProviderPlan::shaped(DataRecipeCategory::Decorations, result, 6, result)
        .ingredient(ingredient)
        .ingredient("minecraft:iron_chain")
        .pattern("X X")
        .pattern("###")
        .pattern("###")
        .group(Some("hanging_sign"))
        .unlock("has_stripped_logs")
}

fn provider_nine_block_storage_recipes(
    unpacked_form: &str,
    packed_form: &str,
    packing_recipe_id: &str,
    packing_recipe_group: Option<&str>,
    unpacking_recipe_id: &str,
    unpacking_recipe_group: Option<&str>,
) -> [RecipeProviderPlan; 2] {
    [
        RecipeProviderPlan::shapeless(
            DataRecipeCategory::Misc,
            unpacked_form,
            9,
            unpacking_recipe_id,
        )
        .ingredient(packed_form)
        .group(unpacking_recipe_group)
        .unlock(&provider_get_has_name(packed_form)),
        RecipeProviderPlan::shaped(DataRecipeCategory::Misc, packed_form, 1, packing_recipe_id)
            .ingredient(unpacked_form)
            .pattern("###")
            .pattern("###")
            .pattern("###")
            .group(packing_recipe_group)
            .unlock(&provider_get_has_name(unpacked_form)),
    ]
}

fn provider_copy_smithing_template(template: &str, base_material: &str) -> RecipeProviderPlan {
    RecipeProviderPlan::shaped(DataRecipeCategory::Misc, template, 2, template)
        .ingredient("minecraft:diamond")
        .ingredient(base_material)
        .ingredient(template)
        .pattern("#S#")
        .pattern("#C#")
        .pattern("###")
        .unlock(&provider_get_has_name(template))
}

fn provider_food_cooking(source: &str, cooking_kind: &'static str) -> Vec<RecipeProviderPlan> {
    [
        ("minecraft:beef", "minecraft:cooked_beef", "0.35"),
        ("minecraft:chicken", "minecraft:cooked_chicken", "0.35"),
        ("minecraft:cod", "minecraft:cooked_cod", "0.35"),
        ("minecraft:kelp", "minecraft:dried_kelp", "0.1"),
        ("minecraft:salmon", "minecraft:cooked_salmon", "0.35"),
        ("minecraft:mutton", "minecraft:cooked_mutton", "0.35"),
        ("minecraft:porkchop", "minecraft:cooked_porkchop", "0.35"),
        ("minecraft:potato", "minecraft:baked_potato", "0.35"),
        ("minecraft:rabbit", "minecraft:cooked_rabbit", "0.35"),
    ]
    .iter()
    .map(|(base, result, experience)| RecipeProviderPlan {
        kind: RecipeProviderPlanKind::Cooking(cooking_kind),
        category: DataRecipeCategory::Food,
        result: (*result).to_string(),
        count: 1,
        ingredients: vec![(*base).to_string(), format!("xp:{experience}")],
        patterns: Vec::new(),
        group: None,
        unlocks: vec![provider_get_has_name(base)],
        id: format!("{}_from_{source}", provider_get_item_name(result)),
    })
    .collect()
}

fn provider_family_shape_builder_variants() -> Vec<(&'static str, RecipeProviderPlanKind)> {
    vec![
        ("BUTTON", RecipeProviderPlanKind::Shapeless),
        ("CHISELED", RecipeProviderPlanKind::Shaped),
        ("CUT", RecipeProviderPlanKind::Shaped),
        ("DOOR", RecipeProviderPlanKind::Shaped),
        ("CUSTOM_FENCE", RecipeProviderPlanKind::Shaped),
        ("FENCE", RecipeProviderPlanKind::Shaped),
        ("CUSTOM_FENCE_GATE", RecipeProviderPlanKind::Shaped),
        ("FENCE_GATE", RecipeProviderPlanKind::Shaped),
        ("SIGN", RecipeProviderPlanKind::Shaped),
        ("SLAB", RecipeProviderPlanKind::Shaped),
        ("STAIRS", RecipeProviderPlanKind::Shaped),
        ("PRESSURE_PLATE", RecipeProviderPlanKind::Shaped),
        ("POLISHED", RecipeProviderPlanKind::Shaped),
        ("TRAPDOOR", RecipeProviderPlanKind::Shaped),
        ("WALL", RecipeProviderPlanKind::Shaped),
        ("BRICKS", RecipeProviderPlanKind::Shaped),
        ("TILES", RecipeProviderPlanKind::Shaped),
    ]
}

fn provider_family_stonecutter_variants() -> Vec<(&'static str, u32)> {
    vec![
        ("SLAB", 2),
        ("STAIRS", 1),
        ("BRICKS", 1),
        ("WALL", 1),
        ("CHISELED", 1),
        ("POLISHED", 1),
        ("CUT", 1),
        ("TILES", 1),
        ("COBBLED", 1),
    ]
}

fn create_crafting_common_info(show_notification: bool) -> CraftingCommonInfoModel {
    CraftingCommonInfoModel { show_notification }
}

fn create_crafting_book_info(
    category: DataRecipeCategory,
    group: Option<&str>,
) -> CraftingBookInfoModel {
    CraftingBookInfoModel {
        category: category.crafting_book_category(),
        group: group.unwrap_or("").to_string(),
    }
}

fn save_with_optional_override(
    default_id: &str,
    requested_id: Option<&str>,
) -> Result<String, String> {
    match requested_id {
        None => Ok(default_id.to_string()),
        Some(id) if id == default_id => Err(format!(
            "Recipe {id} should remove its 'save' argument as it is equal to default one"
        )),
        Some(id) => Ok(id.to_string()),
    }
}

fn default_recipe_id(result_item_id: &str) -> String {
    result_item_id.to_string()
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, expected: &[&str]) {
    for sentinel in expected {
        assert!(
            source.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_category_folder_names_match_java_enum_order() {
        let categories = [
            DataRecipeCategory::BuildingBlocks,
            DataRecipeCategory::Decorations,
            DataRecipeCategory::Redstone,
            DataRecipeCategory::Transportation,
            DataRecipeCategory::Tools,
            DataRecipeCategory::Combat,
            DataRecipeCategory::Food,
            DataRecipeCategory::Brewing,
            DataRecipeCategory::Misc,
        ];
        assert_eq!(
            categories
                .iter()
                .map(|category| category.folder_name())
                .collect::<Vec<_>>(),
            vec![
                "building_blocks",
                "decorations",
                "redstone",
                "transportation",
                "tools",
                "combat",
                "food",
                "brewing",
                "misc",
            ]
        );
    }

    #[test]
    fn recipe_builder_crafting_book_category_mapping_matches_java_switch() {
        assert_eq!(
            DataRecipeCategory::BuildingBlocks.crafting_book_category(),
            "building"
        );
        assert_eq!(
            DataRecipeCategory::Tools.crafting_book_category(),
            "equipment"
        );
        assert_eq!(
            DataRecipeCategory::Combat.crafting_book_category(),
            "equipment"
        );
        assert_eq!(
            DataRecipeCategory::Redstone.crafting_book_category(),
            "redstone"
        );
        for category in [
            DataRecipeCategory::Decorations,
            DataRecipeCategory::Transportation,
            DataRecipeCategory::Food,
            DataRecipeCategory::Brewing,
            DataRecipeCategory::Misc,
        ] {
            assert_eq!(category.crafting_book_category(), "misc");
        }
    }

    #[test]
    fn recipe_builder_common_info_book_info_and_default_id_match_java_helpers() {
        assert_eq!(
            create_crafting_common_info(true),
            CraftingCommonInfoModel {
                show_notification: true
            }
        );
        assert_eq!(
            create_crafting_book_info(DataRecipeCategory::Combat, Some("weapons")),
            CraftingBookInfoModel {
                category: "equipment",
                group: "weapons".to_string(),
            }
        );
        assert_eq!(
            create_crafting_book_info(DataRecipeCategory::Food, None).group,
            ""
        );
        assert_eq!(
            default_recipe_id("minecraft:diamond_pickaxe"),
            "minecraft:diamond_pickaxe"
        );
    }

    #[test]
    fn recipe_builder_save_overload_rejects_redundant_default_key() {
        assert_eq!(
            save_with_optional_override("minecraft:stick", None).unwrap(),
            "minecraft:stick"
        );
        assert_eq!(
            save_with_optional_override("minecraft:stick", Some("minecraft:stick")).unwrap_err(),
            "Recipe minecraft:stick should remove its 'save' argument as it is equal to default one"
        );
        assert_eq!(
            save_with_optional_override("minecraft:stick", Some("minecraft:bamboo_stick")).unwrap(),
            "minecraft:bamboo_stick"
        );
    }

    #[test]
    fn custom_crafting_builder_save_uses_common_info_book_info_factory_and_advancement() {
        let save = CustomCraftingRecipeBuilderModel::custom_crafting(
            DataRecipeCategory::Redstone,
            "SpecialMapCloningRecipe::new",
        )
        .unlocked_by("has_map", "inventory_changed")
        .group(Some("map_tools"))
        .save_by_name("map_cloning");
        assert_eq!(save.id, "minecraft:map_cloning");
        assert!(save.common_info.show_notification);
        assert_eq!(
            save.book_info,
            CraftingBookInfoModel {
                category: "redstone",
                group: "map_tools".to_string(),
            }
        );
        assert_eq!(save.factory_name, "SpecialMapCloningRecipe::new");
        assert_eq!(save.advancement_criteria["has_map"], "inventory_changed");
    }

    #[test]
    fn recipe_output_accepts_nullable_advancement_and_exposes_root_hook() {
        let mut output = RecipeOutputModel::new("Advancement.Builder");
        assert_eq!(output.advancement(), "Advancement.Builder");
        assert!(!output.root_advancement_included);

        output.accept(
            "minecraft:stone_pickaxe",
            "StonePickaxeRecipe",
            Some("recipes/tools/stone_pickaxe"),
        );
        output.accept("minecraft:debug_marker", "DebugMarkerRecipe", None);
        output.include_root_advancement();

        assert_eq!(
            output.accepted,
            vec![
                AcceptedRecipeModel {
                    id: "minecraft:stone_pickaxe".to_string(),
                    recipe: "StonePickaxeRecipe".to_string(),
                    advancement: Some("recipes/tools/stone_pickaxe".to_string()),
                },
                AcceptedRecipeModel {
                    id: "minecraft:debug_marker".to_string(),
                    recipe: "DebugMarkerRecipe".to_string(),
                    advancement: None,
                },
            ]
        );
        assert!(output.root_advancement_included);
    }

    #[test]
    fn recipe_provider_name_helpers_match_java_static_helpers() {
        assert_eq!(provider_get_item_name("minecraft:oak_planks"), "oak_planks");
        assert_eq!(provider_get_has_name("minecraft:oak_log"), "has_oak_log");
        assert_eq!(
            provider_get_conversion_recipe_name("minecraft:stick", "minecraft:bamboo"),
            "stick_from_bamboo"
        );
        assert_eq!(
            provider_get_smelting_recipe_name("minecraft:iron_ingot"),
            "iron_ingot_from_smelting"
        );
        assert_eq!(
            provider_get_blasting_recipe_name("minecraft:iron_ingot"),
            "iron_ingot_from_blasting"
        );
    }

    #[test]
    fn recipe_provider_conversion_and_cooking_helpers_match_java_save_ids() {
        let conversion = provider_one_to_one_conversion(
            "minecraft:orange_dye",
            "minecraft:orange_tulip",
            Some("orange_dye"),
            2,
        );
        assert_eq!(conversion.kind, RecipeProviderPlanKind::Shapeless);
        assert_eq!(conversion.category, DataRecipeCategory::Misc);
        assert_eq!(conversion.count, 2);
        assert_eq!(conversion.group.as_deref(), Some("orange_dye"));
        assert_eq!(conversion.ingredients, vec!["minecraft:orange_tulip"]);
        assert_eq!(conversion.unlocks, vec!["has_orange_tulip"]);
        assert_eq!(conversion.id, "orange_dye_from_orange_tulip");

        let cooking = provider_ore_cooking(
            "smelting",
            &["minecraft:raw_iron", "minecraft:iron_ore"],
            "minecraft:iron_ingot",
            "iron_ingot",
            "_from_smelting",
        );
        assert_eq!(cooking.len(), 2);
        assert_eq!(cooking[0].kind, RecipeProviderPlanKind::Cooking("smelting"));
        assert_eq!(cooking[0].id, "iron_ingot_from_smelting_raw_iron");
        assert_eq!(cooking[1].id, "iron_ingot_from_smelting_iron_ore");
        assert_eq!(cooking[1].unlocks, vec!["has_iron_ore"]);
    }

    #[test]
    fn recipe_provider_structural_recipe_helpers_match_java_patterns() {
        let two_by_two = provider_two_by_two_packer(
            DataRecipeCategory::BuildingBlocks,
            "minecraft:quartz_block",
            "minecraft:quartz",
        );
        assert_eq!(two_by_two.patterns, vec!["##", "##"]);
        assert_eq!(two_by_two.unlocks, vec!["has_quartz"]);

        let three_by_three = provider_three_by_three_packer(
            DataRecipeCategory::Misc,
            "minecraft:diamond_block",
            "minecraft:diamond",
            None,
        );
        assert_eq!(three_by_three.kind, RecipeProviderPlanKind::Shapeless);
        assert_eq!(three_by_three.ingredients.len(), 9);
        assert!(three_by_three
            .ingredients
            .iter()
            .all(|ingredient| ingredient == "minecraft:diamond"));
        assert_eq!(three_by_three.unlocks, vec!["has_diamond"]);

        let wood = provider_wood_from_logs("minecraft:oak_wood", "minecraft:oak_log");
        assert_eq!(wood.count, 3);
        assert_eq!(wood.patterns, vec!["##", "##"]);
        assert_eq!(wood.group.as_deref(), Some("bark"));

        let boat = provider_wooden_boat("minecraft:oak_boat", "minecraft:oak_planks");
        assert_eq!(boat.category, DataRecipeCategory::Transportation);
        assert_eq!(boat.patterns, vec!["# #", "###"]);
        assert_eq!(boat.group.as_deref(), Some("boat"));
        assert_eq!(boat.unlocks, vec!["in_water"]);

        let chest_boat = provider_chest_boat("minecraft:oak_chest_boat", "minecraft:oak_boat");
        assert_eq!(
            chest_boat.ingredients,
            vec!["minecraft:chest", "minecraft:oak_boat"]
        );
        assert_eq!(chest_boat.group.as_deref(), Some("chest_boat"));
    }

    #[test]
    fn recipe_provider_block_builder_edge_cases_match_java() {
        let wooden_fence = provider_fence_builder("minecraft:oak_fence", "minecraft:oak_planks");
        assert_eq!(wooden_fence.count, 3);
        assert_eq!(
            wooden_fence.ingredients,
            vec!["minecraft:oak_planks", "minecraft:stick"]
        );
        assert_eq!(wooden_fence.patterns, vec!["W#W", "W#W"]);

        let nether_fence =
            provider_fence_builder("minecraft:nether_brick_fence", "minecraft:nether_bricks");
        assert_eq!(nether_fence.count, 6);
        assert_eq!(
            nether_fence.ingredients,
            vec!["minecraft:nether_bricks", "minecraft:nether_brick"]
        );

        let hanging_sign =
            provider_hanging_sign("minecraft:oak_hanging_sign", "minecraft:stripped_oak_log");
        assert_eq!(hanging_sign.count, 6);
        assert_eq!(hanging_sign.patterns, vec!["X X", "###", "###"]);
        assert_eq!(hanging_sign.group.as_deref(), Some("hanging_sign"));
        assert_eq!(hanging_sign.unlocks, vec!["has_stripped_logs"]);
    }

    #[test]
    fn recipe_provider_storage_smithing_and_food_helpers_match_java() {
        let [unpack, pack] = provider_nine_block_storage_recipes(
            "minecraft:diamond",
            "minecraft:diamond_block",
            "minecraft:diamond_block",
            None,
            "minecraft:diamond",
            Some("diamond"),
        );
        assert_eq!(unpack.kind, RecipeProviderPlanKind::Shapeless);
        assert_eq!(unpack.count, 9);
        assert_eq!(unpack.group.as_deref(), Some("diamond"));
        assert_eq!(pack.kind, RecipeProviderPlanKind::Shaped);
        assert_eq!(pack.patterns, vec!["###", "###", "###"]);
        assert_eq!(pack.unlocks, vec!["has_diamond"]);

        let copy_template = provider_copy_smithing_template(
            "minecraft:coast_armor_trim_smithing_template",
            "minecraft:cobblestone",
        );
        assert_eq!(copy_template.count, 2);
        assert_eq!(copy_template.patterns, vec!["#S#", "#C#", "###"]);
        assert_eq!(
            copy_template.ingredients,
            vec![
                "minecraft:diamond",
                "minecraft:cobblestone",
                "minecraft:coast_armor_trim_smithing_template",
            ]
        );

        let food = provider_food_cooking("smoking", "smoking");
        assert_eq!(food.len(), 9);
        assert_eq!(food[0].id, "cooked_beef_from_smoking");
        assert_eq!(food[3].result, "minecraft:dried_kelp");
        assert_eq!(food[3].ingredients, vec!["minecraft:kelp", "xp:0.1"]);
        assert_eq!(food[8].unlocks, vec!["has_rabbit"]);
    }

    #[test]
    fn recipe_provider_family_dispatch_maps_match_java() {
        let shape_variants = provider_family_shape_builder_variants();
        assert_eq!(shape_variants.len(), 17);
        assert_eq!(
            shape_variants[0],
            ("BUTTON", RecipeProviderPlanKind::Shapeless)
        );
        assert_eq!(
            shape_variants[1],
            ("CHISELED", RecipeProviderPlanKind::Shaped)
        );
        assert_eq!(
            shape_variants[16],
            ("TILES", RecipeProviderPlanKind::Shaped)
        );

        let stonecutter_variants = provider_family_stonecutter_variants();
        assert_eq!(stonecutter_variants.len(), 9);
        assert_eq!(stonecutter_variants[0], ("SLAB", 2));
        assert_eq!(stonecutter_variants[8], ("COBBLED", 1));
    }

    #[test]
    fn recipe_provider_runner_output_matches_java_duplicate_and_root_behavior() {
        let mut runner = RecipeProviderRunnerModel::new();
        assert_eq!(runner.advancement_parent(), "recipes/root");
        runner
            .accept(
                "minecraft:oak_planks",
                "OakPlanksRecipe",
                Some("minecraft:recipes/building_blocks/oak_planks"),
            )
            .unwrap();
        runner
            .accept("minecraft:debug", "DebugRecipeWithoutAdvancement", None)
            .unwrap();
        assert_eq!(
            runner.accept("minecraft:oak_planks", "Duplicate", None),
            Err("Duplicate recipe minecraft:oak_planks".to_string())
        );
        runner.include_root_advancement();

        assert_eq!(
            runner.saved_recipes,
            vec![
                "recipe:minecraft:oak_planks:OakPlanksRecipe",
                "recipe:minecraft:debug:DebugRecipeWithoutAdvancement",
            ]
        );
        assert_eq!(
            runner.saved_advancements,
            vec![
                "advancement:minecraft:recipes/building_blocks/oak_planks",
                "advancement:recipes/root:impossible",
            ]
        );
    }

    #[test]
    fn recipe_provider_plan_kinds_cover_java_builder_families() {
        let kinds = [
            RecipeProviderPlanKind::SmithingTransform,
            RecipeProviderPlanKind::SmithingTrim,
            RecipeProviderPlanKind::Stonecutting,
            RecipeProviderPlanKind::Special("BannerDuplicateRecipe"),
            RecipeProviderPlanKind::Transmute,
        ];
        assert_eq!(kinds[0], RecipeProviderPlanKind::SmithingTransform);
        assert_eq!(kinds[1], RecipeProviderPlanKind::SmithingTrim);
        assert_eq!(kinds[2], RecipeProviderPlanKind::Stonecutting);
        assert_eq!(
            kinds[3],
            RecipeProviderPlanKind::Special("BannerDuplicateRecipe")
        );
        assert_eq!(kinds[4], RecipeProviderPlanKind::Transmute);
    }

    #[test]
    fn custom_crafting_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA, "save("),
            3
        );
        assert_eq!(
            count_occurrences(CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA, "unlockedBy"),
            2
        );
        assert_eq!(
            count_occurrences(CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA, "group("),
            1
        );
        assert_eq!(
            count_occurrences(
                CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA,
                "RecipeBuilder.createCraftingCommonInfo"
            ),
            1
        );
        assert_eq!(
            count_occurrences(
                CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA,
                "RecipeBuilder.createCraftingBookInfo"
            ),
            1
        );
    }

    #[test]
    fn recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(count_occurrences(RECIPE_BUILDER_JAVA, "save("), 5);
        assert_eq!(count_occurrences(RECIPE_BUILDER_JAVA, "defaultId"), 3);
        assert_eq!(
            count_occurrences(RECIPE_BUILDER_JAVA, "determineCraftingBookCategory"),
            2
        );
        assert_eq!(
            count_occurrences(RECIPE_BUILDER_JAVA, "createCraftingCommonInfo"),
            1
        );
        assert_eq!(
            count_occurrences(RECIPE_BUILDER_JAVA, "createCraftingBookInfo"),
            1
        );
        assert_eq!(
            count_occurrences(RECIPE_BUILDER_JAVA, "getDefaultRecipeId"),
            1
        );
    }

    #[test]
    fn recipe_output_source_counts_match_authoritative_java() {
        assert_eq!(count_occurrences(RECIPE_OUTPUT_JAVA, "accept("), 1);
        assert_eq!(count_occurrences(RECIPE_OUTPUT_JAVA, "advancement()"), 1);
        assert_eq!(
            count_occurrences(RECIPE_OUTPUT_JAVA, "includeRootAdvancement"),
            1
        );
        assert_eq!(count_occurrences(RECIPE_OUTPUT_JAVA, "@Nullable"), 1);
    }

    #[test]
    fn recipe_provider_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "protected void "),
            53
        );
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "protected RecipeBuilder "),
            4
        );
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "protected ShapedRecipeBuilder "),
            3
        );
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "protected ShapelessRecipeBuilder "),
            3
        );
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "private RecipeBuilder "),
            9
        );
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "private ShapedRecipeBuilder "),
            1
        );
        assert_eq!(count_occurrences(RECIPE_PROVIDER_JAVA, "SHAPE_BUILDERS"), 2);
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "STONECUTTER_RECIPE_BUILDERS"),
            2
        );
        assert_eq!(
            count_occurrences(RECIPE_PROVIDER_JAVA, "Duplicate recipe "),
            1
        );
    }

    #[test]
    fn recipe_datagen_java_source_sentinels_match_authoritative_files() {
        assert_source_contains_all(
            CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA,
            &[
                "private final RecipeCategory category;",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "private @Nullable String group;",
                "public static CustomCraftingRecipeBuilder customCrafting(final RecipeCategory category, final CustomCraftingRecipeBuilder.Factory factory)",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "this.save(output, ResourceKey.create(Registries.RECIPE, Identifier.parse(name)));",
                "Recipe.CommonInfo commonInfo = RecipeBuilder.createCraftingCommonInfo(true);",
                "CraftingRecipe.CraftingBookInfo bookInfo = RecipeBuilder.createCraftingBookInfo(this.category, this.group);",
                "Recipe<?> recipe = this.factory.apply(commonInfo, bookInfo);",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.category));",
                "public interface Factory extends BiFunction<Recipe.CommonInfo, CraftingRecipe.CraftingBookInfo, Recipe<?>>",
            ],
        );
        assert_source_contains_all(
            RECIPE_BUILDER_JAVA,
            &[
                "Identifier ROOT_RECIPE_ADVANCEMENT = Identifier.withDefaultNamespace(\"recipes/root\");",
                "if (overriddenKey == defaultKey)",
                "throw new IllegalStateException(\"Recipe \" + id + \" should remove its 'save' argument as it is equal to default one\");",
                "case BUILDING_BLOCKS -> CraftingBookCategory.BUILDING;",
                "case TOOLS, COMBAT -> CraftingBookCategory.EQUIPMENT;",
                "case REDSTONE -> CraftingBookCategory.REDSTONE;",
                "default -> CraftingBookCategory.MISC;",
                "return new Recipe.CommonInfo(showNotification);",
                "Objects.requireNonNullElse(group, \"\")",
                "result.typeHolder().unwrapKey().orElseThrow().identifier()",
            ],
        );
        assert_source_contains_all(
            RECIPE_CATEGORY_JAVA,
            &[
                "BUILDING_BLOCKS(\"building_blocks\")",
                "DECORATIONS(\"decorations\")",
                "REDSTONE(\"redstone\")",
                "TRANSPORTATION(\"transportation\")",
                "TOOLS(\"tools\")",
                "COMBAT(\"combat\")",
                "FOOD(\"food\")",
                "BREWING(\"brewing\")",
                "MISC(\"misc\")",
                "public String getFolderName()",
            ],
        );
        assert_source_contains_all(
            RECIPE_OUTPUT_JAVA,
            &[
                "void accept(ResourceKey<Recipe<?>> id, Recipe<?> recipe, @Nullable AdvancementHolder advancement);",
                "Advancement.Builder advancement();",
                "void includeRootAdvancement();",
            ],
        );
    }

    #[test]
    fn recipe_provider_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            RECIPE_PROVIDER_JAVA,
            &[
                "protected final HolderLookup.Provider registries;",
                "private final HolderGetter<Item> items;",
                "protected final RecipeOutput output;",
                "private static final Map<BlockFamily.Variant, RecipeProvider.FamilyCraftingRecipeProvider> SHAPE_BUILDERS",
                "private static final Map<BlockFamily.Variant, RecipeProvider.FamilyStonecutterRecipeProvider> STONECUTTER_RECIPE_BUILDERS",
                "protected abstract void buildRecipes();",
                "this.shapeless(RecipeCategory.MISC, product, productCount)",
                "SimpleCookingRecipeBuilder.generic(Ingredient.of(item), craftingCategory, cookingCategory, result, experience, cookingTime, factory)",
                "SmithingTransformRecipeBuilder.smithing(",
                "SmithingTrimRecipeBuilder.smithingTrim(",
                "this.shaped(category, result, 1)",
                "this.shapeless(category, result).requires(ingredient, 9)",
                "this.shaped(RecipeCategory.BUILDING_BLOCKS, result, 3)",
                "this.shaped(RecipeCategory.TRANSPORTATION, result)",
                "this.shapeless(RecipeCategory.TRANSPORTATION, chestBoat)",
                "int count = result == Blocks.NETHER_BRICK_FENCE ? 6 : 3;",
                "this.shaped(RecipeCategory.DECORATIONS, result, 6)",
                "this.colorWithDye(dyes, items, null, groupName, category);",
                "new BannerDuplicateRecipe(Ingredient.of(result), new ItemStackTemplate(result.asItem()))",
                "this.shapeless(RecipeCategory.BUILDING_BLOCKS, result, 8)",
                "this.stonecutterResultFromBase(category, result, base, 1);",
                "ResourceKey.create(Registries.RECIPE, Identifier.parse(unpackingRecipeId))",
                "this.simpleCookingRecipe(source, factory, cookingTime, Items.RABBIT, Items.COOKED_RABBIT, 0.35F);",
                "HoneycombItem.WAXABLES",
                "DataComponentPatch.builder().set(DataComponents.SUSPICIOUS_STEW_EFFECTS, effectHolder.getSuspiciousEffects()).build()",
                "CustomCraftingRecipeBuilder.customCrafting(",
                "TransmuteRecipeBuilder.transmute(RecipeCategory.DECORATIONS, this.tag(ItemTags.SHULKER_BOXES), Ingredient.of(dye), dyedResult)",
                "TransmuteRecipeBuilder.transmute(RecipeCategory.TOOLS, this.tag(ItemTags.BUNDLES), Ingredient.of(dye), dyedResult)",
                "if (variant == BlockFamily.Variant.CHISELED)",
                "throw new IllegalStateException(\"Slab is not defined for the family.\");",
                "CriteriaTriggers.ENTER_BLOCK",
                "CriteriaTriggers.BRED_ANIMALS",
                "CriteriaTriggers.INVENTORY_CHANGED",
                "return \"has_\" + getItemName(baseBlock);",
                "return getItemName(product) + \"_from_\" + getItemName(material);",
                "return getItemName(product) + \"_from_smelting\";",
                "return getItemName(product) + \"_from_blasting\";",
                "return Ingredient.of(this.items.getOrThrow(id));",
                "ShapedRecipeBuilder.shaped(this.items, category, item, count);",
                "ShapelessRecipeBuilder.shapeless(this.items, category, item, count);",
                "protected abstract static class Runner implements DataProvider",
                "final Set<ResourceKey<Recipe<?>>> allRecipes = Sets.newHashSet();",
                "throw new IllegalStateException(\"Duplicate recipe \" + id.identifier());",
                "Advancement.Builder.recipeAdvancement().parent(RecipeBuilder.ROOT_RECIPE_ADVANCEMENT);",
                "CriteriaTriggers.IMPOSSIBLE.createCriterion(new ImpossibleTrigger.TriggerInstance())",
                "DataProvider.saveStable(cache, registries, Recipe.CODEC, recipe, recipePathProvider.json(id.identifier()))",
                "DataProvider.saveStable(",
                "protected abstract RecipeProvider createRecipeProvider(HolderLookup.Provider registries, RecipeOutput output);",
            ],
        );
    }
}
