const VANILLA_RECIPE_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/recipes/packs/VanillaRecipeProvider.java"
);
const DATA_RECIPES_PACKS_PACKAGE_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/recipes/packs/package-info.java"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SmeltableGroup {
    java_name: &'static str,
    output: &'static str,
    inputs: &'static [&'static str],
}

const SMELTABLE_GROUPS: &[SmeltableGroup] = &[
    SmeltableGroup {
        java_name: "COAL_SMELTABLES",
        output: "Items.COAL",
        inputs: &["Items.COAL_ORE", "Items.DEEPSLATE_COAL_ORE"],
    },
    SmeltableGroup {
        java_name: "IRON_SMELTABLES",
        output: "Items.IRON_INGOT",
        inputs: &[
            "Items.IRON_ORE",
            "Items.DEEPSLATE_IRON_ORE",
            "Items.RAW_IRON",
        ],
    },
    SmeltableGroup {
        java_name: "COPPER_SMELTABLES",
        output: "Items.COPPER_INGOT",
        inputs: &[
            "Items.COPPER_ORE",
            "Items.DEEPSLATE_COPPER_ORE",
            "Items.RAW_COPPER",
        ],
    },
    SmeltableGroup {
        java_name: "GOLD_SMELTABLES",
        output: "Items.GOLD_INGOT",
        inputs: &[
            "Items.GOLD_ORE",
            "Items.DEEPSLATE_GOLD_ORE",
            "Items.NETHER_GOLD_ORE",
            "Items.RAW_GOLD",
        ],
    },
    SmeltableGroup {
        java_name: "DIAMOND_SMELTABLES",
        output: "Items.DIAMOND",
        inputs: &["Items.DIAMOND_ORE", "Items.DEEPSLATE_DIAMOND_ORE"],
    },
    SmeltableGroup {
        java_name: "LAPIS_SMELTABLES",
        output: "Items.LAPIS_LAZULI",
        inputs: &["Items.LAPIS_ORE", "Items.DEEPSLATE_LAPIS_ORE"],
    },
    SmeltableGroup {
        java_name: "REDSTONE_SMELTABLES",
        output: "Items.REDSTONE",
        inputs: &["Items.REDSTONE_ORE", "Items.DEEPSLATE_REDSTONE_ORE"],
    },
    SmeltableGroup {
        java_name: "EMERALD_SMELTABLES",
        output: "Items.EMERALD",
        inputs: &["Items.EMERALD_ORE", "Items.DEEPSLATE_EMERALD_ORE"],
    },
];

const AUTHORITATIVE_PROVIDER_SENTINELS: &[&str] = &[
    "public class VanillaRecipeProvider extends RecipeProvider",
    "private VanillaRecipeProvider(final HolderLookup.Provider registries, final RecipeOutput output)",
    "protected void buildRecipes()",
    "this.output.includeRootAdvancement();",
    "this.generateForEnabledBlockFamilies(FeatureFlagSet.of(FeatureFlags.VANILLA));",
    "this.planksFromLog(Blocks.ACACIA_PLANKS, ItemTags.ACACIA_LOGS, 4);",
    "this.planksFromLogs(Blocks.OAK_PLANKS, ItemTags.OAK_LOGS, 4);",
    "this.colorItemWithDye(dyes, wools, \"wool\", RecipeCategory.BUILDING_BLOCKS);",
    "this.colorItemWithDye(dyes, harnesses, \"harness_dye\", RecipeCategory.COMBAT);",
    "this.shulkerBoxRecipes();",
    "this.bundleRecipes();",
    "public static Stream<VanillaRecipeProvider.TrimTemplate> smithingTrims()",
    "ResourceKey.create(Registries.RECIPE, Identifier.withDefaultNamespace(getItemName(item) + \"_smithing_trim\"))",
    "public static class Runner extends RecipeProvider.Runner",
    "return new VanillaRecipeProvider(registries, output);",
    "return \"Vanilla Recipes\";",
    "public record TrimTemplate(Item template, ResourceKey<TrimPattern> patternId, ResourceKey<Recipe<?>> recipeId)",
];

