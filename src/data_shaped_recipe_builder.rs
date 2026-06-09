const SHAPED_RECIPE_BUILDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/recipes/ShapedRecipeBuilder.java"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapedRecipeCategory {
    BuildingBlocks,
    Redstone,
    Tools,
}

impl ShapedRecipeCategory {
    fn folder_name(self) -> &'static str {
        match self {
            Self::BuildingBlocks => "building_blocks",
            Self::Redstone => "redstone",
            Self::Tools => "tools",
        }
    }

    fn crafting_book_category(self) -> &'static str {
        match self {
            Self::BuildingBlocks => "building",
            Self::Redstone => "redstone",
            Self::Tools => "equipment",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShapedRecipeBuilderModel {
    category: ShapedRecipeCategory,
    result: String,
    count: u32,
    rows: Vec<String>,
    key: Vec<(char, String)>,
    advancement_criteria: Vec<(String, String)>,
    group: Option<String>,
    show_notification: bool,
}

impl ShapedRecipeBuilderModel {
    fn shaped(category: ShapedRecipeCategory, item: &str) -> Self {
        Self::shaped_count(category, item, 1)
    }

    fn shaped_count(category: ShapedRecipeCategory, item: &str, count: u32) -> Self {
        Self {
            category,
            result: item.to_string(),
            count,
            rows: Vec::new(),
            key: Vec::new(),
            advancement_criteria: Vec::new(),
            group: None,
            show_notification: true,
        }
    }

    fn define(&mut self, symbol: char, ingredient: &str) -> Result<&mut Self, String> {
        if self.key.iter().any(|(defined, _)| *defined == symbol) {
            return Err(format!("Symbol '{symbol}' is already defined!"));
        }
        if symbol == ' ' {
            return Err("Symbol ' ' (whitespace) is reserved and cannot be defined".to_string());
        }
        self.key.push((symbol, ingredient.to_string()));
        Ok(self)
    }

    fn define_tag(&mut self, symbol: char, tag: &str) -> Result<&mut Self, String> {
        self.define(symbol, &format!("tag:{tag}"))
    }

    fn pattern(&mut self, row: &str) -> Result<&mut Self, String> {
        if self
            .rows
            .first()
            .is_some_and(|first_row| row.len() != first_row.len())
        {
            return Err("Pattern must be the same width on every line!".to_string());
        }
        self.rows.push(row.to_string());
        Ok(self)
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

    fn show_notification(&mut self, show_notification: bool) -> &mut Self {
        self.show_notification = show_notification;
        self
    }

    fn default_id(&self) -> String {
        self.result.clone()
    }

    fn save(&self, id: &str) -> Result<ShapedRecipeSaveModel, String> {
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
        Ok(ShapedRecipeSaveModel {
            id: id.to_string(),
            common_info: ShapedCraftingCommonInfoModel {
                show_notification: self.show_notification,
            },
            book_info: ShapedCraftingBookInfoModel {
                category: self.category.crafting_book_category().to_string(),
                group: self.group.clone().unwrap_or_default(),
            },
            pattern: ShapedPatternModel {
                key: self.key.clone(),
                rows: self.rows.clone(),
            },
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
struct ShapedCraftingCommonInfoModel {
    show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShapedCraftingBookInfoModel {
    category: String,
    group: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShapedPatternModel {
    key: Vec<(char, String)>,
    rows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShapedRecipeSaveModel {
    id: String,
    common_info: ShapedCraftingCommonInfoModel,
    book_info: ShapedCraftingBookInfoModel,
    pattern: ShapedPatternModel,
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
    fn shaped_recipe_builder_factories_default_count_and_notification_match_java() {
        let single =
            ShapedRecipeBuilderModel::shaped(ShapedRecipeCategory::Tools, "minecraft:iron_pickaxe");
        assert_eq!(single.count, 1);
        assert_eq!(single.default_id(), "minecraft:iron_pickaxe");
        assert!(single.show_notification);

        let counted = ShapedRecipeBuilderModel::shaped_count(
            ShapedRecipeCategory::BuildingBlocks,
            "minecraft:oak_stairs",
            4,
        );
        assert_eq!(counted.count, 4);
        assert_eq!(counted.default_id(), "minecraft:oak_stairs");
    }

    #[test]
    fn shaped_recipe_builder_define_validation_matches_java() {
        let mut builder =
            ShapedRecipeBuilderModel::shaped(ShapedRecipeCategory::Redstone, "minecraft:lever");
        builder.define('#', "minecraft:cobblestone").unwrap();
        assert_eq!(
            builder.define('#', "minecraft:stone").unwrap_err(),
            "Symbol '#' is already defined!"
        );
        assert_eq!(
            builder.define(' ', "minecraft:stick").unwrap_err(),
            "Symbol ' ' (whitespace) is reserved and cannot be defined"
        );

        let mut tag_builder =
            ShapedRecipeBuilderModel::shaped(ShapedRecipeCategory::Tools, "minecraft:wooden_axe");
        tag_builder.define_tag('P', "minecraft:planks").unwrap();
        assert_eq!(
            tag_builder.key,
            vec![('P', "tag:minecraft:planks".to_string())]
        );
    }

    #[test]
    fn shaped_recipe_builder_pattern_width_validation_matches_java() {
        let mut builder = ShapedRecipeBuilderModel::shaped_count(
            ShapedRecipeCategory::BuildingBlocks,
            "minecraft:stone_bricks",
            4,
        );
        builder.pattern("##").unwrap();
        builder.pattern("##").unwrap();
        assert_eq!(
            builder.pattern("#").unwrap_err(),
            "Pattern must be the same width on every line!"
        );
        assert_eq!(builder.rows, vec!["##", "##"]);
    }

    #[test]
    fn shaped_recipe_builder_save_constructs_recipe_and_advancement_like_java() {
        let mut builder = ShapedRecipeBuilderModel::shaped_count(
            ShapedRecipeCategory::BuildingBlocks,
            "minecraft:oak_slab",
            6,
        );
        builder
            .define('#', "minecraft:oak_planks")
            .unwrap()
            .pattern("###")
            .unwrap()
            .group(Some("wooden_slab"))
            .show_notification(false)
            .unlocked_by("has_oak_planks", "inventory_changed:oak_planks");

        let save = builder.save("minecraft:oak_slab").unwrap();
        assert_eq!(save.id, "minecraft:oak_slab");
        assert_eq!(
            save.common_info,
            ShapedCraftingCommonInfoModel {
                show_notification: false
            }
        );
        assert_eq!(
            save.book_info,
            ShapedCraftingBookInfoModel {
                category: "building".to_string(),
                group: "wooden_slab".to_string(),
            }
        );
        assert_eq!(
            save.pattern,
            ShapedPatternModel {
                key: vec![('#', "minecraft:oak_planks".to_string())],
                rows: vec!["###".to_string()],
            }
        );
        assert_eq!(save.result, "minecraft:oak_slab");
        assert_eq!(save.count, 6);
        assert_eq!(save.advancement_id, "recipes/building_blocks/oak_slab");
        assert_eq!(
            save.advancement_criteria,
            vec![
                (
                    "has_the_recipe".to_string(),
                    "recipe_unlocked:minecraft:oak_slab".to_string(),
                ),
                (
                    "has_oak_planks".to_string(),
                    "inventory_changed:oak_planks".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn shaped_recipe_builder_save_requires_unlock_criteria_via_advancement_builder() {
        let mut builder =
            ShapedRecipeBuilderModel::shaped(ShapedRecipeCategory::Tools, "minecraft:shears");
        builder.define('#', "minecraft:iron_ingot").unwrap();
        builder.pattern("# ").unwrap().pattern(" #").unwrap();
        assert_eq!(
            builder.save("minecraft:shears").unwrap_err(),
            "No way of obtaining recipe shears"
        );
    }

    #[test]
    fn shaped_recipe_builder_nullable_group_becomes_empty_book_group_on_save() {
        let mut builder =
            ShapedRecipeBuilderModel::shaped(ShapedRecipeCategory::Redstone, "minecraft:lever");
        builder
            .define('#', "minecraft:cobblestone")
            .unwrap()
            .define('X', "minecraft:stick")
            .unwrap()
            .pattern("X")
            .unwrap()
            .pattern("#")
            .unwrap()
            .unlocked_by("has_cobblestone", "inventory_changed:cobblestone");
        assert_eq!(
            builder.save("minecraft:lever").unwrap().book_info,
            ShapedCraftingBookInfoModel {
                category: "redstone".to_string(),
                group: String::new(),
            }
        );
    }

    #[test]
    fn shaped_recipe_builder_source_counts_match_authoritative_java() {
        assert_eq!(count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "shaped("), 3);
        assert_eq!(count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "define("), 5);
        assert_eq!(count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "pattern("), 1);
        assert_eq!(
            count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "unlockedBy"),
            2
        );
        assert_eq!(count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "group("), 1);
        assert_eq!(
            count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "showNotification"),
            6
        );
        assert_eq!(
            count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "defaultId"),
            1
        );
        assert_eq!(count_occurrences(SHAPED_RECIPE_BUILDER_JAVA, "save("), 1);
    }

    #[test]
    fn shaped_recipe_builder_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            SHAPED_RECIPE_BUILDER_JAVA,
            &[
                "private final HolderGetter<Item> items;",
                "private final RecipeCategory category;",
                "private final ItemStackTemplate result;",
                "private final List<String> rows = Lists.newArrayList();",
                "private final Map<Character, Ingredient> key = Maps.newLinkedHashMap();",
                "private final RecipeUnlockAdvancementBuilder advancementBuilder = new RecipeUnlockAdvancementBuilder();",
                "private @Nullable String group;",
                "private boolean showNotification = true;",
                "return shaped(items, category, item, 1);",
                "return new ShapedRecipeBuilder(items, category, item, count);",
                "return this.define(symbol, Ingredient.of(this.items.getOrThrow(tag)));",
                "return this.define(symbol, Ingredient.of(item));",
                "throw new IllegalArgumentException(\"Symbol '\" + symbol + \"' is already defined!\");",
                "throw new IllegalArgumentException(\"Symbol ' ' (whitespace) is reserved and cannot be defined\");",
                "throw new IllegalArgumentException(\"Pattern must be the same width on every line!\");",
                "this.advancementBuilder.unlockedBy(name, criterion);",
                "this.group = group;",
                "this.showNotification = showNotification;",
                "return RecipeBuilder.getDefaultRecipeId(this.result);",
                "ShapedRecipePattern pattern = ShapedRecipePattern.of(this.key, this.rows);",
                "RecipeBuilder.createCraftingCommonInfo(this.showNotification)",
                "RecipeBuilder.createCraftingBookInfo(this.category, this.group)",
                "output.accept(id, recipe, this.advancementBuilder.build(output, id, this.category));",
            ],
        );
    }
}
