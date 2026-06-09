const REGISTRIES_DATAPACK_GENERATOR_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/registries/RegistriesDatapackGenerator.java"
);
const REGISTRY_PATCH_GENERATOR_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/registries/RegistryPatchGenerator.java"
);
const TRADE_REBALANCE_REGISTRIES_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/registries/TradeRebalanceRegistries.java"
);
const VANILLA_REGISTRIES_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/registries/VanillaRegistries.java"
);
const DATA_REGISTRIES_PACKAGE_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/registries/package-info.java");

const VANILLA_REGISTRY_BOOTSTRAPS: &[(&str, &str)] = &[
    ("Registries.DIMENSION_TYPE", "DimensionTypes::bootstrap"),
    ("Registries.CONFIGURED_CARVER", "Carvers::bootstrap"),
    ("Registries.CONFIGURED_FEATURE", "FeatureUtils::bootstrap"),
    ("Registries.PLACED_FEATURE", "PlacementUtils::bootstrap"),
    ("Registries.STRUCTURE", "Structures::bootstrap"),
    ("Registries.STRUCTURE_SET", "StructureSets::bootstrap"),
    ("Registries.PROCESSOR_LIST", "ProcessorLists::bootstrap"),
    ("Registries.TEMPLATE_POOL", "Pools::bootstrap"),
    ("Registries.BIOME", "BiomeData::bootstrap"),
    (
        "Registries.MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST",
        "MultiNoiseBiomeSourceParameterLists::bootstrap",
    ),
    ("Registries.NOISE", "NoiseData::bootstrap"),
    ("Registries.DENSITY_FUNCTION", "NoiseRouterData::bootstrap"),
    (
        "Registries.NOISE_SETTINGS",
        "NoiseGeneratorSettings::bootstrap",
    ),
    ("Registries.WORLD_PRESET", "WorldPresets::bootstrap"),
    (
        "Registries.FLAT_LEVEL_GENERATOR_PRESET",
        "FlatLevelGeneratorPresets::bootstrap",
    ),
    ("Registries.CHAT_TYPE", "ChatType::bootstrap"),
    ("Registries.TRIM_PATTERN", "TrimPatterns::bootstrap"),
    ("Registries.TRIM_MATERIAL", "TrimMaterials::bootstrap"),
    (
        "Registries.TRIAL_SPAWNER_CONFIG",
        "TrialSpawnerConfigs::bootstrap",
    ),
    ("Registries.WOLF_VARIANT", "WolfVariants::bootstrap"),
    (
        "Registries.WOLF_SOUND_VARIANT",
        "WolfSoundVariants::bootstrap",
    ),
    ("Registries.PAINTING_VARIANT", "PaintingVariants::bootstrap"),
    ("Registries.DAMAGE_TYPE", "DamageTypes::bootstrap"),
    ("Registries.BANNER_PATTERN", "BannerPatterns::bootstrap"),
    ("Registries.ENCHANTMENT", "Enchantments::bootstrap"),
    (
        "Registries.ENCHANTMENT_PROVIDER",
        "VanillaEnchantmentProviders::bootstrap",
    ),
    ("Registries.JUKEBOX_SONG", "JukeboxSongs::bootstrap"),
    ("Registries.INSTRUMENT", "Instruments::bootstrap"),
    ("Registries.PIG_VARIANT", "PigVariants::bootstrap"),
    (
        "Registries.PIG_SOUND_VARIANT",
        "PigSoundVariants::bootstrap",
    ),
    ("Registries.COW_VARIANT", "CowVariants::bootstrap"),
    (
        "Registries.COW_SOUND_VARIANT",
        "CowSoundVariants::bootstrap",
    ),
    ("Registries.CHICKEN_VARIANT", "ChickenVariants::bootstrap"),
    (
        "Registries.CHICKEN_SOUND_VARIANT",
        "ChickenSoundVariants::bootstrap",
    ),
    (
        "Registries.ZOMBIE_NAUTILUS_VARIANT",
        "ZombieNautilusVariants::bootstrap",
    ),
    (
        "Registries.TEST_ENVIRONMENT",
        "GameTestEnvironments::bootstrap",
    ),
    ("Registries.TEST_INSTANCE", "GameTestInstances::bootstrap"),
    ("Registries.FROG_VARIANT", "FrogVariants::bootstrap"),
    ("Registries.CAT_VARIANT", "CatVariants::bootstrap"),
    (
        "Registries.CAT_SOUND_VARIANT",
        "CatSoundVariants::bootstrap",
    ),
    ("Registries.DIALOG", "Dialogs::bootstrap"),
    ("Registries.WORLD_CLOCK", "WorldClocks::bootstrap"),
    ("Registries.TIMELINE", "Timelines::bootstrap"),
    ("Registries.VILLAGER_TRADE", "VillagerTrades::bootstrap"),
    ("Registries.TRADE_SET", "TradeSets::bootstrap"),
];

