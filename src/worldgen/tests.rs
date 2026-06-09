use super::{
    builtin_density_function, builtin_noise_generator_settings, builtin_noise_router,
    density_function_type, random_state_normal_noise_snapshot, AquiferNoiseSettings,
    BinaryDensityFunction, BiomeGenerationSettingsModel, BlendingDataPacked, BlendingOutput,
    BlockPos, BlockPredicate, BlockPredicateContext, CarverShape, CaveDensityOutput, CaveSurface,
    ConfiguredFeatureSource, DensityFunction, DensityMarker, FeatureConfigurationKind,
    FeatureFamily, FlatLayerInfo, FloatProvider, FluidStatus, FoliagePlacerKind,
    FoliagePlacerModel, GenerationDecorationStep, HeightProvider, HeightRange,
    MappedDensityFunction, MobSpawnerDataModel, NoiseRouterPreset, NoiseSettings,
    OreVeinDecisionInput, OreVeinifierConstants, PlacedFeatureSource, PlacementContextModel,
    PlacementModifier, RandomSpreadType, RandomStateNoiseCache, SpawnBlockKind, SpawnColumnHeights,
    StructureFamily, StructurePlacementKind, SurfaceConditionSource, SurfaceMaterialContext,
    SurfaceRuleKind, SurfaceRulePreset, SurfaceRuleSource, TreePlacementBlockKind, TrunkPlacerKind,
    TrunkPlacerModel, VerticalAnchor, WeightedHeightProvider, WorldCarverType,
    WorldGenerationHeightContext, AQUIFER_NOISE_SETTINGS,
    AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS, BLENDING_CELL_COLUMN_COUNT, BLENDING_CONSTANTS,
    BLENDING_NO_VALUE, BLOCK_PREDICATE_TYPES, BUILTIN_DENSITY_FUNCTIONS,
    BUILTIN_NOISE_GENERATOR_SETTINGS, BUILTIN_NOISE_ROUTERS, BUILTIN_STRUCTURES,
    BUILTIN_STRUCTURE_SETS, BUILTIN_SURFACE_RULE_PRESETS, CAVES_NOISE_SETTINGS,
    CAVE_GENERATION_FAMILIES, CONFIGURED_CARVERS, CONFIGURED_FEATURES, DENSITY_FUNCTION_TYPES,
    END_NOISE_SETTINGS, EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS, FEATURE_BEHAVIOR_MODELS,
    FEATURE_TYPES, FLAT_DEFAULT_LAYERS, FLAT_GENERATOR_PRESETS, FLOATING_ISLANDS_NOISE_SETTINGS,
    HEIGHT_PROVIDER_TYPES, JIGSAW_POOL_BOOTSTRAP_SOURCES, MONSTER_ROOM_BOUNDS,
    NETHER_NOISE_SETTINGS, NORMAL_NOISE_INPUT_FACTOR, NORMAL_NOISE_PARAMETERS,
    NORMAL_NOISE_TARGET_DEVIATION, ORE_VEINIFIER_CONSTANTS, ORE_VEIN_TYPES,
    OVERWORLD_NOISE_SETTINGS, OVERWORLD_SPAWN_TARGET, PLACED_FEATURE_BOOTSTRAP_SOURCES,
    SPAWN_SELECTION_CONSTANTS, STRUCTURE_FAMILIES, STRUCTURE_PIECE_TYPES,
    STRUCTURE_POOL_ELEMENT_TYPES, STRUCTURE_POS_RULE_TEST_TYPES, STRUCTURE_PROCESSOR_LISTS,
    STRUCTURE_PROCESSOR_TYPES, STRUCTURE_RULE_TEST_TYPES, STRUCTURE_TYPES, SURFACE_CONDITION_TYPES,
    SURFACE_RULE_TYPES, SYNTH_NOISE_SOURCES, TEST_NEGATIVE_DENSITY, TEST_POSITIVE_DENSITY,
    UPGRADE_DATA_MODEL, WORLD_CARVER_TYPES, WORLD_PRESETS, Y_DENSITY,
};
use crate::biome::{quantize_coord, BiomeSourceModel};
use crate::storage::chunk::{
    ChunkSection, HeightmapKind, LevelChunk, PalettedContainer, BIOME_SECTION_VOLUME,
    SECTION_VOLUME,
};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;
use std::collections::BTreeMap;
use std::path::Path;

mod noise_parity_tests;

mod placement_registry_tests;

mod world_preset_tests;

mod live_surface_tests;

mod overworld_tree_diagnostics_tests;

mod overworld_surface_context_diagnostics_tests;

mod overworld_mismatch_diagnostics_tests;

mod vegetation_ore_cache_tests;

mod overworld_structure_diagnostics_tests;

fn unpack_heightmap_column(values: &[i64], index: usize) -> i32 {
    const BITS_PER_ENTRY: usize = 9;
    let bit_offset = index * BITS_PER_ENTRY;
    let word_index = bit_offset / 64;
    let bit_index = bit_offset % 64;
    let mut value = ((values[word_index] as u64) >> bit_index) & ((1 << BITS_PER_ENTRY) - 1);
    let spill = bit_index + BITS_PER_ENTRY;
    if spill > 64 {
        value |= (values[word_index + 1] as u64) << (64 - bit_index);
        value &= (1 << BITS_PER_ENTRY) - 1;
    }
    value as i32
}

mod generator_carver_preview_tests;

mod noise_density_core_tests;

mod noise_router_surface_tests;

mod aquifer_ore_tests;

mod configured_carver_tests;

mod feature_registry_tests;

mod feature_registry_nether_tests;

mod biome_generation_basic_tests;

mod biome_neighbor_payload_surface_tests;

mod biome_neighbor_payload_dimension_tests;

mod feature_sorter_decoration_tests;

mod feature_placement_support_provider_tests;

mod feature_placement_support_environment_tests;

mod feature_placement_support_geode_end_tests;

mod feature_placement_support_end_structures_tests;

mod feature_placement_support_dripstone_tests;

mod feature_placement_support_tree_base_tests;

mod feature_placement_support_tree_variant_tests;

mod feature_placement_support_decorator_tests;

mod feature_placement_behavior_tests;

mod structure_placement_tests;

mod structure_piece_tests;

mod jigsaw_core_tests;

mod jigsaw_expansion_tests;

mod structure_template_processor_tests;

mod structure_access_scattered_tests;

mod structure_template_feature_tests;

mod structure_large_generation_tests;

mod structure_large_template_tests;

mod registry_terrain_constants_tests;

mod spawn_planning_tests;

mod mob_entity_nbt_tests;

mod mob_batch_player_spawn_tests;

mod flat_noise_settings_tests;

#[test]
fn biome_generation_settings_cover_neighboring_overworld_payloads() {
    biome_neighbor_payload_surface_tests::assert_surface_and_cave_payloads();
    biome_neighbor_payload_dimension_tests::assert_dimension_and_remaining_overworld_payloads();
}

#[test]
fn feature_placement_support_registries_match_vanilla_type_bootstraps() {
    feature_placement_support_provider_tests::assert_provider_and_ore_support();
    feature_placement_support_geode_end_tests::assert_geode_nether_and_end_support();
    feature_placement_support_dripstone_tests::assert_disk_dripstone_and_large_dripstone_support();
    feature_placement_support_tree_base_tests::assert_tree_size_and_basic_tree_support();
    feature_placement_support_tree_variant_tests::assert_tree_variant_and_trunk_support();
    feature_placement_support_decorator_tests::assert_decorator_spring_and_monster_room_support();
}
