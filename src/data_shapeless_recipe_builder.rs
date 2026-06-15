const SHAPELESS_RECIPE_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/recipes/ShapelessRecipeBuilder.java");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapelessRecipeCategory {
    Decorations,
    Food,
    Misc,
    Tools,
}

impl ShapelessRecipeCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::Decorations => "decorations",
            Self::Food => "food",
            Self::Misc => "misc",
            Self::Tools => "tools",
        }
    }

    fn crafting_book_category(self) -> &'static str {
        match self {
            Self::Tools => "equipment",
            Self::Decorations | Self::Food | Self::Misc => "misc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShapelessRecipeBuilderModel {
    category: ShapelessRecipeCategory,
    result: String,
    count: u32,
    ingredients: Vec<String>,
    advancement_criteria: Vec<(String, String)>,
    group: Option<String>,
}

impl ShapelessRecipeBuilderModel {
    fn shapeless_stack(category: ShapelessRecipeCategory, result: &str, count: u32) -> Self {
        Self {
            category,
            result: result.to_string(),
            count,
            ingredients: Vec::new(),
            advancement_criteria: Vec::new(),
            group: None,
        }
    }

    fn shapeless(category: ShapelessRecipeCategory, item: &str) -> Self {
        Self::shapeless_count(category, item, 1)
    }

    fn shapeless_count(category: ShapelessRecipeCategory, item: &str, count: u32) -> Self {
        Self::shapeless_stack(category, item, count)
    }

    fn requires_tag(&mut self, tag: &str) -> &mut Self {
        self.requires_ingredient(&format!("tag:{tag}"), 1)
    }

    fn requires_item(&mut self, item: &str) -> &mut Self {
        self.requires_item_count(item, 1)
    }

    fn requires_item_count(&mut self, item: &str, count: u32) -> &mut Self {
        for _ in 0..count {
            self.requires_ingredient(item, 1);
        }
        self
    }