const DATAPACK_GENERATOR_SENTINELS: &[&str] = &[
    "public class RegistriesDatapackGenerator implements DataProvider",
    "private final PackOutput output;",
    "private final CompletableFuture<HolderLookup.Provider> registries;",
    "DynamicOps<JsonElement> registryOps = access.createSerializationContext(JsonOps.INSTANCE);",
    "RegistryDataLoader.WORLDGEN_REGISTRIES\n                     .stream()",
    "this.dumpRegistryCap(cache, access, registryOps, (RegistryDataLoader.RegistryData<?>)v)",
    "PackOutput.PathProvider pathProvider = this.output.createRegistryElementsPathProvider(registryKey);",
    "dumpValue(pathProvider.json(e.key().identifier()), cache, writeOps, v.elementCodec(), e.value())",
    "result -> DataProvider.saveStable(cache, result, path)",
    "error -> CompletableFuture.failedFuture(new IllegalStateException(\"Couldn't generate file '\" + path + \"': \" + error.message()))",
    "return \"Registries\";",
];

const PATCH_GENERATOR_SENTINELS: &[&str] = &[
    "RegistryAccess.Frozen staticRegistries = RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY);",
    "Cloner.Factory cloner = new Cloner.Factory();",
    "RegistryDataLoader.WORLDGEN_REGISTRIES.forEach(registryData -> registryData.runWithArguments(cloner::addCodec));",
    "RegistrySetBuilder.PatchedRegistries newRegistries = packBuilder.buildPatch(staticRegistries, parent, cloner);",
    "HolderLookup.Provider fullPatchedRegistry = newRegistries.full();",
    "Optional<? extends HolderLookup.RegistryLookup<Biome>> biomes = fullPatchedRegistry.lookup(Registries.BIOME);",
    "Optional<? extends HolderLookup.RegistryLookup<PlacedFeature>> features = fullPatchedRegistry.lookup(Registries.PLACED_FEATURE);",
    "if (biomes.isPresent() || features.isPresent())",
    "(HolderGetter<PlacedFeature>)DataFixUtils.orElseGet(features, () -> parent.lookupOrThrow(Registries.PLACED_FEATURE))",
    "(HolderLookup<Biome>)DataFixUtils.orElseGet(biomes, () -> parent.lookupOrThrow(Registries.BIOME))",
];

