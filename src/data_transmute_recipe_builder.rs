const TRANSMUTE_RECIPE_BUILDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/recipes/TransmuteRecipeBuilder.java"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransmuteRecipeCategory {
    Decorations,
    Tools,
}

impl TransmuteRecipeCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::Decorations => "decorations",
            Self::Tools => "tools",
        }
    }

    fn crafting_book_category(self) -> &'static str {
        match self {
            Self::Tools => "equipment",
            Self::Decorations => "misc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MaterialCountBounds {
    min: u32,
    max: Option<u32>,
}

impl MaterialCountBounds {
    fn default_material_count() -> Self {
        Self {
            min: 1,
            max: Some(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransmuteRecipeBuilderModel {
    category: TransmuteRecipeCategory,
    result: String,
    input: String,
    material: String,
    advancement_criteria: Vec<(String, String)>,
    group: Option<String>,
    material_count: MaterialCountBounds,
    add_material_count_to_output: bool,
}

impl TransmuteRecipeBuilderModel {
    fn transmute_item(
        category: TransmuteRecipeCategory,
        input: &str,
        material: &str,
        result: &str,
    ) -> Self {
        Self::transmute_stack(category, input, material, result)
    }

    fn transmute_stack(
        category: TransmuteRecipeCategory,
        input: &str,
        material: &str,
        result: &str,
    ) -> Self {
        Self {
            category,
            result: result.to_string(),
            input: input.to_string(),
            material: material.to_string(),
            advancement_criteria: Vec::new(),
            group: None,
            material_count: MaterialCountBounds::default_material_count(),
            add_material_count_to_output: false,
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

    fn add_material_count_to_output(&mut self) -> &mut Self {
        self.add_material_count_to_output = true;
        self
    }

    fn set_material_count(&mut self, min: u32, max: Option<u32>) -> &mut Self {
        self.material_count = MaterialCountBounds { min, max };
        self
    }

    fn default_id(&self) -> String {
        self.result.clone()
    }

    fn save(&self, id: &str) -> Result<TransmuteRecipeSaveModel, String> {
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
        Ok(TransmuteRecipeSaveModel {
            id: id.to_string(),
            common_info: TransmuteCommonInfoModel {
                show_notification: true,
            },
            book_info: TransmuteBookInfoModel {
                category: self.category.crafting_book_category().to_string(),
                group: self.group.clone().unwrap_or_default(),
            },
            input: self.input.clone(),
            material: self.material.clone(),
            material_count: self.material_count.clone(),
            result: self.result.clone(),
            add_material_count_to_output: self.add_material_count_to_output,
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
struct TransmuteCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransmuteBookInfoModel {
    category: String,
    group: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransmuteRecipeSaveModel {
    id: String,
    common_info: TransmuteCommonInfoModel,
    book_info: TransmuteBookInfoModel,
    input: String,
    material: String,
    material_count: MaterialCountBounds,
    result: String,
    add_material_count_to_output: bool,
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
    fn transmute_factories_store_category_input_material_result_and_defaults() {
        let item = TransmuteRecipeBuilderModel::transmute_item(
            TransmuteRecipeCategory::Tools,
            "tag:minecraft:bundles",
            "minecraft:blue_dye",
            "minecraft:blue_bundle",
        );
        assert_eq!(item.category, TransmuteRecipeCategory::Tools);
        assert_eq!(item.input, "tag:minecraft:bundles");
        assert_eq!(item.material, "minecraft:blue_dye");
        assert_eq!(item.result, "minecraft:blue_bundle");
        assert_eq!(item.default_id(), "minecraft:blue_bundle");
        assert_eq!(
            item.material_count,
            MaterialCountBounds::default_material_count()
        );
        assert!(!item.add_material_count_to_output);

        let stack = TransmuteRecipeBuilderModel::transmute_stack(
            TransmuteRecipeCategory::Decorations,
            "tag:minecraft:shulker_boxes",
            "minecraft:red_dye",
            "minecraft:red_shulker_box",
        );
        assert_eq!(stack.result, "minecraft:red_shulker_box");
    }

    #[test]
    fn transmute_mutators_set_group_material_count_and_output_count_flag() {
        let mut builder = TransmuteRecipeBuilderModel::transmute_item(
            TransmuteRecipeCategory::Tools,
            "tag:minecraft:bundles",
            "minecraft:green_dye",
            "minecraft:green_bundle",
        );
        builder
            .group(Some("bundle_dye"))
            .set_material_count(2, Some(4))
            .add_material_count_to_output()
            .unlocked_by("has_green_dye", "inventory_changed:green_dye");
        let save = builder.save("minecraft:green_bundle").unwrap();
        assert_eq!(
            save.book_info,
            TransmuteBookInfoModel {
                category: "equipment".to_string(),
                group: "bundle_dye".to_string(),
            }
        );
        assert_eq!(
            save.material_count,
            MaterialCountBounds {
                min: 2,
                max: Some(4),
            }
        );
        assert!(save.add_material_count_to_output);
    }

    #[test]
    fn transmute_save_constructs_recipe_and_advancement_like_java() {
        let mut builder = TransmuteRecipeBuilderModel::transmute_item(
            TransmuteRecipeCategory::Decorations,
            "tag:minecraft:shulker_boxes",
            "minecraft:purple_dye",
            "minecraft:purple_shulker_box",
        );
        builder
            .group(Some("shulker_box_dye"))
            .unlocked_by("has_shulker_box", "inventory_changed:shulker_box");
        let save = builder.save("minecraft:purple_shulker_box").unwrap();
        assert_eq!(
            save.common_info,
            TransmuteCommonInfoModel {
                show_notification: true
            }
        );
        assert_eq!(
            save.book_info,
            TransmuteBookInfoModel {
                category: "misc".to_string(),
                group: "shulker_box_dye".to_string(),
            }
        );
        assert_eq!(save.input, "tag:minecraft:shulker_boxes");
        assert_eq!(save.material, "minecraft:purple_dye");
        assert_eq!(
            save.material_count,
            MaterialCountBounds::default_material_count()
        );
        assert_eq!(save.result, "minecraft:purple_shulker_box");
        assert!(!save.add_material_count_to_output);
        assert_eq!(
            save.advancement_id,
            "recipes/decorations/purple_shulker_box"
        );
        assert_eq!(
            save.advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:purple_shulker_box".to_string(),
                ),
                (
                    "has_shulker_box".to_string(),
                    "inventory_changed:shulker_box".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn transmute_save_requires_unlock_criteria_via_advancement_builder() {
        let builder = TransmuteRecipeBuilderModel::transmute_item(
            TransmuteRecipeCategory::Tools,
            "tag:minecraft:bundles",
            "minecraft:black_dye",
            "minecraft:black_bundle",
        );
        assert_eq!(
            builder.save("minecraft:black_bundle").unwrap_err(),
            "No way of obtaining recipe black_bundle"
        );
    }

    #[test]
    fn transmute_unlock_replacement_matches_advancement_builder_storage() {
        let mut builder = TransmuteRecipeBuilderModel::transmute_item(
            TransmuteRecipeCategory::Tools,
            "tag:minecraft:bundles",
            "minecraft:yellow_dye",
            "minecraft:yellow_bundle",
        );
        builder
            .unlocked_by("has_bundle", "first")
            .unlocked_by("has_dye", "second")
            .unlocked_by("has_bundle", "replacement");
        assert_eq!(
            builder
                .save("minecraft:yellow_bundle")
                .unwrap()
                .advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:yellow_bundle".to_string(),
                ),
                ("has_bundle".to_string(), "replacement".to_string()),
                ("has_dye".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn transmute_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(TRANSMUTE_RECIPE_BUILDER_JAVA, "transmute("),
            3
        );
        assert_eq!(
            count_occurrences(TRANSMUTE_RECIPE_BUILDER_JAVA, "unlockedBy"),
            2
        );
        assert_eq!(
            count_occurrences(TRANSMUTE_RECIPE_BUILDER_JAVA, "group("),
            1
        );
        assert_eq!(
            count_occurrences(TRANSMUTE_RECIPE_BUILDER_JAVA, "addMaterialCountToOutput"),
            4
        );
        assert_eq!(
            count_occurrences(TRANSMUTE_RECIPE_BUILDER_JAVA, "setMaterialCount"),
            1
        );
        assert_eq!(
            count_occurrences(TRANSMUTE_RECIPE_BUILDER_JAVA, "defaultId"),
            1
        );
        assert_eq!(count_occurrences(TRANSMUTE_RECIPE_BUILDER_JAVA, "save("), 1);
    }

    #[test]
    fn transmute_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            TRANSMUTE_RECIPE_BUILDER_JAVA,
            &[
                "private final RecipeCategory category;",
                "private final ItemStackTemplate result;",
                "private final Ingredient input;",
                "private final Ingredient material;",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "private @Nullable String group;",
                "private MinMaxBounds.Ints materialCount = TransmuteRecipe.DEFAULT_MATERIAL_COUNT;",
                "private boolean addMaterialCountToOutput;",
                "return transmute(category, input, material, new ItemStackTemplate(result));",
                "return new TransmuteRecipeBuilder(category, result, input, material);",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "this.group = group;",
                "this.addMaterialCountToOutput = true;",
                "this.materialCount = materialCount;",
                "return RecipeBuilder.getDefaultRecipeId(this.result);",
                "RecipeBuilder.createCraftingCommonInfo(true)",
                "RecipeBuilder.createCraftingBookInfo(this.category, this.group)",
                "this.input,",
                "this.material,",
                "this.materialCount,",
                "this.result,",
                "this.addMaterialCountToOutput",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.category));",
            ],
        );
    }
}