    fn requires_ingredient(&mut self, ingredient: &str, count: u32) -> &mut Self {
        for _ in 0..count {
            self.ingredients.push(ingredient.to_string());
        }
        self
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

    fn save(&self, id: &str) -> Result<ShapelessRecipeSaveModel, String> {
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
        Ok(ShapelessRecipeSaveModel {
            id: id.to_string(),
            common_info: ShapelessCraftingCommonInfoModel {
                show_notification: true,
            },
            book_info: ShapelessCraftingBookInfoModel {
                category: self.category.crafting_book_category().to_string(),
                group: self.group.clone().unwrap_or_default(),
            },
            result: self.result.clone(),
            count: self.count,
            ingredients: self.ingredients.clone(),
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
struct ShapelessCraftingCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShapelessCraftingBookInfoModel {
    category: String,
    group: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShapelessRecipeSaveModel {
    id: String,
    common_info: ShapelessCraftingCommonInfoModel,
    book_info: ShapelessCraftingBookInfoModel,
    result: String,
    count: u32,
    ingredients: Vec<String>,
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
    fn shapeless_recipe_builder_factories_default_count_and_default_id_match_java() {
        let stack = ShapelessRecipeBuilderModel::shapeless_stack(
            ShapelessRecipeCategory::Food,
            "minecraft:mushroom_stew",
            1,
        );
        assert_eq!(stack.count, 1);
        assert_eq!(stack.default_id(), "minecraft:mushroom_stew");

        let item = ShapelessRecipeBuilderModel::shapeless(
            ShapelessRecipeCategory::Tools,
            "minecraft:flint_and_steel",
        );
        assert_eq!(item.count, 1);

        let counted = ShapelessRecipeBuilderModel::shapeless_count(
            ShapelessRecipeCategory::Misc,
            "minecraft:bone_meal",
            3,
        );
        assert_eq!(counted.count, 3);
        assert_eq!(counted.default_id(), "minecraft:bone_meal");
    }

    #[test]
    fn shapeless_recipe_builder_requires_overloads_expand_ingredients_like_java() {
        let mut builder =
            ShapelessRecipeBuilderModel::shapeless(ShapelessRecipeCategory::Misc, "minecraft:book");
        builder
            .requires_tag("minecraft:planks")
            .requires_item("minecraft:leather")
            .requires_item_count("minecraft:paper", 3)
            .requires_ingredient("compound:custom", 2);

        assert_eq!(
            builder.ingredients,
            vec![
                "tag:minecraft:planks",
                "minecraft:leather",
                "minecraft:paper",
                "minecraft:paper",
                "minecraft:paper",
                "compound:custom",
                "compound:custom",
            ]
        );
    }

    #[test]
    fn shapeless_recipe_builder_save_constructs_recipe_and_advancement_like_java() {
        let mut builder = ShapelessRecipeBuilderModel::shapeless_count(
            ShapelessRecipeCategory::Misc,
            "minecraft:orange_dye",
            2,
        );
        builder
            .requires_item("minecraft:red_dye")
            .requires_item("minecraft:yellow_dye")
            .group(Some("orange_dye"))
            .unlocked_by("has_red_dye", "inventory_changed:red_dye")
            .unlocked_by("has_yellow_dye", "inventory_changed:yellow_dye");

        let save = builder.save("minecraft:orange_dye").unwrap();
        assert_eq!(save.id, "minecraft:orange_dye");
        assert_eq!(
            save.common_info,
            ShapelessCraftingCommonInfoModel {
                show_notification: true
            }
        );
        assert_eq!(
            save.book_info,
            ShapelessCraftingBookInfoModel {
                category: "misc".to_string(),
                group: "orange_dye".to_string(),
            }
        );
        assert_eq!(save.result, "minecraft:orange_dye");
        assert_eq!(save.count, 2);
        assert_eq!(
            save.ingredients,
            vec!["minecraft:red_dye", "minecraft:yellow_dye"]
        );
        assert_eq!(save.advancement_id, "recipes/misc/orange_dye");
        assert_eq!(
            save.advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:orange_dye".to_string(),
                ),
                (
                    "has_red_dye".to_string(),
                    "inventory_changed:red_dye".to_string(),
                ),
                (
                    "has_yellow_dye".to_string(),
                    "inventory_changed:yellow_dye".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn shapeless_recipe_builder_nullable_group_becomes_empty_book_group_on_save() {
        let mut builder = ShapelessRecipeBuilderModel::shapeless(
            ShapelessRecipeCategory::Tools,
            "minecraft:bundle",
        );
        builder
            .requires_tag("minecraft:bundles")
            .requires_item("minecraft:blue_dye")
            .group(None)
            .unlocked_by("has_blue_dye", "inventory_changed:blue_dye");

        let save = builder.save("minecraft:blue_bundle").unwrap();
        assert_eq!(
            save.book_info,
            ShapelessCraftingBookInfoModel {
                category: "equipment".to_string(),
                group: String::new(),
            }
        );
        assert_eq!(save.advancement_id, "recipes/tools/blue_bundle");
    }

    #[test]
    fn shapeless_recipe_builder_save_requires_unlock_criteria_via_advancement_builder() {
        let mut builder = ShapelessRecipeBuilderModel::shapeless(
            ShapelessRecipeCategory::Decorations,
            "minecraft:white_carpet",
        );
        builder.requires_item_count("minecraft:white_wool", 2);
        assert_eq!(
            builder.save("minecraft:white_carpet").unwrap_err(),
            "No way of obtaining recipe white_carpet"
        );
    }

    #[test]
    fn shapeless_recipe_builder_unlock_replacement_matches_advancement_builder_storage() {
        let mut builder = ShapelessRecipeBuilderModel::shapeless(
            ShapelessRecipeCategory::Food,
            "minecraft:pumpkin_pie",
        );
        builder
            .requires_item("minecraft:pumpkin")
            .unlocked_by("has_pumpkin", "first")
            .unlocked_by("has_sugar", "second")
            .unlocked_by("has_pumpkin", "replacement");

        assert_eq!(
            builder
                .save("minecraft:pumpkin_pie")
                .unwrap()
                .advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:pumpkin_pie".to_string(),
                ),
                ("has_pumpkin".to_string(), "replacement".to_string()),
                ("has_sugar".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn shapeless_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(SHAPELESS_RECIPE_BUILDER_JAVA, "shapeless("),
            4
        );
        assert_eq!(
            count_occurrences(SHAPELESS_RECIPE_BUILDER_JAVA, "requires("),
            9
        );
        assert_eq!(
            count_occurrences(SHAPELESS_RECIPE_BUILDER_JAVA, "unlockedBy"),
            2
        );
        assert_eq!(
            count_occurrences(SHAPELESS_RECIPE_BUILDER_JAVA, "group("),
            1
        );
        assert_eq!(
            count_occurrences(SHAPELESS_RECIPE_BUILDER_JAVA, "showNotification"),
            0
        );
        assert_eq!(
            count_occurrences(SHAPELESS_RECIPE_BUILDER_JAVA, "defaultId"),
            1
        );
        assert_eq!(count_occurrences(SHAPELESS_RECIPE_BUILDER_JAVA, "save("), 1);
    }

    #[test]
    fn shapeless_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            SHAPELESS_RECIPE_BUILDER_JAVA,
            &[
                "private final HolderGetter<Item> items;",
                "private final RecipeCategory category;",
                "private final ItemStackTemplate result;",
                "private final List<Ingredient> ingredients = new ArrayList<>();",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "private @Nullable String group;",
                "public static ShapelessRecipeBuilder shapeless(final HolderGetter<Item> items, final RecipeCategory category, final ItemStackTemplate result)",
                "return new ShapelessRecipeBuilder(items, category, result);",
                "return shapeless(items, category, item, 1);",
                "return new ShapelessRecipeBuilder(items, category, new ItemStackTemplate(item.asItem(), count));",
                "return this.requires(Ingredient.of(this.items.getOrThrow(tag)));",
                "return this.requires(item, 1);",
                "for (int i = 0; i < count; i++)",
                "this.requires(Ingredient.of(item));",
                "return this.requires(ingredient, 1);",
                "this.ingredients.add(ingredient);",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "this.group = group;",
                "return RecipeBuilder.getDefaultRecipeId(this.result);",
                "RecipeBuilder.createCraftingCommonInfo(true)",
                "RecipeBuilder.createCraftingBookInfo(this.category, this.group)",
                "this.result, this.ingredients",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.category));",
            ],
        );
    }
}
