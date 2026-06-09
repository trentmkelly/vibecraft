use std::collections::BTreeMap;

const CUSTOM_CRAFTING_RECIPE_BUILDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/recipes/CustomCraftingRecipeBuilder.java"
);
const RECIPE_BUILDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeBuilder.java");
const RECIPE_CATEGORY_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeCategory.java");

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
    }
}
