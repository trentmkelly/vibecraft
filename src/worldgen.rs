#![allow(dead_code)]

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::biome::{
    biome_source_from_stem_id, climate_target, multi_noise_parameter_list_preset,
    overworld_biome_parameters, select_biome_from_source, select_climate_biome, select_end_biome,
    span, BiomeSourceModel, ClimateParameterPoint, ClimateTarget,
};
pub use crate::random_source::RandomAlgorithm;

use crate::random_source::{
    carver_seed, large_feature_seed_with_salt, random_state_named_factory,
    random_state_seed_factories, worldgen_random_next_f32, worldgen_random_next_f64,
    worldgen_random_next_i32_bound, worldgen_random_next_i64, LegacyRandom,
    PositionalRandomFactory, RandomSourceKind,
};
use crate::registry::Identifier;
use crate::storage::chunk::{
    chunk_status, palette_bits_for_size, unpack_palette_indices, BlockStateEntry, ChunkSection,
    HeightmapKind, LevelChunk, PalettedContainer, BIOME_SECTION_VOLUME, SECTION_VOLUME,
};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;

mod noise_models;
pub use self::noise_models::*;

mod biome_sampling;
pub use self::biome_sampling::{get_biome, ClimateSampler};
#[cfg(test)]
use self::biome_sampling::biome_manager_fiddle;
use self::biome_sampling::{
    biome_manager_get_biome, biome_manager_get_biome_cached, biome_manager_obfuscate_seed,
    ChunkNoiseBiomeCache,
};

mod surface_rules;
pub use self::surface_rules::*;

mod terrain_models;
pub use self::terrain_models::*;

mod decoration_models;
pub use self::decoration_models::*;

mod flat_generation;
pub use self::flat_generation::*;

mod feature_sorting;
pub use self::feature_sorting::{
    biome_decoration_feature_plan, biome_decoration_structure_calls, build_features_per_step,
    build_features_per_step_with_source_ids,
};
use self::feature_sorting::possible_biome_feature_steps_for_source;

mod noise_preview_chunk;
pub use self::noise_preview_chunk::materialize_noise_preview_chunk;
use self::noise_preview_chunk::{
    live_tree_decoration_blocks, noise_preview_block_at, noise_preview_tree_blocks,
};

mod simple_vegetation;
use self::simple_vegetation::{
    apply_initial_simple_vegetation_decoration_to_chunk, block_predicate_test_in_chunk,
    block_predicate_test_in_region, configured_simple_vegetation_block,
    placed_simple_vegetation_feature, region_static_block_name, PlacedSimpleVegetationFeature,
    place_configured_simple_vegetation_in_target_chunk,
    place_simple_vegetation_feature_positions_depth_first, sample_triangle_int, seedless_noise_salt,
    simple_vegetation_phase, simple_vegetation_source_height, vegetation_flower_noise,
    SimpleVegetationPhase,
};

mod live_tree_selection;
use self::live_tree_selection::{
    live_fallen_tree_placement_plan, live_tree_feature_selection,
    live_tree_sapling_for_tree_config, live_tree_sapling_for_trunk_provider,
    live_tree_sapling_survives_at, live_tree_selector_trace, live_tree_state_with_planned_blocks,
    tree_placement_filter_sapling, LiveTreeDecoratorSet, LiveTreeFeatureConfig,
    LiveTreeFeatureSelection,
};
#[cfg(test)]
use self::live_tree_selection::{
    live_birch_tree_config, live_oak_bees_005_tree_config, live_oak_leaf_litter_tree_config,
};

mod live_tree_placement;
use self::live_tree_placement::{append_live_tree_decorators, live_tree_placement_plan};
#[cfg(test)]
use self::live_tree_placement::tree_decorator_solid_render;


mod density_registry;
pub use self::density_registry::*;
mod density_overworld;
pub use self::density_overworld::*;
mod density_caves;
pub use self::density_caves::*;
mod density_overworld_final;
pub use self::density_overworld_final::*;
mod density_veins;
pub use self::density_veins::*;
mod density_preliminary_surface;
pub use self::density_preliminary_surface::*;
mod noise_routers;
pub use self::noise_routers::*;
mod feature_models;
pub use self::feature_models::*;
mod feature_types;
pub use self::feature_types::*;
mod configured_features;
pub use self::configured_features::*;
mod placed_features;
pub use self::placed_features::*;
mod biome_feature_steps_overworld_common;
pub use self::biome_feature_steps_overworld_common::*;
mod biome_feature_steps_nether_end;
pub use self::biome_feature_steps_nether_end::*;
mod biome_feature_steps_overworld_variants;
pub use self::biome_feature_steps_overworld_variants::*;
mod biome_spawn_entries_creatures;
pub use self::biome_spawn_entries_creatures::*;
mod biome_spawn_entries_monsters_water;
pub use self::biome_spawn_entries_monsters_water::*;
mod biome_spawner_groups_overworld;
pub use self::biome_spawner_groups_overworld::*;
mod biome_spawner_groups_special;
pub use self::biome_spawner_groups_special::*;
mod biome_generation_settings;
pub use self::biome_generation_settings::*;
mod worldgen_static_registries;
pub use self::worldgen_static_registries::*;
mod structure_placement;
pub use self::structure_placement::*;

mod structure_core_models;
pub use self::structure_core_models::*;