const HIGH_RISK_RECIPE_SENTINELS: &[&str] = &[
    "CustomCraftingRecipeBuilder.customCrafting(",
    "RecipeCategory.MISC,\n            (commonInfo, bookInfo) -> new ImbueRecipe(",
    "Ingredient.of(Items.LINGERING_POTION), Ingredient.of(Items.ARROW), new ItemStackTemplate(Items.TIPPED_ARROW, 8)",
    "new BookCloningRecipe(",
    ".save(this.output, \"book_cloning\");",
    "new FireworkRocketRecipe(",
    ".save(this.output, \"firework_rocket\");",
    "SpecialRecipeBuilder.special(RepairItemRecipe::new).save(this.output, \"repair_item\");",
    "TransmuteRecipeBuilder.transmute(RecipeCategory.MISC, Ingredient.of(Items.FILLED_MAP), Ingredient.of(Items.MAP), new ItemStackTemplate(Items.FILLED_MAP))",
    "SimpleCookingRecipeBuilder.smelting(Ingredient.of(Items.POTATO), RecipeCategory.FOOD, CookingBookCategory.FOOD, Items.BAKED_POTATO, 0.35F, 200)",
    "SimpleCookingRecipeBuilder.blasting(Ingredient.of(Blocks.ANCIENT_DEBRIS), RecipeCategory.MISC, CookingBookCategory.MISC, Items.NETHERITE_SCRAP, 2.0F, 100)",
    "this.suspiciousStew(item, effectHolder);",
    "this.shaped(RecipeCategory.COMBAT, Items.MACE, 1)",
    "this.shapeless(RecipeCategory.MISC, Items.WIND_CHARGE, 4)",
    "this.shaped(RecipeCategory.TOOLS, Items.BUNDLE)",
    "SpecialRecipeBuilder.special(() -> new DecoratedPotRecipe(this.tag(ItemTags.DECORATED_POT_INGREDIENTS), new ItemStackTemplate(Items.DECORATED_POT)))",
];