const VANILLA_VALIDATION_SENTINELS: &[&str] = &[
    "private static void validateThatAllBiomeFeaturesHaveBiomeFilter(final HolderLookup.Provider provider)",
    "provider.lookupOrThrow(Registries.PLACED_FEATURE), provider.lookupOrThrow(Registries.BIOME)",
    "biomes.listElements().forEach(biome -> {",
    "biome.value().getGenerationSettings().features();",
    "biomeFeatures.stream().flatMap(HolderSet::stream).forEach(feature -> feature.unwrap().ifLeft(key -> {",
    "Util.logAndPauseIfInIde(\"Placed feature \" + key.identifier() + \" in biome \" + biomeKey + \" is missing BiomeFilter.biome()\");",
    "Util.logAndPauseIfInIde(\"Placed inline feature in biome \" + biome + \" is missing BiomeFilter.biome()\");",
    "return value.placement().contains(BiomeFilter.biome());",
    "HolderLookup.Provider newRegistries = BUILDER.build(staticRegistries);",
    "validateThatAllBiomeFeaturesHaveBiomeFilter(newRegistries);",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlacedFeatureModel {
    id: &'static str,
    placements: Vec<&'static str>,
}

impl PlacedFeatureModel {
    fn has_biome_filter(&self) -> bool {
        self.placements.contains(&"BiomeFilter.biome()")
    }
}

fn missing_biome_filter_messages(
    biome_id: &str,
    referenced_features: &[PlacedFeatureModel],
    inline_features: &[PlacedFeatureModel],
) -> Vec<String> {
    let mut messages = Vec::new();

    for feature in referenced_features {
        if !feature.has_biome_filter() {
            messages.push(format!(
                "Placed feature {} in biome {} is missing BiomeFilter.biome()",
                feature.id, biome_id
            ));
        }
    }

    for feature in inline_features {
        if !feature.has_biome_filter() {
            messages.push(format!(
                "Placed inline feature in biome {}:{} is missing BiomeFilter.biome()",
                biome_id, feature.id
            ));
        }
    }

    messages
}

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
    fn data_registries_datapack_generator_matches_java_contract() {
        assert_source_contains_all(
            REGISTRIES_DATAPACK_GENERATOR_JAVA,
            DATAPACK_GENERATOR_SENTINELS,
        );
        assert_eq!(REGISTRIES_DATAPACK_GENERATOR_JAVA.lines().count(), 74);
        assert_eq!(
            count_occurrences(
                REGISTRIES_DATAPACK_GENERATOR_JAVA,
                "CompletableFuture.allOf"
            ),
            2
        );
        assert_eq!(
            count_occurrences(
                REGISTRIES_DATAPACK_GENERATOR_JAVA,
                "Optional<CompletableFuture<?>>"
            ),
            1
        );
        assert_eq!(
            count_occurrences(
                REGISTRIES_DATAPACK_GENERATOR_JAVA,
                "DataProvider.saveStable"
            ),
            1
        );
    }

    #[test]
    fn data_registries_patch_generator_matches_java_contract() {
        assert_source_contains_all(REGISTRY_PATCH_GENERATOR_JAVA, PATCH_GENERATOR_SENTINELS);
        assert_eq!(REGISTRY_PATCH_GENERATOR_JAVA.lines().count(), 41);
        assert_eq!(
            count_occurrences(
                REGISTRY_PATCH_GENERATOR_JAVA,
                "VanillaRegistries.validateThatAllBiomeFeaturesHaveBiomeFilter"
            ),
            1
        );
        assert_eq!(
            count_occurrences(REGISTRY_PATCH_GENERATOR_JAVA, "DataFixUtils.orElseGet"),
            2
        );
    }

    #[test]
    fn data_registries_trade_rebalance_delegates_to_patch_generator() {
        assert_eq!(TRADE_REBALANCE_REGISTRIES_JAVA.lines().count(), 15);
        assert_source_contains_all(
            TRADE_REBALANCE_REGISTRIES_JAVA,
            &[
                "public class TradeRebalanceRegistries",
                "private static final RegistrySetBuilder BUILDER = new RegistrySetBuilder().add(Registries.VILLAGER_TRADE, TradeRebalanceVillagerTrades::bootstrap);",
                "public static CompletableFuture<RegistrySetBuilder.PatchedRegistries> createLookup(final CompletableFuture<HolderLookup.Provider> vanilla)",
                "return RegistryPatchGenerator.createLookup(vanilla, BUILDER);",
            ],
        );
    }

    #[test]
    fn data_registries_vanilla_builder_entries_match_java_order() {
        assert_eq!(VANILLA_REGISTRIES_JAVA.lines().count(), 142);
        assert_eq!(
            count_occurrences(VANILLA_REGISTRIES_JAVA, ".add(Registries."),
            VANILLA_REGISTRY_BOOTSTRAPS.len()
        );

        let mut previous_index = 0;
        for (registry, bootstrap) in VANILLA_REGISTRY_BOOTSTRAPS {
            let sentinel = format!(".add({registry}, {bootstrap})");
            let index = VANILLA_REGISTRIES_JAVA
                .find(&sentinel)
                .unwrap_or_else(|| panic!("missing vanilla registry bootstrap {sentinel}"));
            assert!(
                index >= previous_index,
                "registry bootstrap out of Java declaration order: {sentinel}"
            );
            previous_index = index;
        }
    }

    #[test]
    fn data_registries_biome_filter_validation_matches_java_rule() {
        assert_source_contains_all(VANILLA_REGISTRIES_JAVA, VANILLA_VALIDATION_SENTINELS);

        let messages = missing_biome_filter_messages(
            "minecraft:plains",
            &[
                PlacedFeatureModel {
                    id: "minecraft:ore_dirt",
                    placements: vec!["CountPlacement.of(10)", "BiomeFilter.biome()"],
                },
                PlacedFeatureModel {
                    id: "minecraft:missing_filter",
                    placements: vec!["CountPlacement.of(1)"],
                },
            ],
            &[
                PlacedFeatureModel {
                    id: "inline_ok",
                    placements: vec!["BiomeFilter.biome()"],
                },
                PlacedFeatureModel {
                    id: "inline_missing",
                    placements: vec!["RarityFilter.onAverageOnceEvery(4)"],
                },
            ],
        );

        assert_eq!(
            messages,
            vec![
                "Placed feature minecraft:missing_filter in biome minecraft:plains is missing BiomeFilter.biome()",
                "Placed inline feature in biome minecraft:plains:inline_missing is missing BiomeFilter.biome()",
            ]
        );
    }

    #[test]
    fn data_registries_package_is_null_marked() {
        assert!(DATA_REGISTRIES_PACKAGE_JAVA.contains("@NullMarked"));
        assert!(DATA_REGISTRIES_PACKAGE_JAVA.contains("package net.minecraft.data.registries;"));
        assert!(
            DATA_REGISTRIES_PACKAGE_JAVA.contains("import org.jspecify.annotations.NullMarked;")
        );
        assert_eq!(
            count_occurrences(DATA_REGISTRIES_PACKAGE_JAVA, "@NullMarked"),
            1
        );
    }
}