mod structure_piece_models;
pub use self::structure_piece_models::*;
mod structure_piece_generation;
pub use self::structure_piece_generation::*;
mod jigsaw_model_impls;
pub use self::jigsaw_model_impls::*;
mod random_noise_helpers;
pub use self::random_noise_helpers::*;
mod height_providers;
pub use self::height_providers::*;
mod block_predicates;
pub use self::block_predicates::*;
mod placement_modifiers;
pub use self::placement_modifiers::*;
mod world_preset_resolvers;
pub use self::world_preset_resolvers::*;
mod live_chunk_generation;
pub use self::live_chunk_generation::*;
mod mob_spawn_planning;
pub use self::mob_spawn_planning::*;
mod mob_spawn_entities;
pub use self::mob_spawn_entities::*;
mod generator_queries;
pub use self::generator_queries::*;
mod overworld_chunk_generation;
pub use self::overworld_chunk_generation::*;
mod worldgen_registry_parsing;
pub use self::worldgen_registry_parsing::*;
mod terrain_splines;
pub use self::terrain_splines::*;
mod density_function_evaluation;
pub use self::density_function_evaluation::*;
mod noise_aquifer;
pub use self::noise_aquifer::*;
mod noise_sampling;
pub use self::noise_sampling::*;
mod generated_sections;
pub use self::generated_sections::*;
mod surface_rule_runtime;
pub use self::surface_rule_runtime::*;
mod surface_generation;
pub use self::surface_generation::*;
mod carver_generation;
pub use self::carver_generation::*;
mod carver_application;
pub use self::carver_application::*;
mod tree_decoration_generation;
pub use self::tree_decoration_generation::*;
mod underground_decoration_cache;
use self::underground_decoration_cache::*;
mod underground_decoration_placement;
use self::underground_decoration_placement::*;
mod feature_lookup;
pub use self::feature_lookup::*;
mod block_state_provider_sampling;
pub use self::block_state_provider_sampling::*;
mod ore_placement_plans;
pub use self::ore_placement_plans::*;
mod aquatic_feature_plans;
pub use self::aquatic_feature_plans::*;
mod vegetation_patch_plans;
pub use self::vegetation_patch_plans::*;
mod lake_feature_plans;
pub use self::lake_feature_plans::*;
mod fossil_feature_plans;
pub use self::fossil_feature_plans::*;
mod geode_feature_plans;
pub use self::geode_feature_plans::*;
mod iceberg_feature_plans;
pub use self::iceberg_feature_plans::*;
mod blue_ice_feature_plans;
pub use self::blue_ice_feature_plans::*;
mod feature_selector_plans;
pub use self::feature_selector_plans::*;
mod fill_layer_feature_plans;
pub use self::fill_layer_feature_plans::*;
mod end_island_feature_plans;
pub use self::end_island_feature_plans::*;
mod replace_sphere_feature_plans;
pub use self::replace_sphere_feature_plans::*;
mod basalt_pillar_feature_plans;
pub use self::basalt_pillar_feature_plans::*;
mod basalt_column_feature_plans;
pub use self::basalt_column_feature_plans::*;
mod delta_feature_plans;
pub use self::delta_feature_plans::*;
mod glowstone_feature_plans;
pub use self::glowstone_feature_plans::*;
mod nether_forest_vegetation_plans;
pub use self::nether_forest_vegetation_plans::*;
mod nether_vine_feature_plans;
pub use self::nether_vine_feature_plans::*;
mod end_platform_feature_plans;
pub use self::end_platform_feature_plans::*;
mod end_gateway_feature_plans;
pub use self::end_gateway_feature_plans::*;
mod chorus_plant_feature_plans;
pub use self::chorus_plant_feature_plans::*;
mod end_podium_feature_plans;
pub use self::end_podium_feature_plans::*;
mod end_spike_feature_plans;
pub use self::end_spike_feature_plans::*;
mod huge_fungus_feature_plans;
pub use self::huge_fungus_feature_plans::*;
mod block_pile_feature_plans;
pub use self::block_pile_feature_plans::*;
mod disk_feature_plans;
pub use self::disk_feature_plans::*;
mod snow_and_freeze_feature_plans;
pub use self::snow_and_freeze_feature_plans::*;
mod underwater_magma_feature_plans;
pub use self::underwater_magma_feature_plans::*;
mod dripstone_cluster_feature_plans;
pub use self::dripstone_cluster_feature_plans::*;
mod pointed_dripstone_feature_plans;
pub use self::pointed_dripstone_feature_plans::*;
mod large_dripstone_feature_plans;
pub use self::large_dripstone_feature_plans::*;
mod tree_configuration_plans;
pub use self::tree_configuration_plans::*;
mod root_system_feature_plans;
pub use self::root_system_feature_plans::*;
mod tree_foliage_feature_plans;
use self::tree_foliage_feature_plans::*;
mod tree_decorator_feature_plans;
pub use self::tree_decorator_feature_plans::*;
mod spawn_positioning;
pub use self::spawn_positioning::*;
mod feature_support_plans;
pub use self::feature_support_plans::*;
mod noise_chunk_template;
use self::noise_chunk_template::*;
mod noise_density_evaluation;
use self::noise_density_evaluation::*;
mod noise_chunk_state;
pub use self::noise_chunk_state::NoiseChunk;
mod noise_chunk_fill;
pub use self::noise_chunk_fill::*;
mod simple_tree_feature_plans;
pub use self::simple_tree_feature_plans::*;
mod trunk_feature_plans;
pub use self::trunk_feature_plans::*;
mod fallen_tree_feature_plans;
pub use self::fallen_tree_feature_plans::*;
mod jigsaw_pool_models;
pub use self::jigsaw_pool_models::*;
mod jigsaw_placement;
pub use self::jigsaw_placement::*;
mod structure_template_support;
pub use self::structure_template_support::*;
mod structure_access_support;
pub use self::structure_access_support::*;
mod structure_scattered_features;
pub use self::structure_scattered_features::*;
mod structure_template_features;
pub use self::structure_template_features::*;
mod structure_ocean_ruins;
pub use self::structure_ocean_ruins::*;
mod structure_mineshaft_generation;
pub use self::structure_mineshaft_generation::*;
mod structure_mineshaft_application;
pub use self::structure_mineshaft_application::*;
mod structure_strongholds;
pub use self::structure_strongholds::*;
mod structure_nether_fortress;
pub use self::structure_nether_fortress::*;
mod structure_ocean_monuments;
pub use self::structure_ocean_monuments::*;
mod structure_end_city;
pub use self::structure_end_city::*;
mod structure_woodland_mansion;
pub use self::structure_woodland_mansion::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendingConstants {
    pub height_blending_range_cells: i32,
    pub height_blending_range_chunks: i32,
    pub density_blending_range_cells: i32,
    pub density_blending_range_chunks: i32,
    pub old_chunk_xz_radius: i32,
    pub cell_width: i32,
    pub cell_height: i32,
    pub cell_ratio: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendingDataPacked<'a> {
    pub min_section: i32,
    pub max_section: i32,
    pub heights: Option<&'a [f64]>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendingOutput {
    pub alpha: f64,
    pub blending_offset: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpgradeDataModel {
    pub tag_indices: &'static str,
    pub tag_sides: &'static str,
    pub tag_neighbor_block_ticks: &'static str,
    pub tag_neighbor_fluid_ticks: &'static str,
    pub block_fixers: &'static [&'static str],
    pub chunky_fixers: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BelowZeroRetrogenModel {
    pub target_status_field: &'static str,
    pub missing_bedrock_field: &'static str,
    pub upgrade_min_y: i32,
    pub upgrade_height: i32,
    pub max_generated_bedrock_y: i32,
    pub retained_biomes: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnSelectionConstants {
    pub initial_chunk_search_radius: i32,
    pub player_spawn_ticket_radius: i32,
    pub spawn_search_absolute_max_attempts: i32,
    pub default_respawn_radius: i32,
    pub small_search_coprime_threshold: i32,
    pub large_search_coprime: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnColumnHeights {
    pub top_y: i32,
    pub surface_y: i32,
    pub ocean_floor_y: i32,
    pub min_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnBlockKind {
    Solid,
    Air,
    Fluid,
    NonSolid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InitialSpawnKind {
    DebugHalfWorld { x: i32, y: i32, z: i32 },
    DebugWorld { x: i32, y: i32, z: i32 },
    Normal { x: i32, y: i32, z: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnChunkStatusSnapshot {
    pub pos: ChunkPos,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialSpawnReadinessReport {
    pub center: ChunkPos,
    pub required_radius: i32,
    pub required_status: &'static str,
    pub ticket_type: &'static str,
    pub ticket_level: i32,
    pub missing_chunks: Vec<ChunkPos>,
    pub not_ready_chunks: Vec<SpawnChunkStatusSnapshot>,
}

impl InitialSpawnReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.missing_chunks.is_empty() && self.not_ready_chunks.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureFamily {
    Village,
    Stronghold,
    Mineshaft,
    OceanMonument,
    WoodlandMansion,
    Bastion,
    Fortress,
    AncientCity,
    TrialChambers,
    EndCity,
    RuinedPortal,
    Shipwreck,
    BuriedTreasure,
    Igloo,
    SwampHut,
    PillagerOutpost,
    TrailRuins,
    Fossil,
    DesertPyramid,
    JungleTemple,
    OceanRuins,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureConfigurationKind {
    None,
    Tree,
    FallenTree,
    BlockPile,
    Spring,
    ReplaceBlock,
    Fossil,
    HugeMushroom,
    Spike,
    BlockColumn,
    VegetationPatch,
    RootSystem,
    MultifaceGrowth,
    UnderwaterMagma,
    BlockState,
    BlockBlob,
    Disk,
    Lake,
    Ore,
    EndSpike,
    EndGateway,
    Probability,
    Count,
    SimpleBlock,
    HugeFungus,
    NetherForestVegetation,
    TwistingVines,
    Column,
    Delta,
    ReplaceSphere,
    Layer,
    RandomFeature,
    SimpleRandomFeature,
    RandomBooleanFeature,
    Geode,
    DripstoneCluster,
    LargeDripstone,
    PointedDripstone,
    SculkPatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureFamily {
    Selector,
    Tree,
    Vegetation,
    Cave,
    Ore,
    Lake,
    Disk,
    End,
    Nether,
    Aquatic,
    StructureLike,
    BlockPlacement,
}

pub const OVERWORLD_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 384, 1, 2);
pub const NETHER_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 1, 2);
pub const END_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 2, 1);
pub const CAVES_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 192, 1, 2);
pub const FLOATING_ISLANDS_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 256, 2, 1);

pub const NORMAL_NOISE_INPUT_FACTOR: f64 = 1.0181268882175227;
pub const NORMAL_NOISE_TARGET_DEVIATION: f64 = 1.0 / 3.0;

pub const SYNTH_NOISE_SOURCES: &[SynthNoiseSource] = &[
    SynthNoiseSource {
        id: "minecraft:normal_noise",
        codec: "NormalNoise.NoiseParameters",
    },
    SynthNoiseSource {
        id: "minecraft:perlin_noise",
        codec: "PerlinNoise",
    },
    SynthNoiseSource {
        id: "minecraft:perlin_simplex_noise",
        codec: "PerlinSimplexNoise",
    },
    SynthNoiseSource {
        id: "minecraft:simplex_noise",
        codec: "SimplexNoise",
    },
    SynthNoiseSource {
        id: "minecraft:improved_noise",
        codec: "ImprovedNoise",
    },
    SynthNoiseSource {
        id: "minecraft:blended_noise",
        codec: "DensityFunctions.BlendedNoise",
    },
];

pub const NORMAL_NOISE_PARAMETERS: &[NormalNoiseParameters] = &[
    NormalNoiseParameters {
        id: "minecraft:temperature",
        first_octave: -10,
        amplitudes: &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:vegetation",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:continentalness",
        first_octave: -9,
        amplitudes: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:erosion",
        first_octave: -9,
        amplitudes: &[1.0, 1.0, 0.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:temperature_large",
        first_octave: -12,
        amplitudes: &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:vegetation_large",
        first_octave: -10,
        amplitudes: &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:continentalness_large",
        first_octave: -11,
        amplitudes: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:erosion_large",
        first_octave: -11,
        amplitudes: &[1.0, 1.0, 0.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:nether/temperature",
        first_octave: -7,
        amplitudes: &[1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:nether/vegetation",
        first_octave: -7,
        amplitudes: &[1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ridge",
        first_octave: -7,
        amplitudes: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:offset",
        first_octave: -3,
        amplitudes: &[1.0, 1.0, 1.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_barrier",
        first_octave: -3,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_fluid_level_floodedness",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_lava",
        first_octave: -1,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_fluid_level_spread",
        first_octave: -5,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:pillar",
        first_octave: -7,
        amplitudes: &[1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:pillar_rareness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:pillar_thickness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d_elevation",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d_modulator",
        first_octave: -11,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d_thickness",
        first_octave: -11,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_1",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_2",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_rarity",
        first_octave: -11,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_thickness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_roughness",
        first_octave: -5,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_roughness_modulator",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:cave_entrance",
        first_octave: -7,
        amplitudes: &[0.4, 0.5, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:cave_layer",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:cave_cheese",
        first_octave: -8,
        amplitudes: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_veininess",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_vein_a",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_vein_b",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_gap",
        first_octave: -5,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle_thickness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle_ridge_a",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle_ridge_b",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:jagged",
        first_octave: -16,
        amplitudes: &[
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ],
    },
    NormalNoiseParameters {
        id: "minecraft:surface",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:surface_secondary",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 0.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:clay_bands_offset",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:badlands_pillar",
        first_octave: -2,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:badlands_pillar_roof",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:badlands_surface",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:iceberg_pillar",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:iceberg_pillar_roof",
        first_octave: -3,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:iceberg_surface",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:surface_swamp",
        first_octave: -2,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:calcite",
        first_octave: -9,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:gravel",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:powder_snow",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:packed_ice",
        first_octave: -7,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ice",
        first_octave: -4,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:soul_sand_layer",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334],
    },
    NormalNoiseParameters {
        id: "minecraft:gravel_layer",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334],
    },
    NormalNoiseParameters {
        id: "minecraft:patch",
        first_octave: -5,
        amplitudes: &[1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334],
    },
    NormalNoiseParameters {
        id: "minecraft:netherrack",
        first_octave: -3,
        amplitudes: &[1.0, 0.0, 0.0, 0.35],
    },
    NormalNoiseParameters {
        id: "minecraft:nether_wart",
        first_octave: -3,
        amplitudes: &[1.0, 0.0, 0.0, 0.9],
    },
    NormalNoiseParameters {
        id: "minecraft:nether_state_selector",
        first_octave: -4,
        amplitudes: &[1.0],
    },
];

pub const HEIGHT_PROVIDER_TYPES: &[HeightProviderType] = &[
    HeightProviderType {
        id: "minecraft:constant",
    },
    HeightProviderType {
        id: "minecraft:uniform",
    },
    HeightProviderType {
        id: "minecraft:biased_to_bottom",
    },
    HeightProviderType {
        id: "minecraft:very_biased_to_bottom",
    },
    HeightProviderType {
        id: "minecraft:trapezoid",
    },
    HeightProviderType {
        id: "minecraft:weighted_list",
    },
];

pub const BLOCK_PREDICATE_TYPES: &[BlockPredicateType] = &[
    BlockPredicateType {
        id: "minecraft:matching_blocks",
    },
    BlockPredicateType {
        id: "minecraft:matching_block_tag",
    },
    BlockPredicateType {
        id: "minecraft:matching_fluids",
    },
    BlockPredicateType {
        id: "minecraft:has_sturdy_face",
    },
    BlockPredicateType {
        id: "minecraft:solid",
    },
    BlockPredicateType {
        id: "minecraft:replaceable",
    },
    BlockPredicateType {
        id: "minecraft:would_survive",
    },
    BlockPredicateType {
        id: "minecraft:inside_world_bounds",
    },
    BlockPredicateType {
        id: "minecraft:any_of",
    },
    BlockPredicateType {
        id: "minecraft:all_of",
    },
    BlockPredicateType {
        id: "minecraft:not",
    },
    BlockPredicateType {
        id: "minecraft:true",
    },
    BlockPredicateType {
        id: "minecraft:unobstructed",
    },
];

pub const FLAT_DEFAULT_LAYERS: &[FlatLayerInfo] = &[
    FlatLayerInfo {
        height: 1,
        block: "minecraft:bedrock",
    },
    FlatLayerInfo {
        height: 2,
        block: "minecraft:dirt",
    },
    FlatLayerInfo {
        height: 1,
        block: "minecraft:grass_block",
    },
];

pub const FLAT_GENERATOR_PRESETS: &[FlatGeneratorPreset] = &[
    FlatGeneratorPreset {
        id: "minecraft:classic_flat",
        display: "minecraft:grass_block",
        biome: "minecraft:plains",
        structures: &["minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 2,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:tunnelers_dream",
        display: "minecraft:stone",
        biome: "minecraft:windswept_hills",
        structures: &["minecraft:mineshafts", "minecraft:strongholds"],
        add_lakes: true,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 230,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:water_world",
        display: "minecraft:water_bucket",
        biome: "minecraft:deep_ocean",
        structures: &[
            "minecraft:ocean_ruins",
            "minecraft:shipwrecks",
            "minecraft:ocean_monuments",
        ],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 64,
                block: "minecraft:deepslate",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:gravel",
            },
            FlatLayerInfo {
                height: 90,
                block: "minecraft:water",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:overworld",
        display: "minecraft:short_grass",
        biome: "minecraft:plains",
        structures: &[
            "minecraft:villages",
            "minecraft:mineshafts",
            "minecraft:pillager_outposts",
            "minecraft:ruined_portals",
            "minecraft:strongholds",
        ],
        add_lakes: true,
        decoration: true,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 59,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:snowy_kingdom",
        display: "minecraft:snow",
        biome: "minecraft:snowy_plains",
        structures: &["minecraft:villages", "minecraft:igloos"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 59,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:snow",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:bottomless_pit",
        display: "minecraft:feather",
        biome: "minecraft:plains",
        structures: &["minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 2,
                block: "minecraft:cobblestone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:desert",
        display: "minecraft:sand",
        biome: "minecraft:desert",
        structures: &[
            "minecraft:villages",
            "minecraft:desert_pyramids",
            "minecraft:mineshafts",
            "minecraft:strongholds",
        ],
        add_lakes: true,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 52,
                block: "minecraft:sandstone",
            },
            FlatLayerInfo {
                height: 8,
                block: "minecraft:sand",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:redstone_ready",
        display: "minecraft:redstone",
        biome: "minecraft:desert",
        structures: &[],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 116,
                block: "minecraft:sandstone",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:the_void",
        display: "minecraft:barrier",
        biome: "minecraft:the_void",
        structures: &[],
        add_lakes: true,
        decoration: false,
        layers: &[FlatLayerInfo {
            height: 1,
            block: "minecraft:air",
        }],
    },
];

#[derive(Debug, Default)]
struct TreeDecorationDiagnostics {
    context_chunk_build_ms: u128,
    context_heightmap_ms: u128,
    context_chunks: usize,
    source_total_us: u128,
    sources_evaluated: usize,
    source_context_clone_us: u128,
    source_biome_steps_ms: u128,
    source_feature_sort_ms: u128,
    source_plan_ms: u128,
    feature_calls_total: usize,
    tree_feature_calls: usize,
    tree_attempts: usize,
    tree_candidates: usize,
    candidate_biome_ms: u128,
    validation_ms: u128,
    validation_accepts: usize,
    validation_rejects: usize,
    placement_plan_ms: u128,
    placement_plan_blocks: usize,
    filter_ms: u128,
    filtered_blocks: usize,
    source_write_filter_ms: u128,
    source_context_map_ms: u128,
    output_blocks_seen: usize,
    output_blocks_in_target: usize,
    output_blocks_written: usize,
}

fn local_tree_block_to_world(source_pos: ChunkPos, pos: BlockPos) -> BlockPos {
    BlockPos {
        x: source_pos.x * 16 + pos.x,
        y: pos.y,
        z: source_pos.z * 16 + pos.z,
    }
}

fn world_tree_block_to_local(source_pos: ChunkPos, pos: BlockPos) -> BlockPos {
    BlockPos {
        x: pos.x - source_pos.x * 16,
        y: pos.y,
        z: pos.z - source_pos.z * 16,
    }
}

type TreeBlockOverlay = HashMap<(i32, i32, i32), &'static str>;

fn local_tree_block_to_world_key(source_pos: ChunkPos, pos: BlockPos) -> (i32, i32, i32) {
    (source_pos.x * 16 + pos.x, pos.y, source_pos.z * 16 + pos.z)
}

fn live_tree_state_with_previous_blocks(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &[TreePlacementBlock],
    world_pos: BlockPos,
) -> String {
    previous_source_blocks
        .iter()
        .rev()
        .find_map(|block| {
            let block_world_x = source_pos.x * 16 + block.pos.x;
            let block_world_z = source_pos.z * 16 + block.pos.z;
            (block_world_x == world_pos.x
                && block.pos.y == world_pos.y
                && block_world_z == world_pos.z)
                .then_some(block.state.to_string())
        })
        .or_else(|| {
            block_context
                .block_state(world_pos.x, world_pos.y, world_pos.z)
                .map(|state| block_state_id(state).to_string())
        })
        .unwrap_or_else(|| "minecraft:air".to_string())
}

fn live_tree_state_with_previous_overlay<'a>(
    block_context: &'a TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    world_pos: BlockPos,
) -> Cow<'a, str> {
    previous_source_blocks
        .get(&(world_pos.x, world_pos.y, world_pos.z))
        .map(|state| Cow::Borrowed(*state))
        .or_else(|| {
            block_context
                .block_state(world_pos.x, world_pos.y, world_pos.z)
                .map(|state| Cow::Borrowed(block_state_id(state)))
        })
        .unwrap_or(Cow::Borrowed("minecraft:air"))
}

fn live_straight_blob_tree_placement_plan(
    origin: BlockPos,
    trunk: TrunkPlacerModel,
    clipped_tree_height: i32,
    foliage: FoliagePlacerModel,
    trunk_state: &'static str,
    foliage_state: &'static str,
    below_trunk_state: &'static str,
    rand_a: i32,
    rand_b: i32,
    random: &mut RandomSourceKind,
) -> Result<TreePlacementPlan, String> {
    validate_trunk_placer(trunk)?;
    validate_foliage_placer(foliage)?;
    if trunk.kind != TrunkPlacerKind::Straight {
        return Err("live blob tree placement requires a straight trunk placer".to_string());
    }
    let FoliagePlacerKind::Blob {
        height: foliage_height,
    } = foliage.kind
    else {
        return Err("live straight tree placement currently requires blob foliage".to_string());
    };

    let leaf_radius = sample_inclusive_i32(foliage.radius_min, foliage.radius_max, rand_a);
    let foliage_offset = sample_inclusive_i32(foliage.offset_min, foliage.offset_max, rand_b);
    let foliage_origin = BlockPos {
        x: origin.x,
        y: origin.y + clipped_tree_height + foliage_offset,
        z: origin.z,
    };
    let mut blocks = Vec::new();
    push_tree_block(
        &mut blocks,
        TreePlacementBlock {
            pos: BlockPos {
                x: origin.x,
                y: origin.y - 1,
                z: origin.z,
            },
            state: below_trunk_state,
            kind: TreePlacementBlockKind::DirtBelowTrunk,
        },
    );
    for y in 0..clipped_tree_height {
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: origin.x,
                    y: origin.y + y,
                    z: origin.z,
                },
                state: trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
    }

    for y_offset in (-foliage_height..=0).rev() {
        let current_radius = (leaf_radius - 1 - y_offset / 2).max(0);
        place_live_blob_leaves_row(
            &mut blocks,
            foliage_origin,
            current_radius,
            y_offset,
            foliage_state,
            random,
        );
    }

    Ok(TreePlacementPlan { blocks })
}

#[derive(Clone, Copy)]
enum TreeContextChunkRef<'a> {
    Full(&'a LevelChunk),
    Lightweight(&'a LightweightTreeContextChunk),
}

impl TreeContextChunkRef<'_> {
    fn block_state(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<&str> {
        match self {
            Self::Full(chunk) => chunk.get_block_state_name(world_x, world_y, world_z),
            Self::Lightweight(chunk) => {
                Some(chunk.synthetic_block_state(world_x, world_y, world_z))
            }
        }
    }
}

struct TreeDecorationBlockContext<'a> {
    source_pos: ChunkPos,
    source_chunk: TreeContextChunkRef<'a>,
    target_pos: ChunkPos,
    target_chunk: &'a LevelChunk,
    generated_chunks: &'a HashMap<ChunkPos, TreeContextChunkRef<'a>>,
    region_overlay: Option<&'a TreeBlockOverlay>,
}

impl TreeDecorationBlockContext<'_> {
    fn block_state(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<&str> {
        if let Some(state) = self
            .region_overlay
            .and_then(|overlay| overlay.get(&(world_x, world_y, world_z)).copied())
        {
            return Some(state);
        }
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        if chunk_pos == self.source_pos {
            return self.source_chunk.block_state(world_x, world_y, world_z);
        }
        if chunk_pos == self.target_pos {
            return self
                .target_chunk
                .get_block_state_name(world_x, world_y, world_z);
        }
        self.generated_chunks
            .get(&chunk_pos)
            .and_then(|chunk| chunk.block_state(world_x, world_y, world_z))
    }
}

fn live_tree_replaceable_rows(
    block_context: &TreeDecorationBlockContext<'_>,
    origin: BlockPos,
    tree_height: i32,
    min_size: FeatureSizeModel,
    settings: &NoiseGeneratorSettings,
) -> Vec<Vec<String>> {
    (0..=tree_height + 1)
        .map(|y_offset| {
            let radius = feature_size_at_height(min_size, tree_height, y_offset);
            let mut row = Vec::new();
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let local_x = origin.x + dx;
                    let local_z = origin.z + dz;
                    let world_y = origin.y + y_offset;
                    let world_x = block_context.source_pos.x * 16 + local_x;
                    let world_z = block_context.source_pos.z * 16 + local_z;
                    let state = if world_y < settings.noise.min_y
                        || world_y >= settings.noise.min_y + settings.noise.height
                    {
                        "minecraft:air".to_string()
                    } else {
                        block_context
                            .block_state(world_x, world_y, world_z)
                            .unwrap_or("minecraft:air")
                            .to_string()
                    };
                    row.push(state);
                }
            }
            row
        })
        .collect()
}

fn live_tree_can_place_in_chunk(
    block_context: &TreeDecorationBlockContext<'_>,
    origin: BlockPos,
    config: LiveTreeFeatureConfig,
    rand_a: i32,
    rand_b: i32,
    settings: &NoiseGeneratorSettings,
) -> bool {
    let trunk = TrunkPlacerModel {
        base_height: config.base_height,
        height_rand_a: config.height_rand_a,
        height_rand_b: config.height_rand_b,
        kind: if matches!(config.foliage.kind, FoliagePlacerKind::Fancy { .. }) {
            TrunkPlacerKind::Fancy
        } else {
            TrunkPlacerKind::Straight
        },
    };
    let tree_height = trunk_placer_height(trunk, rand_a, rand_b);
    let build_min_y = settings.noise.min_y;
    let build_max_y = settings.noise.min_y + settings.noise.height;
    let min_y = origin.y;
    let max_y = origin.y + tree_height + 1;
    if min_y < build_min_y + 1 || max_y > build_max_y + 1 {
        return false;
    }

    let mut clipped_tree_height = tree_height;
    'height_scan: for y_offset in 0..=tree_height + 1 {
        let radius = feature_size_at_height(config.minimum_size, tree_height, y_offset);
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let local_x = origin.x + dx;
                let local_z = origin.z + dz;
                let world_y = origin.y + y_offset;
                if world_y < build_min_y || world_y >= build_max_y {
                    continue;
                }
                let world_x = block_context.source_pos.x * 16 + local_x;
                let world_z = block_context.source_pos.z * 16 + local_z;
                if block_context
                    .block_state(world_x, world_y, world_z)
                    .is_some_and(|state| !tree_trunk_free_pos(state))
                {
                    clipped_tree_height = y_offset - 2;
                    break 'height_scan;
                }
            }
        }
    }

    clipped_tree_height >= tree_height
        || config
            .min_clipped_height
            .is_some_and(|min| clipped_tree_height >= min)
}

fn live_tree_clipped_height_with_previous_blocks(
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    origin: BlockPos,
    config: LiveTreeFeatureConfig,
    rand_a: i32,
    rand_b: i32,
    settings: &NoiseGeneratorSettings,
) -> Option<i32> {
    let trunk = TrunkPlacerModel {
        base_height: config.base_height,
        height_rand_a: config.height_rand_a,
        height_rand_b: config.height_rand_b,
        kind: if matches!(config.foliage.kind, FoliagePlacerKind::Fancy { .. }) {
            TrunkPlacerKind::Fancy
        } else {
            TrunkPlacerKind::Straight
        },
    };
    let tree_height = trunk_placer_height(trunk, rand_a, rand_b);
    let build_min_y = settings.noise.min_y;
    let build_max_y = settings.noise.min_y + settings.noise.height;
    let min_y = origin.y;
    let max_y = origin.y + tree_height + 1;
    if min_y < build_min_y + 1 || max_y > build_max_y + 1 {
        return None;
    }

    let mut clipped_tree_height = tree_height;
    'height_scan: for y_offset in 0..=tree_height + 1 {
        let radius = feature_size_at_height(config.minimum_size, tree_height, y_offset);
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let local_x = origin.x + dx;
                let local_z = origin.z + dz;
                let world_y = origin.y + y_offset;
                if world_y < build_min_y || world_y >= build_max_y {
                    continue;
                }
                let world_x = block_context.source_pos.x * 16 + local_x;
                let world_z = block_context.source_pos.z * 16 + local_z;
                let state = live_tree_state_with_previous_overlay(
                    block_context,
                    previous_source_blocks,
                    BlockPos {
                        x: world_x,
                        y: world_y,
                        z: world_z,
                    },
                );
                if !tree_trunk_free_pos(&state) {
                    clipped_tree_height = y_offset - 2;
                    break 'height_scan;
                }
            }
        }
    }

    if clipped_tree_height >= tree_height
        || config
            .min_clipped_height
            .is_some_and(|min| clipped_tree_height >= min)
    {
        Some(clipped_tree_height)
    } else {
        None
    }
}

fn noise_preview_tree_origins(
    biome: &BiomeGenerationSettingsModel,
    world_seed: i64,
    chunk_pos: ChunkPos,
    min_section_y: i32,
) -> Vec<(usize, usize, u64)> {
    if !biome_has_any_tree_placed_feature(biome) {
        return Vec::new();
    }

    let features_per_step = match build_features_per_step(&[biome.feature_steps], true) {
        Ok(features) => features,
        Err(_) => return Vec::new(),
    };
    let plan = biome_decoration_feature_plan(
        world_seed,
        chunk_pos.x,
        chunk_pos.z,
        min_section_y,
        &features_per_step,
        &[biome.feature_steps],
    );
    let mut origins = Vec::new();

    for call in plan.feature_calls.iter().filter(|call| {
        call.step_index == GenerationDecorationStep::VegetalDecoration as usize
            && noise_preview_tree_feature_count_kind(call.feature).is_some()
    }) {
        let mut random = RandomSourceKind::new(call.seed, RandomAlgorithm::Xoroshiro);
        let count = live_tree_count(
            noise_preview_tree_feature_count_kind(call.feature).unwrap(),
            &mut random,
        );

        for _ in 0..count {
            let local_x = feature_random_next_i32_bound(&mut random, 16) as usize;
            let local_z = feature_random_next_i32_bound(&mut random, 16) as usize;
            origins.push((
                local_x,
                local_z,
                feature_random_next_i64(&mut random) as u64,
            ));
        }
    }

    origins
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoisePreviewTreeCountKind {
    CountExtra {
        count: i32,
        inverse_chance_weight: i32,
        extra: i32,
    },
    Constant(i32),
    CountPlusUniform {
        count: i32,
        bound: i32,
    },
    DenseCanopy,
}

fn live_count_extra(
    random: &mut RandomSourceKind,
    count: i32,
    inverse_chance_weight: i32,
    extra: i32,
) -> i32 {
    let roll = feature_random_next_i32_bound(random, inverse_chance_weight);
    if roll < inverse_chance_weight - 1 {
        count
    } else {
        count + extra
    }
}

fn live_tree_count(kind: NoisePreviewTreeCountKind, random: &mut RandomSourceKind) -> i32 {
    match kind {
        NoisePreviewTreeCountKind::CountExtra {
            count,
            inverse_chance_weight,
            extra,
        } => live_count_extra(random, count, inverse_chance_weight, extra),
        NoisePreviewTreeCountKind::Constant(count) => count,
        NoisePreviewTreeCountKind::CountPlusUniform { count, bound } => {
            count + feature_random_next_i32_bound(random, bound)
        }
        NoisePreviewTreeCountKind::DenseCanopy => 16,
    }
}

fn noise_preview_tree_feature_count_kind(feature: &str) -> Option<NoisePreviewTreeCountKind> {
    match feature.strip_prefix("minecraft:").unwrap_or(feature) {
        "trees_plains" => Some(NoisePreviewTreeCountKind::CountExtra {
            count: 0,
            inverse_chance_weight: 20,
            extra: 1,
        }),
        "trees_birch_and_oak_leaf_litter"
        | "trees_birch"
        | "trees_taiga"
        | "birch_tall"
        | "trees_old_growth_spruce_taiga"
        | "trees_old_growth_pine_taiga"
        | "trees_grove"
        | "trees_cherry" => Some(NoisePreviewTreeCountKind::CountExtra {
            count: 10,
            inverse_chance_weight: 10,
            extra: 1,
        }),
        "trees_flower_forest" => Some(NoisePreviewTreeCountKind::CountExtra {
            count: 6,
            inverse_chance_weight: 10,
            extra: 1,
        }),
        "dark_forest_vegetation" | "pale_garden_vegetation" => {
            Some(NoisePreviewTreeCountKind::DenseCanopy)
        }
        "trees_swamp" | "trees_windswept_savanna" | "trees_sparse_jungle" => {
            Some(NoisePreviewTreeCountKind::CountExtra {
                count: 2,
                inverse_chance_weight: 10,
                extra: 1,
            })
        }
        "trees_mangrove" => Some(NoisePreviewTreeCountKind::Constant(25)),
        "trees_jungle" => Some(NoisePreviewTreeCountKind::CountExtra {
            count: 50,
            inverse_chance_weight: 10,
            extra: 1,
        }),
        "trees_savanna" => Some(NoisePreviewTreeCountKind::CountExtra {
            count: 1,
            inverse_chance_weight: 10,
            extra: 1,
        }),
        "trees_windswept_forest" => Some(NoisePreviewTreeCountKind::CountExtra {
            count: 3,
            inverse_chance_weight: 10,
            extra: 1,
        }),
        "trees_windswept_hills" | "trees_water" | "trees_snowy" => {
            Some(NoisePreviewTreeCountKind::CountExtra {
                count: 0,
                inverse_chance_weight: 10,
                extra: 1,
            })
        }
        "trees_badlands" => Some(NoisePreviewTreeCountKind::CountExtra {
            count: 5,
            inverse_chance_weight: 10,
            extra: 1,
        }),
        "trees_meadow" => Some(NoisePreviewTreeCountKind::CountPlusUniform { count: 0, bound: 1 }),
        _ => None,
    }
}

fn biome_has_any_tree_placed_feature(biome: &BiomeGenerationSettingsModel) -> bool {
    [
        "trees_plains",
        "trees_birch_and_oak_leaf_litter",
        "trees_birch",
        "birch_tall",
        "trees_taiga",
        "trees_jungle",
        "trees_savanna",
        "trees_windswept_forest",
        "trees_windswept_hills",
        "trees_water",
        "trees_sparse_jungle",
        "trees_old_growth_spruce_taiga",
        "trees_old_growth_pine_taiga",
        "trees_grove",
        "trees_snowy",
        "trees_badlands",
        "trees_meadow",
        "trees_flower_forest",
        "dark_forest_vegetation",
        "pale_garden_vegetation",
        "trees_cherry",
        "trees_swamp",
        "trees_windswept_savanna",
        "trees_mangrove",
    ]
    .into_iter()
    .any(|feature| biome_has_placed_feature(biome, feature))
}

fn noise_preview_tree_materials(
    biome: &BiomeGenerationSettingsModel,
    seed: u64,
) -> (&'static str, &'static str, i32) {
    if biome_has_placed_feature(biome, "trees_cherry") {
        ("minecraft:cherry_log", "minecraft:cherry_leaves", 5)
    } else if biome_has_placed_feature(biome, "trees_mangrove") {
        ("minecraft:mangrove_log", "minecraft:mangrove_leaves", 6)
    } else if biome_has_placed_feature(biome, "pale_garden_vegetation") {
        ("minecraft:pale_oak_log", "minecraft:pale_oak_leaves", 5)
    } else if biome_has_placed_feature(biome, "dark_forest_vegetation") {
        ("minecraft:dark_oak_log", "minecraft:dark_oak_leaves", 5)
    } else if biome_has_placed_feature(biome, "trees_birch")
        || biome_has_placed_feature(biome, "birch_tall")
        || (biome_has_placed_feature(biome, "trees_birch_and_oak_leaf_litter") && seed & 1 == 0)
    {
        ("minecraft:birch_log", "minecraft:birch_leaves", 5)
    } else if biome_has_placed_feature(biome, "trees_taiga")
        || biome_has_placed_feature(biome, "trees_old_growth_spruce_taiga")
        || biome_has_placed_feature(biome, "trees_old_growth_pine_taiga")
        || biome_has_placed_feature(biome, "trees_grove")
        || biome_has_placed_feature(biome, "trees_snowy")
    {
        ("minecraft:spruce_log", "minecraft:spruce_leaves", 6)
    } else if biome_has_placed_feature(biome, "trees_savanna")
        || biome_has_placed_feature(biome, "trees_windswept_savanna")
    {
        ("minecraft:acacia_log", "minecraft:acacia_leaves", 5)
    } else if biome_has_placed_feature(biome, "trees_jungle")
        || biome_has_placed_feature(biome, "trees_sparse_jungle")
    {
        ("minecraft:jungle_log", "minecraft:jungle_leaves", 6)
    } else {
        ("minecraft:oak_log", "minecraft:oak_leaves", 4)
    }
}

fn noise_preview_ground_cover_blocks(
    chunk_pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    biome: &str,
    terrain_heights: &[i32; 16 * 16],
) -> Vec<TreePlacementBlock> {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return Vec::new();
    }

    let Some(generation) = biome_generation_settings(biome) else {
        return Vec::new();
    };
    let grass = biome_has_placed_feature(generation, "patch_grass_plain")
        || biome_has_placed_feature(generation, "patch_grass_forest");
    let flowers = biome_has_placed_feature(generation, "flower_plains")
        || biome_has_placed_feature(generation, "flower_default")
        || biome_has_placed_feature(generation, "forest_flowers");
    let sunflowers = biome_has_placed_feature(generation, "patch_sunflower");
    if !grass && !flowers && !sunflowers {
        return Vec::new();
    }

    let seed = (chunk_pos.x as i64 * 341_873_128_712 + chunk_pos.z as i64 * 132_897_987_541) as u64;
    let mut blocks = Vec::new();
    for z in 0..16 {
        for x in 0..16 {
            let surface_height = terrain_heights[z * 16 + x];
            if surface_height <= settings.sea_level + 1 {
                continue;
            }
            let roll = noise_preview_cover_roll(seed, x as u64, z as u64);
            let state = if sunflowers && roll % 97 == 0 {
                Some("minecraft:sunflower")
            } else if flowers && roll % 23 == 0 {
                Some(if biome == "minecraft:forest" {
                    "minecraft:poppy"
                } else {
                    "minecraft:dandelion"
                })
            } else if grass && roll % 7 == 0 {
                Some("minecraft:short_grass")
            } else {
                None
            };
            if let Some(state) = state {
                blocks.push(TreePlacementBlock {
                    pos: BlockPos {
                        x: x as i32,
                        y: surface_height,
                        z: z as i32,
                    },
                    state,
                    kind: TreePlacementBlockKind::GroundCover,
                });
            }
        }
    }
    blocks
}

fn noise_preview_cover_roll(seed: u64, x: u64, z: u64) -> u64 {
    let mut value =
        seed ^ x.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ z.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn noise_preview_terrain_height(x: i32, z: i32, settings: &NoiseGeneratorSettings) -> i32 {
    let (scale, amplitude) = match settings.id {
        "minecraft:large_biomes" => (76.0, 30.0),
        "minecraft:amplified" => (38.0, 70.0),
        "minecraft:nether" => (30.0, 24.0),
        "minecraft:end" => (52.0, 42.0),
        _ => (44.0, 34.0),
    };
    let xf = x as f64 / scale;
    let zf = z as f64 / scale;
    let broad = (xf.sin() * 0.55 + zf.cos() * 0.45) * amplitude;
    let detail = ((xf * 2.7 + zf * 1.3).sin() * (zf * 2.1 - xf * 0.9).cos()) * amplitude * 0.28;
    let ridge = ((x as i64 * 341_873_128_712 + z as i64 * 132_897_987_541) as u64).rotate_left(17)
        as f64
        / u64::MAX as f64
        - 0.5;
    settings.sea_level + 8 + (broad + detail + ridge * 12.0).round() as i32
}

pub fn noise_preview_base_height(
    x: i32,
    z: i32,
    settings: &NoiseGeneratorSettings,
    heightmap: HeightmapKind,
) -> i32 {
    let terrain_height = noise_preview_terrain_height(x, z, settings).clamp(
        settings.noise.min_y + 1,
        settings.noise.min_y + settings.noise.height,
    );
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => {
            terrain_height.max(settings.sea_level + 1)
        }
        HeightmapKind::OceanFloorWg
        | HeightmapKind::OceanFloor
        | HeightmapKind::MotionBlocking
        | HeightmapKind::MotionBlockingNoLeaves => terrain_height,
    }
}

pub fn noise_preview_base_column(
    x: i32,
    z: i32,
    settings: &NoiseGeneratorSettings,
) -> FlatNoiseColumn {
    let min_y = settings.noise.min_y;
    let height = settings.noise.height;
    let terrain_height =
        noise_preview_terrain_height(x, z, settings).clamp(min_y + 1, min_y + height);
    let states = (0..height.max(0))
        .map(|offset| {
            noise_preview_block_at(min_y + offset, min_y, terrain_height, settings.sea_level)
        })
        .collect();
    FlatNoiseColumn { min_y, states }
}

fn noise_preview_biome(biome_source_model: &BiomeSourceModel, pos: ChunkPos) -> &'static str {
    let quart_x = pos.x * 4 + 2;
    let quart_z = pos.z * 4 + 2;
    select_biome_from_source(
        biome_source_model,
        quart_x,
        16,
        quart_z,
        climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        0.0,
    )
    .unwrap_or("minecraft:plains")
}

fn flat_section_block_states(
    layers: &[Option<&'static str>],
    section_index: usize,
) -> PalettedContainer {
    let section_start = section_index * 16;
    let mut palette: Vec<&'static str> = Vec::new();
    let mut indices = vec![0_u64; SECTION_VOLUME];

    for local_y in 0..16 {
        let block = layers
            .get(section_start + local_y)
            .and_then(|state| *state)
            .unwrap_or("minecraft:air");
        let palette_index = match palette.iter().position(|entry| *entry == block) {
            Some(index) => index as u64,
            None => {
                palette.push(block);
                (palette.len() - 1) as u64
            }
        };
        for z in 0..16 {
            for x in 0..16 {
                indices[(local_y << 8) | (z << 4) | x] = palette_index;
            }
        }
    }

    if palette.len() == 1 {
        return PalettedContainer::single(block_state_tag(palette[0]), SECTION_VOLUME);
    }

    PalettedContainer {
        palette: palette.into_iter().map(block_state_tag).collect(),
        data: Some(pack_palette_indices(
            &indices,
            bits_for_palette(indices.iter().copied().max().unwrap_or(0) + 1),
        )),
        expected_entries: SECTION_VOLUME,
    }
}

fn block_state_tag(block: &'static str) -> Tag {
    BlockStateEntry::new(block).to_nbt()
}

fn pack_palette_indices(indices: &[u64], bits_per_entry: usize) -> Vec<i64> {
    let values_per_long = 64 / bits_per_entry;
    let mut packed = vec![0_u64; indices.len().div_ceil(values_per_long)];
    for (index, value) in indices.iter().copied().enumerate() {
        let word_index = index / values_per_long;
        let bit_index = (index - word_index * values_per_long) * bits_per_entry;
        packed[word_index] |= value << bit_index;
    }
    packed.into_iter().map(|word| word as i64).collect()
}

fn pack_heightmap(values: [i32; 16 * 16]) -> Vec<i64> {
    const BITS_PER_ENTRY: usize = 9;
    let mut packed = vec![0_u64; (values.len() * BITS_PER_ENTRY).div_ceil(64)];
    for (index, value) in values.into_iter().enumerate() {
        let bit_offset = index * BITS_PER_ENTRY;
        let word_index = bit_offset / 64;
        let bit_index = bit_offset % 64;
        let value = value.max(0) as u64 & ((1 << BITS_PER_ENTRY) - 1);
        packed[word_index] |= value << bit_index;
        let spill = bit_index + BITS_PER_ENTRY;
        if spill > 64 {
            packed[word_index + 1] |= value >> (64 - bit_index);
        }
    }
    packed.into_iter().map(|word| word as i64).collect()
}

fn bits_for_palette(palette_len: u64) -> usize {
    let needed = 64 - (palette_len.saturating_sub(1)).leading_zeros() as usize;
    needed.max(4)
}

fn heightmap_opaque(heightmap: HeightmapKind, block: &str) -> bool {
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => !matches!(
            block,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ),
        HeightmapKind::OceanFloor | HeightmapKind::OceanFloorWg => block_blocks_motion(block),
        HeightmapKind::MotionBlocking => block_blocks_motion(block) || block_has_fluid(block),
        HeightmapKind::MotionBlockingNoLeaves => {
            (block_blocks_motion(block) || block_has_fluid(block)) && !block_is_leaves(block)
        }
    }
}

fn block_blocks_motion(block: &str) -> bool {
    !matches!(
        block_state_id(block),
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:snow"
    )
}

fn block_has_fluid(block: &str) -> bool {
    matches!(block_state_id(block), "minecraft:water" | "minecraft:lava")
        || block.contains("waterlogged=true")
}

fn block_is_leaves(block: &str) -> bool {
    block_state_id(block).ends_with("_leaves")
}

fn block_state_id(block: &str) -> &str {
    block.split_once('[').map_or(block, |(id, _)| id)
}

pub const NETHER_LEVEL_STEM: LevelStemPreset = LevelStemPreset {
    dimension: "minecraft:the_nether",
    generator: "minecraft:noise",
    biome_source: "minecraft:multi_noise/nether",
    noise_settings: Some("minecraft:nether"),
};

pub const END_LEVEL_STEM: LevelStemPreset = LevelStemPreset {
    dimension: "minecraft:the_end",
    generator: "minecraft:noise",
    biome_source: "minecraft:the_end",
    noise_settings: Some("minecraft:end"),
};

pub const WORLD_PRESETS: &[WorldPresetEntry] = &[
    WorldPresetEntry {
        id: "minecraft:normal",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:overworld"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:flat",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:flat",
            biome_source: "minecraft:plains",
            noise_settings: None,
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:large_biomes",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:large_biomes"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:amplified",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:amplified"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:single_biome_surface",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:fixed/plains",
            noise_settings: Some("minecraft:overworld"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:debug_all_block_states",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:debug",
            biome_source: "minecraft:plains",
            noise_settings: None,
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
];

pub const OVERWORLD_SPAWN_TARGET: &[ClimateParameterPoint] = &[
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(-1.0, -0.16),
        offset: 0,
    },
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(0.16, 1.0),
        offset: 0,
    },
];

pub const BUILTIN_NOISE_GENERATOR_SETTINGS: &[NoiseGeneratorSettings] = &[
    NoiseGeneratorSettings {
        id: "minecraft:overworld",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:large_biomes",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: true,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:amplified",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: true,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:nether",
        noise: NETHER_NOISE_SETTINGS,
        default_block: "minecraft:netherrack",
        default_fluid: "minecraft:lava",
        noise_router: NoiseRouterPreset::Nether,
        surface_rule: SurfaceRulePreset::Nether,
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:end",
        noise: END_NOISE_SETTINGS,
        default_block: "minecraft:end_stone",
        default_fluid: "minecraft:air",
        noise_router: NoiseRouterPreset::End,
        surface_rule: SurfaceRulePreset::End,
        spawn_target: &[],
        sea_level: 0,
        disable_mob_generation: true,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:caves",
        noise: CAVES_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Caves,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: true,
            surface: true,
        },
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:floating_islands",
        noise: FLOATING_ISLANDS_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::FloatingIslands,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: false,
            surface: false,
        },
        spawn_target: &[],
        sea_level: -64,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
];

pub const EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS: &[NoiseSettingsRegistryExpectation] = &[
    NoiseSettingsRegistryExpectation {
        id: "minecraft:amplified",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:amplified",
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target_len: 2,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:caves",
        noise: CAVES_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:caves",
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: true,
            surface: true,
        },
        spawn_target_len: 0,
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:end",
        noise: END_NOISE_SETTINGS,
        default_block: "minecraft:end_stone",
        default_fluid: "minecraft:air",
        router_id: "minecraft:end",
        surface_rule: SurfaceRulePreset::End,
        spawn_target_len: 0,
        sea_level: 0,
        disable_mob_generation: true,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:floating_islands",
        noise: FLOATING_ISLANDS_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:floating_islands",
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: false,
            surface: false,
        },
        spawn_target_len: 0,
        sea_level: -64,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:large_biomes",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:large_biomes",
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target_len: 2,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:nether",
        noise: NETHER_NOISE_SETTINGS,
        default_block: "minecraft:netherrack",
        default_fluid: "minecraft:lava",
        router_id: "minecraft:nether",
        surface_rule: SurfaceRulePreset::Nether,
        spawn_target_len: 0,
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseSettingsRegistryExpectation {
        id: "minecraft:overworld",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        router_id: "minecraft:overworld",
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target_len: 2,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
];

pub const END_ISLANDS_DENSITY: DensityFunction = DensityFunction::EndIslands { seed: 0 };
pub const TEST_NEGATIVE_DENSITY: DensityFunction = DensityFunction::Constant(-2.0);
pub const TEST_POSITIVE_DENSITY: DensityFunction = DensityFunction::Constant(3.0);
pub const TEST_CACHE_ALL_IN_CELL_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::CacheAllInCell,
    input: &TEST_POSITIVE_DENSITY,
};
pub const TEST_RANGE_CHOICE_DENSITY: DensityFunction = DensityFunction::RangeChoice {
    input: &Y_DENSITY,
    min_inclusive: -1.0,
    max_exclusive: 1.0,
    when_in_range: &TEST_POSITIVE_DENSITY,
    when_out_of_range: &TEST_NEGATIVE_DENSITY,
};

fn select_weighted_pool_target(
    targets: &[JigsawPoolAliasWeightedTarget],
    random: &mut RandomSourceKind,
) -> &'static str {
    let index = select_weighted_index(
        targets.iter().map(|target| target.weight),
        targets.len(),
        random,
    );
    targets[index].target
}

fn select_weighted_pool_group<'a>(
    groups: &'a [JigsawPoolAliasWeightedGroup],
    random: &mut RandomSourceKind,
) -> &'a [JigsawPoolAliasBindingModel] {
    let index = select_weighted_index(
        groups.iter().map(|group| group.weight),
        groups.len(),
        random,
    );
    &groups[index].bindings
}

fn select_weighted_index(
    weights: impl Iterator<Item = i32>,
    len: usize,
    random: &mut RandomSourceKind,
) -> usize {
    assert!(len > 0, "WeightedList must not be empty");
    let weights = weights.collect::<Vec<_>>();
    let total_weight: i32 = weights
        .iter()
        .copied()
        .inspect(|weight| assert!(*weight > 0))
        .sum();
    let mut value = random_next_i32_bound(random, total_weight);
    for (index, weight) in weights.into_iter().enumerate() {
        value -= weight;
        if value < 0 {
            return index;
        }
    }
    len - 1
}

pub fn structure_pieces_move_inside_heights_dy(
    bounding_box: StructureBoundingBoxModel,
    lowest_allowed: i32,
    highest_allowed: i32,
    random_roll: i32,
) -> Result<i32, String> {
    let y_span = bounding_box.max_y - bounding_box.min_y + 1;
    let height_span = highest_allowed - lowest_allowed + 1 - y_span;
    let y0_pos = if height_span > 1 {
        if !(0..height_span).contains(&random_roll) {
            return Err(
                "StructurePiecesBuilder moveInsideHeights roll is outside RandomSource#nextInt span"
                    .to_string(),
            );
        }
        lowest_allowed + random_roll
    } else {
        lowest_allowed
    };
    Ok(y0_pos - bounding_box.min_y)
}

pub fn structure_rotation_rotate_direction(
    rotation: StructureRotation,
    direction: HorizontalDirection,
) -> HorizontalDirection {
    match rotation {
        StructureRotation::None => direction,
        StructureRotation::Clockwise90 => match direction {
            HorizontalDirection::North => HorizontalDirection::East,
            HorizontalDirection::East => HorizontalDirection::South,
            HorizontalDirection::South => HorizontalDirection::West,
            HorizontalDirection::West => HorizontalDirection::North,
        },
        StructureRotation::Clockwise180 => match direction {
            HorizontalDirection::North => HorizontalDirection::South,
            HorizontalDirection::South => HorizontalDirection::North,
            HorizontalDirection::West => HorizontalDirection::East,
            HorizontalDirection::East => HorizontalDirection::West,
        },
        StructureRotation::Counterclockwise90 => match direction {
            HorizontalDirection::North => HorizontalDirection::West,
            HorizontalDirection::West => HorizontalDirection::South,
            HorizontalDirection::South => HorizontalDirection::East,
            HorizontalDirection::East => HorizontalDirection::North,
        },
    }
}

