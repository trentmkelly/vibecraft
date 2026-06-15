const SINGLE_ITEM_RECIPE_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/recipes/SingleItemRecipeBuilder.java");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SingleItemRecipeCategory {
    BuildingBlocks,
    Decorations,
}

impl SingleItemRecipeCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::BuildingBlocks => "building_blocks",
            Self::Decorations => "decorations",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SingleItemFactoryKind {
    Stonecutter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SingleItemRecipeBuilderModel {
    category: SingleItemRecipeCategory,
    result: String,
    count: u32,
    ingredient: String,
    advancement_criteria: Vec<(String, String)>,
    factory: SingleItemFactoryKind,
}

impl SingleItemRecipeBuilderModel {
    fn stonecutting(
        ingredient: &str,
        category: SingleItemRecipeCategory,
        result: &str,
        count: u32,
    ) -> Self {
        Self {
            category,
            result: result.to_string(),
            count,
            ingredient: ingredient.to_string(),
            advancement_criteria: Vec::new(),
            factory: SingleItemFactoryKind::Stonecutter,
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

    fn group(&mut self, _group: Option<&str>) -> &mut Self {
        self
    }

    fn default_id(&self) -> String {
        self.result.clone()
    }

    fn save(&self, id: &str) -> Result<SingleItemRecipeSaveModel, String> {
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
        Ok(SingleItemRecipeSaveModel {
            id: id.to_string(),
            factory: self.factory,
            common_info: SingleItemCommonInfoModel {
                show_notification: true,
            },
            ingredient: self.ingredient.clone(),
            result: self.result.clone(),
            count: self.count,
            advancement_id: format!(
                "recipes/{}/{}",
                self.category.folder_name(),
                identifier_path(id)
            ),
            advancement_criteria,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SingleItemCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SingleItemRecipeSaveModel {
    id: String,
    factory: SingleItemFactoryKind,
    common_info: SingleItemCommonInfoModel,
    ingredient: String,
    result: String,
    count: u32,
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
    fn single_item_stonecutting_factory_result_count_and_default_id_match_java() {
        let builder = SingleItemRecipeBuilderModel::stonecutting(
            "minecraft:stone",
            SingleItemRecipeCategory::BuildingBlocks,
            "minecraft:stone_slab",
            2,
        );
        assert_eq!(builder.factory, SingleItemFactoryKind::Stonecutter);
        assert_eq!(builder.ingredient, "minecraft:stone");
        assert_eq!(builder.result, "minecraft:stone_slab");
        assert_eq!(builder.count, 2);
        assert_eq!(builder.default_id(), "minecraft:stone_slab");
    }

    #[test]
    fn single_item_group_is_noop_like_java() {
        let mut builder = SingleItemRecipeBuilderModel::stonecutting(
            "minecraft:stone",
            SingleItemRecipeCategory::BuildingBlocks,
            "minecraft:stone_brick_wall",
            1,
        );
        builder
            .group(Some("stonecutting"))
            .group(None)
            .unlocked_by("has_stone", "inventory_changed:stone");
        let save = builder
            .save("minecraft:stone_brick_wall_from_stone_stonecutting")
            .unwrap();
        assert_eq!(save.ingredient, "minecraft:stone");
        assert_eq!(save.result, "minecraft:stone_brick_wall");
    }

    #[test]
    fn single_item_save_constructs_common_info_recipe_and_advancement_like_java() {
        let mut builder = SingleItemRecipeBuilderModel::stonecutting(
            "minecraft:deepslate",
            SingleItemRecipeCategory::Decorations,
            "minecraft:cobbled_deepslate_wall",
            1,
        );
        builder.unlocked_by("has_deepslate", "inventory_changed:deepslate");
        let save = builder
            .save("minecraft:cobbled_deepslate_wall_from_deepslate_stonecutting")
            .unwrap();

        assert_eq!(
            save.common_info,
            SingleItemCommonInfoModel {
                show_notification: true
            }
        );
        assert_eq!(save.factory, SingleItemFactoryKind::Stonecutter);
        assert_eq!(save.ingredient, "minecraft:deepslate");
        assert_eq!(save.result, "minecraft:cobbled_deepslate_wall");
        assert_eq!(save.count, 1);
        assert_eq!(
            save.advancement_id,
            "recipes/decorations/cobbled_deepslate_wall_from_deepslate_stonecutting"
        );
        assert_eq!(
            save.advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:cobbled_deepslate_wall_from_deepslate_stonecutting"
                        .to_string(),
                ),
                (
                    "has_deepslate".to_string(),
                    "inventory_changed:deepslate".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn single_item_save_requires_unlock_criteria_via_advancement_builder() {
        let builder = SingleItemRecipeBuilderModel::stonecutting(
            "minecraft:sandstone",
            SingleItemRecipeCategory::BuildingBlocks,
            "minecraft:chiseled_sandstone",
            1,
        );
        assert_eq!(
            builder
                .save("minecraft:chiseled_sandstone_from_sandstone_stonecutting")
                .unwrap_err(),
            "No way of obtaining recipe chiseled_sandstone_from_sandstone_stonecutting"
        );
    }

    #[test]
    fn single_item_unlock_replacement_matches_advancement_builder_storage() {
        let mut builder = SingleItemRecipeBuilderModel::stonecutting(
            "minecraft:granite",
            SingleItemRecipeCategory::BuildingBlocks,
            "minecraft:polished_granite",
            1,
        );
        builder
            .unlocked_by("has_granite", "first")
            .unlocked_by("has_stonecutter", "second")
            .unlocked_by("has_granite", "replacement");
        assert_eq!(
            builder
                .save("minecraft:polished_granite_from_granite_stonecutting")
                .unwrap()
                .advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:polished_granite_from_granite_stonecutting"
                        .to_string(),
                ),
                ("has_granite".to_string(), "replacement".to_string()),
                ("has_stonecutter".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn single_item_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(SINGLE_ITEM_RECIPE_BUILDER_JAVA, "stonecutting("),
            1
        );
        assert_eq!(
            count_occurrences(SINGLE_ITEM_RECIPE_BUILDER_JAVA, "unlockedBy"),
            2
        );
        assert_eq!(
            count_occurrences(SINGLE_ITEM_RECIPE_BUILDER_JAVA, "group("),
            1
        );
        assert_eq!(
            count_occurrences(SINGLE_ITEM_RECIPE_BUILDER_JAVA, "defaultId"),
            1
        );
        assert_eq!(
            count_occurrences(SINGLE_ITEM_RECIPE_BUILDER_JAVA, "save("),
            1
        );
        assert_eq!(
            count_occurrences(SINGLE_ITEM_RECIPE_BUILDER_JAVA, "Recipe.CommonInfo(true)"),
            1
        );
    }

    #[test]
    fn single_item_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            SINGLE_ITEM_RECIPE_BUILDER_JAVA,
            &[
                "private final RecipeCategory category;",
                "private final ItemStackTemplate result;",
                "private final Ingredient ingredient;",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "private final SingleItemRecipe.Factory<?> factory;",
                "new ItemStackTemplate(result.asItem(), count)",
                "public static SingleItemRecipeBuilder stonecutting(final Ingredient ingredient, final RecipeCategory category, final ItemLike result, final int count)",
                "return new SingleItemRecipeBuilder(category, StonecutterRecipe::new, ingredient, result, count);",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "public SingleItemRecipeBuilder group(final @Nullable String group)",
                "return this;",
                "return RecipeBuilder.getDefaultRecipeId(this.result);",
                "SingleItemRecipe recipe = this.factory.create(new Recipe.CommonInfo(true), this.ingredient, this.result);",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.category));",
            ],
        );
    }
}