const TRIM_TEMPLATE_PAIRS: &[(&str, &str)] = &[
    (
        "Items.BOLT_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.BOLT",
    ),
    (
        "Items.COAST_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.COAST",
    ),
    (
        "Items.DUNE_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.DUNE",
    ),
    ("Items.EYE_ARMOR_TRIM_SMITHING_TEMPLATE", "TrimPatterns.EYE"),
    (
        "Items.FLOW_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.FLOW",
    ),
    (
        "Items.HOST_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.HOST",
    ),
    (
        "Items.RAISER_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.RAISER",
    ),
    ("Items.RIB_ARMOR_TRIM_SMITHING_TEMPLATE", "TrimPatterns.RIB"),
    (
        "Items.SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.SENTRY",
    ),
    (
        "Items.SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.SHAPER",
    ),
    (
        "Items.SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.SILENCE",
    ),
    (
        "Items.SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.SNOUT",
    ),
    (
        "Items.SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.SPIRE",
    ),
    (
        "Items.TIDE_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.TIDE",
    ),
    ("Items.VEX_ARMOR_TRIM_SMITHING_TEMPLATE", "TrimPatterns.VEX"),
    (
        "Items.WARD_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.WARD",
    ),
    (
        "Items.WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.WAYFINDER",
    ),
    (
        "Items.WILD_ARMOR_TRIM_SMITHING_TEMPLATE",
        "TrimPatterns.WILD",
    ),
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
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
    fn vanilla_recipe_provider_keeps_java_class_and_runner_contract() {
        assert_source_contains_all(
            VANILLA_RECIPE_PROVIDER_JAVA,
            AUTHORITATIVE_PROVIDER_SENTINELS,
        );
        assert_eq!(
            VANILLA_RECIPE_PROVIDER_JAVA.lines().count(),
            3038,
            "line-count drift means the source-backed audit needs review"
        );
        assert_eq!(
            count_occurrences(VANILLA_RECIPE_PROVIDER_JAVA, "extends RecipeProvider"),
            2
        );
        assert_eq!(
            count_occurrences(
                VANILLA_RECIPE_PROVIDER_JAVA,
                "protected RecipeProvider createRecipeProvider"
            ),
            1
        );
        assert_eq!(
            count_occurrences(VANILLA_RECIPE_PROVIDER_JAVA, "public String getName()"),
            1
        );
    }

    #[test]
    fn vanilla_recipe_provider_declares_exact_smeltable_groups() {
        assert_eq!(
            count_occurrences(
                VANILLA_RECIPE_PROVIDER_JAVA,
                "private static final ImmutableList<ItemLike>"
            ),
            SMELTABLE_GROUPS.len()
        );

        for group in SMELTABLE_GROUPS {
            assert!(
                VANILLA_RECIPE_PROVIDER_JAVA.contains(group.java_name),
                "missing smeltable group {}",
                group.java_name
            );
            assert!(
                VANILLA_RECIPE_PROVIDER_JAVA.contains(group.output),
                "missing smeltable output {}",
                group.output
            );
            for input in group.inputs {
                assert!(
                    VANILLA_RECIPE_PROVIDER_JAVA.contains(input),
                    "missing smeltable input {}",
                    input
                );
            }
        }
    }

    #[test]
    fn vanilla_recipe_provider_recipe_family_counts_match_java() {
        let expected_calls = [
            ("this.shaped(", 185),
            ("this.shapeless(", 60),
            ("this.stonecutterResultFromBase(", 48),
            ("this.oneToOneConversionRecipe(", 26),
            ("this.woodFromLogs(", 22),
            ("this.copySmithingTemplate(", 19),
            ("this.carpet(", 18),
            ("this.harness(", 16),
            ("this.dyedShulkerBoxRecipe(", 16),
            ("this.dyedBundleRecipe(", 16),
            ("this.netheriteSmithing(", 12),
            ("this.woodenBoat(", 10),
            ("this.chestBoat(", 10),
            ("this.oreSmelting(", 8),
            ("this.oreBlasting(", 8),
            ("this.grate(", 8),
            ("this.copperBulb(", 8),
            ("this.colorItemWithDye(", 4),
            ("this.planksFromLog(", 4),
            ("this.planksFromLogs(", 8),
            ("this.cookRecipes(", 2),
            ("this.trimSmithing(", 1),
            ("this.suspiciousStew(", 1),
        ];

        for (needle, expected) in expected_calls {
            assert_eq!(
                count_occurrences(VANILLA_RECIPE_PROVIDER_JAVA, needle),
                expected,
                "unexpected count for {needle}"
            );
        }
    }

    #[test]
    fn vanilla_recipe_provider_builder_families_match_java() {
        assert_source_contains_all(VANILLA_RECIPE_PROVIDER_JAVA, HIGH_RISK_RECIPE_SENTINELS);
        let expected_builder_mentions = [
            ("SimpleCookingRecipeBuilder", 55),
            ("SpecialRecipeBuilder", 9),
            ("TransmuteRecipeBuilder", 2),
            ("CustomCraftingRecipeBuilder", 2),
        ];

        for (needle, expected) in expected_builder_mentions {
            assert_eq!(
                count_occurrences(VANILLA_RECIPE_PROVIDER_JAVA, needle),
                expected,
                "unexpected builder mention count for {needle}"
            );
        }
    }

    #[test]
    fn vanilla_recipe_provider_smithing_trims_match_java() {
        assert_eq!(
            count_occurrences(VANILLA_RECIPE_PROVIDER_JAVA, "Pair.of(Items."),
            TRIM_TEMPLATE_PAIRS.len()
        );

        for (template, pattern) in TRIM_TEMPLATE_PAIRS {
            let sentinel = format!("Pair.of({template}, {pattern})");
            assert!(
                VANILLA_RECIPE_PROVIDER_JAVA.contains(&sentinel),
                "missing trim template pair {sentinel}"
            );
        }
    }

    #[test]
    fn vanilla_recipe_provider_package_is_null_marked() {
        assert!(DATA_RECIPES_PACKS_PACKAGE_JAVA.contains("@NullMarked"));
        assert!(
            DATA_RECIPES_PACKS_PACKAGE_JAVA.contains("package net.minecraft.data.recipes.packs;")
        );
        assert!(
            DATA_RECIPES_PACKS_PACKAGE_JAVA.contains("import org.jspecify.annotations.NullMarked;")
        );
        assert_eq!(
            count_occurrences(DATA_RECIPES_PACKS_PACKAGE_JAVA, "@NullMarked"),
            1
        );
    }
}