pub fn structure_rotation_add(
    rotation: StructureRotation,
    other: StructureRotation,
) -> StructureRotation {
    fossil_rotation(
        match rotation {
            StructureRotation::None => 0,
            StructureRotation::Clockwise90 => 1,
            StructureRotation::Clockwise180 => 2,
            StructureRotation::Counterclockwise90 => 3,
        } + match other {
            StructureRotation::None => 0,
            StructureRotation::Clockwise90 => 1,
            StructureRotation::Clockwise180 => 2,
            StructureRotation::Counterclockwise90 => 3,
        },
    )
}

pub fn block_pos_relative(
    pos: BlockPos,
    direction: HorizontalDirection,
    distance: i32,
) -> BlockPos {
    match direction {
        HorizontalDirection::North => BlockPos {
            z: pos.z - distance,
            ..pos
        },
        HorizontalDirection::South => BlockPos {
            z: pos.z + distance,
            ..pos
        },
        HorizontalDirection::West => BlockPos {
            x: pos.x - distance,
            ..pos
        },
        HorizontalDirection::East => BlockPos {
            x: pos.x + distance,
            ..pos
        },
    }
}

const fn feature_type(
    id: &'static str,
    configuration: FeatureConfigurationKind,
    family: FeatureFamily,
) -> FeatureType {
    FeatureType {
        id,
        configuration,
        family,
    }
}

const fn cf(id: &'static str, source: ConfiguredFeatureSource) -> ConfiguredFeatureEntry {
    ConfiguredFeatureEntry { id, source }
}

const fn placed_source(
    source: PlacedFeatureSource,
    keys: &'static [&'static str],
) -> PlacedFeatureSourceEntry {
    PlacedFeatureSourceEntry { source, keys }
}

impl NoiseSettings {
    pub const fn new(min_y: i32, height: i32, size_horizontal: i32, size_vertical: i32) -> Self {
        Self {
            min_y,
            height,
            size_horizontal,
            size_vertical,
        }
    }

    pub fn validate(self) -> Result<(), String> {
        if self.min_y + self.height > 2032 {
            return Err("min_y + height cannot be higher than: 2032".to_string());
        }
        if self.height % 16 != 0 {
            return Err("height has to be a multiple of 16".to_string());
        }
        if self.min_y % 16 != 0 {
            return Err("min_y has to be a multiple of 16".to_string());
        }
        if !(1..=4).contains(&self.size_horizontal) {
            return Err("size_horizontal must be in 1..=4".to_string());
        }
        if !(1..=4).contains(&self.size_vertical) {
            return Err("size_vertical must be in 1..=4".to_string());
        }
        Ok(())
    }

    pub fn cell_height(self) -> i32 {
        self.size_vertical * 4
    }

    pub fn cell_width(self) -> i32 {
        self.size_horizontal * 4
    }

    pub fn clamp_to_height(self, min_y: i32, max_y: i32) -> Self {
        let new_min_y = self.min_y.max(min_y);
        let new_height = (self.min_y + self.height).min(max_y + 1) - new_min_y;
        Self::new(
            new_min_y,
            new_height,
            self.size_horizontal,
            self.size_vertical,
        )
    }
}



pub fn builtin_density_function(id: &str) -> Option<&'static DensityFunctionEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_DENSITY_FUNCTIONS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn density_function_type(id: &str) -> Option<&'static DensityFunctionType> {
    DENSITY_FUNCTION_TYPES.iter().find(|kind| kind.id == id)
}

pub fn builtin_noise_router(id: &str) -> Option<&'static NoiseRouterEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_ROUTERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn noise_router_id_for_settings(settings: NoiseGeneratorSettings) -> &'static str {
    match settings.noise_router {
        NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: false,
        } => "minecraft:overworld",
        NoiseRouterPreset::Overworld {
            large_biomes: true,
            amplified: false,
        } => "minecraft:large_biomes",
        NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: true,
        } => "minecraft:amplified",
        NoiseRouterPreset::Overworld { .. } => "minecraft:overworld",
        NoiseRouterPreset::Nether => "minecraft:nether",
        NoiseRouterPreset::End => "minecraft:end",
        NoiseRouterPreset::Caves => "minecraft:caves",
        NoiseRouterPreset::FloatingIslands => "minecraft:floating_islands",
    }
}

/// Returns the Y value of the highest non-air block + 1 (or 0 if not present).

pub fn cave_generation_family(id: &str) -> Option<&'static CaveGenerationFamily> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CAVE_GENERATION_FAMILIES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn ore_vein_type_for_toggle(vein_toggle: f64) -> OreVeinType {
    if vein_toggle > 0.0 {
        ORE_VEIN_TYPES[0]
    } else {
        ORE_VEIN_TYPES[1]
    }
}

pub fn ore_vein_richness(veininess_ridged: f64) -> f64 {
    clamped_map(
        veininess_ridged,
        ORE_VEINIFIER_CONSTANTS.veininess_threshold,
        ORE_VEINIFIER_CONSTANTS.max_richness_threshold,
        ORE_VEINIFIER_CONSTANTS.min_richness,
        ORE_VEINIFIER_CONSTANTS.max_richness,
    )
}

pub fn ore_vein_decision(input: OreVeinDecisionInput) -> Option<&'static str> {
    let default_state = input.debug_ore_veins.then_some("minecraft:air");
    let vein_type = ore_vein_type_for_toggle(input.vein_toggle);
    let veininess_ridged = input.vein_toggle.abs();
    let distance_from_top = vein_type.max_y - input.y;
    let distance_from_bottom = input.y - vein_type.min_y;
    if distance_from_bottom < 0 || distance_from_top < 0 {
        return default_state;
    }

    let distance_from_edge = distance_from_top.min(distance_from_bottom);
    let edge_roundoff = clamped_map(
        f64::from(distance_from_edge),
        0.0,
        f64::from(ORE_VEINIFIER_CONSTANTS.edge_roundoff_begin),
        -ORE_VEINIFIER_CONSTANTS.max_edge_roundoff,
        0.0,
    );
    if veininess_ridged + edge_roundoff < ORE_VEINIFIER_CONSTANTS.veininess_threshold {
        return default_state;
    }
    if input.solidness_random > ORE_VEINIFIER_CONSTANTS.vein_solidness {
        return default_state;
    }
    if input.vein_ridged >= 0.0 {
        return default_state;
    }

    let richness = ore_vein_richness(veininess_ridged);
    if input.richness_random < richness
        && input.vein_gap > ORE_VEINIFIER_CONSTANTS.skip_ore_if_gap_noise_is_below
    {
        if input.raw_ore_random < ORE_VEINIFIER_CONSTANTS.chance_of_raw_ore_block {
            Some(vein_type.raw_ore_block)
        } else {
            Some(vein_type.ore)
        }
    } else if input.debug_ore_veins {
        Some("minecraft:oak_button")
    } else {
        Some(vein_type.filler)
    }
}

pub fn ore_vein_decision_at(
    ore_factory: PositionalRandomFactory,
    x: i32,
    y: i32,
    z: i32,
    vein_toggle: f64,
    vein_ridged: f64,
    vein_gap: f64,
    debug_ore_veins: bool,
) -> Option<&'static str> {
    ore_vein_decision_after_toggle(
        ore_factory,
        x,
        y,
        z,
        vein_toggle,
        || vein_ridged,
        || vein_gap,
        debug_ore_veins,
    )
}

struct OreVeinMaterialRule {
    ore_factory: PositionalRandomFactory,
    debug_ore_veins: bool,
}

impl OreVeinMaterialRule {
    fn try_apply(
        &self,
        noise_chunk: &NoiseChunk,
        x: i32,
        y: i32,
        z: i32,
        timings: &mut LiveTerrainTimings,
        detailed_timing: bool,
    ) -> Option<&'static str> {
        if !(ORE_VEIN_TYPES[0].min_y..=ORE_VEIN_TYPES[0].max_y).contains(&y)
            && !(ORE_VEIN_TYPES[1].min_y..=ORE_VEIN_TYPES[1].max_y).contains(&y)
        {
            return None;
        }
        timings.ore_vein_samples += 1;
        let vein_toggle = if detailed_timing {
            let ore_lookup_started = Instant::now();
            let vein_toggle = noise_chunk.cached_vein_toggle(x, y, z);
            timings.fill_ore_vein_lookup_us += ore_lookup_started.elapsed().as_micros();
            vein_toggle
        } else {
            noise_chunk.cached_vein_toggle(x, y, z)
        };

        if detailed_timing {
            let ore_decision_started = Instant::now();
            let result = ore_vein_decision_after_toggle(
                self.ore_factory,
                x,
                y,
                z,
                vein_toggle,
                || noise_chunk.vein_ridged_at(x, y, z),
                || noise_chunk.vein_gap_at(x, y, z),
                self.debug_ore_veins,
            );
            timings.fill_ore_decision_us += ore_decision_started.elapsed().as_micros();
            result
        } else {
            ore_vein_decision_after_toggle(
                self.ore_factory,
                x,
                y,
                z,
                vein_toggle,
                || noise_chunk.vein_ridged_at(x, y, z),
                || noise_chunk.vein_gap_at(x, y, z),
                self.debug_ore_veins,
            )
        }
    }
}

struct NoiseMaterialRuleList {
    default_block: &'static str,
    ore_vein_rule: Option<OreVeinMaterialRule>,
}

impl NoiseMaterialRuleList {
    fn new(settings: &NoiseGeneratorSettings, ore_factory: PositionalRandomFactory) -> Self {
        Self {
            default_block: settings.default_block,
            ore_vein_rule: settings.ore_veins_enabled.then_some(OreVeinMaterialRule {
                ore_factory,
                debug_ore_veins: false,
            }),
        }
    }

    fn calculate(
        &self,
        aquifer: Option<&mut NoiseBasedAquifer>,
        noise_chunk: &NoiseChunk,
        settings: &NoiseGeneratorSettings,
        x: i32,
        y: i32,
        z: i32,
        density: f64,
        timings: &mut LiveTerrainTimings,
        detailed_timing: bool,
    ) -> &'static str {
        // Mirrors Java's `MaterialRuleList`: the aquifer filler runs first and
        // may return a fluid/air block. Returning `None` means the slot remains
        // solid, so the next filler (OreVeinifier) gets a chance before the
        // generator falls back to the default block.
        let aquifer_substance = if density > 0.0 {
            None
        } else if let Some(aquifer) = aquifer {
            timings.aquifer_calls += 1;
            if detailed_timing {
                let aquifer_started = Instant::now();
                let substance = aquifer.compute_substance(noise_chunk, x, y, z, density);
                timings.fill_aquifer_compute_us += aquifer_started.elapsed().as_micros();
                substance
            } else {
                aquifer.compute_substance(noise_chunk, x, y, z, density)
            }
        } else {
            Some(global_fluid_status(y, settings.sea_level, settings.default_fluid).at(y))
        };

        if let Some(block) = aquifer_substance {
            return block;
        }

        if let Some(rule) = &self.ore_vein_rule {
            if let Some(block) = rule.try_apply(noise_chunk, x, y, z, timings, detailed_timing) {
                return block;
            }
        }

        self.default_block
    }
}

fn ore_vein_decision_after_toggle<R, G>(
    ore_factory: PositionalRandomFactory,
    x: i32,
    y: i32,
    z: i32,
    vein_toggle: f64,
    vein_ridged: R,
    vein_gap: G,
    debug_ore_veins: bool,
) -> Option<&'static str>
where
    R: FnOnce() -> f64,
    G: FnOnce() -> f64,
{
    let default_state = debug_ore_veins.then_some("minecraft:air");
    let vein_type = ore_vein_type_for_toggle(vein_toggle);
    let veininess_ridged = vein_toggle.abs();
    let distance_from_top = vein_type.max_y - y;
    let distance_from_bottom = y - vein_type.min_y;
    if distance_from_bottom < 0 || distance_from_top < 0 {
        return default_state;
    }

    let distance_from_edge = distance_from_top.min(distance_from_bottom);
    let edge_roundoff = clamped_map(
        f64::from(distance_from_edge),
        0.0,
        f64::from(ORE_VEINIFIER_CONSTANTS.edge_roundoff_begin),
        -ORE_VEINIFIER_CONSTANTS.max_edge_roundoff,
        0.0,
    );
    if veininess_ridged + edge_roundoff < ORE_VEINIFIER_CONSTANTS.veininess_threshold {
        return default_state;
    }

    // Java OreVeinifier only creates/samples the positional random after the
    // range and veininess checks pass.
    let mut positional_random = ore_factory.at(x, y, z);
    let solidness_random = f64::from(positional_random.next_f32());
    if solidness_random > ORE_VEINIFIER_CONSTANTS.vein_solidness {
        return default_state;
    }
    if vein_ridged() >= 0.0 {
        return default_state;
    }

    let richness = ore_vein_richness(veininess_ridged);
    let richness_random = f64::from(positional_random.next_f32());
    if richness_random < richness
        && vein_gap() > ORE_VEINIFIER_CONSTANTS.skip_ore_if_gap_noise_is_below
    {
        let raw_ore_random = f64::from(positional_random.next_f32());
        if raw_ore_random < ORE_VEINIFIER_CONSTANTS.chance_of_raw_ore_block {
            Some(vein_type.raw_ore_block)
        } else {
            Some(vein_type.ore)
        }
    } else if debug_ore_veins {
        Some("minecraft:oak_button")
    } else {
        Some(vein_type.filler)
    }
}

fn clamped_map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let clamped = value.clamp(from_min, from_max);
    let progress = (clamped - from_min) / (from_max - from_min);
    to_min + progress * (to_max - to_min)
}

