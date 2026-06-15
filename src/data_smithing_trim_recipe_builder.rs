const SMITHING_TRIM_RECIPE_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/recipes/SmithingTrimRecipeBuilder.java");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmithingTrimCategory {
    Misc,
}

impl SmithingTrimCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::Misc => "misc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmithingTrimRecipeBuilderModel {
    category: SmithingTrimCategory,
    template: String,
    base: String,
    addition: String,
    pattern: String,
    advancement_criteria: Vec<(String, String)>,
}

impl SmithingTrimRecipeBuilderModel {
    fn smithing_trim(
        template: &str,
        base: &str,
        addition: &str,
        pattern: &str,
        category: SmithingTrimCategory,
    ) -> Self {
        Self {
            category,
            template: template.to_string(),
            base: base.to_string(),
            addition: addition.to_string(),
            pattern: pattern.to_string(),
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

    fn save(&self, id: &str) -> Result<SmithingTrimSaveModel, String> {
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
        Ok(SmithingTrimSaveModel {
            id: id.to_string(),
            common_info: SmithingTrimCommonInfoModel {
                show_notification: true,
            },
            template: self.template.clone(),
            base: self.base.clone(),
            addition: self.addition.clone(),
            pattern: self.pattern.clone(),
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
struct SmithingTrimCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmithingTrimSaveModel {
    id: String,
    common_info: SmithingTrimCommonInfoModel,
    template: String,
    base: String,
    addition: String,
    pattern: String,
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
    fn smithing_trim_factory_stores_category_template_base_addition_and_pattern() {
        let builder = SmithingTrimRecipeBuilderModel::smithing_trim(
            "minecraft:coast_armor_trim_smithing_template",
            "tag:minecraft:trimmable_armor",
            "tag:minecraft:trim_materials",
            "minecraft:coast",
            SmithingTrimCategory::Misc,
        );
        assert_eq!(builder.category, SmithingTrimCategory::Misc);
        assert_eq!(
            builder.template,
            "minecraft:coast_armor_trim_smithing_template"
        );
        assert_eq!(builder.base, "tag:minecraft:trimmable_armor");
        assert_eq!(builder.addition, "tag:minecraft:trim_materials");
        assert_eq!(builder.pattern, "minecraft:coast");
    }

    #[test]
    fn smithing_trim_save_constructs_common_info_pattern_holder_recipe_and_advancement() {
        let mut builder = SmithingTrimRecipeBuilderModel::smithing_trim(
            "minecraft:spire_armor_trim_smithing_template",
            "tag:minecraft:trimmable_armor",
            "tag:minecraft:trim_materials",
            "minecraft:spire",
            SmithingTrimCategory::Misc,
        );
        builder.unlocks(
            "has_smithing_trim_template",
            "inventory_changed:spire_armor_trim_smithing_template",
        );

        let save = builder
            .save("minecraft:spire_armor_trim_smithing_template_smithing_trim")
            .unwrap();
        assert_eq!(
            save.id,
            "minecraft:spire_armor_trim_smithing_template_smithing_trim"
        );
        assert_eq!(
            save.common_info,
            SmithingTrimCommonInfoModel {
                show_notification: true
            }
        );
        assert_eq!(
            save.template,
            "minecraft:spire_armor_trim_smithing_template"
        );
        assert_eq!(save.base, "tag:minecraft:trimmable_armor");
        assert_eq!(save.addition, "tag:minecraft:trim_materials");
        assert_eq!(save.pattern, "minecraft:spire");
        assert_eq!(
            save.advancement_id,
            "recipes/misc/spire_armor_trim_smithing_template_smithing_trim"
        );
        assert_eq!(
            save.advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:spire_armor_trim_smithing_template_smithing_trim"
                        .to_string(),
                ),
                (
                    "has_smithing_trim_template".to_string(),
                    "inventory_changed:spire_armor_trim_smithing_template".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn smithing_trim_save_requires_unlock_criteria_via_advancement_builder() {
        let builder = SmithingTrimRecipeBuilderModel::smithing_trim(
            "minecraft:eye_armor_trim_smithing_template",
            "tag:minecraft:trimmable_armor",
            "tag:minecraft:trim_materials",
            "minecraft:eye",
            SmithingTrimCategory::Misc,
        );
        assert_eq!(
            builder
                .save("minecraft:eye_armor_trim_smithing_template_smithing_trim")
                .unwrap_err(),
            "No way of obtaining recipe eye_armor_trim_smithing_template_smithing_trim"
        );
    }

    #[test]
    fn smithing_trim_unlock_replacement_matches_advancement_builder_storage() {
        let mut builder = SmithingTrimRecipeBuilderModel::smithing_trim(
            "minecraft:sentry_armor_trim_smithing_template",
            "tag:minecraft:trimmable_armor",
            "tag:minecraft:trim_materials",
            "minecraft:sentry",
            SmithingTrimCategory::Misc,
        );
        builder
            .unlocks("has_template", "first")
            .unlocks("has_material", "second")
            .unlocks("has_template", "replacement");
        assert_eq!(
            builder
                .save("minecraft:sentry_armor_trim_smithing_template_smithing_trim")
                .unwrap()
                .advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:sentry_armor_trim_smithing_template_smithing_trim"
                        .to_string(),
                ),
                ("has_template".to_string(), "replacement".to_string()),
                ("has_material".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn smithing_trim_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(SMITHING_TRIM_RECIPE_BUILDER_JAVA, "smithingTrim("),
            1
        );
        assert_eq!(
            count_occurrences(SMITHING_TRIM_RECIPE_BUILDER_JAVA, "unlocks("),
            1
        );
        assert_eq!(
            count_occurrences(SMITHING_TRIM_RECIPE_BUILDER_JAVA, "save("),
            1
        );
        assert_eq!(
            count_occurrences(SMITHING_TRIM_RECIPE_BUILDER_JAVA, "Recipe.CommonInfo(true)"),
            1
        );
        assert_eq!(
            count_occurrences(SMITHING_TRIM_RECIPE_BUILDER_JAVA, "Optional.of"),
            0
        );
    }

    #[test]
    fn smithing_trim_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            SMITHING_TRIM_RECIPE_BUILDER_JAVA,
            &[
                "private final RecipeCategory category;",
                "private final Ingredient template;",
                "private final Ingredient base;",
                "private final Ingredient addition;",
                "private final Holder<TrimPattern> pattern;",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "return new SmithingTrimRecipeBuilder(category, template, base, addition, pattern);",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "SmithingTrimRecipe recipe = new SmithingTrimRecipe(new Recipe.CommonInfo(true), this.template, this.base, this.addition, this.pattern);",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.category));",
            ],
        );
    }
}
