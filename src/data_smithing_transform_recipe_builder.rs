const SMITHING_TRANSFORM_RECIPE_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/recipes/SmithingTransformRecipeBuilder.java");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmithingTransformCategory {
    Combat,
    Tools,
}

impl SmithingTransformCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::Combat => "combat",
            Self::Tools => "tools",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmithingTransformRecipeBuilderModel {
    template: String,
    base: String,
    addition: String,
    category: SmithingTransformCategory,
    result: String,
    advancement_criteria: Vec<(String, String)>,
}

impl SmithingTransformRecipeBuilderModel {
    fn smithing(
        template: &str,
        base: &str,
        addition: &str,
        category: SmithingTransformCategory,
        result: &str,
    ) -> Self {
        Self {
            template: template.to_string(),
            base: base.to_string(),
            addition: addition.to_string(),
            category,
            result: result.to_string(),
            advancement_criteria: Vec::new(),
        }
    }

    fn unlocks(&mut self, name: &str, criterion: &str) -> &mut Self {
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

    fn save_by_name(&self, id: &str) -> Result<SmithingTransformSaveModel, String> {
        self.save_by_key(&format!("minecraft:{id}"))
    }

    fn save_by_key(&self, id: &str) -> Result<SmithingTransformSaveModel, String> {
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
        Ok(SmithingTransformSaveModel {
            id: id.to_string(),
            common_info: SmithingTransformCommonInfoModel {
                show_notification: true,
            },
            template: Some(self.template.clone()),
            base: self.base.clone(),
            addition: Some(self.addition.clone()),
            result: self.result.clone(),
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
struct SmithingTransformCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmithingTransformSaveModel {
    id: String,
    common_info: SmithingTransformCommonInfoModel,
    template: Option<String>,
    base: String,
    addition: Option<String>,
    result: String,
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
    fn smithing_transform_factory_stores_template_base_addition_category_and_result() {
        let builder = SmithingTransformRecipeBuilderModel::smithing(
            "minecraft:netherite_upgrade_smithing_template",
            "minecraft:diamond_pickaxe",
            "tag:minecraft:netherite_tool_materials",
            SmithingTransformCategory::Tools,
            "minecraft:netherite_pickaxe",
        );
        assert_eq!(
            builder.template,
            "minecraft:netherite_upgrade_smithing_template"
        );
        assert_eq!(builder.base, "minecraft:diamond_pickaxe");
        assert_eq!(builder.addition, "tag:minecraft:netherite_tool_materials");
        assert_eq!(builder.category, SmithingTransformCategory::Tools);
        assert_eq!(builder.result, "minecraft:netherite_pickaxe");
    }

    #[test]
    fn smithing_transform_save_by_string_parses_default_namespace_like_java() {
        let mut builder = SmithingTransformRecipeBuilderModel::smithing(
            "minecraft:netherite_upgrade_smithing_template",
            "minecraft:diamond_chestplate",
            "minecraft:netherite_ingot",
            SmithingTransformCategory::Combat,
            "minecraft:netherite_chestplate",
        );
        builder.unlocks("has_netherite_ingot", "inventory_changed:netherite_ingot");

        let save = builder
            .save_by_name("netherite_chestplate_smithing")
            .unwrap();
        assert_eq!(save.id, "minecraft:netherite_chestplate_smithing");
        assert_eq!(
            save.advancement_id,
            "recipes/combat/netherite_chestplate_smithing"
        );
    }

    #[test]
    fn smithing_transform_save_constructs_recipe_common_info_optionals_and_advancement() {
        let mut builder = SmithingTransformRecipeBuilderModel::smithing(
            "minecraft:netherite_upgrade_smithing_template",
            "minecraft:diamond_sword",
            "minecraft:netherite_ingot",
            SmithingTransformCategory::Combat,
            "minecraft:netherite_sword",
        );
        builder.unlocks("has_netherite_ingot", "inventory_changed:netherite_ingot");
        let save = builder
            .save_by_key("minecraft:netherite_sword_smithing")
            .unwrap();

        assert_eq!(
            save.common_info,
            SmithingTransformCommonInfoModel {
                show_notification: true
            }
        );
        assert_eq!(
            save.template,
            Some("minecraft:netherite_upgrade_smithing_template".to_string())
        );
        assert_eq!(save.base, "minecraft:diamond_sword");
        assert_eq!(save.addition, Some("minecraft:netherite_ingot".to_string()));
        assert_eq!(save.result, "minecraft:netherite_sword");
        assert_eq!(
            save.advancement_id,
            "recipes/combat/netherite_sword_smithing"
        );
        assert_eq!(
            save.advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:netherite_sword_smithing".to_string(),
                ),
                (
                    "has_netherite_ingot".to_string(),
                    "inventory_changed:netherite_ingot".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn smithing_transform_save_requires_unlock_criteria_via_advancement_builder() {
        let builder = SmithingTransformRecipeBuilderModel::smithing(
            "minecraft:netherite_upgrade_smithing_template",
            "minecraft:diamond_axe",
            "minecraft:netherite_ingot",
            SmithingTransformCategory::Tools,
            "minecraft:netherite_axe",
        );
        assert_eq!(
            builder
                .save_by_key("minecraft:netherite_axe_smithing")
                .unwrap_err(),
            "No way of obtaining recipe netherite_axe_smithing"
        );
    }

    #[test]
    fn smithing_transform_unlock_replacement_matches_advancement_builder_storage() {
        let mut builder = SmithingTransformRecipeBuilderModel::smithing(
            "minecraft:netherite_upgrade_smithing_template",
            "minecraft:diamond_hoe",
            "minecraft:netherite_ingot",
            SmithingTransformCategory::Tools,
            "minecraft:netherite_hoe",
        );
        builder
            .unlocks("has_template", "first")
            .unlocks("has_ingot", "second")
            .unlocks("has_template", "replacement");
        assert_eq!(
            builder
                .save_by_key("minecraft:netherite_hoe_smithing")
                .unwrap()
                .advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:netherite_hoe_smithing".to_string(),
                ),
                ("has_template".to_string(), "replacement".to_string()),
                ("has_ingot".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn smithing_transform_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(SMITHING_TRANSFORM_RECIPE_BUILDER_JAVA, "smithing("),
            1
        );
        assert_eq!(
            count_occurrences(SMITHING_TRANSFORM_RECIPE_BUILDER_JAVA, "unlocks("),
            1
        );
        assert_eq!(
            count_occurrences(SMITHING_TRANSFORM_RECIPE_BUILDER_JAVA, "save("),
            3
        );
        assert_eq!(
            count_occurrences(SMITHING_TRANSFORM_RECIPE_BUILDER_JAVA, "Optional.of"),
            2
        );
        assert_eq!(
            count_occurrences(
                SMITHING_TRANSFORM_RECIPE_BUILDER_JAVA,
                "Recipe.CommonInfo(true)"
            ),
            1
        );
    }

    #[test]
    fn smithing_transform_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            SMITHING_TRANSFORM_RECIPE_BUILDER_JAVA,
            &[
                "private final Ingredient template;",
                "private final Ingredient base;",
                "private final Ingredient addition;",
                "private final RecipeCategory category;",
                "private final ItemStackTemplate result;",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "return new SmithingTransformRecipeBuilder(template, base, addition, category, new ItemStackTemplate(result));",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "this.save(output, ResourceKey.create(Registries.RECIPE, Identifier.parse(id)));",
                "SmithingTransformRecipe recipe = new SmithingTransformRecipe(",
                "new Recipe.CommonInfo(true), Optional.of(this.template), this.base, Optional.of(this.addition), this.result",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.category));",
            ],
        );
    }
}