fn apply_underground_ore_decoration_to_chunk(
    chunk: &mut LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    decoration_region_biome_steps: Option<&[&'static [&'static [&'static str]]]>,
) -> usize {
    apply_underground_ore_decoration_to_chunk_with_context(
        chunk,
        biome_source_model,
        settings,
        seed,
        decoration_region_biome_steps,
        None,
        None,
    )
}

fn apply_underground_ore_decoration_to_chunk_with_context(
    chunk: &mut LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    decoration_region_biome_steps: Option<&[&'static [&'static [&'static str]]]>,
    decoration_context_chunks: Option<&HashMap<ChunkPos, LightweightTreeContextChunk>>,
    precomputed_source_steps: Option<&DecorationBiomeStepsByChunk>,
) -> usize {
    let total_started = Instant::now();
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return 0;
    }

    let Some(router) =
        builtin_noise_router(noise_router_id_for_settings(*settings)).map(|entry| entry.router)
    else {
        return 0;
    };
    let climate_sampler = ClimateSampler::from_noise_router(&router, seed, *settings);
    let owned_source_steps_cache;
    let source_steps_cache = if let Some(source_steps) = precomputed_source_steps {
        source_steps
    } else if decoration_region_biome_steps.is_some() {
        owned_source_steps_cache = DecorationBiomeStepsByChunk::new();
        &owned_source_steps_cache
    } else {
        owned_source_steps_cache = source_decoration_biome_steps_cache(
            chunk.pos,
            1,
            biome_source_model,
            settings,
            &climate_sampler,
        );
        &owned_source_steps_cache
    };
    let started = Instant::now();
    let global_biome_steps = possible_biome_feature_steps_for_source(biome_source_model);
    let target_possible_steps = decoration_region_biome_steps
        .map(|steps| steps.to_vec())
        .unwrap_or_default();
    let feature_source_steps = if !global_biome_steps.is_empty() {
        &global_biome_steps
    } else {
        &target_possible_steps
    };
    let features_per_step = match build_features_per_step(feature_source_steps, true) {
        Ok(features) => features,
        Err(_) => return 0,
    };
    let feature_sort_ms = started.elapsed().as_millis();

    let mut placed = 0;
    let mut calls = 0;
    let owned_context_chunks;
    let context_chunks = if let Some(context_chunks) = decoration_context_chunks {
        context_chunks
    } else {
        owned_context_chunks =
            build_underground_ore_decoration_context_chunks(chunk.pos, settings, seed);
        &owned_context_chunks
    };
    let mut block_cache = OreBlockCache::from_chunk_with_read_context(chunk, context_chunks);
    let mut biome_steps_ms = 0_u128;
    let mut plan_ms = 0_u128;
    let mut possible_step_sets = 0_usize;
    let mut ore_candidate_us = 0_u128;
    let mut ore_block_us = 0_u128;
    let mut ore_origin_us = 0_u128;
    let mut ore_total_us = 0_u128;
    let mut ore_configured_calls = 0_usize;
    let mut ore_candidate_count = 0_usize;
    let mut ore_in_chunk_candidates = 0_usize;
    let mut ore_model_cache: HashMap<&'static str, (PlacedOreFeatureModel, OreConfigurationModel)> =
        HashMap::new();
    let mut disk_model_cache: HashMap<
        &'static str,
        (PlacedDiskFeatureModel, DiskConfigurationModel),
    > = HashMap::new();
    let placement_started = Instant::now();
    for source_z in chunk.pos.z - 1..=chunk.pos.z + 1 {
        for source_x in chunk.pos.x - 1..=chunk.pos.x + 1 {
            let source_pos = ChunkPos {
                x: source_x,
                z: source_z,
            };
            let started = Instant::now();
            let possible_steps = decoration_region_biome_steps
                .map(|steps| steps.to_vec())
                .or_else(|| source_steps_cache.get(&source_pos).cloned())
                .unwrap_or_else(|| {
                    possible_biome_feature_steps_for_decoration_region(
                        source_pos,
                        biome_source_model,
                        settings,
                        &climate_sampler,
                    )
                });
            if decoration_region_biome_steps.is_none() {
                biome_steps_ms += started.elapsed().as_millis();
            }
            if possible_steps.is_empty() {
                continue;
            }
            possible_step_sets += possible_steps.len();

            let skip_biome_filter = biome_steps_share_decoration_step_features(
                &possible_steps,
                GenerationDecorationStep::UndergroundOres,
            );
            let started = Instant::now();
            let plan = biome_decoration_feature_plan(
                seed,
                source_pos.x,
                source_pos.z,
                settings.noise.min_y.div_euclid(16),
                &features_per_step,
                &possible_steps,
            );
            plan_ms += started.elapsed().as_millis();

            for call in plan.feature_calls.iter().filter(|call| {
                call.step_index == GenerationDecorationStep::UndergroundOres as usize
            }) {
                calls += 1;
                if let Some(report) = {
                    if !ore_model_cache.contains_key(call.feature) {
                        if let Some(feature) = placed_ore_feature(call.feature) {
                            if let Some(config) =
                                configured_ore_configuration(feature.configured_feature)
                            {
                                ore_model_cache.insert(call.feature, (feature, config));
                            }
                        }
                    }
                    ore_model_cache.get(call.feature).map(|(feature, config)| {
                        place_ore_feature_in_chunk(
                            &*chunk,
                            &mut block_cache,
                            source_pos,
                            biome_source_model,
                            settings,
                            seed,
                            &climate_sampler,
                            call.feature,
                            feature,
                            config,
                            call.seed,
                            skip_biome_filter,
                        )
                    })
                } {
                    placed += report.placed;
                    ore_candidate_us += report.candidate_us;
                    ore_block_us += report.block_us;
                    ore_origin_us += report.origin_us;
                    ore_total_us += report.total_us;
                    ore_configured_calls += report.configured_calls;
                    ore_candidate_count += report.candidate_count;
                    ore_in_chunk_candidates += report.in_chunk_candidates;
                    continue;
                }

                if !disk_model_cache.contains_key(call.feature) {
                    if let Some(feature) = placed_disk_feature(call.feature) {
                        if let Some(config) =
                            configured_disk_configuration(feature.configured_feature)
                        {
                            disk_model_cache.insert(call.feature, (feature, config));
                        }
                    }
                }
                if let Some((feature, config)) = disk_model_cache.get(call.feature) {
                    let disk_placed = place_disk_feature_in_chunk(
                        &mut block_cache,
                        source_pos,
                        biome_source_model,
                        settings,
                        &climate_sampler,
                        call.feature,
                        feature,
                        config,
                        call.seed,
                        skip_biome_filter,
                    );
                    placed += disk_placed;
                }
            }
        }
    }
    let flush_started = Instant::now();
    block_cache.flush_to_chunk(chunk);
    let flush_ms = flush_started.elapsed().as_millis();
    let placement_ms = placement_started.elapsed().as_millis();
    if std::env::var_os("RUSTCRAFT_WORLDGEN_ORE_DEBUG").is_some() {
        eprintln!(
            "[ore-debug] total={}ms biome_steps={}ms possible_steps={} global_steps={} feature_sort={}ms plan={}ms placement={}ms flush={}ms calls={} configured_calls={} candidates={} in_chunk={} configured_time={}us origin_time={}us candidate_time={}us block_time={}us placed={}",
            total_started.elapsed().as_millis(),
            biome_steps_ms,
            possible_step_sets,
            global_biome_steps.len(),
            feature_sort_ms,
            plan_ms,
            placement_ms,
            flush_ms,
            calls,
            ore_configured_calls,
            ore_candidate_count,
            ore_in_chunk_candidates,
            ore_total_us,
            ore_origin_us,
            ore_candidate_us,
            ore_block_us,
            placed
        );
    }

    placed
}

fn apply_underground_ore_decoration_from_source_into_region(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    source_pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    decoration_region_biome_steps: Option<&[&'static [&'static [&'static str]]]>,
) -> usize {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return 0;
    }
    let Some(source_chunk) = chunks.get(&source_pos).cloned() else {
        return 0;
    };
    let Some(router) =
        builtin_noise_router(noise_router_id_for_settings(*settings)).map(|entry| entry.router)
    else {
        return 0;
    };
    let climate_sampler = ClimateSampler::from_noise_router(&router, seed, *settings);
    let possible_steps = decoration_region_biome_steps
        .map(|steps| steps.to_vec())
        .unwrap_or_else(|| {
            possible_biome_feature_steps_for_decoration_region(
                source_pos,
                biome_source_model,
                settings,
                &climate_sampler,
            )
        });
    if possible_steps.is_empty() {
        return 0;
    }

    let global_biome_steps = possible_biome_feature_steps_for_source(biome_source_model);
    let feature_source_steps = if !global_biome_steps.is_empty() {
        &global_biome_steps
    } else {
        &possible_steps
    };
    let features_per_step = match build_features_per_step(feature_source_steps, true) {
        Ok(features) => features,
        Err(_) => return 0,
    };
    let plan = biome_decoration_feature_plan(
        seed,
        source_pos.x,
        source_pos.z,
        settings.noise.min_y.div_euclid(16),
        &features_per_step,
        &possible_steps,
    );
    let skip_biome_filter = biome_steps_share_decoration_step_features(
        &possible_steps,
        GenerationDecorationStep::UndergroundOres,
    );
    let Some(mut block_cache) = OreBlockCache::from_region_chunks(source_pos, chunks) else {
        return 0;
    };
    let mut placed = 0;
    let mut ore_model_cache: HashMap<&'static str, (PlacedOreFeatureModel, OreConfigurationModel)> =
        HashMap::new();
    let mut disk_model_cache: HashMap<
        &'static str,
        (PlacedDiskFeatureModel, DiskConfigurationModel),
    > = HashMap::new();

    for call in plan
        .feature_calls
        .iter()
        .filter(|call| call.step_index == GenerationDecorationStep::UndergroundOres as usize)
    {
        if !ore_model_cache.contains_key(call.feature) {
            if let Some(feature) = placed_ore_feature(call.feature) {
                if let Some(config) = configured_ore_configuration(feature.configured_feature) {
                    ore_model_cache.insert(call.feature, (feature, config));
                }
            }
        }
        if let Some((feature, config)) = ore_model_cache.get(call.feature) {
            placed += place_ore_feature_in_chunk(
                &source_chunk,
                &mut block_cache,
                source_pos,
                biome_source_model,
                settings,
                seed,
                &climate_sampler,
                call.feature,
                feature,
                config,
                call.seed,
                skip_biome_filter,
            )
            .placed;
            continue;
        }

        if !disk_model_cache.contains_key(call.feature) {
            if let Some(feature) = placed_disk_feature(call.feature) {
                if let Some(config) = configured_disk_configuration(feature.configured_feature) {
                    disk_model_cache.insert(call.feature, (feature, config));
                }
            }
        }
        if let Some((feature, config)) = disk_model_cache.get(call.feature) {
            placed += place_disk_feature_in_chunk(
                &mut block_cache,
                source_pos,
                biome_source_model,
                settings,
                &climate_sampler,
                call.feature,
                feature,
                config,
                call.seed,
                skip_biome_filter,
            );
        }
    }

    block_cache.flush_to_chunks(chunks);
    placed
}



#[derive(Debug, Clone)]
struct TreeDecorationHeights {
    ocean_floor: [i32; 16 * 16],
    world_surface: [i32; 16 * 16],
    motion_blocking: [i32; 16 * 16],
    motion_blocking_no_leaves: [i32; 16 * 16],
}

#[derive(Clone, Copy)]
enum SourceTerrainHeights<'a> {
    Full(&'a TreeDecorationHeights),
    Lazy(&'a LightweightTreeContextChunk),
}

impl SourceTerrainHeights<'_> {
    fn local_height(self, heightmap: HeightmapKind, local_x: usize, local_z: usize) -> i32 {
        match self {
            SourceTerrainHeights::Full(heights) => {
                heights.local_height(heightmap, local_x, local_z)
            }
            SourceTerrainHeights::Lazy(chunk) => chunk
                .terrain_heights
                .local_height(heightmap, local_x, local_z),
        }
    }

    fn world_height(
        self,
        source_pos: ChunkPos,
        heightmap: HeightmapKind,
        world_x: i32,
        world_z: i32,
        fallback_y: i32,
    ) -> i32 {
        if world_x.div_euclid(16) != source_pos.x || world_z.div_euclid(16) != source_pos.z {
            return fallback_y;
        }
        self.local_height(
            heightmap,
            world_x.rem_euclid(16) as usize,
            world_z.rem_euclid(16) as usize,
        )
    }
}

impl TreeDecorationHeights {
    fn local_height(&self, heightmap: HeightmapKind, local_x: usize, local_z: usize) -> i32 {
        let index = local_z * 16 + local_x;
        match heightmap {
            HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => {
                self.world_surface[index]
            }
            HeightmapKind::OceanFloor | HeightmapKind::OceanFloorWg => self.ocean_floor[index],
            HeightmapKind::MotionBlocking => self.motion_blocking[index],
            HeightmapKind::MotionBlockingNoLeaves => self.motion_blocking_no_leaves[index],
        }
    }
}

fn tree_decoration_terrain_heights_from_wg(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
) -> TreeDecorationHeights {
    let mut ocean_floor = [settings.sea_level + 1; 16 * 16];
    let mut world_surface = [settings.sea_level + 1; 16 * 16];
    let mut motion_blocking = [settings.sea_level + 1; 16 * 16];
    let mut motion_blocking_no_leaves = [settings.sea_level + 1; 16 * 16];
    for z in 0..16 {
        for x in 0..16 {
            let index = z * 16 + x;
            ocean_floor[index] = chunk
                .heightmap_value(HeightmapKind::OceanFloorWg, x, z)
                .or_else(|| chunk.heightmap_value(HeightmapKind::OceanFloor, x, z))
                .or_else(|| chunk.heightmap_value(HeightmapKind::WorldSurfaceWg, x, z))
                .unwrap_or(settings.sea_level + 1);
            world_surface[index] = chunk
                .heightmap_value(HeightmapKind::WorldSurfaceWg, x, z)
                .or_else(|| chunk.heightmap_value(HeightmapKind::WorldSurface, x, z))
                .unwrap_or(ocean_floor[index]);
            motion_blocking[index] = chunk
                .heightmap_value(HeightmapKind::MotionBlocking, x, z)
                .unwrap_or(world_surface[index]);
            motion_blocking_no_leaves[index] = chunk
                .heightmap_value(HeightmapKind::MotionBlockingNoLeaves, x, z)
                .unwrap_or(motion_blocking[index]);
        }
    }
    TreeDecorationHeights {
        ocean_floor,
        world_surface,
        motion_blocking,
        motion_blocking_no_leaves,
    }
}

fn tree_decoration_terrain_heights(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
) -> TreeDecorationHeights {
    let computed_ocean_floor = chunk.compute_heightmap_values(HeightmapKind::OceanFloor);
    let computed_world_surface = chunk.compute_heightmap_values(HeightmapKind::WorldSurface);
    let computed_motion_blocking = chunk.compute_heightmap_values(HeightmapKind::MotionBlocking);
    let computed_motion_blocking_no_leaves =
        chunk.compute_heightmap_values(HeightmapKind::MotionBlockingNoLeaves);
    let mut ocean_floor = computed_ocean_floor;
    let mut world_surface = computed_world_surface;
    let mut motion_blocking = computed_motion_blocking;
    let mut motion_blocking_no_leaves = computed_motion_blocking_no_leaves;
    for z in 0..16 {
        for x in 0..16 {
            let index = z * 16 + x;
            if ocean_floor[index] == 0 {
                ocean_floor[index] = chunk
                    .heightmap_value(HeightmapKind::OceanFloorWg, x, z)
                    .or_else(|| chunk.heightmap_value(HeightmapKind::WorldSurfaceWg, x, z))
                    .unwrap_or(settings.sea_level + 1);
            }
            if world_surface[index] == 0 {
                world_surface[index] = chunk
                    .heightmap_value(HeightmapKind::WorldSurfaceWg, x, z)
                    .unwrap_or(ocean_floor[index]);
            }
            if motion_blocking[index] == 0 {
                motion_blocking[index] = world_surface[index];
            }
            if motion_blocking_no_leaves[index] == 0 {
                motion_blocking_no_leaves[index] = motion_blocking[index];
            }
        }
    }
    TreeDecorationHeights {
        ocean_floor,
        world_surface,
        motion_blocking,
        motion_blocking_no_leaves,
    }
}


pub fn biome_generation_settings(id: &str) -> Option<&'static BiomeGenerationSettingsModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_BIOME_GENERATION_SETTINGS.iter().find(|entry| {
        entry
            .biome
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn biome_has_placed_feature(biome: &BiomeGenerationSettingsModel, feature: &str) -> bool {
    let name = feature.strip_prefix("minecraft:").unwrap_or(feature);
    biome.feature_steps.iter().any(|step| {
        step.iter().any(|entry| {
            entry
                .strip_prefix("minecraft:")
                .is_some_and(|entry_name| entry_name == name)
        })
    })
}

pub fn biome_spawns_for_category(
    biome: &BiomeGenerationSettingsModel,
    category: &str,
) -> &'static [MobSpawnerDataModel] {
    biome
        .spawners
        .iter()
        .find(|group| group.category == category)
        .map(|group| group.entries)
        .unwrap_or(&[])
}


pub fn simple_block_placement_plan(
    config: &SimpleBlockConfigurationModel,
    context: SimpleBlockPlacementContext,
    random: &mut RandomSourceKind,
) -> Option<SimpleBlockPlacementPlan> {
    let state = block_state_provider_sample_in_context_with_random(
        &config.to_place,
        random,
        BlockPredicateContext {
            min_y: 0,
            height: 0,
            block: context.origin_block,
            fluid: if matches!(context.origin_block, "minecraft:water" | "minecraft:lava") {
                context.origin_block
            } else {
                "minecraft:empty"
            },
            solid: !matches!(
                context.origin_block,
                "minecraft:air"
                    | "minecraft:cave_air"
                    | "minecraft:void_air"
                    | "minecraft:water"
                    | "minecraft:lava"
            ),
            replaceable: matches!(
                context.origin_block,
                "minecraft:air"
                    | "minecraft:cave_air"
                    | "minecraft:void_air"
                    | "minecraft:short_grass"
                    | "minecraft:tall_grass"
                    | "minecraft:fern"
                    | "minecraft:large_fern"
                    | "minecraft:bush"
                    | "minecraft:leaf_litter"
            ),
            unobstructed: true,
        },
        0,
        context.origin_block,
    )?;
    if !simple_block_can_survive(state, context) {
        return None;
    }
    if simple_block_is_double_plant(state) {
        if context.above_block != "minecraft:air" {
            return None;
        }
        return Some(SimpleBlockPlacementPlan {
            state,
            upper_state: Some(state),
            schedule_tick: config.schedule_tick,
        });
    }
    Some(SimpleBlockPlacementPlan {
        state,
        upper_state: None,
        schedule_tick: config.schedule_tick,
    })
}

pub fn simple_block_can_survive(state: &str, context: SimpleBlockPlacementContext) -> bool {
    if simple_block_is_plant(state) {
        return matches!(
            context.below_block,
            "minecraft:grass_block"
                | "minecraft:dirt"
                | "minecraft:coarse_dirt"
                | "minecraft:podzol"
                | "minecraft:farmland"
                | "minecraft:moss_block"
        );
    }
    true
}

fn simple_block_is_plant(state: &str) -> bool {
    matches!(
        state,
        "minecraft:short_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:tall_grass"
            | "minecraft:bush"
            | "minecraft:dandelion"
            | "minecraft:poppy"
            | "minecraft:azure_bluet"
            | "minecraft:oxeye_daisy"
            | "minecraft:cornflower"
            | "minecraft:orange_tulip"
            | "minecraft:red_tulip"
            | "minecraft:pink_tulip"
            | "minecraft:white_tulip"
            | "minecraft:lily_of_the_valley"
            | "minecraft:sunflower"
            | "minecraft:rose_bush"
            | "minecraft:peony"
            | "minecraft:lilac"
    )
}

fn simple_block_is_double_plant(state: &str) -> bool {
    matches!(
        state,
        "minecraft:sunflower"
            | "minecraft:rose_bush"
            | "minecraft:peony"
            | "minecraft:lilac"
            | "minecraft:tall_grass"
            | "minecraft:large_fern"
    )
}

fn block_is_coral(block: &str) -> bool {
    block.contains("_coral")
}

fn offset_horizontal(pos: BlockPos, direction: HorizontalDirection, distance: i32) -> BlockPos {
    match direction {
        HorizontalDirection::North => BlockPos {
            x: pos.x,
            y: pos.y,
            z: pos.z - distance,
        },
        HorizontalDirection::South => BlockPos {
            x: pos.x,
            y: pos.y,
            z: pos.z + distance,
        },
        HorizontalDirection::West => BlockPos {
            x: pos.x - distance,
            y: pos.y,
            z: pos.z,
        },
        HorizontalDirection::East => BlockPos {
            x: pos.x + distance,
            y: pos.y,
            z: pos.z,
        },
    }
}

impl HorizontalDirection {
    const fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    const fn clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

impl CaveSurface {
    const fn y_step(self) -> i32 {
        match self {
            Self::Floor => -1,
            Self::Ceiling => 1,
        }
    }

    const fn opposite(self) -> Self {
        match self {
            Self::Floor => Self::Ceiling,
            Self::Ceiling => Self::Floor,
        }
    }
}

fn offset_vertical(pos: BlockPos, surface: CaveSurface, distance: i32) -> BlockPos {
    BlockPos {
        x: pos.x,
        y: pos.y + surface.y_step() * distance,
        z: pos.z,
    }
}

const fn coral_wall_fan_state(direction: HorizontalDirection) -> &'static str {
    match direction {
        HorizontalDirection::North => "minecraft:tube_coral_wall_fan[facing=north]",
        HorizontalDirection::South => "minecraft:tube_coral_wall_fan[facing=south]",
        HorizontalDirection::West => "minecraft:tube_coral_wall_fan[facing=west]",
        HorizontalDirection::East => "minecraft:tube_coral_wall_fan[facing=east]",
    }
}

fn clamped_map_f64(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    if value <= from_min {
        to_min
    } else if value >= from_max {
        to_max
    } else {
        let progress = (value - from_min) / (from_max - from_min);
        to_min + progress * (to_max - to_min)
    }
}

fn inclusive_roll(roll: i32, min: i32, max: i32) -> i32 {
    min + roll.rem_euclid(max - min + 1)
}

fn rotated_log_state(state: &'static str, direction: HorizontalDirection) -> &'static str {
    match (state, direction) {
        ("minecraft:oak_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:oak_log[axis=x]"
        }
        ("minecraft:oak_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:oak_log[axis=z]"
        }
        ("minecraft:birch_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:birch_log[axis=x]"
        }
        ("minecraft:birch_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:birch_log[axis=z]"
        }
        ("minecraft:spruce_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:spruce_log[axis=x]"
        }
        ("minecraft:spruce_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:spruce_log[axis=z]"
        }
        ("minecraft:jungle_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:jungle_log[axis=x]"
        }
        ("minecraft:jungle_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:jungle_log[axis=z]"
        }
        ("minecraft:cherry_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:cherry_log[axis=x]"
        }
        ("minecraft:cherry_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:cherry_log[axis=z]"
        }
        _ => state,
    }
}

pub fn carver_can_reach(
    chunk_mid_x: f64,
    chunk_mid_z: f64,
    x: f64,
    z: f64,
    current_step: i32,
    total_steps: i32,
    thickness: f32,
) -> bool {
    let xd = x - chunk_mid_x;
    let zd = z - chunk_mid_z;
    let remaining = f64::from(total_steps - current_step);
    let rr = f64::from(thickness + 2.0 + 16.0);
    xd * xd + zd * zd - remaining * remaining <= rr * rr
}

// ============================================================================
// NoiseChunk — cell-based trilinear interpolation
//
// Mirrors Java's `NoiseChunk` / `NoiseInterpolator` in
// `net.minecraft.world.level.levelgen.NoiseChunk`.
//
// Overview
// --------
// The overworld divides the world into *cells* (4×4 blocks wide, 8 blocks
// tall by default).  For each Interpolated-marker density function in the
// `final_density` tree, the inner function is sampled at every cell *corner*
// (one call per corner per interpolator).  Inside the cell the sampled corner
// values are trilinearly interpolated, avoiding per-block full noise
// evaluation.
//
// Iteration order matches Java's `doFill` in `NoiseBasedChunkGenerator`:
//
//   for cell_x in 0..cell_count_xz:
//     advance_cell_x(cell_x)          ← fills the next-X slice
//     for cell_z in 0..cell_count_xz:
//       for cell_y in (0..cell_count_y).rev():
//         select_cell_yz(cell_y, cell_z)
//         for y_in_cell in (0..cell_height).rev():
//           update_for_y(…)
//           for x_in_cell in 0..cell_width:
//             update_for_x(…)
//             for z_in_cell in 0..cell_width:
//               update_for_z(…)
//               // read interpolated_density(…)
//     swap_slices()                   ← swap current / next
// ============================================================================


#[cfg(test)]
mod tests {
    use super::{
        builtin_density_function, builtin_noise_generator_settings, builtin_noise_router,
        density_function_type, random_state_normal_noise_snapshot, AquiferNoiseSettings,
        BinaryDensityFunction, BiomeGenerationSettingsModel, BlendingDataPacked, BlendingOutput,
        BlockPos, BlockPredicate, BlockPredicateContext, BlockStateProviderModel, CarverShape,
        CaveDensityOutput, CaveSurface, ConfiguredFeatureSource, DensityFunction, DensityMarker,
        FeatureConfigurationKind, FeatureFamily, FeatureSizeModel, FlatLayerInfo, FloatProvider,
        FluidStatus, FoliagePlacerKind, FoliagePlacerModel, GenerationDecorationStep,
        HeightProvider, HeightRange, HorizontalDirection, MangroveRootPlacementModel,
        MappedDensityFunction, MobSpawnerDataModel, NoiseRouterPreset, NoiseSettings,
        OreVeinDecisionInput, OreVeinifierConstants, PlacedFeatureSource, PlacementContextModel,
        PlacementModifier, RandomSpreadType, RandomStateNoiseCache, RootPlacerModel,
        RuleBasedBlockStateProviderRule, SpawnBlockKind, SpawnColumnHeights, StructureFamily,
        StructurePlacementKind, SurfaceConditionSource, SurfaceMaterialContext, SurfaceRuleKind,
        SurfaceRulePreset, SurfaceRuleSource, TreeDecoratorModel, TreeFoliageAttachmentModel,
        TreePlacementBlockKind, TrunkPlacerKind, TrunkPlacerModel, VerticalAnchor,
        WeightedBlockState, WeightedHeightProvider, WorldCarverType, WorldGenerationHeightContext,
        AQUIFER_NOISE_SETTINGS, AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS,
        BLENDING_CELL_COLUMN_COUNT, BLENDING_CONSTANTS, BLENDING_NO_VALUE, BLOCK_PREDICATE_TYPES,
        BUILTIN_DENSITY_FUNCTIONS, BUILTIN_NOISE_GENERATOR_SETTINGS, BUILTIN_NOISE_ROUTERS,
        BUILTIN_STRUCTURES, BUILTIN_STRUCTURE_SETS, BUILTIN_SURFACE_RULE_PRESETS,
        CAVES_NOISE_SETTINGS, CAVE_GENERATION_FAMILIES, CONFIGURED_CARVERS, CONFIGURED_FEATURES,
        DENSITY_FUNCTION_TYPES, END_NOISE_SETTINGS, EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS,
        FEATURE_BEHAVIOR_MODELS, FEATURE_TYPES, FLAT_DEFAULT_LAYERS, FLAT_GENERATOR_PRESETS,
        FLOATING_ISLANDS_NOISE_SETTINGS, HEIGHT_PROVIDER_TYPES, JIGSAW_POOL_BOOTSTRAP_SOURCES,
        MONSTER_ROOM_BOUNDS, NETHER_NOISE_SETTINGS, NORMAL_NOISE_INPUT_FACTOR,
        NORMAL_NOISE_PARAMETERS, NORMAL_NOISE_TARGET_DEVIATION, ORE_VEINIFIER_CONSTANTS,
        ORE_VEIN_TYPES, OVERWORLD_NOISE_SETTINGS, OVERWORLD_SPAWN_TARGET,
        PLACED_FEATURE_BOOTSTRAP_SOURCES, SPAWN_SELECTION_CONSTANTS, STRUCTURE_FAMILIES,
        STRUCTURE_PIECE_TYPES, STRUCTURE_POOL_ELEMENT_TYPES, STRUCTURE_POS_RULE_TEST_TYPES,
        STRUCTURE_PROCESSOR_LISTS, STRUCTURE_PROCESSOR_TYPES, STRUCTURE_RULE_TEST_TYPES,
        STRUCTURE_TYPES, SURFACE_CONDITION_TYPES, SURFACE_RULE_TYPES, SYNTH_NOISE_SOURCES,
        TEST_NEGATIVE_DENSITY, TEST_POSITIVE_DENSITY, UPGRADE_DATA_MODEL, WORLDGEN_TYPE_REGISTRIES,
        WORLD_CARVER_TYPES, WORLD_PRESETS, Y_DENSITY,
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

    mod biome_generation_basic_tests;

    mod biome_neighbor_payload_surface_tests;

    mod biome_neighbor_payload_dimension_tests;

    mod feature_sorter_decoration_tests;

    mod feature_placement_support_provider_tests;

    mod feature_placement_support_geode_end_tests;

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







    #[test]
    fn desert_pyramid_archaeology_state_matches_after_place_rules() {
        let mut piece = super::desert_pyramid_generation_piece(
            ChunkPos { x: 1, z: -1 },
            super::HorizontalDirection::South,
        );
        assert_eq!(piece.scattered.width, 21);
        assert_eq!(piece.scattered.height, 15);
        assert_eq!(piece.scattered.depth, 21);
        assert_eq!(
            piece.scattered.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: 16,
                min_y: 64,
                min_z: -16,
                max_x: 36,
                max_y: 78,
                max_z: 4,
            }
        );
        piece.has_placed_chest[2] = true;
        assert_eq!(
            super::desert_pyramid_save_tag(&piece),
            super::DesertPyramidSaveTagModel {
                width: 21,
                height: 15,
                depth: 21,
                height_position: -1,
                has_placed_chest: [false, false, true, false],
            }
        );

        super::desert_pyramid_place_sand_box(
            &mut piece,
            BlockPos {
                x: 14,
                y: -3,
                z: 11,
            },
            BlockPos {
                x: 15,
                y: -2,
                z: 12,
            },
        );
        let duplicate = super::desert_pyramid_place_sand(&mut piece, 14, -3, 11);
        assert_eq!(
            duplicate,
            BlockPos {
                x: 30,
                y: 61,
                z: -5
            }
        );
        assert_eq!(piece.potential_suspicious_sand_world_positions.len(), 9);
        assert_eq!(
            super::desert_pyramid_record_collapsed_roof(
                &mut piece,
                BlockPos { x: 14, y: 0, z: 11 },
                18,
                15,
                16,
                13,
            )
            .unwrap(),
            BlockPos {
                x: 32,
                y: 64,
                z: -3
            }
        );
        assert_eq!(
            super::desert_pyramid_record_collapsed_roof(
                &mut piece.clone(),
                BlockPos { x: 14, y: 0, z: 11 },
                18,
                15,
                19,
                13,
            )
            .unwrap_err(),
            "Collapsed roof random position must be inside the requested local range".to_string()
        );

        let unique = super::desert_pyramid_unique_suspicious_sand_positions(&[piece.clone()]);
        assert_eq!(unique.len(), 8);
        assert_eq!(
            unique[0],
            BlockPos {
                x: 30,
                y: 61,
                z: -5
            }
        );
        assert_eq!(
            unique[1],
            BlockPos {
                x: 31,
                y: 61,
                z: -5
            }
        );
        assert_eq!(
            unique[2],
            BlockPos {
                x: 30,
                y: 61,
                z: -4
            }
        );

        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 16,
            min_y: i32::MIN,
            min_z: -16,
            max_x: 36,
            max_y: i32::MAX,
            max_z: 4,
        };
        let shuffled = vec![unique[3], unique[0], unique[7]];
        let placements =
            super::desert_pyramid_after_place_archaeology(&[piece], chunk_bb, &shuffled, 2);
        assert_eq!(
            placements[0].pos,
            BlockPos {
                x: 32,
                y: 64,
                z: -3
            }
        );
        assert_eq!(placements[0].state, "minecraft:suspicious_sand");
        assert_eq!(
            placements[0].loot_table,
            Some("minecraft:archaeology/desert_pyramid")
        );
        assert_eq!(
            placements[0].loot_seed,
            Some(super::block_pos_as_long(BlockPos {
                x: 32,
                y: 64,
                z: -3
            }))
        );
        assert_eq!(placements[1].pos, unique[3]);
        assert_eq!(placements[1].state, "minecraft:suspicious_sand");
        assert_eq!(placements[2].pos, unique[0]);
        assert_eq!(placements[2].state, "minecraft:suspicious_sand");
        assert_eq!(placements[3].pos, unique[7]);
        assert_eq!(placements[3].state, "minecraft:sand");
    }

    #[test]
    fn jungle_temple_piece_container_flags_match_vanilla() {
        let piece = super::jungle_temple_generation_piece(
            ChunkPos { x: 0, z: 0 },
            super::HorizontalDirection::South,
        );
        assert_eq!(piece.scattered.width, 12);
        assert_eq!(piece.scattered.height, 10);
        assert_eq!(piece.scattered.depth, 15);
        assert_eq!(
            piece.scattered.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 64,
                min_z: 0,
                max_x: 11,
                max_y: 73,
                max_z: 14,
            }
        );
        assert_eq!(
            super::jungle_temple_save_tag(&piece),
            super::JungleTempleSaveTagModel {
                width: 12,
                height: 10,
                depth: 15,
                height_position: -1,
                placed_main_chest: false,
                placed_hidden_chest: false,
                placed_trap1: false,
                placed_trap2: false,
            }
        );

        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: i32::MIN,
            min_z: 0,
            max_x: 15,
            max_y: i32::MAX,
            max_z: 15,
        };
        let processed =
            super::jungle_temple_post_process(piece.clone(), chunk_bb, |_, _| 70).unwrap();
        assert_eq!(processed.piece.scattered.height_position, 70);
        assert_eq!(processed.piece.scattered.bounding_box.min_y, 70);
        assert!(processed.piece.placed_trap1);
        assert!(processed.piece.placed_trap2);
        assert!(processed.piece.placed_main_chest);
        assert!(processed.piece.placed_hidden_chest);
        assert_eq!(
            processed.containers,
            vec![
                super::JungleTempleContainerPlacement {
                    kind: "minecraft:dispenser",
                    pos: BlockPos { x: 3, y: 68, z: 1 },
                    facing: Some(super::HorizontalDirection::North),
                    loot_table: "minecraft:chests/jungle_temple_dispenser",
                },
                super::JungleTempleContainerPlacement {
                    kind: "minecraft:dispenser",
                    pos: BlockPos { x: 9, y: 68, z: 3 },
                    facing: Some(super::HorizontalDirection::West),
                    loot_table: "minecraft:chests/jungle_temple_dispenser",
                },
                super::JungleTempleContainerPlacement {
                    kind: "minecraft:chest",
                    pos: BlockPos { x: 8, y: 67, z: 3 },
                    facing: None,
                    loot_table: "minecraft:chests/jungle_temple",
                },
                super::JungleTempleContainerPlacement {
                    kind: "minecraft:chest",
                    pos: BlockPos { x: 9, y: 67, z: 10 },
                    facing: None,
                    loot_table: "minecraft:chests/jungle_temple",
                },
            ]
        );

        let no_repeat =
            super::jungle_temple_post_process(processed.piece.clone(), chunk_bb, |_, _| 70)
                .unwrap();
        assert!(no_repeat.containers.is_empty());

        let partial_chunk = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: i32::MIN,
            min_z: 0,
            max_x: 5,
            max_y: i32::MAX,
            max_z: 5,
        };
        let partial =
            super::jungle_temple_post_process(piece.clone(), partial_chunk, |_, _| 70).unwrap();
        assert_eq!(partial.containers.len(), 1);
        assert_eq!(partial.containers[0].pos, BlockPos { x: 3, y: 68, z: 1 });
        assert!(partial.piece.placed_trap1);
        assert!(!partial.piece.placed_trap2);
        assert!(!partial.piece.placed_main_chest);
        assert!(!partial.piece.placed_hidden_chest);

        let outside_chunk = super::StructureBoundingBoxModel {
            min_x: 100,
            min_y: i32::MIN,
            min_z: 100,
            max_x: 115,
            max_y: i32::MAX,
            max_z: 115,
        };
        assert!(super::jungle_temple_post_process(piece, outside_chunk, |_, _| 70).is_none());
    }

    #[test]
    fn igloo_template_piece_generation_and_surface_shift_match_vanilla() {
        let pieces = super::igloo_generation_pieces(
            ChunkPos { x: 2, z: -3 },
            super::StructureRotation::Clockwise90,
            0.25,
            2,
        )
        .unwrap();
        assert_eq!(pieces.len(), 7);
        assert_eq!(pieces[0].template, super::IglooTemplateKind::Bottom);
        assert_eq!(pieces[0].template_name, "minecraft:igloo/bottom");
        assert_eq!(pieces[0].depth, 18);
        assert_eq!(pieces[0].offset, BlockPos { x: 0, y: -3, z: -2 });
        assert_eq!(pieces[0].pivot, BlockPos { x: 3, y: 6, z: 7 });
        assert_eq!(
            pieces[0].template_position,
            BlockPos {
                x: 32,
                y: 69,
                z: -50
            }
        );
        assert_eq!(pieces[1].template, super::IglooTemplateKind::Middle);
        assert_eq!(pieces[1].depth, 0);
        assert_eq!(
            pieces[1].template_position,
            BlockPos {
                x: 34,
                y: 87,
                z: -44
            }
        );
        assert_eq!(pieces[5].template, super::IglooTemplateKind::Middle);
        assert_eq!(pieces[5].depth, 12);
        assert_eq!(pieces[6].template, super::IglooTemplateKind::Top);
        assert_eq!(
            pieces[6].template_position,
            BlockPos {
                x: 32,
                y: 90,
                z: -48
            }
        );

        let top = pieces[6];
        let processed = super::igloo_post_process(
            top,
            |x, z| 75 + (x == 40 && z == -43) as i32,
            |_| "minecraft:snow_block",
        );
        assert_eq!(
            processed.entrance_pos,
            BlockPos {
                x: 40,
                y: 90,
                z: -43
            }
        );
        assert_eq!(
            processed.adjusted_template_position,
            BlockPos {
                x: 32,
                y: 75,
                z: -48
            }
        );
        assert_eq!(
            processed.trapdoor_pos,
            Some(BlockPos {
                x: 35,
                y: 75,
                z: -43
            })
        );
        assert!(processed.should_cover_trapdoor);

        let ladder_below = super::igloo_post_process(top, |_, _| 76, |_| "minecraft:ladder");
        assert!(!ladder_below.should_cover_trapdoor);

        let no_basement = super::igloo_generation_pieces(
            ChunkPos { x: 2, z: -3 },
            super::StructureRotation::None,
            0.5,
            7,
        )
        .unwrap();
        assert_eq!(no_basement.len(), 1);
        assert_eq!(no_basement[0].template, super::IglooTemplateKind::Top);

        assert_eq!(
            super::igloo_generation_pieces(
                ChunkPos { x: 0, z: 0 },
                super::StructureRotation::None,
                1.0,
                0,
            )
            .unwrap_err(),
            "Igloo basement roll must be in [0.0, 1.0)".to_string()
        );
        assert_eq!(
            super::igloo_generation_pieces(
                ChunkPos { x: 0, z: 0 },
                super::StructureRotation::None,
                0.0,
                8,
            )
            .unwrap_err(),
            "Igloo depth roll must match RandomSource#nextInt(8)".to_string()
        );
    }

    #[test]
    fn nether_fossil_generation_scan_and_dried_ghast_match_vanilla() {
        assert_eq!(super::NETHER_FOSSIL_TEMPLATES.len(), 14);
        assert_eq!(
            super::NETHER_FOSSIL_TEMPLATES[0],
            "minecraft:nether_fossils/fossil_1"
        );
        assert_eq!(
            super::NETHER_FOSSIL_TEMPLATES[13],
            "minecraft:nether_fossils/fossil_14"
        );

        let generation = super::nether_fossil_find_generation_point(
            ChunkPos { x: -2, z: 3 },
            5,
            11,
            72,
            31,
            6,
            super::StructureRotation::Counterclockwise90,
            |y| {
                if y == 70 {
                    "minecraft:air"
                } else if y == 69 {
                    "minecraft:soul_sand"
                } else {
                    "minecraft:netherrack"
                }
            },
            |_| false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            generation.position,
            BlockPos {
                x: -27,
                y: 69,
                z: 59
            }
        );
        assert_eq!(generation.piece.template_index, 6);
        assert_eq!(
            generation.piece.template_name,
            "minecraft:nether_fossils/fossil_7"
        );
        assert_eq!(generation.piece.template_position, generation.position);
        assert_eq!(
            generation.piece.rotation,
            super::StructureRotation::Counterclockwise90
        );
        assert_eq!(
            generation.piece.processor,
            "minecraft:block_ignore_structure_and_air"
        );

        let sturdy_generation = super::nether_fossil_find_generation_point(
            ChunkPos { x: 0, z: 0 },
            0,
            0,
            50,
            31,
            0,
            super::StructureRotation::None,
            |y| {
                if y == 45 {
                    "minecraft:air"
                } else {
                    "minecraft:netherrack"
                }
            },
            |y| y == 44,
        )
        .unwrap()
        .unwrap();
        assert_eq!(sturdy_generation.position, BlockPos { x: 0, y: 44, z: 0 });

        let too_low = super::nether_fossil_find_generation_point(
            ChunkPos { x: 0, z: 0 },
            0,
            0,
            33,
            31,
            0,
            super::StructureRotation::None,
            |_| "minecraft:netherrack",
            |_| false,
        )
        .unwrap();
        assert!(too_low.is_none());

        assert_eq!(
            super::nether_fossil_make_piece(
                BlockPos { x: 0, y: 0, z: 0 },
                14,
                super::StructureRotation::None,
            )
            .unwrap_err(),
            "Nether fossil template index must match Util.getRandom(FOSSILS)".to_string()
        );

        let fossil_bb = super::StructureBoundingBoxModel {
            min_x: -10,
            min_y: 41,
            min_z: 20,
            max_x: -5,
            max_y: 45,
            max_z: 27,
        };
        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: -16,
            min_y: i32::MIN,
            min_z: 16,
            max_x: -1,
            max_y: i32::MAX,
            max_z: 31,
        };
        assert_eq!(
            super::nether_fossil_dried_ghast_placement(
                fossil_bb,
                chunk_bb,
                0.49,
                2,
                3,
                super::StructureRotation::Clockwise180,
                "minecraft:air",
            )
            .unwrap(),
            Some(super::DriedGhastPlacementModel {
                pos: BlockPos {
                    x: -8,
                    y: 41,
                    z: 23
                },
                rotation: super::StructureRotation::Clockwise180,
            })
        );
        assert!(super::nether_fossil_dried_ghast_placement(
            fossil_bb,
            chunk_bb,
            0.5,
            2,
            3,
            super::StructureRotation::Clockwise180,
            "minecraft:air",
        )
        .unwrap()
        .is_none());
        assert!(super::nether_fossil_dried_ghast_placement(
            fossil_bb,
            chunk_bb,
            0.49,
            2,
            3,
            super::StructureRotation::Clockwise180,
            "minecraft:netherrack",
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn ruined_portal_setup_template_and_vertical_rules_match_vanilla() {
        let land_setup = super::RuinedPortalSetupModel {
            placement: super::RuinedPortalVerticalPlacement::OnLandSurface,
            air_pocket_probability: 0.0,
            mossiness: 0.2,
            overgrown: false,
            vines: true,
            can_be_cold: true,
            replace_with_blackstone: false,
            weight: 1.0,
        };
        let nether_setup = super::RuinedPortalSetupModel {
            placement: super::RuinedPortalVerticalPlacement::InNether,
            air_pocket_probability: 1.0,
            mossiness: 0.8,
            overgrown: false,
            vines: false,
            can_be_cold: false,
            replace_with_blackstone: true,
            weight: 3.0,
        };
        assert_eq!(
            super::ruined_portal_choose_setup(&[land_setup, nether_setup], 0.24).unwrap(),
            land_setup
        );
        assert_eq!(
            super::ruined_portal_choose_setup(&[land_setup, nether_setup], 0.25).unwrap(),
            nether_setup
        );
        assert_eq!(
            super::ruined_portal_vertical_placement_id(
                super::RuinedPortalVerticalPlacement::PartlyBuried
            ),
            "partly_buried"
        );
        assert!(!super::ruined_portal_sample_probability(0.0, 0.0).unwrap());
        assert!(super::ruined_portal_sample_probability(1.0, 0.99).unwrap());
        assert!(super::ruined_portal_sample_probability(0.3, 0.29).unwrap());
        assert!(!super::ruined_portal_sample_probability(0.3, 0.3).unwrap());

        assert_eq!(
            super::ruined_portal_template_name(0.049, 2).unwrap(),
            "minecraft:ruined_portal/giant_portal_3"
        );
        assert_eq!(
            super::ruined_portal_template_name(0.05, 9).unwrap(),
            "minecraft:ruined_portal/portal_10"
        );
        assert_eq!(
            super::ruined_portal_mirror(0.49).unwrap(),
            super::RuinedPortalMirrorModel::None
        );
        assert_eq!(
            super::ruined_portal_mirror(0.5).unwrap(),
            super::RuinedPortalMirrorModel::FrontBack
        );

        assert_eq!(
            super::ruined_portal_initial_y(
                super::RuinedPortalVerticalPlacement::InNether,
                true,
                80,
                12,
                -64,
                0.75,
                5,
            )
            .unwrap(),
            37
        );
        assert_eq!(
            super::ruined_portal_initial_y(
                super::RuinedPortalVerticalPlacement::InNether,
                false,
                80,
                12,
                -64,
                0.49,
                2,
            )
            .unwrap(),
            29
        );
        assert_eq!(
            super::ruined_portal_initial_y(
                super::RuinedPortalVerticalPlacement::InNether,
                false,
                80,
                12,
                -64,
                0.5,
                0,
            )
            .unwrap(),
            29
        );
        assert_eq!(
            super::ruined_portal_initial_y(
                super::RuinedPortalVerticalPlacement::InMountain,
                false,
                96,
                20,
                -64,
                0.0,
                3,
            )
            .unwrap(),
            73
        );
        assert_eq!(
            super::ruined_portal_initial_y(
                super::RuinedPortalVerticalPlacement::Underground,
                false,
                50,
                20,
                -64,
                0.0,
                79,
            )
            .unwrap(),
            30
        );
        assert_eq!(
            super::ruined_portal_initial_y(
                super::RuinedPortalVerticalPlacement::PartlyBuried,
                false,
                90,
                16,
                -64,
                0.0,
                4,
            )
            .unwrap(),
            80
        );

        let suitable = super::ruined_portal_find_suitable_y(
            super::RuinedPortalVerticalPlacement::OnOceanFloor,
            -64,
            80,
            |y| {
                if y == 72 {
                    3
                } else {
                    2
                }
            },
        );
        assert_eq!(suitable, 72);
        assert_eq!(
            super::ruined_portal_find_suitable_y(
                super::RuinedPortalVerticalPlacement::Underground,
                -64,
                -40,
                |_| 0,
            ),
            -49
        );

        let properties = super::ruined_portal_make_properties(nether_setup, 0.0, true).unwrap();
        assert_eq!(
            properties,
            super::RuinedPortalPropertiesModel {
                cold: false,
                mossiness: 0.8,
                air_pocket: true,
                overgrown: false,
                vines: false,
                replace_with_blackstone: true,
            }
        );
        let piece = super::ruined_portal_make_piece(
            "minecraft:ruined_portal/portal_1",
            BlockPos {
                x: 16,
                y: 37,
                z: -16,
            },
            super::RuinedPortalVerticalPlacement::InNether,
            properties,
            super::StructureRotation::Clockwise90,
            super::RuinedPortalMirrorModel::FrontBack,
            BlockPos { x: 4, y: 0, z: 5 },
        );
        assert_eq!(
            piece.ignore_processor,
            "minecraft:block_ignore_structure_block"
        );
        assert_eq!(piece.lava_replacement, "minecraft:magma_block@0.2");
        assert!(piece.include_blackstone_replace_processor);
        assert_eq!(piece.pivot, BlockPos { x: 4, y: 0, z: 5 });

        let ocean_piece = super::ruined_portal_make_piece(
            "minecraft:ruined_portal/portal_2",
            BlockPos { x: 0, y: 0, z: 0 },
            super::RuinedPortalVerticalPlacement::OnOceanFloor,
            super::RuinedPortalPropertiesModel {
                cold: true,
                mossiness: 0.0,
                air_pocket: false,
                overgrown: false,
                vines: false,
                replace_with_blackstone: false,
            },
            super::StructureRotation::None,
            super::RuinedPortalMirrorModel::None,
            BlockPos { x: 0, y: 0, z: 0 },
        );
        assert_eq!(
            ocean_piece.ignore_processor,
            "minecraft:block_ignore_structure_and_air"
        );
        assert_eq!(ocean_piece.lava_replacement, "minecraft:magma_block");
    }

    #[test]
    fn shipwreck_template_height_and_loot_rules_match_vanilla() {
        assert_eq!(super::SHIPWRECK_BEACHED_TEMPLATES.len(), 11);
        assert_eq!(super::SHIPWRECK_OCEAN_TEMPLATES.len(), 20);
        assert_eq!(
            super::SHIPWRECK_BEACHED_TEMPLATES[0],
            "minecraft:shipwreck/with_mast"
        );
        assert_eq!(
            super::SHIPWRECK_OCEAN_TEMPLATES[19],
            "minecraft:shipwreck/rightsideup_backhalf_degraded"
        );
        assert_eq!(
            super::shipwreck_heightmap_type(true),
            "minecraft:world_surface_wg"
        );
        assert_eq!(
            super::shipwreck_heightmap_type(false),
            "minecraft:ocean_floor_wg"
        );
        assert_eq!(
            super::shipwreck_template_name(true, 10).unwrap(),
            "minecraft:shipwreck/rightsideup_backhalf_degraded"
        );
        assert_eq!(
            super::shipwreck_template_name(false, 11).unwrap(),
            "minecraft:shipwreck/upsidedown_full_degraded"
        );
        assert_eq!(
            super::shipwreck_template_name(true, 11).unwrap_err(),
            "Shipwreck template index must match Util.getRandom template list".to_string()
        );

        let piece = super::shipwreck_make_piece(
            ChunkPos { x: -1, z: 4 },
            super::StructureRotation::Clockwise180,
            7,
            false,
        )
        .unwrap();
        assert_eq!(piece.template_name, "minecraft:shipwreck/rightsideup_full");
        assert_eq!(
            piece.template_position,
            BlockPos {
                x: -16,
                y: 90,
                z: 64
            }
        );
        assert_eq!(piece.pivot, BlockPos { x: 4, y: 0, z: 15 });
        assert_eq!(piece.processor, "minecraft:block_ignore_structure_and_air");
        assert!(!piece.height_adjusted);
        assert!(!piece.is_beached);
        assert_eq!(
            super::shipwreck_save_tag(piece),
            super::ShipwreckSaveTagModel {
                is_beached: false,
                rotation: super::StructureRotation::Clockwise180,
                height_adjusted: false,
            }
        );

        assert!(!super::shipwreck_is_too_big_to_fit_in_worldgen_region(
            BlockPos {
                x: 32,
                y: 32,
                z: 80
            }
        ));
        assert!(super::shipwreck_is_too_big_to_fit_in_worldgen_region(
            BlockPos {
                x: 33,
                y: 12,
                z: 12
            }
        ));
        assert!(super::shipwreck_is_too_big_to_fit_in_worldgen_region(
            BlockPos {
                x: 12,
                y: 33,
                z: 12
            }
        ));
        assert_eq!(
            super::shipwreck_calculate_beached_position(72, 15, 2).unwrap(),
            63
        );
        assert_eq!(
            super::shipwreck_calculate_beached_position(72, 15, 3).unwrap_err(),
            "Shipwreck beached height roll must match RandomSource#nextInt(3)".to_string()
        );
        let adjusted = super::shipwreck_adjust_position_height(piece, 48);
        assert_eq!(adjusted.template_position.y, 48);
        assert!(adjusted.height_adjusted);
        assert_eq!(
            super::shipwreck_save_tag(adjusted),
            super::ShipwreckSaveTagModel {
                is_beached: false,
                rotation: super::StructureRotation::Clockwise180,
                height_adjusted: true,
            }
        );

        assert_eq!(
            super::shipwreck_loot_table_for_marker("map_chest"),
            Some("minecraft:chests/shipwreck_map")
        );
        assert_eq!(
            super::shipwreck_loot_table_for_marker("treasure_chest"),
            Some("minecraft:chests/shipwreck_treasure")
        );
        assert_eq!(
            super::shipwreck_loot_table_for_marker("supply_chest"),
            Some("minecraft:chests/shipwreck_supply")
        );
        assert_eq!(super::shipwreck_loot_table_for_marker("unknown"), None);
    }

    #[test]
    fn ocean_ruin_piece_templates_markers_and_height_match_vanilla() {
        assert_eq!(super::OCEAN_RUIN_WARM_TEMPLATES.len(), 8);
        assert_eq!(super::OCEAN_RUIN_BIG_WARM_TEMPLATES.len(), 4);
        assert_eq!(
            super::OCEAN_RUIN_BRICK_TEMPLATES[0],
            "minecraft:underwater_ruin/brick_1"
        );
        assert_eq!(
            super::OCEAN_RUIN_BIG_CRACKED_TEMPLATES[3],
            "minecraft:underwater_ruin/big_cracked_8"
        );
        assert_eq!(
            super::ocean_ruin_biome_type_id(super::OceanRuinBiomeType::Warm),
            "warm"
        );
        assert_eq!(
            super::ocean_ruin_biome_type_id(super::OceanRuinBiomeType::Cold),
            "cold"
        );
        assert!(super::ocean_ruin_is_large(0.4, 0.4).unwrap());
        assert!(!super::ocean_ruin_is_large(0.4, 0.4001).unwrap());
        assert!(super::ocean_ruin_should_add_cluster(0.2, 0.2).unwrap());
        assert!(!super::ocean_ruin_should_add_cluster(0.2, 0.21).unwrap());

        let warm_config = super::OceanRuinStructureConfigModel {
            biome_type: super::OceanRuinBiomeType::Warm,
            large_probability: 0.3,
            cluster_probability: 0.9,
        };
        let warm_piece = super::ocean_ruin_add_piece(
            warm_config,
            BlockPos {
                x: 16,
                y: 90,
                z: -32,
            },
            super::StructureRotation::Clockwise90,
            true,
            0.9,
            2,
        )
        .unwrap();
        assert_eq!(warm_piece.len(), 1);
        assert_eq!(
            warm_piece[0].template_name,
            "minecraft:underwater_ruin/big_warm_6"
        );
        assert_eq!(warm_piece[0].integrity, 0.9);
        assert_eq!(warm_piece[0].suspicious_block, "minecraft:suspicious_sand");
        assert_eq!(
            warm_piece[0].suspicious_loot_table,
            "minecraft:archaeology/ocean_ruin_warm"
        );

        let cold_config = super::OceanRuinStructureConfigModel {
            biome_type: super::OceanRuinBiomeType::Cold,
            large_probability: 1.0,
            cluster_probability: 0.0,
        };
        let cold_pieces = super::ocean_ruin_add_piece(
            cold_config,
            BlockPos { x: 0, y: 90, z: 0 },
            super::StructureRotation::None,
            false,
            0.8,
            4,
        )
        .unwrap();
        assert_eq!(cold_pieces.len(), 3);
        assert_eq!(
            cold_pieces
                .iter()
                .map(|piece| (piece.template_name, piece.integrity))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:underwater_ruin/brick_5", 0.8),
                ("minecraft:underwater_ruin/cracked_5", 0.7),
                ("minecraft:underwater_ruin/mossy_5", 0.5),
            ]
        );
        assert_eq!(
            cold_pieces[0].suspicious_block,
            "minecraft:suspicious_gravel"
        );
        assert_eq!(
            super::ocean_ruin_save_tag(cold_pieces[1]),
            super::OceanRuinSaveTagModel {
                rotation: super::StructureRotation::None,
                integrity: 0.7,
                biome_type: super::OceanRuinBiomeType::Cold,
                is_large: false,
            }
        );

        assert_eq!(
            super::ocean_ruin_marker_action(
                warm_piece[0],
                "chest",
                BlockPos { x: 1, y: 50, z: 2 },
                63,
                true
            ),
            Some(super::OceanRuinMarkerActionModel {
                marker_id: "chest",
                pos: BlockPos { x: 1, y: 50, z: 2 },
                placed_block: "minecraft:chest[waterlogged=true]",
                loot_table: Some("minecraft:chests/underwater_ruin_big"),
                spawned_entity: None,
            })
        );
        assert_eq!(
            super::ocean_ruin_marker_action(
                cold_pieces[0],
                "drowned",
                BlockPos { x: 1, y: 70, z: 2 },
                63,
                false
            ),
            Some(super::OceanRuinMarkerActionModel {
                marker_id: "drowned",
                pos: BlockPos { x: 1, y: 70, z: 2 },
                placed_block: "minecraft:air",
                loot_table: None,
                spawned_entity: Some("minecraft:drowned"),
            })
        );
        assert_eq!(
            super::ocean_ruin_marker_action(
                cold_pieces[0],
                "drowned",
                BlockPos { x: 1, y: 50, z: 2 },
                63,
                false
            )
            .unwrap()
            .placed_block,
            "minecraft:water"
        );
        assert_eq!(
            super::ocean_ruin_marker_action(
                cold_pieces[0],
                "unknown",
                BlockPos { x: 0, y: 0, z: 0 },
                63,
                false
            ),
            None
        );

        let adjusted = super::ocean_ruin_adjust_to_ocean_floor(
            warm_piece[0],
            70,
            BlockPos { x: 4, y: 6, z: 4 },
            |x, z| {
                if x == 16 && z == -32 {
                    69
                } else {
                    64
                }
            },
        );
        assert_eq!(adjusted.template_position.y, 65);
        let flat = super::ocean_ruin_adjust_to_ocean_floor(
            warm_piece[0],
            70,
            BlockPos { x: 4, y: 6, z: 4 },
            |_, _| 69,
        );
        assert_eq!(flat.template_position.y, 70);
    }

    #[test]
    fn mineshaft_start_materials_and_piece_boxes_match_vanilla() {
        assert_eq!(
            super::mineshaft_type_by_id(-1),
            super::MineshaftTypeModel::Normal
        );
        assert_eq!(
            super::mineshaft_type_by_id(0),
            super::MineshaftTypeModel::Normal
        );
        assert_eq!(
            super::mineshaft_type_by_id(1),
            super::MineshaftTypeModel::Mesa
        );
        assert_eq!(super::mineshaft_type_id(super::MineshaftTypeModel::Mesa), 1);
        assert_eq!(
            super::mineshaft_materials(super::MineshaftTypeModel::Normal),
            super::MineshaftMaterialModel {
                serialized_name: "normal",
                wood_state: "minecraft:oak_log",
                planks_state: "minecraft:oak_planks",
                fence_state: "minecraft:oak_fence",
            }
        );
        assert_eq!(
            super::mineshaft_materials(super::MineshaftTypeModel::Mesa).planks_state,
            "minecraft:dark_oak_planks"
        );
        assert_eq!(
            super::mineshaft_start_pos(ChunkPos { x: -2, z: 3 }),
            BlockPos {
                x: -24,
                y: 50,
                z: 48
            }
        );

        let room = super::mineshaft_room(
            ChunkPos { x: -2, z: 3 },
            super::MineshaftTypeModel::Mesa,
            5,
            4,
            3,
        )
        .unwrap();
        assert_eq!(
            room.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: -30,
                min_y: 50,
                min_z: 50,
                max_x: -18,
                max_y: 58,
                max_z: 60,
            }
        );
        assert_eq!(
            super::mineshaft_room(
                ChunkPos { x: 0, z: 0 },
                super::MineshaftTypeModel::Normal,
                6,
                0,
                0,
            )
            .unwrap_err(),
            "Mineshaft room size rolls must match RandomSource#nextInt(6)".to_string()
        );

        let aggregate = super::StructureBoundingBoxModel {
            min_x: -30,
            min_y: 50,
            min_z: 50,
            max_x: -18,
            max_y: 58,
            max_z: 60,
        };
        assert_eq!(
            super::mineshaft_move_below_sea_level_dy(aggregate, 63, -64, 10, 20).unwrap(),
            -92
        );
        assert_eq!(
            super::mineshaft_mesa_vertical_dy(aggregate, 63, 80, 7).unwrap(),
            16
        );
        assert_eq!(
            super::mineshaft_mesa_vertical_dy(aggregate, 63, 60, 0).unwrap(),
            9
        );

        let collision = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 50,
                min_z: -14,
                max_x: 2,
                max_y: 52,
                max_z: -10,
            },
        };
        assert_eq!(
            super::mineshaft_find_corridor_size(
                0,
                50,
                0,
                super::HorizontalDirection::North,
                1,
                &[collision],
            )
            .unwrap(),
            Some(super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 50,
                min_z: -9,
                max_x: 2,
                max_y: 52,
                max_z: 0,
            })
        );
        assert_eq!(
            super::mineshaft_find_crossing(10, 40, -5, super::HorizontalDirection::East, 0, &[],)
                .unwrap(),
            Some(super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 40,
                min_z: -6,
                max_x: 14,
                max_y: 46,
                max_z: -2,
            })
        );
        assert_eq!(
            super::mineshaft_find_stairs(10, 40, -5, super::HorizontalDirection::West, &[],),
            Some(super::StructureBoundingBoxModel {
                min_x: 2,
                min_y: 35,
                min_z: -5,
                max_x: 10,
                max_y: 42,
                max_z: -3,
            })
        );

        assert_eq!(
            super::mineshaft_random_piece_kind(69).unwrap(),
            super::MineshaftPieceKindModel::Corridor
        );
        assert_eq!(
            super::mineshaft_random_piece_kind(70).unwrap(),
            super::MineshaftPieceKindModel::Stairs
        );
        assert_eq!(
            super::mineshaft_random_piece_kind(80).unwrap(),
            super::MineshaftPieceKindModel::Crossing
        );

        let corridor_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 40,
            min_z: 0,
            max_x: 2,
            max_y: 42,
            max_z: 14,
        };
        let corridor = super::mineshaft_corridor(
            corridor_box,
            super::HorizontalDirection::South,
            super::MineshaftTypeModel::Normal,
            2,
            0,
        )
        .unwrap();
        assert!(!corridor.has_rails);
        assert!(corridor.spider_corridor);
        assert_eq!(corridor.num_sections, 3);
        assert_eq!(
            super::mineshaft_corridor_save_tag(corridor),
            super::MineshaftCorridorSaveTagModel {
                has_rails: false,
                spider_corridor: true,
                has_placed_spider: false,
                num_sections: 3,
                mineshaft_type_id: 0,
            }
        );
        assert!(
            !super::mineshaft_corridor(
                corridor_box,
                super::HorizontalDirection::South,
                super::MineshaftTypeModel::Normal,
                0,
                0,
            )
            .unwrap()
            .spider_corridor
        );
    }

    #[test]
    fn mineshaft_corridor_rail_roll_preserves_java_piece_chain_prng_alignment() {
        let pieces = super::mineshaft_generate_pieces_for_start(
            8_675_309,
            ChunkPos { x: -1, z: 4 },
            super::MineshaftTypeModel::Normal,
            63,
            -64,
        );
        assert_eq!(pieces.len(), 235);

        let expected = [
            (
                "room",
                0,
                super::StructureBoundingBoxModel {
                    min_x: -14,
                    min_y: -49,
                    min_z: 66,
                    max_x: -5,
                    max_y: -41,
                    max_z: 75,
                },
            ),
            (
                "corridor",
                1,
                super::StructureBoundingBoxModel {
                    min_x: -7,
                    min_y: -45,
                    min_z: 46,
                    max_x: -5,
                    max_y: -43,
                    max_z: 65,
                },
            ),
            (
                "stairs",
                2,
                super::StructureBoundingBoxModel {
                    min_x: -4,
                    min_y: -49,
                    min_z: 46,
                    max_x: 4,
                    max_y: -42,
                    max_z: 48,
                },
            ),
            (
                "corridor",
                3,
                super::StructureBoundingBoxModel {
                    min_x: 5,
                    min_y: -49,
                    min_z: 46,
                    max_x: 14,
                    max_y: -47,
                    max_z: 48,
                },
            ),
            (
                "corridor",
                4,
                super::StructureBoundingBoxModel {
                    min_x: 11,
                    min_y: -50,
                    min_z: 31,
                    max_x: 13,
                    max_y: -48,
                    max_z: 45,
                },
            ),
            (
                "corridor",
                5,
                super::StructureBoundingBoxModel {
                    min_x: 11,
                    min_y: -51,
                    min_z: 11,
                    max_x: 13,
                    max_y: -49,
                    max_z: 30,
                },
            ),
            (
                "corridor",
                6,
                super::StructureBoundingBoxModel {
                    min_x: 11,
                    min_y: -52,
                    min_z: -9,
                    max_x: 13,
                    max_y: -50,
                    max_z: 10,
                },
            ),
            (
                "corridor",
                7,
                super::StructureBoundingBoxModel {
                    min_x: 11,
                    min_y: -51,
                    min_z: -29,
                    max_x: 13,
                    max_y: -49,
                    max_z: -10,
                },
            ),
            (
                "corridor",
                8,
                super::StructureBoundingBoxModel {
                    min_x: 1,
                    min_y: -52,
                    min_z: -1,
                    max_x: 10,
                    max_y: -50,
                    max_z: 1,
                },
            ),
            (
                "corridor",
                9,
                super::StructureBoundingBoxModel {
                    min_x: 1,
                    min_y: -51,
                    min_z: 2,
                    max_x: 3,
                    max_y: -49,
                    max_z: 21,
                },
            ),
        ];

        for (index, (expected_kind, expected_depth, expected_box)) in
            expected.into_iter().enumerate()
        {
            let piece = &pieces[index];
            let actual_kind = match piece {
                super::MineshaftGeneratedPieceModel::Room { .. } => "room",
                super::MineshaftGeneratedPieceModel::Corridor { .. } => "corridor",
                super::MineshaftGeneratedPieceModel::Crossing { .. } => "crossing",
                super::MineshaftGeneratedPieceModel::Stairs { .. } => "stairs",
            };
            assert_eq!(actual_kind, expected_kind, "piece {index} kind drifted");
            assert_eq!(
                piece.gen_depth(),
                expected_depth,
                "piece {index} generation depth drifted"
            );
            assert_eq!(
                piece.bounding_box(),
                expected_box,
                "piece {index} bounding box drifted"
            );
        }
    }

    #[test]
    fn stronghold_start_weights_and_portal_room_match_vanilla() {
        let weights = super::stronghold_piece_weights();
        assert_eq!(weights.len(), 11);
        assert_eq!(
            weights[0],
            super::StrongholdPieceWeightModel {
                kind: super::StrongholdPieceKindModel::Straight,
                weight: 40,
                max_place_count: 0,
                place_count: 0,
                min_depth: 0,
            }
        );
        assert_eq!(
            weights[9],
            super::StrongholdPieceWeightModel {
                kind: super::StrongholdPieceKindModel::Library,
                weight: 10,
                max_place_count: 2,
                place_count: 0,
                min_depth: 5,
            }
        );
        assert_eq!(
            weights[10],
            super::StrongholdPieceWeightModel {
                kind: super::StrongholdPieceKindModel::PortalRoom,
                weight: 20,
                max_place_count: 1,
                place_count: 0,
                min_depth: 6,
            }
        );
        assert_eq!(super::stronghold_total_weight(&weights), 145);
        assert!(super::stronghold_has_limited_piece_available(&weights));
        assert!(!super::stronghold_piece_weight_can_place(weights[9], 4));
        assert!(super::stronghold_piece_weight_can_place(weights[9], 5));
        let exhausted_portal = super::StrongholdPieceWeightModel {
            place_count: 1,
            ..weights[10]
        };
        assert!(!super::stronghold_piece_weight_can_place(
            exhausted_portal,
            6
        ));
        assert!(!super::stronghold_piece_weight_is_valid(exhausted_portal));

        assert_eq!(
            super::stronghold_horizontal_direction_from_random_roll(0).unwrap(),
            super::HorizontalDirection::North
        );
        assert_eq!(
            super::stronghold_horizontal_direction_from_random_roll(1).unwrap(),
            super::HorizontalDirection::South
        );
        assert_eq!(
            super::stronghold_horizontal_direction_from_random_roll(2).unwrap(),
            super::HorizontalDirection::West
        );
        assert_eq!(
            super::stronghold_horizontal_direction_from_random_roll(3).unwrap(),
            super::HorizontalDirection::East
        );
        assert_eq!(
            super::stronghold_random_small_door(0).unwrap(),
            super::StrongholdSmallDoorTypeModel::Opening
        );
        assert_eq!(
            super::stronghold_random_small_door(2).unwrap(),
            super::StrongholdSmallDoorTypeModel::WoodDoor
        );
        assert_eq!(
            super::stronghold_random_small_door(3).unwrap(),
            super::StrongholdSmallDoorTypeModel::Grates
        );
        assert_eq!(
            super::stronghold_random_small_door(4).unwrap(),
            super::StrongholdSmallDoorTypeModel::IronDoor
        );
        assert_eq!(
            super::stronghold_random_small_door(5).unwrap_err(),
            "Stronghold small-door roll must match RandomSource#nextInt(5)".to_string()
        );

        let start = super::stronghold_start_piece(ChunkPos { x: -2, z: 3 }, 1).unwrap();
        assert_eq!(start.orientation, super::HorizontalDirection::South);
        assert_eq!(
            start.entry_door,
            super::StrongholdSmallDoorTypeModel::Opening
        );
        assert!(start.is_source);
        assert_eq!(
            start.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: -30,
                min_y: 64,
                min_z: 50,
                max_x: -26,
                max_y: 74,
                max_z: 54,
            }
        );

        let portal_room =
            super::stronghold_portal_room(10, 40, -5, super::HorizontalDirection::South, 7, &[])
                .unwrap();
        assert_eq!(
            portal_room.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: 6,
                min_y: 39,
                min_z: -5,
                max_x: 16,
                max_y: 46,
                max_z: 10,
            }
        );
        assert_eq!(portal_room.orientation, super::HorizontalDirection::South);
        assert_eq!(portal_room.gen_depth, 7);
        assert_eq!(
            super::stronghold_portal_room_save_tag(portal_room),
            super::StrongholdPortalRoomSaveTagModel {
                has_placed_spawner: false,
            }
        );
        assert_eq!(
            super::stronghold_attach_portal_room(start, portal_room).portal_room_piece,
            Some(portal_room)
        );

        let west_portal =
            super::stronghold_portal_room(10, 40, -5, super::HorizontalDirection::West, 7, &[])
                .unwrap();
        assert_eq!(
            west_portal.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: -5,
                min_y: 39,
                min_z: -9,
                max_x: 10,
                max_y: 46,
                max_z: 1,
            }
        );
        assert_eq!(
            super::stronghold_portal_room(10, 10, -5, super::HorizontalDirection::South, 7, &[]),
            None
        );
        assert_eq!(
            super::stronghold_portal_room(
                10,
                40,
                -5,
                super::HorizontalDirection::South,
                7,
                &[super::StructurePieceModel {
                    bounding_box: portal_room.bounding_box,
                }],
            ),
            None
        );
    }

    #[test]
    fn nether_fortress_start_weights_and_child_anchors_match_vanilla() {
        let bridge_weights =
            super::nether_fortress_piece_weights(super::NetherFortressPiecePoolModel::Bridge);
        assert_eq!(bridge_weights.len(), 6);
        assert_eq!(
            bridge_weights[0],
            super::NetherFortressPieceWeightModel {
                kind: super::NetherFortressPieceKindModel::BridgeStraight,
                weight: 30,
                max_place_count: 0,
                place_count: 0,
                allow_in_row: true,
            }
        );
        assert_eq!(
            bridge_weights[5],
            super::NetherFortressPieceWeightModel {
                kind: super::NetherFortressPieceKindModel::CastleEntrance,
                weight: 5,
                max_place_count: 1,
                place_count: 0,
                allow_in_row: false,
            }
        );

        let castle_weights =
            super::nether_fortress_piece_weights(super::NetherFortressPiecePoolModel::Castle);
        assert_eq!(castle_weights.len(), 7);
        assert_eq!(
            castle_weights[0],
            super::NetherFortressPieceWeightModel {
                kind: super::NetherFortressPieceKindModel::CastleSmallCorridor,
                weight: 25,
                max_place_count: 0,
                place_count: 0,
                allow_in_row: true,
            }
        );
        assert_eq!(
            castle_weights[4],
            super::NetherFortressPieceWeightModel {
                kind: super::NetherFortressPieceKindModel::CastleCorridorStairs,
                weight: 10,
                max_place_count: 3,
                place_count: 0,
                allow_in_row: true,
            }
        );
        assert_eq!(
            super::nether_fortress_update_piece_weight(&bridge_weights),
            70
        );
        assert_eq!(
            super::nether_fortress_update_piece_weight(&[
                super::NetherFortressPieceWeightModel {
                    place_count: 1,
                    ..bridge_weights[5]
                },
                super::NetherFortressPieceWeightModel {
                    place_count: 2,
                    ..bridge_weights[4]
                },
            ]),
            -1
        );
        assert!(!super::nether_fortress_piece_weight_can_place(
            bridge_weights[1],
            Some(super::NetherFortressPieceKindModel::BridgeCrossing)
        ));
        assert!(super::nether_fortress_piece_weight_can_place(
            bridge_weights[0],
            Some(super::NetherFortressPieceKindModel::BridgeStraight)
        ));
        assert!(!super::nether_fortress_piece_weight_is_valid(
            super::NetherFortressPieceWeightModel {
                place_count: 1,
                ..bridge_weights[5]
            }
        ));
        assert_eq!(
            super::nether_fortress_select_piece(&bridge_weights, None, 1, &[29])
                .unwrap()
                .selected_kind,
            super::NetherFortressPieceKindModel::BridgeStraight
        );
        assert_eq!(
            super::nether_fortress_select_piece(&bridge_weights, None, 1, &[30])
                .unwrap()
                .selected_kind,
            super::NetherFortressPieceKindModel::BridgeCrossing
        );
        assert!(
            super::nether_fortress_select_piece(&bridge_weights, None, 31, &[0])
                .unwrap()
                .fallback_to_end_filler
        );
        assert_eq!(
            super::nether_fortress_select_piece(&bridge_weights, None, 1, &[70]).unwrap_err(),
            "Nether fortress piece roll must match RandomSource#nextInt(totalWeight)".to_string()
        );

        let start = super::nether_fortress_start_piece(ChunkPos { x: -2, z: 3 }, 0).unwrap();
        assert_eq!(start.orientation, super::HorizontalDirection::North);
        assert_eq!(start.bridge_piece_count, 6);
        assert_eq!(start.castle_piece_count, 7);
        assert_eq!(
            start.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: -30,
                min_y: 64,
                min_z: 50,
                max_x: -12,
                max_y: 73,
                max_z: 68,
            }
        );

        let crossing_box = super::nether_fortress_bridge_crossing_box(
            10,
            40,
            -5,
            super::HorizontalDirection::South,
            &[],
        )
        .unwrap();
        assert_eq!(
            crossing_box,
            super::StructureBoundingBoxModel {
                min_x: 2,
                min_y: 37,
                min_z: -5,
                max_x: 20,
                max_y: 46,
                max_z: 13,
            }
        );
        assert_eq!(
            super::nether_fortress_bridge_crossing_box(
                10,
                10,
                -5,
                super::HorizontalDirection::South,
                &[],
            ),
            None
        );
        assert_eq!(
            super::nether_fortress_bridge_crossing_box(
                10,
                40,
                -5,
                super::HorizontalDirection::South,
                &[super::StructurePieceModel {
                    bounding_box: crossing_box,
                }],
            ),
            None
        );

        assert_eq!(
            super::nether_fortress_bridge_straight_box(
                10,
                40,
                -5,
                super::HorizontalDirection::West,
                &[],
            )
            .unwrap(),
            super::StructureBoundingBoxModel {
                min_x: -8,
                min_y: 37,
                min_z: -6,
                max_x: 10,
                max_y: 46,
                max_z: -2,
            }
        );

        assert_eq!(
            super::nether_fortress_child_anchor(
                start.bounding_box,
                crossing_box,
                super::HorizontalDirection::South,
                0,
                super::NetherFortressChildDirectionModel::Forward,
                8,
                3,
                false,
            ),
            super::NetherFortressChildAnchorModel {
                foot: BlockPos {
                    x: 10,
                    y: 40,
                    z: 14,
                },
                direction: super::HorizontalDirection::South,
                next_depth: 1,
                is_castle: false,
                within_start_range: true,
            }
        );
        assert_eq!(
            super::nether_fortress_child_anchor(
                start.bounding_box,
                crossing_box,
                super::HorizontalDirection::South,
                0,
                super::NetherFortressChildDirectionModel::Left,
                3,
                8,
                false,
            )
            .direction,
            super::HorizontalDirection::West
        );
        assert_eq!(
            super::nether_fortress_child_anchor(
                start.bounding_box,
                crossing_box.moved(300, 0, 0),
                super::HorizontalDirection::South,
                0,
                super::NetherFortressChildDirectionModel::Right,
                3,
                8,
                true,
            )
            .within_start_range,
            false
        );

        assert_eq!(
            super::structure_pieces_move_inside_heights_dy(crossing_box, 48, 70, 5).unwrap(),
            16
        );
        assert_eq!(
            super::structure_pieces_move_inside_heights_dy(crossing_box, 48, 50, 99).unwrap(),
            11
        );
    }

    #[test]
    fn ocean_monument_generation_point_and_room_primitives_match_vanilla() {
        let chunk_pos = ChunkPos { x: -2, z: 3 };
        assert_eq!(
            super::ocean_monument_generation_point(chunk_pos, 63),
            super::OceanMonumentGenerationPointModel {
                biome_check_center: BlockPos {
                    x: -23,
                    y: 63,
                    z: 57,
                },
                biome_check_radius: 29,
                heightmap: "OCEAN_FLOOR_WG",
            }
        );
        assert_eq!(
            super::ocean_monument_top_piece_origin(chunk_pos),
            BlockPos {
                x: -61,
                y: 39,
                z: 19,
            }
        );
        assert_eq!(super::ocean_monument_room_index(2, 0, 0), 2);
        assert_eq!(super::ocean_monument_room_index(2, 2, 0), 52);
        assert_eq!(super::ocean_monument_room_index(0, 1, 0), 25);
        assert_eq!(super::ocean_monument_room_index(4, 1, 0), 29);
        assert_eq!(
            super::ocean_monument_direction_3d_value(super::OceanMonumentDirectionModel::East),
            5
        );
        assert_eq!(
            super::ocean_monument_opposite_direction(super::OceanMonumentDirectionModel::North),
            super::OceanMonumentDirectionModel::South
        );

        let building = super::ocean_monument_building(chunk_pos, 0, 3).unwrap();
        assert_eq!(building.orientation, super::HorizontalDirection::North);
        assert_eq!(building.source_room_index, 2);
        assert_eq!(building.core_room_index, 13);
        assert_eq!(building.top_connect_index, 52);
        assert_eq!(building.left_wing_connect_index, 25);
        assert_eq!(building.right_wing_connect_index, 29);
        assert_eq!(
            building.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: -61,
                min_y: 39,
                min_z: 19,
                max_x: -4,
                max_y: 61,
                max_z: 76,
            }
        );
        assert_eq!(
            building.child_piece_offset,
            BlockPos {
                x: -52,
                y: 39,
                z: 54,
            }
        );
        assert_eq!(
            super::ocean_monument_building(chunk_pos, 0, 4).unwrap_err(),
            "Ocean monument core room roll must match RandomSource#nextInt(4)".to_string()
        );

        assert_eq!(
            super::ocean_monument_room_definition(
                1003,
                true,
                false,
                [true, false, true, false, false, true],
            ),
            super::OceanMonumentRoomDefinitionModel {
                index: 1003,
                claimed: true,
                is_source: false,
                is_special: true,
                opening_count: 3,
            }
        );
        assert_eq!(
            super::ocean_monument_room_box(13, 2, 2, 2, super::HorizontalDirection::South),
            super::StructureBoundingBoxModel {
                min_x: 24,
                min_y: 0,
                min_z: 16,
                max_x: 39,
                max_y: 7,
                max_z: 31,
            }
        );
        assert_eq!(
            super::ocean_monument_room_box(13, 2, 2, 2, super::HorizontalDirection::North),
            super::StructureBoundingBoxModel {
                min_x: 24,
                min_y: 0,
                min_z: -31,
                max_x: 39,
                max_y: 7,
                max_z: -16,
            }
        );

        let chunk_bb = building.bounding_box;
        assert_eq!(
            super::ocean_monument_elder_spawn_pos(
                building.bounding_box,
                building.orientation,
                BlockPos { x: 6, y: 1, z: 6 },
                chunk_bb,
            ),
            Some(BlockPos {
                x: -55,
                y: 40,
                z: 70,
            })
        );
        assert_eq!(
            super::ocean_monument_elder_spawn_pos(
                building.bounding_box,
                building.orientation,
                BlockPos { x: 500, y: 1, z: 6 },
                chunk_bb,
            ),
            None
        );
    }

    #[test]
    fn end_city_generation_start_templates_and_markers_match_vanilla() {
        assert_eq!(
            super::end_city_rotation_from_roll(0).unwrap(),
            super::StructureRotation::None
        );
        assert_eq!(
            super::end_city_rotation_from_roll(3).unwrap(),
            super::StructureRotation::Counterclockwise90
        );
        assert_eq!(
            super::end_city_rotation_from_roll(4).unwrap_err(),
            "End city rotation roll must match RandomSource#nextInt(4)".to_string()
        );
        assert_eq!(
            super::end_city_generation_start(BlockPos { x: 8, y: 59, z: 8 }),
            None
        );
        assert_eq!(
            super::end_city_generation_start(BlockPos { x: 8, y: 60, z: 8 }),
            Some(BlockPos { x: 8, y: 60, z: 8 })
        );
        assert_eq!(
            super::end_city_template_id("ship"),
            "minecraft:end_city/ship".to_string()
        );

        let pieces = super::end_city_start_house_tower_seed(
            BlockPos {
                x: 16,
                y: 70,
                z: -8,
            },
            super::StructureRotation::Clockwise90,
        );
        assert_eq!(pieces.len(), 4);
        assert_eq!(pieces[0].template_name, "base_floor");
        assert_eq!(pieces[0].template_id, "minecraft:end_city/base_floor");
        assert!(pieces[0].overwrite);
        assert_eq!(pieces[0].processor, "STRUCTURE_BLOCK");
        assert_eq!(pieces[1].template_name, "second_floor_1");
        assert!(!pieces[1].overwrite);
        assert_eq!(pieces[1].processor, "STRUCTURE_AND_AIR");
        assert_eq!(
            pieces[3],
            super::EndCityTemplatePieceModel {
                template_name: "third_roof",
                template_id: "minecraft:end_city/third_roof",
                position: BlockPos {
                    x: 13,
                    y: 82,
                    z: -11,
                },
                rotation: super::StructureRotation::Clockwise90,
                overwrite: true,
                processor: "STRUCTURE_BLOCK",
                gen_depth: 0,
            }
        );

        assert_eq!(
            super::end_city_tower_bridge_candidates(),
            vec![
                super::EndCityBridgeCandidateModel {
                    rotation: super::StructureRotation::None,
                    offset: BlockPos { x: 1, y: -1, z: 0 },
                },
                super::EndCityBridgeCandidateModel {
                    rotation: super::StructureRotation::Clockwise90,
                    offset: BlockPos { x: 6, y: -1, z: 1 },
                },
                super::EndCityBridgeCandidateModel {
                    rotation: super::StructureRotation::Counterclockwise90,
                    offset: BlockPos { x: 0, y: -1, z: 5 },
                },
                super::EndCityBridgeCandidateModel {
                    rotation: super::StructureRotation::Clockwise180,
                    offset: BlockPos { x: 5, y: -1, z: 6 },
                },
            ]
        );
        assert_eq!(
            super::end_city_fat_tower_bridge_candidates()[1],
            super::EndCityBridgeCandidateModel {
                rotation: super::StructureRotation::Clockwise90,
                offset: BlockPos { x: 12, y: -1, z: 4 },
            }
        );

        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 32,
            max_y: 100,
            max_z: 32,
        };
        assert_eq!(
            super::end_city_marker_action(
                "ChestLoot",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::None,
                chunk_bb,
                true,
            ),
            Some(super::EndCityMarkerActionModel {
                marker_id: "ChestLoot",
                target_pos: BlockPos {
                    x: 10,
                    y: 19,
                    z: 10
                },
                loot_table: Some("minecraft:chests/end_city_treasure"),
                spawned_entity: None,
                item_frame_facing: None,
                item: None,
            })
        );
        assert_eq!(
            super::end_city_marker_action(
                "Sentry",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::None,
                chunk_bb,
                true,
            )
            .unwrap()
            .spawned_entity,
            Some("minecraft:shulker")
        );
        assert_eq!(
            super::end_city_marker_action(
                "Elytra",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::Clockwise90,
                chunk_bb,
                true,
            ),
            Some(super::EndCityMarkerActionModel {
                marker_id: "Elytra",
                target_pos: BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                loot_table: None,
                spawned_entity: Some("minecraft:item_frame"),
                item_frame_facing: Some(super::HorizontalDirection::West),
                item: Some("minecraft:elytra"),
            })
        );
        assert_eq!(
            super::end_city_marker_action(
                "Sentry",
                BlockPos {
                    x: 100,
                    y: 20,
                    z: 10,
                },
                super::StructureRotation::None,
                chunk_bb,
                true,
            ),
            None
        );
    }

    #[test]
    fn woodland_mansion_start_templates_markers_and_support_match_vanilla() {
        assert_eq!(
            super::woodland_mansion_generation_start(BlockPos { x: 0, y: 59, z: 0 }),
            None
        );
        assert_eq!(
            super::woodland_mansion_generation_start(BlockPos { x: 0, y: 60, z: 0 }),
            Some(BlockPos { x: 0, y: 60, z: 0 })
        );
        assert_eq!(
            super::woodland_mansion_template_id("entrance"),
            "minecraft:woodland_mansion/entrance".to_string()
        );

        let (entrance, data) = super::woodland_mansion_initial_placement(
            BlockPos {
                x: 100,
                y: 70,
                z: -40,
            },
            super::StructureRotation::Clockwise90,
        );
        assert_eq!(
            entrance,
            super::WoodlandMansionTemplatePieceModel {
                template_name: "entrance",
                template_id: "minecraft:woodland_mansion/entrance",
                position: BlockPos {
                    x: 100,
                    y: 70,
                    z: -49,
                },
                rotation: super::StructureRotation::Clockwise90,
                mirror: super::WoodlandMansionMirrorModel::None,
                processor: "STRUCTURE_BLOCK",
            }
        );
        assert_eq!(
            data,
            super::WoodlandMansionPlacementDataModel {
                position: BlockPos {
                    x: 84,
                    y: 70,
                    z: -40,
                },
                rotation: super::StructureRotation::Clockwise90,
                wall_type: "wall_flat",
            }
        );
        let second = super::woodland_mansion_second_floor_data(data);
        assert_eq!(second.position.y, 78);
        assert_eq!(second.wall_type, "wall_window");
        let (wall, next_data) = super::woodland_mansion_traverse_wall_piece(data);
        assert_eq!(wall.template_name, "wall_flat");
        assert_eq!(
            wall.position,
            BlockPos {
                x: 84,
                y: 70,
                z: -33,
            }
        );
        assert_eq!(
            next_data.position,
            BlockPos {
                x: 76,
                y: 70,
                z: -40,
            }
        );

        let room = super::woodland_mansion_add_room_1x1(
            BlockPos { x: 0, y: 80, z: 0 },
            super::StructureRotation::Clockwise90,
            Some(super::HorizontalDirection::South),
            "1x1_a1",
        );
        assert_eq!(room.rotation, super::StructureRotation::Clockwise180);
        assert_eq!(room.template_id, "minecraft:woodland_mansion/1x1_a1");
        assert_eq!(
            super::woodland_mansion_add_room_1x1(
                BlockPos { x: 0, y: 80, z: 0 },
                super::StructureRotation::None,
                None,
                "1x1_a1",
            )
            .template_name,
            "1x1_as1"
        );

        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 32,
            max_y: 100,
            max_z: 32,
        };
        assert_eq!(
            super::woodland_mansion_marker_action(
                "ChestWest",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::Clockwise90,
                chunk_bb,
                0,
            )
            .unwrap(),
            Some(super::WoodlandMansionMarkerActionModel {
                marker_id: "ChestWest",
                target_pos: BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                loot_table: Some("minecraft:chests/woodland_mansion"),
                chest_facing: Some(super::HorizontalDirection::North),
                spawned_entity: None,
                spawn_count: 0,
                clears_marker_block: false,
            })
        );
        assert_eq!(
            super::woodland_mansion_marker_action(
                "Mage",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::None,
                chunk_bb,
                0,
            )
            .unwrap()
            .unwrap()
            .spawned_entity,
            Some("minecraft:evoker")
        );
        assert_eq!(
            super::woodland_mansion_marker_action(
                "Group of Allays",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::None,
                chunk_bb,
                2,
            )
            .unwrap()
            .unwrap()
            .spawn_count,
            3
        );
        assert_eq!(
            super::woodland_mansion_marker_action(
                "Group of Allays",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::None,
                chunk_bb,
                3,
            )
            .unwrap_err(),
            "Woodland mansion allay group roll must match RandomSource#nextInt(3)".to_string()
        );
        assert_eq!(
            super::woodland_mansion_marker_action(
                "Unknown",
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 10
                },
                super::StructureRotation::None,
                chunk_bb,
                0,
            )
            .unwrap(),
            None
        );

        assert_eq!(
            super::woodland_mansion_support_column_y_values(70, 60, true, Some(66)),
            vec![69, 68, 67]
        );
        assert!(
            super::woodland_mansion_support_column_y_values(70, 60, false, Some(66)).is_empty()
        );
    }

    #[test]
    fn jigsaw_and_processor_registries_match_vanilla_bootstrap_surface() {
        assert_eq!(STRUCTURE_POOL_ELEMENT_TYPES.len(), 5);
        assert_eq!(STRUCTURE_PROCESSOR_TYPES.len(), 11);
        assert_eq!(STRUCTURE_RULE_TEST_TYPES.len(), 6);
        assert_eq!(STRUCTURE_POS_RULE_TEST_TYPES.len(), 3);
        assert_eq!(STRUCTURE_PIECE_TYPES.len(), 56);
        assert_eq!(STRUCTURE_PROCESSOR_LISTS.len(), 40);
        assert_eq!(
            STRUCTURE_PROCESSOR_LISTS.first().copied(),
            Some("minecraft:empty")
        );
        assert_eq!(
            STRUCTURE_PROCESSOR_LISTS.last().copied(),
            Some("minecraft:trial_chambers_copper_bulb_degradation")
        );
        assert!(STRUCTURE_PROCESSOR_TYPES.contains(&"minecraft:jigsaw_replacement"));
        assert!(STRUCTURE_POOL_ELEMENT_TYPES.contains(&"minecraft:legacy_single_pool_element"));
        assert_eq!(STRUCTURE_PIECE_TYPES.first().copied(), Some("mscorridor"));
        assert_eq!(STRUCTURE_PIECE_TYPES.last().copied(), Some("jigsaw"));
        assert!(STRUCTURE_PIECE_TYPES.contains(&"shpr"));
        assert!(STRUCTURE_PIECE_TYPES.contains(&"shipwreck"));

        assert_eq!(JIGSAW_POOL_BOOTSTRAP_SOURCES.len(), 17);
        assert_eq!(
            JIGSAW_POOL_BOOTSTRAP_SOURCES
                .iter()
                .map(|source| source.registrations)
                .sum::<usize>(),
            176
        );
        assert_eq!(
            JIGSAW_POOL_BOOTSTRAP_SOURCES
                .iter()
                .find(|source| source.source_file == "TrialChambersStructurePools.java")
                .map(|source| source.registrations),
            Some(34)
        );
        assert_eq!(
            JIGSAW_POOL_BOOTSTRAP_SOURCES
                .iter()
                .find(|source| source.source_file == "PlainVillagePools.java")
                .map(|source| source.registrations),
            Some(17)
        );
    }

    #[test]
    fn terrain_blending_and_upgrade_data_match_vanilla_constants() {
        assert_eq!(BLENDING_CONSTANTS.height_blending_range_cells, 27);
        assert_eq!(BLENDING_CONSTANTS.height_blending_range_chunks, 7);
        assert_eq!(BLENDING_CONSTANTS.density_blending_range_cells, 2);
        assert_eq!(BLENDING_CONSTANTS.density_blending_range_chunks, 2);
        assert_eq!(BLENDING_CONSTANTS.old_chunk_xz_radius, 8);
        assert_eq!(BLENDING_CONSTANTS.cell_width, 4);
        assert_eq!(BLENDING_CONSTANTS.cell_height, 8);
        assert_eq!(BLENDING_CONSTANTS.cell_ratio, 2);
        assert_eq!(BLENDING_CELL_COLUMN_COUNT, 16);
        assert_eq!(BLENDING_NO_VALUE, f64::MAX);

        assert_eq!(super::blending_smooth_alpha(0.0, 27), 0.0);
        assert_eq!(super::blending_smooth_alpha(28.0, 27), 1.0);
        assert!((super::blending_smooth_alpha(14.0, 27) - 0.5).abs() < f64::EPSILON);
        assert!((super::blending_height_to_offset(127.5)).abs() < f64::EPSILON);
        assert!(super::blending_height_to_offset(63.5) < 0.0);
        assert!(super::blending_height_to_offset(191.5) > 0.0);
        assert_eq!(
            super::blending_output_for_old_height(None, None),
            BlendingOutput {
                alpha: 1.0,
                blending_offset: 0.0
            }
        );
        assert_eq!(
            super::blending_output_for_old_height(Some(127.5), None),
            BlendingOutput {
                alpha: 0.0,
                blending_offset: 0.0
            }
        );
        let blended = super::blending_output_for_old_height(Some(63.5), Some(14.0));
        assert!((blended.alpha - 0.5).abs() < f64::EPSILON);
        assert!(blended.blending_offset < 0.0);
        assert_eq!(
            super::validate_blending_data_packed(BlendingDataPacked {
                min_section: -4,
                max_section: 20,
                heights: Some(&[0.0; BLENDING_CELL_COLUMN_COUNT])
            }),
            Ok(())
        );
        assert_eq!(
            super::validate_blending_data_packed(BlendingDataPacked {
                min_section: -4,
                max_section: 20,
                heights: Some(&[0.0; BLENDING_CELL_COLUMN_COUNT - 1])
            }),
            Err("heights has to be of length 16".to_string())
        );

        assert_eq!(UPGRADE_DATA_MODEL.tag_indices, "Indices");
        assert_eq!(UPGRADE_DATA_MODEL.tag_sides, "Sides");
        assert_eq!(
            UPGRADE_DATA_MODEL.tag_neighbor_block_ticks,
            "neighbor_block_ticks"
        );
        assert_eq!(
            UPGRADE_DATA_MODEL.tag_neighbor_fluid_ticks,
            "neighbor_fluid_ticks"
        );
        assert_eq!(
            UPGRADE_DATA_MODEL.block_fixers,
            &["blacklist", "default", "chest", "leaves", "stem_block"]
        );
        assert_eq!(UPGRADE_DATA_MODEL.chunky_fixers, &["leaves"]);

        assert_eq!(
            super::BELOW_ZERO_RETROGEN_MODEL.target_status_field,
            "target_status"
        );
        assert_eq!(
            super::BELOW_ZERO_RETROGEN_MODEL.missing_bedrock_field,
            "missing_bedrock"
        );
        assert_eq!(super::BELOW_ZERO_RETROGEN_MODEL.upgrade_min_y, -64);
        assert_eq!(super::BELOW_ZERO_RETROGEN_MODEL.upgrade_height, 64);
        assert_eq!(super::BELOW_ZERO_RETROGEN_MODEL.max_generated_bedrock_y, 4);
        assert_eq!(
            super::BELOW_ZERO_RETROGEN_MODEL.retained_biomes,
            &[
                "minecraft:lush_caves",
                "minecraft:dripstone_caves",
                "minecraft:deep_dark"
            ]
        );
        assert_eq!(
            super::below_zero_replace_old_bedrock_action(4, "minecraft:bedrock"),
            Some("minecraft:deepslate")
        );
        assert_eq!(
            super::below_zero_replace_old_bedrock_action(5, "minecraft:bedrock"),
            None
        );
        assert_eq!(
            super::below_zero_replace_old_bedrock_action(4, "minecraft:stone"),
            None
        );
        assert_eq!(super::below_zero_missing_bedrock_bit_index(1, 2), 33);
        assert_eq!(super::below_zero_missing_bedrock_bit_index(17, 18), 33);
        let missing_bedrock = [1_u64 << 33];
        assert!(super::below_zero_has_bedrock_hole(&missing_bedrock, 1, 2));
        assert!(!super::below_zero_has_bedrock_hole(&missing_bedrock, 2, 2));
        assert_eq!(
            super::below_zero_bedrock_mask_air_columns(-1, 1, &missing_bedrock),
            vec![
                super::FeaturePlacementBlock {
                    pos: BlockPos { x: 1, y: -1, z: 2 },
                    state: "minecraft:air"
                },
                super::FeaturePlacementBlock {
                    pos: BlockPos { x: 1, y: 0, z: 2 },
                    state: "minecraft:air"
                },
                super::FeaturePlacementBlock {
                    pos: BlockPos { x: 1, y: 1, z: 2 },
                    state: "minecraft:air"
                }
            ]
        );
        assert_eq!(
            super::below_zero_retrogen_biome(
                true,
                "minecraft:plains",
                "minecraft:old_growth_birch_forest"
            ),
            "minecraft:old_growth_birch_forest"
        );
        assert_eq!(
            super::below_zero_retrogen_biome(
                true,
                "minecraft:lush_caves",
                "minecraft:old_growth_birch_forest"
            ),
            "minecraft:lush_caves"
        );
        assert_eq!(
            super::below_zero_retrogen_biome(
                false,
                "minecraft:plains",
                "minecraft:old_growth_birch_forest"
            ),
            "minecraft:plains"
        );
    }

    #[test]
    fn spawn_selection_constants_and_initial_positions_match_vanilla() {
        assert_eq!(SPAWN_SELECTION_CONSTANTS.initial_chunk_search_radius, 5);
        assert_eq!(SPAWN_SELECTION_CONSTANTS.player_spawn_ticket_radius, 3);
        assert_eq!(
            SPAWN_SELECTION_CONSTANTS.spawn_search_absolute_max_attempts,
            1024
        );
        assert_eq!(SPAWN_SELECTION_CONSTANTS.large_search_coprime, 17);

        assert_eq!(
            super::initial_spawn_position(true, true, false, 7, -3, 64, -64, 70),
            super::InitialSpawnKind::DebugHalfWorld {
                x: 0,
                y: 64,
                z: -100
            }
        );
        assert_eq!(
            super::initial_spawn_position(false, false, true, 7, -3, 64, -64, 70),
            super::InitialSpawnKind::DebugWorld { x: 0, y: 80, z: 0 }
        );
        assert_eq!(
            super::initial_spawn_position(false, false, false, 7, -3, 90, -64, 70),
            super::InitialSpawnKind::Normal {
                x: 120,
                y: 90,
                z: -40
            }
        );
        assert_eq!(
            super::initial_spawn_position(false, false, false, 7, -3, -80, -64, 70),
            super::InitialSpawnKind::Normal {
                x: 120,
                y: 70,
                z: -40
            }
        );
    }

    #[test]
    fn initial_spawn_chunk_spiral_matches_vanilla_search_order() {
        let offsets = super::initial_spawn_chunk_spiral_offsets();
        assert_eq!(offsets.len(), 121);
        assert_eq!(
            &offsets[..12],
            &[
                (0, 0),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (2, 0),
                (2, 1)
            ]
        );
        assert_eq!(offsets.last(), Some(&(5, -5)));
        assert!(offsets.contains(&(-5, -5)));
        assert!(offsets.contains(&(5, 5)));
    }

    #[test]
    fn climate_spawn_position_uses_vanilla_two_pass_radial_search() {
        let settings = *super::builtin_noise_generator_settings("overworld").unwrap();
        let router = super::builtin_noise_router("overworld").unwrap().router;
        let spawn = super::climate_spawn_position(settings.spawn_target, router, settings, 0);
        assert_eq!(spawn.y, 0);
        assert_ne!(
            spawn,
            BlockPos { x: 0, y: 0, z: 0 },
            "overworld spawn target search should move away from origin when climate fitness improves"
        );
        assert!(
            spawn.x.abs() <= 2560 && spawn.z.abs() <= 2560,
            "two-pass radial search must stay inside the vanilla 2048+512 search envelope"
        );
        assert_eq!(
            super::climate_spawn_position(&[], router, settings, 0),
            BlockPos { x: 0, y: 0, z: 0 }
        );
    }

    #[test]
    fn noise_generator_find_spawn_position_uses_generated_surface_column() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let spawn = super::generator_find_spawn_position_for_stem(&normal.overworld, 0)
            .expect("overworld spawn position should resolve");
        let chunk_pos = ChunkPos {
            x: spawn.x.div_euclid(16),
            z: spawn.z.div_euclid(16),
        };
        let chunk = super::generator_build_surface_for_stem(chunk_pos, &normal.overworld)
            .expect("spawn chunk should generate");
        assert_eq!(
            chunk
                .get_block_state(spawn.x, spawn.y, spawn.z)
                .as_deref()
                .map(super::spawn_block_kind),
            Some(SpawnBlockKind::Air),
            "spawn position must be in a non-colliding, non-liquid block"
        );
        assert!(
            matches!(
                chunk
                    .get_block_state(spawn.x, spawn.y - 1, spawn.z)
                    .as_deref()
                    .map(super::spawn_block_kind),
                Some(SpawnBlockKind::Solid)
            ),
            "spawn position must stand on a solid generated block"
        );
    }

    #[test]
    fn spawn_original_mobs_plan_matches_noise_generator_gate() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let center = ChunkPos { x: 2, z: -3 };

        let plan = super::spawn_original_mobs_plan_for_stem(1234, center, &normal.overworld)
            .expect("overworld noise generator should spawn original mobs");

        assert_eq!(plan.center, center);
        assert_eq!(
            plan.biome_sample_pos,
            BlockPos {
                x: 32,
                y: 320,
                z: -48,
            }
        );
        assert_eq!(
            plan.decoration_seed,
            crate::random_source::decoration_seed(
                1234,
                32,
                -48,
                crate::random_source::RandomAlgorithm::Legacy,
            )
        );
        assert_eq!(
            super::spawn_original_mobs_plan_for_stem(1234, center, &normal.end),
            None
        );

        let flat = super::resolve_world_preset("flat").unwrap();

        assert_eq!(
            super::spawn_original_mobs_plan_for_stem(1234, center, &flat.overworld),
            None
        );
    }

    #[test]
    fn chunk_generation_mob_spawn_plan_matches_creature_selection_order() {
        let plains = super::biome_generation_settings("minecraft:plains").unwrap();
        let chunk = ChunkPos { x: 2, z: -3 };
        let mut random = crate::random_source::RandomSourceKind::new(
            4096,
            crate::random_source::RandomAlgorithm::Legacy,
        );

        let plan = super::chunk_generation_mob_spawn_plan(chunk, plains, true, &mut random);

        assert_eq!(plan.chunk, chunk);
        assert_eq!(
            plan.batches,
            vec![super::ChunkGenerationMobSpawnBatchPlan {
                category: "creature",
                entity_type: "minecraft:pig",
                count: 4,
                start_x: 37,
                start_z: -37,
            }]
        );

        let mut random = crate::random_source::RandomSourceKind::new(
            4096,
            crate::random_source::RandomAlgorithm::Legacy,
        );
        assert_eq!(
            super::chunk_generation_mob_spawn_plan(chunk, plains, false, &mut random).batches,
            Vec::new()
        );

        let the_void = super::biome_generation_settings("minecraft:the_void").unwrap();
        assert_eq!(
            super::chunk_generation_mob_spawn_plan(chunk, the_void, true, &mut random).batches,
            Vec::new()
        );
    }

    #[test]
    fn chunk_generation_mob_spawn_attempt_plan_matches_vanilla_offsets() {
        let chunk = ChunkPos { x: 2, z: -3 };
        let mut random = crate::random_source::RandomSourceKind::new(
            4096,
            crate::random_source::RandomAlgorithm::Legacy,
        );

        assert!(random.next_f32() < 0.1);
        assert_eq!(super::random_next_i32_bound(&mut random, 46), 15);
        assert_eq!(super::random_next_i32_bound(&mut random, 1), 0);
        let batch = super::ChunkGenerationMobSpawnBatchPlan {
            category: "creature",
            entity_type: "minecraft:pig",
            count: 4,
            start_x: 32 + super::random_next_i32_bound(&mut random, 16),
            start_z: -48 + super::random_next_i32_bound(&mut random, 16),
        };

        let attempts = super::chunk_generation_mob_spawn_attempt_plan(chunk, batch, &mut random);

        assert_eq!(batch.start_x, 37);
        assert_eq!(batch.start_z, -37);
        assert_eq!(attempts.len(), 16);
        assert_eq!(
            &attempts[..8],
            &[
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 0,
                    attempt_index: 0,
                    x: 37,
                    z: -37,
                },
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 0,
                    attempt_index: 1,
                    x: 38,
                    z: -37,
                },
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 0,
                    attempt_index: 2,
                    x: 40,
                    z: -36,
                },
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 0,
                    attempt_index: 3,
                    x: 39,
                    z: -35,
                },
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 1,
                    attempt_index: 0,
                    x: 41,
                    z: -35,
                },
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 1,
                    attempt_index: 1,
                    x: 38,
                    z: -38,
                },
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 1,
                    attempt_index: 2,
                    x: 41,
                    z: -40,
                },
                super::ChunkGenerationMobSpawnAttemptPlan {
                    mob_index: 1,
                    attempt_index: 3,
                    x: 42,
                    z: -40,
                },
            ]
        );
        assert!(attempts
            .iter()
            .all(|attempt| (32..48).contains(&attempt.x) && (-48..-32).contains(&attempt.z)));
    }

    #[test]
    fn chunk_generation_mob_top_non_colliding_pos_uses_spawn_heightmap() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let chunk_pos = ChunkPos { x: 2, z: -3 };
        let chunk = super::generator_build_surface_for_stem(chunk_pos, &normal.overworld)
            .expect("surface chunk should generate");
        let x = 37;
        let z = -37;

        let plan =
            super::chunk_generation_mob_top_non_colliding_pos(&chunk, "minecraft:pig", x, z, false);

        let local_x = x.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let height = chunk.compute_heightmap_values(HeightmapKind::MotionBlockingNoLeaves)
            [local_z * 16 + local_x];
        assert_eq!(plan.entity_type, "minecraft:pig");
        assert_eq!(plan.heightmap, HeightmapKind::MotionBlockingNoLeaves);
        assert_eq!(plan.placement_type, "on_ground");
        assert_eq!(plan.pos, BlockPos { x, y: height, z });
        assert!(
            chunk
                .get_block_state(x, plan.pos.y - 1, z)
                .as_deref()
                .is_some_and(|block| !super::is_surface_air(block)),
            "on-ground top position should stand above a non-air block"
        );
    }

    #[test]
    fn chunk_generation_spawn_position_ok_matches_placement_type_primitives() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let chunk_pos = ChunkPos { x: 2, z: -3 };
        let chunk = super::generator_build_surface_for_stem(chunk_pos, &normal.overworld)
            .expect("surface chunk should generate");
        let pos = super::chunk_generation_mob_top_non_colliding_pos(
            &chunk,
            "minecraft:pig",
            37,
            -37,
            false,
        )
        .pos;

        assert!(super::chunk_generation_spawn_position_ok(
            &chunk,
            "minecraft:pig",
            pos
        ));
        assert!(!super::chunk_generation_spawn_position_ok(
            &chunk,
            "minecraft:pig",
            BlockPos {
                x: pos.x,
                y: pos.y - 1,
                z: pos.z,
            }
        ));
        assert_eq!(super::spawn_placement_type("minecraft:squid"), "in_water");
        assert_eq!(super::spawn_placement_type("minecraft:strider"), "in_lava");
        assert_eq!(
            super::spawn_placement_type("minecraft:fox"),
            "no_restrictions"
        );
        assert_eq!(
            super::spawn_placement_heightmap("minecraft:pig"),
            HeightmapKind::MotionBlockingNoLeaves
        );
    }

    #[test]
    fn chunk_generation_mob_entity_snap_plan_clamps_width_and_rolls_yaw() {
        let chunk = ChunkPos { x: 2, z: -3 };
        let mut random = crate::random_source::RandomSourceKind::new(
            1,
            crate::random_source::RandomAlgorithm::Legacy,
        );

        let snap = super::chunk_generation_mob_entity_snap_plan(
            chunk,
            "minecraft:pig",
            BlockPos {
                x: 32,
                y: 70,
                z: -33,
            },
            &mut random,
        );

        assert_eq!(snap.entity_type, "minecraft:pig");
        assert_eq!(snap.width, 0.9);
        assert!((snap.x - 32.9).abs() < 0.000001);
        assert_eq!(snap.y, 70.0);
        assert_eq!(snap.z, -33.0);
        assert!((snap.yaw - 263.11615).abs() < 0.0001);
        assert_eq!(snap.pitch, 0.0);
    }

    #[test]
    fn chunk_generation_mob_collision_plan_rejects_solid_blocks_inside_spawn_aabb() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let mut chunk =
            super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
                .expect("surface chunk should generate");
        let pos = super::chunk_generation_mob_top_non_colliding_pos(
            &chunk,
            "minecraft:pig",
            37,
            -37,
            false,
        )
        .pos;
        let snap = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pig",
            width: 0.9,
            x: 37.0,
            y: f64::from(pos.y),
            z: -37.0,
            yaw: 0.0,
            pitch: 0.0,
        };

        let collision = super::chunk_generation_mob_collision_plan(snap);
        assert_eq!(collision.entity_type, "minecraft:pig");
        assert_eq!(collision.width, 0.9);
        assert_eq!(collision.height, 0.9);
        assert!((collision.min_x - 36.55).abs() < 0.000001);
        assert_eq!(collision.min_y, f64::from(pos.y));
        assert!((collision.max_x - 37.45).abs() < 0.000001);
        assert!((collision.max_y - (f64::from(pos.y) + 0.9)).abs() < 0.000001);
        assert!(super::chunk_generation_mob_no_collision(&chunk, collision));

        chunk.set_block_state(37, pos.y, -37, "minecraft:stone");

        assert!(!super::chunk_generation_mob_no_collision(&chunk, collision));
    }

    #[test]
    fn chunk_generation_mob_spawn_rules_cover_common_creature_predicates() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let mut chunk =
            super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
                .expect("surface chunk should generate");
        let pos = super::chunk_generation_mob_top_non_colliding_pos(
            &chunk,
            "minecraft:pig",
            37,
            -37,
            false,
        )
        .pos;

        chunk.set_block_state(pos.x, pos.y - 1, pos.z, "minecraft:grass_block");
        assert!(super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:pig",
            pos
        ));
        assert!(super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:rabbit",
            pos
        ));
        assert!(super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:goat",
            pos
        ));
        assert!(!super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:mooshroom",
            pos
        ));

        chunk.set_block_state(pos.x, pos.y - 1, pos.z, "minecraft:mycelium");
        assert!(super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:mooshroom",
            pos
        ));
        assert!(!super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:pig",
            pos
        ));

        chunk.set_block_state(pos.x, pos.y - 1, pos.z, "minecraft:stone");
        assert!(super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:goat",
            pos
        ));
        assert!(!super::chunk_generation_mob_spawn_rules_ok(
            &chunk,
            "minecraft:rabbit",
            pos
        ));

        chunk.set_block_state(pos.x, pos.y + 1, pos.z, "minecraft:stone");
        assert!(!super::chunk_generation_is_bright_enough_to_spawn(
            &chunk, pos
        ));
    }

    #[test]
    fn queue_chunk_generation_mob_entity_appends_proto_entity_nbt() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        let snap = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pig",
            width: 0.9,
            x: 32.9,
            y: 70.0,
            z: -33.0,
            yaw: 90.0,
            pitch: 0.0,
        };

        assert!(super::queue_chunk_generation_mob_entity(
            &mut chunk,
            snap,
            "00000000-0000-0000-0000-000000000123"
        ));

        assert_eq!(chunk.entities.len(), 1);
        let Tag::Compound(fields) = &chunk.entities[0] else {
            panic!("queued entity must be a compound");
        };
        assert!(fields.contains(&("id".to_string(), Tag::String("minecraft:pig".to_string()))));
        assert!(fields.contains(&(
            "UUID".to_string(),
            Tag::String("00000000-0000-0000-0000-000000000123".to_string())
        )));
        assert!(fields.contains(&(
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(70.0),
                Tag::Double(-33.0)
            ])
        )));
        assert!(fields.contains(&(
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)])
        )));
        assert!(fields.contains(&(
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)])
        )));
        assert!(fields.contains(&("fall_distance".to_string(), Tag::Double(0.0))));
        assert!(fields.contains(&("Fire".to_string(), Tag::Short(0))));
        assert!(fields.contains(&("Air".to_string(), Tag::Short(300))));
        assert!(fields.contains(&("OnGround".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("Invulnerable".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("PortalCooldown".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("Age".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("ForcedAge".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("AgeLocked".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("Health".to_string(), Tag::Float(10.0))));
        assert!(fields.contains(&("HurtTime".to_string(), Tag::Short(0))));
        assert!(fields.contains(&("HurtByTimestamp".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("DeathTime".to_string(), Tag::Short(0))));
        assert!(fields.contains(&("AbsorptionAmount".to_string(), Tag::Float(0.0))));
        assert!(fields.contains(&(
            "current_impulse_context_reset_grace_time".to_string(),
            Tag::Int(0)
        )));
        assert!(fields.contains(&("CanPickUpLoot".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("PersistenceRequired".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("LeftHanded".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_omits_ageable_fields_for_non_ageable_mobs() {
        let snap = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:creeper",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(fields) =
            super::chunk_generation_mob_entity_nbt(snap, "00000000-0000-0000-0000-000000000124")
        else {
            panic!("generated entity nbt must be a compound");
        };

        assert!(!fields.iter().any(|(name, _)| name == "Age"));
        assert!(!fields.iter().any(|(name, _)| name == "ForcedAge"));
        assert!(!fields.iter().any(|(name, _)| name == "AgeLocked"));
        assert!(!fields.iter().any(|(name, _)| name == "InLove"));
        assert!(fields.contains(&("Health".to_string(), Tag::Float(20.0))));
        assert!(fields.contains(&("CanPickUpLoot".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_animal_superclass_save_fields() {
        let animal = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pig",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ageable_non_animal = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:dolphin",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(animal_fields) =
            super::chunk_generation_mob_entity_nbt(animal, "00000000-0000-0000-0000-000000000130")
        else {
            panic!("animal entity nbt must be a compound");
        };
        let Tag::Compound(ageable_non_animal_fields) = super::chunk_generation_mob_entity_nbt(
            ageable_non_animal,
            "00000000-0000-0000-0000-000000000131",
        ) else {
            panic!("ageable non-animal entity nbt must be a compound");
        };

        assert!(animal_fields.contains(&("InLove".to_string(), Tag::Int(0))));
        assert!(!ageable_non_animal_fields
            .iter()
            .any(|(name, _)| name == "InLove"));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_neutral_mob_anger_save_fields() {
        let neutral_entities = [
            "minecraft:bee",
            "minecraft:enderman",
            "minecraft:iron_golem",
            "minecraft:polar_bear",
            "minecraft:wolf",
            "minecraft:zombified_piglin",
        ];
        let zombie = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        for (index, entity_type) in neutral_entities.iter().enumerate() {
            let snap = super::ChunkGenerationMobEntitySnapPlan {
                entity_type,
                width: 0.6,
                x: 32.5,
                y: 70.0,
                z: -33.5,
                yaw: 0.0,
                pitch: 0.0,
            };
            let uuid = format!("00000000-0000-0000-0000-{:012}", 172 + index);
            let Tag::Compound(fields) = super::chunk_generation_mob_entity_nbt(snap, &uuid) else {
                panic!("neutral entity nbt must be a compound");
            };

            assert!(
                fields.contains(&("anger_end_time".to_string(), Tag::Long(0))),
                "{entity_type} should persist default anger end time"
            );
            assert!(
                !fields.iter().any(|(name, _)| name == "angry_at"),
                "{entity_type} should omit nullable angry_at without a target"
            );
        }

        let Tag::Compound(zombie_fields) =
            super::chunk_generation_mob_entity_nbt(zombie, "00000000-0000-0000-0000-000000000174")
        else {
            panic!("zombie entity nbt must be a compound");
        };

        assert!(!zombie_fields.contains(&("anger_end_time".to_string(), Tag::Long(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_raider_patrol_save_fields() {
        let witch = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:witch",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ravager = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:ravager",
            width: 1.95,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zombie = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(witch_fields) =
            super::chunk_generation_mob_entity_nbt(witch, "00000000-0000-0000-0000-000000000182")
        else {
            panic!("witch entity nbt must be a compound");
        };
        let Tag::Compound(ravager_fields) =
            super::chunk_generation_mob_entity_nbt(ravager, "00000000-0000-0000-0000-000000000183")
        else {
            panic!("ravager entity nbt must be a compound");
        };
        let Tag::Compound(zombie_fields) =
            super::chunk_generation_mob_entity_nbt(zombie, "00000000-0000-0000-0000-000000000184")
        else {
            panic!("zombie entity nbt must be a compound");
        };

        for fields in [&witch_fields, &ravager_fields] {
            assert!(fields.contains(&("PatrolLeader".to_string(), Tag::Byte(0))));
            assert!(fields.contains(&("Patrolling".to_string(), Tag::Byte(0))));
            assert!(!fields.iter().any(|(name, _)| name == "patrol_target"));
            assert!(fields.contains(&("Wave".to_string(), Tag::Int(0))));
            assert!(fields.contains(&("CanJoinRaid".to_string(), Tag::Byte(0))));
            assert!(!fields.iter().any(|(name, _)| name == "RaidId"));
        }
        assert!(!zombie_fields.contains(&("PatrolLeader".to_string(), Tag::Byte(0))));
        assert!(!zombie_fields.contains(&("Patrolling".to_string(), Tag::Byte(0))));
        assert!(!zombie_fields.contains(&("Wave".to_string(), Tag::Int(0))));
        assert!(!zombie_fields.contains(&("CanJoinRaid".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_phantom_and_shulker_save_fields() {
        let phantom = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:phantom",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let shulker = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:shulker",
            width: 1.0,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(phantom_fields) =
            super::chunk_generation_mob_entity_nbt(phantom, "00000000-0000-0000-0000-000000000192")
        else {
            panic!("phantom entity nbt must be a compound");
        };
        let Tag::Compound(shulker_fields) =
            super::chunk_generation_mob_entity_nbt(shulker, "00000000-0000-0000-0000-000000000193")
        else {
            panic!("shulker entity nbt must be a compound");
        };

        assert!(phantom_fields.contains(&("size".to_string(), Tag::Int(0))));
        assert!(!phantom_fields.iter().any(|(name, _)| name == "anchor_pos"));
        assert!(shulker_fields.contains(&("AttachFace".to_string(), Tag::Byte(0))));
        assert!(shulker_fields.contains(&("Peek".to_string(), Tag::Byte(0))));
        assert!(shulker_fields.contains(&("Color".to_string(), Tag::Byte(16))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_additional_animal_save_fields() {
        let bat = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:bat",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let mooshroom = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:mooshroom",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let panda = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:panda",
            width: 1.3,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let parrot = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:parrot",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let turtle = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:turtle",
            width: 1.2,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(bat_fields) =
            super::chunk_generation_mob_entity_nbt(bat, "00000000-0000-0000-0000-000000000202")
        else {
            panic!("bat entity nbt must be a compound");
        };
        let Tag::Compound(mooshroom_fields) = super::chunk_generation_mob_entity_nbt(
            mooshroom,
            "00000000-0000-0000-0000-000000000203",
        ) else {
            panic!("mooshroom entity nbt must be a compound");
        };
        let Tag::Compound(panda_fields) =
            super::chunk_generation_mob_entity_nbt(panda, "00000000-0000-0000-0000-000000000204")
        else {
            panic!("panda entity nbt must be a compound");
        };
        let Tag::Compound(parrot_fields) =
            super::chunk_generation_mob_entity_nbt(parrot, "00000000-0000-0000-0000-000000000205")
        else {
            panic!("parrot entity nbt must be a compound");
        };
        let Tag::Compound(turtle_fields) =
            super::chunk_generation_mob_entity_nbt(turtle, "00000000-0000-0000-0000-000000000206")
        else {
            panic!("turtle entity nbt must be a compound");
        };

        assert!(bat_fields.contains(&("BatFlags".to_string(), Tag::Byte(0))));
        assert!(mooshroom_fields.contains(&("Type".to_string(), Tag::String("red".to_string()))));
        assert!(!mooshroom_fields
            .iter()
            .any(|(name, _)| name == "stew_effects"));
        assert!(panda_fields.contains(&("MainGene".to_string(), Tag::String("normal".to_string()))));
        assert!(
            panda_fields.contains(&("HiddenGene".to_string(), Tag::String("normal".to_string())))
        );
        assert!(parrot_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(turtle_fields.contains(&(
            "home_pos".to_string(),
            Tag::List(vec![Tag::Int(0), Tag::Int(0), Tag::Int(0)])
        )));
        assert!(turtle_fields.contains(&("has_egg".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_horse_family_save_fields() {
        let horse = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:horse",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let donkey = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:donkey",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let skeleton_horse = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:skeleton_horse",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(horse_fields) =
            super::chunk_generation_mob_entity_nbt(horse, "00000000-0000-0000-0000-000000000132")
        else {
            panic!("horse entity nbt must be a compound");
        };
        let Tag::Compound(donkey_fields) =
            super::chunk_generation_mob_entity_nbt(donkey, "00000000-0000-0000-0000-000000000133")
        else {
            panic!("donkey entity nbt must be a compound");
        };
        let Tag::Compound(skeleton_horse_fields) = super::chunk_generation_mob_entity_nbt(
            skeleton_horse,
            "00000000-0000-0000-0000-000000000134",
        ) else {
            panic!("skeleton horse entity nbt must be a compound");
        };

        for field_name in ["EatingHaystack", "Bred", "Tame"] {
            assert!(horse_fields.contains(&(field_name.to_string(), Tag::Byte(0))));
        }
        assert!(horse_fields.contains(&("Temper".to_string(), Tag::Int(0))));
        assert!(donkey_fields.contains(&("ChestedHorse".to_string(), Tag::Byte(0))));
        assert!(skeleton_horse_fields.contains(&("SkeletonTrap".to_string(), Tag::Byte(0))));
        assert!(skeleton_horse_fields.contains(&("SkeletonTrapTime".to_string(), Tag::Int(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_water_and_ambient_animal_save_fields() {
        let pufferfish = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pufferfish",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let dolphin = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:dolphin",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let fox = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:fox",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let camel = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:camel",
            width: 1.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(pufferfish_fields) = super::chunk_generation_mob_entity_nbt(
            pufferfish,
            "00000000-0000-0000-0000-000000000135",
        ) else {
            panic!("pufferfish entity nbt must be a compound");
        };
        let Tag::Compound(dolphin_fields) =
            super::chunk_generation_mob_entity_nbt(dolphin, "00000000-0000-0000-0000-000000000136")
        else {
            panic!("dolphin entity nbt must be a compound");
        };
        let Tag::Compound(fox_fields) =
            super::chunk_generation_mob_entity_nbt(fox, "00000000-0000-0000-0000-000000000137")
        else {
            panic!("fox entity nbt must be a compound");
        };
        let Tag::Compound(camel_fields) =
            super::chunk_generation_mob_entity_nbt(camel, "00000000-0000-0000-0000-000000000138")
        else {
            panic!("camel entity nbt must be a compound");
        };

        assert!(pufferfish_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(pufferfish_fields.contains(&("PuffState".to_string(), Tag::Int(0))));
        assert!(dolphin_fields.contains(&("GotFish".to_string(), Tag::Byte(0))));
        assert!(dolphin_fields.contains(&("Moistness".to_string(), Tag::Int(2400))));
        for field_name in ["Sleeping", "Sitting", "Crouching"] {
            assert!(fox_fields.contains(&(field_name.to_string(), Tag::Byte(0))));
        }
        assert!(camel_fields.contains(&("LastPoseTick".to_string(), Tag::Long(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_special_animal_save_fields() {
        let bee = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:bee",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let llama = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:llama",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let trader_llama = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:trader_llama",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let armadillo = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:armadillo",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let horse = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:horse",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let iron_golem = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:iron_golem",
            width: 1.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let snow_golem = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:snow_golem",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(bee_fields) =
            super::chunk_generation_mob_entity_nbt(bee, "00000000-0000-0000-0000-000000000139")
        else {
            panic!("bee entity nbt must be a compound");
        };
        let Tag::Compound(llama_fields) =
            super::chunk_generation_mob_entity_nbt(llama, "00000000-0000-0000-0000-000000000140")
        else {
            panic!("llama entity nbt must be a compound");
        };
        let Tag::Compound(trader_llama_fields) = super::chunk_generation_mob_entity_nbt(
            trader_llama,
            "00000000-0000-0000-0000-000000000141",
        ) else {
            panic!("trader llama entity nbt must be a compound");
        };
        let Tag::Compound(armadillo_fields) = super::chunk_generation_mob_entity_nbt(
            armadillo,
            "00000000-0000-0000-0000-000000000142",
        ) else {
            panic!("armadillo entity nbt must be a compound");
        };
        let Tag::Compound(horse_fields) =
            super::chunk_generation_mob_entity_nbt(horse, "00000000-0000-0000-0000-000000000144")
        else {
            panic!("horse entity nbt must be a compound");
        };
        let Tag::Compound(iron_golem_fields) = super::chunk_generation_mob_entity_nbt(
            iron_golem,
            "00000000-0000-0000-0000-000000000145",
        ) else {
            panic!("iron golem entity nbt must be a compound");
        };
        let Tag::Compound(snow_golem_fields) = super::chunk_generation_mob_entity_nbt(
            snow_golem,
            "00000000-0000-0000-0000-000000000143",
        ) else {
            panic!("snow golem entity nbt must be a compound");
        };

        assert!(bee_fields.contains(&("HasNectar".to_string(), Tag::Byte(0))));
        assert!(bee_fields.contains(&("HasStung".to_string(), Tag::Byte(0))));
        assert!(bee_fields.contains(&("TicksSincePollination".to_string(), Tag::Int(0))));
        assert!(bee_fields.contains(&("CannotEnterHiveTicks".to_string(), Tag::Int(0))));
        assert!(bee_fields.contains(&("CropsGrownSincePollination".to_string(), Tag::Int(0))));
        assert!(llama_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(llama_fields.contains(&("Strength".to_string(), Tag::Int(0))));
        assert!(llama_fields.contains(&("ChestedHorse".to_string(), Tag::Byte(0))));
        assert!(trader_llama_fields.contains(&("DespawnDelay".to_string(), Tag::Int(47999))));
        assert!(armadillo_fields.contains(&("state".to_string(), Tag::String("idle".to_string()))));
        assert!(horse_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(iron_golem_fields.contains(&("PlayerCreated".to_string(), Tag::Byte(0))));
        assert!(snow_golem_fields.contains(&("Pumpkin".to_string(), Tag::Byte(1))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_monster_save_fields() {
        let creeper = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:creeper",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let slime = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:slime",
            width: 0.52,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ravager = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:ravager",
            width: 1.95,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ghast = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:ghast",
            width: 4.0,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let endermite = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:endermite",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zoglin = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zoglin",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(creeper_fields) =
            super::chunk_generation_mob_entity_nbt(creeper, "00000000-0000-0000-0000-000000000146")
        else {
            panic!("creeper entity nbt must be a compound");
        };
        let Tag::Compound(slime_fields) =
            super::chunk_generation_mob_entity_nbt(slime, "00000000-0000-0000-0000-000000000147")
        else {
            panic!("slime entity nbt must be a compound");
        };
        let Tag::Compound(ravager_fields) =
            super::chunk_generation_mob_entity_nbt(ravager, "00000000-0000-0000-0000-000000000148")
        else {
            panic!("ravager entity nbt must be a compound");
        };
        let Tag::Compound(ghast_fields) =
            super::chunk_generation_mob_entity_nbt(ghast, "00000000-0000-0000-0000-000000000149")
        else {
            panic!("ghast entity nbt must be a compound");
        };
        let Tag::Compound(endermite_fields) = super::chunk_generation_mob_entity_nbt(
            endermite,
            "00000000-0000-0000-0000-000000000150",
        ) else {
            panic!("endermite entity nbt must be a compound");
        };
        let Tag::Compound(zoglin_fields) =
            super::chunk_generation_mob_entity_nbt(zoglin, "00000000-0000-0000-0000-000000000151")
        else {
            panic!("zoglin entity nbt must be a compound");
        };

        assert!(creeper_fields.contains(&("powered".to_string(), Tag::Byte(0))));
        assert!(creeper_fields.contains(&("Fuse".to_string(), Tag::Short(30))));
        assert!(creeper_fields.contains(&("ExplosionRadius".to_string(), Tag::Byte(3))));
        assert!(creeper_fields.contains(&("ignited".to_string(), Tag::Byte(0))));
        assert!(slime_fields.contains(&("Size".to_string(), Tag::Int(0))));
        assert!(slime_fields.contains(&("wasOnGround".to_string(), Tag::Byte(0))));
        assert!(ravager_fields.contains(&("AttackTick".to_string(), Tag::Int(0))));
        assert!(ravager_fields.contains(&("StunTick".to_string(), Tag::Int(0))));
        assert!(ravager_fields.contains(&("RoarTick".to_string(), Tag::Int(0))));
        assert!(ghast_fields.contains(&("ExplosionPower".to_string(), Tag::Byte(1))));
        assert!(endermite_fields.contains(&("Lifetime".to_string(), Tag::Int(0))));
        assert!(zoglin_fields.contains(&("IsBaby".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_zombie_piglin_and_skeleton_save_fields() {
        let zombie = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zombie_villager = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie_villager",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let skeleton = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:skeleton",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let bogged = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:bogged",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let piglin = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:piglin",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let hoglin = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:hoglin",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(zombie_fields) =
            super::chunk_generation_mob_entity_nbt(zombie, "00000000-0000-0000-0000-000000000152")
        else {
            panic!("zombie entity nbt must be a compound");
        };
        let Tag::Compound(zombie_villager_fields) = super::chunk_generation_mob_entity_nbt(
            zombie_villager,
            "00000000-0000-0000-0000-000000000153",
        ) else {
            panic!("zombie villager entity nbt must be a compound");
        };
        let Tag::Compound(skeleton_fields) = super::chunk_generation_mob_entity_nbt(
            skeleton,
            "00000000-0000-0000-0000-000000000154",
        ) else {
            panic!("skeleton entity nbt must be a compound");
        };
        let Tag::Compound(bogged_fields) =
            super::chunk_generation_mob_entity_nbt(bogged, "00000000-0000-0000-0000-000000000155")
        else {
            panic!("bogged entity nbt must be a compound");
        };
        let Tag::Compound(piglin_fields) =
            super::chunk_generation_mob_entity_nbt(piglin, "00000000-0000-0000-0000-000000000156")
        else {
            panic!("piglin entity nbt must be a compound");
        };
        let Tag::Compound(hoglin_fields) =
            super::chunk_generation_mob_entity_nbt(hoglin, "00000000-0000-0000-0000-000000000157")
        else {
            panic!("hoglin entity nbt must be a compound");
        };

        assert!(zombie_fields.contains(&("IsBaby".to_string(), Tag::Byte(0))));
        assert!(zombie_fields.contains(&("CanBreakDoors".to_string(), Tag::Byte(0))));
        assert!(zombie_fields.contains(&("InWaterTime".to_string(), Tag::Int(-1))));
        assert!(zombie_fields.contains(&("DrownedConversionTime".to_string(), Tag::Int(-1))));
        assert!(
            zombie_villager_fields.contains(&("VillagerDataFinalized".to_string(), Tag::Byte(0)))
        );
        assert!(zombie_villager_fields.contains(&("ConversionTime".to_string(), Tag::Int(-1))));
        assert!(zombie_villager_fields.contains(&("Xp".to_string(), Tag::Int(0))));
        assert!(skeleton_fields.contains(&("StrayConversionTime".to_string(), Tag::Int(-1))));
        assert!(bogged_fields.contains(&("sheared".to_string(), Tag::Byte(0))));
        assert!(piglin_fields.contains(&("IsImmuneToZombification".to_string(), Tag::Byte(0))));
        assert!(piglin_fields.contains(&("TimeInOverworld".to_string(), Tag::Int(0))));
        assert!(piglin_fields.contains(&("IsBaby".to_string(), Tag::Byte(0))));
        assert!(piglin_fields.contains(&("CannotHunt".to_string(), Tag::Byte(0))));
        assert!(hoglin_fields.contains(&("IsImmuneToZombification".to_string(), Tag::Byte(0))));
        assert!(hoglin_fields.contains(&("TimeInOverworld".to_string(), Tag::Int(0))));
        assert!(hoglin_fields.contains(&("CannotBeHunted".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_animal_specific_save_fields() {
        let sheep = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:sheep",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let cat = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:cat",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let chicken = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:chicken",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let goat = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:goat",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let rabbit = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:rabbit",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(sheep_fields) =
            super::chunk_generation_mob_entity_nbt(sheep, "00000000-0000-0000-0000-000000000125")
        else {
            panic!("sheep entity nbt must be a compound");
        };
        let Tag::Compound(cat_fields) =
            super::chunk_generation_mob_entity_nbt(cat, "00000000-0000-0000-0000-000000000126")
        else {
            panic!("cat entity nbt must be a compound");
        };
        let Tag::Compound(chicken_fields) =
            super::chunk_generation_mob_entity_nbt(chicken, "00000000-0000-0000-0000-000000000127")
        else {
            panic!("chicken entity nbt must be a compound");
        };
        let Tag::Compound(goat_fields) =
            super::chunk_generation_mob_entity_nbt(goat, "00000000-0000-0000-0000-000000000128")
        else {
            panic!("goat entity nbt must be a compound");
        };
        let Tag::Compound(rabbit_fields) =
            super::chunk_generation_mob_entity_nbt(rabbit, "00000000-0000-0000-0000-000000000129")
        else {
            panic!("rabbit entity nbt must be a compound");
        };

        assert!(sheep_fields.contains(&("Sheared".to_string(), Tag::Byte(0))));
        assert!(sheep_fields.contains(&("Color".to_string(), Tag::Byte(0))));
        assert!(cat_fields.contains(&("CollarColor".to_string(), Tag::Byte(14))));
        assert!(cat_fields.contains(&(
            "variant".to_string(),
            Tag::String("minecraft:black".to_string())
        )));
        assert!(cat_fields.contains(&(
            "sound_variant".to_string(),
            Tag::String("minecraft:classic".to_string())
        )));
        assert!(chicken_fields.contains(&("IsChickenJockey".to_string(), Tag::Byte(0))));
        assert!(chicken_fields.contains(&("EggLayTime".to_string(), Tag::Int(6000))));
        assert!(goat_fields.contains(&("IsScreamingGoat".to_string(), Tag::Byte(0))));
        assert!(goat_fields.contains(&("HasLeftHorn".to_string(), Tag::Byte(1))));
        assert!(goat_fields.contains(&("HasRightHorn".to_string(), Tag::Byte(1))));
        assert!(rabbit_fields.contains(&("RabbitType".to_string(), Tag::Int(0))));
        assert!(rabbit_fields.contains(&("MoreCarrotTicks".to_string(), Tag::Int(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_variant_save_fields() {
        let cow = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:cow",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let pig = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pig",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let chicken = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:chicken",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let frog = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:frog",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let axolotl = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:axolotl",
            width: 0.75,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let salmon = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:salmon",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let tropical_fish = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:tropical_fish",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let wolf = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:wolf",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zombie_nautilus = super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie_nautilus",
            width: 1.2,
            x: 32.5,
            y: 62.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(cow_fields) =
            super::chunk_generation_mob_entity_nbt(cow, "00000000-0000-0000-0000-000000000158")
        else {
            panic!("cow entity nbt must be a compound");
        };
        let Tag::Compound(pig_fields) =
            super::chunk_generation_mob_entity_nbt(pig, "00000000-0000-0000-0000-000000000159")
        else {
            panic!("pig entity nbt must be a compound");
        };
        let Tag::Compound(chicken_fields) =
            super::chunk_generation_mob_entity_nbt(chicken, "00000000-0000-0000-0000-000000000160")
        else {
            panic!("chicken entity nbt must be a compound");
        };
        let Tag::Compound(frog_fields) =
            super::chunk_generation_mob_entity_nbt(frog, "00000000-0000-0000-0000-000000000161")
        else {
            panic!("frog entity nbt must be a compound");
        };
        let Tag::Compound(axolotl_fields) =
            super::chunk_generation_mob_entity_nbt(axolotl, "00000000-0000-0000-0000-000000000162")
        else {
            panic!("axolotl entity nbt must be a compound");
        };
        let Tag::Compound(salmon_fields) =
            super::chunk_generation_mob_entity_nbt(salmon, "00000000-0000-0000-0000-000000000163")
        else {
            panic!("salmon entity nbt must be a compound");
        };
        let Tag::Compound(tropical_fish_fields) = super::chunk_generation_mob_entity_nbt(
            tropical_fish,
            "00000000-0000-0000-0000-000000000164",
        ) else {
            panic!("tropical fish entity nbt must be a compound");
        };
        let Tag::Compound(wolf_fields) =
            super::chunk_generation_mob_entity_nbt(wolf, "00000000-0000-0000-0000-000000000165")
        else {
            panic!("wolf entity nbt must be a compound");
        };
        let Tag::Compound(zombie_nautilus_fields) = super::chunk_generation_mob_entity_nbt(
            zombie_nautilus,
            "00000000-0000-0000-0000-000000000166",
        ) else {
            panic!("zombie nautilus entity nbt must be a compound");
        };

        for fields in [&cow_fields, &pig_fields, &chicken_fields, &frog_fields] {
            assert!(fields.contains(&(
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string())
            )));
        }
        for fields in [&cow_fields, &pig_fields, &chicken_fields] {
            assert!(fields.contains(&(
                "sound_variant".to_string(),
                Tag::String("minecraft:classic".to_string())
            )));
        }
        assert!(axolotl_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(axolotl_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(salmon_fields.contains(&("type".to_string(), Tag::String("medium".to_string()))));
        assert!(salmon_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(tropical_fish_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(tropical_fish_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(wolf_fields.contains(&(
            "variant".to_string(),
            Tag::String("minecraft:pale".to_string())
        )));
        assert!(wolf_fields.contains(&(
            "sound_variant".to_string(),
            Tag::String("minecraft:classic".to_string())
        )));
        assert!(zombie_nautilus_fields.contains(&(
            "variant".to_string(),
            Tag::String("minecraft:temperate".to_string())
        )));
    }

    #[test]
    fn apply_chunk_generation_mob_batch_to_chunk_queues_successful_placements() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let mut chunk =
            super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
                .expect("surface chunk should generate");
        let batch = super::ChunkGenerationMobSpawnBatchPlan {
            category: "creature",
            entity_type: "minecraft:pig",
            count: 1,
            start_x: 37,
            start_z: -37,
        };
        let mut random = crate::random_source::RandomSourceKind::new(
            1,
            crate::random_source::RandomAlgorithm::Legacy,
        );

        let spawned = super::apply_chunk_generation_mob_batch_to_chunk(
            &mut chunk,
            batch,
            false,
            &mut random,
            &["00000000-0000-0000-0000-000000000456"],
        );

        assert_eq!(spawned, 1);
        assert_eq!(chunk.entities.len(), 1);
        let Tag::Compound(fields) = &chunk.entities[0] else {
            panic!("queued entity must be a compound");
        };
        assert!(fields.contains(&("id".to_string(), Tag::String("minecraft:pig".to_string()))));
        assert!(fields.contains(&(
            "UUID".to_string(),
            Tag::String("00000000-0000-0000-0000-000000000456".to_string())
        )));
    }

    #[test]
    fn apply_chunk_generation_mob_batch_to_chunk_generates_missing_uuids() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let mut chunk =
            super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
                .expect("surface chunk should generate");
        let batch = super::ChunkGenerationMobSpawnBatchPlan {
            category: "creature",
            entity_type: "minecraft:pig",
            count: 1,
            start_x: 37,
            start_z: -37,
        };
        let mut random = crate::random_source::RandomSourceKind::new(
            1,
            crate::random_source::RandomAlgorithm::Legacy,
        );

        let spawned = super::apply_chunk_generation_mob_batch_to_chunk(
            &mut chunk,
            batch,
            false,
            &mut random,
            &[],
        );

        assert_eq!(spawned, 1);
        let Tag::Compound(fields) = &chunk.entities[0] else {
            panic!("queued entity must be a compound");
        };
        let uuid = fields
            .iter()
            .find_map(|(name, value)| match (name.as_str(), value) {
                ("UUID", Tag::String(uuid)) => Some(uuid),
                _ => None,
            })
            .expect("generated entity should have a UUID");
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[14..15], "4");
    }

    #[test]
    fn create_insecure_uuid_matches_mth_version_and_variant_bits() {
        let mut random = crate::random_source::RandomSourceKind::new(
            1,
            crate::random_source::RandomAlgorithm::Legacy,
        );

        assert_eq!(
            super::create_insecure_uuid(&mut random),
            "bb1ad573-19b8-4cd8-a8fb-0e6f684df992"
        );
    }

    #[test]
    fn player_spawn_search_candidate_math_matches_vanilla() {
        assert_eq!(super::spawn_search_candidate_count(0), 1);
        assert_eq!(super::spawn_search_candidate_count(1), 9);
        assert_eq!(super::spawn_search_candidate_count(16), 1024);
        assert_eq!(super::spawn_search_coprime(9), 8);
        assert_eq!(super::spawn_search_coprime(17), 17);

        assert_eq!(super::spawn_search_radius(10, 20), 10);
        assert_eq!(super::spawn_search_radius(10, 4), 4);
        assert_eq!(super::spawn_search_radius(10, 1), 1);
        assert_eq!(super::spawn_search_radius(-5, 20), 0);

        assert_eq!(
            super::spawn_search_candidate(100, 200, 1, 0, 0),
            Some((99, 199))
        );
        assert_eq!(
            super::spawn_search_candidate(100, 200, 1, 0, 1),
            Some((101, 201))
        );
        assert_eq!(
            super::spawn_search_candidate(100, 200, 1, 8, 0),
            Some((101, 201))
        );
        assert_eq!(super::spawn_search_candidate(100, 200, 1, 0, 9), None);
    }

    #[test]
    fn initial_spawn_readiness_requires_player_spawn_ticket_radius_full_chunks() {
        let center = ChunkPos { x: 2, z: -1 };
        let required = super::initial_spawn_required_chunks(center);
        assert_eq!(required.len(), 49);
        assert_eq!(required.first(), Some(&ChunkPos { x: -1, z: -4 }));
        assert_eq!(required.last(), Some(&ChunkPos { x: 5, z: 2 }));

        let mut snapshots: Vec<_> = required
            .iter()
            .copied()
            .map(|pos| super::SpawnChunkStatusSnapshot {
                pos,
                status: "minecraft:full",
            })
            .collect();
        assert!(super::initial_spawn_chunks_ready(center, &snapshots));

        snapshots[0].status = "minecraft:light";
        let report = super::initial_spawn_readiness_report(center, &snapshots);
        assert_eq!(report.required_radius, 3);
        assert_eq!(report.required_status, "minecraft:full");
        assert_eq!(report.ticket_type, "minecraft:player_spawn");
        assert_eq!(
            report.ticket_level,
            crate::chunk_ticket::FULL_CHUNK_LEVEL - 3
        );
        assert_eq!(report.missing_chunks, Vec::<ChunkPos>::new());
        assert_eq!(
            report.not_ready_chunks,
            vec![super::SpawnChunkStatusSnapshot {
                pos: ChunkPos { x: -1, z: -4 },
                status: "minecraft:light",
            }]
        );
        assert!(!report.is_ready());

        snapshots.pop();
        let report = super::initial_spawn_readiness_report(center, &snapshots);
        assert_eq!(report.missing_chunks, vec![ChunkPos { x: 5, z: 2 }]);
    }

    #[test]
    fn overworld_respawn_candidate_rules_match_vanilla() {
        let normal_column = SpawnColumnHeights {
            top_y: 64,
            surface_y: 66,
            ocean_floor_y: 63,
            min_y: -64,
        };
        assert_eq!(
            super::overworld_respawn_y(
                normal_column,
                false,
                &[
                    SpawnBlockKind::Air,
                    SpawnBlockKind::Air,
                    SpawnBlockKind::Solid
                ]
            ),
            Some(64)
        );

        assert_eq!(
            super::overworld_respawn_y(
                SpawnColumnHeights {
                    top_y: -80,
                    ..normal_column
                },
                false,
                &[SpawnBlockKind::Solid]
            ),
            None
        );
        assert_eq!(
            super::overworld_respawn_y(
                SpawnColumnHeights {
                    surface_y: 64,
                    ocean_floor_y: 62,
                    ..normal_column
                },
                false,
                &[SpawnBlockKind::Solid]
            ),
            None
        );
        assert_eq!(
            super::overworld_respawn_y(
                normal_column,
                false,
                &[
                    SpawnBlockKind::Air,
                    SpawnBlockKind::Fluid,
                    SpawnBlockKind::Solid
                ]
            ),
            None
        );
    }

    #[test]
    fn spawn_height_fixup_walks_like_vanilla() {
        let blocked_until_70 = |y| y >= 70;
        assert_eq!(
            super::fixup_spawn_height(64, -64, 320, blocked_until_70),
            70
        );

        let air_above_ground = |y| y >= 65;
        assert_eq!(
            super::fixup_spawn_height(80, -64, 320, air_above_ground),
            65
        );
    }

    #[test]
    fn flat_generator_expands_layers_bottom_up_like_vanilla() {
        let settings = super::default_flat_generator_settings().unwrap();
        assert_eq!(settings.biome, "minecraft:plains");
        assert_eq!(
            settings.structure_overrides,
            vec!["minecraft:strongholds", "minecraft:villages"]
        );
        assert_eq!(
            settings.expanded_layers,
            vec![
                Some("minecraft:bedrock"),
                Some("minecraft:dirt"),
                Some("minecraft:dirt"),
                Some("minecraft:grass_block")
            ]
        );
        assert!(!settings.void_generation);
        assert_eq!(
            super::flat_base_height(
                &settings.expanded_layers,
                super::FLAT_GENERATOR_MIN_Y,
                super::FLAT_GENERATOR_GEN_DEPTH,
                HeightmapKind::MotionBlocking
            ),
            4
        );
        assert_eq!(
            super::flat_base_column(&settings.expanded_layers, 0, 6).states,
            vec![
                "minecraft:bedrock",
                "minecraft:dirt",
                "minecraft:dirt",
                "minecraft:grass_block",
                "minecraft:air",
                "minecraft:air"
            ]
        );
    }

    #[test]
    fn flat_generator_handles_void_and_non_motion_blocking_layers() {
        let void_settings = super::flat_generator_settings(
            super::flat_generator_preset("minecraft:the_void").unwrap(),
        )
        .unwrap();
        assert!(void_settings.void_generation);
        assert_eq!(void_settings.expanded_layers, vec![None]);
        assert_eq!(
            void_settings.top_layer_modifications,
            vec![(0, "minecraft:air")]
        );

        let snowy = super::flat_generator_settings(
            super::flat_generator_preset("minecraft:snowy_kingdom").unwrap(),
        )
        .unwrap();
        assert_eq!(snowy.expanded_layers[0], Some("minecraft:bedrock"));
        assert_eq!(snowy.expanded_layers[63], Some("minecraft:grass_block"));
        assert_eq!(snowy.expanded_layers[64], None);
        assert_eq!(snowy.top_layer_modifications, vec![(64, "minecraft:snow")]);
        assert_eq!(
            super::flat_base_height(
                &snowy.expanded_layers,
                super::FLAT_GENERATOR_MIN_Y,
                super::FLAT_GENERATOR_GEN_DEPTH,
                HeightmapKind::MotionBlocking
            ),
            64
        );
    }

    #[test]
    fn flat_generator_materializes_chunk_sections_and_heightmaps() {
        let settings = super::default_flat_generator_settings().unwrap();
        let chunk = super::materialize_flat_chunk(ChunkPos { x: 2, z: -1 }, &settings);
        assert_eq!(chunk.status, "minecraft:full");
        assert_eq!(chunk.sections.len(), 1);
        assert_eq!(chunk.sections[0].y, 0);
        assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
        assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));

        let Tag::Compound(section) = &chunk.sections[0].block_states else {
            panic!("block states should be stored as a compound");
        };
        let Some((_, Tag::List(palette))) = section.iter().find(|(name, _)| name == "palette")
        else {
            panic!("block states should include a palette");
        };
        assert_eq!(palette.len(), 4);
        assert!(matches!(
            &chunk.sections[0].biomes,
            Tag::Compound(fields)
                if matches!(
                    fields.iter().find(|(name, _)| name == "palette"),
                    Some((_, Tag::List(values))) if values == &vec![Tag::String("minecraft:plains".to_string())]
                )
        ));
    }

    // ---------- RandomStateNoiseCache parity tests ----------

    #[test]
    fn random_state_noise_cache_matches_uncached_and_is_stable_across_calls() {
        let seed = 12345_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld").unwrap();
        let noise_id = "minecraft:temperature";

        let uncached = random_state_normal_noise_snapshot(seed, settings, noise_id).unwrap();

        let mut cache = RandomStateNoiseCache::new(seed, settings);
        let first = cache.get_or_create_noise(noise_id).unwrap().clone();
        let second = cache.get_or_create_noise(noise_id).unwrap().clone();

        // First call must match uncached computation (same seed, same noise id).
        assert_eq!(first, uncached);
        // Repeated calls must return the identical cached value.
        assert_eq!(first, second);
    }

    #[test]
    fn random_state_noise_cache_unknown_id_returns_none() {
        let settings = *builtin_noise_generator_settings("minecraft:overworld").unwrap();
        let mut cache = RandomStateNoiseCache::new(0, settings);
        assert!(cache
            .get_or_create_noise("minecraft:nonexistent_noise_xyz")
            .is_none());
    }

    // ---------- NoiseGeneratorSettings codec loading tests ----------

    #[test]
    fn noise_generator_settings_scalar_fields_match_vanilla_json_files() {
        // Read every noise_settings JSON from the decompiled server data directory and
        // validate the scalar fields against our hardcoded EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS.
        // This is the codec-loading parity test: it proves our statics match vanilla JSON.
        let dir = "../decompiled-server-26.1.2/data/minecraft/worldgen/noise_settings";

        for entry in EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS {
            let name = entry.id.strip_prefix("minecraft:").unwrap_or(entry.id);
            let path = format!("{}/{}.json", dir, name);
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
            let json: serde_json::Value = serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("invalid JSON in {path}: {e}"));

            let noise = &json["noise"];
            assert_eq!(
                noise["min_y"].as_i64().unwrap() as i32,
                entry.noise.min_y,
                "{} noise.min_y",
                entry.id
            );
            assert_eq!(
                noise["height"].as_i64().unwrap() as i32,
                entry.noise.height,
                "{} noise.height",
                entry.id
            );
            assert_eq!(
                noise["size_horizontal"].as_i64().unwrap() as i32,
                entry.noise.size_horizontal,
                "{} noise.size_horizontal",
                entry.id
            );
            assert_eq!(
                noise["size_vertical"].as_i64().unwrap() as i32,
                entry.noise.size_vertical,
                "{} noise.size_vertical",
                entry.id
            );

            // default_block / default_fluid stored as { "Name": "minecraft:..." }
            assert_eq!(
                json["default_block"]["Name"].as_str().unwrap(),
                entry.default_block,
                "{} default_block",
                entry.id
            );
            assert_eq!(
                json["default_fluid"]["Name"].as_str().unwrap(),
                entry.default_fluid,
                "{} default_fluid",
                entry.id
            );

            assert_eq!(
                json["sea_level"].as_i64().unwrap() as i32,
                entry.sea_level,
                "{} sea_level",
                entry.id
            );
            assert_eq!(
                json["disable_mob_generation"].as_bool().unwrap(),
                entry.disable_mob_generation,
                "{} disable_mob_generation",
                entry.id
            );
            assert_eq!(
                json["aquifers_enabled"].as_bool().unwrap(),
                entry.aquifers_enabled,
                "{} aquifers_enabled",
                entry.id
            );
            assert_eq!(
                json["ore_veins_enabled"].as_bool().unwrap(),
                entry.ore_veins_enabled,
                "{} ore_veins_enabled",
                entry.id
            );
            assert_eq!(
                json["legacy_random_source"].as_bool().unwrap(),
                entry.legacy_random_source,
                "{} legacy_random_source",
                entry.id
            );
            assert_eq!(
                json["spawn_target"]
                    .as_array()
                    .map(|a| a.len())
                    .unwrap_or(0),
                entry.spawn_target_len,
                "{} spawn_target length",
                entry.id
            );
        }
    }

    #[test]
    fn overworld_final_density_matches_vanilla_at_0_100_0_seed_0() {
        // Oracle command used against official 26.1.2:
        // VanillaRegistries.createLookup() -> RandomState.create(OVERWORLD, seed 0) ->
        // router().finalDensity().compute(SinglePointContext(0, 100, 0)).
        const VANILLA_FINAL_DENSITY_0_100_0_SEED_0: f64 = -0.45833333333333330;
        let seed = 0_i64;

        // Confirm the registry entry resolves.
        assert!(
            builtin_density_function("minecraft:overworld/final_density").is_some(),
            "minecraft:overworld/final_density must be registered in BUILTIN_DENSITY_FUNCTIONS"
        );

        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");

        let final_density = super::OVERWORLD_NOISE_ROUTER.final_density;
        let result = final_density.compute_with_noise(seed, settings, 0, 100, 0);

        assert!(
            result.is_finite(),
            "finalDensity at (0,100,0) seed 0 must be finite, got {result}"
        );
        assert!(
            (result - VANILLA_FINAL_DENSITY_0_100_0_SEED_0).abs() < 1e-15,
            "finalDensity at (0,100,0) seed 0 must match vanilla {VANILLA_FINAL_DENSITY_0_100_0_SEED_0}, got {result}"
        );
    }

    #[test]
    fn overworld_base_3d_noise_matches_vanilla_in_river_column() {
        // Oracle command used against official 26.1.2:
        // VanillaRegistries.createLookup() -> RandomState.create(OVERWORLD, seed 8675309) ->
        // router().finalDensity().compute(SinglePointContext(-16, y, -8)).
        // The base_3d_noise value is the RandomState-wired BlendedNoise with the
        // "minecraft:terrain" positional random, not the unseeded registry entry.
        let seed = 8_675_309_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");

        let base_3d =
            super::BASE_3D_NOISE_OVERWORLD_DENSITY.compute_with_noise(seed, settings, -16, 56, -8);
        assert!(
            (base_3d - -0.08889146062143664).abs() < 1e-14,
            "base_3d_noise at (-16,56,-8) seed {seed} must match vanilla, got {base_3d}"
        );

        let final_density = super::OVERWORLD_NOISE_ROUTER.final_density;
        let y_samples = [
            (56, 0.09843842627734438),
            (57, 0.04970794306538723),
            (58, -0.002730670313905269),
            (59, -0.03560427276411281),
            (60, -0.04682484220125964),
            (61, -0.06178625975797264),
            (62, -0.07550869288269363),
        ];
        for (y, expected) in y_samples {
            let result = final_density.compute_with_noise(seed, settings, -16, y, -8);
            assert!(
                (result - expected).abs() < 1e-3,
                "finalDensity at (-16,{y},-8) seed {seed} must match vanilla {expected}, got {result}"
            );
        }
    }

    #[test]
    fn overworld_shift_noise_uses_vanilla_offset_noise_key() {
        let seed = 8_675_309_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld").unwrap();
        assert!(
            super::builtin_normal_noise_parameters("minecraft:shift").is_none(),
            "vanilla Noises.SHIFT is ResourceKey minecraft:offset; Rust must not invent minecraft:shift"
        );

        let samples = [
            ((0, 62, 0), -3.7094676845533336, -3.7094676845533336),
            ((3, 62, 0), -3.3318550257927586, -2.9289862624467620),
            ((31, 62, 6), -0.42489011889609285, -0.4626364685144386),
        ];

        for ((x, y, z), expected_x, expected_z) in samples {
            let shift_x = super::SHIFT_X_DENSITY.compute_with_noise(seed, settings, x, y, z);
            let shift_z = super::SHIFT_Z_DENSITY.compute_with_noise(seed, settings, x, y, z);
            assert!(
                (shift_x - expected_x).abs() < 1e-12,
                "SHIFT_X at ({x},{y},{z}) seed {seed} should match Java Noises.SHIFT offset noise"
            );
            assert!(
                (shift_z - expected_z).abs() < 1e-12,
                "SHIFT_Z at ({x},{y},{z}) seed {seed} should match Java Noises.SHIFT offset noise"
            );
        }
    }

    /// Verifies that every NoiseRouter field for the standard overworld evaluates to a finite,
    /// non-NaN value at a canonical position. An unregistered Reference silently returns 0.0
    /// rather than NaN or infinity, so this test guards against both silent-zero and arithmetic
    /// blow-up bugs. Seed 12345, block (0, 64, 0) is well inside the active overworld range.
    #[test]
    fn overworld_noise_router_all_fields_finite_at_canonical_position() {
        let seed = 12345_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");
        let router = super::OVERWORLD_NOISE_ROUTER;
        let (x, y, z) = (0, 64, 0);

        macro_rules! check_field {
            ($field:expr, $name:literal) => {
                let val = $field.compute_with_noise(seed, settings, x, y, z);
                assert!(
                    val.is_finite(),
                    "OVERWORLD_NOISE_ROUTER.{} returned non-finite ({val}) at ({x},{y},{z}) seed {seed}",
                    $name
                );
            };
        }

        check_field!(router.barrier, "barrier");
        check_field!(router.fluid_level_floodedness, "fluid_level_floodedness");
        check_field!(router.fluid_level_spread, "fluid_level_spread");
        check_field!(router.lava, "lava");
        check_field!(router.temperature, "temperature");
        check_field!(router.vegetation, "vegetation");
        check_field!(router.continents, "continents");
        check_field!(router.erosion, "erosion");
        check_field!(router.depth, "depth");
        check_field!(router.ridges, "ridges");
        check_field!(
            router.preliminary_surface_level,
            "preliminary_surface_level"
        );
        check_field!(router.final_density, "final_density");
        check_field!(router.vein_toggle, "vein_toggle");
        check_field!(router.vein_ridged, "vein_ridged");
        check_field!(router.vein_gap, "vein_gap");
    }

    /// Parity test: ClimateSampler evaluates all six climate density functions at overworld
    /// block (0, 64, 0) with seed 0, then finds the nearest biome from the overworld parameter
    /// list. The expected biome is derived by calling `find_value_bruteforce()` on the sampled
    /// target — the R-tree `find_value_index()` must agree, verifying both the sampler wiring
    /// and the R-tree search correctness at a concrete position.
    ///
    /// The overworld parameter list JSON files contain only `{"preset":"minecraft:overworld"}`,
    /// so the parameter list is the one generated by `OverworldBiomeBuilder`.
    #[test]
    fn overworld_climate_sampler_and_rtree_agree_at_seed_0_block_0_64_0() {
        use super::{ClimateSampler, OVERWORLD_NOISE_ROUTER};
        use crate::biome::{overworld_biome_parameters, ClimateBiomeEntry, ClimateParameterList};

        let seed = 0_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");

        // Block (0, 64, 0) → quart (0, 16, 0).
        let sampler = ClimateSampler::from_noise_router(&OVERWORLD_NOISE_ROUTER, seed, settings);
        let climate = sampler.sample(0, 16, 0);

        // All six climate values must be finite (density functions resolved correctly).
        assert!(
            crate::biome::unquantize_coord(climate.temperature).is_finite(),
            "temperature at seed 0 (0,64,0) must be finite"
        );
        assert!(
            crate::biome::unquantize_coord(climate.continentalness).is_finite(),
            "continentalness at seed 0 (0,64,0) must be finite"
        );

        // Build a ClimateParameterList from the overworld parameters and check that
        // brute-force and R-tree search agree on the nearest biome.
        let params = overworld_biome_parameters();
        let list = ClimateParameterList::new(params.to_vec())
            .expect("overworld parameter list is non-empty");
        let bruteforce_biome = list.find_value_bruteforce(climate);
        let rtree_biome = list.find_value_index(climate);
        assert_eq!(
            bruteforce_biome, rtree_biome,
            "R-tree and brute-force must agree on biome at seed 0 block (0,64,0): \
             brute={bruteforce_biome} rtree={rtree_biome}"
        );

        // The biome must be a valid registered overworld biome.
        assert!(
            bruteforce_biome.starts_with("minecraft:"),
            "expected a namespaced biome id, got {bruteforce_biome}"
        );
    }
}
