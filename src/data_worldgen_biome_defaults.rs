use crate::worldgen::{
    MobSpawnerDataModel, DEEP_DARK_FEATURE_STEPS, DESERT_CREATURE_SPAWNS, DESERT_MONSTER_SPAWNS,
    DRIPSTONE_CAVES_FEATURE_STEPS, DRIPSTONE_CAVES_MONSTER_SPAWNS, LUSH_CAVES_FEATURE_STEPS,
    PLAINS_CREATURE_SPAWNS, PLAINS_FEATURE_STEPS, PLAINS_MONSTER_SPAWNS, SWAMP_MONSTER_SPAWNS,
};

const BIOME_DEFAULT_FEATURES_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BiomeDefaultFeatures.java"
);

const BIOME_DEFAULT_FEATURE_SENTINELS: &[&str] = &[
    "public class BiomeDefaultFeatures",
    "public static void addDefaultCarversAndLakes(final BiomeGenerationSettings.Builder builder)",
    "builder.addCarver(Carvers.CAVE);",
    "builder.addCarver(Carvers.CAVE_EXTRA_UNDERGROUND);",
    "builder.addCarver(Carvers.CANYON);",
    "builder.addFeature(GenerationStep.Decoration.UNDERGROUND_STRUCTURES, CavePlacements.MONSTER_ROOM_DEEP);",
    "builder.addFeature(GenerationStep.Decoration.UNDERGROUND_ORES, largeCopperBlobs ? OrePlacements.ORE_COPPER_LARGE : OrePlacements.ORE_COPPER);",
    "builder.addFeature(GenerationStep.Decoration.VEGETAL_DECORATION, CavePlacements.LUSH_CAVES_CEILING_VEGETATION);",
    "builder.addFeature(GenerationStep.Decoration.VEGETAL_DECORATION, CavePlacements.CLASSIC_VINES);",
    "builder.addFeature(GenerationStep.Decoration.VEGETAL_DECORATION, VegetationPlacements.WILDFLOWERS_MEADOW);",
    "builder.addFeature(GenerationStep.Decoration.VEGETAL_DECORATION, VegetationPlacements.PATCH_FIREFLY_BUSH_NEAR_WATER);",
    "builder.addFeature(GenerationStep.Decoration.VEGETAL_DECORATION, VegetationPlacements.PATCH_LEAF_LITTER);",
    "builder.addFeature(GenerationStep.Decoration.UNDERGROUND_DECORATION, OrePlacements.ORE_ANCIENT_DEBRIS_SMALL);",
    "builder.addSpawn(MobCategory.CREATURE, 12, new MobSpawnSettings.SpawnerData(EntityType.SHEEP, 4, 4));",
    "builder.addSpawn(MobCategory.MONSTER, 30, new MobSpawnSettings.SpawnerData(EntityType.BOGGED, 4, 4));",
    "builder.addSpawn(MobCategory.MONSTER, 50, new MobSpawnSettings.SpawnerData(EntityType.PARCHED, 4, 4));",
    "builder.addSpawn(MobCategory.MONSTER, zombieHorseWeight, new MobSpawnSettings.SpawnerData(EntityType.ZOMBIE_HORSE, 1, 1));",
    "builder.addSpawn(MobCategory.MONSTER, 95, new MobSpawnSettings.SpawnerData(EntityType.DROWNED, 4, 4));",
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

fn assert_step_contains(steps: &[&[&str]], step: usize, feature: &str) {
    assert!(
        steps[step].contains(&feature),
        "missing feature {feature} in decoration step {step}"
    );
}

fn spawn<'a>(spawns: &'a [MobSpawnerDataModel], entity_type: &str) -> &'a MobSpawnerDataModel {
    spawns
        .iter()
        .find(|spawn| spawn.entity_type == entity_type)
        .unwrap_or_else(|| panic!("missing spawn entry for {entity_type}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_default_features_source_contract_matches_java() {
        assert_source_contains_all(BIOME_DEFAULT_FEATURES_JAVA, BIOME_DEFAULT_FEATURE_SENTINELS);
        assert_eq!(
            BIOME_DEFAULT_FEATURES_JAVA.lines().count(),
            543,
            "BiomeDefaultFeatures.java line-count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, "public static void "),
            96,
            "BiomeDefaultFeatures method count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, ".addFeature("),
            163,
            "BiomeDefaultFeatures addFeature count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, ".addCarver("),
            3,
            "BiomeDefaultFeatures addCarver count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, ".addSpawn("),
            38,
            "BiomeDefaultFeatures addSpawn count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, "GenerationStep.Decoration."),
            163,
            "generation decoration token count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, "MobCategory."),
            38,
            "mob category token count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, "VegetationPlacements."),
            89,
            "vegetation placement token count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, "OrePlacements."),
            36,
            "ore placement token count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_DEFAULT_FEATURES_JAVA, "CavePlacements."),
            19,
            "cave placement token count drift"
        );
    }

    #[test]
    fn rust_biome_feature_steps_cover_representative_java_defaults() {
        assert_step_contains(PLAINS_FEATURE_STEPS, 1, "minecraft:lake_lava_underground");
        assert_step_contains(PLAINS_FEATURE_STEPS, 1, "minecraft:lake_lava_surface");
        assert_step_contains(PLAINS_FEATURE_STEPS, 3, "minecraft:monster_room_deep");
        assert_step_contains(PLAINS_FEATURE_STEPS, 6, "minecraft:ore_copper");
        assert_step_contains(PLAINS_FEATURE_STEPS, 6, "minecraft:underwater_magma");
        assert_step_contains(
            PLAINS_FEATURE_STEPS,
            9,
            "minecraft:patch_firefly_bush_near_water",
        );
        assert_eq!(PLAINS_FEATURE_STEPS[6].len(), 29);

        assert_step_contains(LUSH_CAVES_FEATURE_STEPS, 6, "minecraft:ore_clay");
        assert_step_contains(
            LUSH_CAVES_FEATURE_STEPS,
            9,
            "minecraft:lush_caves_ceiling_vegetation",
        );
        assert_step_contains(LUSH_CAVES_FEATURE_STEPS, 9, "minecraft:cave_vines");
        assert_step_contains(
            LUSH_CAVES_FEATURE_STEPS,
            9,
            "minecraft:classic_vines_cave_feature",
        );

        assert_step_contains(
            DRIPSTONE_CAVES_FEATURE_STEPS,
            2,
            "minecraft:large_dripstone",
        );
        assert_step_contains(
            DRIPSTONE_CAVES_FEATURE_STEPS,
            6,
            "minecraft:ore_copper_large",
        );
        assert_step_contains(
            DRIPSTONE_CAVES_FEATURE_STEPS,
            7,
            "minecraft:dripstone_cluster",
        );
        assert_step_contains(
            DRIPSTONE_CAVES_FEATURE_STEPS,
            7,
            "minecraft:pointed_dripstone",
        );
        assert_step_contains(DEEP_DARK_FEATURE_STEPS, 7, "minecraft:sculk_vein");
        assert_step_contains(
            DEEP_DARK_FEATURE_STEPS,
            7,
            "minecraft:sculk_patch_deep_dark",
        );
    }

    #[test]
    fn rust_biome_spawns_cover_representative_java_defaults() {
        let sheep = spawn(PLAINS_CREATURE_SPAWNS, "minecraft:sheep");
        assert_eq!((sheep.weight, sheep.min_count, sheep.max_count), (12, 4, 4));
        let horse = spawn(PLAINS_CREATURE_SPAWNS, "minecraft:horse");
        assert_eq!((horse.weight, horse.min_count, horse.max_count), (5, 2, 6));
        let zombie_horse = spawn(PLAINS_MONSTER_SPAWNS, "minecraft:zombie_horse");
        assert_eq!(
            (
                zombie_horse.weight,
                zombie_horse.min_count,
                zombie_horse.max_count
            ),
            (5, 1, 1)
        );

        let camel = spawn(DESERT_CREATURE_SPAWNS, "minecraft:camel");
        assert_eq!((camel.weight, camel.min_count, camel.max_count), (1, 1, 1));
        let parched = spawn(DESERT_MONSTER_SPAWNS, "minecraft:parched");
        assert_eq!(
            (parched.weight, parched.min_count, parched.max_count),
            (50, 4, 4)
        );

        let bogged = spawn(SWAMP_MONSTER_SPAWNS, "minecraft:bogged");
        assert_eq!(
            (bogged.weight, bogged.min_count, bogged.max_count),
            (30, 4, 4)
        );
        let drowned = spawn(DRIPSTONE_CAVES_MONSTER_SPAWNS, "minecraft:drowned");
        assert_eq!(
            (drowned.weight, drowned.min_count, drowned.max_count),
            (95, 4, 4)
        );
    }
}
