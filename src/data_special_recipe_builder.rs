const SPECIAL_RECIPE_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/recipes/SpecialRecipeBuilder.java");

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpecialRecipeBuilderModel {
    factory_name: String,
    advancement_criteria: Option<Vec<(String, String)>>,
}

impl SpecialRecipeBuilderModel {
    fn special(factory_name: &str) -> Self {
        Self {
            factory_name: factory_name.to_string(),
            advancement_criteria: None,
        }
    }

    fn unlocked_by(&mut self, name: &str, criterion: &str) -> &mut Self {
        let criteria = self.advancement_criteria.get_or_insert_with(Vec::new);
        if let Some((_, existing)) = criteria
            .iter_mut()
            .find(|(existing_name, _)| existing_name == name)
        {
            *existing = criterion.to_string();
        } else {
            criteria.push((name.to_string(), criterion.to_string()));
        }
        self
    }

    fn save_by_name(&self, name: &str) -> SpecialRecipeSaveModel {
        self.save_by_key(&format!("minecraft:{name}"))
    }

    fn save_by_key(&self, id: &str) -> SpecialRecipeSaveModel {
        SpecialRecipeSaveModel {
            id: id.to_string(),
            recipe_from_factory: self.factory_name.clone(),
            advancement: self.advancement_criteria.as_ref().map(|criteria| {
                let mut advancement_criteria = vec![(
                    "has_the_recipe".to_string(),
                    format!("recipe_unlocked:{id}"),
                )];
                advancement_criteria.extend(criteria.iter().cloned());
                SpecialRecipeAdvancementModel {
                    id: format!("recipes/misc/{}", identifier_path(id)),
                    criteria: advancement_criteria,
                }
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpecialRecipeSaveModel {
    id: String,
    recipe_from_factory: String,
    advancement: Option<SpecialRecipeAdvancementModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpecialRecipeAdvancementModel {
    id: String,
    criteria: Vec<(String, String)>,
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
    fn special_recipe_factory_stores_supplier_name_without_eager_advancement_builder() {
        let builder = SpecialRecipeBuilderModel::special("BannerDuplicateRecipe::new");
        assert_eq!(builder.factory_name, "BannerDuplicateRecipe::new");
        assert!(builder.advancement_criteria.is_none());
    }

    #[test]
    fn special_recipe_save_without_unlock_passes_null_advancement_like_java() {
        let builder = SpecialRecipeBuilderModel::special("MapCloningRecipe::new");
        let save = builder.save_by_key("minecraft:map_cloning");
        assert_eq!(save.id, "minecraft:map_cloning");
        assert_eq!(save.recipe_from_factory, "MapCloningRecipe::new");
        assert_eq!(save.advancement, None);
    }

    #[test]
    fn special_recipe_save_by_name_parses_default_namespace_like_java() {
        let builder = SpecialRecipeBuilderModel::special("FireworkStarRecipe::new");
        assert_eq!(
            builder.save_by_name("firework_star").id,
            "minecraft:firework_star"
        );
    }

    #[test]
    fn special_recipe_unlocked_by_lazily_creates_advancement_builder_and_saves_misc_advancement() {
        let mut builder = SpecialRecipeBuilderModel::special("ArmorDyeRecipe::new");
        builder
            .unlocked_by("has_dye", "inventory_changed:dye")
            .unlocked_by("has_armor", "inventory_changed:leather_armor");
        let save = builder.save_by_key("minecraft:armor_dye");
        assert_eq!(
            save.advancement,
            Some(SpecialRecipeAdvancementModel {
                id: "recipes/misc/armor_dye".to_string(),
                criteria: vec![
                    (
                        "has_the_recipe".to_string(),
                        "recipe_unlocked:minecraft:armor_dye".to_string(),
                    ),
                    ("has_dye".to_string(), "inventory_changed:dye".to_string()),
                    (
                        "has_armor".to_string(),
                        "inventory_changed:leather_armor".to_string(),
                    ),
                ],
            })
        );
    }

    #[test]
    fn special_recipe_unlock_replacement_matches_advancement_builder_storage() {
        let mut builder = SpecialRecipeBuilderModel::special("BookCloningRecipe::new");
        builder
            .unlocked_by("has_book", "first")
            .unlocked_by("has_written_book", "second")
            .unlocked_by("has_book", "replacement");
        assert_eq!(
            builder
                .save_by_key("minecraft:book_cloning")
                .advancement
                .unwrap()
                .criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:book_cloning".to_string(),
                ),
                ("has_book".to_string(), "replacement".to_string()),
                ("has_written_book".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn special_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(SPECIAL_RECIPE_BUILDER_JAVA, "special("),
            1
        );
        assert_eq!(
            count_occurrences(SPECIAL_RECIPE_BUILDER_JAVA, "unlockedBy"),
            2
        );
        assert_eq!(count_occurrences(SPECIAL_RECIPE_BUILDER_JAVA, "save("), 3);
        assert_eq!(
            count_occurrences(SPECIAL_RECIPE_BUILDER_JAVA, "advancementBuilder"),
            6
        );
        assert_eq!(
            count_occurrences(SPECIAL_RECIPE_BUILDER_JAVA, "ResourceKey.create"),
            1
        );
        assert_eq!(
            count_occurrences(SPECIAL_RECIPE_BUILDER_JAVA, "Identifier.parse"),
            1
        );
        assert_eq!(
            count_occurrences(SPECIAL_RECIPE_BUILDER_JAVA, "factory.get()"),
            1
        );
    }

    #[test]
    fn special_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            SPECIAL_RECIPE_BUILDER_JAVA,
            &[
                "private @Nullable RecipeUnlockAdvancementBuilder advancementBuilder;",
                "private final Supplier<Recipe<?>> factory;",
                "public SpecialRecipeBuilder(final Supplier<Recipe<?>> factory)",
                "return new SpecialRecipeBuilder(factory);",
                "if (this.advancementBuilder == null)",
                "this.advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "this.save(output, ResourceKey.create(Registries.RECIPE, Identifier.parse(name)));",
                "AdvancementHolder advancement;",
                "advancement = this.advancementBuilder.build(output, id, RecipeCategory.MISC);",
                "advancement = null;",
                "output.accept(id, this.factory.get(), advancement);",
            ],
        );
    }
}
