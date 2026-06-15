const SIMPLE_COOKING_RECIPE_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/recipes/SimpleCookingRecipeBuilder.java");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimpleCookingRecipeCategory {
    BuildingBlocks,
    Food,
    Misc,
}

impl SimpleCookingRecipeCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::BuildingBlocks => "building_blocks",
            Self::Food => "food",
            Self::Misc => "misc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimpleCookingBookCategory {
    Blocks,
    Food,
    Misc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimpleCookingFactoryKind {
    Generic(&'static str),
    CampfireCooking,
    Blasting,
    Smelting,
    Smoking,
}

#[derive(Debug, Clone, PartialEq)]
struct SimpleCookingRecipeBuilderModel {
    crafting_category: SimpleCookingRecipeCategory,
    cooking_category: SimpleCookingBookCategory,
    result: String,
    ingredient: String,
    experience: f32,
    cooking_time: u32,
    advancement_criteria: Vec<(String, String)>,
    group: Option<String>,
    factory: SimpleCookingFactoryKind,
}

impl SimpleCookingRecipeBuilderModel {
    fn generic(
        ingredient: &str,
        crafting_category: SimpleCookingRecipeCategory,
        cooking_category: SimpleCookingBookCategory,
        result: &str,
        experience: f32,
        cooking_time: u32,
        factory_name: &'static str,
    ) -> Self {
        Self::new(
            crafting_category,
            cooking_category,
            result,
            ingredient,
            experience,
            cooking_time,
            SimpleCookingFactoryKind::Generic(factory_name),
        )
    }

    fn campfire_cooking(
        ingredient: &str,
        crafting_category: SimpleCookingRecipeCategory,
        result: &str,
        experience: f32,
        cooking_time: u32,
    ) -> Self {
        Self::new(
            crafting_category,
            SimpleCookingBookCategory::Food,
            result,
            ingredient,
            experience,
            cooking_time,
            SimpleCookingFactoryKind::CampfireCooking,
        )
    }

    fn blasting(
        ingredient: &str,
        crafting_category: SimpleCookingRecipeCategory,
        cooking_category: SimpleCookingBookCategory,
        result: &str,
        experience: f32,
        cooking_time: u32,
    ) -> Self {
        Self::new(
            crafting_category,
            cooking_category,
            result,
            ingredient,
            experience,
            cooking_time,
            SimpleCookingFactoryKind::Blasting,
        )
    }

    fn smelting(
        ingredient: &str,
        crafting_category: SimpleCookingRecipeCategory,
        cooking_category: SimpleCookingBookCategory,
        result: &str,
        experience: f32,
        cooking_time: u32,
    ) -> Self {
        Self::new(
            crafting_category,
            cooking_category,
            result,
            ingredient,
            experience,
            cooking_time,
            SimpleCookingFactoryKind::Smelting,
        )
    }

    fn smoking(
        ingredient: &str,
        crafting_category: SimpleCookingRecipeCategory,
        result: &str,
        experience: f32,
        cooking_time: u32,
    ) -> Self {
        Self::new(
            crafting_category,
            SimpleCookingBookCategory::Food,
            result,
            ingredient,
            experience,
            cooking_time,
            SimpleCookingFactoryKind::Smoking,
        )
    }

    fn new(
        crafting_category: SimpleCookingRecipeCategory,
        cooking_category: SimpleCookingBookCategory,
        result: &str,
        ingredient: &str,
        experience: f32,
        cooking_time: u32,
        factory: SimpleCookingFactoryKind,
    ) -> Self {
        Self {
            crafting_category,
            cooking_category,
            result: result.to_string(),
            ingredient: ingredient.to_string(),
            experience,
            cooking_time,
            advancement_criteria: Vec::new(),
            group: None,
            factory,
        }
    }

    fn unlocked_by(&mut self, name: &str, criterion: &str) -> &mut Self {
        if let Some((_, existing)) = self
            .advancement_criteria
            .iter_mut()
            .find(|(existing_name, _)| existing_name == name)
        {
            *existing = criterion.to_string();
        } else {
            self.advancement_criteria
                .push((name.to_string(), criterion.to_string()));
        }
        self
    }

    fn group(&mut self, group: Option<&str>) -> &mut Self {
        self.group = group.map(str::to_string);
        self
    }

    fn default_id(&self) -> String {
        self.result.clone()
    }

    fn save(&self, id: &str) -> Result<SimpleCookingRecipeSaveModel, String> {
        if self.advancement_criteria.is_empty() {
            return Err(format!(
                "No way of obtaining recipe {}",
                identifier_path(id)
            ));
        }
        let mut advancement_criteria = vec![(
            "has_the_recipe".to_string(),
            format!("recipe_unlocked:{id}"),
        )];
        advancement_criteria.extend(self.advancement_criteria.iter().cloned());
        Ok(SimpleCookingRecipeSaveModel {
            id: id.to_string(),
            factory: self.factory,
            common_info: SimpleCookingCommonInfoModel {
                show_notification: true,
            },
            book_info: SimpleCookingBookInfoModel {
                category: self.cooking_category,
                group: self.group.clone().unwrap_or_default(),
            },
            ingredient: self.ingredient.clone(),
            result: self.result.clone(),
            experience: self.experience,
            cooking_time: self.cooking_time,
            advancement_id: format!(
                "recipes/{}/{}",
                self.crafting_category.folder_name(),
                identifier_path(id)
            ),
            advancement_criteria,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleCookingCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleCookingBookInfoModel {
    category: SimpleCookingBookCategory,
    group: String,
}

#[derive(Debug, Clone, PartialEq)]
struct SimpleCookingRecipeSaveModel {
    id: String,
    factory: SimpleCookingFactoryKind,
    common_info: SimpleCookingCommonInfoModel,
    book_info: SimpleCookingBookInfoModel,
    ingredient: String,
    result: String,
    experience: f32,
    cooking_time: u32,
    advancement_id: String,
    advancement_criteria: Vec<(String, String)>,
}

fn identifier_path(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, path)| path)
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
    fn simple_cooking_factories_select_java_factory_and_book_categories() {
        let generic = SimpleCookingRecipeBuilderModel::generic(
            "minecraft:raw_iron",
            SimpleCookingRecipeCategory::Misc,
            SimpleCookingBookCategory::Misc,
            "minecraft:iron_ingot",
            0.7,
            200,
            "CustomCookingRecipe::new",
        );
        assert_eq!(
            generic.factory,
            SimpleCookingFactoryKind::Generic("CustomCookingRecipe::new")
        );
        assert_eq!(generic.cooking_category, SimpleCookingBookCategory::Misc);

        let campfire = SimpleCookingRecipeBuilderModel::campfire_cooking(
            "minecraft:beef",
            SimpleCookingRecipeCategory::Food,
            "minecraft:cooked_beef",
            0.35,
            600,
        );
        assert_eq!(campfire.factory, SimpleCookingFactoryKind::CampfireCooking);
        assert_eq!(campfire.cooking_category, SimpleCookingBookCategory::Food);

        let blasting = SimpleCookingRecipeBuilderModel::blasting(
            "minecraft:iron_ore",
            SimpleCookingRecipeCategory::Misc,
            SimpleCookingBookCategory::Blocks,
            "minecraft:iron_ingot",
            0.7,
            100,
        );
        assert_eq!(blasting.factory, SimpleCookingFactoryKind::Blasting);
        assert_eq!(blasting.cooking_category, SimpleCookingBookCategory::Blocks);

        let smelting = SimpleCookingRecipeBuilderModel::smelting(
            "minecraft:cobblestone",
            SimpleCookingRecipeCategory::BuildingBlocks,
            SimpleCookingBookCategory::Blocks,
            "minecraft:stone",
            0.1,
            200,
        );
        assert_eq!(smelting.factory, SimpleCookingFactoryKind::Smelting);

        let smoking = SimpleCookingRecipeBuilderModel::smoking(
            "minecraft:porkchop",
            SimpleCookingRecipeCategory::Food,
            "minecraft:cooked_porkchop",
            0.35,
            100,
        );
        assert_eq!(smoking.factory, SimpleCookingFactoryKind::Smoking);
        assert_eq!(smoking.cooking_category, SimpleCookingBookCategory::Food);
    }

    #[test]
    fn simple_cooking_default_id_group_and_save_payload_match_java() {
        let mut builder = SimpleCookingRecipeBuilderModel::smelting(
            "minecraft:raw_copper",
            SimpleCookingRecipeCategory::Misc,
            SimpleCookingBookCategory::Misc,
            "minecraft:copper_ingot",
            0.7,
            200,
        );
        assert_eq!(builder.default_id(), "minecraft:copper_ingot");
        builder
            .group(Some("copper_ingot"))
            .unlocked_by("has_raw_copper", "inventory_changed:raw_copper");

        let save = builder
            .save("minecraft:copper_ingot_from_smelting_raw_copper")
            .unwrap();
        assert_eq!(save.id, "minecraft:copper_ingot_from_smelting_raw_copper");
        assert_eq!(save.factory, SimpleCookingFactoryKind::Smelting);
        assert_eq!(
            save.common_info,
            SimpleCookingCommonInfoModel {
                show_notification: true
            }
        );
        assert_eq!(
            save.book_info,
            SimpleCookingBookInfoModel {
                category: SimpleCookingBookCategory::Misc,
                group: "copper_ingot".to_string(),
            }
        );
        assert_eq!(save.ingredient, "minecraft:raw_copper");
        assert_eq!(save.result, "minecraft:copper_ingot");
        assert_eq!(save.experience, 0.7);
        assert_eq!(save.cooking_time, 200);
        assert_eq!(
            save.advancement_id,
            "recipes/misc/copper_ingot_from_smelting_raw_copper"
        );
        assert_eq!(
            save.advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:copper_ingot_from_smelting_raw_copper".to_string(),
                ),
                (
                    "has_raw_copper".to_string(),
                    "inventory_changed:raw_copper".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn simple_cooking_nullable_group_becomes_empty_cooking_book_group() {
        let mut builder = SimpleCookingRecipeBuilderModel::blasting(
            "minecraft:gold_ore",
            SimpleCookingRecipeCategory::Misc,
            SimpleCookingBookCategory::Blocks,
            "minecraft:gold_ingot",
            1.0,
            100,
        );
        builder
            .group(None)
            .unlocked_by("has_gold_ore", "inventory_changed:gold_ore");
        assert_eq!(
            builder
                .save("minecraft:gold_ingot_from_blasting_gold_ore")
                .unwrap()
                .book_info,
            SimpleCookingBookInfoModel {
                category: SimpleCookingBookCategory::Blocks,
                group: String::new(),
            }
        );
    }

    #[test]
    fn simple_cooking_save_requires_unlock_criteria_via_advancement_builder() {
        let builder = SimpleCookingRecipeBuilderModel::campfire_cooking(
            "minecraft:cod",
            SimpleCookingRecipeCategory::Food,
            "minecraft:cooked_cod",
            0.35,
            600,
        );
        assert_eq!(
            builder
                .save("minecraft:cooked_cod_from_campfire_cooking")
                .unwrap_err(),
            "No way of obtaining recipe cooked_cod_from_campfire_cooking"
        );
    }

    #[test]
    fn simple_cooking_unlock_replacement_matches_advancement_builder_storage() {
        let mut builder = SimpleCookingRecipeBuilderModel::smoking(
            "minecraft:kelp",
            SimpleCookingRecipeCategory::Food,
            "minecraft:dried_kelp",
            0.1,
            100,
        );
        builder
            .unlocked_by("has_kelp", "first")
            .unlocked_by("has_fuel", "second")
            .unlocked_by("has_kelp", "replacement");
        assert_eq!(
            builder
                .save("minecraft:dried_kelp_from_smoking")
                .unwrap()
                .advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:dried_kelp_from_smoking".to_string(),
                ),
                ("has_kelp".to_string(), "replacement".to_string()),
                ("has_fuel".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn simple_cooking_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "generic("),
            1
        );
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "campfireCooking"),
            1
        );
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "blasting("),
            1
        );
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "smelting("),
            1
        );
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "smoking("),
            1
        );
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "unlockedBy"),
            2
        );
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "group("),
            1
        );
        assert_eq!(
            count_occurrences(SIMPLE_COOKING_RECIPE_BUILDER_JAVA, "save("),
            1
        );
    }

    #[test]
    fn simple_cooking_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            SIMPLE_COOKING_RECIPE_BUILDER_JAVA,
            &[
                "private final RecipeCategory craftingCategory;",
                "private final CookingBookCategory cookingCategory;",
                "private final ItemStackTemplate result;",
                "private final Ingredient ingredient;",
                "private final float experience;",
                "private final int cookingTime;",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "private @Nullable String group;",
                "private final AbstractCookingRecipe.Factory<?> factory;",
                "new ItemStackTemplate(result.asItem())",
                "public static <T extends AbstractCookingRecipe> SimpleCookingRecipeBuilder generic(",
                "return new SimpleCookingRecipeBuilder(craftingCategory, cookingCategory, result, ingredient, experience, cookingTime, factory);",
                "CookingBookCategory.FOOD, result, ingredient, experience, cookingTime, CampfireCookingRecipe::new",
                "result, ingredient, experience, cookingTime, BlastingRecipe::new",
                "result, ingredient, experience, cookingTime, SmeltingRecipe::new",
                "CookingBookCategory.FOOD, result, ingredient, experience, cookingTime, SmokingRecipe::new",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "this.group = group;",
                "return RecipeBuilder.getDefaultRecipeId(this.result);",
                "this.factory",
                "RecipeBuilder.createCraftingCommonInfo(true)",
                "new AbstractCookingRecipe.CookingBookInfo(this.cookingCategory, Objects.requireNonNullElse(this.group, \"\"))",
                "this.ingredient,",
                "this.result,",
                "this.experience,",
                "this.cookingTime",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.craftingCategory));",
            ],
        );
    }
}
