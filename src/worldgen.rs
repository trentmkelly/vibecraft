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

    #[test]
    fn final_client_heightmaps_are_computed_from_blocks() {
        let mut chunk = crate::storage::chunk::LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        let mut block_states = crate::storage::chunk::PalettedContainer::single(
            super::block_state_tag("minecraft:air"),
            crate::storage::chunk::SECTION_VOLUME,
        );
        block_states.set_entry(
            1 * 256 + 0 * 16 + 0,
            super::block_state_tag("minecraft:dirt"),
        );
        block_states.set_entry(
            2 * 256 + 0 * 16 + 0,
            super::block_state_tag("minecraft:oak_leaves"),
        );
        chunk.sections.push(crate::storage::chunk::ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: crate::storage::chunk::PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                crate::storage::chunk::BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        super::add_client_heightmaps_from_blocks(&mut chunk);

        let Tag::LongArray(world_surface) = chunk.heightmaps.get("WORLD_SURFACE").unwrap() else {
            panic!("WORLD_SURFACE should be stored as a long array");
        };
        let Tag::LongArray(motion_blocking) = chunk.heightmaps.get("MOTION_BLOCKING").unwrap()
        else {
            panic!("MOTION_BLOCKING should be stored as a long array");
        };
        let Tag::LongArray(motion_blocking_no_leaves) =
            chunk.heightmaps.get("MOTION_BLOCKING_NO_LEAVES").unwrap()
        else {
            panic!("MOTION_BLOCKING_NO_LEAVES should be stored as a long array");
        };

        assert_eq!(unpack_heightmap_column(world_surface, 0), 3);
        assert_eq!(unpack_heightmap_column(motion_blocking, 0), 3);
        assert_eq!(unpack_heightmap_column(motion_blocking_no_leaves, 0), 2);
    }

    #[test]
    fn placed_simple_vegetation_models_cover_common_plains_features() {
        let grass = super::placed_simple_vegetation_feature("minecraft:patch_grass_plain")
            .expect("patch_grass_plain should have a simple vegetation model");
        assert_eq!(grass.configured_feature, "minecraft:grass");
        assert!(matches!(
            grass.placement.as_slice(),
            [
                super::PlacementModifier::NoiseThresholdCount { .. },
                super::PlacementModifier::InSquare,
                super::PlacementModifier::Heightmap { .. },
                super::PlacementModifier::BiomeFilter,
                super::PlacementModifier::Count { count: 32 },
                super::PlacementModifier::RandomOffset { .. },
                super::PlacementModifier::BlockPredicateFilter { .. },
            ]
        ));

        let flower = super::placed_simple_vegetation_feature("flower_plains")
            .expect("flower_plains should have a simple vegetation model");
        assert_eq!(flower.configured_feature, "minecraft:flower_plain");
        let leaf_litter = super::placed_simple_vegetation_feature("patch_leaf_litter")
            .expect("patch_leaf_litter should share the simple vegetation executor");
        assert_eq!(leaf_litter.configured_feature, "minecraft:leaf_litter");
        assert!(matches!(
            leaf_litter.placement.as_slice(),
            [
                super::PlacementModifier::Count { count: 2 },
                super::PlacementModifier::InSquare,
                super::PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::WorldSurface
                },
                super::PlacementModifier::BiomeFilter,
                super::PlacementModifier::Count { count: 32 },
                super::PlacementModifier::RandomOffset { .. },
                super::PlacementModifier::BlockPredicateFilter { .. },
            ]
        ));
        assert!(super::placed_simple_vegetation_feature("trees_plains").is_none());
    }

    #[test]
    fn simple_vegetation_phase_matches_vanilla_forest_feature_order() {
        assert_eq!(
            super::simple_vegetation_phase("minecraft:forest_flowers"),
            super::SimpleVegetationPhase::BeforeTrees,
            "forest.json lists forest_flowers before trees_birch_and_oak_leaf_litter"
        );
        assert_eq!(
            super::simple_vegetation_phase("minecraft:patch_bush"),
            super::SimpleVegetationPhase::AfterTrees,
            "forest.json lists patch_bush after trees_birch_and_oak_leaf_litter"
        );
        assert_eq!(
            super::simple_vegetation_phase("minecraft:flower_default"),
            super::SimpleVegetationPhase::AfterTrees,
            "forest.json lists flower_default after tree placement"
        );
    }

    #[test]
    fn configured_simple_vegetation_places_single_and_double_plants() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.min_section_y = 0;
        let mut block_states =
            PalettedContainer::single(super::block_state_tag("minecraft:air"), SECTION_VOLUME);
        block_states.set_entry(0, super::block_state_tag("minecraft:grass_block"));
        chunk.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let mut random = super::RandomSourceKind::new(1, super::RandomAlgorithm::Xoroshiro);
        assert_eq!(
            super::place_configured_simple_vegetation_in_target_chunk(
                &mut chunk,
                settings,
                "minecraft:grass",
                BlockPos { x: 0, y: 1, z: 0 },
                &mut random,
            ),
            1
        );
        assert_eq!(
            chunk.get_block_state_name(0, 1, 0),
            Some("minecraft:short_grass")
        );

        chunk.set_block_state(1, 0, 0, "minecraft:grass_block");
        let mut random = super::RandomSourceKind::new(2, super::RandomAlgorithm::Xoroshiro);
        assert_eq!(
            super::place_configured_simple_vegetation_in_target_chunk(
                &mut chunk,
                settings,
                "minecraft:sunflower",
                BlockPos { x: 1, y: 1, z: 0 },
                &mut random,
            ),
            2
        );
        assert_eq!(
            chunk.get_block_state_name(1, 1, 0),
            Some("minecraft:sunflower")
        );
        assert_eq!(
            chunk.get_block_state_name(1, 2, 0),
            Some("minecraft:sunflower")
        );
    }

    #[test]
    fn simple_vegetation_air_filter_rejects_property_bearing_tree_logs() {
        let log = super::carver_static_block_name("minecraft:oak_log[axis=y]")
            .expect("tree logs with properties should normalize for placement predicates");
        let context = super::block_predicate_context_for_state(log, -64, 384);

        assert!(!super::block_predicate_test(
            super::BlockPredicate::MatchingBlockTag {
                tag: "minecraft:air"
            },
            context,
            64,
        ));
    }

    #[test]
    fn simple_vegetation_block_predicate_filter_reads_below_block() {
        let settings = super::builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings should exist");
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.min_section_y = 0;
        let mut block_states =
            PalettedContainer::single(super::block_state_tag("minecraft:air"), SECTION_VOLUME);
        block_states.set_entry(
            0 * 256 + 1 * 16 + 1,
            super::block_state_tag("minecraft:grass_block"),
        );
        chunk.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:forest".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        let predicate = BlockPredicate::AllOf {
            predicates: &[
                BlockPredicate::MatchingBlockTag {
                    tag: "minecraft:air",
                },
                BlockPredicate::MatchingBlocksAt {
                    offset_y: -1,
                    blocks: &["minecraft:grass_block"],
                },
            ],
        };

        assert!(super::block_predicate_test_in_chunk(
            &chunk,
            settings,
            predicate,
            BlockPos { x: 1, y: 1, z: 1 },
        ));
        assert!(!super::block_predicate_test_in_chunk(
            &chunk,
            settings,
            predicate,
            BlockPos { x: 2, y: 1, z: 1 },
        ));
    }

    #[test]
    fn simple_vegetation_region_predicates_and_writes_cross_chunk_edges() {
        let settings = super::builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings should exist");
        let mut chunks = BTreeMap::new();
        let source_pos = ChunkPos { x: 0, z: 0 };
        let target_pos = ChunkPos { x: 1, z: 0 };
        chunks.insert(source_pos, LevelChunk::empty(source_pos));

        let mut target = LevelChunk::empty(target_pos);
        target.min_section_y = 0;
        let mut block_states =
            PalettedContainer::single(super::block_state_tag("minecraft:air"), SECTION_VOLUME);
        block_states.set_entry(0, super::block_state_tag("minecraft:grass_block"));
        target.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:forest".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });
        chunks.insert(target_pos, target);

        let predicate = BlockPredicate::AllOf {
            predicates: &[
                BlockPredicate::MatchingBlockTag {
                    tag: "minecraft:air",
                },
                BlockPredicate::MatchingBlocksAt {
                    offset_y: -1,
                    blocks: &["minecraft:grass_block"],
                },
            ],
        };
        let edge_position = BlockPos { x: 16, y: 1, z: 0 };
        assert!(super::block_predicate_test_in_region(
            &chunks,
            settings,
            predicate,
            edge_position,
        ));

        let mut random = super::RandomSourceKind::new(1, super::RandomAlgorithm::Xoroshiro);
        assert_eq!(
            super::place_configured_simple_vegetation_in_region(
                &mut chunks,
                source_pos,
                settings,
                "minecraft:grass",
                edge_position,
                &mut random,
            ),
            1
        );
        assert_eq!(
            chunks
                .get(&target_pos)
                .and_then(|chunk| chunk.get_block_state_name(16, 1, 0)),
            Some("minecraft:short_grass")
        );

        let far_position = BlockPos { x: 32, y: 1, z: 0 };
        assert_eq!(
            super::place_configured_simple_vegetation_in_region(
                &mut chunks,
                source_pos,
                settings,
                "minecraft:grass",
                far_position,
                &mut random,
            ),
            0,
            "WorldGenRegion only permits feature writes within one chunk of the source"
        );
    }

    #[test]
    fn tree_final_write_filter_allows_trunks_to_replace_grass() {
        assert!(super::tree_placement_block_can_replace(
            TreePlacementBlockKind::Log,
            "minecraft:short_grass",
        ));
        assert!(super::tree_placement_block_can_replace(
            TreePlacementBlockKind::Log,
            "minecraft:tall_grass",
        ));
        assert!(!super::tree_placement_block_can_replace(
            TreePlacementBlockKind::Log,
            "minecraft:stone",
        ));
    }

    #[test]
    fn ore_block_cache_uses_world_coordinates_without_chunk_wrapping() {
        let pos = ChunkPos { x: 2, z: -3 };
        let origin_x = pos.x * 16;
        let origin_z = pos.z * 16;
        let mut chunk = LevelChunk::empty(pos);
        chunk.min_section_y = 0;

        let mut block_states =
            PalettedContainer::single(super::block_state_tag("minecraft:stone"), SECTION_VOLUME);
        block_states.set_entry(
            1 * 256 + 5 * 16 + 15,
            super::block_state_tag("minecraft:dirt"),
        );
        chunk.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        let mut block_cache = super::OreBlockCache::from_chunk(&chunk);
        assert_eq!(
            block_cache.block_state_name(origin_x + 15, 1, origin_z + 5),
            Some("minecraft:dirt")
        );
        assert_eq!(
            block_cache.block_state_name(origin_x - 1, 1, origin_z + 5),
            None,
            "neighboring chunk reads must not wrap onto local x=15"
        );

        block_cache.set_block_state(origin_x - 1, 1, origin_z + 5, "minecraft:gold_ore");
        block_cache.set_block_state(origin_x + 1, 1, origin_z + 1, "minecraft:iron_ore");
        block_cache.flush_to_chunk(&mut chunk);

        assert_eq!(
            chunk
                .get_block_state(origin_x + 15, 1, origin_z + 5)
                .as_deref(),
            Some("minecraft:dirt"),
            "neighboring chunk writes must not wrap onto local x=15"
        );
        assert_eq!(
            chunk
                .get_block_state(origin_x + 1, 1, origin_z + 1)
                .as_deref(),
            Some("minecraft:iron_ore")
        );
    }

    #[test]
    fn ore_region_cache_reads_and_flushes_neighboring_chunks() {
        let center_pos = ChunkPos { x: 2, z: -3 };
        let west_pos = ChunkPos { x: 1, z: -3 };
        let far_west_pos = ChunkPos { x: 0, z: -3 };
        let mut chunks = BTreeMap::new();
        for (pos, marker) in [
            (center_pos, "minecraft:stone"),
            (west_pos, "minecraft:dirt"),
            (far_west_pos, "minecraft:deepslate"),
        ] {
            let mut chunk = LevelChunk::empty(pos);
            chunk.min_section_y = 0;
            chunk.sections.push(ChunkSection {
                y: 0,
                block_states: PalettedContainer::single(
                    super::block_state_tag(marker),
                    SECTION_VOLUME,
                )
                .to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String("minecraft:plains".to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: None,
            });
            chunks.insert(pos, chunk);
        }

        let center_min_x = center_pos.x * 16;
        let center_min_z = center_pos.z * 16;
        let west_world_x = center_min_x - 1;
        let far_west_world_x = center_min_x - 17;
        let world_z = center_min_z + 5;
        let mut block_cache =
            super::OreBlockCache::from_region_chunks(center_pos, &chunks).unwrap();

        assert_eq!(
            block_cache.block_state_name(west_world_x, 1, world_z),
            Some("minecraft:dirt")
        );
        assert_eq!(
            block_cache.block_state_name(far_west_world_x, 1, world_z),
            Some("minecraft:deepslate")
        );
        block_cache.set_block_state(west_world_x, 1, world_z, "minecraft:gold_ore");
        block_cache.set_block_state(far_west_world_x, 1, world_z, "minecraft:diamond_ore");
        block_cache.set_block_state(center_min_x + 1, 1, world_z, "minecraft:iron_ore");
        block_cache.flush_to_chunks(&mut chunks);

        assert_eq!(
            chunks[&west_pos]
                .get_block_state(west_world_x, 1, world_z)
                .as_deref(),
            Some("minecraft:gold_ore")
        );
        assert_eq!(
            chunks[&far_west_pos]
                .get_block_state(far_west_world_x, 1, world_z)
                .as_deref(),
            Some("minecraft:deepslate"),
            "FEATURES region writes must be rejected outside the source chunk's block-state write radius"
        );
        assert_eq!(
            chunks[&center_pos]
                .get_block_state(center_min_x + 1, 1, world_z)
                .as_deref(),
            Some("minecraft:iron_ore")
        );
    }

    #[test]
    #[ignore = "diagnostic for missing cave-air that may come from underground structures"]
    fn normal_overworld_cave_air_structure_candidate_diagnostic() {
        let fixture_json =
            include_str!("../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let fixture_chunks = fixture
            .get("chunks")
            .and_then(serde_json::Value::as_array)
            .expect("fixture should include chunks");

        let preset = super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };

        let mut generated_chunks = Vec::new();
        for fixture_chunk in fixture_chunks {
            let pos = ChunkPos {
                x: fixture_chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX") as i32,
                z: fixture_chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ") as i32,
            };
            let (base, _, _) = super::generate_real_surface_base_chunk(
                pos,
                biome_source_model,
                noise_settings,
                seed,
            )
            .expect("real-surface base generation should succeed");
            let mut chunk = base;
            super::apply_configured_carvers_for_biome_source(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
            );
            super::apply_underground_ore_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
            );
            super::apply_initial_tree_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
                None,
                None,
            );
            generated_chunks.push(chunk);
        }

        let mut missing_cave_air = Vec::new();
        for (chunk_index, fixture_chunk) in fixture_chunks.iter().enumerate() {
            let chunk_x = fixture_chunk
                .get("chunkX")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkX") as i32;
            let chunk_z = fixture_chunk
                .get("chunkZ")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkZ") as i32;
            let y_min = fixture_chunk
                .get("yMin")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include yMin") as i32;
            let blocks = fixture_chunk
                .get("blocks")
                .and_then(serde_json::Value::as_array)
                .expect("fixture chunk should include blocks");
            let generated = &generated_chunks[chunk_index];

            for (local_x, y_column) in blocks.iter().enumerate() {
                let y_column = y_column.as_array().expect("x column should be an array");
                for (y_offset, z_column) in y_column.iter().enumerate() {
                    let world_y = y_min + y_offset as i32;
                    let z_column = z_column.as_array().expect("z column should be an array");
                    for (local_z, expected) in z_column.iter().enumerate() {
                        let expected = expected
                            .as_str()
                            .expect("fixture block should be a string")
                            .split_once('[')
                            .map_or_else(|| expected.as_str().unwrap(), |(id, _)| id);
                        if expected != "minecraft:cave_air" {
                            continue;
                        }
                        let world_x = chunk_x * 16 + local_x as i32;
                        let world_z = chunk_z * 16 + local_z as i32;
                        let actual = generated
                            .get_block_state(world_x, world_y, world_z)
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        if actual == "minecraft:deepslate" || actual == "minecraft:stone" {
                            missing_cave_air.push(BlockPos {
                                x: world_x,
                                y: world_y,
                                z: world_z,
                            });
                        }
                    }
                }
            }
        }

        let missing_box = missing_cave_air.iter().copied().fold(None, |acc, pos| {
            Some(acc.map_or_else(
                || super::StructureBoundingBoxModel {
                    min_x: pos.x,
                    min_y: pos.y,
                    min_z: pos.z,
                    max_x: pos.x,
                    max_y: pos.y,
                    max_z: pos.z,
                },
                |box_: super::StructureBoundingBoxModel| box_.encapsulate_pos(pos),
            ))
        });
        eprintln!(
            "[cave-air-structure] missing_cave_air={} missing_box={:?}",
            missing_cave_air.len(),
            missing_box
        );

        let mut candidate_count = 0usize;
        let mut near_missing_count = 0usize;
        let mut closest_candidates = Vec::new();
        for source_x in -64..=64 {
            for source_z in -64..=64 {
                if !super::structure_frequency_reducer_should_generate(
                    super::FrequencyReductionMethod::LegacyType3,
                    seed,
                    0,
                    source_x,
                    source_z,
                    0.004,
                )
                .expect("mineshaft frequency check should be valid")
                {
                    continue;
                }
                candidate_count += 1;

                let mut random = super::RandomSourceKind::Legacy(super::LegacyRandom::new(
                    crate::random_source::large_feature_seed(seed, source_x, source_z),
                ));
                let first_roll = super::random_next_f64(&mut random);
                let room = super::mineshaft_room(
                    ChunkPos {
                        x: source_x,
                        z: source_z,
                    },
                    super::MineshaftTypeModel::Normal,
                    super::random_next_i32_bound(&mut random, 6),
                    super::random_next_i32_bound(&mut random, 6),
                    super::random_next_i32_bound(&mut random, 6),
                )
                .expect("room rolls should be valid");
                let intersects_missing = missing_box
                    .is_some_and(|missing| room.bounding_box.inflated_by(96).intersects(missing));
                let pieces = super::mineshaft_generate_pieces_for_start(
                    seed,
                    ChunkPos {
                        x: source_x,
                        z: source_z,
                    },
                    super::MineshaftTypeModel::Normal,
                    63,
                    -64,
                );
                let intersecting_pieces = pieces
                    .iter()
                    .filter(|piece| {
                        missing_box.is_some_and(|missing| piece.bounding_box().intersects(missing))
                    })
                    .count();
                let closest_piece = missing_box.and_then(|missing| {
                    pieces
                        .iter()
                        .enumerate()
                        .map(|(index, piece)| {
                            let bb = piece.bounding_box();
                            let dx = if bb.max_x < missing.min_x {
                                missing.min_x - bb.max_x
                            } else if missing.max_x < bb.min_x {
                                bb.min_x - missing.max_x
                            } else {
                                0
                            };
                            let dy = if bb.max_y < missing.min_y {
                                missing.min_y - bb.max_y
                            } else if missing.max_y < bb.min_y {
                                bb.min_y - missing.max_y
                            } else {
                                0
                            };
                            let dz = if bb.max_z < missing.min_z {
                                missing.min_z - bb.max_z
                            } else if missing.max_z < bb.min_z {
                                bb.min_z - missing.max_z
                            } else {
                                0
                            };
                            (dx + dy + dz, index, bb)
                        })
                        .min_by_key(|(distance, _, _)| *distance)
                });
                if let Some((distance, index, bb)) = closest_piece {
                    closest_candidates.push((
                        distance,
                        source_x,
                        source_z,
                        pieces.len(),
                        index,
                        bb,
                    ));
                }
                if intersects_missing || intersecting_pieces > 0 {
                    if intersects_missing {
                        near_missing_count += 1;
                    }
                    eprintln!(
                        "[cave-air-structure-candidate] chunk=({}, {}) first_roll={:.9} room={:?} generated_pieces={} intersecting_pieces={} closest_piece={:?}",
                        source_x,
                        source_z,
                        first_roll,
                        room.bounding_box,
                        pieces.len(),
                        intersecting_pieces,
                        closest_piece
                    );
                }
            }
        }
        closest_candidates.sort_by_key(|(distance, _, _, _, _, _)| *distance);
        for (distance, source_x, source_z, piece_count, piece_index, bb) in
            closest_candidates.into_iter().take(8)
        {
            eprintln!(
                "[cave-air-structure-closest] distance={} chunk=({}, {}) generated_pieces={} piece_index={} box={:?}",
                distance, source_x, source_z, piece_count, piece_index, bb
            );
        }
        eprintln!(
            "[cave-air-structure] candidate_count={} near_missing_count={}",
            candidate_count, near_missing_count
        );
    }

    #[test]
    #[ignore = "diagnostic for the mineshaft start nearest the vanilla cave-air mismatch"]
    fn normal_overworld_mineshaft_nearest_start_diagnostic() {
        let fixture_json =
            include_str!("../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let preset = super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };
        let router =
            super::builtin_noise_router(super::noise_router_id_for_settings(**noise_settings))
                .expect("normal overworld should have a router")
                .router;
        let climate_sampler =
            super::ClimateSampler::from_noise_router(&router, seed, **noise_settings);

        let source_pos = ChunkPos { x: -1, z: 4 };
        let start_pos = super::mineshaft_start_pos(source_pos);
        let pieces = super::mineshaft_generate_pieces_for_start(
            seed,
            source_pos,
            super::MineshaftTypeModel::Normal,
            noise_settings.sea_level,
            noise_settings.noise.min_y,
        );
        let room_box = pieces[0].bounding_box();
        let y_offset = room_box.min_y - 50;
        let stub_pos = BlockPos {
            x: start_pos.x,
            y: start_pos.y + y_offset,
            z: start_pos.z,
        };
        let biome = super::get_biome(
            biome_source_model,
            stub_pos.x >> 2,
            stub_pos.y >> 2,
            stub_pos.z >> 2,
            &climate_sampler,
        )
        .unwrap_or("minecraft:unknown");
        eprintln!(
            "[mineshaft-nearest] source=({}, {}) seed={} start={:?} y_offset={} stub={:?} biome={} pieces={}",
            source_pos.x,
            source_pos.z,
            seed,
            start_pos,
            y_offset,
            stub_pos,
            biome,
            pieces.len()
        );

        for (index, piece) in pieces.iter().enumerate().take(40) {
            let kind = match piece {
                super::MineshaftGeneratedPieceModel::Room { .. } => "room",
                super::MineshaftGeneratedPieceModel::Corridor { .. } => "corridor",
                super::MineshaftGeneratedPieceModel::Crossing { .. } => "crossing",
                super::MineshaftGeneratedPieceModel::Stairs { .. } => "stairs",
            };
            eprintln!(
                "[mineshaft-nearest-first] index={} kind={} depth={} box={:?}",
                index,
                kind,
                piece.gen_depth(),
                piece.bounding_box()
            );
        }

        let missing = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: -56,
            min_z: 0,
            max_x: 22,
            max_y: -45,
            max_z: 15,
        };
        for (index, piece) in pieces.iter().enumerate() {
            let bb = piece.bounding_box();
            let dx = if bb.max_x < missing.min_x {
                missing.min_x - bb.max_x
            } else if missing.max_x < bb.min_x {
                bb.min_x - missing.max_x
            } else {
                0
            };
            let dy = if bb.max_y < missing.min_y {
                missing.min_y - bb.max_y
            } else if missing.max_y < bb.min_y {
                bb.min_y - missing.max_y
            } else {
                0
            };
            let dz = if bb.max_z < missing.min_z {
                missing.min_z - bb.max_z
            } else if missing.max_z < bb.min_z {
                bb.min_z - missing.max_z
            } else {
                0
            };
            let distance = dx + dy + dz;
            if distance <= 24 {
                let kind = match piece {
                    super::MineshaftGeneratedPieceModel::Room { .. } => "room",
                    super::MineshaftGeneratedPieceModel::Corridor { .. } => "corridor",
                    super::MineshaftGeneratedPieceModel::Crossing { .. } => "crossing",
                    super::MineshaftGeneratedPieceModel::Stairs { .. } => "stairs",
                };
                eprintln!(
                    "[mineshaft-nearest-piece] index={} kind={} depth={} distance={} box={:?}",
                    index,
                    kind,
                    piece.gen_depth(),
                    distance,
                    bb
                );
            }
        }
    }

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

    #[test]
    fn real_surface_generation_mode_depends_on_world_seed() {
        let chunk_seed_0 = super::generate_overworld_chunk_for_preset_with_mode(
            ChunkPos { x: 0, z: 0 },
            "normal",
            super::LiveChunkGenerationMode::RealSurface,
            0,
        )
        .expect("real-surface mode should generate seed 0 terrain");
        let chunk_seed_1 = super::generate_overworld_chunk_for_preset_with_mode(
            ChunkPos { x: 0, z: 0 },
            "normal",
            super::LiveChunkGenerationMode::RealSurface,
            1,
        )
        .expect("real-surface mode should generate seed 1 terrain");

        assert_ne!(
            chunk_seed_0.heightmaps.get("WORLD_SURFACE_WG"),
            chunk_seed_1.heightmaps.get("WORLD_SURFACE_WG"),
            "real-surface generation must not ignore the world seed"
        );
    }

    #[test]
    fn generator_method_facade_exposes_vanilla_status_task_names() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let pos = ChunkPos { x: 1, z: -1 };

        assert_eq!(
            super::generator_create_structures_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:structure_starts"
        );
        assert_eq!(
            super::generator_create_references_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:structure_references"
        );
        assert_eq!(
            super::generator_create_biomes_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:biomes"
        );
        assert_eq!(
            super::generator_fill_from_noise_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:noise"
        );
        assert_eq!(
            super::generator_build_surface_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:surface"
        );
        assert_eq!(
            super::generator_apply_carvers_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:carvers"
        );
        assert_eq!(
            super::generator_apply_biome_decoration_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:features"
        );
        assert_eq!(
            super::generator_spawn_original_mobs_for_stem(pos, &normal.overworld)
                .unwrap()
                .status,
            "minecraft:spawn"
        );

        let noise_chunk =
            super::generator_fill_from_noise_for_stem(pos, &normal.overworld).unwrap();
        assert_eq!(noise_chunk.sections.len(), 24);
        assert!(noise_chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
    }

    #[test]
    fn generator_apply_carvers_mutates_surface_chunk_blocks() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let pos = ChunkPos { x: 1, z: -1 };
        let surface = super::generator_build_surface_for_stem(pos, &normal.overworld)
            .expect("surface chunk must generate before carvers");
        let carvers = super::generator_apply_carvers_for_stem(pos, &normal.overworld)
            .expect("carver status must execute");

        assert_eq!(carvers.status, "minecraft:carvers");
        assert!(
            carvers
                .carving_mask
                .as_ref()
                .is_some_and(|mask| !mask.is_empty()),
            "carver execution should persist a carving mask on the generated chunk"
        );
        assert_ne!(
            surface.heightmaps.get("WORLD_SURFACE"),
            carvers.heightmaps.get("WORLD_SURFACE"),
            "carver execution should recompute client heightmaps after mutating blocks"
        );

        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let mut changed_blocks = 0;
        'scan: for y in settings.noise.min_y..settings.noise.min_y + settings.noise.height {
            for z in pos.z * 16..pos.z * 16 + 16 {
                for x in pos.x * 16..pos.x * 16 + 16 {
                    if surface.get_block_state(x, y, z) != carvers.get_block_state(x, y, z) {
                        changed_blocks += 1;
                        if changed_blocks >= 16 {
                            break 'scan;
                        }
                    }
                }
            }
        }
        assert!(
            changed_blocks >= 16,
            "configured carvers should replace terrain blocks with cave air or lava"
        );
    }

    #[test]
    fn generator_apply_carvers_selects_carvers_from_resolved_biome_source() {
        let normal = super::resolve_world_preset("normal").unwrap();
        let pos = ChunkPos { x: 0, z: 0 };

        let super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &normal.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };
        assert_eq!(
            super::carvers_for_biome_source_and_noise_settings(
                biome_source_model,
                pos,
                noise_settings,
                0,
            ),
            super::OVERWORLD_COMMON_CARVERS
        );

        let super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &normal.nether.generator
        else {
            panic!("normal nether should use a noise generator");
        };
        assert_eq!(
            super::carvers_for_biome_source_and_noise_settings(
                biome_source_model,
                pos,
                noise_settings,
                0,
            ),
            super::NETHER_COMMON_CARVERS
        );

        let super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &normal.end.generator
        else {
            panic!("normal end should use a noise generator");
        };
        assert_eq!(
            super::carvers_for_biome_source_and_noise_settings(
                biome_source_model,
                pos,
                noise_settings,
                0,
            ),
            &[] as &[&'static str]
        );
    }

    #[test]
    fn cave_tunnel_carver_mutates_chunk_blocks_and_mask() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut chunk = LevelChunk::empty(pos);
        let height_context = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        chunk.min_section_y = 3;
        chunk.sections = (3..=5)
            .map(|section_y| ChunkSection {
                y: section_y,
                block_states: PalettedContainer::single(
                    Tag::Compound(vec![(
                        "Name".to_string(),
                        Tag::String("minecraft:stone".to_string()),
                    )]),
                    SECTION_VOLUME,
                )
                .to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String("minecraft:plains".to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: Some(vec![-1; 2048]),
            })
            .collect();

        let cave = super::configured_carver("cave").unwrap();
        let mut mask = Vec::new();
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let noise_router = super::OVERWORLD_NOISE_ROUTER;
        let noise_chunk = super::NoiseChunk::new(0, 0, *settings, 0, noise_router);
        let carved = super::carve_cave_tunnel_into_chunk(
            &mut chunk,
            height_context,
            cave,
            0,
            0,
            12_345,
            8.0,
            64.0,
            8.0,
            1.0,
            1.0,
            2.5,
            0.0,
            0.0,
            0,
            24,
            1.0,
            -0.7,
            &mut mask,
            0,
            settings,
            &noise_chunk,
            None,
        );

        assert!(carved > 0, "cave tunnel walking should carve stone blocks");
        assert!(
            !mask.is_empty(),
            "cave tunnel carving should record mask bits"
        );
        assert!(
            (48..=80).any(|y| (0..16).any(|z| (0..16)
                .any(|x| chunk.get_block_state(x, y, z).as_deref() == Some("minecraft:cave_air")))),
            "cave tunnel carving should replace at least one local stone block with cave air"
        );
    }

    #[test]
    fn canyon_tunnel_carver_mutates_chunk_blocks_and_mask() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut chunk = LevelChunk::empty(pos);
        let height_context = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        chunk.min_section_y = 2;
        chunk.sections = (2..=5)
            .map(|section_y| ChunkSection {
                y: section_y,
                block_states: PalettedContainer::single(
                    Tag::Compound(vec![(
                        "Name".to_string(),
                        Tag::String("minecraft:stone".to_string()),
                    )]),
                    SECTION_VOLUME,
                )
                .to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String("minecraft:plains".to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: Some(vec![-1; 2048]),
            })
            .collect();

        let canyon = super::configured_carver("canyon").unwrap();
        let CarverShape::Canyon { shape, .. } = canyon.shape else {
            panic!("canyon configured carver must use canyon shape");
        };
        let mut mask = Vec::new();
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let noise_router = super::OVERWORLD_NOISE_ROUTER;
        let noise_chunk = super::NoiseChunk::new(0, 0, *settings, 0, noise_router);
        let carved = super::carve_canyon_tunnel_into_chunk(
            &mut chunk,
            height_context,
            canyon,
            0,
            0,
            98_765,
            8.0,
            56.0,
            8.0,
            4.0,
            0.0,
            0.0,
            32,
            3.0,
            &shape,
            &mut mask,
            settings,
            &noise_chunk,
            None,
        );

        assert!(
            carved > 0,
            "canyon tunnel walking should carve stone blocks"
        );
        assert!(
            !mask.is_empty(),
            "canyon tunnel carving should record mask bits"
        );
        assert!(
            (32..=80).any(|y| (0..16).any(|z| (0..16)
                .any(|x| chunk.get_block_state(x, y, z).as_deref() == Some("minecraft:cave_air")))),
            "canyon tunnel carving should replace at least one local stone block with cave air"
        );
    }

    #[test]
    fn carving_mask_indices_pack_to_persisted_bitset_words() {
        let packed = super::pack_carving_mask_indices(&[0, 1, 63, 64, 130]);
        assert_eq!(packed.len(), 3);
        assert_eq!(packed[0] as u64, 0x8000_0000_0000_0003);
        assert_eq!(packed[1] as u64, 0x0000_0000_0000_0001);
        assert_eq!(packed[2] as u64, 0x0000_0000_0000_0004);
    }

    #[test]
    fn overworld_chunk_at_origin_has_correct_section_count_and_heightmaps() {
        // Mirrors Java's ChunkStatus parity expectation: overworld (0,0) produced by the
        // noise generator must have 24 sections (minY=-64, height=384, 384/16=24) and
        // the WORLD_SURFACE_WG and OCEAN_FLOOR_WG heightmaps required for worldgen.
        // Source: decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Heightmap.java
        let preset = super::resolve_world_preset("normal").unwrap();
        let chunk = super::generate_chunk_for_stem(ChunkPos { x: 0, z: 0 }, &preset.overworld)
            .expect("overworld chunk generation must not fail at (0,0)");

        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let expected_sections = (settings.noise.height / 16) as usize;
        assert_eq!(
            chunk.sections.len(),
            expected_sections,
            "expected {expected_sections} sections for height={}",
            settings.noise.height
        );
        assert!(
            chunk.heightmaps.contains_key("WORLD_SURFACE_WG"),
            "WORLD_SURFACE_WG heightmap must be present"
        );
        assert!(
            chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"),
            "OCEAN_FLOOR_WG heightmap must be present"
        );
    }

    #[test]
    fn lightweight_tree_context_heights_match_noise_chunk_heightmaps() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let router = super::builtin_noise_router(super::noise_router_id_for_settings(*settings))
            .unwrap()
            .router;
        for (seed, pos) in [
            (0, ChunkPos { x: 1, z: -1 }),
            (0, ChunkPos { x: -1, z: -1 }),
            (1, ChunkPos { x: 2, z: 0 }),
            (42, ChunkPos { x: -2, z: 3 }),
        ] {
            let chunk = super::fill_from_noise_chunk(pos, settings, seed, router);
            let full_heights = super::tree_decoration_terrain_heights_from_wg(&chunk, settings);
            let lightweight_heights =
                super::noise_tree_context_heights(pos, settings, seed, router);

            assert_eq!(
                lightweight_heights.ocean_floor, full_heights.ocean_floor,
                "lightweight ocean floor heightmap should match full noise chunk for seed={seed} pos=({}, {})",
                pos.x, pos.z
            );
            assert_eq!(
                lightweight_heights.world_surface, full_heights.world_surface,
                "lightweight world surface heightmap should match full noise chunk for seed={seed} pos=({}, {})",
                pos.x, pos.z
            );
        }
    }

    #[test]
    fn noise_preview_trees_follow_biome_generation_settings() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let terrain_heights = [settings.sea_level + 8; 16 * 16];
        let plains = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:plains",
            &terrain_heights,
        );
        let forest = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:forest",
            &terrain_heights,
        );
        let unknown = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:badlands",
            &terrain_heights,
        );

        assert!(forest.len() > plains.len());
        assert!(forest
            .iter()
            .any(|block| block.state == "minecraft:birch_log"));
        let cherry = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:cherry_grove",
            &terrain_heights,
        );
        let swamp = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:swamp",
            &terrain_heights,
        );
        let mangrove = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:mangrove_swamp",
            &terrain_heights,
        );
        let dark_forest = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:dark_forest",
            &terrain_heights,
        );
        assert!(cherry
            .iter()
            .any(|block| block.state == "minecraft:cherry_log"));
        assert!(swamp.iter().any(|block| block.state == "minecraft:oak_log"));
        assert!(mangrove
            .iter()
            .any(|block| block.state == "minecraft:mangrove_log"));
        assert!(dark_forest
            .iter()
            .any(|block| block.state == "minecraft:dark_oak_log"));
        assert!(unknown.is_empty());
    }

    #[test]
    fn forest_preview_chunk_uses_forest_decoration_palette() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let chunk = super::materialize_noise_preview_chunk(
            ChunkPos { x: 0, z: 0 },
            &BiomeSourceModel::Fixed {
                biome: "minecraft:forest",
            },
            settings,
        );

        assert!(chunk.sections.iter().any(|section| {
            let Tag::Compound(block_states) = &section.block_states else {
                return false;
            };
            let Some((_, Tag::List(palette))) =
                block_states.iter().find(|(name, _)| name == "palette")
            else {
                return false;
            };
            palette.contains(&super::block_state_tag("minecraft:birch_log"))
                || palette.contains(&super::block_state_tag("minecraft:birch_leaves"))
        }));
    }

    #[test]
    fn preview_tree_overlay_updates_world_surface_wg_heightmap() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let mut terrain_heights = [0; 16 * 16];
        for z in 0..16 {
            for x in 0..16 {
                terrain_heights[z * 16 + x] =
                    super::noise_preview_terrain_height(x as i32, z as i32, settings).clamp(
                        settings.noise.min_y + 1,
                        settings.noise.min_y + settings.noise.height,
                    );
            }
        }
        let overlay = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            0,
            settings,
            "minecraft:forest",
            &terrain_heights,
        );
        let tallest = overlay
            .iter()
            .max_by_key(|block| block.pos.y)
            .expect("forest preview should emit visible tree blocks");
        let column = tallest.pos.z as usize * 16 + tallest.pos.x as usize;

        let chunk = super::materialize_noise_preview_chunk(
            ChunkPos { x: 0, z: 0 },
            &BiomeSourceModel::Fixed {
                biome: "minecraft:forest",
            },
            settings,
        );
        let Tag::LongArray(world_surface_wg) = chunk.heightmaps.get("WORLD_SURFACE_WG").unwrap()
        else {
            panic!("WORLD_SURFACE_WG should be stored as a long array");
        };

        assert_eq!(
            unpack_heightmap_column(world_surface_wg, column),
            tallest.pos.y + 1
        );
        assert!(
            tallest.pos.y + 1 > terrain_heights[column].max(settings.sea_level + 1),
            "tree overlay should raise the world surface above terrain"
        );
    }

    #[test]
    fn noise_preview_ground_cover_follows_biome_features() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let terrain_heights = [settings.sea_level + 8; 16 * 16];
        let plains = super::noise_preview_ground_cover_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:plains",
            &terrain_heights,
        );
        let sunflower = super::noise_preview_ground_cover_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:sunflower_plains",
            &terrain_heights,
        );
        let unknown = super::noise_preview_ground_cover_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:badlands",
            &terrain_heights,
        );

        assert!(plains
            .iter()
            .any(|block| matches!(block.state, "minecraft:short_grass" | "minecraft:dandelion")));
        assert!(sunflower
            .iter()
            .any(|block| block.state == "minecraft:sunflower"));
        assert!(sunflower.len() >= plains.len());
        assert!(unknown.is_empty());
    }

    #[test]
    fn resolved_generators_answer_base_height_and_column_queries() {
        let flat = super::resolve_world_preset("flat").unwrap();
        assert_eq!(
            super::generator_base_height_for_stem(
                0,
                0,
                HeightmapKind::WorldSurfaceWg,
                &flat.overworld
            )
            .unwrap(),
            4
        );
        let flat_column = super::generator_base_column_for_stem(0, 0, &flat.overworld).unwrap();
        assert_eq!(flat_column.min_y, super::FLAT_GENERATOR_MIN_Y);
        assert_eq!(flat_column.states[0], "minecraft:bedrock");
        assert_eq!(flat_column.states[3], "minecraft:grass_block");

        let normal = super::resolve_world_preset("normal").unwrap();
        let world_surface = super::generator_base_height_for_stem(
            96,
            -48,
            HeightmapKind::WorldSurfaceWg,
            &normal.overworld,
        )
        .unwrap();
        let ocean_floor = super::generator_base_height_for_stem(
            96,
            -48,
            HeightmapKind::OceanFloorWg,
            &normal.overworld,
        )
        .unwrap();
        assert!(world_surface >= ocean_floor);

        let column = super::generator_base_column_for_stem(96, -48, &normal.overworld).unwrap();
        assert_eq!(column.min_y, super::OVERWORLD_NOISE_SETTINGS.min_y);
        assert_eq!(
            column.states.len(),
            super::OVERWORLD_NOISE_SETTINGS.height as usize
        );
        assert_eq!(column.states[0], "minecraft:bedrock");
        assert!(column.states.contains(&"minecraft:stone"));
        assert!(column.states.contains(&"minecraft:air"));
    }

    #[test]
    fn unresolved_debug_generation_fails_closed() {
        assert_eq!(
            super::generate_overworld_chunk_for_preset(
                ChunkPos { x: 0, z: 0 },
                "debug_all_block_states"
            )
            .unwrap_err(),
            "Debug chunk generation for minecraft:overworld is not implemented".to_string()
        );
    }

    #[test]
    fn noise_generator_settings_bootstrap_matches_vanilla_order_and_flags() {
        assert_eq!(
            BUILTIN_NOISE_GENERATOR_SETTINGS
                .iter()
                .map(|settings| settings.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:caves",
                "minecraft:floating_islands",
            ]
        );

        let overworld = builtin_noise_generator_settings("overworld").unwrap();
        assert_eq!(overworld.default_block, "minecraft:stone");
        assert_eq!(overworld.default_fluid, "minecraft:water");
        assert_eq!(
            overworld.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: false
            }
        );
        assert_eq!(overworld.surface_rule, SurfaceRulePreset::Overworld);
        assert_eq!(overworld.sea_level, 63);
        assert!(overworld.aquifers_enabled);
        assert!(overworld.ore_veins_enabled);
        assert!(!overworld.legacy_random_source);

        let large = builtin_noise_generator_settings("large_biomes").unwrap();
        assert_eq!(
            large.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: true,
                amplified: false
            }
        );
        let amplified = builtin_noise_generator_settings("amplified").unwrap();
        assert_eq!(
            amplified.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: true
            }
        );

        let nether = builtin_noise_generator_settings("nether").unwrap();
        assert_eq!(nether.default_block, "minecraft:netherrack");
        assert_eq!(nether.default_fluid, "minecraft:lava");
        assert_eq!(nether.sea_level, 32);
        assert!(nether.legacy_random_source);

        let end = builtin_noise_generator_settings("end").unwrap();
        assert_eq!(end.default_block, "minecraft:end_stone");
        assert_eq!(end.default_fluid, "minecraft:air");
        assert!(end.disable_mob_generation);
        assert_eq!(end.sea_level, 0);
    }

    #[test]
    fn noise_generator_settings_cover_extracted_registry_jsons() {
        assert_eq!(
            super::EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:amplified",
                "minecraft:caves",
                "minecraft:end",
                "minecraft:floating_islands",
                "minecraft:large_biomes",
                "minecraft:nether",
                "minecraft:overworld",
            ]
        );
        assert_eq!(
            super::EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS.len(),
            BUILTIN_NOISE_GENERATOR_SETTINGS.len()
        );

        for expected in super::EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS {
            let settings = builtin_noise_generator_settings(expected.id)
                .unwrap_or_else(|| panic!("missing {}", expected.id));
            assert_eq!(settings.noise, expected.noise, "{}", expected.id);
            assert_eq!(
                settings.default_block, expected.default_block,
                "{}",
                expected.id
            );
            assert_eq!(
                settings.default_fluid, expected.default_fluid,
                "{}",
                expected.id
            );
            assert_eq!(
                super::noise_router_id_for_settings(*settings),
                expected.router_id,
                "{}",
                expected.id
            );
            assert_eq!(
                settings.surface_rule, expected.surface_rule,
                "{}",
                expected.id
            );
            assert_eq!(
                settings.spawn_target.len(),
                expected.spawn_target_len,
                "{}",
                expected.id
            );
            assert_eq!(settings.sea_level, expected.sea_level, "{}", expected.id);
            assert_eq!(
                settings.disable_mob_generation, expected.disable_mob_generation,
                "{}",
                expected.id
            );
            assert_eq!(
                settings.aquifers_enabled, expected.aquifers_enabled,
                "{}",
                expected.id
            );
            assert_eq!(
                settings.ore_veins_enabled, expected.ore_veins_enabled,
                "{}",
                expected.id
            );
            assert_eq!(
                settings.legacy_random_source, expected.legacy_random_source,
                "{}",
                expected.id
            );
            assert!(builtin_noise_router(expected.router_id).is_some());
        }
    }

    #[test]
    fn overworld_spawn_target_matches_overworld_biome_builder() {
        assert_eq!(OVERWORLD_SPAWN_TARGET.len(), 2);
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.min,
            quantize_coord(-0.11)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.max,
            quantize_coord(1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.min,
            quantize_coord(-1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.max,
            quantize_coord(-0.16)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[1].weirdness.min,
            quantize_coord(0.16)
        );
        assert_eq!(OVERWORLD_SPAWN_TARGET[1].weirdness.max, quantize_coord(1.0));
    }

    #[test]
    fn density_function_type_registry_matches_densityfunctions_bootstrap_order() {
        assert_eq!(DENSITY_FUNCTION_TYPES.len(), 34);
        assert_eq!(
            DENSITY_FUNCTION_TYPES
                .iter()
                .map(|kind| kind.id)
                .collect::<Vec<_>>(),
            vec![
                "blend_alpha",
                "blend_offset",
                "beardifier",
                "old_blended_noise",
                "interpolated",
                "flat_cache",
                "cache_2d",
                "cache_once",
                "cache_all_in_cell",
                "noise",
                "end_islands",
                "weird_scaled_sampler",
                "shifted_noise",
                "range_choice",
                "shift_a",
                "shift_b",
                "shift",
                "blend_density",
                "clamp",
                "abs",
                "square",
                "cube",
                "half_negative",
                "quarter_negative",
                "invert",
                "squeeze",
                "add",
                "mul",
                "min",
                "max",
                "spline",
                "constant",
                "y_clamped_gradient",
                "find_top_surface",
            ]
        );
        assert!(density_function_type("shifted_noise").is_some());
        assert!(density_function_type("missing").is_none());
    }

    #[test]
    fn cache_all_in_cell_uses_wrapper_owned_cell_values_for_scalar_and_array_reads() {
        let settings = *builtin_noise_generator_settings("overworld").unwrap();
        let router = super::NoiseRouter {
            barrier: DensityFunction::Constant(0.0),
            fluid_level_floodedness: DensityFunction::Constant(0.0),
            fluid_level_spread: DensityFunction::Constant(0.0),
            lava: DensityFunction::Constant(0.0),
            temperature: DensityFunction::Constant(0.0),
            vegetation: DensityFunction::Constant(0.0),
            continents: DensityFunction::Constant(0.0),
            erosion: DensityFunction::Constant(0.0),
            depth: DensityFunction::Constant(0.0),
            ridges: DensityFunction::Constant(0.0),
            preliminary_surface_level: DensityFunction::Constant(0.0),
            final_density: super::TEST_CACHE_ALL_IN_CELL_DENSITY,
            vein_toggle: DensityFunction::Constant(0.0),
            vein_ridged: DensityFunction::Constant(0.0),
            vein_gap: DensityFunction::Constant(0.0),
        };
        let mut chunk = super::NoiseChunk::new(0, 0, settings, 0, router);

        chunk.advance_cell_x(0);
        chunk.select_cell_yz(0, 0);
        chunk.update_for_x(0, 0.0);
        chunk.update_for_y(settings.noise.min_y, 0.0);
        chunk.update_for_z(0, 0.0);

        let value_index = chunk
            .cache_all_cell_index()
            .expect("selected block should be inside the current noise cell");
        assert_eq!(chunk.cache_all_in_cell.len(), 1);
        chunk.cache_all_in_cell[0].values[value_index] = 42.25;

        assert_eq!(
            super::eval_density_fn_with_interp(
                super::TEST_CACHE_ALL_IN_CELL_DENSITY,
                &chunk,
                0,
                settings.noise.min_y,
                0,
            ),
            42.25,
            "CacheAllInCell scalar compute should read the wrapper-owned cell cache"
        );

        let mut output = vec![0.0; chunk.cache_all_in_cell[0].values.len()];
        super::fill_density_array_with_interp(
            super::TEST_CACHE_ALL_IN_CELL_DENSITY,
            &mut chunk,
            &mut output,
            super::DensityArrayFillMode::Cell,
        );
        assert_eq!(
            output, chunk.cache_all_in_cell[0].values,
            "CacheAllInCell fillArray should copy the wrapper-owned cell cache"
        );
    }

    #[test]
    fn density_function_core_evaluators_follow_vanilla_transform_rules() {
        assert_eq!(Y_DENSITY.compute(-5000), -4064.0);
        assert_eq!(Y_DENSITY.compute(5000), 4062.0);
        assert_eq!(Y_DENSITY.compute(0), 0.0);

        assert_eq!(MappedDensityFunction::Abs.transform(-2.0), 2.0);
        assert_eq!(MappedDensityFunction::Square.transform(-2.0), 4.0);
        assert_eq!(MappedDensityFunction::Cube.transform(-2.0), -8.0);
        assert_eq!(MappedDensityFunction::HalfNegative.transform(-2.0), -1.0);
        assert_eq!(MappedDensityFunction::QuarterNegative.transform(-2.0), -0.5);
        assert_eq!(MappedDensityFunction::Invert.transform(2.0), 0.5);
        assert!((MappedDensityFunction::Squeeze.transform(1.0) - 0.4583333333333333).abs() < 1e-12);

        let add = DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &TEST_NEGATIVE_DENSITY,
            argument2: &TEST_POSITIVE_DENSITY,
        };
        assert_eq!(add.compute(0), 1.0);
        assert_eq!(BinaryDensityFunction::Mul.apply(-2.0, 3.0), -6.0);
        assert_eq!(BinaryDensityFunction::Min.apply(-2.0, 3.0), -2.0);
        assert_eq!(BinaryDensityFunction::Max.apply(-2.0, 3.0), 3.0);
        assert_eq!(super::SHIFT_A_DENSITY.type_name(), "shift_a");
        assert_eq!(super::SHIFT_B_DENSITY.type_name(), "shift_b");
        let shift_bounds = super::SHIFT_A_DENSITY.value_bounds();
        let shift_noise_bounds = super::normal_noise_value_bounds("minecraft:offset").unwrap();
        assert_eq!(
            shift_bounds,
            (shift_noise_bounds.0 * 4.0, shift_noise_bounds.1 * 4.0)
        );
        assert_eq!(
            BinaryDensityFunction::Mul.apply_lazy(0.0, (3.0, 3.0), || panic!(
                "mul should skip zero second argument"
            )),
            0.0
        );
        assert_eq!(
            BinaryDensityFunction::Min.apply_lazy(-5.0, (3.0, 3.0), || panic!(
                "min should skip higher second argument"
            )),
            -5.0
        );
        assert_eq!(
            BinaryDensityFunction::Max.apply_lazy(5.0, (-3.0, -3.0), || panic!(
                "max should skip lower second argument"
            )),
            5.0
        );
        assert_eq!(Y_DENSITY.value_bounds(), (-4064.0, 4062.0));
        assert_eq!(
            DensityFunction::Binary {
                kind: BinaryDensityFunction::Add,
                argument1: &TEST_NEGATIVE_DENSITY,
                argument2: &TEST_POSITIVE_DENSITY,
            }
            .value_bounds(),
            (1.0, 1.0)
        );
        assert_eq!(
            super::find_top_surface_compute(
                |sample_y| if sample_y <= 72 { 0.25 } else { -0.25 },
                83.9,
                -64,
                4,
            ),
            72.0
        );
        assert_eq!(
            super::find_top_surface_compute(|_| -0.25, -80.0, -64, 4),
            -64.0
        );
        assert_eq!(
            DensityFunction::FindTopSurface {
                density: &TEST_POSITIVE_DENSITY,
                upper_bound: &TEST_POSITIVE_DENSITY,
                lower_bound: -64,
                cell_height: 4,
            }
            .type_name(),
            "find_top_surface"
        );
        assert_eq!(
            DensityFunction::FindTopSurface {
                density: &TEST_POSITIVE_DENSITY,
                upper_bound: &TEST_POSITIVE_DENSITY,
                lower_bound: -64,
                cell_height: 4,
            }
            .value_bounds(),
            (-64.0, 3.0)
        );
        assert_eq!(super::RarityValueMapper::Type1.max_rarity(), 2.0);
        assert_eq!(super::RarityValueMapper::Type2.max_rarity(), 3.0);
        let weird_bounds = DensityFunction::WeirdScaledSampler {
            input: &TEST_POSITIVE_DENSITY,
            noise: "minecraft:spaghetti_3d_1",
            rarity_mapper: super::RarityValueMapper::Type1,
        }
        .value_bounds();
        assert_eq!(weird_bounds.0, 0.0);
        assert!(weird_bounds.1.is_finite());
        assert!(weird_bounds.1 > 0.0);
        assert_eq!(
            DensityFunction::Noise {
                noise: "minecraft:temperature",
                xz_scale: 0.25,
                y_scale: 0.0,
            }
            .value_bounds(),
            (-4.444444444444445, 4.444444444444445)
        );
        assert_eq!(
            DensityFunction::ShiftedNoise {
                shift_x: &super::SHIFT_X_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::SHIFT_Z_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:temperature",
            }
            .value_bounds(),
            (-4.444444444444445, 4.444444444444445)
        );

        assert_eq!(super::TEST_RANGE_CHOICE_DENSITY.type_name(), "range_choice");
        assert_eq!(super::TEST_RANGE_CHOICE_DENSITY.compute(0), 3.0);
        assert_eq!(super::TEST_RANGE_CHOICE_DENSITY.compute(-64), -2.0);
        assert_eq!(super::TEST_RANGE_CHOICE_DENSITY.compute(64), -2.0);
        let overworld = *super::builtin_noise_generator_settings("overworld").unwrap();
        assert_eq!(
            super::TEST_RANGE_CHOICE_DENSITY.compute_with_noise(12345, overworld, 0, 0, 0),
            3.0
        );
        assert_eq!(
            super::TEST_RANGE_CHOICE_DENSITY.compute_with_noise(12345, overworld, 0, 64, 0),
            -2.0
        );
    }

    #[test]
    fn density_function_noise_evaluators_resolve_random_state_noise_holders() {
        let overworld = *super::builtin_noise_generator_settings("overworld").unwrap();
        let noise = DensityFunction::Noise {
            noise: "minecraft:temperature",
            xz_scale: 0.25,
            y_scale: 0.0,
        };
        let shifted = DensityFunction::ShiftedNoise {
            shift_x: &super::SHIFT_X_DENSITY,
            shift_y: &super::ZERO_DENSITY,
            shift_z: &super::SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:temperature",
        };
        let weird = DensityFunction::WeirdScaledSampler {
            input: &TEST_POSITIVE_DENSITY,
            noise: "minecraft:spaghetti_3d_1",
            rarity_mapper: super::RarityValueMapper::Type1,
        };

        let noise_value = noise.compute_with_noise(12345, overworld, 16, 64, -32);
        let shifted_value = shifted.compute_with_noise(12345, overworld, 16, 64, -32);
        let weird_value = weird.compute_with_noise(12345, overworld, 16, 64, -32);
        let shift_a_value =
            super::SHIFT_A_DENSITY.compute_with_noise(12345, overworld, 16, 64, -32);
        assert!((noise_value - -0.02846337681055331).abs() < 1e-12);
        assert!((shifted_value - -0.02854944566757223).abs() < 1e-12);
        assert!((weird_value - 0.5833159524778098).abs() < 1e-12);
        assert!(
            (shift_a_value
                - super::density_shift_noise_sample(
                    12345,
                    overworld,
                    "minecraft:offset",
                    16.0,
                    0.0,
                    -32.0,
                ))
            .abs()
                < 1e-12
        );

        assert_eq!(super::RarityValueMapper::Type1.map_value(-0.75), 0.75);
        assert_eq!(super::RarityValueMapper::Type1.map_value(0.25), 1.5);
        assert_eq!(super::RarityValueMapper::Type2.map_value(-0.8), 0.5);
        assert_eq!(super::RarityValueMapper::Type2.map_value(0.8), 3.0);
    }

    #[test]
    fn blended_noise_evaluator_uses_vanilla_legacy_octave_stack() {
        let overworld = *super::builtin_noise_generator_settings("overworld").unwrap();
        let snapshot = super::blended_noise_snapshot(
            super::random_state_terrain_random(12345, overworld),
            0.25,
            0.125,
            80.0,
            160.0,
            8.0,
        )
        .unwrap();
        assert_eq!(snapshot.min_limit_noise.levels.len(), 16);
        assert_eq!(snapshot.max_limit_noise.levels.len(), 16);
        assert_eq!(snapshot.main_noise.levels.len(), 8);
        assert!(snapshot.min_limit_noise.levels.iter().all(Option::is_some));
        assert!(snapshot.max_limit_noise.levels.iter().all(Option::is_some));
        assert!(snapshot.main_noise.levels.iter().all(Option::is_some));
        assert!((snapshot.max_value - super::blended_noise_max_value(0.125)).abs() < 1e-12);
        assert_eq!(
            super::BASE_3D_NOISE_OVERWORLD_DENSITY.value_bounds(),
            (-snapshot.max_value, snapshot.max_value)
        );

        let sample = super::blended_noise_sample(&snapshot, 16.0, 64.0, -32.0);
        let density_sample = super::BASE_3D_NOISE_OVERWORLD_DENSITY
            .compute_with_noise(12345, overworld, 16, 64, -32);
        assert!((sample - -0.12125330841368123).abs() < 1e-12);
        assert!((sample - density_sample).abs() < 1e-12);

        let nether = *super::builtin_noise_generator_settings("nether").unwrap();
        let nether_sample =
            super::BASE_3D_NOISE_NETHER_DENSITY.compute_with_noise(12345, nether, 16, 64, -32);
        assert!((nether_sample - 0.28724193742768880).abs() < 1e-12);
    }

    #[test]
    fn end_island_density_uses_seeded_simplex_height_scan() {
        let mut random = super::RandomSourceKind::Legacy(super::LegacyRandom::new(12345));
        random.consume_count(17_292);
        let simplex = super::simplex_noise_snapshot(&mut random);
        assert!((simplex.xo - 217.28203870227557).abs() < 1e-12);
        assert!((simplex.yo - 16.202358521842683).abs() < 1e-12);
        assert!((simplex.zo - 80.38840973560625).abs() < 1e-12);
        assert_eq!(
            &simplex.permutation[0..8],
            &[133, 54, 101, 16, 13, 4, 149, 66]
        );

        let simplex_value = super::simplex_noise_sample_2d(&simplex, 65.0, -71.0);
        let height = super::end_island_height_value(&simplex, 16 / 8, -32 / 8);
        let density = super::end_island_density_sample(12345, 16, -32);
        let end = *super::builtin_noise_generator_settings("end").unwrap();
        let density_function_value =
            super::END_ISLANDS_DENSITY.compute_with_noise(12345, end, 16, 64, -32);
        assert!((simplex_value - -0.40209723583721246).abs() < 1e-12);
        assert!((height - 64.222916).abs() < 1e-5);
        assert!((density - 0.43924152851104736).abs() < 1e-12);
        assert!((density - density_function_value).abs() < 1e-12);
    }

    #[test]
    fn simplex_noise_2d_sampling_covers_both_skew_branches() {
        let mut random = super::RandomSourceKind::Legacy(super::LegacyRandom::new(12345));
        let simplex = super::simplex_noise_snapshot(&mut random);
        let lower_triangle = super::simplex_noise_sample_2d(&simplex, 1.25, -3.5);
        let upper_triangle = super::simplex_noise_sample_2d(&simplex, -3.5, 1.25);
        assert!((lower_triangle - -0.18271295126292367).abs() < 1e-12);
        assert!((upper_triangle - -0.45718095628667355).abs() < 1e-12);
    }

    #[test]
    fn simplex_noise_3d_sampling_matches_vanilla_corner_path() {
        let mut random = super::RandomSourceKind::Legacy(super::LegacyRandom::new(12345));
        let simplex = super::simplex_noise_snapshot(&mut random);
        let first = super::simplex_noise_sample_3d(&simplex, 1.25, -3.5, 8.75);
        let second = super::simplex_noise_sample_3d(&simplex, -12.125, 0.5, 33.25);
        assert!((first - 0.124169920267489).abs() < 1e-12);
        assert!((second - 0.29129930814264227).abs() < 1e-12);
    }

    #[test]
    fn simplex_noise_3d_sampling_covers_all_rank_order_corner_paths() {
        let mut random = super::RandomSourceKind::Legacy(super::LegacyRandom::new(12345));
        let simplex = super::simplex_noise_snapshot(&mut random);
        let samples = [
            ((-5.0, -4.5, -2.75), -0.2787272376543217),
            ((-5.0, -4.75, -2.5), 0.16925000482253008),
            ((-5.0, -4.5, -3.75), -0.17981134259259166),
            ((-5.0, -4.75, -4.5), -0.05465644531249998),
            ((-5.0, -4.5, -4.75), 0.1639693359374999),
            ((-5.0, -4.75, -4.25), 0.07816971450617194),
        ];
        for ((x, y, z), expected) in samples {
            let actual = super::simplex_noise_sample_3d(&simplex, x, y, z);
            assert!(
                (actual - expected).abs() < 1e-12,
                "simplex sample at ({x}, {y}, {z})"
            );
        }
    }

    #[test]
    fn terrain_spline_holders_evaluate_vanilla_terrain_provider_shapes() {
        let context = super::TerrainSplineContext {
            continents: 0.2,
            erosion: -0.45,
            weirdness: 0.35,
            ridges: super::peaks_and_valleys(0.35),
        };
        let offset = super::terrain_spline(super::TerrainSplineKind::OverworldOffset);
        let factor = super::terrain_spline(super::TerrainSplineKind::OverworldFactor);
        let jaggedness = super::terrain_spline(super::TerrainSplineKind::OverworldJaggedness);
        let amplified_offset =
            super::terrain_spline(super::TerrainSplineKind::OverworldAmplifiedOffset);
        let amplified_factor =
            super::terrain_spline(super::TerrainSplineKind::OverworldAmplifiedFactor);
        let amplified_jaggedness =
            super::terrain_spline(super::TerrainSplineKind::OverworldAmplifiedJaggedness);

        assert!((offset.apply(context) - 0.3207085726435902).abs() < 1e-12);
        assert!((factor.apply(context) - 3.1937037037037035).abs() < 1e-12);
        assert!((jaggedness.apply(context) - 0.0).abs() < 1e-12);
        assert!((amplified_offset.apply(context) - 0.6257336663769374).abs() < 1e-12);
        assert!((amplified_factor.apply(context) - 0.47917681702730064).abs() < 1e-12);
        assert!((amplified_jaggedness.apply(context) - 0.0).abs() < 1e-12);

        for spline in [
            offset,
            factor,
            jaggedness,
            amplified_offset,
            amplified_factor,
            amplified_jaggedness,
        ] {
            let bounds = spline.bounds();
            assert!(bounds.0.is_finite());
            assert!(bounds.1.is_finite());
            assert!(bounds.0 <= bounds.1);
        }
    }

    fn extracted_density_function_ids_from_json_tree() -> Vec<String> {
        fn visit_density_function_jsons(root: &Path, dir: &Path, ids: &mut Vec<String>) {
            let entries = std::fs::read_dir(dir)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
            for entry in entries {
                let entry = entry.unwrap_or_else(|err| {
                    panic!("failed to read entry under {}: {err}", dir.display())
                });
                let path = entry.path();
                if path.is_dir() {
                    visit_density_function_jsons(root, &path, ids);
                    continue;
                }
                if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                    continue;
                }
                let relative = path
                    .strip_prefix(root)
                    .unwrap_or_else(|err| panic!("failed to relativize {}: {err}", path.display()))
                    .with_extension("");
                let key = relative
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                ids.push(format!("minecraft:{key}"));
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../decompiled-server-26.1.2/data/minecraft/worldgen/density_function");
        let mut ids = Vec::new();
        visit_density_function_jsons(&root, &root, &mut ids);
        ids.sort_unstable();
        ids
    }

    #[test]
    fn noise_router_density_function_bootstrap_keys_match_vanilla_prefix() {
        let mut extracted_density_function_ids = extracted_density_function_ids_from_json_tree();
        extracted_density_function_ids.push("minecraft:overworld/final_density".to_string());
        extracted_density_function_ids.sort_unstable();
        let mut actual_density_function_ids = BUILTIN_DENSITY_FUNCTIONS
            .iter()
            .map(|entry| entry.id.to_string())
            .collect::<Vec<_>>();
        actual_density_function_ids.sort_unstable();
        assert_eq!(actual_density_function_ids, extracted_density_function_ids);
        assert_eq!(
            BUILTIN_DENSITY_FUNCTIONS
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:zero",
                "minecraft:y",
                "minecraft:shift_x",
                "minecraft:shift_z",
                "minecraft:overworld/base_3d_noise",
                "minecraft:nether/base_3d_noise",
                "minecraft:end/base_3d_noise",
                "minecraft:overworld/continents",
                "minecraft:overworld/erosion",
                "minecraft:overworld/ridges",
                "minecraft:overworld/ridges_folded",
                "minecraft:overworld/offset",
                "minecraft:overworld/factor",
                "minecraft:overworld/jaggedness",
                "minecraft:overworld/depth",
                "minecraft:overworld/sloped_cheese",
                "minecraft:overworld_large_biomes/continents",
                "minecraft:overworld_large_biomes/erosion",
                "minecraft:overworld_large_biomes/offset",
                "minecraft:overworld_large_biomes/factor",
                "minecraft:overworld_large_biomes/jaggedness",
                "minecraft:overworld_large_biomes/depth",
                "minecraft:overworld_large_biomes/sloped_cheese",
                "minecraft:overworld_amplified/offset",
                "minecraft:overworld_amplified/factor",
                "minecraft:overworld_amplified/jaggedness",
                "minecraft:overworld_amplified/depth",
                "minecraft:overworld_amplified/sloped_cheese",
                "minecraft:end/sloped_cheese",
                "minecraft:overworld/caves/spaghetti_2d_thickness_modulator",
                "minecraft:overworld/caves/spaghetti_roughness_function",
                "minecraft:overworld/caves/pillars",
                "minecraft:overworld/caves/spaghetti_2d",
                "minecraft:overworld/caves/noodle",
                "minecraft:overworld/caves/entrances",
                "minecraft:overworld/final_density",
            ]
        );
        assert_eq!(
            builtin_density_function("overworld/base_3d_noise")
                .unwrap()
                .function
                .type_name(),
            "old_blended_noise"
        );
        assert_eq!(
            builtin_density_function("shift_x")
                .unwrap()
                .function
                .type_name(),
            DensityMarker::FlatCache.serialized_name()
        );
        let ridges_folded = builtin_density_function("overworld/ridges_folded")
            .unwrap()
            .function;
        assert_eq!(ridges_folded.type_name(), "mul");
        assert_eq!(ridges_folded.value_bounds(), (-14.14285714285714, 1.0));
        let offset = builtin_density_function("overworld/offset")
            .unwrap()
            .function;
        assert_eq!(
            offset.type_name(),
            DensityMarker::FlatCache.serialized_name()
        );
        assert_eq!(offset, super::OVERWORLD_OFFSET_DENSITY);
        assert_eq!(
            super::OVERWORLD_OFFSET_BLENDED_DENSITY,
            DensityFunction::Binary {
                kind: BinaryDensityFunction::Add,
                argument1: &super::OVERWORLD_OFFSET_BLEND_TARGET_DENSITY,
                argument2: &super::OVERWORLD_OFFSET_SPLINE_WEIGHTED_DENSITY,
            }
        );
        assert_eq!(
            super::OVERWORLD_OFFSET_BLEND_TARGET_DENSITY,
            DensityFunction::Binary {
                kind: BinaryDensityFunction::Mul,
                argument1: &super::BLEND_OFFSET_DENSITY,
                argument2: &super::BLEND_ALPHA_INVERSE_DENSITY,
            }
        );
        assert_eq!(
            super::OVERWORLD_OFFSET_SPLINE_WEIGHTED_DENSITY,
            DensityFunction::Binary {
                kind: BinaryDensityFunction::Mul,
                argument1: &super::OVERWORLD_OFFSET_SPLINE_WITH_OFFSET_DENSITY,
                argument2: &super::BLEND_ALPHA_CACHE_ONCE_DENSITY,
            }
        );
        assert_eq!(
            offset.value_bounds(),
            (-1.3037500262260437, 1.5722867486489562)
        );
        assert_eq!(
            builtin_density_function("overworld/factor")
                .unwrap()
                .function,
            super::OVERWORLD_FACTOR_DENSITY
        );
        assert_eq!(
            super::OVERWORLD_FACTOR_BLENDED_DENSITY,
            DensityFunction::Binary {
                kind: BinaryDensityFunction::Add,
                argument1: &super::BLENDING_FACTOR_DENSITY,
                argument2: &super::OVERWORLD_FACTOR_SPLINE_WEIGHTED_DENSITY,
            }
        );
        assert_eq!(
            super::OVERWORLD_FACTOR_SPLINE_WEIGHTED_DENSITY,
            DensityFunction::Binary {
                kind: BinaryDensityFunction::Mul,
                argument1: &super::BLEND_ALPHA_DENSITY,
                argument2: &super::OVERWORLD_FACTOR_SPLINE_DELTA_DENSITY,
            }
        );
        assert_eq!(
            builtin_density_function("overworld/jaggedness")
                .unwrap()
                .function,
            super::OVERWORLD_JAGGEDNESS_DENSITY
        );
        assert_eq!(
            super::OVERWORLD_JAGGEDNESS_BLENDED_DENSITY,
            DensityFunction::Binary {
                kind: BinaryDensityFunction::Add,
                argument1: &super::BLENDING_JAGGEDNESS_DENSITY,
                argument2: &super::OVERWORLD_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY,
            }
        );
        for id in [
            "overworld/factor",
            "overworld/jaggedness",
            "overworld_large_biomes/factor",
            "overworld_large_biomes/jaggedness",
            "overworld_amplified/factor",
            "overworld_amplified/jaggedness",
        ] {
            let entry = builtin_density_function(id).unwrap().function;
            assert_eq!(
                entry.type_name(),
                DensityMarker::FlatCache.serialized_name()
            );
            let bounds = entry.value_bounds();
            assert!(bounds.0.is_finite(), "{id} min bound should be finite");
            assert!(bounds.1.is_finite(), "{id} max bound should be finite");
            assert!(bounds.0 <= bounds.1, "{id} bounds should be ordered");
        }
        let depth = builtin_density_function("overworld/depth")
            .unwrap()
            .function;
        assert_eq!(depth.type_name(), "add");
        let depth_bounds = depth.value_bounds();
        assert!(depth_bounds.0.is_finite());
        assert!(depth_bounds.1.is_finite());
        assert_eq!(
            builtin_density_function("overworld_large_biomes/depth")
                .unwrap()
                .function
                .type_name(),
            "add"
        );
        assert_eq!(
            builtin_density_function("overworld_amplified/depth")
                .unwrap()
                .function
                .type_name(),
            "add"
        );
        for id in [
            "overworld/sloped_cheese",
            "overworld_large_biomes/sloped_cheese",
            "overworld_amplified/sloped_cheese",
        ] {
            let entry = builtin_density_function(id).unwrap().function;
            assert_eq!(entry.type_name(), "add");
            let bounds = entry.value_bounds();
            assert!(bounds.0.is_finite(), "{id} min bound should be finite");
            assert!(bounds.1.is_finite(), "{id} max bound should be finite");
        }
        assert_eq!(
            builtin_density_function("overworld/caves/spaghetti_2d_thickness_modulator")
                .unwrap()
                .function
                .type_name(),
            "cache_once"
        );
        let roughness = builtin_density_function("overworld/caves/spaghetti_roughness_function")
            .unwrap()
            .function;
        assert_eq!(roughness.type_name(), "cache_once");
        assert_eq!(
            roughness.value_bounds(),
            (-0.6355555555555555, 0.34222222222222215)
        );
        let pillars = builtin_density_function("overworld/caves/pillars")
            .unwrap()
            .function;
        assert_eq!(pillars.type_name(), "cache_once");
        assert_eq!(
            pillars.value_bounds(),
            (-179.00238323045266, 151.9263924897119)
        );
        let spaghetti_2d = builtin_density_function("overworld/caves/spaghetti_2d")
            .unwrap()
            .function;
        assert_eq!(spaghetti_2d.type_name(), "clamp");
        assert_eq!(spaghetti_2d.value_bounds(), (-1.0, 1.0));
        let noodle = builtin_density_function("overworld/caves/noodle")
            .unwrap()
            .function;
        assert_eq!(noodle.type_name(), "range_choice");
        assert_eq!(noodle.value_bounds(), (-0.15833333333333333, 64.0));
        let entrances = builtin_density_function("overworld/caves/entrances")
            .unwrap()
            .function;
        assert_eq!(entrances.type_name(), "cache_once");
        assert_eq!(
            entrances.value_bounds(),
            (-2.201428571428571, 1.3422222222222222)
        );
    }

    #[test]
    fn noise_router_record_shape_and_presets_match_noise_router_data() {
        assert_eq!(
            BUILTIN_NOISE_ROUTERS
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:caves",
                "minecraft:floating_islands",
                "minecraft:none",
            ]
        );

        let overworld = builtin_noise_router("overworld").unwrap().router;
        assert_eq!(
            overworld.field_type_names(),
            [
                "noise",            // barrier
                "noise",            // fluid_level_floodedness
                "noise",            // fluid_level_spread
                "noise",            // lava
                "shifted_noise",    // temperature
                "shifted_noise",    // vegetation
                "reference",        // continents
                "reference",        // erosion
                "reference",        // depth
                "reference",        // ridges
                "find_top_surface", // preliminary_surface_level (inline tree, not a registry reference)
                "reference",        // final_density
                "interpolated",     // vein_toggle (yLimitedInterpolatable wraps rangeChoice)
                "add",              // vein_ridged (add(-0.08f, max(veinA, veinB)))
                "noise",            // vein_gap
            ]
        );
        assert_eq!(
            overworld.barrier,
            DensityFunction::Noise {
                noise: "minecraft:aquifer_barrier",
                xz_scale: 1.0,
                y_scale: 0.5,
            }
        );
        assert_eq!(
            overworld.final_density,
            DensityFunction::Reference("minecraft:overworld/final_density")
        );

        let large = builtin_noise_router("large_biomes").unwrap().router;
        assert_eq!(
            large.temperature,
            DensityFunction::ShiftedNoise {
                shift_x: &super::SHIFT_X_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::SHIFT_Z_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:temperature_large",
            }
        );
        assert_eq!(
            large.continents,
            DensityFunction::Reference("minecraft:overworld_large_biomes/continents")
        );

        let nether = builtin_noise_router("nether").unwrap().router;
        assert_eq!(
            nether.temperature,
            DensityFunction::ShiftedNoise {
                shift_x: &super::ZERO_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::ZERO_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:nether/temperature",
            }
        );
        assert_eq!(
            nether.vegetation,
            DensityFunction::ShiftedNoise {
                shift_x: &super::ZERO_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::ZERO_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:nether/vegetation",
            }
        );
        assert_eq!(
            builtin_noise_router("end")
                .unwrap()
                .router
                .erosion
                .type_name(),
            "cache_2d"
        );
        assert_eq!(
            builtin_noise_router("none").unwrap().router.final_density,
            DensityFunction::Constant(0.0)
        );
    }

    #[test]
    fn surface_rule_codecs_and_presets_match_surface_rule_data() {
        assert_eq!(
            SURFACE_RULE_TYPES
                .iter()
                .map(|kind| kind.id)
                .collect::<Vec<_>>(),
            vec!["bandlands", "block", "sequence", "condition"]
        );
        assert_eq!(
            SURFACE_CONDITION_TYPES
                .iter()
                .map(|kind| kind.id)
                .collect::<Vec<_>>(),
            vec![
                "biome",
                "noise_threshold",
                "vertical_gradient",
                "y_above",
                "water",
                "stone_depth",
                "not",
                "steep",
                "hole",
                "above_preliminary_surface",
                "temperature",
            ]
        );

        assert_eq!(
            BUILTIN_SURFACE_RULE_PRESETS
                .iter()
                .map(|preset| preset.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:caves",
                "minecraft:floating_islands",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:air",
            ]
        );

        let overworld = super::builtin_surface_rule_preset("overworld").unwrap();
        assert_eq!(
            overworld.rule,
            SurfaceRuleKind::OverworldLike {
                preliminary_surface_check: true,
                bedrock_roof: false,
                bedrock_floor: true,
                deepslate: true,
            }
        );
        assert!(overworld.blocks.contains(&"minecraft:grass_block"));
        assert!(overworld.blocks.contains(&"minecraft:deepslate"));
        assert!(overworld.blocks.contains(&"minecraft:powder_snow"));
        assert!(overworld.conditions.contains(&"above_preliminary_surface"));
        assert!(overworld.conditions.contains(&"temperature"));

        let caves = super::builtin_surface_rule_preset("caves").unwrap();
        assert_eq!(
            caves.rule,
            SurfaceRuleKind::OverworldLike {
                preliminary_surface_check: false,
                bedrock_roof: true,
                bedrock_floor: true,
                deepslate: true,
            }
        );
        let floating = super::builtin_surface_rule_preset("floating_islands").unwrap();
        assert_eq!(
            floating.rule,
            SurfaceRuleKind::OverworldLike {
                preliminary_surface_check: false,
                bedrock_roof: false,
                bedrock_floor: false,
                deepslate: true,
            }
        );

        let nether = super::builtin_surface_rule_preset("nether").unwrap();
        assert_eq!(nether.rule, SurfaceRuleKind::Nether);
        assert!(nether.blocks.contains(&"minecraft:netherrack"));
        assert!(nether.blocks.contains(&"minecraft:warped_nylium"));
        assert!(nether.blocks.contains(&"minecraft:crimson_nylium"));
        assert!(nether
            .conditions
            .contains(&"bedrock_roof_vertical_gradient"));
        assert!(nether.conditions.contains(&"noise_threshold"));

        assert_eq!(
            super::builtin_surface_rule_preset("end").unwrap().rule,
            SurfaceRuleKind::State("minecraft:end_stone")
        );
        assert_eq!(
            super::builtin_surface_rule_preset("air").unwrap().rule,
            SurfaceRuleKind::State("minecraft:air")
        );
    }

    /// Parity test: after applying the overworld surface rule, any land column (top solid
    /// block above sea level) should have grass on top, dirt directly below, and stone
    /// several blocks further down.  Because chunk (0,0) at seed 0 may be partially or
    /// fully underwater, we scan all 256 columns and pick the first one that is land.
    ///
    /// Verifies items 155-158 of CHECKLIST_WORLDGEN.md.
    #[test]
    fn overworld_surface_rules_place_grass_dirt_stone_in_plains_column() {
        use super::{
            builtin_noise_generator_settings, builtin_noise_router, fill_noise_and_build_surface,
            load_surface_rule, noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
        };
        use crate::biome::BiomeSourceModel;

        let settings = builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings must exist");
        let router_id = noise_router_id_for_settings(*settings);
        let noise_router = builtin_noise_router(router_id)
            .map(|e| e.router)
            .unwrap_or(NONE_NOISE_ROUTER);
        let rule = load_surface_rule("minecraft:overworld")
            .expect("overworld surface rule must load from JSON");

        let sea_level = settings.sea_level;
        let min_y = settings.noise.min_y;
        let max_y = min_y + settings.noise.height - 1;

        // Try a range of nearby chunks until we find one with at least one land column.
        // Seed 0 overworld terrain around the origin includes both ocean and land chunks.
        let chunk = 'found: {
            for cz in 0..4_i32 {
                for cx in 0..4_i32 {
                    let c = fill_noise_and_build_surface(
                        ChunkPos { x: cx, z: cz },
                        &BiomeSourceModel::Fixed {
                            biome: "minecraft:plains",
                        },
                        settings,
                        0,
                        noise_router,
                        &rule,
                    );
                    // Check if any column in this chunk has its solid surface above sea level.
                    let has_land = (0..16_i32).any(|lz| {
                        (0..16_i32).any(|lx| {
                            let bx = cx * 16 + lx;
                            let bz = cz * 16 + lz;
                            (min_y..=max_y).rev().any(|y| {
                                match c.get_block_state(bx, y, bz).as_deref() {
                                    Some(b)
                                        if b != "minecraft:air"
                                            && b != "minecraft:cave_air"
                                            && b != "minecraft:void_air"
                                            && b != "minecraft:water"
                                            && b != "minecraft:lava" =>
                                    {
                                        y > sea_level
                                    }
                                    _ => false,
                                }
                            })
                        })
                    });
                    if has_land {
                        break 'found c;
                    }
                }
            }
            panic!("no land column found in any of the 16 chunks near the origin — terrain generation may be broken");
        };

        // Find the first land column in the chunk (top solid block above sea level).
        let land_column = (0..16_i32)
            .flat_map(|lz| (0..16_i32).map(move |lx| (lx, lz)))
            .find_map(|(lx, lz)| {
                let bx = chunk.pos.x * 16 + lx;
                let bz = chunk.pos.z * 16 + lz;
                let top = (min_y..=max_y).rev().find_map(|y| {
                    match chunk.get_block_state(bx, y, bz).as_deref() {
                        Some(b)
                            if b != "minecraft:air"
                                && b != "minecraft:cave_air"
                                && b != "minecraft:void_air"
                                && b != "minecraft:water"
                                && b != "minecraft:lava" =>
                        {
                            if y > sea_level {
                                Some((bx, y, bz, b.to_string()))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                });
                top
            })
            .expect("a land column must exist in the chunk we selected");

        let (bx, top_y, bz, top_block_name) = land_column;

        // The surface block should be grass for the fixed-plains biome source.
        assert_eq!(
            top_block_name, "minecraft:grass_block",
            "top solid block at ({bx},{top_y},{bz}) should be grass_block (got {top_block_name})"
        );

        // The block directly below should be dirt.
        let below = chunk.get_block_state(bx, top_y - 1, bz).unwrap_or_default();
        assert_eq!(
            below,
            "minecraft:dirt",
            "block below grass at ({bx},{},{bz}) should be dirt (got {below})",
            top_y - 1
        );

        // The exact dirt-depth depends on vanilla surface noise, but the column
        // should transition back to stone below the generated soil layer.
        let has_stone_below_soil = ((min_y + 1)..=(top_y - 2)).rev().any(|y| {
            chunk
                .get_block_state(bx, y, bz)
                .is_some_and(|block| block == "minecraft:stone")
        });
        assert!(
            has_stone_below_soil,
            "column at ({bx},{bz}) should contain stone below the plains soil layer"
        );
    }

    #[test]
    fn material_rule_sources_evaluate_surface_conditions_in_vanilla_order() {
        assert!(super::surface_rule_type("block").is_some());
        assert!(super::surface_rule_type("sequence").is_some());
        assert!(super::surface_rule_type("condition").is_some());
        assert!(super::surface_condition_type("biome").is_some());
        assert!(super::surface_condition_type("noise_threshold").is_some());
        assert!(super::surface_condition_type("stone_depth").is_some());

        static PLAINS_OR_FOREST: SurfaceConditionSource =
            SurfaceConditionSource::Biome(&["minecraft:plains", "minecraft:forest"]);
        static DRY_NOISE: SurfaceConditionSource = SurfaceConditionSource::NoiseThreshold {
            min: -0.25,
            max: 0.25,
        };
        static NOT_DRY_NOISE: SurfaceConditionSource = SurfaceConditionSource::Not(&DRY_NOISE);
        static FLOOR: SurfaceConditionSource = SurfaceConditionSource::StoneDepth {
            offset: 0,
            add_surface_depth: true,
            secondary_depth_range: 0,
            surface: CaveSurface::Floor,
        };
        static GRASS: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:grass_block");
        static DIRT: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:dirt");
        static STONE: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:stone");
        static PLAINS_GRASS: SurfaceRuleSource = SurfaceRuleSource::Condition {
            condition: &PLAINS_OR_FOREST,
            rule: &GRASS,
        };
        static DRY_STONE: SurfaceRuleSource = SurfaceRuleSource::Condition {
            condition: &NOT_DRY_NOISE,
            rule: &STONE,
        };
        static FLOOR_DIRT: SurfaceRuleSource = SurfaceRuleSource::Condition {
            condition: &FLOOR,
            rule: &DIRT,
        };
        static RULES: &[SurfaceRuleSource] = &[PLAINS_GRASS, DRY_STONE, FLOOR_DIRT];
        static SEQUENCE: SurfaceRuleSource = SurfaceRuleSource::Sequence(RULES);

        let heights = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        let plains_surface = SurfaceMaterialContext {
            seed: 12345,
            random_algorithm: super::RandomAlgorithm::Xoroshiro,
            x: 12,
            y: 64,
            z: -4,
            biome: "minecraft:plains",
            stone_depth_above: 0,
            stone_depth_below: 3,
            surface_depth: 3,
            preliminary_surface_y: 62,
            water_height: 63,
            temperature: 0.8,
            noise: 0.0,
            steep: false,
            hole: false,
        };
        assert_eq!(
            super::surface_rule_apply(&SEQUENCE, &plains_surface, &heights),
            Some("minecraft:grass_block")
        );

        let noisy_desert = SurfaceMaterialContext {
            biome: "minecraft:desert",
            noise: 0.6,
            ..plains_surface
        };
        assert_eq!(
            super::surface_rule_apply(&SEQUENCE, &noisy_desert, &heights),
            Some("minecraft:stone")
        );

        let quiet_desert_floor = SurfaceMaterialContext {
            biome: "minecraft:desert",
            noise: 0.0,
            stone_depth_above: 2,
            ..plains_surface
        };
        assert_eq!(
            super::surface_rule_apply(&SEQUENCE, &quiet_desert_floor, &heights),
            Some("minecraft:dirt")
        );

        assert!(super::surface_condition_test(
            &SurfaceConditionSource::YAbove {
                anchor: VerticalAnchor::Absolute(59),
                surface_depth_multiplier: 1,
                add_stone_depth: true,
            },
            &quiet_desert_floor,
            &heights
        ));
        assert!(super::surface_condition_test(
            &SurfaceConditionSource::Water {
                offset: 2,
                surface_depth_multiplier: 0,
                add_stone_depth: false,
            },
            &SurfaceMaterialContext {
                y: 65,
                ..quiet_desert_floor
            },
            &heights
        ));
        assert!(!super::surface_condition_test(
            &SurfaceConditionSource::Water {
                offset: 2,
                surface_depth_multiplier: 0,
                add_stone_depth: false,
            },
            &quiet_desert_floor,
            &heights
        ));
        assert!(super::surface_condition_test(
            &SurfaceConditionSource::Water {
                offset: 0,
                surface_depth_multiplier: 0,
                add_stone_depth: false,
            },
            &SurfaceMaterialContext {
                water_height: i32::MIN,
                ..quiet_desert_floor
            },
            &heights
        ));
        assert!(super::surface_condition_test(
            &SurfaceConditionSource::Water {
                offset: 0,
                surface_depth_multiplier: 0,
                add_stone_depth: true,
            },
            &SurfaceMaterialContext {
                y: 61,
                water_height: 63,
                stone_depth_above: 2,
                ..quiet_desert_floor
            },
            &heights
        ));
        assert!(!super::surface_condition_test(
            &SurfaceConditionSource::Water {
                offset: 1,
                surface_depth_multiplier: 1,
                add_stone_depth: true,
            },
            &SurfaceMaterialContext {
                y: 63,
                water_height: 63,
                surface_depth: 2,
                stone_depth_above: 2,
                ..quiet_desert_floor
            },
            &heights
        ));
        assert!(super::surface_condition_test(
            &SurfaceConditionSource::StoneDepth {
                offset: 0,
                add_surface_depth: true,
                secondary_depth_range: 6,
                surface: CaveSurface::Ceiling,
            },
            &SurfaceMaterialContext {
                stone_depth_below: 7,
                noise: 0.0,
                ..quiet_desert_floor
            },
            &heights
        ));
        let dynamic_water_state = super::BuildSurfaceColumnState {
            seed: quiet_desert_floor.seed,
            algorithm: quiet_desert_floor.random_algorithm,
            heights,
            last_update_xz: 1,
            last_update_y: 1,
            condition_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            profile: None,
            block_x: quiet_desert_floor.x,
            block_z: quiet_desert_floor.z,
            surface_depth: 0,
            surface_secondary: quiet_desert_floor.noise,
            steep: quiet_desert_floor.steep,
            hole: quiet_desert_floor.hole,
            min_surface_level: quiet_desert_floor.preliminary_surface_y,
            block_y: 61,
            water_height: 63,
            stone_depth_above: 2,
            stone_depth_below: quiet_desert_floor.stone_depth_below,
            biome: quiet_desert_floor.biome,
            temperature: quiet_desert_floor.temperature,
            biome_needs_update: false,
        };
        assert!(super::dyn_surface_condition_test(
            &super::DynSurfaceCondition::Water {
                offset: 0,
                surface_depth_multiplier: 0,
                add_stone_depth: true,
            },
            &dynamic_water_state,
            BUILTIN_NOISE_GENERATOR_SETTINGS[0],
        ));
        assert!(!super::surface_condition_test(
            &SurfaceConditionSource::StoneDepth {
                offset: 0,
                add_surface_depth: false,
                secondary_depth_range: 0,
                surface: CaveSurface::Ceiling,
            },
            &SurfaceMaterialContext {
                stone_depth_below: 2,
                ..quiet_desert_floor
            },
            &heights
        ));
        let vertical_gradient = SurfaceConditionSource::VerticalGradient {
            random_name: "minecraft:bedrock_floor",
            true_at_and_below: VerticalAnchor::Absolute(60),
            false_at_and_above: VerticalAnchor::Absolute(70),
        };
        assert!(super::surface_condition_test(
            &vertical_gradient,
            &SurfaceMaterialContext {
                y: 60,
                noise: 1.0,
                ..quiet_desert_floor
            },
            &heights
        ));
        assert!(!super::surface_condition_test(
            &vertical_gradient,
            &SurfaceMaterialContext {
                y: 70,
                noise: -1.0,
                ..quiet_desert_floor
            },
            &heights
        ));
        let random_float = super::surface_positional_random_float(
            quiet_desert_floor.seed,
            quiet_desert_floor.random_algorithm,
            "minecraft:bedrock_floor",
            quiet_desert_floor.x,
            65,
            quiet_desert_floor.z,
        );
        assert!((random_float - 0.96467084).abs() < f32::EPSILON);
        assert!(!super::surface_condition_test(
            &vertical_gradient,
            &SurfaceMaterialContext {
                y: 65,
                noise: -1.0,
                ..quiet_desert_floor
            },
            &heights
        ));
        assert!(!super::dyn_surface_condition_test(
            &super::DynSurfaceCondition::VerticalGradient {
                random_name: "minecraft:bedrock_floor".to_string(),
                true_at_and_below: VerticalAnchor::Absolute(60),
                false_at_and_above: VerticalAnchor::Absolute(70),
            },
            &super::BuildSurfaceColumnState {
                block_y: 65,
                ..dynamic_water_state
            },
            BUILTIN_NOISE_GENERATOR_SETTINGS[0],
        ));
        assert_eq!(
            super::surface_rule_apply(&SurfaceRuleSource::Bandlands, &quiet_desert_floor, &heights),
            Some("minecraft:red_sand")
        );
    }

    #[test]
    fn biome_manager_seed_obfuscation_matches_java_hash_long() {
        // Java: BiomeManager.obfuscateSeed(seed) =
        // Hashing.sha256().hashLong(seed).asLong().
        assert_eq!(super::biome_manager_obfuscate_seed(0), 8794265229978523055);
        assert_eq!(
            super::biome_manager_obfuscate_seed(8_675_309),
            8580917108473614843
        );
        assert_eq!(super::biome_manager_obfuscate_seed(-1), 6759447113877070610);
    }

    #[test]
    fn biome_manager_fiddle_mask_matches_java_floor_mod() {
        for value in [
            0,
            1,
            -1,
            i64::MIN,
            i64::MAX,
            0x1234_5678_9abc_def0_i64,
            -0x1234_5678_9abc_def_i64,
            6_364_136_223_846_793_005_i64,
            -6_364_136_223_846_793_005_i64,
        ] {
            let java_floor_mod = ((value >> 24).rem_euclid(1024) as f64) / 1024.0;
            let expected = (java_floor_mod - 0.5) * 0.9;
            assert_eq!(super::biome_manager_fiddle(value), expected);
        }
    }

    #[test]
    fn aquifer_constants_and_disabled_behavior_match_decompiled_rules() {
        assert_eq!(
            AQUIFER_NOISE_SETTINGS,
            AquiferNoiseSettings {
                x_range: 10,
                y_range: 9,
                z_range: 10,
                x_separation: 6,
                y_separation: 3,
                z_separation: 6,
                x_spacing: 16,
                y_spacing: 12,
                z_spacing: 16,
                max_reasonable_distance_to_center: 11,
                sample_offset_x: -5,
                sample_offset_y: 1,
                sample_offset_z: -5,
            }
        );
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS.len(), 13);
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS[0], (0, 0));
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS[5], (-3, 0));

        let water = FluidStatus {
            fluid_level: 63,
            fluid_type: "minecraft:water",
        };
        assert_eq!(water.at(62), "minecraft:water");
        assert_eq!(water.at(63), "minecraft:air");
        assert_eq!(super::disabled_aquifer_substance(0.1, water, 62), None);
        assert_eq!(
            super::disabled_aquifer_substance(-0.1, water, 62),
            Some("minecraft:water")
        );
        assert_eq!(super::aquifer_similarity(100, 144), -0.76);
    }

    /// Parity test: global fluid picker returns lava below y = -54 and water below sea level.
    /// Also verifies that no water appears at y = -55 in a generated overworld chunk
    /// (those blocks must be lava or solid — the aquifer respects the global lava floor).
    ///
    /// Verifies items 143, 144, 146 of CHECKLIST_WORLDGEN.md.
    #[test]
    fn aquifer_global_fluid_picker_and_no_water_below_lava_floor() {
        use super::{
            builtin_noise_generator_settings, builtin_noise_router, fill_from_noise_chunk,
            global_fluid_status, noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
        };

        let settings = builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings must exist");

        // ── Unit-test the global fluid picker ────────────────────────────────
        // y = -55 < min(-54, seaLevel=63) = -54 → lava status.
        let lava_status = global_fluid_status(-55, settings.sea_level, settings.default_fluid);
        assert_eq!(
            lava_status.at(-55),
            "minecraft:lava",
            "y=-55 (below bedrock lava floor at -54) must be lava"
        );
        // at the fluid_level boundary (-54), the block is air (< relation is strict).
        assert_eq!(
            lava_status.at(-54),
            "minecraft:air",
            "y=-54 is the fluid_level itself, so at(-54) must be air"
        );

        // y = 0 → sea-level water status.
        let water_status = global_fluid_status(0, settings.sea_level, settings.default_fluid);
        assert_eq!(
            water_status.at(0),
            "minecraft:water",
            "y=0 is below sea level (63) so the global fluid is water"
        );
        assert_eq!(
            water_status.at(settings.sea_level),
            "minecraft:air",
            "y=sea_level is at the fluid_level boundary → air"
        );

        // ── Integration: no water at y = -55 in a real chunk ─────────────────
        let router_id = noise_router_id_for_settings(*settings);
        let noise_router = builtin_noise_router(router_id)
            .map(|e| e.router)
            .unwrap_or(NONE_NOISE_ROUTER);
        let chunk = fill_from_noise_chunk(ChunkPos { x: 0, z: 0 }, settings, 0, noise_router);

        // Scan all 256 columns at y=-55: any non-solid, non-air block must be lava, not water.
        for lx in 0..16_i32 {
            for lz in 0..16_i32 {
                if let Some(block) = chunk.get_block_state(lx, -55, lz) {
                    assert_ne!(
                        block, "minecraft:water",
                        "y=-55 at ({lx}, {lz}) must not be water (got {block}); \
                         the global lava floor takes priority"
                    );
                }
            }
        }
    }

    /// Parity test: with aquifers enabled, the overworld generates water somewhere
    /// underground between sea level and the lava floor, demonstrating that the
    /// NoiseBasedAquifer is active and produces fluid pockets.
    ///
    /// Verifies items 142, 145 of CHECKLIST_WORLDGEN.md.
    #[test]
    fn noise_based_aquifer_produces_underground_water_in_overworld() {
        use super::{
            builtin_noise_generator_settings, builtin_noise_router, fill_from_noise_chunk,
            noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
        };

        let settings = builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings must exist");
        assert!(
            settings.aquifers_enabled,
            "overworld must have aquifers enabled"
        );

        let router_id = noise_router_id_for_settings(*settings);
        let noise_router = builtin_noise_router(router_id)
            .map(|e| e.router)
            .unwrap_or(NONE_NOISE_ROUTER);

        // Try a few nearby chunks until we find one with underground water.
        let sea = settings.sea_level;
        let found_water = (0..4_i32)
            .flat_map(|cz| (0..4_i32).map(move |cx| (cx, cz)))
            .any(|(cx, cz)| {
                let chunk =
                    fill_from_noise_chunk(ChunkPos { x: cx, z: cz }, settings, 0, noise_router);
                let min_y = settings.noise.min_y;
                // Look for water between the lava floor (-54) and sea level in any column.
                (-54..sea).any(|y| {
                    (0..16_i32)
                        .flat_map(|lz| (0..16_i32).map(move |lx| (lx, lz)))
                        .any(|(lx, lz)| {
                            let bx = cx * 16 + lx;
                            let bz = cz * 16 + lz;
                            chunk.get_block_state(bx, y, bz).as_deref() == Some("minecraft:water")
                                || chunk.get_block_state(bx, y, bz).as_deref()
                                    == Some("minecraft:lava")
                        })
                })
            });
        assert!(
            found_water,
            "no underground fluid (water or lava) found in 16 chunks near origin — \
             NoiseBasedAquifer may not be running"
        );
    }

    #[test]
    fn cave_generation_families_cover_noise_router_data_cave_builders() {
        assert_eq!(
            CAVE_GENERATION_FAMILIES
                .iter()
                .map(|family| family.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld/caves/spaghetti_roughness_function",
                "minecraft:overworld/caves/entrances",
                "minecraft:overworld/caves/noodle",
                "minecraft:overworld/caves/pillars",
                "minecraft:overworld/caves/spaghetti_2d",
                "minecraft:overworld/caves/underground",
            ]
        );
        let entrances = super::cave_generation_family("overworld/caves/entrances").unwrap();
        assert!(entrances.noises.contains(&"minecraft:spaghetti_3d_1"));
        assert!(entrances.noises.contains(&"minecraft:cave_entrance"));
        assert_eq!(entrances.output, CaveDensityOutput::CacheOnce);

        let noodle = super::cave_generation_family("overworld/caves/noodle").unwrap();
        assert!(noodle.noises.contains(&"minecraft:noodle_ridge_a"));
        assert_eq!(noodle.output, CaveDensityOutput::RangeChoice);

        let spaghetti_2d = super::cave_generation_family("overworld/caves/spaghetti_2d").unwrap();
        assert!(spaghetti_2d
            .noises
            .contains(&"minecraft:spaghetti_2d_elevation"));
        assert_eq!(
            spaghetti_2d.output,
            CaveDensityOutput::Clamp { min: -1, max: 1 }
        );
    }

    #[test]
    fn ore_veinifier_constants_and_vein_types_match_decompiled_values() {
        assert_eq!(
            ORE_VEINIFIER_CONSTANTS,
            OreVeinifierConstants {
                veininess_threshold: 0.4,
                edge_roundoff_begin: 20,
                max_edge_roundoff: 0.2,
                vein_solidness: 0.7,
                min_richness: 0.1,
                max_richness: 0.3,
                max_richness_threshold: 0.6,
                chance_of_raw_ore_block: 0.02,
                skip_ore_if_gap_noise_is_below: -0.3,
            }
        );
        assert_eq!(ORE_VEIN_TYPES.len(), 2);
        assert_eq!(ORE_VEIN_TYPES[0].id, "copper");
        assert_eq!(ORE_VEIN_TYPES[0].ore, "minecraft:copper_ore");
        assert_eq!(
            ORE_VEIN_TYPES[0].raw_ore_block,
            "minecraft:raw_copper_block"
        );
        assert_eq!(ORE_VEIN_TYPES[0].filler, "minecraft:granite");
        assert_eq!((ORE_VEIN_TYPES[0].min_y, ORE_VEIN_TYPES[0].max_y), (0, 50));
        assert_eq!(ORE_VEIN_TYPES[1].id, "iron");
        assert_eq!(ORE_VEIN_TYPES[1].ore, "minecraft:deepslate_iron_ore");
        assert_eq!(ORE_VEIN_TYPES[1].raw_ore_block, "minecraft:raw_iron_block");
        assert_eq!(ORE_VEIN_TYPES[1].filler, "minecraft:tuff");
        assert_eq!(
            (ORE_VEIN_TYPES[1].min_y, ORE_VEIN_TYPES[1].max_y),
            (-60, -8)
        );
    }

    #[test]
    fn ore_veinifier_decision_matches_vanilla_branching() {
        let base = OreVeinDecisionInput {
            y: 25,
            vein_toggle: 0.61,
            vein_ridged: -0.1,
            vein_gap: 0.0,
            solidness_random: 0.5,
            richness_random: 0.2,
            raw_ore_random: 0.5,
            debug_ore_veins: false,
        };
        assert_eq!(super::ore_vein_richness(0.4), 0.1);
        assert_eq!(super::ore_vein_richness(0.6), 0.3);
        assert_eq!(super::ore_vein_decision(base), Some("minecraft:copper_ore"));
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                raw_ore_random: 0.01,
                ..base
            }),
            Some("minecraft:raw_copper_block")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                y: -30,
                vein_toggle: -0.61,
                raw_ore_random: 0.5,
                ..base
            }),
            Some("minecraft:deepslate_iron_ore")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                richness_random: 0.99,
                ..base
            }),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                vein_gap: -0.31,
                ..base
            }),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                solidness_random: 0.71,
                ..base
            }),
            None
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                y: 100,
                debug_ore_veins: true,
                ..base
            }),
            Some("minecraft:air")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                richness_random: 0.99,
                debug_ore_veins: true,
                ..base
            }),
            Some("minecraft:oak_button")
        );
    }

    #[test]
    fn ore_veinifier_uses_positional_random_factory_sequence() {
        let ore_factory = crate::random_source::random_state_seed_factories(
            8675309,
            super::RandomAlgorithm::Xoroshiro,
        )
        .ore;
        let mut positional_random = ore_factory.at(4, 25, -9);
        let manual = super::ore_vein_decision(OreVeinDecisionInput {
            y: 25,
            vein_toggle: 0.61,
            vein_ridged: -0.1,
            vein_gap: 0.0,
            solidness_random: f64::from(positional_random.next_f32()),
            richness_random: f64::from(positional_random.next_f32()),
            raw_ore_random: f64::from(positional_random.next_f32()),
            debug_ore_veins: false,
        });
        assert_eq!(
            super::ore_vein_decision_at(ore_factory, 4, 25, -9, 0.61, -0.1, 0.0, false),
            manual
        );

        let legacy_ore_factory = crate::random_source::random_state_seed_factories(
            8675309,
            super::RandomAlgorithm::Legacy,
        )
        .ore;
        assert_eq!(
            super::ore_vein_decision_at(legacy_ore_factory, 4, 25, -9, 0.61, -0.1, 0.0, false),
            None
        );
    }

    /// Verifies that ore veins are integrated into solid-block placement in
    /// `fill_from_noise_chunk`.  The OreVeinifier can place three block types per vein:
    ///
    /// - **Iron vein** (Y -60..=-8): deepslate_iron_ore (ore), raw_iron_block (2% raw),
    ///   tuff (filler — most common; appears in the halo around the ore core)
    /// - **Copper vein** (Y 0..=50): copper_ore (ore), raw_copper_block (2% raw),
    ///   granite (filler — same halo role)
    ///
    /// Regular iron_ore is never placed by veins: the iron VeinType explicitly uses
    /// deepslate_iron_ore because iron veins only spawn in the deepslate zone (Y ≤ -8).
    /// Verifies that ore veins are integrated into solid-block placement in
    /// `fill_from_noise_chunk`.  The OreVeinifier can place three block types per vein:
    ///
    /// - **Iron vein** (Y -60..=-8): deepslate_iron_ore (ore), raw_iron_block (2% raw),
    ///   tuff (filler — most common; appears in the halo around the ore core)
    /// - **Copper vein** (Y 0..=50): copper_ore (ore), raw_copper_block (2% raw),
    ///   granite (filler — same halo role)
    ///
    /// Regular iron_ore is never placed by veins: the iron VeinType explicitly uses
    /// deepslate_iron_ore because iron veins only spawn in the deepslate zone (Y ≤ -8).
    ///
    /// Strategy: the ore_veininess noise (scale 1.5) has a ~170-block wavelength, so
    /// the near-origin region may be entirely in a low-veininess trough at seed 0.
    /// We first scan the raw noise over a ±512-block grid (no chunk generation needed)
    /// to locate a world position guaranteed to have high vein activity, then generate
    /// exactly that one chunk and scan its sections efficiently (one decode per section).
    #[test]
    fn ore_veins_integrated_in_chunk_generation() {
        use super::{
            builtin_noise_generator_settings, builtin_noise_router, fill_from_noise_chunk,
            noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
            OVERWORLD_VEIN_TOGGLE_NOISE_DENSITY,
        };
        use crate::storage::chunk::{PalettedContainer, SECTION_VOLUME};
        use crate::storage::nbt::Tag;

        let settings = builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings must exist");
        assert!(
            settings.ore_veins_enabled,
            "overworld must have ore veins enabled"
        );

        let router_id = noise_router_id_for_settings(*settings);
        let noise_router = builtin_noise_router(router_id)
            .map(|e| e.router)
            .unwrap_or(NONE_NOISE_ROUTER);

        // Iron vein blocks (Y -60..=-8).
        const IRON_VEIN_BLOCKS: &[&str] = &[
            "minecraft:deepslate_iron_ore",
            "minecraft:raw_iron_block",
            "minecraft:tuff",
        ];
        // Copper vein blocks (Y 0..=50).
        const COPPER_VEIN_BLOCKS: &[&str] = &[
            "minecraft:copper_ore",
            "minecraft:raw_copper_block",
            "minecraft:granite",
        ];

        // Fast veininess probe: evaluate the raw (non-interpolated) noise without
        // generating any chunk.  Scans a ±512 block grid at step 8 to cover many noise
        // wavelengths (~170 blocks at scale 1.5).  Returns unique chunk positions where the
        // toggle noise satisfies `predicate` at the given probe Y.  probe_y is chosen deep
        // underground so blocks there are nearly always solid stone.
        //
        // Positive vein_toggle (> 0.4) → COPPER vein type, Y range 0..=50.
        // Negative vein_toggle (< -0.4) → IRON vein type, Y range -60..=-8.
        // These are the same noise; sign determines which type is active.
        fn find_vein_chunks(
            settings: &super::NoiseGeneratorSettings,
            probe_y: i32,
            predicate: impl Fn(f64) -> bool,
        ) -> Vec<(i32, i32)> {
            let mut results = Vec::new();
            for z in (-512_i32..512).step_by(8) {
                for x in (-512_i32..512).step_by(8) {
                    let vt = OVERWORLD_VEIN_TOGGLE_NOISE_DENSITY
                        .compute_with_noise(0, *settings, x, probe_y, z);
                    if predicate(vt) {
                        let cx = x >> 4;
                        let cz = z >> 4;
                        if !results.contains(&(cx, cz)) {
                            results.push((cx, cz));
                        }
                    }
                }
            }
            results
        }

        /// Decode each relevant section once and look for any `targets` block name.
        fn any_vein_block_in_range(
            chunk: &crate::storage::chunk::LevelChunk,
            y_min: i32,
            y_max: i32,
            targets: &[&str],
        ) -> bool {
            for section in &chunk.sections {
                let s_min = section.y as i32 * 16;
                let s_max = s_min + 15;
                if s_max < y_min || s_min > y_max {
                    continue;
                }
                let Ok(container) =
                    PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
                else {
                    continue;
                };
                let lo = (y_min - s_min).clamp(0, 15) as usize;
                let hi = (y_max - s_min).clamp(0, 15) as usize;
                for local_y in lo..=hi {
                    for local_z in 0..16_usize {
                        for local_x in 0..16_usize {
                            let idx = local_y * 256 + local_z * 16 + local_x;
                            if let Some(Tag::Compound(fields)) = container.get_entry(idx) {
                                if let Some((_, Tag::String(name))) =
                                    fields.iter().find(|(k, _)| k == "Name")
                                {
                                    if targets.contains(&name.as_str()) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            false
        }

        // --- Iron vein check ---
        // Probe at Y=-30: always solid stone/deepslate, dead centre of the iron vein Y range.
        // Iron is selected when vein_toggle < -0.4 (negative).
        let iron_chunks = find_vein_chunks(settings, -30, |vt| vt < -0.4);
        assert!(
            !iron_chunks.is_empty(),
            "ore_veininess noise never exceeded 0.4 at Y=-30 over a ±512 block grid at seed 0 \
             — noise setup may be broken"
        );
        // Take only the first 3 candidate chunks — vein zones span ~170 blocks so multiple
        // consecutive chunks are inside the same zone; 3 is enough to hit a ridgeline.
        let found_iron = iron_chunks.iter().take(3).any(|&(cx, cz)| {
            let chunk = fill_from_noise_chunk(ChunkPos { x: cx, z: cz }, settings, 0, noise_router);
            any_vein_block_in_range(&chunk, -60, -8, IRON_VEIN_BLOCKS)
        });
        assert!(
            found_iron,
            "no iron vein blocks (deepslate_iron_ore / raw_iron_block / tuff) found in \
             the first 3 vein-zone chunks at Y=-60..=-8 — ore vein integration broken"
        );

        // --- Copper vein check ---
        // Probe at Y=5: copper vein range is 0..=50; Y=5 is deep underground and nearly
        // always solid stone, unlike Y=25 which can be aquifer-filled ocean water.
        // Copper is selected when vein_toggle > 0.4 (positive).
        let copper_chunks = find_vein_chunks(settings, 5, |vt| vt > 0.4);
        assert!(
            !copper_chunks.is_empty(),
            "ore_veininess noise never exceeded 0.4 at Y=5 over a ±512 block grid at seed 0 \
             — noise setup may be broken"
        );
        let found_copper = copper_chunks.iter().take(3).any(|&(cx, cz)| {
            let chunk = fill_from_noise_chunk(ChunkPos { x: cx, z: cz }, settings, 0, noise_router);
            any_vein_block_in_range(&chunk, 0, 50, COPPER_VEIN_BLOCKS)
        });
        assert!(
            found_copper,
            "no copper vein blocks (copper_ore / raw_copper_block / granite) found in \
             the first 3 vein-zone chunks at Y=0..=50 — ore vein integration broken"
        );
    }

    #[test]
    fn configured_carvers_match_vanilla_bootstrap_entries() {
        assert_eq!(
            WORLD_CARVER_TYPES
                .iter()
                .map(|carver| carver.id())
                .collect::<Vec<_>>(),
            vec![
                "minecraft:cave",
                "minecraft:nether_cave",
                "minecraft:canyon"
            ]
        );
        assert_eq!(
            super::world_carver_type("cave"),
            Some(WorldCarverType::Cave)
        );
        assert_eq!(
            super::world_carver_type("minecraft:nether_cave"),
            Some(WorldCarverType::NetherCave)
        );

        assert_eq!(
            CONFIGURED_CARVERS
                .iter()
                .map(|carver| carver.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:cave",
                "minecraft:cave_extra_underground",
                "minecraft:canyon",
                "minecraft:nether_cave",
            ]
        );

        let cave = super::configured_carver("cave").unwrap();
        assert_eq!(cave.carver_type, WorldCarverType::Cave);
        assert_eq!(cave.probability, 0.15);
        assert!(super::carver_is_start_chunk(cave, 0.15));
        assert!(!super::carver_is_start_chunk(cave, 0.150_001));
        assert_eq!(
            cave.y,
            HeightRange {
                min: VerticalAnchor::AboveBottom(8),
                max: VerticalAnchor::Absolute(180),
            }
        );
        assert_eq!(cave.y_scale, FloatProvider::Uniform { min: 0.1, max: 0.9 });
        assert_eq!(cave.lava_level, VerticalAnchor::AboveBottom(8));
        assert_eq!(cave.debug.barrier_state, "minecraft:crimson_button");
        assert_eq!(
            cave.replaceable_tag,
            "#minecraft:overworld_carver_replaceables"
        );

        let extra = super::configured_carver("cave_extra_underground").unwrap();
        assert_eq!(extra.probability, 0.07);
        assert_eq!(extra.y.max, VerticalAnchor::Absolute(47));
        assert_eq!(extra.debug.barrier_state, "minecraft:oak_button");

        let canyon = super::configured_carver("canyon").unwrap();
        assert_eq!(canyon.carver_type, WorldCarverType::Canyon);
        assert_eq!(canyon.probability, 0.01);
        assert_eq!(canyon.y_scale, FloatProvider::Constant(3.0));
        assert_eq!(canyon.debug.barrier_state, "minecraft:warped_button");
        assert!(matches!(
            canyon.shape,
            CarverShape::Canyon {
                vertical_rotation: FloatProvider::Uniform {
                    min: -0.125,
                    max: 0.125
                },
                ..
            }
        ));

        let nether = super::configured_carver("nether_cave").unwrap();
        assert_eq!(nether.carver_type, WorldCarverType::NetherCave);
        assert_eq!(nether.probability, 0.2);
        assert_eq!(nether.y.min, VerticalAnchor::Absolute(0));
        assert_eq!(nether.y.max, VerticalAnchor::BelowTop(1));
        assert_eq!(super::carver_cave_bound(WorldCarverType::NetherCave), 10);
        assert_eq!(
            super::carver_tunnel_y_scale(WorldCarverType::NetherCave),
            5.0
        );
        assert_eq!(super::nether_carver_thickness(0.5, 0.25), 2.5);
        assert_eq!(
            nether.replaceable_tag,
            "#minecraft:nether_carver_replaceables"
        );
        assert!(matches!(
            nether.shape,
            CarverShape::Cave {
                floor_level: FloatProvider::Constant(-0.7),
                ..
            }
        ));
    }

    #[test]
    fn configured_carver_json_codec_matches_vanilla_registry_files() {
        let dir = "../decompiled-server-26.1.2/data/minecraft/worldgen/configured_carver";
        for id in [
            "minecraft:cave",
            "minecraft:cave_extra_underground",
            "minecraft:canyon",
            "minecraft:nether_cave",
        ] {
            let path = format!("{}/{}.json", dir, id.strip_prefix("minecraft:").unwrap());
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
            let json: serde_json::Value = serde_json::from_str(&raw)
                .unwrap_or_else(|err| panic!("failed to parse {path}: {err}"));
            let parsed = super::parse_configured_carver_from_json(id, &json)
                .unwrap_or_else(|err| panic!("failed to decode {path}: {err}"));
            assert_eq!(
                parsed,
                *super::configured_carver(id).unwrap(),
                "{id} JSON codec output must match the builtin configured carver"
            );
        }
    }

    #[test]
    fn world_carver_can_reach_matches_vanilla_distance_gate() {
        assert!(super::carver_can_reach(8.0, 8.0, 8.0, 8.0, 0, 10, 1.0));
        assert!(super::carver_can_reach(8.0, 8.0, 30.0, 8.0, 0, 10, 4.0));
        assert!(!super::carver_can_reach(8.0, 8.0, 80.0, 8.0, 9, 10, 1.0));
        assert_eq!(super::carver_mask_index(17, -60, 31, -64), Some(1_265));
        assert_eq!(
            super::carver_mask_position(1_265, 16, 16, -64),
            BlockPos {
                x: 17,
                y: -60,
                z: 31,
            }
        );

        let cave = super::configured_carver("cave").unwrap();
        let nether = super::configured_carver("nether_cave").unwrap();
        let height_context = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        let nether_height_context = WorldGenerationHeightContext {
            min_y: 0,
            height: 128,
        };
        assert!(super::carver_can_replace_block(cave, "minecraft:stone"));
        assert!(!super::carver_can_replace_block(cave, "minecraft:bedrock"));
        assert_eq!(super::carver_effective_lava_y(cave, height_context), -56);
        assert_eq!(
            super::carver_effective_lava_y(nether, nether_height_context),
            31
        );
        assert_eq!(
            super::carver_carve_block(
                cave,
                height_context,
                super::CarverBlockInput {
                    pos: BlockPos { x: 1, y: -57, z: 2 },
                    block: "minecraft:stone",
                    was_masked: false,
                    aquifer_state: Some("minecraft:air"),
                    should_schedule_fluid_update: false,
                    debug_enabled: false,
                },
            )
            .unwrap()
            .state,
            "minecraft:lava"
        );
        assert_eq!(
            super::carver_carve_block(
                cave,
                height_context,
                super::CarverBlockInput {
                    pos: BlockPos { x: 1, y: 60, z: 2 },
                    block: "minecraft:stone",
                    was_masked: false,
                    aquifer_state: Some("minecraft:water"),
                    should_schedule_fluid_update: true,
                    debug_enabled: false,
                },
            )
            .unwrap(),
            super::CarverBlockOutcome {
                pos: BlockPos { x: 1, y: 60, z: 2 },
                state: "minecraft:water",
                mask_index: 31_777,
                mark_postprocessing: true,
            }
        );
        assert!(super::carver_carve_block(
            cave,
            height_context,
            super::CarverBlockInput {
                pos: BlockPos { x: 1, y: 60, z: 2 },
                block: "minecraft:bedrock",
                was_masked: false,
                aquifer_state: Some("minecraft:air"),
                should_schedule_fluid_update: false,
                debug_enabled: false,
            },
        )
        .is_none());
        assert_eq!(
            super::carver_carve_block(
                cave,
                height_context,
                super::CarverBlockInput {
                    pos: BlockPos { x: 1, y: 60, z: 2 },
                    block: "minecraft:bedrock",
                    was_masked: true,
                    aquifer_state: None,
                    should_schedule_fluid_update: false,
                    debug_enabled: true,
                },
            )
            .unwrap()
            .state,
            "minecraft:crimson_button"
        );
        assert_eq!(
            super::carver_carve_block(
                nether,
                nether_height_context,
                super::CarverBlockInput {
                    pos: BlockPos { x: 1, y: 31, z: 2 },
                    block: "minecraft:netherrack",
                    was_masked: false,
                    aquifer_state: Some("minecraft:air"),
                    should_schedule_fluid_update: true,
                    debug_enabled: false,
                },
            )
            .unwrap(),
            super::CarverBlockOutcome {
                pos: BlockPos { x: 1, y: 31, z: 2 },
                state: "minecraft:lava",
                mask_index: 7_969,
                mark_postprocessing: false,
            }
        );
        assert_eq!(
            super::carver_carve_block(
                nether,
                nether_height_context,
                super::CarverBlockInput {
                    pos: BlockPos { x: 1, y: 32, z: 2 },
                    block: "minecraft:netherrack",
                    was_masked: false,
                    aquifer_state: Some("minecraft:water"),
                    should_schedule_fluid_update: true,
                    debug_enabled: false,
                },
            )
            .unwrap()
            .state,
            "minecraft:cave_air"
        );
        let ellipsoid = super::carver_ellipsoid_candidate_positions(
            0,
            0,
            height_context,
            false,
            8.0,
            64.0,
            8.0,
            2.0,
            2.0,
            &[],
            false,
            super::CarverSkipModel::Cave { floor_level: -0.7 },
        );
        assert!(ellipsoid.contains(&BlockPos { x: 8, y: 65, z: 8 }));
        assert!(!ellipsoid.contains(&BlockPos { x: 8, y: 62, z: 8 }));
        let masked_index = super::carver_mask_index(8, 65, 8, -64).unwrap();
        assert!(!super::carver_ellipsoid_candidate_positions(
            0,
            0,
            height_context,
            false,
            8.0,
            64.0,
            8.0,
            2.0,
            2.0,
            &[masked_index],
            false,
            super::CarverSkipModel::Cave { floor_level: -0.7 },
        )
        .contains(&BlockPos { x: 8, y: 65, z: 8 }));
        assert!(super::carver_ellipsoid_candidate_positions(
            0,
            0,
            height_context,
            false,
            8.0,
            64.0,
            8.0,
            2.0,
            2.0,
            &[masked_index],
            true,
            super::CarverSkipModel::Cave { floor_level: -0.7 },
        )
        .contains(&BlockPos { x: 8, y: 65, z: 8 }));
        assert!(super::carver_ellipsoid_candidate_positions(
            0,
            0,
            height_context,
            false,
            100.0,
            64.0,
            8.0,
            2.0,
            2.0,
            &[],
            false,
            super::CarverSkipModel::None,
        )
        .is_empty());
        assert_eq!(super::cave_carver_cave_count(15, 14, 7, 3), 3);
        assert_eq!(super::cave_carver_cave_count(15, 0, 9, 9), 0);
        let mut cave_random = super::LegacyRandom::new(12345);
        assert_eq!(
            super::sample_cave_carver_cave_count(super::WorldCarverType::Cave, &mut cave_random),
            1
        );
        let mut nether_cave_random = super::LegacyRandom::new(8675309);
        assert_eq!(
            super::sample_cave_carver_cave_count(
                super::WorldCarverType::NetherCave,
                &mut nether_cave_random
            ),
            0
        );
        assert!((super::cave_carver_thickness(0.5, 0.25, 1, 1.0, 1.0) - 1.25).abs() < 0.0001);
        assert!((super::cave_carver_thickness(0.5, 0.25, 0, 0.5, 0.5) - 2.1875).abs() < 0.0001);
        assert_eq!(super::cave_room_radii(2.5, 0.5), (4.0, 2.0));
        let tunnel_steps = super::cave_tunnel_steps(
            8.0,
            8.0,
            8.0,
            64.0,
            8.0,
            2.0,
            0.0,
            0.0,
            4,
            1.0,
            1.0,
            1.0,
            &[1, 1, 1, 1],
            &[(0.5, 0.5, 0.0, 0.5, 0.5, 0.0); 4],
        );
        assert_eq!(tunnel_steps.len(), 4);
        assert!(tunnel_steps.iter().all(|step| step.carve && step.can_reach));
        assert!((tunnel_steps[0].x - 9.0).abs() < 0.0001);
        assert!((tunnel_steps[2].horizontal_radius - 3.5).abs() < 0.0001);
        let branch = super::cave_tunnel_split_branch(
            8.0,
            64.0,
            8.0,
            2.0,
            0.0,
            0.0,
            8,
            0,
            1,
            0.25,
            0.75,
            &[(0.5, 0.5, 0.0, 0.5, 0.5, 0.0); 8],
        )
        .unwrap();
        assert_eq!(branch.split_step, 2);
        assert!((branch.x - 11.0).abs() < 0.0001);
        assert!((branch.left_thickness - 0.625).abs() < 0.0001);
        assert!((branch.right_thickness - 0.875).abs() < 0.0001);
        assert!((branch.left_horizontal_rotation + std::f32::consts::FRAC_PI_2).abs() < 0.0001);
        assert!((branch.right_horizontal_rotation - std::f32::consts::FRAC_PI_2).abs() < 0.0001);
        assert!(super::cave_tunnel_split_branch(
            8.0,
            64.0,
            8.0,
            1.0,
            0.0,
            0.0,
            8,
            0,
            1,
            0.25,
            0.75,
            &[]
        )
        .is_none());
        let canyon_steps = super::canyon_tunnel_steps(
            8.0,
            8.0,
            8.0,
            64.0,
            8.0,
            2.0,
            0.0,
            0.0,
            4,
            1.0,
            0.75,
            1.0,
            &[1, 1, 1, 1],
            &[0.5, 1.0, 1.5, 2.0],
            &[1.0, 1.0, 1.0, 1.0],
            &[(0.5, 0.5, 0.0, 0.5, 0.5, 0.0); 4],
        );
        assert_eq!(canyon_steps.len(), 4);
        assert!((canyon_steps[0].horizontal_radius - 0.75).abs() < 0.0001);
        assert!((canyon_steps[2].horizontal_radius - 5.25).abs() < 0.0001);
        assert!((canyon_steps[2].vertical_radius - 6.125).abs() < 0.0001);
        assert_eq!(
            super::canyon_width_factors(4, 2, &[(0, 0.5), (1, 1.0), (0, 0.25), (1, 0.0)]),
            vec![1.5625, 1.5625, 1.1289063, 1.1289063]
        );
        assert!((super::canyon_vertical_radius(0.75, 1.0, 4.0, 8, 4, 1.0) - 7.0).abs() < 0.0001);
    }

    #[test]
    fn feature_type_registry_matches_vanilla_feature_order() {
        assert_eq!(FEATURE_TYPES.len(), 60);
        assert_eq!(
            FEATURE_TYPES
                .iter()
                .map(|feature| feature.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:no_op",
                "minecraft:tree",
                "minecraft:fallen_tree",
                "minecraft:block_pile",
                "minecraft:spring_feature",
                "minecraft:chorus_plant",
                "minecraft:replace_single_block",
                "minecraft:void_start_platform",
                "minecraft:desert_well",
                "minecraft:fossil",
                "minecraft:huge_red_mushroom",
                "minecraft:huge_brown_mushroom",
                "minecraft:spike",
                "minecraft:glowstone_blob",
                "minecraft:freeze_top_layer",
                "minecraft:vines",
                "minecraft:block_column",
                "minecraft:vegetation_patch",
                "minecraft:waterlogged_vegetation_patch",
                "minecraft:root_system",
                "minecraft:multiface_growth",
                "minecraft:underwater_magma",
                "minecraft:monster_room",
                "minecraft:blue_ice",
                "minecraft:iceberg",
                "minecraft:block_blob",
                "minecraft:disk",
                "minecraft:lake",
                "minecraft:ore",
                "minecraft:end_platform",
                "minecraft:end_spike",
                "minecraft:end_island",
                "minecraft:end_gateway",
                "minecraft:seagrass",
                "minecraft:kelp",
                "minecraft:coral_tree",
                "minecraft:coral_mushroom",
                "minecraft:coral_claw",
                "minecraft:sea_pickle",
                "minecraft:simple_block",
                "minecraft:bamboo",
                "minecraft:huge_fungus",
                "minecraft:nether_forest_vegetation",
                "minecraft:weeping_vines",
                "minecraft:twisting_vines",
                "minecraft:basalt_columns",
                "minecraft:delta_feature",
                "minecraft:netherrack_replace_blobs",
                "minecraft:fill_layer",
                "minecraft:bonus_chest",
                "minecraft:basalt_pillar",
                "minecraft:scattered_ore",
                "minecraft:random_selector",
                "minecraft:simple_random_selector",
                "minecraft:random_boolean_selector",
                "minecraft:geode",
                "minecraft:dripstone_cluster",
                "minecraft:large_dripstone",
                "minecraft:pointed_dripstone",
                "minecraft:sculk_patch",
            ]
        );

        let tree = super::feature_type_by_id("tree").unwrap();
        assert_eq!(tree.configuration, FeatureConfigurationKind::Tree);
        assert_eq!(tree.family, FeatureFamily::Tree);

        let ore = super::feature_type_by_id("minecraft:ore").unwrap();
        assert_eq!(ore.configuration, FeatureConfigurationKind::Ore);
        assert_eq!(ore.family, FeatureFamily::Ore);

        let random_selector = super::feature_type_by_id("random_selector").unwrap();
        assert_eq!(
            random_selector.configuration,
            FeatureConfigurationKind::RandomFeature
        );
        assert_eq!(random_selector.family, FeatureFamily::Selector);

        let sculk_patch = super::feature_type_by_id("sculk_patch").unwrap();
        assert_eq!(
            sculk_patch.configuration,
            FeatureConfigurationKind::SculkPatch
        );
        assert_eq!(sculk_patch.family, FeatureFamily::Cave);
    }

    #[test]
    fn configured_feature_bootstrap_keys_match_vanilla_sources() {
        assert_eq!(CONFIGURED_FEATURES.len(), 221);

        let source_counts = [
            (ConfiguredFeatureSource::Aquatic, 7),
            (ConfiguredFeatureSource::Cave, 24),
            (ConfiguredFeatureSource::End, 6),
            (ConfiguredFeatureSource::MiscOverworld, 18),
            (ConfiguredFeatureSource::Nether, 22),
            (ConfiguredFeatureSource::Ore, 32),
            (ConfiguredFeatureSource::Pile, 5),
            (ConfiguredFeatureSource::Tree, 50),
            (ConfiguredFeatureSource::Vegetation, 57),
        ];
        for (source, expected_count) in source_counts {
            assert_eq!(
                CONFIGURED_FEATURES
                    .iter()
                    .filter(|feature| feature.source == source)
                    .count(),
                expected_count
            );
        }

        assert_eq!(
            CONFIGURED_FEATURES
                .iter()
                .take(7)
                .map(|feature| feature.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:seagrass_short",
                "minecraft:seagrass_slightly_less_short",
                "minecraft:seagrass_mid",
                "minecraft:seagrass_tall",
                "minecraft:sea_pickle",
                "minecraft:kelp",
                "minecraft:warm_ocean_vegetation",
            ]
        );
        assert_eq!(
            CONFIGURED_FEATURES.last().map(|feature| feature.id),
            Some("minecraft:mangrove_vegetation")
        );

        assert_eq!(
            super::configured_feature("ore_diamond_buried").map(|feature| feature.source),
            Some(ConfiguredFeatureSource::Ore)
        );
        assert_eq!(
            super::configured_feature("minecraft:pale_oak_creaking").map(|feature| feature.source),
            Some(ConfiguredFeatureSource::Tree)
        );
        assert_eq!(
            super::configured_feature("sculk_patch_ancient_city").map(|feature| feature.source),
            Some(ConfiguredFeatureSource::Cave)
        );
    }

    #[test]
    fn placed_feature_bootstrap_keys_match_vanilla_sources() {
        assert_eq!(PLACED_FEATURE_BOOTSTRAP_SOURCES.len(), 9);
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES
                .iter()
                .map(|entry| (entry.source, entry.keys.len()))
                .collect::<Vec<_>>(),
            vec![
                (PlacedFeatureSource::Aquatic, 12),
                (PlacedFeatureSource::Cave, 20),
                (PlacedFeatureSource::End, 5),
                (PlacedFeatureSource::MiscOverworld, 18),
                (PlacedFeatureSource::Nether, 20),
                (PlacedFeatureSource::Ore, 40),
                (PlacedFeatureSource::Tree, 41),
                (PlacedFeatureSource::Vegetation, 89),
                (PlacedFeatureSource::Village, 13),
            ]
        );
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES
                .iter()
                .map(|entry| entry.keys.len())
                .sum::<usize>(),
            258
        );
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES[0].keys.first().copied(),
            Some("minecraft:seagrass_warm")
        );
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES
                .last()
                .and_then(|entry| entry.keys.last())
                .copied(),
            Some("minecraft:patch_berry_bush")
        );
        assert_eq!(
            super::placed_feature_source("ore_diamond"),
            Some(PlacedFeatureSource::Ore)
        );
        assert_eq!(
            super::placed_feature_source("minecraft:pale_oak_creaking_checked"),
            Some(PlacedFeatureSource::Tree)
        );
        assert_eq!(
            super::placed_feature_source("trees_mangrove"),
            Some(PlacedFeatureSource::Vegetation)
        );
    }

    #[test]
    fn placed_ore_feature_models_follow_vanilla_ore_placements() {
        let tuff = super::placed_ore_feature("minecraft:ore_tuff").unwrap();
        assert_eq!(tuff.configured_feature, "minecraft:ore_tuff");
        assert_eq!(
            tuff.placement,
            vec![
                PlacementModifier::Count { count: 2 },
                PlacementModifier::InSquare,
                PlacementModifier::HeightRange {
                    height: HeightProvider::Uniform {
                        min_inclusive: VerticalAnchor::AboveBottom(0),
                        max_inclusive: VerticalAnchor::Absolute(0),
                    }
                },
                PlacementModifier::BiomeFilter,
            ]
        );

        let granite_upper = super::placed_ore_feature("ore_granite_upper").unwrap();
        assert_eq!(granite_upper.configured_feature, "minecraft:ore_granite");
        assert_eq!(
            granite_upper.placement,
            vec![
                PlacementModifier::RarityFilter { chance: 6 },
                PlacementModifier::InSquare,
                PlacementModifier::HeightRange {
                    height: HeightProvider::Uniform {
                        min_inclusive: VerticalAnchor::Absolute(64),
                        max_inclusive: VerticalAnchor::Absolute(128),
                    }
                },
                PlacementModifier::BiomeFilter,
            ]
        );

        let diamond = super::placed_ore_feature("minecraft:ore_diamond").unwrap();
        assert_eq!(diamond.configured_feature, "minecraft:ore_diamond_small");
        assert_eq!(
            diamond.placement,
            vec![
                PlacementModifier::Count { count: 7 },
                PlacementModifier::InSquare,
                PlacementModifier::HeightRange {
                    height: HeightProvider::Trapezoid {
                        min_inclusive: VerticalAnchor::AboveBottom(-80),
                        max_inclusive: VerticalAnchor::AboveBottom(80),
                        plateau: 0,
                    }
                },
                PlacementModifier::BiomeFilter,
            ]
        );

        let gold_lower = super::placed_ore_feature("ore_gold_lower").unwrap();
        assert_eq!(gold_lower.configured_feature, "minecraft:ore_gold_buried");
        assert_eq!(
            gold_lower.placement[0],
            PlacementModifier::CountProvider {
                provider: super::IntProviderModel::Uniform {
                    min_inclusive: 0,
                    max_inclusive: 1,
                },
                sampled_count: 0,
            }
        );
        assert_eq!(
            gold_lower.placement[2],
            PlacementModifier::HeightRange {
                height: HeightProvider::Uniform {
                    min_inclusive: VerticalAnchor::Absolute(-64),
                    max_inclusive: VerticalAnchor::Absolute(-48),
                }
            }
        );

        let debris_small = super::placed_ore_feature("ore_debris_small").unwrap();
        assert_eq!(
            debris_small.configured_feature,
            "minecraft:ore_ancient_debris_small"
        );
        assert_eq!(
            debris_small.placement,
            vec![
                PlacementModifier::InSquare,
                PlacementModifier::HeightRange {
                    height: HeightProvider::Uniform {
                        min_inclusive: VerticalAnchor::AboveBottom(8),
                        max_inclusive: VerticalAnchor::BelowTop(8),
                    }
                },
                PlacementModifier::BiomeFilter,
            ]
        );

        let copper = super::placed_ore_feature("ore_copper").unwrap();
        assert_eq!(copper.configured_feature, "minecraft:ore_copper_small");
        assert_eq!(super::placed_ore_feature("minecraft:not_ore"), None);

        let disk_sand = super::placed_disk_feature("minecraft:disk_sand").unwrap();
        assert_eq!(disk_sand.configured_feature, "minecraft:disk_sand");
        assert_eq!(
            disk_sand.placement,
            vec![
                PlacementModifier::Count { count: 3 },
                PlacementModifier::InSquare,
                PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::OceanFloorWg,
                },
                PlacementModifier::BlockPredicateFilter {
                    predicate: BlockPredicate::MatchingFluids {
                        fluids: &["minecraft:water"],
                    },
                },
                PlacementModifier::BiomeFilter,
            ]
        );
        let disk_sand_config =
            super::configured_disk_configuration(disk_sand.configured_feature).unwrap();
        assert_eq!(
            disk_sand_config.radius,
            super::IntProviderModel::Uniform {
                min_inclusive: 2,
                max_inclusive: 6,
            }
        );
        assert_eq!(disk_sand_config.half_height, 2);
        assert_eq!(super::placed_disk_feature("minecraft:not_disk"), None);
    }

    #[test]
    fn biome_generation_settings_plains_matches_registry_payload() {
        let plains = super::biome_generation_settings("plains").unwrap();
        assert_eq!(
            *plains,
            BiomeGenerationSettingsModel {
                biome: "minecraft:plains",
                carvers: &[
                    "minecraft:cave",
                    "minecraft:cave_extra_underground",
                    "minecraft:canyon",
                ],
                feature_steps: super::PLAINS_FEATURE_STEPS,
                creature_spawn_probability: 0.1,
                spawn_costs: &[],
                spawners: super::PLAINS_SPAWNER_GROUPS,
            }
        );
        assert_eq!(plains.feature_steps.len(), 11);
        assert_eq!(plains.feature_steps[1].len(), 2);
        assert_eq!(plains.feature_steps[6].len(), 29);
        assert_eq!(
            plains.feature_steps[9],
            &[
                "minecraft:glow_lichen",
                "minecraft:patch_tall_grass_2",
                "minecraft:patch_bush",
                "minecraft:trees_plains",
                "minecraft:flower_plains",
                "minecraft:patch_grass_plain",
                "minecraft:brown_mushroom_normal",
                "minecraft:red_mushroom_normal",
                "minecraft:patch_pumpkin",
                "minecraft:patch_sugar_cane",
                "minecraft:patch_firefly_bush_near_water",
            ]
        );
        assert!(super::biome_has_placed_feature(plains, "trees_plains"));
        assert!(super::biome_has_placed_feature(
            plains,
            "minecraft:ore_diamond_buried"
        ));
        assert!(!super::biome_has_placed_feature(plains, "trees_jungle"));
        assert_eq!(
            super::biome_spawns_for_category(plains, "creature"),
            &[
                MobSpawnerDataModel {
                    entity_type: "minecraft:sheep",
                    weight: 12,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:pig",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:chicken",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:cow",
                    weight: 8,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:horse",
                    weight: 5,
                    min_count: 2,
                    max_count: 6,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:donkey",
                    weight: 1,
                    min_count: 1,
                    max_count: 3,
                },
            ]
        );
        assert_eq!(
            super::biome_spawns_for_category(plains, "underground_water_creature"),
            &[MobSpawnerDataModel {
                entity_type: "minecraft:glow_squid",
                weight: 10,
                min_count: 4,
                max_count: 6,
            }]
        );
        assert!(super::biome_spawns_for_category(plains, "water_creature").is_empty());
        assert!(super::biome_generation_settings("minecraft:badlands").is_some());
    }

    #[test]
    fn builtin_biome_generation_settings_cover_every_builtin_biome() {
        let mut builtins = crate::biome::BUILTIN_BIOMES
            .iter()
            .map(|biome| biome.id)
            .collect::<Vec<_>>();
        let mut generation_settings = super::BUILTIN_BIOME_GENERATION_SETTINGS
            .iter()
            .map(|entry| entry.biome)
            .collect::<Vec<_>>();
        builtins.sort_unstable();
        generation_settings.sort_unstable();
        assert_eq!(builtins, generation_settings);
    }

    #[test]
    fn the_void_biome_generation_settings_match_json_contract() {
        let the_void = super::biome_generation_settings("minecraft:the_void").unwrap();
        assert_eq!(
            *the_void,
            BiomeGenerationSettingsModel {
                biome: "minecraft:the_void",
                carvers: &[],
                feature_steps: super::THE_VOID_FEATURE_STEPS,
                creature_spawn_probability: 0.1,
                spawn_costs: &[],
                spawners: super::THE_VOID_SPAWNER_GROUPS,
            }
        );
        assert_eq!(the_void.feature_steps.len(), 11);
        assert_eq!(
            the_void.feature_steps[10],
            &["minecraft:void_start_platform"]
        );
        assert_eq!(super::biome_spawns_for_category(the_void, "monster"), &[]);
    }

    #[test]
    fn biome_generation_settings_cover_neighboring_overworld_payloads() {
        let sunflower = super::biome_generation_settings("sunflower_plains").unwrap();
        assert_eq!(sunflower.biome, "minecraft:sunflower_plains");
        assert_eq!(sunflower.carvers, super::OVERWORLD_COMMON_CARVERS);
        assert!(super::biome_has_placed_feature(
            sunflower,
            "minecraft:patch_sunflower"
        ));
        assert!(super::biome_has_placed_feature(
            sunflower,
            "minecraft:trees_plains"
        ));
        assert!(!super::biome_has_placed_feature(
            sunflower,
            "minecraft:trees_birch_and_oak_leaf_litter"
        ));
        assert_eq!(
            super::biome_spawns_for_category(sunflower, "creature"),
            super::PLAINS_CREATURE_SPAWNS
        );

        let forest = super::biome_generation_settings("minecraft:forest").unwrap();
        assert_eq!(forest.biome, "minecraft:forest");
        assert_eq!(forest.feature_steps.len(), 11);
        assert!(super::biome_has_placed_feature(
            forest,
            "minecraft:forest_flowers"
        ));
        assert!(super::biome_has_placed_feature(
            forest,
            "minecraft:trees_birch_and_oak_leaf_litter"
        ));
        assert!(!super::biome_has_placed_feature(
            forest,
            "minecraft:flower_plains"
        ));
        assert_eq!(
            super::biome_spawns_for_category(forest, "creature"),
            &[
                MobSpawnerDataModel {
                    entity_type: "minecraft:sheep",
                    weight: 12,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:pig",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:chicken",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:cow",
                    weight: 8,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:wolf",
                    weight: 5,
                    min_count: 4,
                    max_count: 4,
                },
            ]
        );
        assert_eq!(
            super::biome_spawns_for_category(forest, "monster")[1],
            MobSpawnerDataModel {
                entity_type: "minecraft:zombie",
                weight: 95,
                min_count: 4,
                max_count: 4,
            }
        );

        let birch = super::biome_generation_settings("birch_forest").unwrap();
        assert_eq!(birch.biome, "minecraft:birch_forest");
        assert!(super::biome_has_placed_feature(
            birch,
            "minecraft:trees_birch"
        ));
        assert!(super::biome_has_placed_feature(
            birch,
            "minecraft:wildflowers_birch_forest"
        ));
        assert!(!super::biome_has_placed_feature(
            birch,
            "minecraft:birch_tall"
        ));
        assert_eq!(
            super::biome_spawns_for_category(birch, "creature"),
            super::BIRCH_FOREST_CREATURE_SPAWNS
        );
        assert!(!super::biome_spawns_for_category(birch, "creature")
            .iter()
            .any(|spawn| spawn.entity_type == "minecraft:wolf"));
        assert_eq!(
            super::biome_spawns_for_category(birch, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );

        let old_growth_birch = super::biome_generation_settings("old_growth_birch_forest").unwrap();
        assert_eq!(old_growth_birch.biome, "minecraft:old_growth_birch_forest");
        assert!(super::biome_has_placed_feature(
            old_growth_birch,
            "minecraft:birch_tall"
        ));
        assert!(!super::biome_has_placed_feature(
            old_growth_birch,
            "minecraft:trees_birch"
        ));
        assert_eq!(
            super::biome_spawns_for_category(old_growth_birch, "creature"),
            super::BIRCH_FOREST_CREATURE_SPAWNS
        );

        let dark_forest = super::biome_generation_settings("dark_forest").unwrap();
        assert_eq!(dark_forest.biome, "minecraft:dark_forest");
        assert!(super::biome_has_placed_feature(
            dark_forest,
            "minecraft:dark_forest_vegetation"
        ));
        assert!(super::biome_has_placed_feature(
            dark_forest,
            "minecraft:patch_leaf_litter"
        ));
        assert!(!super::biome_has_placed_feature(
            dark_forest,
            "minecraft:trees_birch"
        ));
        assert_eq!(
            super::biome_spawns_for_category(dark_forest, "creature"),
            super::BIRCH_FOREST_CREATURE_SPAWNS
        );

        let flower_forest = super::biome_generation_settings("flower_forest").unwrap();
        assert_eq!(flower_forest.biome, "minecraft:flower_forest");
        assert!(super::biome_has_placed_feature(
            flower_forest,
            "minecraft:flower_forest_flowers"
        ));
        assert!(super::biome_has_placed_feature(
            flower_forest,
            "minecraft:trees_flower_forest"
        ));
        assert!(super::biome_has_placed_feature(
            flower_forest,
            "minecraft:patch_grass_badlands"
        ));
        assert!(!super::biome_has_placed_feature(
            flower_forest,
            "minecraft:patch_grass_forest"
        ));
        assert_eq!(
            super::biome_spawns_for_category(flower_forest, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:rabbit",
                weight: 4,
                min_count: 2,
                max_count: 3,
            })
        );

        let river = super::biome_generation_settings("river").unwrap();
        assert_eq!(river.biome, "minecraft:river");
        assert_eq!(river.carvers, super::OVERWORLD_COMMON_CARVERS);
        assert_eq!(river.feature_steps.len(), 11);
        assert!(super::biome_has_placed_feature(
            river,
            "minecraft:trees_water"
        ));
        assert!(super::biome_has_placed_feature(
            river,
            "minecraft:seagrass_river"
        ));
        assert!(!super::biome_has_placed_feature(
            river,
            "minecraft:trees_plains"
        ));
        assert!(super::biome_spawns_for_category(river, "creature").is_empty());
        assert_eq!(
            super::biome_spawns_for_category(river, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:drowned",
                weight: 100,
                min_count: 1,
                max_count: 1,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(river, "water_ambient"),
            super::RIVER_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(river, "water_creature"),
            super::RIVER_WATER_CREATURE_SPAWNS
        );

        let frozen_river = super::biome_generation_settings("frozen_river").unwrap();
        assert_eq!(frozen_river.biome, "minecraft:frozen_river");
        assert!(super::biome_has_placed_feature(
            frozen_river,
            "minecraft:patch_bush"
        ));
        assert!(!super::biome_has_placed_feature(
            frozen_river,
            "minecraft:seagrass_river"
        ));
        assert_eq!(
            super::biome_spawns_for_category(frozen_river, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:drowned",
                weight: 1,
                min_count: 1,
                max_count: 1,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(frozen_river, "water_ambient"),
            super::FROZEN_RIVER_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(frozen_river, "water_creature"),
            super::FROZEN_RIVER_WATER_CREATURE_SPAWNS
        );

        let ocean = super::biome_generation_settings("ocean").unwrap();
        assert_eq!(ocean.biome, "minecraft:ocean");
        assert!(super::biome_has_placed_feature(
            ocean,
            "minecraft:seagrass_normal"
        ));
        assert!(super::biome_has_placed_feature(
            ocean,
            "minecraft:kelp_cold"
        ));
        assert!(!super::biome_has_placed_feature(
            ocean,
            "minecraft:warm_ocean_vegetation"
        ));
        assert_eq!(
            super::biome_spawns_for_category(ocean, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:drowned",
                weight: 5,
                min_count: 1,
                max_count: 1,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(ocean, "water_ambient"),
            super::OCEAN_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(ocean, "water_creature"),
            super::OCEAN_WATER_CREATURE_SPAWNS
        );

        let deep_ocean = super::biome_generation_settings("deep_ocean").unwrap();
        assert_eq!(deep_ocean.biome, "minecraft:deep_ocean");
        assert!(super::biome_has_placed_feature(
            deep_ocean,
            "minecraft:seagrass_deep"
        ));
        assert!(!super::biome_has_placed_feature(
            deep_ocean,
            "minecraft:seagrass_normal"
        ));
        assert_eq!(
            super::biome_spawns_for_category(deep_ocean, "water_ambient"),
            super::OCEAN_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(deep_ocean, "water_creature"),
            super::OCEAN_WATER_CREATURE_SPAWNS
        );

        let cold_ocean = super::biome_generation_settings("cold_ocean").unwrap();
        assert_eq!(cold_ocean.biome, "minecraft:cold_ocean");
        assert!(super::biome_has_placed_feature(
            cold_ocean,
            "minecraft:seagrass_cold"
        ));
        assert!(super::biome_has_placed_feature(
            cold_ocean,
            "minecraft:kelp_cold"
        ));
        assert!(!super::biome_has_placed_feature(
            cold_ocean,
            "minecraft:seagrass_normal"
        ));
        assert_eq!(
            super::biome_spawns_for_category(cold_ocean, "water_ambient"),
            super::COLD_OCEAN_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(cold_ocean, "water_creature"),
            super::COLD_OCEAN_WATER_CREATURE_SPAWNS
        );

        let deep_cold_ocean = super::biome_generation_settings("deep_cold_ocean").unwrap();
        assert_eq!(deep_cold_ocean.biome, "minecraft:deep_cold_ocean");
        assert!(super::biome_has_placed_feature(
            deep_cold_ocean,
            "minecraft:seagrass_deep_cold"
        ));
        assert!(!super::biome_has_placed_feature(
            deep_cold_ocean,
            "minecraft:seagrass_cold"
        ));
        assert_eq!(
            super::biome_spawns_for_category(deep_cold_ocean, "water_ambient"),
            super::COLD_OCEAN_WATER_AMBIENT_SPAWNS
        );

        let lukewarm_ocean = super::biome_generation_settings("lukewarm_ocean").unwrap();
        assert_eq!(lukewarm_ocean.biome, "minecraft:lukewarm_ocean");
        assert!(super::biome_has_placed_feature(
            lukewarm_ocean,
            "minecraft:seagrass_warm"
        ));
        assert!(super::biome_has_placed_feature(
            lukewarm_ocean,
            "minecraft:kelp_warm"
        ));
        assert!(!super::biome_has_placed_feature(
            lukewarm_ocean,
            "minecraft:warm_ocean_vegetation"
        ));
        assert_eq!(
            super::biome_spawns_for_category(lukewarm_ocean, "water_ambient"),
            super::LUKEWARM_OCEAN_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(lukewarm_ocean, "water_creature"),
            super::LUKEWARM_OCEAN_WATER_CREATURE_SPAWNS
        );

        let deep_lukewarm_ocean = super::biome_generation_settings("deep_lukewarm_ocean").unwrap();
        assert_eq!(deep_lukewarm_ocean.biome, "minecraft:deep_lukewarm_ocean");
        assert!(super::biome_has_placed_feature(
            deep_lukewarm_ocean,
            "minecraft:seagrass_deep_warm"
        ));
        assert!(super::biome_has_placed_feature(
            deep_lukewarm_ocean,
            "minecraft:kelp_warm"
        ));
        assert_eq!(
            super::biome_spawns_for_category(deep_lukewarm_ocean, "water_ambient"),
            super::DEEP_LUKEWARM_OCEAN_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(deep_lukewarm_ocean, "water_creature"),
            super::DEEP_LUKEWARM_OCEAN_WATER_CREATURE_SPAWNS
        );

        let warm_ocean = super::biome_generation_settings("warm_ocean").unwrap();
        assert_eq!(warm_ocean.biome, "minecraft:warm_ocean");
        assert!(super::biome_has_placed_feature(
            warm_ocean,
            "minecraft:warm_ocean_vegetation"
        ));
        assert!(super::biome_has_placed_feature(
            warm_ocean,
            "minecraft:seagrass_warm"
        ));
        assert!(super::biome_has_placed_feature(
            warm_ocean,
            "minecraft:sea_pickle"
        ));
        assert!(!super::biome_has_placed_feature(
            warm_ocean,
            "minecraft:kelp_cold"
        ));
        assert_eq!(
            super::biome_spawns_for_category(warm_ocean, "monster").first(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:drowned",
                weight: 5,
                min_count: 1,
                max_count: 1,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(warm_ocean, "water_ambient"),
            super::WARM_OCEAN_WATER_AMBIENT_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(warm_ocean, "water_creature"),
            super::WARM_OCEAN_WATER_CREATURE_SPAWNS
        );

        let deep_frozen_ocean = super::biome_generation_settings("deep_frozen_ocean").unwrap();
        assert_eq!(deep_frozen_ocean.biome, "minecraft:deep_frozen_ocean");
        assert!(super::biome_has_placed_feature(
            deep_frozen_ocean,
            "minecraft:iceberg_packed"
        ));
        assert!(super::biome_has_placed_feature(
            deep_frozen_ocean,
            "minecraft:blue_ice"
        ));
        assert!(!super::biome_has_placed_feature(
            deep_frozen_ocean,
            "minecraft:kelp_cold"
        ));
        assert_eq!(
            super::biome_spawns_for_category(deep_frozen_ocean, "creature"),
            super::DEEP_FROZEN_OCEAN_CREATURE_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(deep_frozen_ocean, "water_ambient"),
            super::DEEP_FROZEN_OCEAN_WATER_AMBIENT_SPAWNS
        );

        let frozen_ocean = super::biome_generation_settings("frozen_ocean").unwrap();
        assert_eq!(frozen_ocean.biome, "minecraft:frozen_ocean");
        assert_eq!(
            frozen_ocean.feature_steps,
            super::DEEP_FROZEN_OCEAN_FEATURE_STEPS
        );
        assert_eq!(
            super::biome_spawns_for_category(frozen_ocean, "creature"),
            super::DEEP_FROZEN_OCEAN_CREATURE_SPAWNS
        );

        let beach = super::biome_generation_settings("minecraft:beach").unwrap();
        assert_eq!(beach.biome, "minecraft:beach");
        assert_eq!(beach.carvers, super::OVERWORLD_COMMON_CARVERS);
        assert!(super::biome_has_placed_feature(
            beach,
            "minecraft:flower_default"
        ));
        assert!(!super::biome_has_placed_feature(
            beach,
            "minecraft:trees_water"
        ));
        assert!(!super::biome_has_placed_feature(
            beach,
            "minecraft:seagrass_river"
        ));
        assert_eq!(
            super::biome_spawns_for_category(beach, "creature"),
            super::BEACH_CREATURE_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(beach, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );
        assert!(super::biome_spawns_for_category(beach, "water_ambient").is_empty());
        assert!(super::biome_spawns_for_category(beach, "water_creature").is_empty());

        let snowy_beach = super::biome_generation_settings("snowy_beach").unwrap();
        assert_eq!(snowy_beach.biome, "minecraft:snowy_beach");
        assert!(super::biome_has_placed_feature(
            snowy_beach,
            "minecraft:flower_default"
        ));
        assert!(!super::biome_has_placed_feature(
            snowy_beach,
            "minecraft:trees_water"
        ));
        assert!(super::biome_spawns_for_category(snowy_beach, "creature").is_empty());
        assert_eq!(
            super::biome_spawns_for_category(snowy_beach, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );

        let stony_shore = super::biome_generation_settings("stony_shore").unwrap();
        assert_eq!(stony_shore.biome, "minecraft:stony_shore");
        assert!(super::biome_has_placed_feature(
            stony_shore,
            "minecraft:flower_default"
        ));
        assert!(super::biome_has_placed_feature(
            stony_shore,
            "minecraft:patch_grass_badlands"
        ));
        assert!(!super::biome_has_placed_feature(
            stony_shore,
            "minecraft:trees_water"
        ));
        assert!(super::biome_spawns_for_category(stony_shore, "creature").is_empty());
        assert_eq!(
            super::biome_spawns_for_category(stony_shore, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );

        let mushroom_fields = super::biome_generation_settings("mushroom_fields").unwrap();
        assert_eq!(mushroom_fields.biome, "minecraft:mushroom_fields");
        assert!(super::biome_has_placed_feature(
            mushroom_fields,
            "minecraft:mushroom_island_vegetation"
        ));
        assert!(super::biome_has_placed_feature(
            mushroom_fields,
            "minecraft:brown_mushroom_taiga"
        ));
        assert!(!super::biome_has_placed_feature(
            mushroom_fields,
            "minecraft:flower_default"
        ));
        assert_eq!(
            super::biome_spawns_for_category(mushroom_fields, "creature"),
            super::MUSHROOM_FIELDS_CREATURE_SPAWNS
        );
        assert!(super::biome_spawns_for_category(mushroom_fields, "monster").is_empty());

        let badlands = super::biome_generation_settings("badlands").unwrap();
        assert_eq!(badlands.biome, "minecraft:badlands");
        assert_eq!(badlands.creature_spawn_probability, 0.03);
        assert!(super::biome_has_placed_feature(
            badlands,
            "minecraft:ore_gold_extra"
        ));
        assert!(super::biome_has_placed_feature(
            badlands,
            "minecraft:patch_cactus_decorated"
        ));
        assert!(super::biome_has_placed_feature(
            badlands,
            "minecraft:patch_sugar_cane_badlands"
        ));
        assert!(!super::biome_has_placed_feature(
            badlands,
            "minecraft:patch_cactus_desert"
        ));
        assert_eq!(
            super::biome_spawns_for_category(badlands, "creature"),
            super::BADLANDS_CREATURE_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(badlands, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );

        let eroded_badlands = super::biome_generation_settings("eroded_badlands").unwrap();
        assert_eq!(eroded_badlands.biome, "minecraft:eroded_badlands");
        assert_eq!(eroded_badlands.feature_steps, super::BADLANDS_FEATURE_STEPS);
        assert_eq!(eroded_badlands.creature_spawn_probability, 0.03);
        assert_eq!(
            super::biome_spawns_for_category(eroded_badlands, "creature"),
            super::BADLANDS_CREATURE_SPAWNS
        );

        let wooded_badlands = super::biome_generation_settings("wooded_badlands").unwrap();
        assert_eq!(wooded_badlands.biome, "minecraft:wooded_badlands");
        assert_eq!(wooded_badlands.creature_spawn_probability, 0.04);
        assert!(super::biome_has_placed_feature(
            wooded_badlands,
            "minecraft:trees_badlands"
        ));
        assert_eq!(
            super::biome_spawns_for_category(wooded_badlands, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:wolf",
                weight: 2,
                min_count: 4,
                max_count: 8,
            })
        );

        let meadow = super::biome_generation_settings("meadow").unwrap();
        assert_eq!(meadow.biome, "minecraft:meadow");
        assert!(super::biome_has_placed_feature(
            meadow,
            "minecraft:ore_emerald"
        ));
        assert!(super::biome_has_placed_feature(
            meadow,
            "minecraft:ore_infested"
        ));
        assert!(super::biome_has_placed_feature(
            meadow,
            "minecraft:wildflowers_meadow"
        ));
        assert!(!super::biome_has_placed_feature(
            meadow,
            "minecraft:flower_default"
        ));
        assert_eq!(
            super::biome_spawns_for_category(meadow, "creature"),
            super::MEADOW_CREATURE_SPAWNS
        );

        let cherry_grove = super::biome_generation_settings("cherry_grove").unwrap();
        assert_eq!(cherry_grove.biome, "minecraft:cherry_grove");
        assert!(super::biome_has_placed_feature(
            cherry_grove,
            "minecraft:trees_cherry"
        ));
        assert!(super::biome_has_placed_feature(
            cherry_grove,
            "minecraft:flower_cherry"
        ));
        assert!(super::biome_has_placed_feature(
            cherry_grove,
            "minecraft:ore_emerald"
        ));
        assert_eq!(
            super::biome_spawns_for_category(cherry_grove, "creature"),
            super::CHERRY_GROVE_CREATURE_SPAWNS
        );

        let pale_garden = super::biome_generation_settings("pale_garden").unwrap();
        assert_eq!(pale_garden.biome, "minecraft:pale_garden");
        assert!(super::biome_has_placed_feature(
            pale_garden,
            "minecraft:pale_garden_vegetation"
        ));
        assert!(super::biome_has_placed_feature(
            pale_garden,
            "minecraft:pale_moss_patch"
        ));
        assert!(super::biome_has_placed_feature(
            pale_garden,
            "minecraft:flower_pale_garden"
        ));
        assert!(super::biome_spawns_for_category(pale_garden, "creature").is_empty());

        let lush_caves = super::biome_generation_settings("lush_caves").unwrap();
        assert_eq!(lush_caves.biome, "minecraft:lush_caves");
        assert!(super::biome_has_placed_feature(
            lush_caves,
            "minecraft:ore_clay"
        ));
        assert!(super::biome_has_placed_feature(
            lush_caves,
            "minecraft:lush_caves_ceiling_vegetation"
        ));
        assert!(super::biome_has_placed_feature(
            lush_caves,
            "minecraft:cave_vines"
        ));
        assert!(!super::biome_has_placed_feature(
            lush_caves,
            "minecraft:trees_plains"
        ));
        assert_eq!(
            super::biome_spawns_for_category(lush_caves, "axolotls"),
            super::LUSH_CAVES_AXOLOTL_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(lush_caves, "water_ambient"),
            super::MANGROVE_SWAMP_WATER_AMBIENT_SPAWNS
        );

        let dripstone_caves = super::biome_generation_settings("dripstone_caves").unwrap();
        assert_eq!(dripstone_caves.biome, "minecraft:dripstone_caves");
        assert!(super::biome_has_placed_feature(
            dripstone_caves,
            "minecraft:large_dripstone"
        ));
        assert!(super::biome_has_placed_feature(
            dripstone_caves,
            "minecraft:ore_copper_large"
        ));
        assert!(super::biome_has_placed_feature(
            dripstone_caves,
            "minecraft:dripstone_cluster"
        ));
        assert!(!super::biome_has_placed_feature(
            dripstone_caves,
            "minecraft:ore_copper"
        ));
        assert_eq!(
            super::biome_spawns_for_category(dripstone_caves, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:drowned",
                weight: 95,
                min_count: 4,
                max_count: 4,
            })
        );

        let deep_dark = super::biome_generation_settings("deep_dark").unwrap();
        assert_eq!(deep_dark.biome, "minecraft:deep_dark");
        assert!(!super::biome_has_placed_feature(
            deep_dark,
            "minecraft:lake_lava_surface"
        ));
        assert!(super::biome_has_placed_feature(
            deep_dark,
            "minecraft:sculk_vein"
        ));
        assert!(super::biome_has_placed_feature(
            deep_dark,
            "minecraft:sculk_patch_deep_dark"
        ));
        assert!(!super::biome_has_placed_feature(
            deep_dark,
            "minecraft:spring_lava"
        ));
        assert!(deep_dark
            .spawners
            .iter()
            .all(|group| group.entries.is_empty()));

        let nether_wastes = super::biome_generation_settings("nether_wastes").unwrap();
        assert_eq!(nether_wastes.biome, "minecraft:nether_wastes");
        assert_eq!(nether_wastes.carvers, super::NETHER_COMMON_CARVERS);
        assert_eq!(nether_wastes.feature_steps.len(), 10);
        assert!(super::biome_has_placed_feature(
            nether_wastes,
            "minecraft:patch_soul_fire"
        ));
        assert!(super::biome_has_placed_feature(
            nether_wastes,
            "minecraft:brown_mushroom_nether"
        ));
        assert_eq!(
            super::biome_spawns_for_category(nether_wastes, "creature"),
            super::NETHER_CREATURE_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(nether_wastes, "monster"),
            super::NETHER_WASTES_MONSTER_SPAWNS
        );

        let crimson_forest = super::biome_generation_settings("crimson_forest").unwrap();
        assert_eq!(crimson_forest.biome, "minecraft:crimson_forest");
        assert!(super::biome_has_placed_feature(
            crimson_forest,
            "minecraft:weeping_vines"
        ));
        assert!(super::biome_has_placed_feature(
            crimson_forest,
            "minecraft:crimson_fungi"
        ));
        assert!(!super::biome_has_placed_feature(
            crimson_forest,
            "minecraft:patch_soul_fire"
        ));
        assert_eq!(
            super::biome_spawns_for_category(crimson_forest, "monster"),
            super::CRIMSON_FOREST_MONSTER_SPAWNS
        );

        let warped_forest = super::biome_generation_settings("warped_forest").unwrap();
        assert_eq!(warped_forest.biome, "minecraft:warped_forest");
        assert!(super::biome_has_placed_feature(
            warped_forest,
            "minecraft:warped_forest_vegetation"
        ));
        assert!(super::biome_has_placed_feature(
            warped_forest,
            "minecraft:twisting_vines"
        ));
        assert_eq!(warped_forest.spawn_costs, super::WARPED_FOREST_SPAWN_COSTS);
        assert_eq!(
            super::biome_spawns_for_category(warped_forest, "monster"),
            super::WARPED_FOREST_MONSTER_SPAWNS
        );

        let soul_sand_valley = super::biome_generation_settings("soul_sand_valley").unwrap();
        assert_eq!(soul_sand_valley.biome, "minecraft:soul_sand_valley");
        assert!(super::biome_has_placed_feature(
            soul_sand_valley,
            "minecraft:basalt_pillar"
        ));
        assert!(super::biome_has_placed_feature(
            soul_sand_valley,
            "minecraft:ore_soul_sand"
        ));
        assert_eq!(
            soul_sand_valley.spawn_costs,
            super::SOUL_SAND_VALLEY_SPAWN_COSTS
        );
        assert_eq!(
            super::biome_spawns_for_category(soul_sand_valley, "monster")[0],
            MobSpawnerDataModel {
                entity_type: "minecraft:skeleton",
                weight: 20,
                min_count: 5,
                max_count: 5,
            }
        );

        let basalt_deltas = super::biome_generation_settings("basalt_deltas").unwrap();
        assert_eq!(basalt_deltas.biome, "minecraft:basalt_deltas");
        assert_eq!(basalt_deltas.feature_steps.len(), 8);
        assert!(super::biome_has_placed_feature(
            basalt_deltas,
            "minecraft:delta"
        ));
        assert!(super::biome_has_placed_feature(
            basalt_deltas,
            "minecraft:spring_closed_double"
        ));
        assert!(!super::biome_has_placed_feature(
            basalt_deltas,
            "minecraft:spring_lava"
        ));
        assert_eq!(
            super::biome_spawns_for_category(basalt_deltas, "monster"),
            super::BASALT_DELTAS_MONSTER_SPAWNS
        );

        let the_end = super::biome_generation_settings("the_end").unwrap();
        assert_eq!(the_end.biome, "minecraft:the_end");
        assert!(the_end.carvers.is_empty());
        assert_eq!(the_end.feature_steps.len(), 11);
        assert!(super::biome_has_placed_feature(
            the_end,
            "minecraft:end_spike"
        ));
        assert!(super::biome_has_placed_feature(
            the_end,
            "minecraft:end_platform"
        ));
        assert_eq!(
            super::biome_spawns_for_category(the_end, "monster"),
            super::END_MONSTER_SPAWNS
        );

        let end_highlands = super::biome_generation_settings("end_highlands").unwrap();
        assert_eq!(end_highlands.biome, "minecraft:end_highlands");
        assert_eq!(end_highlands.feature_steps.len(), 10);
        assert!(super::biome_has_placed_feature(
            end_highlands,
            "minecraft:end_gateway_return"
        ));
        assert!(super::biome_has_placed_feature(
            end_highlands,
            "minecraft:chorus_plant"
        ));
        assert!(!super::biome_has_placed_feature(
            end_highlands,
            "minecraft:end_platform"
        ));

        let end_midlands = super::biome_generation_settings("end_midlands").unwrap();
        assert_eq!(end_midlands.biome, "minecraft:end_midlands");
        assert!(end_midlands.carvers.is_empty());
        assert!(end_midlands.feature_steps.is_empty());
        assert_eq!(
            super::biome_spawns_for_category(end_midlands, "monster"),
            super::END_MONSTER_SPAWNS
        );

        let small_end_islands = super::biome_generation_settings("small_end_islands").unwrap();
        assert_eq!(small_end_islands.biome, "minecraft:small_end_islands");
        assert_eq!(small_end_islands.feature_steps.len(), 1);
        assert!(super::biome_has_placed_feature(
            small_end_islands,
            "minecraft:end_island_decorated"
        ));

        let end_barrens = super::biome_generation_settings("end_barrens").unwrap();
        assert_eq!(end_barrens.biome, "minecraft:end_barrens");
        assert!(end_barrens.feature_steps.is_empty());
        assert_eq!(
            super::biome_spawns_for_category(end_barrens, "monster"),
            super::END_MONSTER_SPAWNS
        );

        let grove = super::biome_generation_settings("grove").unwrap();
        assert_eq!(grove.biome, "minecraft:grove");
        assert!(super::biome_has_placed_feature(
            grove,
            "minecraft:spring_lava_frozen"
        ));
        assert!(super::biome_has_placed_feature(
            grove,
            "minecraft:trees_grove"
        ));
        assert!(!super::biome_has_placed_feature(
            grove,
            "minecraft:patch_grass_meadow"
        ));
        assert_eq!(
            super::biome_spawns_for_category(grove, "creature"),
            super::GROVE_CREATURE_SPAWNS
        );

        let snowy_slopes = super::biome_generation_settings("snowy_slopes").unwrap();
        assert_eq!(snowy_slopes.biome, "minecraft:snowy_slopes");
        assert!(super::biome_has_placed_feature(
            snowy_slopes,
            "minecraft:spring_lava_frozen"
        ));
        assert!(super::biome_has_placed_feature(
            snowy_slopes,
            "minecraft:patch_pumpkin"
        ));
        assert!(!super::biome_has_placed_feature(
            snowy_slopes,
            "minecraft:trees_grove"
        ));
        assert_eq!(
            super::biome_spawns_for_category(snowy_slopes, "creature"),
            super::SNOWY_SLOPES_CREATURE_SPAWNS
        );

        let frozen_peaks = super::biome_generation_settings("frozen_peaks").unwrap();
        assert_eq!(frozen_peaks.biome, "minecraft:frozen_peaks");
        assert!(super::biome_has_placed_feature(
            frozen_peaks,
            "minecraft:ore_emerald"
        ));
        assert!(super::biome_has_placed_feature(
            frozen_peaks,
            "minecraft:spring_lava_frozen"
        ));
        assert!(!super::biome_has_placed_feature(
            frozen_peaks,
            "minecraft:patch_pumpkin"
        ));
        assert_eq!(
            super::biome_spawns_for_category(frozen_peaks, "creature"),
            super::GOAT_CREATURE_SPAWNS
        );

        let jagged_peaks = super::biome_generation_settings("jagged_peaks").unwrap();
        assert_eq!(jagged_peaks.biome, "minecraft:jagged_peaks");
        assert_eq!(
            jagged_peaks.feature_steps,
            super::FROZEN_PEAKS_FEATURE_STEPS
        );
        assert_eq!(
            super::biome_spawns_for_category(jagged_peaks, "creature"),
            super::GOAT_CREATURE_SPAWNS
        );

        let stony_peaks = super::biome_generation_settings("stony_peaks").unwrap();
        assert_eq!(stony_peaks.biome, "minecraft:stony_peaks");
        assert!(super::biome_has_placed_feature(
            stony_peaks,
            "minecraft:ore_infested"
        ));
        assert!(!super::biome_has_placed_feature(
            stony_peaks,
            "minecraft:spring_lava_frozen"
        ));
        assert!(super::biome_spawns_for_category(stony_peaks, "creature").is_empty());

        let windswept_hills = super::biome_generation_settings("windswept_hills").unwrap();
        assert_eq!(windswept_hills.biome, "minecraft:windswept_hills");
        assert!(super::biome_has_placed_feature(
            windswept_hills,
            "minecraft:trees_windswept_hills"
        ));
        assert!(super::biome_has_placed_feature(
            windswept_hills,
            "minecraft:ore_emerald"
        ));
        assert!(!super::biome_has_placed_feature(
            windswept_hills,
            "minecraft:trees_windswept_forest"
        ));
        assert_eq!(
            super::biome_spawns_for_category(windswept_hills, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:llama",
                weight: 5,
                min_count: 4,
                max_count: 6,
            })
        );

        let windswept_gravelly =
            super::biome_generation_settings("windswept_gravelly_hills").unwrap();
        assert_eq!(
            windswept_gravelly.biome,
            "minecraft:windswept_gravelly_hills"
        );
        assert_eq!(
            windswept_gravelly.feature_steps,
            super::WINDSWEPT_HILLS_FEATURE_STEPS
        );

        let windswept_forest = super::biome_generation_settings("windswept_forest").unwrap();
        assert_eq!(windswept_forest.biome, "minecraft:windswept_forest");
        assert!(super::biome_has_placed_feature(
            windswept_forest,
            "minecraft:trees_windswept_forest"
        ));
        assert!(!super::biome_has_placed_feature(
            windswept_forest,
            "minecraft:trees_windswept_hills"
        ));
        assert_eq!(
            super::biome_spawns_for_category(windswept_forest, "creature"),
            super::WINDSWEPT_CREATURE_SPAWNS
        );

        let desert = super::biome_generation_settings("desert").unwrap();
        assert_eq!(desert.biome, "minecraft:desert");
        assert!(super::biome_has_placed_feature(
            desert,
            "minecraft:desert_well"
        ));
        assert!(super::biome_has_placed_feature(
            desert,
            "minecraft:fossil_upper"
        ));
        assert!(super::biome_has_placed_feature(
            desert,
            "minecraft:patch_cactus_desert"
        ));
        assert!(!super::biome_has_placed_feature(
            desert,
            "minecraft:patch_sugar_cane"
        ));
        assert_eq!(
            super::biome_spawns_for_category(desert, "creature"),
            super::DESERT_CREATURE_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(desert, "monster")[1],
            MobSpawnerDataModel {
                entity_type: "minecraft:zombie",
                weight: 19,
                min_count: 4,
                max_count: 4,
            }
        );
        assert_eq!(
            super::biome_spawns_for_category(desert, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:parched",
                weight: 50,
                min_count: 4,
                max_count: 4,
            })
        );

        let savanna = super::biome_generation_settings("savanna").unwrap();
        assert_eq!(savanna.biome, "minecraft:savanna");
        assert!(super::biome_has_placed_feature(
            savanna,
            "minecraft:trees_savanna"
        ));
        assert!(super::biome_has_placed_feature(
            savanna,
            "minecraft:flower_warm"
        ));
        assert!(super::biome_has_placed_feature(
            savanna,
            "minecraft:patch_grass_savanna"
        ));
        assert!(!super::biome_has_placed_feature(
            savanna,
            "minecraft:trees_plains"
        ));
        assert_eq!(
            super::biome_spawns_for_category(savanna, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:armadillo",
                weight: 10,
                min_count: 2,
                max_count: 3,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(savanna, "monster"),
            super::PLAINS_MONSTER_SPAWNS
        );

        let savanna_plateau = super::biome_generation_settings("savanna_plateau").unwrap();
        assert_eq!(savanna_plateau.biome, "minecraft:savanna_plateau");
        assert_eq!(savanna_plateau.feature_steps, super::SAVANNA_FEATURE_STEPS);
        assert_eq!(
            super::biome_spawns_for_category(savanna_plateau, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:wolf",
                weight: 8,
                min_count: 4,
                max_count: 8,
            })
        );

        let windswept_savanna = super::biome_generation_settings("windswept_savanna").unwrap();
        assert_eq!(windswept_savanna.biome, "minecraft:windswept_savanna");
        assert!(super::biome_has_placed_feature(
            windswept_savanna,
            "minecraft:trees_windswept_savanna"
        ));
        assert!(super::biome_has_placed_feature(
            windswept_savanna,
            "minecraft:patch_grass_normal"
        ));
        assert!(!super::biome_has_placed_feature(
            windswept_savanna,
            "minecraft:flower_warm"
        ));
        assert_eq!(
            super::biome_spawns_for_category(windswept_savanna, "creature"),
            super::SAVANNA_CREATURE_SPAWNS
        );

        let taiga = super::biome_generation_settings("taiga").unwrap();
        assert_eq!(taiga.biome, "minecraft:taiga");
        assert!(super::biome_has_placed_feature(
            taiga,
            "minecraft:patch_large_fern"
        ));
        assert!(super::biome_has_placed_feature(
            taiga,
            "minecraft:trees_taiga"
        ));
        assert!(super::biome_has_placed_feature(
            taiga,
            "minecraft:patch_berry_common"
        ));
        assert!(super::biome_has_placed_feature(
            taiga,
            "minecraft:brown_mushroom_taiga"
        ));
        assert!(!super::biome_has_placed_feature(
            taiga,
            "minecraft:trees_savanna"
        ));
        assert_eq!(
            super::biome_spawns_for_category(taiga, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:fox",
                weight: 8,
                min_count: 2,
                max_count: 4,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(taiga, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );

        let snowy_taiga = super::biome_generation_settings("snowy_taiga").unwrap();
        assert_eq!(snowy_taiga.biome, "minecraft:snowy_taiga");
        assert!(super::biome_has_placed_feature(
            snowy_taiga,
            "minecraft:patch_berry_rare"
        ));
        assert!(!super::biome_has_placed_feature(
            snowy_taiga,
            "minecraft:patch_berry_common"
        ));
        assert_eq!(
            super::biome_spawns_for_category(snowy_taiga, "creature"),
            super::TAIGA_CREATURE_SPAWNS
        );

        let old_growth_pine = super::biome_generation_settings("old_growth_pine_taiga").unwrap();
        assert_eq!(old_growth_pine.biome, "minecraft:old_growth_pine_taiga");
        assert!(super::biome_has_placed_feature(
            old_growth_pine,
            "minecraft:forest_rock"
        ));
        assert!(super::biome_has_placed_feature(
            old_growth_pine,
            "minecraft:trees_old_growth_pine_taiga"
        ));
        assert!(super::biome_has_placed_feature(
            old_growth_pine,
            "minecraft:brown_mushroom_old_growth"
        ));
        assert!(!super::biome_has_placed_feature(
            old_growth_pine,
            "minecraft:trees_taiga"
        ));
        assert_eq!(
            super::biome_spawns_for_category(old_growth_pine, "monster")[1],
            MobSpawnerDataModel {
                entity_type: "minecraft:zombie",
                weight: 100,
                min_count: 4,
                max_count: 4,
            }
        );
        assert_eq!(
            super::biome_spawns_for_category(old_growth_pine, "monster")[2],
            MobSpawnerDataModel {
                entity_type: "minecraft:zombie_villager",
                weight: 25,
                min_count: 1,
                max_count: 1,
            }
        );

        let old_growth_spruce =
            super::biome_generation_settings("old_growth_spruce_taiga").unwrap();
        assert_eq!(old_growth_spruce.biome, "minecraft:old_growth_spruce_taiga");
        assert!(super::biome_has_placed_feature(
            old_growth_spruce,
            "minecraft:trees_old_growth_spruce_taiga"
        ));
        assert!(!super::biome_has_placed_feature(
            old_growth_spruce,
            "minecraft:trees_old_growth_pine_taiga"
        ));
        assert_eq!(
            super::biome_spawns_for_category(old_growth_spruce, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );

        let snowy = super::biome_generation_settings("snowy_plains").unwrap();
        assert_eq!(snowy.biome, "minecraft:snowy_plains");
        assert_eq!(snowy.creature_spawn_probability, 0.07);
        assert!(super::biome_has_placed_feature(
            snowy,
            "minecraft:trees_snowy"
        ));
        assert!(super::biome_has_placed_feature(
            snowy,
            "minecraft:patch_grass_badlands"
        ));
        assert!(!super::biome_has_placed_feature(
            snowy,
            "minecraft:trees_taiga"
        ));
        assert_eq!(
            super::biome_spawns_for_category(snowy, "creature"),
            super::SNOWY_PLAINS_CREATURE_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(snowy, "monster")[4],
            MobSpawnerDataModel {
                entity_type: "minecraft:skeleton",
                weight: 20,
                min_count: 4,
                max_count: 4,
            }
        );
        assert_eq!(
            super::biome_spawns_for_category(snowy, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:stray",
                weight: 80,
                min_count: 4,
                max_count: 4,
            })
        );

        let ice_spikes = super::biome_generation_settings("ice_spikes").unwrap();
        assert_eq!(ice_spikes.biome, "minecraft:ice_spikes");
        assert_eq!(ice_spikes.creature_spawn_probability, 0.07);
        assert!(super::biome_has_placed_feature(
            ice_spikes,
            "minecraft:ice_spike"
        ));
        assert!(super::biome_has_placed_feature(
            ice_spikes,
            "minecraft:ice_patch"
        ));
        assert_eq!(
            super::biome_spawns_for_category(ice_spikes, "monster"),
            super::SNOWY_PLAINS_MONSTER_SPAWNS
        );

        let jungle = super::biome_generation_settings("jungle").unwrap();
        assert_eq!(jungle.biome, "minecraft:jungle");
        assert!(super::biome_has_placed_feature(
            jungle,
            "minecraft:bamboo_light"
        ));
        assert!(super::biome_has_placed_feature(
            jungle,
            "minecraft:trees_jungle"
        ));
        assert!(super::biome_has_placed_feature(jungle, "minecraft:vines"));
        assert!(super::biome_has_placed_feature(
            jungle,
            "minecraft:patch_melon"
        ));
        assert!(!super::biome_has_placed_feature(
            jungle,
            "minecraft:trees_taiga"
        ));
        assert_eq!(
            super::biome_spawns_for_category(jungle, "creature")[4],
            MobSpawnerDataModel {
                entity_type: "minecraft:chicken",
                weight: 10,
                min_count: 4,
                max_count: 4,
            }
        );
        assert_eq!(
            super::biome_spawns_for_category(jungle, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:panda",
                weight: 1,
                min_count: 1,
                max_count: 2,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(jungle, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:ocelot",
                weight: 2,
                min_count: 1,
                max_count: 3,
            })
        );

        let sparse_jungle = super::biome_generation_settings("sparse_jungle").unwrap();
        assert_eq!(sparse_jungle.biome, "minecraft:sparse_jungle");
        assert!(super::biome_has_placed_feature(
            sparse_jungle,
            "minecraft:trees_sparse_jungle"
        ));
        assert!(super::biome_has_placed_feature(
            sparse_jungle,
            "minecraft:patch_melon_sparse"
        ));
        assert!(!super::biome_has_placed_feature(
            sparse_jungle,
            "minecraft:trees_jungle"
        ));
        assert_eq!(
            super::biome_spawns_for_category(sparse_jungle, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:wolf",
                weight: 8,
                min_count: 2,
                max_count: 4,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(sparse_jungle, "monster"),
            super::FOREST_MONSTER_SPAWNS
        );

        let bamboo_jungle = super::biome_generation_settings("bamboo_jungle").unwrap();
        assert_eq!(bamboo_jungle.biome, "minecraft:bamboo_jungle");
        assert!(super::biome_has_placed_feature(
            bamboo_jungle,
            "minecraft:bamboo"
        ));
        assert!(super::biome_has_placed_feature(
            bamboo_jungle,
            "minecraft:bamboo_vegetation"
        ));
        assert!(!super::biome_has_placed_feature(
            bamboo_jungle,
            "minecraft:bamboo_light"
        ));
        assert_eq!(
            super::biome_spawns_for_category(bamboo_jungle, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:panda",
                weight: 80,
                min_count: 1,
                max_count: 2,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(bamboo_jungle, "monster"),
            super::JUNGLE_MONSTER_SPAWNS
        );

        let swamp = super::biome_generation_settings("swamp").unwrap();
        assert_eq!(swamp.biome, "minecraft:swamp");
        assert!(super::biome_has_placed_feature(
            swamp,
            "minecraft:fossil_upper"
        ));
        assert!(super::biome_has_placed_feature(
            swamp,
            "minecraft:trees_swamp"
        ));
        assert!(super::biome_has_placed_feature(
            swamp,
            "minecraft:patch_waterlily"
        ));
        assert!(super::biome_has_placed_feature(
            swamp,
            "minecraft:seagrass_swamp"
        ));
        assert!(!super::biome_has_placed_feature(
            swamp,
            "minecraft:disk_sand"
        ));
        assert_eq!(
            super::biome_spawns_for_category(swamp, "creature").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:frog",
                weight: 10,
                min_count: 2,
                max_count: 5,
            })
        );
        assert_eq!(
            super::biome_spawns_for_category(swamp, "monster")[3],
            MobSpawnerDataModel {
                entity_type: "minecraft:skeleton",
                weight: 70,
                min_count: 4,
                max_count: 4,
            }
        );
        assert_eq!(
            super::biome_spawns_for_category(swamp, "monster")[8],
            MobSpawnerDataModel {
                entity_type: "minecraft:slime",
                weight: 1,
                min_count: 1,
                max_count: 1,
            }
        );
        assert_eq!(
            super::biome_spawns_for_category(swamp, "monster").last(),
            Some(&MobSpawnerDataModel {
                entity_type: "minecraft:bogged",
                weight: 30,
                min_count: 4,
                max_count: 4,
            })
        );

        let mangrove = super::biome_generation_settings("mangrove_swamp").unwrap();
        assert_eq!(mangrove.biome, "minecraft:mangrove_swamp");
        assert!(super::biome_has_placed_feature(
            mangrove,
            "minecraft:fossil_lower"
        ));
        assert!(super::biome_has_placed_feature(
            mangrove,
            "minecraft:trees_mangrove"
        ));
        assert!(super::biome_has_placed_feature(
            mangrove,
            "minecraft:disk_grass"
        ));
        assert!(super::biome_has_placed_feature(
            mangrove,
            "minecraft:seagrass_swamp"
        ));
        assert!(!super::biome_has_placed_feature(
            mangrove,
            "minecraft:trees_swamp"
        ));
        assert_eq!(
            super::biome_spawns_for_category(mangrove, "creature"),
            super::MANGROVE_SWAMP_CREATURE_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(mangrove, "monster"),
            super::SWAMP_MONSTER_SPAWNS
        );
        assert_eq!(
            super::biome_spawns_for_category(mangrove, "water_ambient"),
            super::MANGROVE_SWAMP_WATER_AMBIENT_SPAWNS
        );
    }

    #[test]
    fn feature_sorter_builds_step_order_and_index_mapping_like_vanilla() {
        let plains = super::biome_generation_settings("plains").unwrap();
        let forest = super::biome_generation_settings("forest").unwrap();
        let sorted =
            super::build_features_per_step(&[plains.feature_steps, forest.feature_steps], true)
                .unwrap();

        assert_eq!(sorted.len(), 11);
        assert!(sorted[0].features.is_empty());
        assert_eq!(
            sorted[1].feature_names(),
            vec![
                "minecraft:lake_lava_underground",
                "minecraft:lake_lava_surface"
            ]
        );
        assert!(sorted[6]
            .feature_names()
            .contains(&"minecraft:ore_diamond_buried"));
        assert!(sorted[9]
            .feature_names()
            .contains(&"minecraft:trees_plains"));
        assert!(sorted[9]
            .feature_names()
            .contains(&"minecraft:trees_birch_and_oak_leaf_litter"));
        assert!(
            super::FeatureSorterData {
                feature_index: 99,
                step: 1,
                feature: "minecraft:earlier_step",
            } < super::FeatureSorterData {
                feature_index: 0,
                step: 9,
                feature: "minecraft:later_step",
            },
            "FeatureSorter comparator must match Java: step first, then feature index"
        );
        let trees_plains_index = sorted[9]
            .index_mapping("minecraft:trees_plains")
            .expect("trees_plains should have an index in the vegetal decoration step");
        assert_eq!(
            sorted[9].features[trees_plains_index].feature,
            "minecraft:trees_plains"
        );
        assert_eq!(sorted[9].index_mapping("minecraft:missing"), None);
    }

    #[test]
    fn biome_decoration_feature_plan_uses_possible_biomes_sorted_indices_and_feature_seeds() {
        let plains = super::biome_generation_settings("plains").unwrap();
        let forest = super::biome_generation_settings("forest").unwrap();
        let sorted =
            super::build_features_per_step(&[plains.feature_steps, forest.feature_steps], true)
                .unwrap();
        let plan = super::biome_decoration_feature_plan(
            12_345,
            4,
            -7,
            -4,
            &sorted,
            &[plains.feature_steps],
        );

        assert_eq!(
            plan.origin,
            BlockPos {
                x: 64,
                y: -64,
                z: -112
            }
        );
        assert_eq!(
            plan.decoration_seed,
            crate::random_source::decoration_seed(
                12_345,
                64,
                -112,
                crate::random_source::RandomAlgorithm::Xoroshiro
            )
        );
        assert!(plan.feature_calls.windows(2).all(|calls| (
            calls[0].step_index,
            calls[0].global_feature_index
        ) <= (
            calls[1].step_index,
            calls[1].global_feature_index
        )));

        let plains_tree_call = plan
            .feature_calls
            .iter()
            .find(|call| call.feature == "minecraft:trees_plains")
            .expect("plains trees should be planned for plains biome decoration");
        assert_eq!(
            plains_tree_call.global_feature_index,
            sorted[plains_tree_call.step_index]
                .index_mapping("minecraft:trees_plains")
                .unwrap()
        );
        assert_eq!(
            plains_tree_call.seed,
            crate::random_source::feature_seed(
                plan.decoration_seed,
                plains_tree_call.global_feature_index as i32,
                plains_tree_call.step_index as i32
            )
        );
        assert!(!plan
            .feature_calls
            .iter()
            .any(|call| call.feature == "minecraft:trees_birch_and_oak_leaf_litter"));

        let mixed_plan = super::biome_decoration_feature_plan(
            12_345,
            4,
            -7,
            -4,
            &sorted,
            &[plains.feature_steps, forest.feature_steps],
        );
        assert!(mixed_plan
            .feature_calls
            .iter()
            .any(|call| call.feature == "minecraft:trees_birch_and_oak_leaf_litter"));
    }

    #[test]
    fn decoration_seed_for_fixed_biome_chunk_and_step_matches_vanilla() {
        let plains = super::biome_generation_settings("plains").unwrap();
        let sorted = super::build_features_per_step(&[plains.feature_steps], true).unwrap();
        let plan = super::biome_decoration_feature_plan(
            12_345,
            4,
            -7,
            -4,
            &sorted,
            &[plains.feature_steps],
        );
        let trees = plan
            .feature_calls
            .iter()
            .find(|call| call.feature == "minecraft:trees_plains")
            .expect("plains decoration should schedule trees_plains");

        assert_eq!(plan.decoration_seed, -6_006_185_048_957_774_615);
        assert_eq!(
            trees.step_index,
            GenerationDecorationStep::VegetalDecoration as usize
        );
        assert_eq!(trees.global_feature_index, 3);
        assert_eq!(trees.seed, -6_006_185_048_957_684_612);
    }

    #[test]
    fn biome_decoration_structure_calls_use_per_step_indices_before_features() {
        let decoration_seed = crate::random_source::decoration_seed(
            12_345,
            64,
            -112,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        let calls = super::biome_decoration_structure_calls(
            decoration_seed,
            5,
            &[
                &[],
                &["minecraft:mineshaft", "minecraft:village"],
                &[],
                &["minecraft:stronghold"],
            ],
        );

        assert_eq!(
            calls,
            vec![
                super::BiomeDecorationStructureCall {
                    step_index: 1,
                    step_structure_index: 0,
                    structure: "minecraft:mineshaft",
                    seed: crate::random_source::feature_seed(decoration_seed, 0, 1),
                },
                super::BiomeDecorationStructureCall {
                    step_index: 1,
                    step_structure_index: 1,
                    structure: "minecraft:village",
                    seed: crate::random_source::feature_seed(decoration_seed, 1, 1),
                },
                super::BiomeDecorationStructureCall {
                    step_index: 3,
                    step_structure_index: 0,
                    structure: "minecraft:stronghold",
                    seed: crate::random_source::feature_seed(decoration_seed, 0, 3),
                },
            ]
        );
    }

    #[test]
    fn feature_sorter_reports_order_cycles() {
        static SOURCE_A: &[&[&str]] = &[&["minecraft:a", "minecraft:b"]];
        static SOURCE_B: &[&[&str]] = &[&["minecraft:b", "minecraft:a"]];
        static SOURCE_C: &[&[&str]] = &[&["minecraft:c", "minecraft:d"]];

        assert_eq!(
            super::build_features_per_step(&[SOURCE_A, SOURCE_B], false).unwrap_err(),
            "Feature order cycle found".to_string()
        );
        assert_eq!(
            super::build_features_per_step(&[SOURCE_A, SOURCE_B], true).unwrap_err(),
            "Feature order cycle found, involved sources: 2".to_string()
        );
        assert_eq!(
            super::build_features_per_step_with_source_ids(
                &[
                    super::FeatureSorterSourceModel {
                        id: "source_a",
                        feature_steps: SOURCE_A,
                    },
                    super::FeatureSorterSourceModel {
                        id: "source_b",
                        feature_steps: SOURCE_B,
                    },
                    super::FeatureSorterSourceModel {
                        id: "irrelevant_source_c",
                        feature_steps: SOURCE_C,
                    },
                ],
                true,
            )
            .unwrap_err(),
            "Feature order cycle found, involved sources: [source_a, source_b]".to_string()
        );
    }

    #[test]
    fn feature_placement_support_registries_match_vanilla_type_bootstraps() {
        assert_eq!(
            WORLDGEN_TYPE_REGISTRIES
                .iter()
                .map(|registry| (registry.id, registry.entries.len()))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:height_provider_type", 6),
                ("minecraft:block_predicate_type", 13),
                ("minecraft:placement_modifier_type", 14),
                ("minecraft:trunk_placer_type", 9),
                ("minecraft:foliage_placer_type", 11),
                ("minecraft:block_state_provider_type", 8),
                ("minecraft:tree_decorator_type", 10),
                ("minecraft:feature_size_type", 2),
                ("minecraft:root_placer_type", 1),
            ]
        );
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:placement_modifier_type")
            .unwrap()
            .entries
            .contains(&"minecraft:environment_scan"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:tree_decorator_type")
            .unwrap()
            .entries
            .contains(&"minecraft:creaking_heart"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:trunk_placer_type")
            .unwrap()
            .entries
            .contains(&"minecraft:upwards_branching_trunk_placer"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:block_state_provider_type")
            .unwrap()
            .entries
            .contains(&"minecraft:rule_based_state_provider"));
        assert_eq!(
            super::block_state_provider_type("simple_state_provider"),
            Some("minecraft:simple_state_provider")
        );
        assert_eq!(
            super::block_state_provider_type("minecraft:weighted_state_provider"),
            Some("minecraft:weighted_state_provider")
        );
        assert_eq!(
            super::block_state_provider_type("minecraft:rotated_block_provider"),
            Some("minecraft:rotated_block_provider")
        );
        assert_eq!(super::block_state_provider_type("missing"), None);
        assert_eq!(
            super::tree_placement_filter_sapling("minecraft:trees_birch"),
            Some("minecraft:birch_sapling")
        );
        assert_eq!(
            super::tree_placement_filter_sapling("minecraft:trees_birch_and_oak_leaf_litter"),
            None,
            "forest leaf-litter trees use vanilla treePlacement without a sapling BlockPredicateFilter"
        );
        assert_eq!(
            super::block_state_provider_sample(
                &BlockStateProviderModel::Simple("minecraft:oak_log"),
                99
            ),
            Some("minecraft:oak_log")
        );
        let weighted = BlockStateProviderModel::Weighted(vec![
            WeightedBlockState {
                state: "minecraft:stone",
                weight: 2,
            },
            WeightedBlockState {
                state: "minecraft:andesite",
                weight: 1,
            },
        ]);
        assert_eq!(
            super::block_state_provider_sample(&weighted, 0),
            Some("minecraft:stone")
        );
        assert_eq!(
            super::block_state_provider_sample(&weighted, 2),
            Some("minecraft:andesite")
        );
        assert_eq!(
            super::block_state_provider_sample(
                &BlockStateProviderModel::Weighted(vec![WeightedBlockState {
                    state: "minecraft:air",
                    weight: 0,
                }]),
                0,
            ),
            None
        );
        let rotated = BlockStateProviderModel::RotatedBlock("minecraft:hay_block");
        assert_eq!(
            super::block_state_provider_sample(&rotated, 0),
            Some("minecraft:hay_block[axis=x]")
        );
        assert_eq!(
            super::block_state_provider_sample(&rotated, 1),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::block_state_provider_sample(&rotated, 2),
            Some("minecraft:hay_block[axis=z]")
        );
        let rotated_log = BlockStateProviderModel::RotatedBlock("minecraft:oak_log");
        assert_eq!(
            super::block_state_provider_sample(&rotated_log, 0),
            Some("minecraft:oak_log[axis=x]")
        );
        assert_eq!(
            super::block_state_provider_sample(&rotated_log, 1),
            Some("minecraft:oak_log")
        );
        assert_eq!(
            super::block_state_provider_sample(&rotated_log, 2),
            Some("minecraft:oak_log[axis=z]")
        );
        let rotated_basalt = BlockStateProviderModel::RotatedBlock("minecraft:basalt");
        assert_eq!(
            super::block_state_provider_sample(&rotated_basalt, 0),
            Some("minecraft:basalt[axis=x]")
        );
        assert_eq!(
            super::block_state_provider_sample(&rotated_basalt, 1),
            Some("minecraft:basalt")
        );
        assert_eq!(
            super::block_state_provider_sample(&rotated_basalt, 2),
            Some("minecraft:basalt[axis=z]")
        );
        let randomized_cave_vines = BlockStateProviderModel::RandomizedInt {
            source: Box::new(BlockStateProviderModel::Weighted(vec![
                WeightedBlockState {
                    state: "minecraft:cave_vines[age=0,berries=false]",
                    weight: 4,
                },
                WeightedBlockState {
                    state: "minecraft:cave_vines[age=0,berries=true]",
                    weight: 1,
                },
            ])),
            property: "age",
            min_inclusive: 23,
            max_inclusive: 25,
        };
        assert_eq!(
            super::block_state_provider_sample(&randomized_cave_vines, 0),
            Some("minecraft:cave_vines[age=23,berries=false]")
        );
        assert_eq!(
            super::block_state_provider_sample(&randomized_cave_vines, 4),
            Some("minecraft:cave_vines[age=24,berries=true]")
        );
        let randomized_propagule = BlockStateProviderModel::RandomizedInt {
            source: Box::new(BlockStateProviderModel::Simple(
                "minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]",
            )),
            property: "age",
            min_inclusive: 0,
            max_inclusive: 4,
        };
        assert_eq!(
            super::block_state_provider_sample(&randomized_propagule, 3),
            Some("minecraft:mangrove_propagule[age=3,hanging=true,stage=0,waterlogged=false]")
        );
        let missing_property = BlockStateProviderModel::RandomizedInt {
            source: Box::new(BlockStateProviderModel::Simple("minecraft:stone")),
            property: "age",
            min_inclusive: 0,
            max_inclusive: 4,
        };
        assert_eq!(
            super::block_state_provider_sample(&missing_property, 3),
            Some("minecraft:stone")
        );
        let rule_based_disk = BlockStateProviderModel::RuleBased {
            fallback: Some(Box::new(BlockStateProviderModel::Simple("minecraft:sand"))),
            rules: vec![RuleBasedBlockStateProviderRule {
                if_true: BlockPredicate::MatchingBlocks {
                    blocks: &["minecraft:air"],
                },
                then: Box::new(BlockStateProviderModel::Simple("minecraft:sandstone")),
            }],
        };
        assert_eq!(
            super::block_state_provider_sample_in_context(
                &rule_based_disk,
                0,
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:air",
                    fluid: "minecraft:empty",
                    solid: false,
                    replaceable: true,
                    unobstructed: true,
                },
                64,
                "minecraft:air",
            ),
            Some("minecraft:sandstone")
        );
        assert_eq!(
            super::block_state_provider_sample_in_context(
                &rule_based_disk,
                0,
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:dirt",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
                64,
                "minecraft:dirt",
            ),
            Some("minecraft:sand")
        );
        let mut disk_random = crate::random_source::RandomSourceKind::new(
            1234,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        let mut expected_disk_random = disk_random;
        assert_eq!(
            super::block_state_provider_sample_in_context_with_random(
                &rule_based_disk,
                &mut disk_random,
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:dirt",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
                64,
                "minecraft:dirt",
            ),
            Some("minecraft:sand")
        );
        assert_eq!(
            super::random_next_i32_bound(&mut disk_random, 10_000),
            super::random_next_i32_bound(&mut expected_disk_random, 10_000),
            "Java SimpleStateProvider and simple RuleBasedStateProvider branches do not consume RandomSource"
        );
        let no_fallback = BlockStateProviderModel::RuleBased {
            fallback: None,
            rules: vec![RuleBasedBlockStateProviderRule {
                if_true: BlockPredicate::MatchingBlocks {
                    blocks: &["minecraft:air"],
                },
                then: Box::new(BlockStateProviderModel::Simple("minecraft:dirt")),
            }],
        };
        assert_eq!(
            super::block_state_provider_sample_in_context(
                &no_fallback,
                0,
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:stone",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
                64,
                "minecraft:stone",
            ),
            Some("minecraft:stone")
        );
        let flower_forest_noise = BlockStateProviderModel::Noise {
            states: vec![
                "minecraft:dandelion",
                "minecraft:poppy",
                "minecraft:allium",
                "minecraft:azure_bluet",
                "minecraft:red_tulip",
                "minecraft:orange_tulip",
                "minecraft:white_tulip",
                "minecraft:pink_tulip",
                "minecraft:oxeye_daisy",
                "minecraft:cornflower",
                "minecraft:lily_of_the_valley",
            ],
        };
        assert_eq!(
            super::block_state_provider_sample_with_noise_value(&flower_forest_noise, 0, -1.5),
            Some("minecraft:dandelion")
        );
        assert_eq!(
            super::block_state_provider_sample_with_noise_value(&flower_forest_noise, 0, 0.0),
            Some("minecraft:orange_tulip")
        );
        assert_eq!(
            super::block_state_provider_sample_with_noise_value(&flower_forest_noise, 0, 1.5),
            Some("minecraft:lily_of_the_valley")
        );
        let plains_flower_threshold = BlockStateProviderModel::NoiseThreshold {
            threshold: -0.8,
            high_chance: 0.33333334,
            default_state: "minecraft:dandelion",
            low_states: vec![
                "minecraft:orange_tulip",
                "minecraft:red_tulip",
                "minecraft:pink_tulip",
                "minecraft:white_tulip",
            ],
            high_states: vec![
                "minecraft:poppy",
                "minecraft:azure_bluet",
                "minecraft:oxeye_daisy",
                "minecraft:cornflower",
            ],
        };
        assert_eq!(
            super::block_state_provider_sample_with_noise_value(&plains_flower_threshold, 2, -0.9),
            Some("minecraft:pink_tulip")
        );
        assert_eq!(
            super::block_state_provider_sample_with_noise_value(
                &plains_flower_threshold,
                200_000,
                -0.7
            ),
            Some("minecraft:poppy")
        );
        assert_eq!(
            super::block_state_provider_sample_with_noise_value(
                &plains_flower_threshold,
                900_000,
                -0.7
            ),
            Some("minecraft:dandelion")
        );
        let meadow_dual_noise = BlockStateProviderModel::DualNoise {
            variety_min: 1,
            variety_max: 3,
            states: vec![
                "minecraft:tall_grass[half=lower]",
                "minecraft:allium",
                "minecraft:poppy",
                "minecraft:azure_bluet",
                "minecraft:dandelion",
                "minecraft:cornflower",
                "minecraft:oxeye_daisy",
                "minecraft:short_grass",
            ],
        };
        assert_eq!(
            super::block_state_provider_sample_dual_noise_values(
                &meadow_dual_noise,
                -1.0,
                &[1.0],
                0.0,
            ),
            Some("minecraft:short_grass")
        );
        assert_eq!(
            super::block_state_provider_sample_dual_noise_values(
                &meadow_dual_noise,
                1.0,
                &[-1.0, -0.5, 0.0, 1.0],
                0.9999,
            ),
            Some("minecraft:short_grass")
        );
        assert_eq!(
            super::block_state_provider_sample_dual_noise_values(
                &meadow_dual_noise,
                0.0,
                &[-1.0, 1.0],
                -1.0,
            ),
            Some("minecraft:tall_grass[half=lower]")
        );
        let flower_provider = BlockStateProviderModel::Simple("minecraft:dandelion");
        let simple_config = super::SimpleBlockConfigurationModel {
            to_place: flower_provider,
            schedule_tick: false,
        };
        let mut simple_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &simple_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                &mut simple_random,
            ),
            Some(super::SimpleBlockPlacementPlan {
                state: "minecraft:dandelion",
                upper_state: None,
                schedule_tick: false,
            })
        );
        let mut replace_leaf_litter_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &simple_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:leaf_litter",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                &mut replace_leaf_litter_random,
            ),
            Some(super::SimpleBlockPlacementPlan {
                state: "minecraft:dandelion",
                upper_state: None,
                schedule_tick: false,
            })
        );
        let mut bad_support_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &simple_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:stone",
                    above_block: "minecraft:air",
                },
                &mut bad_support_random,
            ),
            None
        );

        let sunflower_provider = BlockStateProviderModel::Simple("minecraft:sunflower");
        let sunflower_config = super::SimpleBlockConfigurationModel {
            to_place: sunflower_provider,
            schedule_tick: true,
        };
        let mut sunflower_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &sunflower_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                &mut sunflower_random,
            ),
            Some(super::SimpleBlockPlacementPlan {
                state: "minecraft:sunflower",
                upper_state: Some("minecraft:sunflower"),
                schedule_tick: true,
            })
        );
        let mut obstructed_sunflower_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &sunflower_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:oak_leaves",
                },
                &mut obstructed_sunflower_random,
            ),
            None
        );

        let replace_targets = [
            super::TargetBlockStateModel {
                target: super::RuleTestModel::BlockMatch("minecraft:stone"),
                state: "minecraft:granite",
            },
            super::TargetBlockStateModel {
                target: super::RuleTestModel::TagMatch(&[
                    "minecraft:dirt",
                    "minecraft:grass_block",
                ]),
                state: "minecraft:coarse_dirt",
            },
            super::TargetBlockStateModel {
                target: super::RuleTestModel::AlwaysTrue,
                state: "minecraft:air",
            },
        ];
        assert_eq!(
            super::replace_block_result("minecraft:stone", &replace_targets),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::replace_block_result("minecraft:grass_block", &replace_targets),
            Some("minecraft:coarse_dirt")
        );
        assert_eq!(
            super::replace_block_result("minecraft:deepslate", &replace_targets),
            Some("minecraft:air")
        );
        assert_eq!(super::replace_block_result("minecraft:stone", &[]), None);
        assert!(super::block_matches_tag(
            "minecraft:stone",
            "minecraft:stone_ore_replaceables"
        ));
        assert!(super::block_matches_tag(
            "minecraft:andesite",
            "minecraft:stone_ore_replaceables"
        ));
        assert!(!super::block_matches_tag(
            "minecraft:tuff",
            "minecraft:stone_ore_replaceables"
        ));
        assert!(super::block_matches_tag(
            "minecraft:deepslate",
            "minecraft:deepslate_ore_replaceables"
        ));
        assert!(super::block_matches_tag(
            "minecraft:tuff",
            "minecraft:deepslate_ore_replaceables"
        ));
        assert!(super::block_matches_tag(
            "minecraft:granite",
            "minecraft:base_stone_overworld"
        ));
        assert!(super::block_matches_tag(
            "minecraft:tuff",
            "minecraft:base_stone_overworld"
        ));
        assert!(!super::block_matches_tag(
            "minecraft:netherrack",
            "minecraft:base_stone_overworld"
        ));
        assert!(super::block_matches_tag(
            "minecraft:blackstone",
            "minecraft:base_stone_nether"
        ));
        assert!(super::rule_test_matches(
            super::RuleTestModel::BlockTag("minecraft:base_stone_overworld"),
            "minecraft:deepslate"
        ));
        assert!(!super::rule_test_matches(
            super::RuleTestModel::BlockTag("minecraft:base_stone_overworld"),
            "minecraft:netherrack"
        ));

        let ore_config = super::OreConfigurationModel {
            target_states: vec![super::TargetBlockStateModel {
                target: super::RuleTestModel::BlockMatch("minecraft:stone"),
                state: "minecraft:iron_ore",
            }],
            size: 9,
            discard_chance_on_air_exposure: 0.5,
        };
        let tuff_config = super::configured_ore_configuration("minecraft:ore_tuff").unwrap();
        assert_eq!(tuff_config.size, 64);
        assert_eq!(tuff_config.discard_chance_on_air_exposure, 0.0);
        assert_eq!(tuff_config.target_states.len(), 1);
        assert_eq!(tuff_config.target_states[0].state, "minecraft:tuff");
        assert!(super::rule_test_matches(
            tuff_config.target_states[0].target,
            "minecraft:deepslate"
        ));
        let diamond_config =
            super::configured_ore_configuration("minecraft:ore_diamond_large").unwrap();
        assert_eq!(diamond_config.size, 12);
        assert_eq!(diamond_config.discard_chance_on_air_exposure, 0.7);
        assert_eq!(
            diamond_config
                .target_states
                .iter()
                .map(|target| target.state)
                .collect::<Vec<_>>(),
            vec!["minecraft:diamond_ore", "minecraft:deepslate_diamond_ore"]
        );
        assert!(super::rule_test_matches(
            diamond_config.target_states[0].target,
            "minecraft:granite"
        ));
        assert!(super::rule_test_matches(
            diamond_config.target_states[1].target,
            "minecraft:tuff"
        ));
        let debris_config =
            super::configured_ore_configuration("minecraft:ore_ancient_debris_large").unwrap();
        assert_eq!(debris_config.size, 3);
        assert_eq!(debris_config.discard_chance_on_air_exposure, 1.0);
        assert!(super::rule_test_matches(
            debris_config.target_states[0].target,
            "minecraft:blackstone"
        ));
        assert_eq!(
            super::configured_ore_configuration("minecraft:not_ore"),
            None
        );

        let ore_target = ore_config.target_states[0];
        assert!(super::ore_should_skip_air_check(0.0, 0.0));
        assert!(!super::ore_should_skip_air_check(1.0, 1.0));
        assert!(!super::ore_should_skip_air_check(0.5, 0.49));
        assert!(super::ore_should_skip_air_check(0.5, 0.5));
        assert!(super::ore_can_place(
            "minecraft:stone",
            false,
            &ore_config,
            ore_target,
            0.0,
        ));
        assert!(!super::ore_can_place(
            "minecraft:dirt",
            false,
            &ore_config,
            ore_target,
            1.0,
        ));
        assert!(!super::ore_can_place(
            "minecraft:stone",
            true,
            &ore_config,
            ore_target,
            0.0,
        ));
        assert_eq!(
            super::scattered_ore_offset(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                9,
                [(1.0, 0.0), (0.0, 1.0), (0.75, 0.25)],
            ),
            BlockPos {
                x: 17,
                y: 13,
                z: 34,
            }
        );
        assert_eq!(
            super::scattered_ore_attempt(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                2,
                [(1.0, 0.0), (0.0, 1.0), (0.5, 0.5)],
                "minecraft:stone",
                false,
                &ore_config,
                0.0,
            ),
            Some(super::ScatteredOreAttempt {
                pos: BlockPos {
                    x: 12,
                    y: 18,
                    z: 30,
                },
                state: "minecraft:iron_ore",
            })
        );
        let ore_spheres =
            super::ore_vein_spheres(BlockPos { x: 8, y: 32, z: 8 }, 8, 0.0, &[(2, 2)], &[1.0; 8]);
        assert!(!ore_spheres.is_empty());
        assert!(ore_spheres
            .iter()
            .any(|sphere| (sphere.center_z - 8.5).abs() < f64::EPSILON));
        let ore_candidates =
            super::ore_vein_position_candidates(&ore_spheres, 6, 28, 6, 6, 6, 0..384);
        assert!(!ore_candidates.is_empty());
        let boundary_candidates = super::ore_vein_position_candidates(
            &[super::OreVeinSphere {
                center_x: 6.5,
                center_y: 28.5,
                center_z: 12.5,
                radius: 0.51,
            }],
            6,
            28,
            6,
            6,
            6,
            0..384,
        );
        assert!(
            boundary_candidates.contains(&BlockPos { x: 6, y: 28, z: 12 }),
            "OreFeature's Java BitSet expands for inclusive high-z boundary candidates"
        );
        let sampled_candidate = ore_candidates[0];
        assert_eq!(
            ore_candidates
                .iter()
                .filter(|pos| **pos == sampled_candidate)
                .count(),
            1
        );
        let ore_plan = super::ore_placement_plan(
            &ore_config,
            &[
                super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 32, z: 8 },
                    current_block: "minecraft:stone",
                    adjacent_to_air: false,
                    air_check_roll: 0.0,
                },
                super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 33, z: 8 },
                    current_block: "minecraft:stone",
                    adjacent_to_air: true,
                    air_check_roll: 0.0,
                },
                super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 34, z: 8 },
                    current_block: "minecraft:dirt",
                    adjacent_to_air: false,
                    air_check_roll: 1.0,
                },
            ],
        );
        assert_eq!(
            ore_plan,
            vec![super::OrePlacementBlock {
                pos: BlockPos { x: 8, y: 32, z: 8 },
                state: "minecraft:iron_ore",
            }]
        );
        assert_eq!(
            super::aquatic_feature_offset(
                BlockPos {
                    x: 20,
                    y: 60,
                    z: 30
                },
                (7, 3),
                (1, 6)
            ),
            (24, 25)
        );
        assert_eq!(
            super::seagrass_placement_plan(
                BlockPos { x: 1, y: 62, z: 1 },
                "minecraft:water",
                "minecraft:water",
                true,
                0.7,
                0.1,
            ),
            vec![
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 1, y: 62, z: 1 },
                    state: "minecraft:tall_seagrass",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 1, y: 63, z: 1 },
                    state: "minecraft:tall_seagrass[half=upper]",
                },
            ]
        );
        assert_eq!(
            super::seagrass_placement_plan(
                BlockPos { x: 1, y: 62, z: 1 },
                "minecraft:water",
                "minecraft:air",
                true,
                0.7,
                0.1,
            ),
            Vec::new()
        );
        assert_eq!(
            super::sea_pickle_placement_plan(
                BlockPos { x: 2, y: 61, z: 2 },
                "minecraft:water",
                true,
                2,
            ),
            Some(super::AquaticPlacementBlock {
                pos: BlockPos { x: 2, y: 61, z: 2 },
                state: "minecraft:sea_pickle[pickles=3]",
            })
        );
        assert_eq!(
            super::kelp_placement_plan(
                BlockPos { x: 3, y: 50, z: 3 },
                &[true, true, true, true],
                &[true, true, true],
                1,
                &[2],
                false,
            ),
            vec![
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 50, z: 3 },
                    state: "minecraft:kelp_plant",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 51, z: 3 },
                    state: "minecraft:kelp_plant",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 52, z: 3 },
                    state: "minecraft:kelp[age=22]",
                },
            ]
        );
        assert_eq!(
            super::kelp_placement_plan(
                BlockPos { x: 3, y: 50, z: 3 },
                &[true, true, false],
                &[true, true],
                5,
                &[0],
                false,
            ),
            vec![super::AquaticPlacementBlock {
                pos: BlockPos { x: 3, y: 50, z: 3 },
                state: "minecraft:kelp[age=20]",
            }]
        );
        assert_eq!(
            super::coral_block_placement_plan(
                BlockPos { x: 4, y: 55, z: 4 },
                "minecraft:water",
                "minecraft:water",
                "minecraft:brain_coral_block",
                0.9,
                0.01,
                1,
                &[(super::HorizontalDirection::East, 0.1, true)],
            ),
            vec![
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 4, y: 55, z: 4 },
                    state: "minecraft:brain_coral_block",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 4, y: 56, z: 4 },
                    state: "minecraft:sea_pickle[pickles=2]",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 5, y: 55, z: 4 },
                    state: "minecraft:tube_coral_wall_fan[facing=east]",
                },
            ]
        );
        assert!(super::coral_block_placement_plan(
            BlockPos { x: 4, y: 55, z: 4 },
            "minecraft:stone",
            "minecraft:water",
            "minecraft:brain_coral_block",
            0.0,
            0.0,
            0,
            &[],
        )
        .is_empty());
        let coral_tree = super::coral_tree_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            1,
            &[
                super::HorizontalDirection::North,
                super::HorizontalDirection::East,
            ],
            &[0, 1],
            &[1.0; 10],
        );
        assert!(coral_tree.contains(&BlockPos { x: 0, y: 60, z: 0 }));
        assert!(coral_tree.contains(&BlockPos { x: 0, y: 62, z: -1 }));
        assert!(coral_tree.contains(&BlockPos { x: 1, y: 62, z: 0 }));
        let coral_mushroom = super::coral_mushroom_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            0,
            0,
            0,
            0,
            &[1.0; 128],
        );
        assert!(coral_mushroom.contains(&BlockPos { x: 1, y: 59, z: 1 }));
        assert!(!coral_mushroom.contains(&BlockPos { x: 0, y: 59, z: 0 }));
        let coral_claw = super::coral_claw_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            super::HorizontalDirection::North,
            &[
                super::HorizontalDirection::North,
                super::HorizontalDirection::East,
            ],
            &[0, 0],
            &[0, 0],
            &[1.0; 10],
        );
        assert!(coral_claw.contains(&BlockPos { x: 0, y: 60, z: 0 }));
        assert!(coral_claw.contains(&BlockPos { x: 0, y: 60, z: -1 }));
        assert!(coral_claw.contains(&BlockPos { x: 1, y: 61, z: 0 }));
        let vegetation_config = super::VegetationPatchConfigurationModel {
            replaceable: &["minecraft:dirt", "minecraft:grass_block"],
            ground_state: BlockStateProviderModel::Simple("minecraft:moss_block"),
            vegetation_feature: "minecraft:patch_grass",
            surface: CaveSurface::Floor,
            depth_min: 1,
            depth_max: 2,
            extra_bottom_block_chance: 0.5,
            vertical_range: 5,
            vegetation_chance: 0.75,
            xz_radius_min: 1,
            xz_radius_max: 2,
            extra_edge_column_chance: 0.25,
        };
        assert_eq!(super::vegetation_patch_radius(1, 2, 1), 3);
        assert!(!super::vegetation_patch_should_try_column(
            3, 3, 3, 3, 1.0, 0.0
        ));
        assert!(!super::vegetation_patch_should_try_column(
            3, 0, 3, 3, 0.25, 0.5
        ));
        assert!(super::vegetation_patch_should_try_column(
            3, 0, 3, 3, 0.25, 0.25
        ));
        assert_eq!(super::vegetation_patch_depth(1, 2, 0, 0.5, 0.25), 2);
        assert_eq!(
            super::vegetation_patch_place_ground(
                &vegetation_config,
                BlockPos { x: 5, y: 63, z: 5 },
                &["minecraft:dirt", "minecraft:stone"],
                3,
                0,
            ),
            Some(vec![super::VegetationPatchBlock {
                pos: BlockPos { x: 5, y: 63, z: 5 },
                state: "minecraft:moss_block",
            }])
        );
        let vegetation_plan = super::vegetation_patch_plan(
            &vegetation_config,
            &[super::VegetationPatchGroundColumn {
                surface_pos: BlockPos { x: 5, y: 64, z: 5 },
                ground_start: BlockPos { x: 5, y: 63, z: 5 },
                depth: 1,
            }],
            &[&["minecraft:dirt"]],
            &[0.25],
        );
        assert_eq!(
            vegetation_plan.ground,
            vec![super::VegetationPatchBlock {
                pos: BlockPos { x: 5, y: 63, z: 5 },
                state: "minecraft:moss_block",
            }]
        );
        assert_eq!(
            vegetation_plan.vegetation_origins,
            vec![BlockPos { x: 5, y: 65, z: 5 }]
        );
        let lake_config = super::LakeConfigurationModel {
            fluid: BlockStateProviderModel::Simple("minecraft:water"),
            barrier: BlockStateProviderModel::Simple("minecraft:stone"),
        };
        let mut lake_grid = vec![false; 2048];
        lake_grid[super::lake_grid_index(8, 3, 8)] = true;
        assert!(super::lake_is_boundary(&lake_grid, 8, 4, 8));
        assert_eq!(
            super::lake_grid_index(8, 3, 8),
            ((8 * 16 + 8) * 8 + 3) as usize
        );
        let lake_boundary = [
            super::LakeBoundaryBlock {
                x: 8,
                y: 4,
                z: 8,
                state: "minecraft:stone",
                solid: true,
                liquid: false,
                cannot_replace: false,
                should_freeze: true,
            },
            super::LakeBoundaryBlock {
                x: 8,
                y: 2,
                z: 8,
                state: "minecraft:stone",
                solid: true,
                liquid: false,
                cannot_replace: false,
                should_freeze: false,
            },
        ];
        assert!(super::lake_can_place(
            -64,
            70,
            &lake_grid,
            &lake_boundary,
            "minecraft:water"
        ));
        let invalid_lake_boundary = [super::LakeBoundaryBlock {
            x: 8,
            y: 4,
            z: 8,
            state: "minecraft:water",
            solid: false,
            liquid: true,
            cannot_replace: false,
            should_freeze: false,
        }];
        assert!(!super::lake_can_place(
            -64,
            70,
            &lake_grid,
            &invalid_lake_boundary,
            "minecraft:water"
        ));
        let lake_plan = super::lake_placement_plan(
            BlockPos { x: 0, y: 60, z: 0 },
            &lake_config,
            &lake_grid,
            &lake_boundary,
            &[1; 2048],
            true,
        )
        .unwrap();
        assert!(lake_plan.contains(&super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 63, z: 8 },
            state: "minecraft:water",
            schedule_tick: false,
            mark_above_for_post_processing: false,
        }));
        assert!(lake_plan.contains(&super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 64, z: 8 },
            state: "minecraft:stone",
            schedule_tick: false,
            mark_above_for_post_processing: true,
        }));
        assert!(lake_plan.contains(&super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 64, z: 8 },
            state: "minecraft:ice",
            schedule_tick: false,
            mark_above_for_post_processing: false,
        }));
        let fossil_config = super::FossilFeatureConfigurationModel {
            fossil_structures: vec!["minecraft:fossil/spine_1", "minecraft:fossil/skull_1"],
            overlay_structures: vec![
                "minecraft:fossil/spine_1_coal",
                "minecraft:fossil/skull_1_coal",
            ],
            fossil_processors: "minecraft:fossil_rot",
            overlay_processors: "minecraft:fossil_coal",
            max_empty_corners_allowed: 4,
        };
        assert_eq!(super::validate_fossil_config(&fossil_config), Ok(()));
        assert_eq!(
            super::validate_fossil_config(&super::FossilFeatureConfigurationModel {
                fossil_structures: Vec::new(),
                overlay_structures: Vec::new(),
                fossil_processors: "minecraft:fossil_rot",
                overlay_processors: "minecraft:fossil_coal",
                max_empty_corners_allowed: 4,
            }),
            Err("Fossil structure lists need at least one entry")
        );
        assert_eq!(
            super::fossil_rotation(3),
            super::StructureRotation::Counterclockwise90
        );
        assert_eq!(super::fossil_target_y(50, -64, 9), 26);
        assert_eq!(
            super::fossil_low_corner(
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8
            ),
            BlockPos {
                x: 94,
                y: 40,
                z: 196
            }
        );
        assert_eq!(
            super::fossil_placement_plan(
                &fossil_config,
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8,
                50,
                -64,
                1,
                1,
                0,
                4,
            ),
            Some(super::FossilPlacementPlan {
                fossil_structure: "minecraft:fossil/skull_1",
                overlay_structure: "minecraft:fossil/skull_1_coal",
                rotation: super::StructureRotation::Clockwise90,
                target_pos: BlockPos {
                    x: 94,
                    y: 35,
                    z: 196
                },
                fossil_processors: "minecraft:fossil_rot",
                overlay_processors: "minecraft:fossil_coal",
            })
        );
        assert_eq!(
            super::fossil_placement_plan(
                &fossil_config,
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8,
                50,
                -64,
                1,
                1,
                0,
                5,
            ),
            None
        );
        let geode_config = super::GeodeConfigurationModel {
            filling_provider: BlockStateProviderModel::Simple("minecraft:air"),
            inner_layer_provider: BlockStateProviderModel::Simple("minecraft:amethyst_block"),
            alternate_inner_layer_provider: BlockStateProviderModel::Simple(
                "minecraft:budding_amethyst",
            ),
            middle_layer_provider: BlockStateProviderModel::Simple("minecraft:calcite"),
            outer_layer_provider: BlockStateProviderModel::Simple("minecraft:smooth_basalt"),
            inner_placements: &[
                "minecraft:small_amethyst_bud",
                "minecraft:medium_amethyst_bud",
                "minecraft:large_amethyst_bud",
                "minecraft:amethyst_cluster",
            ],
            cannot_replace: &["minecraft:bedrock"],
            invalid_blocks: &["minecraft:water", "minecraft:lava"],
            layers: super::GeodeLayerSettingsModel {
                filling: 1.7,
                inner_layer: 2.2,
                middle_layer: 3.2,
                outer_layer: 4.2,
            },
            crack: super::GeodeCrackSettingsModel {
                generate_crack_chance: 0.95,
                base_crack_size: 2.0,
                crack_point_offset: 2,
            },
            use_potential_placements_chance: 0.35,
            use_alternate_layer0_chance: 0.083,
            placements_require_layer0_alternate: true,
            outer_wall_distance_max: 6,
            invalid_blocks_threshold: 1,
        };
        assert_eq!(super::validate_geode_config(&geode_config), Ok(()));
        assert!(super::geode_can_place(
            &geode_config,
            &["minecraft:stone", "minecraft:water"]
        ));
        assert!(!super::geode_can_place(
            &geode_config,
            &["minecraft:air", "minecraft:water"]
        ));
        let geode_thresholds = super::geode_layer_thresholds(
            geode_config.layers,
            geode_config.crack,
            4,
            geode_config.outer_wall_distance_max,
            0.0,
        );
        assert!(geode_thresholds.inner_air > geode_thresholds.innermost_block_layer);
        assert!(geode_thresholds.innermost_block_layer > geode_thresholds.inner_crust);
        assert!(geode_thresholds.inner_crust > geode_thresholds.outer_crust);
        assert!(super::geode_should_generate_crack(geode_config.crack, 0.94));
        let crack_points = super::geode_crack_points(BlockPos { x: 0, y: 0, z: 0 }, 4, 2);
        assert_eq!(crack_points[0], BlockPos { x: 9, y: 7, z: 9 });
        let distribution_points = [super::GeodeDistributionPoint {
            pos: BlockPos { x: 0, y: 0, z: 0 },
            offset: 1,
        }];
        assert!(
            super::geode_shell_density(BlockPos { x: 0, y: 0, z: 0 }, &distribution_points, 0.0)
                > super::geode_shell_density(
                    BlockPos { x: 16, y: 0, z: 0 },
                    &distribution_points,
                    0.0
                )
        );
        assert_eq!(
            super::geode_layer_for_density(
                geode_thresholds.inner_air + 0.1,
                0.0,
                geode_thresholds,
                false,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::Filling)
        );
        assert_eq!(
            super::geode_layer_for_density(
                (geode_thresholds.innermost_block_layer + geode_thresholds.inner_air) / 2.0,
                0.0,
                geode_thresholds,
                false,
                0.0,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::AlternateInner)
        );
        assert_eq!(
            super::geode_layer_for_density(
                (geode_thresholds.outer_crust + geode_thresholds.inner_crust) / 2.0,
                0.0,
                geode_thresholds,
                false,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::Outer)
        );
        assert_eq!(
            super::geode_layer_for_density(
                (geode_thresholds.outer_crust + geode_thresholds.inner_crust) / 2.0,
                geode_thresholds.crack_size,
                geode_thresholds,
                true,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::CrackAir)
        );
        assert_eq!(
            super::geode_placement_block(
                &geode_config,
                BlockPos { x: 1, y: 2, z: 3 },
                super::GeodeLayer::AlternateInner,
                0,
                0.0,
            ),
            Some(super::GeodePlacementBlock {
                pos: BlockPos { x: 1, y: 2, z: 3 },
                state: "minecraft:budding_amethyst",
                layer: super::GeodeLayer::AlternateInner,
                potential_crystal_source: true,
            })
        );
        assert_eq!(
            super::geode_inner_placement(&geode_config, 3),
            Some("minecraft:amethyst_cluster")
        );
        let iceberg_shape = super::iceberg_shape_model(0.8, 0.25, 0, 2, 0.8, 5, 0.0, 0, 10, 6, 0);
        assert_eq!(iceberg_shape.shape_ellipse_a, 11);
        assert_eq!(iceberg_shape.shape_ellipse_c, 5);
        assert!(iceberg_shape.is_ellipse);
        assert_eq!(iceberg_shape.over_water_height, 11);
        assert_eq!(iceberg_shape.under_water_height, 18);
        assert_eq!(iceberg_shape.width, 11);
        assert_eq!(super::iceberg_ellipse_c(9, 11, 5), 3);
        assert!(
            super::iceberg_signed_distance_circle(0, 0, BlockPos { x: 0, y: 0, z: 0 }, 5, 0.5,)
                < 0.0
        );
        assert!(
            super::iceberg_signed_distance_ellipse(0, 0, BlockPos { x: 0, y: 0, z: 0 }, 11, 5, 0.0,)
                < 0.0
        );
        assert_eq!(super::iceberg_height_radius_ellipse(0, 11, 11), 6);
        assert_eq!(super::iceberg_height_radius_steep(1, 11, 11, 0.0), 5);
        assert!(super::iceberg_height_radius_round(0, 11, 11, 0.5, 0, 0) > 0);
        assert_eq!(
            super::iceberg_set_block_action("minecraft:air", 1, 11, true, true, 0, 0.1,),
            super::IcebergBlockAction::SnowBlock
        );
        assert_eq!(
            super::iceberg_set_block_action("minecraft:stone", 1, 11, true, true, 0, 0.1,),
            super::IcebergBlockAction::Keep
        );
        assert!(super::iceberg_should_skip_surface_noise(-0.25, true, 0.95));
        assert_eq!(
            super::iceberg_carve_action("minecraft:packed_ice", true),
            super::IcebergBlockAction::Water
        );
        assert_eq!(
            super::iceberg_carve_action("minecraft:blue_ice", false),
            super::IcebergBlockAction::Air
        );
        assert_eq!(
            super::iceberg_smooth_action("minecraft:packed_ice", false, 3),
            super::IcebergBlockAction::Air
        );
        assert_eq!(
            super::iceberg_smooth_action("minecraft:snow", true, 0),
            super::IcebergBlockAction::Air
        );
        assert!(super::blue_ice_can_start(
            62,
            63,
            "minecraft:water",
            "minecraft:air",
            &["minecraft:packed_ice"]
        ));
        assert!(!super::blue_ice_can_start(
            63,
            63,
            "minecraft:water",
            "minecraft:air",
            &["minecraft:packed_ice"]
        ));
        assert!(!super::blue_ice_can_start(
            62,
            63,
            "minecraft:air",
            "minecraft:stone",
            &["minecraft:packed_ice"]
        ));
        assert!(!super::blue_ice_can_start(
            62,
            63,
            "minecraft:water",
            "minecraft:air",
            &["minecraft:ice"]
        ));
        assert_eq!(super::blue_ice_xz_diff(1), 3);
        assert_eq!(super::blue_ice_xz_diff(-5), 1);
        assert_eq!(super::blue_ice_xz_diff(-6), 0);
        assert_eq!(
            super::blue_ice_spread_candidate(
                BlockPos {
                    x: 10,
                    y: 64,
                    z: 10
                },
                -1,
                2,
                0,
                1,
                0
            ),
            Some(BlockPos {
                x: 12,
                y: 63,
                z: 11
            })
        );
        assert_eq!(
            super::blue_ice_spread_candidate(
                BlockPos {
                    x: 10,
                    y: 64,
                    z: 10
                },
                -6,
                0,
                0,
                0,
                0
            ),
            None
        );
        assert!(super::blue_ice_spread_can_place(
            "minecraft:water",
            &["minecraft:blue_ice"]
        ));
        assert!(super::blue_ice_spread_can_place(
            "minecraft:ice",
            &["minecraft:blue_ice"]
        ));
        assert!(!super::blue_ice_spread_can_place(
            "minecraft:stone",
            &["minecraft:blue_ice"]
        ));
        assert!(!super::blue_ice_spread_can_place(
            "minecraft:water",
            &["minecraft:packed_ice"]
        ));
        let random_feature = super::RandomFeatureConfigurationModel {
            features: vec![
                super::WeightedPlacedFeatureModel {
                    feature: "minecraft:patch_tulip",
                    chance: 0.2,
                },
                super::WeightedPlacedFeatureModel {
                    feature: "minecraft:patch_grass",
                    chance: 0.5,
                },
            ],
            default_feature: "minecraft:flower_default",
        };
        assert_eq!(
            super::validate_weighted_placed_feature(random_feature.features[0]),
            Ok(random_feature.features[0])
        );
        assert_eq!(
            super::validate_weighted_placed_feature(super::WeightedPlacedFeatureModel {
                feature: "minecraft:bad",
                chance: 1.1,
            }),
            Err("weighted placed feature chance must be in 0.0..=1.0")
        );
        assert_eq!(
            super::random_selector_feature(&random_feature, &[0.3, 0.25]),
            Some("minecraft:patch_grass")
        );
        assert_eq!(
            super::random_selector_feature(&random_feature, &[0.3, 0.6]),
            Some("minecraft:flower_default")
        );
        let simple_random_feature = super::SimpleRandomFeatureConfigurationModel {
            features: vec![
                "minecraft:flower_plain",
                "minecraft:patch_grass",
                "minecraft:patch_sunflower",
            ],
        };
        assert_eq!(
            super::simple_random_selector_feature(&simple_random_feature, 4),
            Some("minecraft:patch_grass")
        );
        assert_eq!(
            super::simple_random_selector_feature(
                &super::SimpleRandomFeatureConfigurationModel { features: vec![] },
                0,
            ),
            None
        );
        let random_boolean_feature = super::RandomBooleanFeatureConfigurationModel {
            feature_true: "minecraft:flower_cherry",
            feature_false: "minecraft:patch_grass",
        };
        assert_eq!(
            super::random_boolean_selector_feature(random_boolean_feature, true),
            "minecraft:flower_cherry"
        );
        assert_eq!(
            super::random_boolean_selector_feature(random_boolean_feature, false),
            "minecraft:patch_grass"
        );
        let fill_layer_config = super::FillLayerConfigurationModel {
            height: 32,
            state: "minecraft:lava",
        };
        assert_eq!(
            super::validate_fill_layer_config(fill_layer_config, 384),
            Ok(fill_layer_config)
        );
        assert_eq!(
            super::validate_fill_layer_config(
                super::FillLayerConfigurationModel {
                    height: 385,
                    state: "minecraft:lava",
                },
                384,
            ),
            Err("fill layer height must be in 0..=dimension_y_size")
        );
        let mut fill_air = vec![false; 256];
        fill_air[0] = true;
        fill_air[17] = true;
        assert_eq!(
            super::fill_layer_placement_plan(
                BlockPos { x: 16, y: 0, z: 32 },
                -64,
                fill_layer_config,
                &fill_air,
            ),
            vec![
                BlockPos {
                    x: 16,
                    y: -32,
                    z: 32
                },
                BlockPos {
                    x: 17,
                    y: -32,
                    z: 33
                },
            ]
        );
        assert_eq!(super::end_island_layer_radius(4.0), 4);
        assert_eq!(super::end_island_next_size(4.0, 1), 2.5);
        let end_island =
            super::end_island_placement_plan(BlockPos { x: 0, y: 80, z: 0 }, 0, &[1, 1, 1]);
        assert!(end_island.contains(&super::EndIslandPlacementBlock {
            pos: BlockPos { x: 0, y: 80, z: 0 },
            state: "minecraft:end_stone",
        }));
        assert!(end_island.contains(&super::EndIslandPlacementBlock {
            pos: BlockPos { x: 0, y: 79, z: 0 },
            state: "minecraft:end_stone",
        }));
        assert!(!end_island.contains(&super::EndIslandPlacementBlock {
            pos: BlockPos { x: 5, y: 80, z: 5 },
            state: "minecraft:end_stone",
        }));
        let replace_sphere = super::ReplaceSphereConfigurationModel {
            target_state: "minecraft:netherrack",
            replace_state: "minecraft:basalt",
            radius_min: 3,
            radius_max: 7,
        };
        assert_eq!(
            super::validate_replace_sphere_config(replace_sphere),
            Ok(replace_sphere)
        );
        assert_eq!(super::replace_sphere_radius(replace_sphere, 5), 3);
        assert_eq!(
            super::validate_replace_sphere_config(super::ReplaceSphereConfigurationModel {
                target_state: "minecraft:netherrack",
                replace_state: "minecraft:basalt",
                radius_min: 8,
                radius_max: 7,
            }),
            Err("replace sphere radius bounds must be ordered in 0..=12")
        );
        assert_eq!(
            super::replace_sphere_find_target(
                BlockPos { x: 4, y: 70, z: 8 },
                -64,
                320,
                &["minecraft:air", "minecraft:netherrack"],
                "minecraft:netherrack",
            ),
            Some(BlockPos { x: 4, y: 69, z: 8 })
        );
        let sphere_positions =
            super::replace_sphere_positions(BlockPos { x: 0, y: 0, z: 0 }, 1, 2, 3);
        assert!(sphere_positions.contains(&BlockPos { x: 0, y: 0, z: 0 }));
        assert!(sphere_positions.contains(&BlockPos { x: 0, y: -2, z: 0 }));
        assert!(!sphere_positions.contains(&BlockPos { x: 1, y: 2, z: 3 }));
        assert!(super::basalt_pillar_can_start(true, false));
        assert!(!super::basalt_pillar_can_start(true, true));
        assert!(super::basalt_pillar_hangoff_places(9));
        assert!(!super::basalt_pillar_hangoff_places(10));
        assert!(super::basalt_pillar_base_places(1, 2, 7));
        assert!(!super::basalt_pillar_base_places(3, 3, 1));
        let base_drop = vec![&[][..]; 49];
        let base_supported = vec![true; 49];
        let basalt_pillar = super::basalt_pillar_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &[true, true, false],
            &[false, false],
            &[(1, 1, 10, 1), (1, 10, 1, 1)],
            &[0; 49],
            &base_drop,
            &base_supported,
        );
        assert!(basalt_pillar.contains(&super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            kind: super::BasaltPillarBlockKind::Core,
        }));
        assert!(basalt_pillar.contains(&super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: -1 },
            kind: super::BasaltPillarBlockKind::HangOff,
        }));
        assert!(!basalt_pillar.contains(&super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: -1, y: 64, z: 0 },
            kind: super::BasaltPillarBlockKind::HangOff,
        }));
        assert!(basalt_pillar.iter().any(|block| {
            block.kind == super::BasaltPillarBlockKind::Base && block.pos.y == 62
        }));
        assert!(super::basalt_columns_cannot_place_on(
            "minecraft:magma_block"
        ));
        assert!(!super::basalt_columns_cannot_place_on(
            "minecraft:netherrack"
        ));
        assert!(super::basalt_columns_is_air_or_lava_ocean(
            "minecraft:lava",
            31,
            32,
        ));
        assert!(!super::basalt_columns_is_air_or_lava_ocean(
            "minecraft:lava",
            33,
            32,
        ));
        assert!(super::basalt_columns_can_place_at(
            "minecraft:air",
            "minecraft:netherrack",
            64,
            32,
        ));
        assert!(!super::basalt_columns_can_place_at(
            "minecraft:air",
            "minecraft:magma_block",
            64,
            32,
        ));
        let column_config = super::ColumnFeatureConfigurationModel {
            reach_min: 1,
            reach_max: 3,
            height_min: 5,
            height_max: 10,
        };
        assert_eq!(
            super::validate_column_feature_config(column_config),
            Ok(column_config)
        );
        assert_eq!(
            super::validate_column_feature_config(super::ColumnFeatureConfigurationModel {
                reach_min: 4,
                reach_max: 3,
                height_min: 5,
                height_max: 10,
            }),
            Err("column reach bounds must be ordered in 0..=3")
        );
        assert_eq!(
            super::basalt_columns_cluster_parameters(7, 0.5),
            (true, 5, 50)
        );
        assert_eq!(
            super::basalt_columns_cluster_parameters(7, 0.95),
            (false, 7, 15)
        );
        let column_blocks = super::basalt_column_blocks_from_surface(
            BlockPos { x: 1, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            4,
            3,
            &[true, true, false, true],
            &[false, true, false, false],
        );
        assert!(column_blocks.contains(&super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 0 },
        }));
        assert!(column_blocks.contains(&super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 65, z: 0 },
        }));
        assert!(!column_blocks.contains(&super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 67, z: 0 },
        }));
        let delta_config = super::DeltaFeatureConfigurationModel {
            contents: "minecraft:lava",
            rim: "minecraft:magma_block",
            size_min: 3,
            size_max: 7,
            rim_size_min: 0,
            rim_size_max: 2,
        };
        assert_eq!(super::validate_delta_config(delta_config), Ok(delta_config));
        assert_eq!(
            super::validate_delta_config(super::DeltaFeatureConfigurationModel {
                contents: "minecraft:lava",
                rim: "minecraft:magma_block",
                size_min: 17,
                size_max: 17,
                rim_size_min: 0,
                rim_size_max: 2,
            }),
            Err("delta size bounds must be ordered in 0..=16")
        );
        assert!(super::delta_cannot_replace("minecraft:bedrock"));
        assert!(!super::delta_cannot_replace("minecraft:netherrack"));
        assert!(super::delta_is_clear(
            "minecraft:netherrack",
            "minecraft:lava",
            false,
            false,
            false,
            false,
            false,
            false,
        ));
        assert!(!super::delta_is_clear(
            "minecraft:netherrack",
            "minecraft:lava",
            false,
            false,
            true,
            false,
            false,
            false,
        ));
        assert!(super::delta_has_rim(0.5, 1, 2));
        assert!(!super::delta_has_rim(0.95, 1, 2));
        let delta_offsets = super::delta_candidate_offsets(2, 1);
        assert!(delta_offsets.contains(&(0, 0)));
        assert!(delta_offsets.contains(&(2, 0)));
        assert!(!delta_offsets.contains(&(2, 1)));
        assert!(super::glowstone_can_start(true, "minecraft:netherrack"));
        assert!(!super::glowstone_can_start(true, "minecraft:air"));
        assert_eq!(
            super::glowstone_candidate_offset(7, 1, 11, 2, 6),
            BlockPos {
                x: 6,
                y: -11,
                z: -4
            }
        );
        assert!(super::glowstone_can_grow(true, 1));
        assert!(!super::glowstone_can_grow(true, 2));
        let nether_vegetation = super::NetherForestVegetationConfigModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:crimson_roots"),
            spread_width: 8,
            spread_height: 4,
        };
        assert_eq!(
            super::validate_nether_forest_vegetation_config(&nether_vegetation),
            Ok(())
        );
        assert!(super::nether_forest_vegetation_can_start(
            "minecraft:crimson_nylium",
            64,
            -64,
            320,
        ));
        assert!(!super::nether_forest_vegetation_can_start(
            "minecraft:netherrack",
            64,
            -64,
            320,
        ));
        assert_eq!(super::nether_forest_vegetation_attempts(8), 64);
        assert_eq!(
            super::nether_forest_vegetation_offset(8, 4, 7, 1, 3, 1, 2, 6),
            BlockPos { x: 6, y: 2, z: -4 }
        );
        assert!(super::twisting_vines_valid_ground(
            "minecraft:warped_nylium"
        ));
        assert!(super::weeping_vines_valid_ceiling(
            "minecraft:nether_wart_block"
        ));
        assert_eq!(super::vine_height(2, 8, 6, 1), 6);
        assert_eq!(super::vine_height(2, 8, 1, 5), 1);
        assert_eq!(super::vine_age(17, 25, 9), 17);
        let twisting_column = super::twisting_vines_column(
            BlockPos { x: 0, y: 64, z: 0 },
            3,
            &[true, true, true],
            &[false, true, false],
            1,
        );
        assert_eq!(
            twisting_column,
            vec![
                super::VineColumnBlock {
                    pos: BlockPos { x: 0, y: 64, z: 0 },
                    state: "minecraft:twisting_vines_plant",
                    kind: super::VineColumnBlockKind::Plant,
                    age: None,
                },
                super::VineColumnBlock {
                    pos: BlockPos { x: 0, y: 65, z: 0 },
                    state: "minecraft:twisting_vines",
                    kind: super::VineColumnBlockKind::Head,
                    age: Some(18),
                },
            ]
        );
        let weeping_column = super::weeping_vines_column(
            BlockPos { x: 0, y: 70, z: 0 },
            2,
            &[true, true, true],
            &[false, false, true],
            2,
        );
        assert_eq!(
            weeping_column.last(),
            Some(&super::VineColumnBlock {
                pos: BlockPos { x: 0, y: 68, z: 0 },
                state: "minecraft:weeping_vines",
                kind: super::VineColumnBlockKind::Head,
                age: Some(19),
            })
        );
        assert!(super::weeping_vines_wart_can_grow(true, 1));
        assert!(!super::weeping_vines_wart_can_grow(true, 2));
        let end_platform = super::end_platform_blocks(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(end_platform.len(), 100);
        assert_eq!(
            end_platform
                .iter()
                .filter(|block| block.state == "minecraft:obsidian")
                .count(),
            25
        );
        assert!(end_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos {
                x: -2,
                y: 63,
                z: -2
            },
            state: "minecraft:obsidian",
        }));
        assert!(end_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 2, y: 66, z: 2 },
            state: "minecraft:air",
        }));
        assert_eq!(
            super::void_start_platform_origin(64),
            BlockPos { x: 8, y: 67, z: 8 }
        );
        assert!(super::void_start_platform_applies_to_chunk(ChunkPos {
            x: 1,
            z: 1
        }));
        assert!(!super::void_start_platform_applies_to_chunk(ChunkPos {
            x: 2,
            z: 0
        }));
        let void_platform = super::void_start_platform_blocks(ChunkPos { x: 0, z: 0 }, 64);
        assert!(void_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 8, y: 67, z: 8 },
            state: "minecraft:cobblestone",
        }));
        assert!(void_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 67, z: 0 },
            state: "minecraft:stone",
        }));
        assert_eq!(
            super::void_start_platform_blocks(ChunkPos { x: 2, z: 0 }, 64),
            Vec::new()
        );
        assert_eq!(
            super::end_gateway_known_exit(BlockPos { x: 1, y: 2, z: 3 }, true),
            super::EndGatewayConfigurationModel {
                exit: Some(BlockPos { x: 1, y: 2, z: 3 }),
                exact: true,
            }
        );
        assert_eq!(
            super::end_gateway_delayed_exit_search(),
            super::EndGatewayConfigurationModel {
                exit: None,
                exact: false,
            }
        );
        let gateway = super::end_gateway_blocks(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(gateway.len(), 45);
        assert!(gateway.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            state: "minecraft:end_gateway",
        }));
        assert!(gateway.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: 0 },
            state: "minecraft:bedrock",
        }));
        assert!(gateway.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 0 },
            state: "minecraft:air",
        }));
        assert_eq!(
            gateway
                .iter()
                .filter(|block| block.state == "minecraft:bedrock")
                .count(),
            12
        );
        assert!(super::chorus_plant_can_start(true, "minecraft:end_stone"));
        assert!(!super::chorus_plant_can_start(false, "minecraft:end_stone"));
        assert!(!super::chorus_plant_can_start(true, "minecraft:stone"));
        assert!(super::chorus_all_horizontal_neighbors_empty(
            [true, false, true, true],
            Some(1),
        ));
        assert!(!super::chorus_all_horizontal_neighbors_empty(
            [true, false, true, true],
            None,
        ));
        assert!(super::chorus_branch_target_within_spread(
            BlockPos { x: 7, y: 68, z: -7 },
            BlockPos { x: 0, y: 64, z: 0 },
            8,
        ));
        assert!(!super::chorus_branch_target_within_spread(
            BlockPos { x: 8, y: 68, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            8,
        ));
        assert_eq!(super::chorus_trunk_height(0, 0), 2);
        assert_eq!(super::chorus_trunk_height(1, 3), 4);
        assert_eq!(super::chorus_stem_attempts(0, 0), 1);
        assert_eq!(super::chorus_stem_attempts(1, 3), 3);
        let chorus_trunk =
            super::chorus_trunk_and_terminal_flower(BlockPos { x: 0, y: 64, z: 0 }, 0, 0, false);
        assert_eq!(
            chorus_trunk.last(),
            Some(&super::ChorusPlantPlacementBlock {
                pos: BlockPos { x: 0, y: 66, z: 0 },
                kind: super::ChorusPlantPlacementKind::Flower,
                age: Some(5),
            })
        );
        assert_eq!(
            chorus_trunk
                .iter()
                .filter(|block| block.kind == super::ChorusPlantPlacementKind::Plant)
                .count(),
            3
        );
        assert_eq!(
            super::end_podium_location(BlockPos { x: 1, y: 2, z: 3 }),
            BlockPos { x: 1, y: 2, z: 3 }
        );
        assert!(super::end_podium_inside_rim(
            BlockPos { x: 2, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        assert!(!super::end_podium_inside_rim(
            BlockPos { x: 3, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        assert!(super::end_podium_inside_body(
            BlockPos { x: 3, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        let inactive_podium = super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, false);
        let active_podium = super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, true);
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 63, z: 0 },
            kind: super::EndPodiumBlockKind::Bedrock,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 63, z: 0 },
            kind: super::EndPodiumBlockKind::EndStone,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 64, z: 0 },
            kind: super::EndPodiumBlockKind::Bedrock,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: -1 },
            kind: super::EndPodiumBlockKind::WallTorch(super::HorizontalDirection::North),
        }));
        assert!(active_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::EndPodiumBlockKind::EndPortal,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::EndPodiumBlockKind::Air,
        }));
        assert_eq!(
            active_podium
                .iter()
                .filter(|block| block.kind
                    == super::EndPodiumBlockKind::WallTorch(super::HorizontalDirection::East))
                .count(),
            1
        );
        let spike = super::end_spike_from_size(0, 2);
        assert_eq!(
            spike,
            super::EndSpikeModel {
                center_x: 42,
                center_z: 0,
                radius: 2,
                height: 82,
                guarded: true,
            }
        );
        assert!(super::end_spike_is_center_within_chunk(
            spike,
            BlockPos { x: 32, y: 0, z: 0 },
        ));
        assert!(!super::end_spike_is_center_within_chunk(
            spike,
            BlockPos { x: 16, y: 0, z: 0 },
        ));
        assert_eq!(
            super::end_spike_top_bounding_box(spike, -64, 320),
            (
                BlockPos {
                    x: 40,
                    y: -64,
                    z: -2
                },
                BlockPos {
                    x: 44,
                    y: 320,
                    z: 2
                },
            )
        );
        let spike_blocks = super::end_spike_cylinder_and_air_blocks(spike, 64);
        assert!(spike_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos { x: 42, y: 64, z: 0 },
            kind: super::EndSpikeBlockKind::Obsidian,
        }));
        assert!(spike_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos {
                x: 40,
                y: 82,
                z: -2
            },
            kind: super::EndSpikeBlockKind::Air,
        }));
        let cage_blocks = super::end_spike_guard_cage_blocks(spike);
        assert!(cage_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos { x: 40, y: 82, z: 0 },
            kind: super::EndSpikeBlockKind::IronBars {
                north: true,
                south: true,
                west: false,
                east: false,
            },
        }));
        assert!(cage_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos { x: 42, y: 85, z: 0 },
            kind: super::EndSpikeBlockKind::IronBars {
                north: true,
                south: true,
                west: true,
                east: true,
            },
        }));
        assert_eq!(
            super::end_spike_guard_cage_blocks(super::EndSpikeModel {
                guarded: false,
                ..spike
            }),
            Vec::new()
        );
        let spike_config = super::EndSpikeConfigurationModel {
            crystal_invulnerable: true,
            spikes: vec![spike],
            crystal_beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
        };
        assert_eq!(
            super::end_crystal_for_spike(spike, &spike_config, 0.25),
            super::EndCrystalPlacement {
                x: 42.5,
                y: 83.0,
                z: 0.5,
                yaw: 90.0,
                beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
                invulnerable: true,
            }
        );
        assert_eq!(
            super::end_spike_crystal_support_blocks(spike),
            vec![
                super::EndSpikePlacementBlock {
                    pos: BlockPos { x: 42, y: 82, z: 0 },
                    kind: super::EndSpikeBlockKind::Bedrock,
                },
                super::EndSpikePlacementBlock {
                    pos: BlockPos { x: 42, y: 83, z: 0 },
                    kind: super::EndSpikeBlockKind::Fire,
                },
            ]
        );
        let huge_fungus_config = super::HugeFungusConfigurationModel {
            valid_base_state: "minecraft:crimson_nylium",
            stem_state: "minecraft:crimson_stem",
            hat_state: "minecraft:nether_wart_block",
            decor_state: "minecraft:shroomlight",
            planted: false,
        };
        assert!(super::huge_fungus_can_start(
            &huge_fungus_config,
            "minecraft:crimson_nylium",
        ));
        assert!(!super::huge_fungus_can_start(
            &huge_fungus_config,
            "minecraft:warped_nylium",
        ));
        assert_eq!(super::huge_fungus_total_height(0, 1), 4);
        assert_eq!(super::huge_fungus_total_height(9, 0), 26);
        assert!(super::huge_fungus_fits_height(64, 10, 80, false));
        assert!(!super::huge_fungus_fits_height(64, 15, 80, false));
        assert!(super::huge_fungus_fits_height(64, 15, 80, true));
        assert!(super::huge_fungus_is_huge(false, 0.05));
        assert!(!super::huge_fungus_is_huge(true, 0.05));
        let stem_blocks = super::huge_fungus_stem_blocks(
            BlockPos { x: 0, y: 64, z: 0 },
            3,
            true,
            &[0.0, 1.0, 1.0, 1.0],
        );
        assert!(stem_blocks.contains(&super::HugeFungusStemBlock {
            pos: BlockPos {
                x: -1,
                y: 64,
                z: -1
            },
            kind: super::HugeFungusStemKind::CornerStem,
        }));
        assert!(!stem_blocks.contains(&super::HugeFungusStemBlock {
            pos: BlockPos { x: -1, y: 64, z: 1 },
            kind: super::HugeFungusStemKind::CornerStem,
        }));
        assert!(stem_blocks.contains(&super::HugeFungusStemBlock {
            pos: BlockPos { x: 0, y: 66, z: 0 },
            kind: super::HugeFungusStemKind::Stem,
        }));
        assert_eq!(super::huge_fungus_hat_height(12, 0), 5);
        assert_eq!(super::huge_fungus_hat_radius(7, 12, 5, false, 0), 2);
        assert_eq!(super::huge_fungus_hat_radius(10, 12, 5, false, 2), 1);
        assert_eq!(super::huge_fungus_hat_radius(4, 12, 9, true, 0), 4);
        let hat_cells =
            super::huge_fungus_hat_cells(BlockPos { x: 0, y: 64, z: 0 }, 8, 0, &[0; 9], false);
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::HugeFungusHatRole::Bottom));
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::HugeFungusHatRole::Inside));
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::HugeFungusHatRole::Corner));
        assert_eq!(
            super::huge_fungus_hat_drop_outcome(false, 0.1, 0, true),
            super::HugeFungusHatPlacement::HatWithWeepingVines
        );
        assert_eq!(
            super::huge_fungus_hat_drop_outcome(false, 0.2, 0, true),
            super::HugeFungusHatPlacement::None
        );
        assert_eq!(
            super::huge_fungus_hat_block_outcome(0.05, 0.0, 0.0, 0.1, 0.2, 0.1),
            super::HugeFungusHatPlacement::Decor
        );
        assert_eq!(
            super::huge_fungus_hat_block_outcome(0.2, 0.1, 0.05, 0.1, 0.2, 0.1),
            super::HugeFungusHatPlacement::HatWithWeepingVines
        );
        assert_eq!(
            super::huge_fungus_hat_probabilities(super::HugeFungusHatRole::Edge, true),
            Some((0.0005, 0.98, 0.07))
        );
        assert_eq!(super::huge_fungus_weeping_vine_height(4, 7), 10);

        let pile_config = super::BlockPileConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:hay_block"),
        };
        let all_shape_rolls = vec![(1.0, 0.0, 1.0); 7 * 7 * 2];
        let pile_positions = super::block_pile_placement_candidates(
            BlockPos { x: 0, y: 64, z: 0 },
            -64,
            1,
            1,
            &all_shape_rolls,
        );
        assert!(pile_positions.contains(&BlockPos { x: 0, y: 64, z: 0 }));
        assert!(pile_positions.contains(&BlockPos { x: 0, y: 65, z: 0 }));
        assert!(pile_positions.len() > 20);
        assert!(super::block_pile_placement_candidates(
            BlockPos { x: 0, y: -60, z: 0 },
            -64,
            0,
            0,
            &all_shape_rolls,
        )
        .is_empty());
        assert_eq!(
            super::block_pile_try_place(
                &pile_config,
                true,
                "minecraft:grass_block",
                true,
                false,
                0,
            ),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::block_pile_try_place(&pile_config, true, "minecraft:dirt_path", true, false, 0,),
            None
        );
        assert_eq!(
            super::block_pile_try_place(&pile_config, true, "minecraft:dirt_path", false, true, 0,),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::block_pile_try_place(
                &pile_config,
                false,
                "minecraft:grass_block",
                true,
                true,
                0,
            ),
            None
        );

        let disk_config = super::DiskConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:clay"),
            target: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt"],
            },
            radius: super::IntProviderModel::Constant(1),
            half_height: 1,
        };
        let mut disk_contexts = Vec::new();
        for y in 63..=65 {
            disk_contexts.push((
                BlockPos { x: 0, y, z: 0 },
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:dirt",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
            ));
        }
        let disk = super::disk_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &disk_config,
            &disk_contexts,
            &[],
        );
        assert_eq!(disk.len(), 3);
        assert_eq!(disk[0].pos, BlockPos { x: 0, y: 65, z: 0 });
        assert!(disk[0].mark_above_for_post_processing);
        assert!(!disk[1].mark_above_for_post_processing);
        assert!(disk.iter().all(|block| block.state == "minecraft:clay"));

        disk_contexts[1].1.block = "minecraft:stone";
        let disk_with_gap = super::disk_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &disk_config,
            &disk_contexts,
            &[],
        );
        assert_eq!(disk_with_gap.len(), 2);
        assert!(disk_with_gap[0].mark_above_for_post_processing);
        assert!(disk_with_gap[1].mark_above_for_post_processing);

        let snow_plan = super::snow_and_freeze_placement_plan(
            BlockPos {
                x: 32,
                y: 0,
                z: -16,
            },
            &[
                super::SnowAndFreezeColumn {
                    x: 32,
                    z: -16,
                    motion_blocking_height: 70,
                    should_freeze: true,
                    should_snow: true,
                    below_has_snowy_property: true,
                },
                super::SnowAndFreezeColumn {
                    x: 33,
                    z: -16,
                    motion_blocking_height: 65,
                    should_freeze: false,
                    should_snow: true,
                    below_has_snowy_property: false,
                },
            ],
        );
        assert_eq!(
            snow_plan,
            vec![
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 69,
                        z: -16
                    },
                    state: "minecraft:ice",
                },
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 70,
                        z: -16
                    },
                    state: "minecraft:snow",
                },
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 69,
                        z: -16
                    },
                    state: "minecraft:snowy=true",
                },
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 33,
                        y: 65,
                        z: -16
                    },
                    state: "minecraft:snow",
                },
            ]
        );

        let magma_config = super::UnderwaterMagmaConfigurationModel {
            floor_search_range: 12,
            placement_radius_around_floor: 1,
            placement_probability_per_valid_position: 0.5,
        };
        let valid_magma = super::UnderwaterMagmaCandidate {
            pos: BlockPos { x: 0, y: 62, z: 0 },
            block: "minecraft:stone",
            below_visible_from_above: false,
            horizontal_visible_from_outside: false,
        };
        assert!(super::underwater_magma_is_valid_placement(&valid_magma));
        assert!(!super::underwater_magma_is_valid_placement(
            &super::UnderwaterMagmaCandidate {
                block: "minecraft:water",
                ..valid_magma
            }
        ));
        assert!(!super::underwater_magma_is_valid_placement(
            &super::UnderwaterMagmaCandidate {
                below_visible_from_above: true,
                ..valid_magma
            }
        ));
        assert!(!super::underwater_magma_is_valid_placement(
            &super::UnderwaterMagmaCandidate {
                horizontal_visible_from_outside: true,
                ..valid_magma
            }
        ));
        assert_eq!(
            super::underwater_magma_placement_plan(
                BlockPos { x: 0, y: 70, z: 0 },
                Some(62),
                magma_config,
                &[valid_magma],
                &[0.0; 27],
            ),
            vec![BlockPos { x: 0, y: 62, z: 0 }]
        );
        assert!(super::underwater_magma_placement_plan(
            BlockPos { x: 0, y: 70, z: 0 },
            None,
            magma_config,
            &[valid_magma],
            &[0.0; 27],
        )
        .is_empty());
        let dripstone_config = super::DripstoneClusterSampledConfig {
            floor_to_ceiling_search_range: 12,
            height: 6,
            x_radius: 3,
            z_radius: 3,
            max_stalagmite_stalactite_height_diff: 1,
            height_deviation: 2,
            dripstone_block_layer_thickness: 2,
            density: 1.0,
            wetness: 0.5,
            chance_of_dripstone_column_at_max_distance_from_center: 0.2,
            max_distance_from_edge_affecting_chance_of_dripstone_column: 3,
            max_distance_from_center_affecting_height_bias: 4,
        };
        assert_eq!(
            super::validate_dripstone_cluster_sampled_config(dripstone_config),
            Ok(dripstone_config)
        );
        assert!(
            (super::dripstone_cluster_chance_of_column(3, 3, 3, 0, dripstone_config) - 0.2).abs()
                < 0.000001
        );
        assert_eq!(
            super::dripstone_cluster_chance_of_column(3, 3, 0, 0, dripstone_config),
            1.0
        );
        assert_eq!(
            super::dripstone_cluster_height_for_column(1, 1, 0.5, 6, dripstone_config, 0.75, 5.0,),
            0
        );
        assert_eq!(
            super::pointed_dripstone_column(
                BlockPos { x: 0, y: 70, z: 0 },
                super::PointedDripstoneDirection::Down,
                4,
                true,
            ),
            vec![
                super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 70, z: 0 },
                    direction: super::PointedDripstoneDirection::Down,
                    thickness: super::PointedDripstoneThickness::Base,
                },
                super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 69, z: 0 },
                    direction: super::PointedDripstoneDirection::Down,
                    thickness: super::PointedDripstoneThickness::Middle,
                },
                super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 68, z: 0 },
                    direction: super::PointedDripstoneDirection::Down,
                    thickness: super::PointedDripstoneThickness::Frustum,
                },
                super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 67, z: 0 },
                    direction: super::PointedDripstoneDirection::Down,
                    thickness: super::PointedDripstoneThickness::TipMerge,
                },
            ]
        );
        let dripstone_plan = super::dripstone_cluster_column_plan(
            BlockPos {
                x: 10,
                y: 64,
                z: 10,
            },
            dripstone_config,
            super::DripstoneClusterColumnInput {
                dx: 0,
                dz: 0,
                ceiling_y: Some(74),
                floor_y: Some(68),
                floor_pool_supported: false,
                ceiling_is_lava: false,
                floor_is_lava: false,
            },
            super::DripstoneClusterColumnRolls {
                water_roll: 0.75,
                stalactite_roll: 0.0,
                stalactite_density_roll: 0.0,
                stalactite_biased_height: 4.0,
                stalagmite_roll: 0.0,
                stalagmite_density_roll: 0.0,
                stalagmite_biased_height: 3.0,
                stalagmite_height_diff_roll: 1,
                overlap_split_roll: 0,
                merge_tips_roll: true,
            },
        );
        assert_eq!(
            dripstone_plan.ceiling_dripstone_blocks,
            vec![
                BlockPos {
                    x: 10,
                    y: 74,
                    z: 10
                },
                BlockPos {
                    x: 10,
                    y: 75,
                    z: 10
                },
            ]
        );
        assert_eq!(
            dripstone_plan.floor_dripstone_blocks,
            vec![
                BlockPos {
                    x: 10,
                    y: 68,
                    z: 10
                },
                BlockPos {
                    x: 10,
                    y: 67,
                    z: 10
                },
            ]
        );
        assert!(!dripstone_plan.merge_tips);
        assert_eq!(dripstone_plan.stalactite.len(), 4);
        assert_eq!(dripstone_plan.stalagmite.len(), 1);
        let pointed_config = super::PointedDripstoneConfigurationModel {
            chance_of_taller_dripstone: 0.5,
            chance_of_directional_spread: 0.7,
            chance_of_spread_radius2: 0.5,
            chance_of_spread_radius3: 0.5,
        };
        assert_eq!(
            super::validate_pointed_dripstone_configuration(pointed_config),
            Ok(pointed_config)
        );
        assert_eq!(
            super::pointed_dripstone_tip_direction(true, true, true),
            Some(super::PointedDripstoneDirection::Down)
        );
        assert_eq!(
            super::pointed_dripstone_tip_direction(false, true, true),
            Some(super::PointedDripstoneDirection::Up)
        );
        assert_eq!(
            super::pointed_dripstone_tip_direction(false, false, true),
            None
        );
        let pointed_plan = super::pointed_dripstone_feature_plan(
            BlockPos { x: 4, y: 70, z: 4 },
            pointed_config,
            true,
            false,
            false,
            0.25,
            true,
            &[
                super::PointedDripstoneSpreadRoll {
                    direction: HorizontalDirection::East,
                    direction_roll: 0.2,
                    radius2_roll: 0.2,
                    radius2_direction: HorizontalDirection::South,
                    radius3_roll: 0.2,
                    radius3_direction: HorizontalDirection::West,
                },
                super::PointedDripstoneSpreadRoll {
                    direction: HorizontalDirection::North,
                    direction_roll: 0.9,
                    radius2_roll: 0.0,
                    radius2_direction: HorizontalDirection::North,
                    radius3_roll: 0.0,
                    radius3_direction: HorizontalDirection::North,
                },
            ],
        )
        .unwrap();
        assert_eq!(
            pointed_plan.dripstone_blocks,
            vec![
                BlockPos { x: 4, y: 71, z: 4 },
                BlockPos { x: 5, y: 71, z: 4 },
                BlockPos { x: 5, y: 71, z: 5 },
                BlockPos { x: 4, y: 71, z: 5 },
            ]
        );
        assert_eq!(
            pointed_plan.pointed_blocks,
            vec![
                super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 4, y: 70, z: 4 },
                    direction: super::PointedDripstoneDirection::Down,
                    thickness: super::PointedDripstoneThickness::Frustum,
                },
                super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 4, y: 69, z: 4 },
                    direction: super::PointedDripstoneDirection::Down,
                    thickness: super::PointedDripstoneThickness::Tip,
                },
            ]
        );
        let large_dripstone_config = super::LargeDripstoneSampledConfig {
            floor_to_ceiling_search_range: 30,
            column_radius_min: 2,
            column_radius_max: 6,
            height_scale: 2.0,
            max_column_radius_to_cave_height_ratio: 1.0,
            stalactite_bluntness: 1.0,
            stalagmite_bluntness: 1.5,
            wind_speed: 0.5,
            wind_direction_radians: 0.0,
            min_radius_for_wind: 2,
            min_bluntness_for_wind: 1.0,
        };
        assert_eq!(
            super::validate_large_dripstone_sampled_config(large_dripstone_config),
            Ok(large_dripstone_config)
        );
        assert_eq!(
            super::large_dripstone_selected_radius(4, large_dripstone_config, 99),
            Some(2)
        );
        assert_eq!(
            super::large_dripstone_selected_radius(3, large_dripstone_config, 0),
            None
        );
        assert_eq!(
            super::large_dripstone_wind_offset(
                BlockPos { x: 4, y: 62, z: 4 },
                64,
                large_dripstone_config,
            ),
            BlockPos { x: 5, y: 62, z: 4 }
        );
        assert!(
            super::large_dripstone_height_at_radius(0.0, 5, 2.0, 1.0)
                > super::large_dripstone_height_at_radius(5.0, 5, 2.0, 1.0)
        );
        let large_dripstone_plan = super::large_dripstone_placement_plan(
            BlockPos { x: 4, y: 64, z: 4 },
            60,
            65,
            large_dripstone_config,
            0,
            &[1.0; 13],
            &[1.0; 13],
        )
        .unwrap();
        assert_eq!(
            large_dripstone_plan.stalactite.root,
            BlockPos { x: 4, y: 64, z: 4 }
        );
        assert_eq!(
            large_dripstone_plan.stalagmite.root,
            BlockPos { x: 4, y: 61, z: 4 }
        );
        assert!(large_dripstone_plan.wind_enabled);
        assert!(large_dripstone_plan
            .stalactite_blocks
            .iter()
            .any(|block| block.pos.x > 4 && !block.pointing_up));
        let full_large_dripstone_blocks = super::large_dripstone_blocks(
            super::LargeDripstoneModel {
                root: BlockPos { x: 0, y: 70, z: 0 },
                pointing_up: false,
                radius: 2,
            },
            2.0,
            1.0,
            70,
            None,
            &[1.0; 13],
            &[1.0; 13],
        );
        let shrunken_large_dripstone_blocks = super::large_dripstone_blocks(
            super::LargeDripstoneModel {
                root: BlockPos { x: 0, y: 70, z: 0 },
                pointing_up: false,
                radius: 2,
            },
            2.0,
            1.0,
            70,
            None,
            &[0.0; 13],
            &[0.0; 13],
        );
        assert!(shrunken_large_dripstone_blocks.len() < full_large_dripstone_blocks.len());
        assert_eq!(
            super::feature_size_type("two_layers_feature_size"),
            Some("minecraft:two_layers_feature_size")
        );
        assert_eq!(
            super::feature_size_type("minecraft:three_layers_feature_size"),
            Some("minecraft:three_layers_feature_size")
        );
        assert_eq!(super::feature_size_type("missing"), None);

        let two = FeatureSizeModel::TwoLayers {
            limit: 2,
            lower_size: 0,
            upper_size: 1,
            min_clipped_height: None,
        };
        assert_eq!(super::validate_feature_size(two), Ok(two));
        assert_eq!(super::feature_size_at_height(two, 7, 1), 0);
        assert_eq!(super::feature_size_at_height(two, 7, 2), 1);

        let three = FeatureSizeModel::ThreeLayers {
            limit: 1,
            upper_limit: 2,
            lower_size: 0,
            middle_size: 1,
            upper_size: 2,
            min_clipped_height: Some(80),
        };
        assert_eq!(super::validate_feature_size(three), Ok(three));
        assert_eq!(super::feature_size_at_height(three, 8, 0), 0);
        assert_eq!(super::feature_size_at_height(three, 8, 5), 1);
        assert_eq!(super::feature_size_at_height(three, 8, 6), 2);
        let leaf_updates = super::tree_leaf_distance_updates(
            &[BlockPos { x: 0, y: 0, z: 0 }],
            &[
                (BlockPos { x: 1, y: 0, z: 0 }, 7),
                (BlockPos { x: 2, y: 0, z: 0 }, 7),
                (BlockPos { x: 3, y: 0, z: 0 }, 7),
                (BlockPos { x: 4, y: 0, z: 0 }, 7),
                (BlockPos { x: 5, y: 0, z: 0 }, 7),
                (BlockPos { x: 6, y: 0, z: 0 }, 7),
                (BlockPos { x: 7, y: 0, z: 0 }, 7),
                (BlockPos { x: 1, y: 1, z: 0 }, 1),
            ],
            &[],
            &[],
        );
        assert_eq!(
            leaf_updates,
            vec![
                super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 1, y: 0, z: 0 },
                    distance: 1,
                },
                super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 1, y: 1, z: 0 },
                    distance: 1,
                },
                super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 2, y: 0, z: 0 },
                    distance: 2,
                },
                super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 3, y: 0, z: 0 },
                    distance: 3,
                },
                super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 4, y: 0, z: 0 },
                    distance: 4,
                },
                super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 5, y: 0, z: 0 },
                    distance: 5,
                },
                super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 6, y: 0, z: 0 },
                    distance: 6,
                },
            ]
        );
        assert!(super::tree_leaf_distance_updates(
            &[],
            &[(BlockPos { x: 1, y: 0, z: 0 }, 7)],
            &[],
            &[]
        )
        .is_empty());
        assert_eq!(
            super::validate_feature_size(FeatureSizeModel::TwoLayers {
                limit: 82,
                lower_size: 0,
                upper_size: 1,
                min_clipped_height: None,
            })
            .unwrap_err(),
            "feature size fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::validate_feature_size(FeatureSizeModel::ThreeLayers {
                limit: 1,
                upper_limit: 1,
                lower_size: 0,
                middle_size: 1,
                upper_size: 2,
                min_clipped_height: Some(81),
            })
            .unwrap_err(),
            "min_clipped_height must be in 0..=80".to_string()
        );
        assert_eq!(
            [
                "trunk_vine",
                "leave_vine",
                "pale_moss",
                "creaking_heart",
                "cocoa",
                "beehive",
                "alter_ground",
                "attached_to_leaves",
                "place_on_ground",
                "attached_to_logs",
            ]
            .iter()
            .filter_map(|decorator_type| super::tree_decorator_type(decorator_type))
            .collect::<Vec<_>>(),
            vec![
                "minecraft:trunk_vine",
                "minecraft:leave_vine",
                "minecraft:pale_moss",
                "minecraft:creaking_heart",
                "minecraft:cocoa",
                "minecraft:beehive",
                "minecraft:alter_ground",
                "minecraft:attached_to_leaves",
                "minecraft:place_on_ground",
                "minecraft:attached_to_logs",
            ]
        );
        assert_eq!(
            super::tree_decorator_type("minecraft:attached_to_logs"),
            Some("minecraft:attached_to_logs")
        );
        assert_eq!(super::tree_decorator_type("missing"), None);
        assert_eq!(
            super::trunk_placer_type("straight_trunk_placer"),
            Some("minecraft:straight_trunk_placer")
        );
        assert_eq!(
            super::foliage_placer_type("minecraft:cherry_foliage_placer"),
            Some("minecraft:cherry_foliage_placer")
        );
        assert_eq!(
            super::root_placer_type("mangrove_root_placer"),
            Some("minecraft:mangrove_root_placer")
        );
        let straight_trunk = TrunkPlacerModel {
            base_height: 5,
            height_rand_a: 2,
            height_rand_b: 1,
            kind: TrunkPlacerKind::Straight,
        };
        assert_eq!(
            super::validate_trunk_placer(straight_trunk),
            Ok(straight_trunk)
        );
        assert_eq!(super::trunk_placer_height(straight_trunk, 1, 1), 7);
        assert!(super::tree_valid_pos("minecraft:air"));
        assert!(super::tree_valid_pos("minecraft:oak_leaves"));
        assert!(super::tree_valid_pos("minecraft:dandelion"));
        assert!(super::tree_valid_pos("minecraft:water"));
        assert!(super::tree_valid_pos("minecraft:vine"));
        assert!(super::tree_valid_pos("minecraft:hanging_roots"));
        assert!(super::tree_valid_pos("minecraft:bush"));
        assert!(super::tree_valid_pos("minecraft:pale_moss_carpet"));
        assert!(super::tree_valid_pos("minecraft:short_dry_grass"));
        assert!(super::block_matches_tag(
            "minecraft:dirt",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(super::block_matches_tag(
            "minecraft:podzol",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(super::block_matches_tag(
            "minecraft:moss_block",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(!super::block_matches_tag(
            "minecraft:grass_block",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(!super::block_matches_tag(
            "minecraft:farmland",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(!super::tree_valid_pos("minecraft:oak_sapling"));
        assert!(!super::tree_valid_pos("minecraft:stone"));
        assert!(super::tree_trunk_free_pos("minecraft:oak_log"));
        assert!(super::tree_trunk_free_pos("minecraft:birch_log[axis=y]"));
        assert!(!super::tree_valid_pos("minecraft:oak_log"));
        let min_size = FeatureSizeModel::TwoLayers {
            limit: 1,
            lower_size: 0,
            upper_size: 1,
            min_clipped_height: Some(3),
        };
        let free_row = ["minecraft:air"; 9];
        let vine_row = ["minecraft:vine"; 9];
        let log_row = ["minecraft:oak_log"; 9];
        let stone_row = ["minecraft:stone"; 9];
        assert_eq!(
            super::tree_max_free_height(
                5,
                min_size,
                &[&free_row, &log_row, &log_row, &log_row, &log_row, &log_row, &log_row],
                true,
            ),
            5
        );
        assert_eq!(
            super::tree_max_free_height(
                5,
                min_size,
                &[&free_row, &free_row, &free_row, &stone_row],
                true,
            ),
            1
        );
        assert_eq!(
            super::tree_max_free_height(5, min_size, &[&vine_row], false),
            -2
        );
        assert!(super::tree_can_place(
            BlockPos { x: 0, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            5,
            min_size,
            Some(3),
            -64,
            320,
            &[&free_row, &free_row, &free_row, &free_row, &free_row, &free_row, &free_row,],
            true,
        ));
        assert!(!super::tree_can_place(
            BlockPos { x: 0, y: -64, z: 0 },
            BlockPos { x: 0, y: -64, z: 0 },
            5,
            min_size,
            Some(3),
            -64,
            320,
            &[&free_row],
            true,
        ));
        assert_eq!(
            super::validate_trunk_placer(TrunkPlacerModel {
                base_height: 33,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            })
            .unwrap_err(),
            "trunk placer base fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::validate_trunk_placer(TrunkPlacerModel {
                base_height: 5,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Cherry {
                    branch_count_min: 1,
                    branch_count_max: 3,
                    branch_horizontal_length_min: 2,
                    branch_horizontal_length_max: 16,
                    branch_start_offset_from_top_min: -1,
                    branch_start_offset_from_top_max: -1,
                    branch_end_offset_from_top_min: -16,
                    branch_end_offset_from_top_max: 16,
                },
            })
            .unwrap_err(),
            "trunk placer variant fields are outside vanilla codec ranges".to_string()
        );
        let blob_foliage = FoliagePlacerModel {
            radius_min: 1,
            radius_max: 2,
            offset_min: 0,
            offset_max: 1,
            kind: FoliagePlacerKind::Blob { height: 3 },
        };
        assert_eq!(
            super::validate_foliage_placer(blob_foliage),
            Ok(blob_foliage)
        );
        let max_height_pine = FoliagePlacerModel {
            radius_min: 0,
            radius_max: 16,
            offset_min: 0,
            offset_max: 16,
            kind: FoliagePlacerKind::Pine {
                height_min: 0,
                height_max: 24,
            },
        };
        assert_eq!(
            super::validate_foliage_placer(max_height_pine),
            Ok(max_height_pine)
        );
        assert_eq!(
            super::validate_foliage_placer(FoliagePlacerModel {
                kind: FoliagePlacerKind::Spruce {
                    height_min: 0,
                    height_max: 25,
                },
                ..max_height_pine
            })
            .unwrap_err(),
            "foliage placer variant fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::validate_foliage_placer(FoliagePlacerModel {
                radius_min: 0,
                radius_max: 16,
                offset_min: 0,
                offset_max: 16,
                kind: FoliagePlacerKind::RandomSpread {
                    foliage_height_min: 0,
                    foliage_height_max: 1,
                    leaf_placement_attempts: 1,
                },
            })
            .unwrap_err(),
            "foliage placer variant fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::validate_foliage_placer(FoliagePlacerModel {
                radius_min: 0,
                radius_max: 16,
                offset_min: 0,
                offset_max: 16,
                kind: FoliagePlacerKind::Cherry {
                    height: 3,
                    wide_bottom_layer_hole_chance: 0.0,
                    corner_hole_chance: 0.0,
                    hanging_leaves_chance: 0.0,
                    hanging_leaves_extension_chance: 0.0,
                },
            })
            .unwrap_err(),
            "foliage placer variant fields are outside vanilla codec ranges".to_string()
        );
        let mangrove_root = RootPlacerModel {
            above_root_placement_chance: Some(0.5),
            mangrove_root_placement: MangroveRootPlacementModel {
                max_root_width: 8,
                max_root_length: 15,
                random_skew_chance: 0.2,
            },
        };
        assert_eq!(
            super::validate_root_placer(mangrove_root),
            Ok(mangrove_root)
        );
        assert_eq!(
            super::validate_root_placer(RootPlacerModel {
                above_root_placement_chance: Some(1.25),
                mangrove_root_placement: MangroveRootPlacementModel {
                    max_root_width: 8,
                    max_root_length: 15,
                    random_skew_chance: 0.2,
                },
            })
            .unwrap_err(),
            "root placer fields are outside vanilla codec ranges".to_string()
        );
        let root_system_config = super::RootSystemConfigurationModel {
            tree_feature: "minecraft:azalea_tree",
            required_vertical_space_for_tree: 3,
            root_radius: 3,
            root_replaceable: "#minecraft:dirt",
            root_state_provider: BlockStateProviderModel::Simple("minecraft:rooted_dirt"),
            root_placement_attempts: 2,
            root_column_max_height: 4,
            hanging_root_radius: 3,
            hanging_roots_vertical_span: 2,
            hanging_root_state_provider: BlockStateProviderModel::Simple("minecraft:hanging_roots"),
            hanging_root_placement_attempts: 2,
            allowed_vertical_water_for_tree: 2,
        };
        assert_eq!(
            super::validate_root_system_configuration(&root_system_config),
            Ok(())
        );
        assert_eq!(
            super::mangrove_potential_root_positions(
                BlockPos {
                    x: 404,
                    y: 64,
                    z: 400,
                },
                HorizontalDirection::East,
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                4,
                0.5,
                0.25,
                false,
            ),
            vec![
                BlockPos {
                    x: 404,
                    y: 63,
                    z: 400,
                },
                BlockPos {
                    x: 405,
                    y: 63,
                    z: 400,
                },
            ]
        );
        assert_eq!(
            super::mangrove_potential_root_positions(
                BlockPos {
                    x: 401,
                    y: 64,
                    z: 400,
                },
                HorizontalDirection::East,
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                4,
                0.0,
                0.5,
                true,
            ),
            vec![BlockPos {
                x: 402,
                y: 64,
                z: 400,
            }]
        );
        assert_eq!(
            super::mangrove_potential_root_positions(
                BlockPos {
                    x: 406,
                    y: 64,
                    z: 400,
                },
                HorizontalDirection::East,
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                4,
                1.0,
                0.0,
                true,
            ),
            vec![BlockPos {
                x: 406,
                y: 63,
                z: 400,
            }]
        );
        assert!(super::root_system_is_allowed_tree_space(
            "minecraft:water",
            1,
            2
        ));
        assert!(!super::root_system_is_allowed_tree_space(
            "minecraft:water",
            2,
            2
        ));
        let root_plan = super::root_system_placement_plan(
            BlockPos {
                x: 400,
                y: 64,
                z: 400,
            },
            true,
            &root_system_config,
            &[
                super::RootSystemTreeCandidateModel {
                    pos: BlockPos {
                        x: 400,
                        y: 65,
                        z: 400,
                    },
                    allowed_tree_position: true,
                    vertical_space_states: vec![
                        "minecraft:air",
                        "minecraft:water",
                        "minecraft:air",
                    ],
                    below_state: "minecraft:stone",
                    tree_feature_places: false,
                },
                super::RootSystemTreeCandidateModel {
                    pos: BlockPos {
                        x: 400,
                        y: 66,
                        z: 400,
                    },
                    allowed_tree_position: true,
                    vertical_space_states: vec!["minecraft:air", "minecraft:air", "minecraft:air"],
                    below_state: "minecraft:dirt",
                    tree_feature_places: true,
                },
            ],
            &[
                super::RootSystemOffsetRoll {
                    positive_x: 2,
                    negative_x: 1,
                    positive_z: 1,
                    ..Default::default()
                },
                super::RootSystemOffsetRoll {
                    positive_x: 0,
                    negative_x: 0,
                    positive_z: 0,
                    negative_z: 1,
                    ..Default::default()
                },
            ],
            &[
                super::RootSystemOffsetRoll {
                    positive_x: 1,
                    positive_y: 1,
                    positive_z: 0,
                    ..Default::default()
                },
                super::RootSystemOffsetRoll {
                    negative_x: 1,
                    negative_y: 1,
                    negative_z: 1,
                    ..Default::default()
                },
            ],
            &[
                BlockPos {
                    x: 401,
                    y: 64,
                    z: 401,
                },
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 399,
                },
            ],
            &[BlockPos {
                x: 401,
                y: 65,
                z: 400,
            }],
        )
        .unwrap();
        assert_eq!(
            root_plan.tree_origin,
            Some(BlockPos {
                x: 400,
                y: 66,
                z: 400,
            })
        );
        assert_eq!(
            root_plan.blocks,
            vec![
                super::RootSystemPlacementBlock {
                    pos: BlockPos {
                        x: 401,
                        y: 64,
                        z: 401,
                    },
                    state: "minecraft:rooted_dirt",
                    kind: super::RootSystemPlacementKind::RootedDirt,
                },
                super::RootSystemPlacementBlock {
                    pos: BlockPos {
                        x: 400,
                        y: 64,
                        z: 399,
                    },
                    state: "minecraft:rooted_dirt",
                    kind: super::RootSystemPlacementKind::RootedDirt,
                },
                super::RootSystemPlacementBlock {
                    pos: BlockPos {
                        x: 401,
                        y: 65,
                        z: 400,
                    },
                    state: "minecraft:hanging_roots",
                    kind: super::RootSystemPlacementKind::HangingRoot,
                },
            ]
        );
        assert!(
            !super::root_system_placement_plan(
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                false,
                &root_system_config,
                &[],
                &[],
                &[],
                &[],
                &[],
            )
            .unwrap()
            .attempted_roots
        );
        let tree_plan = super::simple_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            straight_trunk,
            FoliagePlacerModel {
                offset_min: 0,
                offset_max: 0,
                ..blob_foliage
            },
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:dirt",
            1,
            1,
        )
        .unwrap();
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk
                && block.pos == BlockPos { x: 8, y: 63, z: 8 }
                && block.state == "minecraft:dirt"
        }));
        assert_eq!(
            tree_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::Log)
                .count(),
            7
        );
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 8, y: 71, z: 8 }
                && block.state == "minecraft:oak_leaves"
        }));
        assert!(!tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 7, y: 71, z: 7 }
        }));
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 6, y: 68, z: 8 }
        }));
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 10, y: 68, z: 8 }
        }));
        let tree_config = super::TreeConfigurationModel {
            trunk_provider: BlockStateProviderModel::Simple("minecraft:oak_log"),
            foliage_provider: BlockStateProviderModel::Simple("minecraft:oak_leaves"),
            dirt_provider: BlockStateProviderModel::Simple("minecraft:dirt"),
            trunk_placer: straight_trunk,
            foliage_placer: FoliagePlacerModel {
                offset_min: 0,
                offset_max: 0,
                ..blob_foliage
            },
            minimum_size: min_size,
            root_placer: Some(mangrove_root),
            decorators: vec![TreeDecoratorModel::Cocoa { probability: 0.25 }],
            ignore_vines: false,
        };
        assert_eq!(super::validate_tree_configuration(&tree_config), Ok(()));
        let full_tree_rows = [
            &free_row[..],
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
        ];
        let configured_plan = super::configured_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            &tree_config,
            -64,
            320,
            &full_tree_rows,
            1,
            1,
        )
        .unwrap()
        .expect("valid tree config should place");
        assert!(configured_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk && block.state == "minecraft:dirt"
        }));
        assert!(super::configured_tree_placement_plan(
            BlockPos { x: 8, y: -64, z: 8 },
            &tree_config,
            -64,
            320,
            &full_tree_rows,
            1,
            1,
        )
        .unwrap()
        .is_none());
        let vine_blocked_rows = [&vine_row[..]];
        assert!(super::configured_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            &tree_config,
            -64,
            320,
            &vine_blocked_rows,
            1,
            1,
        )
        .unwrap()
        .is_none());
        let clipped_rows = [
            &free_row[..],
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &stone_row,
        ];
        assert!(super::configured_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            &tree_config,
            -64,
            320,
            &clipped_rows,
            1,
            1,
        )
        .unwrap()
        .is_some());
        assert_eq!(
            super::validate_tree_configuration(&super::TreeConfigurationModel {
                decorators: vec![TreeDecoratorModel::Cocoa { probability: 1.25 }],
                ..tree_config.clone()
            })
            .unwrap_err(),
            "tree decorator probability must be in 0.0..=1.0".to_string()
        );
        let bush_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 20,
                y: 64,
                z: 20,
            },
            TrunkPlacerModel {
                base_height: 3,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 1,
                offset_max: 1,
                kind: FoliagePlacerKind::Bush { height: 2 },
            },
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:dirt",
            2,
            4,
        )
        .unwrap();
        assert!(bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 20,
                        y: 68,
                        z: 20,
                    }
        }));
        assert!(!bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 21,
                        y: 68,
                        z: 20,
                    }
        }));
        assert!(bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 21,
                        y: 67,
                        z: 20,
                    }
        }));
        assert!(!bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 22,
                        y: 67,
                        z: 20,
                    }
        }));
        assert!(bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 22,
                        y: 66,
                        z: 20,
                    }
        }));
        let acacia_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 40,
                y: 64,
                z: 40,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Acacia,
            },
            "minecraft:acacia_log",
            "minecraft:acacia_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert!(acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 42,
                        y: 67,
                        z: 40,
                    }
        }));
        assert!(!acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 42,
                        y: 67,
                        z: 42,
                    }
        }));
        assert!(acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 41,
                        y: 68,
                        z: 41,
                    }
        }));
        assert!(acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 41,
                        y: 68,
                        z: 40,
                    }
        }));
        let dark_oak_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 60,
                y: 64,
                z: 60,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::DarkOak,
            },
            "minecraft:dark_oak_log",
            "minecraft:dark_oak_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert!(dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 64,
                        y: 67,
                        z: 60,
                    }
        }));
        assert!(!dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 64,
                        y: 67,
                        z: 64,
                    }
        }));
        assert!(dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 63,
                        y: 68,
                        z: 63,
                    }
        }));
        let fancy_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 70,
                y: 64,
                z: 70,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Fancy { height: 2 },
            },
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::fancy_foliage_rows(0, 2, 2),
            vec![(0, 2), (-1, 3), (-2, 2)]
        );
        assert!(fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 72,
                        y: 67,
                        z: 70,
                    }
        }));
        assert!(!fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 73,
                        y: 67,
                        z: 70,
                    }
        }));
        assert!(!fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 68,
                        y: 68,
                        z: 70,
                    }
        }));
        assert!(!fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 72,
                        y: 68,
                        z: 70,
                    }
        }));
        assert!(super::fancy_leaves_row_should_skip(-2, 0, 2));
        assert!(super::fancy_leaves_row_should_skip(2, 0, 2));
        let mega_jungle_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 140,
                y: 64,
                z: 140,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Jungle { height: 4 },
            },
            "minecraft:jungle_log",
            "minecraft:jungle_leaves",
            "minecraft:dirt",
            0,
            1,
        )
        .unwrap();
        assert_eq!(
            super::mega_jungle_foliage_rows(0, 2, 2),
            vec![(0, 3), (-1, 4), (-2, 5)]
        );
        assert!(mega_jungle_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 145,
                        y: 66,
                        z: 140,
                    }
        }));
        assert!(!mega_jungle_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 145,
                        y: 66,
                        z: 145,
                    }
        }));
        let random_spread_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 160,
                y: 64,
                z: 160,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 3,
                radius_max: 3,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::RandomSpread {
                    foliage_height_min: 4,
                    foliage_height_max: 4,
                    leaf_placement_attempts: 4,
                },
            },
            "minecraft:azalea_log",
            "minecraft:azalea_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::random_spread_foliage_positions(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                4,
                3,
                2,
                &[2, 0, 3, 1, 1, 0, 0, 2, 0, 3, 0, 1],
            ),
            vec![
                BlockPos {
                    x: 12,
                    y: 22,
                    z: 31
                },
                BlockPos { x: 8, y: 17, z: 29 },
            ]
        );
        assert_eq!(
            random_spread_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
                .count(),
            4
        );
        let cherry_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 180,
                y: 64,
                z: 180,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 4,
                radius_max: 4,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Cherry {
                    height: 5,
                    wide_bottom_layer_hole_chance: 0.0,
                    corner_hole_chance: 0.0,
                    hanging_leaves_chance: 0.0,
                    hanging_leaves_extension_chance: 0.0,
                },
            },
            "minecraft:cherry_log",
            "minecraft:cherry_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::cherry_foliage_rows(5, 4),
            vec![(2, 1), (1, 2), (0, 3), (-1, 3), (-2, 2)]
        );
        assert!(cherry_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 183,
                        y: 68,
                        z: 182,
                    }
        }));
        assert!(!cherry_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 183,
                        y: 68,
                        z: 183,
                    }
        }));
        assert!(!super::cherry_leaves_row_should_skip(
            3, -1, 0, 3, 0.0, 0.0, 0, 0
        ));
        assert!(super::cherry_leaves_row_should_skip(
            3, -1, 0, 3, 1.0, 0.0, 0, 0
        ));
        let pine_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 80,
                y: 64,
                z: 80,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Pine {
                    height_min: 3,
                    height_max: 3,
                },
            },
            "minecraft:spruce_log",
            "minecraft:spruce_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::pine_foliage_rows(0, 3, 2),
            vec![(0, 0), (-1, 1), (-2, 2), (-3, 1)]
        );
        assert!(pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 82,
                        y: 66,
                        z: 80,
                    }
        }));
        assert!(!pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 82,
                        y: 66,
                        z: 82,
                    }
        }));
        assert!(pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 81,
                        y: 65,
                        z: 80,
                    }
        }));
        let spruce_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 100,
                y: 64,
                z: 100,
            },
            TrunkPlacerModel {
                base_height: 6,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Spruce {
                    height_min: 2,
                    height_max: 2,
                },
            },
            "minecraft:spruce_log",
            "minecraft:spruce_leaves",
            "minecraft:dirt",
            0,
            1,
        )
        .unwrap();
        assert_eq!(
            super::spruce_foliage_rows(0, 4, 2, 1),
            vec![(0, 1), (-1, 0), (-2, 1), (-3, 2), (-4, 1)]
        );
        assert!(spruce_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 102,
                        y: 67,
                        z: 100,
                    }
        }));
        assert!(!spruce_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 102,
                        y: 67,
                        z: 102,
                    }
        }));
        assert!(spruce_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 101,
                        y: 66,
                        z: 100,
                    }
        }));
        let mega_pine_plan = super::simple_tree_placement_plan(
            BlockPos {
                x: 120,
                y: 64,
                z: 120,
            },
            TrunkPlacerModel {
                base_height: 6,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::MegaPine {
                    height_min: 6,
                    height_max: 6,
                },
            },
            "minecraft:spruce_log",
            "minecraft:spruce_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::mega_pine_foliage_rows(70, 0, 6, 2),
            vec![(-6, 5), (-5, 4), (-4, 5), (-3, 3), (-2, 4), (-1, 2), (0, 2)]
        );
        assert!(mega_pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 124,
                        y: 68,
                        z: 120,
                    }
        }));
        assert!(!mega_pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 124,
                        y: 68,
                        z: 124,
                    }
        }));
        assert!(!super::mega_pine_leaves_row_should_skip(4, 0, 4));
        assert!(super::mega_pine_leaves_row_should_skip(4, 4, 5));
        assert_eq!(
            super::simple_tree_placement_plan(
                BlockPos { x: 8, y: 64, z: 8 },
                TrunkPlacerModel {
                    base_height: 5,
                    height_rand_a: 0,
                    height_rand_b: 0,
                    kind: TrunkPlacerKind::Forking,
                },
                blob_foliage,
                "minecraft:oak_log",
                "minecraft:oak_leaves",
                "minecraft:dirt",
                0,
                0,
            )
            .unwrap_err(),
            "only straight trunk placement is modeled by simple_tree_placement_plan".to_string()
        );
        let forking_plan = super::forking_trunk_placement_plan(
            BlockPos {
                x: 200,
                y: 64,
                z: 200,
            },
            6,
            "minecraft:oak_log",
            "minecraft:dirt",
            HorizontalDirection::East,
            HorizontalDirection::North,
            1,
            1,
            0,
            2,
        );
        assert!(forking_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 202,
                        y: 69,
                        z: 200,
                    }
        }));
        assert!(forking_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 200,
                        y: 69,
                        z: 197,
                    }
        }));
        assert_eq!(
            forking_plan.attachments,
            vec![
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 202,
                        y: 70,
                        z: 200,
                    },
                    radius_offset: 1,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 200,
                        y: 70,
                        z: 197,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
            ]
        );
        assert_eq!(
            super::forking_trunk_placement_plan(
                BlockPos {
                    x: 200,
                    y: 64,
                    z: 200,
                },
                6,
                "minecraft:oak_log",
                "minecraft:dirt",
                HorizontalDirection::East,
                HorizontalDirection::East,
                1,
                1,
                0,
                2,
            )
            .attachments
            .len(),
            1
        );
        let bending_plan = super::bending_trunk_placement_plan(
            BlockPos {
                x: 220,
                y: 64,
                z: 220,
            },
            5,
            "minecraft:oak_log",
            "minecraft:dirt",
            HorizontalDirection::South,
            2,
            2,
            0,
        );
        assert!(bending_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 220,
                        y: 67,
                        z: 221,
                    }
        }));
        assert!(bending_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 220,
                        y: 69,
                        z: 224,
                    }
        }));
        assert_eq!(
            bending_plan.attachments.first().copied(),
            Some(TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 220,
                    y: 66,
                    z: 220,
                },
                radius_offset: 0,
                double_trunk: false,
            })
        );
        assert_eq!(
            bending_plan.attachments.last().copied(),
            Some(TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 220,
                    y: 69,
                    z: 224,
                },
                radius_offset: 0,
                double_trunk: false,
            })
        );
        let giant_plan = super::giant_trunk_placement_plan(
            BlockPos {
                x: 240,
                y: 64,
                z: 240,
            },
            3,
            "minecraft:jungle_log",
            "minecraft:dirt",
        );
        assert_eq!(
            giant_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::DirtBelowTrunk)
                .count(),
            4
        );
        assert_eq!(
            giant_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::Log)
                .count(),
            9
        );
        assert!(giant_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk
                && block.pos
                    == BlockPos {
                        x: 241,
                        y: 63,
                        z: 241,
                    }
        }));
        assert!(giant_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 241,
                        y: 65,
                        z: 241,
                    }
        }));
        assert!(!giant_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 241,
                        y: 66,
                        z: 241,
                    }
        }));
        assert_eq!(
            giant_plan.attachments,
            vec![TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 240,
                    y: 67,
                    z: 240,
                },
                radius_offset: 0,
                double_trunk: true,
            }]
        );
        let mega_jungle_trunk_plan = super::mega_jungle_trunk_placement_plan(
            BlockPos {
                x: 250,
                y: 64,
                z: 250,
            },
            6,
            "minecraft:jungle_log",
            "minecraft:dirt",
            &[super::MegaJungleBranchModel {
                branch_height: 4,
                angle_radians: 0.0,
            }],
        );
        assert!(mega_jungle_trunk_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 255,
                        y: 67,
                        z: 251,
                    }
        }));
        assert_eq!(
            mega_jungle_trunk_plan.attachments,
            vec![
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 250,
                        y: 70,
                        z: 250,
                    },
                    radius_offset: 0,
                    double_trunk: true,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 255,
                        y: 68,
                        z: 251,
                    },
                    radius_offset: -2,
                    double_trunk: false,
                },
            ]
        );
        let dark_oak_plan = super::dark_oak_trunk_placement_plan(
            BlockPos {
                x: 260,
                y: 64,
                z: 260,
            },
            6,
            "minecraft:dark_oak_log",
            "minecraft:dirt",
            HorizontalDirection::East,
            1,
            1,
            &[1, 1, 1, 1, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1],
        );
        assert_eq!(
            dark_oak_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::DirtBelowTrunk)
                .count(),
            4
        );
        assert!(dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 262,
                        y: 69,
                        z: 261,
                    }
        }));
        assert!(dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 260,
                        y: 68,
                        z: 259,
                    }
        }));
        assert!(dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 260,
                        y: 65,
                        z: 259,
                    }
        }));
        assert_eq!(
            dark_oak_plan.attachments,
            vec![
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 261,
                        y: 69,
                        z: 260,
                    },
                    radius_offset: 0,
                    double_trunk: true,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 260,
                        y: 69,
                        z: 259,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
            ]
        );
        let upwards_branching_plan = super::upwards_branching_trunk_placement_plan(
            BlockPos {
                x: 280,
                y: 64,
                z: 280,
            },
            5,
            "minecraft:spruce_log",
            &[super::UpwardsBranchingBranchModel {
                trunk_y_offset: 1,
                direction: HorizontalDirection::North,
                branch_pos: 1,
                branch_steps: 3,
            }],
        );
        assert_eq!(
            upwards_branching_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::Log)
                .count(),
            8
        );
        assert!(upwards_branching_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.pos
                    == BlockPos {
                        x: 280,
                        y: 67,
                        z: 278,
                    }
        }));
        assert_eq!(
            upwards_branching_plan.attachments,
            vec![
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 280,
                        y: 66,
                        z: 279,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 280,
                        y: 67,
                        z: 278,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 280,
                        y: 68,
                        z: 277,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 280,
                        y: 69,
                        z: 277,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 280,
                        y: 67,
                        z: 277,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 280,
                        y: 69,
                        z: 280,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
            ]
        );
        let cherry_trunk_plan = super::cherry_trunk_placement_plan(
            BlockPos {
                x: 300,
                y: 64,
                z: 300,
            },
            6,
            "minecraft:cherry_log",
            "minecraft:dirt",
            2,
            &[
                super::CherryBranchModel {
                    start_offset_from_origin: 3,
                    direction: HorizontalDirection::East,
                    horizontal_length: 2,
                    end_offset_from_origin: 5,
                    middle_continues_upwards: false,
                    grow_vertically: vec![false, true, true],
                },
                super::CherryBranchModel {
                    start_offset_from_origin: 2,
                    direction: HorizontalDirection::West,
                    horizontal_length: 2,
                    end_offset_from_origin: 1,
                    middle_continues_upwards: true,
                    grow_vertically: vec![true, false],
                },
            ],
        );
        assert!(cherry_trunk_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk
                && block.pos
                    == BlockPos {
                        x: 300,
                        y: 63,
                        z: 300,
                    }
        }));
        assert!(cherry_trunk_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.state == "minecraft:cherry_log[axis=x]"
                && block.pos
                    == BlockPos {
                        x: 302,
                        y: 67,
                        z: 300,
                    }
        }));
        assert!(cherry_trunk_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.state == "minecraft:cherry_log"
                && block.pos
                    == BlockPos {
                        x: 302,
                        y: 69,
                        z: 300,
                    }
        }));
        assert_eq!(
            cherry_trunk_plan.attachments,
            vec![
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 302,
                        y: 70,
                        z: 300,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 297,
                        y: 66,
                        z: 300,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
            ]
        );
        let fancy_trunk_plan = super::fancy_trunk_placement_plan(
            BlockPos {
                x: 320,
                y: 64,
                z: 320,
            },
            8,
            "minecraft:oak_log",
            "minecraft:dirt",
            &[super::FancyTrunkClusterRollModel {
                shape_float: 0.5,
                angle_float: 0.25,
            }],
        );
        assert_eq!(super::fancy_trunk_tree_shape(10, 2), -1.0);
        assert_eq!(super::fancy_trunk_tree_shape(10, 0), -1.0);
        assert_eq!(super::fancy_trunk_cluster_roll_count(8), 3);
        assert!(fancy_trunk_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk
                && block.pos
                    == BlockPos {
                        x: 320,
                        y: 63,
                        z: 320,
                    }
        }));
        assert!(fancy_trunk_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.state == "minecraft:oak_log"
                && block.pos
                    == BlockPos {
                        x: 320,
                        y: 70,
                        z: 320,
                    }
        }));
        assert!(fancy_trunk_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Log
                && block.state == "minecraft:oak_log[axis=x]"
                && block.pos
                    == BlockPos {
                        x: 322,
                        y: 68,
                        z: 320,
                    }
        }));
        assert_eq!(
            fancy_trunk_plan.attachments,
            vec![
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 320,
                        y: 69,
                        z: 320,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 322,
                        y: 68,
                        z: 320,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
                TreeFoliageAttachmentModel {
                    pos: BlockPos {
                        x: 320,
                        y: 67,
                        z: 321,
                    },
                    radius_offset: 0,
                    double_trunk: false,
                },
            ]
        );
        let fallen_config = super::FallenTreeConfigurationModel {
            trunk_provider: BlockStateProviderModel::Simple("minecraft:oak_log"),
            min_log_length: 4,
            max_log_length: 7,
            stump_decorators: vec![TreeDecoratorModel::TrunkVine],
            log_decorators: vec![TreeDecoratorModel::AttachedToLogs { probability: 0.1 }],
        };
        assert_eq!(super::fallen_tree_log_length(4, 7, 0), 2);
        assert_eq!(
            super::fallen_tree_start_pos(
                BlockPos { x: 0, y: 64, z: 0 },
                super::HorizontalDirection::East,
                1,
                &[false, true],
            ),
            Some(BlockPos { x: 3, y: 64, z: 0 })
        );
        assert!(super::fallen_tree_can_place_log(
            &[true, true, true, true],
            &[true, false, false, true],
        ));
        assert!(!super::fallen_tree_can_place_log(
            &[true, true, true],
            &[false, false, false],
        ));
        let fallen_plan = super::fallen_tree_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &fallen_config,
            super::HorizontalDirection::East,
            1,
            0,
            &[true],
            &[true, true, true],
            &[true, true, true],
        )
        .unwrap();
        assert_eq!(fallen_plan.stump_decorators, 1);
        assert_eq!(fallen_plan.log_decorators, 1);
        assert_eq!(
            fallen_plan.blocks[0],
            super::FallenTreeBlock {
                pos: BlockPos { x: 0, y: 64, z: 0 },
                state: "minecraft:oak_log",
                mark_above_for_post_processing: true,
            }
        );
        assert!(fallen_plan.blocks.contains(&super::FallenTreeBlock {
            pos: BlockPos { x: 2, y: 65, z: 0 },
            state: "minecraft:oak_log[axis=x]",
            mark_above_for_post_processing: true,
        }));
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::Cocoa { probability: 0.25 }),
            Ok(TreeDecoratorModel::Cocoa { probability: 0.25 })
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::LeaveVine { probability: 1.0 }),
            Ok(TreeDecoratorModel::LeaveVine { probability: 1.0 })
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::CreakingHeart { probability: 0.0 }),
            Ok(TreeDecoratorModel::CreakingHeart { probability: 0.0 })
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::AttachedToLeaves {
                probability: 1.0
            }),
            Ok(TreeDecoratorModel::AttachedToLeaves { probability: 1.0 })
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::AttachedToLogs { probability: 0.0 }),
            Ok(TreeDecoratorModel::AttachedToLogs { probability: 0.0 })
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::PaleMoss {
                leaves_probability: 0.25,
                trunk_probability: 0.5,
                ground_probability: 1.0,
            }),
            Ok(TreeDecoratorModel::PaleMoss {
                leaves_probability: 0.25,
                trunk_probability: 0.5,
                ground_probability: 1.0,
            })
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::PaleMoss {
                leaves_probability: 1.1,
                trunk_probability: 0.5,
                ground_probability: 1.0,
            })
            .unwrap_err(),
            "pale moss decorator probabilities must be in 0.0..=1.0".to_string()
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::Beehive { probability: 1.5 })
                .unwrap_err(),
            "tree decorator probability must be in 0.0..=1.0".to_string()
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::AttachedToLogs {
                probability: -0.1
            })
            .unwrap_err(),
            "tree decorator probability must be in 0.0..=1.0".to_string()
        );
        assert!(super::tree_decorator_should_place(0.25, 0.249));
        assert!(!super::tree_decorator_should_place(0.25, 0.25));
        assert_eq!(
            super::tree_lowest_trunk_or_root_positions(
                &[BlockPos { x: 0, y: 3, z: 0 }, BlockPos { x: 0, y: 1, z: 0 },],
                &[],
            ),
            vec![BlockPos { x: 0, y: 1, z: 0 }, BlockPos { x: 0, y: 3, z: 0 },]
        );
        assert_eq!(
            super::tree_lowest_trunk_or_root_positions(
                &[BlockPos { x: 0, y: 1, z: 0 }],
                &[
                    BlockPos { x: 1, y: 0, z: 0 },
                    BlockPos { x: 1, y: -1, z: 0 },
                ],
            ),
            vec![
                BlockPos { x: 1, y: -1, z: 0 },
                BlockPos { x: 1, y: 0, z: 0 },
            ]
        );
        assert_eq!(
            super::tree_lowest_trunk_or_root_positions(
                &[BlockPos { x: 0, y: 1, z: 0 }],
                &[BlockPos { x: 1, y: 1, z: 0 }],
            ),
            vec![BlockPos { x: 0, y: 1, z: 0 }, BlockPos { x: 1, y: 1, z: 0 },]
        );
        assert_eq!(
            super::trunk_vine_decorator_placement(
                &[
                    super::TrunkVineLogContext {
                        pos: BlockPos {
                            x: 10,
                            y: 66,
                            z: 10
                        },
                        west_air: true,
                        east_air: true,
                        north_air: true,
                        south_air: true,
                    },
                    super::TrunkVineLogContext {
                        pos: BlockPos {
                            x: 10,
                            y: 64,
                            z: 10
                        },
                        west_air: true,
                        east_air: false,
                        north_air: true,
                        south_air: true,
                    },
                ],
                &[1, 2, 0, 1, 0, 1, 2, 0],
            ),
            vec![
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 9, y: 64, z: 10 },
                    state: "minecraft:vine[east=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 10,
                        y: 64,
                        z: 11
                    },
                    state: "minecraft:vine[north=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 11,
                        y: 66,
                        z: 10
                    },
                    state: "minecraft:vine[west=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 10, y: 66, z: 9 },
                    state: "minecraft:vine[south=true]",
                },
            ]
        );
        assert_eq!(
            super::leave_vine_decorator_placement(
                &[
                    super::LeaveVineLeafContext {
                        pos: BlockPos {
                            x: 30,
                            y: 70,
                            z: 30
                        },
                        west_air: true,
                        east_air: true,
                        north_air: true,
                        south_air: true,
                        west_below_air: [true, true, false, true],
                        east_below_air: [false, true, true, true],
                        north_below_air: [true, true, true, true],
                        south_below_air: [true, true, true, true],
                    },
                    super::LeaveVineLeafContext {
                        pos: BlockPos {
                            x: 30,
                            y: 68,
                            z: 30
                        },
                        west_air: true,
                        east_air: false,
                        north_air: true,
                        south_air: true,
                        west_below_air: [true, false, true, true],
                        east_below_air: [true, true, true, true],
                        north_below_air: [true, true, true, true],
                        south_below_air: [true, true, true, true],
                    },
                ],
                0.5,
                &[0.1, 0.2, 0.6, 0.5, 0.6, 0.6, 0.0, 0.7],
            ),
            vec![
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 29,
                        y: 68,
                        z: 30
                    },
                    state: "minecraft:vine[east=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 29,
                        y: 67,
                        z: 30
                    },
                    state: "minecraft:vine[east=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 30,
                        y: 70,
                        z: 29
                    },
                    state: "minecraft:vine[south=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 30,
                        y: 69,
                        z: 29
                    },
                    state: "minecraft:vine[south=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 30,
                        y: 68,
                        z: 29
                    },
                    state: "minecraft:vine[south=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 30,
                        y: 67,
                        z: 29
                    },
                    state: "minecraft:vine[south=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 30,
                        y: 66,
                        z: 29
                    },
                    state: "minecraft:vine[south=true]",
                },
            ]
        );
        assert_eq!(
            super::cocoa_decorator_placement(
                &[
                    super::CocoaLogContext {
                        pos: BlockPos {
                            x: 20,
                            y: 67,
                            z: 20
                        },
                        north_air: true,
                        east_air: true,
                        south_air: true,
                        west_air: true,
                    },
                    super::CocoaLogContext {
                        pos: BlockPos {
                            x: 20,
                            y: 64,
                            z: 20
                        },
                        north_air: true,
                        east_air: true,
                        south_air: false,
                        west_air: true,
                    },
                ],
                0.5,
                0.49,
                &[0.1, 0.2, 0.3, 0.25, 0.1, 0.1, 0.1, 0.1],
                &[1, 2, 4, 0, 1, 2],
            ),
            vec![
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 19,
                        y: 64,
                        z: 20
                    },
                    state: "minecraft:cocoa[age=1,facing=east]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 21,
                        y: 64,
                        z: 20
                    },
                    state: "minecraft:cocoa[age=2,facing=west]",
                },
            ]
        );
        assert!(super::cocoa_decorator_placement(
            &[super::CocoaLogContext {
                pos: BlockPos {
                    x: 20,
                    y: 64,
                    z: 20
                },
                north_air: true,
                east_air: true,
                south_air: true,
                west_air: true,
            }],
            0.5,
            0.5,
            &[0.0; 4],
            &[0; 4],
        )
        .is_empty());
        assert_eq!(
            super::beehive_decorator_placement(
                &[
                    BlockPos { x: 0, y: 64, z: 0 },
                    BlockPos { x: 0, y: 65, z: 0 },
                    BlockPos { x: 0, y: 66, z: 0 },
                ],
                &[BlockPos { x: 0, y: 66, z: 0 }],
                1.0,
                0.0,
                0,
                &[2, 0, 1],
                &[
                    (BlockPos { x: -1, y: 65, z: 0 }, false, true),
                    (BlockPos { x: 1, y: 65, z: 0 }, true, false),
                    (BlockPos { x: 0, y: 65, z: 1 }, true, true),
                ],
                1,
                &[598, 599, 600],
            ),
            Some(super::BeehiveDecoratorPlacement {
                pos: BlockPos { x: 0, y: 65, z: 1 },
                state: "minecraft:bee_nest[facing=south,honey_level=0]",
                bee_ticks_in_hive: vec![598, 0, 1],
            })
        );
        assert!(super::beehive_decorator_placement(
            &[BlockPos { x: 0, y: 64, z: 0 }],
            &[],
            0.5,
            0.5,
            0,
            &[],
            &[],
            0,
            &[],
        )
        .is_none());
        assert_eq!(
            super::creaking_heart_decorator_placement(
                &[
                    BlockPos { x: 5, y: 66, z: 5 },
                    BlockPos { x: 5, y: 64, z: 5 },
                    BlockPos { x: 5, y: 65, z: 5 },
                ],
                0.75,
                0.5,
                &[2, 0, 1],
                &[
                    (
                        BlockPos { x: 5, y: 66, z: 5 },
                        [true, true, false, true, true, true]
                    ),
                    (
                        BlockPos { x: 5, y: 64, z: 5 },
                        [true, true, true, true, true, true]
                    ),
                    (
                        BlockPos { x: 5, y: 65, z: 5 },
                        [true, true, true, true, false, true]
                    ),
                ],
            ),
            Some(super::TreeDecoratorPlacement {
                pos: BlockPos { x: 5, y: 64, z: 5 },
                state: "minecraft:creaking_heart[active=false,axis=y,natural=true]",
            })
        );
        assert!(super::creaking_heart_decorator_placement(
            &[BlockPos { x: 5, y: 64, z: 5 }],
            0.75,
            0.75,
            &[0],
            &[(BlockPos { x: 5, y: 64, z: 5 }, [true; 6])],
        )
        .is_none());
        assert_eq!(
            super::pale_moss_decorator_placement(
                &[
                    super::PaleMossAttachmentContext {
                        pos: BlockPos { x: 6, y: 66, z: 6 },
                        down_air: true,
                        below_air: vec![true, true, false],
                    },
                    super::PaleMossAttachmentContext {
                        pos: BlockPos { x: 6, y: 64, z: 6 },
                        down_air: true,
                        below_air: vec![true, true, true],
                    },
                    super::PaleMossAttachmentContext {
                        pos: BlockPos { x: 6, y: 65, z: 6 },
                        down_air: false,
                        below_air: vec![true],
                    },
                ],
                &[super::PaleMossAttachmentContext {
                    pos: BlockPos { x: 7, y: 68, z: 7 },
                    down_air: true,
                    below_air: vec![true, false],
                }],
                0.5,
                0.75,
                0.25,
                0.1,
                &[0.2, 0.9, 0.8],
                &[0.25],
                &[0.8, 0.3, 0.6],
            ),
            vec![
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 6, y: 65, z: 6 },
                    state: "minecraft:configured_feature/pale_moss_patch",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 6, y: 63, z: 6 },
                    state: "minecraft:pale_hanging_moss[tip=false]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 6, y: 62, z: 6 },
                    state: "minecraft:pale_hanging_moss[tip=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 7, y: 67, z: 7 },
                    state: "minecraft:pale_hanging_moss[tip=false]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 7, y: 66, z: 7 },
                    state: "minecraft:pale_hanging_moss[tip=true]",
                },
            ]
        );
        assert!(
            super::pale_moss_decorator_placement(&[], &[], 1.0, 1.0, 1.0, 0.0, &[], &[], &[],)
                .is_empty()
        );
        assert_eq!(
            super::validate_place_on_ground_decorator_fields(1, 0, 0),
            Ok(())
        );
        assert_eq!(
            super::validate_place_on_ground_decorator_fields(0, 0, 0).unwrap_err(),
            "place-on-ground tries must be positive".to_string()
        );
        assert_eq!(
            super::validate_place_on_ground_decorator_fields(1, -1, 0).unwrap_err(),
            "place-on-ground radius and height must be non-negative".to_string()
        );
        assert_eq!(
            super::place_on_ground_decorator_placement(
                &[
                    BlockPos { x: 0, y: 64, z: 0 },
                    BlockPos { x: 2, y: 64, z: 1 },
                    BlockPos {
                        x: 10,
                        y: 65,
                        z: 10
                    },
                ],
                5,
                2,
                1,
                "minecraft:pale_moss_carpet",
                &[
                    super::PlaceOnGroundAttemptContext {
                        pos: BlockPos {
                            x: -2,
                            y: 64,
                            z: -2
                        },
                        above_is_air_or_vine: true,
                        pos_is_solid_render: true,
                        motion_blocking_no_leaves_height: 65,
                    },
                    super::PlaceOnGroundAttemptContext {
                        pos: BlockPos { x: 4, y: 65, z: 3 },
                        above_is_air_or_vine: true,
                        pos_is_solid_render: true,
                        motion_blocking_no_leaves_height: 67,
                    },
                    super::PlaceOnGroundAttemptContext {
                        pos: BlockPos { x: 0, y: 63, z: 0 },
                        above_is_air_or_vine: false,
                        pos_is_solid_render: true,
                        motion_blocking_no_leaves_height: 64,
                    },
                    super::PlaceOnGroundAttemptContext {
                        pos: BlockPos { x: 0, y: 64, z: 0 },
                        above_is_air_or_vine: true,
                        pos_is_solid_render: false,
                        motion_blocking_no_leaves_height: 65,
                    },
                    super::PlaceOnGroundAttemptContext {
                        pos: BlockPos { x: 2, y: 64, z: 1 },
                        above_is_air_or_vine: true,
                        pos_is_solid_render: true,
                        motion_blocking_no_leaves_height: 64,
                    },
                    super::PlaceOnGroundAttemptContext {
                        pos: BlockPos { x: 0, y: 64, z: 0 },
                        above_is_air_or_vine: true,
                        pos_is_solid_render: true,
                        motion_blocking_no_leaves_height: 65,
                    },
                ],
            )
            .unwrap(),
            vec![
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: -2,
                        y: 65,
                        z: -2
                    },
                    state: "minecraft:pale_moss_carpet",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos { x: 2, y: 65, z: 1 },
                    state: "minecraft:pale_moss_carpet",
                },
            ]
        );
        assert!(super::place_on_ground_decorator_placement(
            &[],
            1,
            0,
            0,
            "minecraft:pale_moss_carpet",
            &[],
        )
        .unwrap()
        .is_empty());
        assert!(!super::tree_decorator_solid_render("minecraft:leaf_litter"));
        assert!(!super::tree_decorator_solid_render("minecraft:oak_leaves"));
        assert!(super::tree_decorator_solid_render("minecraft:grass_block"));
        let alter_ground = super::alter_ground_decorator_placement(
            &[
                BlockPos { x: 0, y: 64, z: 0 },
                BlockPos { x: 2, y: 65, z: 0 },
            ],
            &[9, 18, 27, 36, 63],
            &[
                super::AlterGroundScanContext {
                    pos: BlockPos {
                        x: -1,
                        y: 66,
                        z: -1,
                    },
                    provider_state: Some("minecraft:rooted_dirt"),
                    is_air: true,
                },
                super::AlterGroundScanContext {
                    pos: BlockPos {
                        x: -3,
                        y: 66,
                        z: -3,
                    },
                    provider_state: Some("minecraft:moss_block"),
                    is_air: true,
                },
                super::AlterGroundScanContext {
                    pos: BlockPos { x: 4, y: 66, z: 4 },
                    provider_state: Some("minecraft:podzol"),
                    is_air: true,
                },
                super::AlterGroundScanContext {
                    pos: BlockPos { x: 1, y: 63, z: -1 },
                    provider_state: None,
                    is_air: false,
                },
                super::AlterGroundScanContext {
                    pos: BlockPos { x: 1, y: 62, z: -1 },
                    provider_state: Some("minecraft:coarse_dirt"),
                    is_air: true,
                },
                super::AlterGroundScanContext {
                    pos: BlockPos { x: 2, y: 61, z: 2 },
                    provider_state: Some("minecraft:coarse_dirt"),
                    is_air: true,
                },
            ],
        );
        assert!(alter_ground.contains(&super::TreeDecoratorPlacement {
            pos: BlockPos {
                x: -1,
                y: 66,
                z: -1
            },
            state: "minecraft:rooted_dirt",
        }));
        assert!(alter_ground.contains(&super::TreeDecoratorPlacement {
            pos: BlockPos { x: 4, y: 66, z: 4 },
            state: "minecraft:podzol",
        }));
        assert!(alter_ground.contains(&super::TreeDecoratorPlacement {
            pos: BlockPos { x: 2, y: 61, z: 2 },
            state: "minecraft:coarse_dirt",
        }));
        assert!(!alter_ground.contains(&super::TreeDecoratorPlacement {
            pos: BlockPos {
                x: -3,
                y: 66,
                z: -3
            },
            state: "minecraft:moss_block",
        }));
        assert!(!alter_ground.contains(&super::TreeDecoratorPlacement {
            pos: BlockPos { x: 1, y: 62, z: -1 },
            state: "minecraft:coarse_dirt",
        }));
        assert!(super::alter_ground_decorator_placement(&[], &[], &[]).is_empty());
        assert_eq!(
            super::attached_to_logs_decorator_placement(
                &[
                    BlockPos {
                        x: 40,
                        y: 66,
                        z: 40
                    },
                    BlockPos {
                        x: 40,
                        y: 64,
                        z: 40
                    },
                    BlockPos {
                        x: 40,
                        y: 65,
                        z: 40
                    },
                ],
                0.5,
                "minecraft:glow_lichen",
                &[2, 0, 1],
                &["north", "east", "south"],
                &[0.5, 0.49, 0.1],
                &[
                    (
                        BlockPos {
                            x: 40,
                            y: 66,
                            z: 39
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 41,
                            y: 64,
                            z: 40
                        },
                        false,
                    ),
                    (
                        BlockPos {
                            x: 40,
                            y: 65,
                            z: 41
                        },
                        true,
                    ),
                ],
            ),
            vec![
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 40,
                        y: 66,
                        z: 39
                    },
                    state: "minecraft:glow_lichen",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 40,
                        y: 65,
                        z: 41
                    },
                    state: "minecraft:glow_lichen",
                },
            ]
        );
        assert_eq!(
            super::validate_attached_to_leaves_decorator_fields(16, 0, 1, 1),
            Ok(())
        );
        assert_eq!(
            super::validate_attached_to_leaves_decorator_fields(17, 0, 1, 1).unwrap_err(),
            "attached-to-leaves exclusion radii must be in 0..=16".to_string()
        );
        assert_eq!(
            super::validate_attached_to_leaves_decorator_fields(0, 0, 0, 1).unwrap_err(),
            "attached-to-leaves required_empty_blocks must be in 1..=16".to_string()
        );
        assert_eq!(
            super::validate_attached_to_leaves_decorator_fields(0, 0, 1, 0).unwrap_err(),
            "attached-to-leaves directions list must be non-empty".to_string()
        );
        assert_eq!(
            super::attached_to_leaves_decorator_placement(
                &[
                    BlockPos {
                        x: 50,
                        y: 64,
                        z: 50
                    },
                    BlockPos {
                        x: 51,
                        y: 64,
                        z: 50
                    },
                    BlockPos {
                        x: 50,
                        y: 65,
                        z: 50
                    },
                ],
                0.5,
                1,
                0,
                2,
                "minecraft:mangrove_propagule[hanging=true]",
                &[0, 1, 2],
                &["down", "down", "east"],
                &[0.1, 0.1, 0.49],
                &[
                    (
                        BlockPos {
                            x: 50,
                            y: 63,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 50,
                            y: 62,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 51,
                            y: 63,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 51,
                            y: 62,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 51,
                            y: 65,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 52,
                            y: 65,
                            z: 50
                        },
                        true,
                    ),
                ],
            )
            .unwrap(),
            vec![
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 50,
                        y: 63,
                        z: 50
                    },
                    state: "minecraft:mangrove_propagule[hanging=true]",
                },
                super::TreeDecoratorPlacement {
                    pos: BlockPos {
                        x: 51,
                        y: 65,
                        z: 50
                    },
                    state: "minecraft:mangrove_propagule[hanging=true]",
                },
            ]
        );
        assert!(super::attached_to_leaves_decorator_placement(
            &[BlockPos { x: 0, y: 64, z: 0 }],
            0.5,
            0,
            0,
            1,
            "minecraft:mangrove_propagule[hanging=true]",
            &[0],
            &["down"],
            &[0.5],
            &[(BlockPos { x: 0, y: 63, z: 0 }, true)],
        )
        .unwrap()
        .is_empty());
    }

    #[test]
    fn feature_placement_behavior_models_cover_required_families() {
        assert_eq!(
            FEATURE_BEHAVIOR_MODELS
                .iter()
                .map(|model| model.feature_type)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:tree",
                "minecraft:vegetation_patch",
                "minecraft:spring_feature",
                "minecraft:ore",
                "minecraft:scattered_ore",
                "minecraft:disk",
                "minecraft:lake",
                "minecraft:geode",
                "minecraft:fossil",
                "minecraft:monster_room",
            ]
        );

        assert!(super::spring_feature_can_place(
            true, true, true, true, 4, 1, 4, 1
        ));
        assert!(!super::spring_feature_can_place(
            true, true, false, true, 4, 1, 4, 1
        ));
        assert!(!super::spring_feature_can_place(
            true, false, false, true, 3, 1, 4, 1
        ));
        let spring_config = super::SpringConfigurationModel {
            state: "minecraft:water",
            requires_block_below: true,
            rock_count: 4,
            hole_count: 1,
            valid_blocks: &["minecraft:stone", "minecraft:dirt"],
        };
        assert_eq!(
            super::spring_placement_plan(
                &spring_config,
                super::SpringPlacementContext {
                    origin: BlockPos { x: 4, y: 32, z: 4 },
                    above_block: "minecraft:stone",
                    below_block: "minecraft:stone",
                    current_block: "minecraft:air",
                    west_block: "minecraft:stone",
                    east_block: "minecraft:stone",
                    north_block: "minecraft:air",
                    south_block: "minecraft:dirt",
                },
            ),
            Some(super::SpringPlacementPlan {
                pos: BlockPos { x: 4, y: 32, z: 4 },
                state: "minecraft:water",
                schedule_tick: true,
            })
        );
        assert_eq!(
            super::spring_placement_plan(
                &spring_config,
                super::SpringPlacementContext {
                    origin: BlockPos { x: 4, y: 32, z: 4 },
                    above_block: "minecraft:stone",
                    below_block: "minecraft:air",
                    current_block: "minecraft:air",
                    west_block: "minecraft:stone",
                    east_block: "minecraft:stone",
                    north_block: "minecraft:air",
                    south_block: "minecraft:dirt",
                },
            ),
            None
        );
        assert_eq!(MONSTER_ROOM_BOUNDS.min_y, -1);
        assert_eq!(MONSTER_ROOM_BOUNDS.max_y, 4);
        assert!(!super::monster_room_opening_count_is_valid(0));
        assert!(super::monster_room_opening_count_is_valid(1));
        assert!(super::monster_room_opening_count_is_valid(5));
        assert!(!super::monster_room_opening_count_is_valid(6));
        let room_radii = super::monster_room_radii(0, 1);
        assert_eq!(
            room_radii,
            super::MonsterRoomRadii {
                x_radius: 2,
                z_radius: 3,
            }
        );
        assert_eq!(
            super::monster_room_bounds_for_radius(room_radii.x_radius),
            (-3, 3)
        );
        let probes = [
            super::MonsterRoomProbe {
                dx: 0,
                dy: -1,
                dz: 0,
                solid: true,
                empty: false,
                above_empty: false,
            },
            super::MonsterRoomProbe {
                dx: 0,
                dy: 4,
                dz: 0,
                solid: true,
                empty: false,
                above_empty: false,
            },
            super::MonsterRoomProbe {
                dx: -3,
                dy: 0,
                dz: 0,
                solid: false,
                empty: true,
                above_empty: true,
            },
        ];
        assert_eq!(
            super::monster_room_opening_count(room_radii, &probes),
            Some(1)
        );
        assert!(super::monster_room_can_place(room_radii, &probes));
        let invalid_floor = [super::MonsterRoomProbe {
            dx: 0,
            dy: -1,
            dz: 0,
            solid: false,
            empty: true,
            above_empty: true,
        }];
        assert_eq!(
            super::monster_room_opening_count(room_radii, &invalid_floor),
            None
        );
        assert_eq!(
            super::monster_room_shell_state(-3, -1, 0, room_radii, 31, true, true, false, 1),
            Some("minecraft:mossy_cobblestone")
        );
        assert_eq!(
            super::monster_room_shell_state(-3, 0, 0, room_radii, 32, false, true, false, 0),
            Some("minecraft:cave_air")
        );
        assert_eq!(
            super::monster_room_shell_state(0, 0, 0, room_radii, 32, true, true, false, 0),
            Some("minecraft:cave_air")
        );
        assert!(super::monster_room_chest_can_place(true, 1));
        assert!(!super::monster_room_chest_can_place(true, 2));
        assert_eq!(super::monster_room_spawner_mob(0), "minecraft:skeleton");
        assert_eq!(super::monster_room_spawner_mob(1), "minecraft:zombie");
        assert_eq!(super::monster_room_spawner_mob(2), "minecraft:zombie");
        assert_eq!(super::monster_room_spawner_mob(3), "minecraft:spider");
        assert!(super::ore_vein_sphere_is_shadowed(3.0, 1.0, 1.0, 1.0));
        assert!(!super::ore_vein_sphere_is_shadowed(1.0, 2.0, 0.0, 0.0));
    }

    #[test]
    fn structure_registries_and_sets_match_vanilla_bootstrap() {
        assert_eq!(STRUCTURE_TYPES.len(), 16);
        assert_eq!(BUILTIN_STRUCTURES.len(), 34);
        assert_eq!(BUILTIN_STRUCTURE_SETS.len(), 20);
        assert_eq!(
            BUILTIN_STRUCTURE_SETS
                .iter()
                .map(|set| set.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:villages",
                "minecraft:desert_pyramids",
                "minecraft:igloos",
                "minecraft:jungle_temples",
                "minecraft:swamp_huts",
                "minecraft:pillager_outposts",
                "minecraft:ancient_cities",
                "minecraft:ocean_monuments",
                "minecraft:woodland_mansions",
                "minecraft:buried_treasures",
                "minecraft:mineshafts",
                "minecraft:ruined_portals",
                "minecraft:shipwrecks",
                "minecraft:ocean_ruins",
                "minecraft:nether_complexes",
                "minecraft:nether_fossils",
                "minecraft:end_cities",
                "minecraft:strongholds",
                "minecraft:trail_ruins",
                "minecraft:trial_chambers",
            ]
        );

        let villages = &BUILTIN_STRUCTURE_SETS[0];
        assert_eq!(villages.structures.len(), 5);
        assert_eq!(
            villages.placement,
            StructurePlacementKind::RandomSpread {
                spacing: 34,
                separation: 8,
                salt: 10387312,
                spread_type: RandomSpreadType::Linear,
            }
        );

        let strongholds = BUILTIN_STRUCTURE_SETS
            .iter()
            .find(|set| set.id == "minecraft:strongholds")
            .unwrap();
        assert_eq!(
            strongholds.placement,
            StructurePlacementKind::ConcentricRings {
                distance: 32,
                spread: 3,
                count: 128,
            }
        );

        let mansions = BUILTIN_STRUCTURE_SETS
            .iter()
            .find(|set| set.id == "minecraft:woodland_mansions")
            .unwrap();
        assert_eq!(
            mansions.placement,
            StructurePlacementKind::RandomSpread {
                spacing: 80,
                separation: 20,
                salt: 10387319,
                spread_type: RandomSpreadType::Triangular,
            }
        );
    }

    #[test]
    fn random_spread_structure_placement_uses_vanilla_grid_and_salt_math() {
        assert_eq!(
            super::validate_random_spread_placement(32, 32).unwrap_err(),
            "Spacing has to be larger than separation".to_string()
        );
        assert_eq!(
            super::validate_random_spread_placement(4097, 0).unwrap_err(),
            "Random spread spacing and separation must be in 0..=4096".to_string()
        );

        let village = super::random_spread_potential_structure_chunk(
            12345,
            0,
            0,
            34,
            8,
            10387312,
            RandomSpreadType::Linear,
        )
        .unwrap();
        assert_eq!(village, ChunkPos { x: 21, z: 5 });
        assert!(!super::random_spread_is_placement_chunk(
            12345,
            0,
            0,
            34,
            8,
            10387312,
            RandomSpreadType::Linear,
        )
        .unwrap());
        assert!(super::random_spread_is_placement_chunk(
            12345,
            village.x,
            village.z,
            34,
            8,
            10387312,
            RandomSpreadType::Linear,
        )
        .unwrap());

        assert_eq!(
            super::random_spread_potential_structure_chunk(
                12345,
                -1,
                -1,
                80,
                20,
                10387319,
                RandomSpreadType::Triangular,
            )
            .unwrap(),
            ChunkPos { x: -37, z: -54 }
        );
    }

    #[test]
    fn structure_frequency_reducers_match_vanilla_methods() {
        assert_eq!(super::FrequencyReductionMethod::Default.id(), "default");
        assert_eq!(
            super::FrequencyReductionMethod::LegacyType1.id(),
            "legacy_type_1"
        );
        assert_eq!(
            super::FrequencyReductionMethod::LegacyType2.id(),
            "legacy_type_2"
        );
        assert_eq!(
            super::FrequencyReductionMethod::LegacyType3.id(),
            "legacy_type_3"
        );
        assert_eq!(
            super::validate_structure_frequency(1.25).unwrap_err(),
            "Structure placement frequency must be in 0.0..=1.0".to_string()
        );

        assert!(super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::Default,
            12345,
            10387312,
            21,
            5,
            1.0,
        )
        .unwrap());
        assert!(!super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::Default,
            12345,
            10387312,
            21,
            5,
            0.0,
        )
        .unwrap());
        assert!(!super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::Default,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
        assert!(super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::LegacyType1,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
        assert!(super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::LegacyType2,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
        assert!(!super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::LegacyType3,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
    }

    #[test]
    fn structure_locate_pos_uses_chunk_min_block_and_validated_offset() {
        assert_eq!(
            super::structure_locate_pos(ChunkPos { x: 21, z: -5 }, BlockPos { x: 8, y: 0, z: 8 })
                .unwrap(),
            BlockPos {
                x: 344,
                y: 0,
                z: -72
            }
        );
        assert_eq!(
            super::structure_locate_pos(
                ChunkPos { x: -2, z: 3 },
                BlockPos {
                    x: -16,
                    y: 16,
                    z: 16
                }
            )
            .unwrap(),
            BlockPos {
                x: -48,
                y: 16,
                z: 64
            }
        );
        assert_eq!(
            super::structure_locate_pos(ChunkPos { x: 0, z: 0 }, BlockPos { x: 17, y: 0, z: 0 })
                .unwrap_err(),
            "Structure locate offset components must be in -16..=16".to_string()
        );
    }

    #[test]
    fn concentric_rings_initial_candidates_follow_vanilla_ring_progression() {
        assert_eq!(
            super::validate_concentric_rings_placement(1024, 3, 128).unwrap_err(),
            "Concentric rings distance and spread must be in 0..=1023".to_string()
        );
        assert_eq!(
            super::validate_concentric_rings_placement(32, 3, 0).unwrap_err(),
            "Concentric rings count must be in 1..=4095".to_string()
        );

        let candidates = super::concentric_ring_initial_candidates(12345, 32, 3, 8).unwrap();
        assert_eq!(candidates.len(), 8);
        assert_eq!(
            candidates[0],
            super::ConcentricRingPlacementCandidate {
                index: 0,
                circle: 0,
                chunk_pos: ChunkPos { x: -105, z: 124 },
            }
        );
        assert_eq!(
            candidates[2],
            super::ConcentricRingPlacementCandidate {
                index: 2,
                circle: 0,
                chunk_pos: ChunkPos { x: 114, z: 21 },
            }
        );
        assert_eq!(candidates[3].circle, 1);
        let ring_positions = candidates
            .iter()
            .map(|candidate| candidate.chunk_pos)
            .collect::<Vec<_>>();
        assert!(super::concentric_rings_is_placement_chunk(
            &ring_positions,
            -105,
            124
        ));
        assert!(!super::concentric_rings_is_placement_chunk(
            &ring_positions,
            0,
            0
        ));

        assert_eq!(
            super::concentric_ring_candidate_search_center(candidates[0]),
            BlockPos {
                x: -1672,
                y: 0,
                z: 1992,
            }
        );
        assert_eq!(
            super::concentric_ring_adjusted_position(
                candidates[0],
                Some(super::ConcentricRingBiomeSearchResult {
                    block_x: -1601,
                    block_z: 2047,
                }),
            ),
            ChunkPos { x: -101, z: 127 }
        );
        assert_eq!(
            super::concentric_ring_adjusted_position(candidates[1], None),
            candidates[1].chunk_pos
        );
        assert_eq!(
            super::concentric_ring_adjusted_positions(
                &candidates[0..3],
                &[
                    Some(super::ConcentricRingBiomeSearchResult {
                        block_x: -1601,
                        block_z: 2047,
                    }),
                    None,
                    Some(super::ConcentricRingBiomeSearchResult {
                        block_x: 0,
                        block_z: -1,
                    }),
                ],
            ),
            vec![
                ChunkPos { x: -101, z: 127 },
                candidates[1].chunk_pos,
                ChunkPos { x: 0, z: -1 },
            ]
        );
    }

    #[test]
    fn structure_exclusion_zone_checks_square_chunk_range() {
        let zone = super::StructureExclusionZoneModel {
            other_set: "minecraft:villages",
            chunk_count: 3,
        };
        assert_eq!(super::validate_structure_exclusion_zone(zone), Ok(zone));
        assert_eq!(
            super::validate_structure_exclusion_zone(super::StructureExclusionZoneModel {
                other_set: "minecraft:villages",
                chunk_count: 17,
            })
            .unwrap_err(),
            "Structure exclusion zone chunk_count must be in 1..=16".to_string()
        );

        let other_chunks = [
            ChunkPos { x: 10, z: -4 },
            ChunkPos { x: -12, z: 8 },
            ChunkPos { x: 40, z: 40 },
        ];
        assert!(super::structure_has_chunk_in_range(&other_chunks, 7, -1, 3));
        assert!(!super::structure_has_chunk_in_range(
            &other_chunks,
            6,
            -1,
            3
        ));
        assert!(super::structure_exclusion_zone_forbids(zone, &other_chunks, 7, -1).unwrap());
        assert!(!super::structure_exclusion_zone_forbids(zone, &other_chunks, 0, 0).unwrap());
    }

    #[test]
    fn chunk_generator_structure_state_caches_placeable_sets_and_ring_positions() {
        let placeable = ["minecraft:stronghold", "minecraft:village_plains"];
        let mut normal = super::ChunkGeneratorStructureStateModel::create_for_normal(
            12345,
            super::BUILTIN_STRUCTURE_SETS,
            &placeable,
        );
        assert_eq!(normal.level_seed, 12345);
        assert_eq!(normal.concentric_rings_seed, 12345);
        assert!(normal
            .possible_structure_sets
            .iter()
            .any(|set| set.id == "minecraft:villages"));
        assert!(normal
            .possible_structure_sets
            .iter()
            .any(|set| set.id == "minecraft:strongholds"));
        assert!(!normal
            .possible_structure_sets
            .iter()
            .any(|set| set.id == "minecraft:desert_pyramids"));

        assert_eq!(
            normal.get_placements_for_structure("minecraft:village_plains", &placeable),
            vec![super::StructurePlacementKind::RandomSpread {
                spacing: 34,
                separation: 8,
                salt: 10387312,
                spread_type: super::RandomSpreadType::Linear,
            }]
        );
        assert!(normal
            .get_placements_for_structure("minecraft:desert_pyramid", &placeable)
            .is_empty());
        let rings = normal
            .get_ring_positions_for("minecraft:strongholds", &placeable)
            .unwrap();
        assert_eq!(rings.len(), 128);
        assert_eq!(rings[0], ChunkPos { x: -105, z: 124 });
        assert!(normal
            .has_structure_chunk_in_range("minecraft:strongholds", -105, 124, 0, &placeable)
            .unwrap());

        let village_chunk = super::random_spread_potential_structure_chunk(
            normal.level_seed,
            0,
            0,
            34,
            8,
            10387312,
            super::RandomSpreadType::Linear,
        )
        .unwrap();
        assert!(normal
            .has_structure_chunk_in_range(
                "minecraft:villages",
                village_chunk.x - 1,
                village_chunk.z,
                1,
                &placeable,
            )
            .unwrap());
        assert!(!normal
            .has_structure_chunk_in_range("minecraft:desert_pyramids", 0, 0, 10, &placeable)
            .unwrap());

        let mut flat = super::ChunkGeneratorStructureStateModel::create_for_flat(
            12345,
            super::BUILTIN_STRUCTURE_SETS,
            &["minecraft:stronghold"],
        );
        assert_eq!(flat.concentric_rings_seed, 0);
        assert_ne!(
            flat.get_ring_positions_for("minecraft:strongholds", &["minecraft:stronghold"])
                .unwrap()[0],
            rings[0]
        );
    }

    #[test]
    fn structure_start_validity_references_tags_and_piece_queries_match_vanilla() {
        let invalid = super::StructureStartModel::invalid();
        assert!(!invalid.is_valid());
        assert_eq!(
            invalid.create_tag(ChunkPos { x: 4, z: -7 }),
            super::StructureStartTagModel {
                id: "INVALID",
                chunk_x: None,
                chunk_z: None,
                references: None,
                children: 0,
            }
        );

        let first_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 20,
                min_z: -16,
                max_x: 47,
                max_y: 35,
                max_z: -1,
            },
        };
        let second_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 48,
                min_y: 18,
                min_z: -8,
                max_x: 63,
                max_y: 30,
                max_z: 7,
            },
        };
        let mut start = super::StructureStartModel {
            structure: Some("minecraft:village_plains"),
            chunk_pos: ChunkPos { x: 2, z: -1 },
            references: 0,
            pieces: vec![first_piece, second_piece],
        };

        assert!(start.is_valid());
        assert!(start.can_be_referenced());
        start.add_reference();
        assert_eq!(start.references, 1);
        assert!(!start.can_be_referenced());
        assert_eq!(
            start.bounding_box(),
            Some(super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 18,
                min_z: -16,
                max_x: 63,
                max_y: 35,
                max_z: 7,
            })
        );
        assert_eq!(
            start.create_tag(ChunkPos { x: 2, z: -1 }),
            super::StructureStartTagModel {
                id: "minecraft:village_plains",
                chunk_x: Some(2),
                chunk_z: Some(-1),
                references: Some(1),
                children: 2,
            }
        );
        assert_eq!(
            super::structure_start_reference_pos(first_piece),
            BlockPos {
                x: 40,
                y: 20,
                z: -8,
            }
        );
        assert_eq!(
            super::structure_pieces_intersecting_chunk(
                &start,
                super::StructureBoundingBoxModel {
                    min_x: 48,
                    min_y: -64,
                    min_z: -16,
                    max_x: 63,
                    max_y: 320,
                    max_z: -1,
                }
            ),
            vec![second_piece]
        );
    }

    #[test]
    fn structure_manager_position_queries_use_union_and_piece_boxes() {
        let first_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 10,
                min_z: 0,
                max_x: 4,
                max_y: 20,
                max_z: 4,
            },
        };
        let second_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 10,
                min_z: 10,
                max_x: 14,
                max_y: 20,
                max_z: 14,
            },
        };
        let start = super::StructureStartModel {
            structure: Some("minecraft:stronghold"),
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: vec![first_piece, second_piece],
        };
        let inside_first_edge = BlockPos { x: 4, y: 20, z: 4 };
        let inside_union_gap = BlockPos { x: 7, y: 15, z: 7 };
        let outside = BlockPos {
            x: 15,
            y: 15,
            z: 15,
        };

        assert!(first_piece.bounding_box.is_inside(inside_first_edge));
        assert!(super::structure_start_contains_pos(
            inside_first_edge,
            &start
        ));
        assert!(super::structure_has_piece_at(inside_first_edge, &start));

        assert!(super::structure_start_contains_pos(
            inside_union_gap,
            &start
        ));
        assert!(!super::structure_has_piece_at(inside_union_gap, &start));
        assert!(!super::structure_start_contains_pos(outside, &start));
        assert!(!super::structure_has_piece_at(outside, &start));

        let invalid = super::StructureStartModel::invalid();
        assert_eq!(
            super::first_structure_start_containing_pos(
                inside_union_gap,
                &[invalid.clone(), start.clone()]
            ),
            Some(start.clone())
        );
        assert_eq!(
            super::first_structure_start_with_piece_at(inside_union_gap, &[start.clone()]),
            None
        );
        assert_eq!(
            super::first_structure_start_with_piece_at(
                inside_first_edge,
                &[invalid, start.clone()]
            ),
            Some(start)
        );
    }

    #[test]
    fn chunk_generator_mob_lookup_prefers_matching_structure_spawn_overrides() {
        let first_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 10,
                min_z: 0,
                max_x: 4,
                max_y: 20,
                max_z: 4,
            },
        };
        let second_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 10,
                min_z: 10,
                max_x: 14,
                max_y: 20,
                max_z: 14,
            },
        };
        let start = super::StructureStartModel {
            structure: Some("minecraft:swamp_hut"),
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: vec![first_piece, second_piece],
        };
        let piece_override = super::StructureSpawnOverrideModel {
            category: "monster",
            bounding_box: super::StructureSpawnBoundingBoxTypeModel::Piece,
            spawns: &["minecraft:witch"],
        };
        let full_override = super::StructureSpawnOverrideModel {
            category: "monster",
            bounding_box: super::StructureSpawnBoundingBoxTypeModel::Full,
            spawns: &["minecraft:guardian"],
        };
        let inside_piece = BlockPos { x: 4, y: 20, z: 4 };
        let inside_union_gap = BlockPos { x: 7, y: 15, z: 7 };
        let biome_spawns = &["minecraft:zombie", "minecraft:skeleton"];

        assert!(super::structure_spawn_override_applies(
            inside_piece,
            &start,
            piece_override
        ));
        assert!(!super::structure_spawn_override_applies(
            inside_union_gap,
            &start,
            piece_override
        ));
        assert!(super::structure_spawn_override_applies(
            inside_union_gap,
            &start,
            full_override
        ));
        assert_eq!(
            super::chunk_generator_mobs_at(
                biome_spawns,
                "monster",
                inside_piece,
                &[super::StructureSpawnCandidateModel {
                    structure: "minecraft:swamp_hut",
                    start: &start,
                    override_model: Some(piece_override),
                }]
            ),
            vec!["minecraft:witch"]
        );
        assert_eq!(
            super::chunk_generator_mobs_at(
                biome_spawns,
                "monster",
                inside_union_gap,
                &[super::StructureSpawnCandidateModel {
                    structure: "minecraft:swamp_hut",
                    start: &start,
                    override_model: Some(piece_override),
                }]
            ),
            biome_spawns.to_vec()
        );
        assert_eq!(
            super::chunk_generator_mobs_at(
                biome_spawns,
                "monster",
                inside_union_gap,
                &[super::StructureSpawnCandidateModel {
                    structure: "minecraft:ocean_monument",
                    start: &start,
                    override_model: Some(full_override),
                }]
            ),
            vec!["minecraft:guardian"]
        );
        assert_eq!(
            super::chunk_generator_mobs_at(
                biome_spawns,
                "creature",
                inside_piece,
                &[super::StructureSpawnCandidateModel {
                    structure: "minecraft:swamp_hut",
                    start: &start,
                    override_model: Some(piece_override),
                }]
            ),
            biome_spawns.to_vec()
        );
    }

    #[test]
    fn structure_piece_bounding_box_and_world_coordinates_match_vanilla_orientation() {
        assert_eq!(
            super::structure_make_bounding_box(
                10,
                20,
                30,
                super::HorizontalDirection::South,
                3,
                4,
                5
            ),
            super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 20,
                min_z: 30,
                max_x: 12,
                max_y: 23,
                max_z: 34,
            }
        );
        assert_eq!(
            super::structure_make_bounding_box(
                10,
                20,
                30,
                super::HorizontalDirection::East,
                3,
                4,
                5
            ),
            super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 20,
                min_z: 30,
                max_x: 14,
                max_y: 23,
                max_z: 32,
            }
        );

        let foot = BlockPos {
            x: 100,
            y: 40,
            z: 200,
        };
        let offset = BlockPos { x: 2, y: 3, z: 4 };
        assert_eq!(
            super::structure_orient_box(foot, offset, 5, 6, 7, super::HorizontalDirection::North),
            super::StructureBoundingBoxModel {
                min_x: 102,
                min_y: 43,
                min_z: 198,
                max_x: 106,
                max_y: 48,
                max_z: 204,
            }
        );
        assert_eq!(
            super::structure_orient_box(foot, offset, 5, 6, 7, super::HorizontalDirection::West),
            super::StructureBoundingBoxModel {
                min_x: 98,
                min_y: 43,
                min_z: 202,
                max_x: 104,
                max_y: 48,
                max_z: 206,
            }
        );
        assert_eq!(
            super::structure_orient_box(foot, offset, 5, 6, 7, super::HorizontalDirection::East),
            super::StructureBoundingBoxModel {
                min_x: 104,
                min_y: 43,
                min_z: 202,
                max_x: 110,
                max_y: 48,
                max_z: 206,
            }
        );

        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 50,
            min_y: 60,
            min_z: 70,
            max_x: 59,
            max_y: 69,
            max_z: 79,
        };
        assert_eq!(
            super::structure_piece_world_pos(bounding_box, None, 1, 2, 3),
            BlockPos { x: 1, y: 2, z: 3 }
        );
        assert_eq!(
            super::structure_piece_world_pos(
                bounding_box,
                Some(super::HorizontalDirection::North),
                1,
                2,
                3
            ),
            BlockPos {
                x: 51,
                y: 62,
                z: 76,
            }
        );
        assert_eq!(
            super::structure_piece_world_pos(
                bounding_box,
                Some(super::HorizontalDirection::West),
                1,
                2,
                3
            ),
            BlockPos {
                x: 56,
                y: 62,
                z: 71,
            }
        );
        assert_eq!(
            super::structure_piece_world_pos(
                bounding_box,
                Some(super::HorizontalDirection::East),
                1,
                2,
                3
            ),
            BlockPos {
                x: 53,
                y: 62,
                z: 71,
            }
        );
    }

    #[test]
    fn mineshaft_corridor_ceiling_uses_java_maybe_box_probability() {
        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 15,
            max_y: 15,
            max_z: 15,
        };
        let piece_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 2,
            max_y: 2,
            max_z: 4,
        };
        let rolls = [
            0.0, 0.79, 0.8, 0.81, 0.2, 0.95, 0.1, 0.7, 0.99, 0.3, 0.4, 0.5, 0.6, 0.85, 0.75,
        ];

        let blocks = super::structure_piece_generate_maybe_box(
            piece_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            &rolls,
            0.8,
            BlockPos { x: 0, y: 2, z: 0 },
            BlockPos { x: 2, y: 2, z: 4 },
            "minecraft:cave_air",
            "minecraft:cave_air",
            false,
            false,
            |_| false,
            |_| true,
        );

        assert_eq!(
            blocks.len(),
            rolls.iter().filter(|roll| **roll <= 0.8).count(),
            "Java StructurePiece#generateMaybeBox places when random.nextFloat() <= probability"
        );
        assert!(!blocks
            .iter()
            .any(|block| block.world_pos == BlockPos { x: 0, y: 2, z: 3 }));
    }

    #[test]
    fn structure_piece_chunk_proximity_and_locator_position_match_vanilla() {
        let even_sized_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 20,
                min_z: -16,
                max_x: 47,
                max_y: 35,
                max_z: -1,
            },
        };
        assert_eq!(
            super::structure_piece_locator_position(even_sized_piece),
            BlockPos {
                x: 40,
                y: 28,
                z: -8,
            }
        );

        let piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 20,
                min_y: -10,
                min_z: 20,
                max_x: 25,
                max_y: 120,
                max_z: 25,
            },
        };
        assert!(super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 1, z: 1 },
            0
        ));
        assert!(!super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 2, z: 1 },
            0
        ));
        assert!(super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 2, z: 1 },
            7
        ));
    }

    #[test]
    fn structure_piece_box_generation_uses_vanilla_loop_edges_and_chunk_clipping() {
        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 20,
            max_y: 30,
            max_z: 40,
        };
        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: 20,
            min_z: 30,
            max_x: 12,
            max_y: 22,
            max_z: 32,
        };
        let blocks = super::structure_piece_generate_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 2, y: 2, z: 2 },
            "minecraft:cobblestone",
            "minecraft:mossy_cobblestone",
            false,
            |_| false,
        );

        assert_eq!(blocks.len(), 18);
        assert_eq!(
            blocks.first().copied(),
            Some(super::StructurePiecePlacementBlock {
                local_pos: BlockPos { x: 1, y: 0, z: 0 },
                world_pos: BlockPos {
                    x: 11,
                    y: 20,
                    z: 30,
                },
                state: "minecraft:cobblestone",
                edge: true,
            })
        );
        assert!(blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 1, y: 1, z: 1 }
                && block.world_pos
                    == BlockPos {
                        x: 11,
                        y: 21,
                        z: 31,
                    }
                && block.state == "minecraft:mossy_cobblestone"
                && !block.edge
        }));
        assert!(blocks
            .iter()
            .all(|block| chunk_bb.is_inside(block.world_pos)));
    }

    #[test]
    fn structure_piece_air_box_and_skip_air_match_vanilla_generation_rules() {
        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 10,
            max_y: 74,
            max_z: 10,
        };
        let chunk_bb = bounding_box;
        let air_blocks = super::structure_piece_generate_air_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 2, y: 3, z: 4 },
        );
        assert_eq!(air_blocks.len(), 8);
        assert!(air_blocks
            .iter()
            .all(|block| block.state == "minecraft:air" && block.edge));

        let skipped_world_pos = BlockPos { x: 1, y: 66, z: 3 };
        let blocks = super::structure_piece_generate_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 2, y: 3, z: 4 },
            "minecraft:stone_bricks",
            "minecraft:cracked_stone_bricks",
            true,
            |world_pos| world_pos == skipped_world_pos,
        );
        assert_eq!(blocks.len(), 7);
        assert!(!blocks
            .iter()
            .any(|block| block.world_pos == skipped_world_pos));
    }

    #[test]
    fn structure_piece_maybe_box_and_single_block_use_vanilla_probability_edges() {
        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 10,
            min_z: 0,
            max_x: 10,
            max_y: 20,
            max_z: 10,
        };
        let chunk_bb = bounding_box;
        let blocks = super::structure_piece_generate_maybe_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            &[0.75, 0.76, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60],
            0.75,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 1, y: 1, z: 1 },
            "minecraft:stone_bricks",
            "minecraft:cracked_stone_bricks",
            true,
            true,
            |world_pos| world_pos == BlockPos { x: 0, y: 10, z: 0 },
            |world_pos| world_pos.y >= 11,
        );

        assert_eq!(blocks.len(), 4);
        assert!(blocks
            .iter()
            .all(|block| block.state == "minecraft:stone_bricks" && block.edge));
        assert!(!blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 0, y: 0, z: 0 }
                || block.local_pos == BlockPos { x: 0, y: 0, z: 1 }
                || block.local_pos == BlockPos { x: 1, y: 0, z: 0 }
        }));

        assert_eq!(
            super::structure_piece_maybe_generate_block(
                bounding_box,
                Some(super::HorizontalDirection::South),
                chunk_bb,
                0.49,
                0.5,
                BlockPos { x: 2, y: 3, z: 4 },
                "minecraft:lantern"
            )
            .map(|block| block.world_pos),
            Some(BlockPos { x: 2, y: 13, z: 4 })
        );
        assert_eq!(
            super::structure_piece_maybe_generate_block(
                bounding_box,
                Some(super::HorizontalDirection::South),
                chunk_bb,
                0.5,
                0.5,
                BlockPos { x: 2, y: 3, z: 4 },
                "minecraft:lantern"
            ),
            None
        );
    }

    #[test]
    fn structure_piece_upper_half_sphere_matches_vanilla_shape_and_clipping() {
        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 20,
            max_y: 30,
            max_z: 40,
        };
        let full_chunk = super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 20,
            max_y: 30,
            max_z: 40,
        };
        let blocks = super::structure_piece_generate_upper_half_sphere(
            bounding_box,
            Some(super::HorizontalDirection::South),
            full_chunk,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 4, y: 4, z: 4 },
            "minecraft:smooth_sandstone",
            false,
            |_| false,
        );

        assert_eq!(blocks.len(), 76);
        assert!(blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 2, y: 0, z: 2 }
                && block.world_pos
                    == BlockPos {
                        x: 12,
                        y: 20,
                        z: 32,
                    }
        }));
        assert!(!blocks
            .iter()
            .any(|block| block.local_pos == BlockPos { x: 0, y: 0, z: 0 }));
        assert!(blocks
            .iter()
            .all(|block| block.state == "minecraft:smooth_sandstone" && !block.edge));

        let clipped_chunk = super::StructureBoundingBoxModel {
            min_x: 12,
            min_y: 20,
            min_z: 32,
            max_x: 14,
            max_y: 24,
            max_z: 34,
        };
        let skipped = BlockPos {
            x: 12,
            y: 20,
            z: 32,
        };
        let clipped = super::structure_piece_generate_upper_half_sphere(
            bounding_box,
            Some(super::HorizontalDirection::South),
            clipped_chunk,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 4, y: 4, z: 4 },
            "minecraft:smooth_sandstone",
            true,
            |world_pos| world_pos == skipped,
        );
        assert!(clipped.len() < blocks.len());
        assert!(clipped
            .iter()
            .all(|block| clipped_chunk.is_inside(block.world_pos)));
        assert!(!clipped.iter().any(|block| block.world_pos == skipped));
    }

    #[test]
    fn structure_piece_fill_column_down_matches_vanilla_replaceable_loop() {
        assert!(super::structure_piece_is_replaceable_by_structures(
            "minecraft:air"
        ));
        assert!(super::structure_piece_is_replaceable_by_structures(
            "minecraft:water"
        ));
        assert!(super::structure_piece_is_replaceable_by_structures(
            "minecraft:glow_lichen"
        ));
        assert!(super::structure_piece_is_replaceable_by_structures(
            "minecraft:tall_seagrass[half=upper]"
        ));
        assert!(!super::structure_piece_is_replaceable_by_structures(
            "minecraft:stone"
        ));

        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 15,
            max_y: 80,
            max_z: 15,
        };
        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: -64,
            min_z: 0,
            max_x: 15,
            max_y: 320,
            max_z: 15,
        };
        let blocks = super::structure_piece_fill_column_down(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            3,
            4,
            5,
            64,
            "minecraft:sandstone",
            |pos| {
                if pos.y >= 66 {
                    "minecraft:air"
                } else {
                    "minecraft:stone"
                }
            },
        );
        assert_eq!(
            blocks
                .iter()
                .map(|block| block.world_pos)
                .collect::<Vec<_>>(),
            vec![
                BlockPos { x: 3, y: 68, z: 5 },
                BlockPos { x: 3, y: 67, z: 5 },
                BlockPos { x: 3, y: 66, z: 5 },
            ]
        );

        let min_limited = super::structure_piece_fill_column_down(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            3,
            2,
            5,
            64,
            "minecraft:sandstone",
            |_| "minecraft:air",
        );
        assert_eq!(min_limited.len(), 1);
        assert_eq!(min_limited[0].world_pos.y, 66);

        let outside_chunk = super::structure_piece_fill_column_down(
            bounding_box,
            Some(super::HorizontalDirection::South),
            super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: -64,
                min_z: 10,
                max_x: 15,
                max_y: 320,
                max_z: 15,
            },
            3,
            4,
            5,
            64,
            "minecraft:sandstone",
            |_| "minecraft:air",
        );
        assert!(outside_chunk.is_empty());
    }

    #[test]
    fn structure_piece_reorient_facing_matches_vanilla_neighbor_rules() {
        use super::HorizontalDirection::{East, North, South, West};

        assert_eq!(
            super::structure_piece_reorient_facing(
                North,
                &[super::StructurePieceNeighborState {
                    direction: East,
                    chest: true,
                    solid_render: true,
                }]
            ),
            North
        );
        assert_eq!(
            super::structure_piece_reorient_facing(
                North,
                &[super::StructurePieceNeighborState {
                    direction: West,
                    chest: false,
                    solid_render: true,
                }]
            ),
            East
        );
        assert_eq!(
            super::structure_piece_reorient_facing(
                North,
                &[
                    super::StructurePieceNeighborState {
                        direction: West,
                        chest: false,
                        solid_render: true,
                    },
                    super::StructurePieceNeighborState {
                        direction: East,
                        chest: false,
                        solid_render: true,
                    },
                ]
            ),
            North
        );
        assert_eq!(
            super::structure_piece_reorient_facing(
                North,
                &[
                    super::StructurePieceNeighborState {
                        direction: North,
                        chest: false,
                        solid_render: true,
                    },
                    super::StructurePieceNeighborState {
                        direction: South,
                        chest: false,
                        solid_render: true,
                    },
                ]
            ),
            West
        );
        assert_eq!(
            super::structure_piece_reorient_facing(
                North,
                &[
                    super::StructurePieceNeighborState {
                        direction: North,
                        chest: false,
                        solid_render: true,
                    },
                    super::StructurePieceNeighborState {
                        direction: South,
                        chest: false,
                        solid_render: true,
                    },
                    super::StructurePieceNeighborState {
                        direction: West,
                        chest: false,
                        solid_render: true,
                    },
                ]
            ),
            East
        );
    }

    #[test]
    fn structure_piece_move_aggregate_box_and_collision_match_vanilla_helpers() {
        let first = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 10,
                min_z: 0,
                max_x: 4,
                max_y: 14,
                max_z: 4,
            },
        };
        let second = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 8,
                min_z: -2,
                max_x: 12,
                max_y: 18,
                max_z: 2,
            },
        };

        assert_eq!(
            super::structure_piece_moved(first, 3, -2, 5),
            super::StructurePieceModel {
                bounding_box: super::StructureBoundingBoxModel {
                    min_x: 3,
                    min_y: 8,
                    min_z: 5,
                    max_x: 7,
                    max_y: 12,
                    max_z: 9,
                },
            }
        );
        assert_eq!(
            super::structure_piece_create_bounding_box(&[first, second]),
            Ok(super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 8,
                min_z: -2,
                max_x: 12,
                max_y: 18,
                max_z: 4,
            })
        );
        assert_eq!(
            super::structure_piece_create_bounding_box(&[]),
            Err("Unable to calculate boundingbox without pieces".to_string())
        );
        assert_eq!(
            super::structure_piece_find_collision_piece(
                &[first, second],
                super::StructureBoundingBoxModel {
                    min_x: 11,
                    min_y: 0,
                    min_z: 0,
                    max_x: 20,
                    max_y: 20,
                    max_z: 10,
                },
            ),
            Some(second)
        );
        assert_eq!(
            super::structure_piece_find_collision_piece(
                &[first, second],
                super::StructureBoundingBoxModel {
                    min_x: 5,
                    min_y: 0,
                    min_z: 5,
                    max_x: 9,
                    max_y: 20,
                    max_z: 9,
                },
            ),
            None
        );
    }

    #[test]
    fn structure_piece_orientation_sets_mirror_and_rotation_like_vanilla() {
        use super::HorizontalDirection::{East, North, South, West};

        assert_eq!(
            super::structure_piece_orientation_state(None),
            super::StructurePieceOrientationState {
                orientation: None,
                mirror: super::StructurePieceMirror::None,
                rotation: super::StructurePieceRotation::None,
            }
        );
        assert_eq!(
            super::structure_piece_orientation_state(Some(North)),
            super::StructurePieceOrientationState {
                orientation: Some(North),
                mirror: super::StructurePieceMirror::None,
                rotation: super::StructurePieceRotation::None,
            }
        );
        assert_eq!(
            super::structure_piece_orientation_state(Some(South)),
            super::StructurePieceOrientationState {
                orientation: Some(South),
                mirror: super::StructurePieceMirror::LeftRight,
                rotation: super::StructurePieceRotation::None,
            }
        );
        assert_eq!(
            super::structure_piece_orientation_state(Some(West)),
            super::StructurePieceOrientationState {
                orientation: Some(West),
                mirror: super::StructurePieceMirror::LeftRight,
                rotation: super::StructurePieceRotation::Clockwise90,
            }
        );
        assert_eq!(
            super::structure_piece_orientation_state(Some(East)),
            super::StructurePieceOrientationState {
                orientation: Some(East),
                mirror: super::StructurePieceMirror::None,
                rotation: super::StructurePieceRotation::Clockwise90,
            }
        );
    }

    #[test]
    fn jigsaw_projection_liquid_padding_and_distance_models_match_vanilla_codecs() {
        assert_eq!(
            super::JigsawProjectionModel::TerrainMatching.id(),
            "terrain_matching"
        );
        assert_eq!(super::JigsawProjectionModel::Rigid.id(), "rigid");
        assert_eq!(
            super::JigsawProjectionModel::from_id("terrain_matching"),
            Some(super::JigsawProjectionModel::TerrainMatching)
        );
        assert_eq!(super::JigsawProjectionModel::from_id("loose"), None);
        assert_eq!(
            super::JigsawProjectionModel::TerrainMatching.processor_ids(),
            &["minecraft:gravity"]
        );
        assert!(super::JigsawProjectionModel::Rigid
            .processor_ids()
            .is_empty());

        assert_eq!(
            super::LiquidSettingsModel::from_id("apply_waterlogging"),
            Some(super::LiquidSettingsModel::ApplyWaterlogging)
        );
        assert_eq!(
            super::LiquidSettingsModel::IgnoreWaterlogging.id(),
            "ignore_waterlogging"
        );
        assert!(super::LiquidSettingsModel::ApplyWaterlogging.should_apply_waterlogging());
        assert!(!super::LiquidSettingsModel::IgnoreWaterlogging.should_apply_waterlogging());

        assert_eq!(
            super::DimensionPaddingModel::ZERO,
            super::DimensionPaddingModel { bottom: 0, top: 0 }
        );
        assert_eq!(
            super::DimensionPaddingModel::uniform(12),
            Ok(super::DimensionPaddingModel {
                bottom: 12,
                top: 12
            })
        );
        assert_eq!(
            super::DimensionPaddingModel::new(3, 7),
            Ok(super::DimensionPaddingModel { bottom: 3, top: 7 })
        );
        assert!(super::DimensionPaddingModel::uniform(12)
            .unwrap()
            .has_equal_top_and_bottom());
        assert!(!super::DimensionPaddingModel::new(3, 7)
            .unwrap()
            .has_equal_top_and_bottom());
        assert_eq!(
            super::DimensionPaddingModel::new(-1, 0),
            Err("dimension padding values must be non-negative".to_string())
        );

        assert_eq!(
            super::JigsawMaxDistanceModel::DEFAULT,
            super::JigsawMaxDistanceModel {
                horizontal: 80,
                vertical: 80,
            }
        );
        assert_eq!(
            super::JigsawMaxDistanceModel::uniform(80),
            Ok(super::JigsawMaxDistanceModel {
                horizontal: 80,
                vertical: 80,
            })
        );
        assert_eq!(
            super::JigsawMaxDistanceModel::new(128, 384),
            Ok(super::JigsawMaxDistanceModel {
                horizontal: 128,
                vertical: 384,
            })
        );
        assert!(super::JigsawMaxDistanceModel::uniform(32)
            .unwrap()
            .can_encode_as_uniform());
        assert!(!super::JigsawMaxDistanceModel::new(32, 64)
            .unwrap()
            .can_encode_as_uniform());
        assert_eq!(
            super::JigsawMaxDistanceModel::new(0, 64),
            Err("jigsaw horizontal max distance must be in 1..=128".to_string())
        );
        assert_eq!(
            super::JigsawMaxDistanceModel::new(64, 385),
            Err("jigsaw vertical max distance must be in 1..=384".to_string())
        );
    }

    #[test]
    fn jigsaw_connector_orientation_and_attachment_match_vanilla_can_attach() {
        use super::JigsawDirectionModel::{East, North, South, Up, West};
        use super::JigsawJointTypeModel::{Aligned, Rollable};

        assert_eq!(super::JigsawDirectionModel::from_id("north"), Some(North));
        assert_eq!(super::JigsawDirectionModel::from_id("sideways"), None);
        assert_eq!(North.id(), "north");
        assert_eq!(North.opposite(), South);
        assert_eq!(East.step(), super::BlockPos { x: 1, y: 0, z: 0 });
        assert_eq!(
            super::JigsawJointTypeModel::from_id("aligned"),
            Some(Aligned)
        );
        assert_eq!(Rollable.id(), "rollable");

        let source = super::JigsawConnectorModel {
            name: "minecraft:road",
            target: "minecraft:house",
            pool: "minecraft:village/plains/houses",
            front: North,
            top: Up,
            joint: Aligned,
            placement_priority: 3,
            selection_priority: 7,
        };
        let matching_target = super::JigsawConnectorModel {
            name: "minecraft:house",
            target: "minecraft:road",
            pool: "minecraft:empty",
            front: South,
            top: Up,
            joint: Rollable,
            placement_priority: 0,
            selection_priority: 0,
        };
        assert!(super::jigsaw_connectors_can_attach(
            &source,
            &matching_target
        ));
        assert_eq!(
            source.target_pos(super::BlockPos {
                x: 10,
                y: 64,
                z: -5
            }),
            super::BlockPos {
                x: 10,
                y: 64,
                z: -6
            }
        );

        let wrong_front = super::JigsawConnectorModel {
            front: West,
            ..matching_target
        };
        assert!(!super::jigsaw_connectors_can_attach(&source, &wrong_front));

        let wrong_top = super::JigsawConnectorModel {
            top: East,
            ..matching_target
        };
        assert!(!super::jigsaw_connectors_can_attach(&source, &wrong_top));

        let rollable_source = super::JigsawConnectorModel {
            joint: Rollable,
            ..source
        };
        assert!(super::jigsaw_connectors_can_attach(
            &rollable_source,
            &wrong_top
        ));

        let wrong_name = super::JigsawConnectorModel {
            name: "minecraft:stable",
            ..matching_target
        };
        assert!(!super::jigsaw_connectors_can_attach(&source, &wrong_name));
    }

    #[test]
    fn sequenced_priority_queue_matches_java_highest_priority_fifo_order() {
        let mut queue = super::SequencedPriorityQueueModel::new();
        assert!(queue.is_empty());
        assert_eq!(queue.highest_priority(), None);

        queue.add("low-a", -2);
        queue.add("high-a", 5);
        queue.add("mid-a", 1);
        queue.add("high-b", 5);
        queue.add("top-a", 9);

        assert_eq!(queue.highest_priority(), Some(9));
        assert_eq!(queue.next_item(), Some("top-a"));

        assert_eq!(queue.highest_priority(), Some(5));
        assert_eq!(queue.next_item(), Some("high-a"));

        queue.add("high-c", 5);
        queue.add("higher-late", 7);
        assert_eq!(queue.highest_priority(), Some(7));
        assert_eq!(queue.next_item(), Some("higher-late"));

        assert_eq!(queue.highest_priority(), Some(5));
        assert_eq!(queue.next_item(), Some("high-b"));
        assert_eq!(queue.next_item(), Some("high-c"));
        assert_eq!(queue.next_item(), Some("mid-a"));
        assert_eq!(queue.next_item(), Some("low-a"));
        assert_eq!(queue.next_item(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn jigsaw_start_height_limit_rejection_matches_dimension_padding_rules() {
        let fits_at_padded_edges = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: -60,
            min_z: 0,
            max_x: 8,
            max_y: 311,
            max_z: 8,
        };
        assert!(!super::jigsaw_start_too_close_to_world_height_limits(
            -64,
            384,
            super::DimensionPaddingModel { bottom: 4, top: 8 },
            fits_at_padded_edges,
        ));

        let below_padding = super::StructureBoundingBoxModel {
            min_y: -61,
            ..fits_at_padded_edges
        };
        assert!(super::jigsaw_start_too_close_to_world_height_limits(
            -64,
            384,
            super::DimensionPaddingModel { bottom: 4, top: 8 },
            below_padding,
        ));

        let above_padding = super::StructureBoundingBoxModel {
            max_y: 312,
            ..fits_at_padded_edges
        };
        assert!(super::jigsaw_start_too_close_to_world_height_limits(
            -64,
            384,
            super::DimensionPaddingModel { bottom: 4, top: 8 },
            above_padding,
        ));

        assert!(!super::jigsaw_start_too_close_to_world_height_limits(
            -64,
            384,
            super::DimensionPaddingModel::ZERO,
            super::StructureBoundingBoxModel {
                min_y: -10_000,
                max_y: 10_000,
                ..fits_at_padded_edges
            },
        ));
    }

    #[test]
    fn jigsaw_initial_expansion_bounds_match_java_aabb_and_padding_clamps() {
        assert_eq!(
            super::jigsaw_initial_expansion_bounds(
                100,
                70,
                -30,
                super::JigsawMaxDistanceModel {
                    horizontal: 80,
                    vertical: 96,
                },
                -64,
                384,
                super::DimensionPaddingModel { bottom: 4, top: 8 },
            ),
            super::JigsawExpansionBoundsModel {
                min_x: 20,
                min_y: -26,
                min_z: -110,
                max_x_exclusive: 181,
                max_y_exclusive: 167,
                max_z_exclusive: 51,
            }
        );

        assert_eq!(
            super::jigsaw_initial_expansion_bounds(
                0,
                300,
                0,
                super::JigsawMaxDistanceModel {
                    horizontal: 1,
                    vertical: 80,
                },
                -64,
                384,
                super::DimensionPaddingModel { bottom: 0, top: 16 },
            ),
            super::JigsawExpansionBoundsModel {
                min_x: -1,
                min_y: 220,
                min_z: -1,
                max_x_exclusive: 2,
                max_y_exclusive: 304,
                max_z_exclusive: 2,
            }
        );
    }

    #[test]
    fn jigsaw_start_anchor_adjustment_matches_named_start_jigsaw_math() {
        assert_eq!(
            super::jigsaw_start_anchor_adjustment(
                super::BlockPos {
                    x: 160,
                    y: 72,
                    z: -48,
                },
                super::BlockPos {
                    x: 166,
                    y: 75,
                    z: -61,
                },
            ),
            super::JigsawStartAnchorAdjustmentModel {
                local_anchor: super::BlockPos { x: 6, y: 3, z: -13 },
                adjusted_position: super::BlockPos {
                    x: 154,
                    y: 69,
                    z: -35,
                },
            }
        );

        let unchanged = super::BlockPos { x: 0, y: 64, z: 0 };
        assert_eq!(
            super::jigsaw_start_anchor_adjustment(unchanged, unchanged),
            super::JigsawStartAnchorAdjustmentModel {
                local_anchor: super::BlockPos { x: 0, y: 0, z: 0 },
                adjusted_position: unchanged,
            }
        );
    }

    #[test]
    fn jigsaw_junction_serialization_and_java_equality_match_vanilla() {
        let junction = super::JigsawJunctionModel {
            source_x: 12,
            source_ground_y: 70,
            source_z: -4,
            delta_y: 3,
            dest_projection: super::JigsawProjectionModel::TerrainMatching,
        };
        let tag = junction.serialize();
        assert_eq!(
            tag,
            super::JigsawJunctionTagModel {
                source_x: 12,
                source_ground_y: 70,
                source_z: -4,
                delta_y: 3,
                dest_proj: "terrain_matching",
            }
        );
        assert_eq!(
            super::JigsawJunctionModel::deserialize(tag),
            Some(junction.clone())
        );
        assert_eq!(
            super::JigsawJunctionModel::deserialize(super::JigsawJunctionTagModel {
                source_x: 0,
                source_ground_y: 0,
                source_z: 0,
                delta_y: 0,
                dest_proj: "",
            }),
            None
        );

        let different_ground_y = super::JigsawJunctionModel {
            source_ground_y: 99,
            ..junction.clone()
        };
        assert!(junction.java_equals(&different_ground_y));
        assert_ne!(
            junction.java_hash_inputs(),
            different_ground_y.java_hash_inputs()
        );

        let different_projection = super::JigsawJunctionModel {
            dest_projection: super::JigsawProjectionModel::Rigid,
            ..junction.clone()
        };
        assert!(!junction.java_equals(&different_projection));
    }

    #[test]
    fn jigsaw_pool_alias_lookup_resolves_direct_random_and_group_bindings_like_vanilla() {
        let direct = super::JigsawPoolAliasBindingModel::Direct {
            alias: "minecraft:village/common/well",
            target: "minecraft:village/plains/well",
        };
        assert_eq!(direct.codec_id(), "minecraft:direct");
        assert_eq!(direct.all_targets(), vec!["minecraft:village/plains/well"]);

        let group = super::JigsawPoolAliasBindingModel::RandomGroup {
            groups: vec![super::JigsawPoolAliasWeightedGroup {
                weight: 1,
                bindings: vec![
                    super::JigsawPoolAliasBindingModel::Random {
                        alias: "minecraft:village/common/houses",
                        targets: vec![super::JigsawPoolAliasWeightedTarget {
                            target: "minecraft:village/savanna/houses",
                            weight: 1,
                        }],
                    },
                    super::JigsawPoolAliasBindingModel::Direct {
                        alias: "minecraft:village/common/terminators",
                        target: "minecraft:village/savanna/terminators",
                    },
                ],
            }],
        };
        assert_eq!(group.codec_id(), "minecraft:random_group");
        assert_eq!(
            group.all_targets(),
            vec![
                "minecraft:village/savanna/houses",
                "minecraft:village/savanna/terminators",
            ]
        );

        let lookup =
            super::JigsawPoolAliasLookupModel::create(&[direct, group], (16, 72, -32), 12345);
        assert_eq!(
            lookup.lookup("minecraft:village/common/well"),
            "minecraft:village/plains/well"
        );
        assert_eq!(
            lookup.lookup("minecraft:village/common/houses"),
            "minecraft:village/savanna/houses"
        );
        assert_eq!(
            lookup.lookup("minecraft:village/common/terminators"),
            "minecraft:village/savanna/terminators"
        );
        assert_eq!(
            lookup.lookup("minecraft:village/plains/streets"),
            "minecraft:village/plains/streets"
        );

        let empty = super::JigsawPoolAliasLookupModel::create(&[], (0, 0, 0), 0);
        assert_eq!(empty.lookup("minecraft:empty"), "minecraft:empty");
    }

    #[test]
    fn jigsaw_pool_element_surfaces_match_vanilla_registry_and_pool_rules() {
        assert_eq!(
            super::JigsawPoolElementTypeModel::REGISTRY_ORDER.map(|element_type| element_type.id()),
            [
                "minecraft:single_pool_element",
                "minecraft:list_pool_element",
                "minecraft:feature_pool_element",
                "minecraft:empty_pool_element",
                "minecraft:legacy_single_pool_element",
            ]
        );

        let single = super::JigsawPoolElementModel::single(
            "minecraft:village/plains/houses/plains_small_house_1",
            &["minecraft:mossify_10_percent"],
            super::JigsawProjectionModel::TerrainMatching,
            Some(super::LiquidSettingsModel::IgnoreWaterlogging),
        );
        assert_eq!(single.ground_level_delta(), 1);
        assert_eq!(
            single.placement_processors(false),
            vec![
                "minecraft:structure_block",
                "minecraft:jigsaw_replacement",
                "minecraft:mossify_10_percent",
                "minecraft:gravity",
            ]
        );
        assert_eq!(
            single.placement_processors(true),
            vec![
                "minecraft:structure_block",
                "minecraft:mossify_10_percent",
                "minecraft:gravity",
            ]
        );

        let legacy = super::JigsawPoolElementModel::legacy_single(
            "minecraft:village/plains/town_centers/plains_fountain_01",
            &[],
            super::JigsawProjectionModel::Rigid,
            None,
        );
        assert_eq!(
            legacy.placement_processors(false),
            vec![
                "minecraft:structure_and_air",
                "minecraft:jigsaw_replacement"
            ]
        );

        let feature = super::JigsawPoolElementModel::feature(
            "minecraft:patch_grass",
            super::JigsawProjectionModel::TerrainMatching,
        );
        assert_eq!(
            feature.default_feature_jigsaw(),
            Some(super::DefaultFeatureJigsawModel {
                name: "minecraft:bottom",
                final_state: "minecraft:air",
                pool: "minecraft:empty",
                target: "minecraft:empty",
                joint: "rollable",
                orientation: "down_south",
            })
        );

        let empty = super::JigsawPoolElementModel::empty();
        assert_eq!(empty.empty_size(), Some((0, 0, 0)));
        assert_eq!(empty.empty_place_result(), Some(true));

        let list = super::JigsawPoolElementModel::list(
            vec![single.clone(), feature.clone()],
            super::JigsawProjectionModel::Rigid,
        )
        .expect("non-empty list element");
        assert!(list
            .children
            .iter()
            .all(|child| child.projection == super::JigsawProjectionModel::Rigid));
        assert_eq!(
            super::JigsawPoolElementModel::list(Vec::new(), super::JigsawProjectionModel::Rigid),
            Err("Elements are empty".to_string())
        );

        let pool = super::JigsawTemplatePoolModel::new(
            "minecraft:empty",
            vec![
                super::JigsawTemplatePoolElementEntry {
                    element: legacy,
                    weight: 2,
                },
                super::JigsawTemplatePoolElementEntry {
                    element: empty,
                    weight: 1,
                },
            ],
        )
        .expect("valid pool weights");
        assert_eq!(pool.size(), 3);
        assert_eq!(pool.get_weighted_template_index(0), Some(0));
        assert_eq!(pool.get_weighted_template_index(1), Some(0));
        assert_eq!(pool.get_weighted_template_index(2), Some(1));
        assert_eq!(pool.get_weighted_template_index(3), None);
        assert_eq!(
            super::JigsawTemplatePoolModel::new(
                "minecraft:empty",
                vec![super::JigsawTemplatePoolElementEntry {
                    element: feature,
                    weight: 151,
                }],
            ),
            Err("template pool element weight must be in 1..=150".to_string())
        );
    }

    #[test]
    fn jigsaw_candidate_pool_iteration_order_matches_java_target_fallback_and_empty_break() {
        let target_a = super::JigsawPoolElementModel::single(
            "minecraft:village/plains/houses/a",
            &[],
            super::JigsawProjectionModel::Rigid,
            None,
        );
        let target_b = super::JigsawPoolElementModel::single(
            "minecraft:village/plains/houses/b",
            &[],
            super::JigsawProjectionModel::Rigid,
            None,
        );
        let fallback = super::JigsawPoolElementModel::legacy_single(
            "minecraft:village/plains/fallback",
            &[],
            super::JigsawProjectionModel::TerrainMatching,
            None,
        );
        let target_pool = super::JigsawTemplatePoolModel::new(
            "minecraft:empty",
            vec![
                super::JigsawTemplatePoolElementEntry {
                    element: target_a.clone(),
                    weight: 2,
                },
                super::JigsawTemplatePoolElementEntry {
                    element: target_b.clone(),
                    weight: 1,
                },
            ],
        )
        .unwrap();
        let fallback_pool = super::JigsawTemplatePoolModel::new(
            "minecraft:empty",
            vec![
                super::JigsawTemplatePoolElementEntry {
                    element: fallback.clone(),
                    weight: 1,
                },
                super::JigsawTemplatePoolElementEntry {
                    element: super::JigsawPoolElementModel::empty(),
                    weight: 1,
                },
                super::JigsawTemplatePoolElementEntry {
                    element: target_b.clone(),
                    weight: 1,
                },
            ],
        )
        .unwrap();

        let candidates = super::jigsaw_candidate_elements_in_iteration_order(
            &target_pool,
            &fallback_pool,
            1,
            3,
            &[2, 0, 1],
            &[0, 1, 2],
        );
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| (candidate.source, candidate.raw_template_index))
                .collect::<Vec<_>>(),
            vec![
                (super::JigsawCandidatePoolSource::Target, 1),
                (super::JigsawCandidatePoolSource::Target, 0),
                (super::JigsawCandidatePoolSource::Target, 0),
                (super::JigsawCandidatePoolSource::Fallback, 0),
            ]
        );
        assert_eq!(candidates[0].element, target_b);
        assert_eq!(candidates[3].element, fallback);

        let max_depth_candidates = super::jigsaw_candidate_elements_in_iteration_order(
            &target_pool,
            &fallback_pool,
            3,
            3,
            &[2, 0, 1],
            &[0, 1, 2],
        );
        assert_eq!(
            max_depth_candidates
                .iter()
                .map(|candidate| candidate.source)
                .collect::<Vec<_>>(),
            vec![super::JigsawCandidatePoolSource::Fallback]
        );
    }

    #[test]
    fn jigsaw_expansion_hack_target_size_matches_java_pool_max_size_lookup() {
        let hack_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 15,
            max_y: 15,
            max_z: 15,
        };
        let jigsaws = vec![
            super::JigsawLocalConnectorModel {
                connector: super::JigsawConnectorModel {
                    name: "minecraft:street",
                    target: "minecraft:street",
                    pool: "minecraft:village/common/houses",
                    front: super::JigsawDirectionModel::East,
                    top: super::JigsawDirectionModel::Up,
                    joint: super::JigsawJointTypeModel::Aligned,
                    placement_priority: 0,
                    selection_priority: 0,
                },
                local_pos: super::BlockPos { x: 14, y: 6, z: 7 },
            },
            super::JigsawLocalConnectorModel {
                connector: super::JigsawConnectorModel {
                    name: "minecraft:street",
                    target: "minecraft:street",
                    pool: "minecraft:village/plains/ignored_outside",
                    front: super::JigsawDirectionModel::East,
                    top: super::JigsawDirectionModel::Up,
                    joint: super::JigsawJointTypeModel::Aligned,
                    placement_priority: 0,
                    selection_priority: 0,
                },
                local_pos: super::BlockPos { x: 15, y: 6, z: 7 },
            },
            super::JigsawLocalConnectorModel {
                connector: super::JigsawConnectorModel {
                    name: "minecraft:street",
                    target: "minecraft:street",
                    pool: "minecraft:village/plains/missing",
                    front: super::JigsawDirectionModel::North,
                    top: super::JigsawDirectionModel::Up,
                    joint: super::JigsawJointTypeModel::Aligned,
                    placement_priority: 0,
                    selection_priority: 0,
                },
                local_pos: super::BlockPos { x: 4, y: 6, z: 4 },
            },
        ];
        let pools = [
            super::JigsawPoolSizeModel {
                name: "minecraft:village/plains/houses",
                fallback: Some("minecraft:village/plains/terminators"),
                max_size: 4,
            },
            super::JigsawPoolSizeModel {
                name: "minecraft:village/plains/terminators",
                fallback: None,
                max_size: 9,
            },
            super::JigsawPoolSizeModel {
                name: "minecraft:village/plains/ignored_outside",
                fallback: None,
                max_size: 99,
            },
        ];
        let lookup = super::JigsawPoolAliasLookupModel {
            mappings: BTreeMap::from([(
                "minecraft:village/common/houses",
                "minecraft:village/plains/houses",
            )]),
        };

        assert_eq!(
            super::jigsaw_expansion_hack_target_size(true, hack_box, &jigsaws, &pools, &lookup),
            9
        );
        assert_eq!(
            super::jigsaw_expansion_hack_target_size(false, hack_box, &jigsaws, &pools, &lookup),
            0
        );
        assert_eq!(
            super::jigsaw_expansion_hack_target_size(
                true,
                super::StructureBoundingBoxModel {
                    max_y: 16,
                    ..hack_box
                },
                &jigsaws,
                &pools,
                &lookup,
            ),
            0
        );
    }

    #[test]
    fn jigsaw_pool_availability_decision_matches_java_warning_and_skip_rules() {
        assert_eq!(
            super::jigsaw_pool_availability_decision(false, 0, "minecraft:empty", 0),
            super::JigsawPoolAvailabilityDecisionModel {
                can_place_children: false,
                warning: Some(super::JigsawPoolAvailabilityWarning::EmptyOrNonExistentTarget),
            }
        );
        assert_eq!(
            super::jigsaw_pool_availability_decision(true, 0, "minecraft:empty", 0),
            super::JigsawPoolAvailabilityDecisionModel {
                can_place_children: false,
                warning: Some(super::JigsawPoolAvailabilityWarning::EmptyOrNonExistentTarget),
            }
        );
        assert_eq!(
            super::jigsaw_pool_availability_decision(true, 3, "minecraft:village/bad_fallback", 0),
            super::JigsawPoolAvailabilityDecisionModel {
                can_place_children: false,
                warning: Some(super::JigsawPoolAvailabilityWarning::EmptyOrNonExistentFallback),
            }
        );
        assert_eq!(
            super::jigsaw_pool_availability_decision(true, 3, "minecraft:empty", 0),
            super::JigsawPoolAvailabilityDecisionModel {
                can_place_children: true,
                warning: None,
            }
        );
        assert_eq!(
            super::jigsaw_pool_availability_decision(
                true,
                3,
                "minecraft:village/plains/terminators",
                2,
            ),
            super::JigsawPoolAvailabilityDecisionModel {
                can_place_children: true,
                warning: None,
            }
        );
    }

    #[test]
    fn jigsaw_structure_start_pools_match_java_bootstrap_start_keys() {
        assert_eq!(
            super::JIGSAW_STRUCTURE_START_POOLS
                .iter()
                .map(|entry| entry.pool)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:village/plains/town_centers",
                "minecraft:village/desert/town_centers",
                "minecraft:village/savanna/town_centers",
                "minecraft:village/snowy/town_centers",
                "minecraft:village/taiga/town_centers",
                "minecraft:pillager_outpost/base_plates",
                "minecraft:bastion/starts",
                "minecraft:ancient_city/city_center",
                "minecraft:trail_ruins/tower",
                "minecraft:trial_chambers/chamber/end",
            ]
        );
        assert!(super::JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .all(|entry| entry.pool.starts_with("minecraft:")));
        assert_eq!(
            super::JIGSAW_STRUCTURE_START_POOLS
                .iter()
                .filter(|entry| entry.structure_family.starts_with("village/"))
                .count(),
            5
        );
        assert!(super::JIGSAW_STRUCTURE_START_POOLS.iter().any(|entry| {
            entry.source_file == "TrialChambersStructurePools.java"
                && entry.pool == "minecraft:trial_chambers/chamber/end"
        }));
    }

    #[test]
    fn pool_element_structure_piece_state_and_junction_y_math_match_vanilla() {
        let element = super::JigsawPoolElementModel::single(
            "minecraft:bastion/starts/start",
            &[],
            super::JigsawProjectionModel::Rigid,
            None,
        );
        let mut piece = super::PoolElementStructurePieceModel::new(
            element,
            (10, 64, -8),
            1,
            super::StructurePieceRotation::Clockwise90,
            super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 64,
                min_z: -8,
                max_x: 20,
                max_y: 72,
                max_z: 2,
            },
            super::PoolElementStructurePieceModel::DEFAULT_LIQUID_SETTINGS,
        );
        piece.add_junction(super::JigsawJunctionModel {
            source_x: 21,
            source_ground_y: 66,
            source_z: -2,
            delta_y: 0,
            dest_projection: super::JigsawProjectionModel::Rigid,
        });
        piece.move_by(1, -2, 3);
        assert_eq!(piece.position, (11, 62, -5));
        assert_eq!(
            piece.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: 11,
                min_y: 62,
                min_z: -5,
                max_x: 21,
                max_y: 70,
                max_z: 5,
            }
        );
        assert_eq!(
            piece.save_tag(),
            super::PoolElementStructurePieceTagModel {
                pos_x: 11,
                pos_y: 62,
                pos_z: -5,
                ground_level_delta: 1,
                rotation: super::StructurePieceRotation::Clockwise90,
                junctions: vec![super::JigsawJunctionTagModel {
                    source_x: 21,
                    source_ground_y: 66,
                    source_z: -2,
                    delta_y: 0,
                    dest_proj: "rigid",
                }],
                liquid_settings: None,
            }
        );
        piece.liquid_settings = super::LiquidSettingsModel::IgnoreWaterlogging;
        assert_eq!(
            piece.save_tag().liquid_settings,
            Some(super::LiquidSettingsModel::IgnoreWaterlogging)
        );

        let both_rigid = super::jigsaw_child_placement_y(
            super::JigsawProjectionModel::Rigid,
            super::JigsawProjectionModel::Rigid,
            64,
            3,
            1,
            0,
            5,
            1,
            90,
        );
        assert_eq!(
            both_rigid,
            super::JigsawChildPlacementYModel {
                target_box_y: 66,
                target_ground_level_delta: 3,
                junction_y: 67,
                case: super::JigsawJunctionYOffsetCase::BothRigid,
            }
        );

        let terrain_target = super::jigsaw_child_placement_y(
            super::JigsawProjectionModel::Rigid,
            super::JigsawProjectionModel::TerrainMatching,
            64,
            3,
            1,
            0,
            5,
            1,
            90,
        );
        assert_eq!(terrain_target.target_box_y, 89);
        assert_eq!(terrain_target.target_ground_level_delta, 1);
        assert_eq!(
            terrain_target.case,
            super::JigsawJunctionYOffsetCase::SourceRigid
        );

        let both_terrain = super::jigsaw_child_placement_y(
            super::JigsawProjectionModel::TerrainMatching,
            super::JigsawProjectionModel::TerrainMatching,
            64,
            4,
            1,
            -1,
            2,
            1,
            91,
        );
        let delta_y = 4 - 1 - 1;
        assert_eq!(both_terrain.target_box_y, 90);
        assert_eq!(both_terrain.junction_y, 91 + delta_y / 2);
        assert_eq!(
            super::jigsaw_source_junction(
                (30, 68, -4),
                both_terrain,
                4,
                2,
                delta_y,
                super::JigsawProjectionModel::TerrainMatching,
            ),
            super::JigsawJunctionModel {
                source_x: 30,
                source_ground_y: 90,
                source_z: -4,
                delta_y,
                dest_projection: super::JigsawProjectionModel::TerrainMatching,
            }
        );
        assert_eq!(
            super::jigsaw_target_junction(
                (29, 67, -4),
                both_terrain,
                1,
                1,
                delta_y,
                super::JigsawProjectionModel::TerrainMatching,
            ),
            super::JigsawJunctionModel {
                source_x: 29,
                source_ground_y: 92,
                source_z: -4,
                delta_y: -delta_y,
                dest_projection: super::JigsawProjectionModel::TerrainMatching,
            }
        );
    }

    #[test]
    fn jigsaw_child_box_placement_matches_java_raw_box_and_y_offset_math() {
        let placement = super::jigsaw_child_placement_y(
            super::JigsawProjectionModel::Rigid,
            super::JigsawProjectionModel::Rigid,
            64,
            5,
            2,
            1,
            0,
            0,
            90,
        );
        assert_eq!(placement.target_box_y, 68);

        let box_placement = super::jigsaw_child_box_placement(
            super::BlockPos {
                x: 101,
                y: 70,
                z: -31,
            },
            super::BlockPos { x: 4, y: 2, z: 6 },
            super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 9,
                max_y: 7,
                max_z: 11,
            },
            placement,
        );

        assert_eq!(
            box_placement,
            super::JigsawChildBoxPlacementModel {
                raw_target_box_pos: super::BlockPos {
                    x: 97,
                    y: 68,
                    z: -37,
                },
                raw_target_bounding_box: super::StructureBoundingBoxModel {
                    min_x: 97,
                    min_y: 68,
                    min_z: -37,
                    max_x: 106,
                    max_y: 75,
                    max_z: -26,
                },
                y_offset: 0,
                target_box_position: super::BlockPos {
                    x: 97,
                    y: 68,
                    z: -37,
                },
                target_bounding_box: super::StructureBoundingBoxModel {
                    min_x: 97,
                    min_y: 68,
                    min_z: -37,
                    max_x: 106,
                    max_y: 75,
                    max_z: -26,
                },
            }
        );

        let terrain_placement = super::jigsaw_child_placement_y(
            super::JigsawProjectionModel::TerrainMatching,
            super::JigsawProjectionModel::Rigid,
            64,
            5,
            2,
            -1,
            0,
            0,
            91,
        );
        let moved = super::jigsaw_child_box_placement(
            super::BlockPos {
                x: 101,
                y: 70,
                z: -31,
            },
            super::BlockPos { x: 4, y: 2, z: 6 },
            super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 9,
                max_y: 7,
                max_z: 11,
            },
            terrain_placement,
        );
        assert_eq!(terrain_placement.target_box_y, 89);
        assert_eq!(moved.y_offset, 21);
        assert_eq!(moved.target_box_position.y, 89);
        assert_eq!(moved.target_bounding_box.min_y, 89);
        assert_eq!(moved.target_bounding_box.max_y, 96);
    }

    #[test]
    fn jigsaw_expansion_hack_box_encapsulation_matches_java_height_math() {
        let target_box = super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 64,
            min_z: -4,
            max_x: 18,
            max_y: 71,
            max_z: 4,
        };
        assert_eq!(
            super::jigsaw_apply_expansion_hack_to_target_box(target_box, 0),
            target_box
        );
        assert_eq!(
            super::jigsaw_apply_expansion_hack_to_target_box(target_box, 2),
            target_box
        );
        assert_eq!(
            super::jigsaw_apply_expansion_hack_to_target_box(target_box, 9),
            super::StructureBoundingBoxModel {
                max_y: 74,
                ..target_box
            }
        );
        assert_eq!(
            super::jigsaw_apply_expansion_hack_to_target_box(target_box, -1),
            target_box
        );
    }

    #[test]
    fn jigsaw_child_free_shape_selection_matches_java_source_inside_check() {
        let source_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 15,
            max_y: 79,
            max_z: 15,
        };
        assert_eq!(
            super::jigsaw_child_free_shape_selection(
                source_box,
                super::BlockPos {
                    x: 15,
                    y: 79,
                    z: 15
                },
                false,
            ),
            super::JigsawChildFreeShapeSelectionModel {
                scope: super::JigsawChildFreeShapeScope::SourcePiece,
                initialized_source_shape: Some(source_box),
            }
        );
        assert_eq!(
            super::jigsaw_child_free_shape_selection(
                source_box,
                super::BlockPos { x: 4, y: 70, z: 4 },
                true,
            ),
            super::JigsawChildFreeShapeSelectionModel {
                scope: super::JigsawChildFreeShapeScope::SourcePiece,
                initialized_source_shape: None,
            }
        );
        assert_eq!(
            super::jigsaw_child_free_shape_selection(
                source_box,
                super::BlockPos { x: 16, y: 70, z: 4 },
                false,
            ),
            super::JigsawChildFreeShapeSelectionModel {
                scope: super::JigsawChildFreeShapeScope::Context,
                initialized_source_shape: None,
            }
        );
    }

    #[test]
    fn jigsaw_accepted_child_scheduling_matches_java_depth_and_priority_rule() {
        assert_eq!(
            super::jigsaw_accepted_child_scheduling(0, 1, 7),
            super::JigsawAcceptedChildSchedulingModel {
                child_depth: 1,
                queue_for_expansion: true,
                placement_priority: 7,
            }
        );
        assert_eq!(
            super::jigsaw_accepted_child_scheduling(1, 1, -3),
            super::JigsawAcceptedChildSchedulingModel {
                child_depth: 2,
                queue_for_expansion: false,
                placement_priority: -3,
            }
        );
        assert_eq!(
            super::jigsaw_accepted_child_scheduling(19, 20, i32::MAX),
            super::JigsawAcceptedChildSchedulingModel {
                child_depth: 20,
                queue_for_expansion: true,
                placement_priority: i32::MAX,
            }
        );
    }

    #[test]
    fn structure_processor_surfaces_and_core_decisions_match_vanilla_templatesystem() {
        assert_eq!(
            super::StructureProcessorTypeModel::REGISTRY_ORDER.map(|processor| processor.id()),
            [
                "minecraft:block_ignore",
                "minecraft:block_rot",
                "minecraft:gravity",
                "minecraft:jigsaw_replacement",
                "minecraft:rule",
                "minecraft:nop",
                "minecraft:block_age",
                "minecraft:blackstone_replace",
                "minecraft:lava_submerged_block",
                "minecraft:protected_blocks",
                "minecraft:capped",
            ]
        );
        assert_eq!(
            super::StructureRuleTestTypeModel::REGISTRY_ORDER.map(|rule| rule.id()),
            [
                "minecraft:always_true",
                "minecraft:block_match",
                "minecraft:blockstate_match",
                "minecraft:tag_match",
                "minecraft:random_block_match",
                "minecraft:random_blockstate_match",
            ]
        );
        assert_eq!(
            super::StructurePosRuleTestTypeModel::REGISTRY_ORDER.map(|rule| rule.id()),
            [
                "minecraft:always_true",
                "minecraft:linear_pos",
                "minecraft:axis_aligned_linear_pos",
            ]
        );

        let ignore = super::StructureProcessorModel::BlockIgnore {
            blocks: vec!["minecraft:air", "minecraft:structure_block"],
        };
        assert_eq!(ignore.codec_id(), "minecraft:block_ignore");
        assert!(ignore.block_ignore_should_drop("minecraft:air"));
        assert!(!ignore.block_ignore_should_drop("minecraft:stone"));

        let rot_all = super::StructureProcessorModel::BlockRot {
            rottable_blocks: None,
            integrity: 0.35,
        };
        assert!(rot_all.block_rot_keeps(false, 0.35));
        assert!(!rot_all.block_rot_keeps(false, 0.35001));
        let rot_tagged = super::StructureProcessorModel::BlockRot {
            rottable_blocks: Some("minecraft:replaceable"),
            integrity: 0.0,
        };
        assert!(rot_tagged.block_rot_keeps(false, 0.99));
        assert!(!rot_tagged.block_rot_keeps(true, 0.01));

        let jigsaw = super::StructureProcessorModel::JigsawReplacement;
        assert_eq!(
            jigsaw.jigsaw_replacement_output(
                "minecraft:jigsaw",
                Some("minecraft:oak_planks"),
                false
            ),
            Some("minecraft:oak_planks")
        );
        assert_eq!(
            jigsaw.jigsaw_replacement_output(
                "minecraft:jigsaw",
                Some("minecraft:structure_void"),
                false
            ),
            None
        );
        assert_eq!(
            jigsaw.jigsaw_replacement_output(
                "minecraft:jigsaw",
                Some("minecraft:oak_planks"),
                true
            ),
            Some("minecraft:jigsaw")
        );

        let gravity = super::StructureProcessorModel::Gravity {
            heightmap: "WORLD_SURFACE_WG",
            offset: -1,
        };
        assert_eq!(
            gravity.gravity_adjusted_y(80, 3, true),
            Some(("WORLD_SURFACE", 82))
        );
        assert_eq!(
            gravity.gravity_adjusted_y(80, 3, false),
            Some(("WORLD_SURFACE_WG", 82))
        );

        let lava = super::StructureProcessorModel::LavaSubmergedBlock;
        assert_eq!(
            lava.lava_submerged_output("minecraft:lava", false, "minecraft:chain"),
            "minecraft:lava"
        );
        assert_eq!(
            lava.lava_submerged_output("minecraft:lava", true, "minecraft:stone"),
            "minecraft:stone"
        );

        let capped = super::StructureProcessorModel::Capped {
            delegate: Box::new(super::StructureProcessorModel::Nop),
            limit: 2,
        };
        assert!(capped.capped_can_run(3, 3, 2));
        assert!(!capped.capped_can_run(3, 2, 2));
        assert!(!capped.capped_can_run(3, 3, 0));

        assert!(super::structure_random_rule_test_matches(true, 0.25, 0.249));
        assert!(!super::structure_random_rule_test_matches(true, 0.25, 0.25));
        assert_eq!(
            super::structure_linear_pos_chance(5, 0, 10, 0.2, 0.8),
            Ok(0.5)
        );
        assert_eq!(
            super::structure_linear_pos_chance(0, 4, 4, 0.0, 1.0),
            Err("Invalid range: [4,4]".to_string())
        );
        assert_eq!(
            super::structure_axis_aligned_distance((10, 65, -4), (3, 60, 1), 'x'),
            7
        );
        assert_eq!(
            super::structure_axis_aligned_distance((10, 65, -4), (3, 60, 1), 'z'),
            5
        );
    }

    #[test]
    fn rule_block_entity_modifiers_match_vanilla_template_rule_behavior() {
        assert_eq!(
            super::RuleBlockEntityModifierTypeModel::REGISTRY_ORDER.map(|modifier| modifier.id()),
            [
                "minecraft:clear",
                "minecraft:passthrough",
                "minecraft:append_static",
                "minecraft:append_loot",
            ]
        );

        let existing = super::TemplateCompoundTagModel::default()
            .with_string("id", "minecraft:chest")
            .with_string("CustomName", "{\"text\":\"Old\"}");
        let static_data = super::TemplateCompoundTagModel::default()
            .with_string("CustomName", "{\"text\":\"New\"}")
            .with_string("Lock", "key");

        let mut random =
            super::RandomSourceKind::new(12345, crate::random_source::RandomAlgorithm::Legacy);
        assert_eq!(
            super::RuleBlockEntityModifierModel::Passthrough
                .apply(&mut random, Some(existing.clone())),
            Some(existing.clone())
        );
        assert_eq!(
            super::RuleBlockEntityModifierModel::Passthrough.apply(&mut random, None),
            None
        );
        assert_eq!(
            super::RuleBlockEntityModifierModel::Clear.apply(&mut random, Some(existing.clone())),
            Some(super::TemplateCompoundTagModel::default())
        );

        let appended = super::RuleBlockEntityModifierModel::AppendStatic { data: static_data }
            .apply(&mut random, Some(existing.clone()))
            .expect("append static returns tag");
        assert_eq!(
            appended.values.get("id"),
            Some(&super::TemplateNbtValueModel::String("minecraft:chest"))
        );
        assert_eq!(
            appended.values.get("CustomName"),
            Some(&super::TemplateNbtValueModel::String("{\"text\":\"New\"}"))
        );
        assert_eq!(
            appended.values.get("Lock"),
            Some(&super::TemplateNbtValueModel::String("key"))
        );

        let mut loot_random =
            super::RandomSourceKind::new(12345, crate::random_source::RandomAlgorithm::Legacy);
        let loot = super::RuleBlockEntityModifierModel::AppendLoot {
            loot_table: "minecraft:chests/simple_dungeon",
        };
        assert_eq!(loot.codec_id(), "minecraft:append_loot");
        let loot_tag = loot
            .apply(&mut loot_random, Some(existing))
            .expect("append loot returns tag");
        assert_eq!(
            loot_tag.values.get("LootTable"),
            Some(&super::TemplateNbtValueModel::String(
                "minecraft:chests/simple_dungeon"
            ))
        );
        assert_eq!(
            loot_tag.values.get("LootTableSeed"),
            Some(&super::TemplateNbtValueModel::Long(6674089274190705457))
        );
    }

    #[test]
    fn structure_template_manager_paths_and_cache_follow_vanilla_loader_rules() {
        let factory = super::TemplatePathFactoryModel::new("/world/generated");
        let village = crate::registry::Identifier::parse(
            "minecraft:village/plains/houses/plains_small_house_1",
        )
        .expect("valid template id");
        assert_eq!(
            factory.create_and_validate_path_to_structure(
                &village,
                super::StructureTemplateFileKind::Nbt,
            ),
            Ok("/world/generated/minecraft/structure/village/plains/houses/plains_small_house_1.nbt"
                .to_string())
        );
        assert_eq!(
            factory.create_and_validate_path_to_structure(
                &village,
                super::StructureTemplateFileKind::Snbt,
            ),
            Ok("/world/generated/minecraft/structure/village/plains/houses/plains_small_house_1.snbt"
                .to_string())
        );

        let data_factory = super::TemplatePathFactoryModel::for_pack_type("/tmp/tests", "data");
        let resource =
            crate::registry::Identifier::parse("minecraft:structure/trial_chambers/start.nbt")
                .expect("valid resource path");
        assert_eq!(
            data_factory.create_and_validate_path_to_resource(&resource),
            Ok("/tmp/tests/data/minecraft/structure/trial_chambers/start.nbt".to_string())
        );

        let traversal = crate::registry::Identifier::parse("minecraft:structure/../bad.nbt")
            .expect("registry identifier allows dotted path segments");
        assert_eq!(
            factory.create_and_validate_path_to_resource(&traversal),
            Err(
                "Invalid file path 'minecraft:structure/../bad.nbt': invalid path segment '..'"
                    .to_string()
            )
        );
        let uppercase = crate::registry::Identifier::new("minecraft", "structure/Bad.nbt");
        assert!(uppercase.is_err());
        let dot_segment =
            crate::registry::Identifier::parse("minecraft:structure/./bad.nbt").unwrap();
        assert_eq!(
            factory.create_and_validate_path_to_resource(&dot_segment),
            Err(
                "Invalid file path 'minecraft:structure/./bad.nbt': invalid path segment '.'"
                    .to_string()
            )
        );

        assert_eq!(
            super::StructureTemplateManagerModel::save_kind(false),
            super::StructureTemplateFileKind::Nbt
        );
        assert_eq!(
            super::StructureTemplateManagerModel::save_kind(true),
            super::StructureTemplateFileKind::Snbt
        );

        let mut manager = super::StructureTemplateManagerModel::default();
        let missing = crate::registry::Identifier::parse("minecraft:missing").unwrap();
        assert_eq!(manager.get_or_try_load(missing.clone(), |_| None), None);
        assert_eq!(
            manager.get_or_try_load(missing.clone(), |_| Some("should_not_reload")),
            None
        );
        manager.remove(&missing);
        assert_eq!(
            manager.get_or_try_load(missing.clone(), |_| Some("loaded_after_remove")),
            Some("loaded_after_remove")
        );
        manager.on_resource_manager_reload();
        assert!(manager.cache.is_empty());
        assert_eq!(manager.get_or_create(missing), "runtime_template");
    }

    #[test]
    fn template_sources_load_in_vanilla_order_and_list_distinct_templates() {
        let id = crate::registry::Identifier::parse(
            "minecraft:village/plains/town_centers/plains_fountain_01",
        )
        .expect("valid template id");
        let generated = super::TemplateSourceModel::directory(
            Some("/world/generated"),
            false,
            &[(
                "minecraft:village/plains/town_centers/plains_fountain_01",
                "generated_nbt",
            )],
            &[],
        );
        let tests = super::TemplateSourceModel::directory(
            Some("/tmp/tests/data"),
            true,
            &[("minecraft:test/marker", "test_snbt")],
            &[],
        );
        let resources = super::TemplateSourceModel::resource_manager(
            &[
                (
                    "minecraft:village/plains/town_centers/plains_fountain_01",
                    "resource_pack_nbt",
                ),
                ("minecraft:bastion/starts/start", "bastion_nbt"),
            ],
            &[],
        );

        let mut manager = super::StructureTemplateManagerModel::default();
        let (loaded, attempts) = manager.try_load_from_sources(
            id.clone(),
            &[generated.clone(), tests.clone(), resources.clone()],
        );
        assert_eq!(loaded, Some("generated_nbt"));
        assert_eq!(
            attempts,
            vec![super::TemplateLoadAttemptModel {
                source_kind: super::TemplateSourceKindModel::Directory,
                id: id.clone(),
                result: super::TemplateLoadAttemptResultModel::Loaded,
            }]
        );

        let (cached, cached_attempts) =
            manager.try_load_from_sources(id.clone(), &[resources.clone()]);
        assert_eq!(cached, Some("generated_nbt"));
        assert!(cached_attempts.is_empty());

        let missing = crate::registry::Identifier::parse("minecraft:missing/template").unwrap();
        let unavailable = super::TemplateSourceModel::directory(None, false, &[], &[]);
        let failing_resource = super::TemplateSourceModel::resource_manager(
            &[("minecraft:missing/template", "would_have_loaded")],
            &["minecraft:missing/template"],
        );
        let (missing_result, missing_attempts) =
            manager.try_load_from_sources(missing.clone(), &[unavailable, failing_resource]);
        assert_eq!(missing_result, None);
        assert_eq!(
            missing_attempts
                .iter()
                .map(|attempt| attempt.result)
                .collect::<Vec<_>>(),
            vec![
                super::TemplateLoadAttemptResultModel::SourceUnavailable,
                super::TemplateLoadAttemptResultModel::ErrorSuppressed,
            ]
        );
        let (cached_missing, cached_missing_attempts) =
            manager.try_load_from_sources(missing, &[resources.clone()]);
        assert_eq!(cached_missing, None);
        assert!(cached_missing_attempts.is_empty());

        let listed = super::StructureTemplateManagerModel::list_templates_from_sources(&[
            generated,
            tests.clone(),
            resources,
        ]);
        assert_eq!(
            listed,
            vec![
                crate::registry::Identifier::parse(
                    "minecraft:village/plains/town_centers/plains_fountain_01"
                )
                .unwrap(),
                crate::registry::Identifier::parse("minecraft:test/marker").unwrap(),
                crate::registry::Identifier::parse("minecraft:bastion/starts/start").unwrap(),
            ]
        );
        assert!(tests.load_as_text);
    }

    #[test]
    fn jigsaw_structure_config_validation_and_generation_point_match_vanilla() {
        let start_height = super::HeightProvider::Constant {
            value: super::VerticalAnchor::Absolute(72),
        };
        let structure = super::JigsawStructureModel::new_full(
            "minecraft:village/plains/town_centers",
            Some("minecraft:town_centers"),
            7,
            start_height,
            true,
            Some("WORLD_SURFACE_WG"),
            super::JigsawMaxDistanceModel::new(80, 128).expect("valid max distance"),
            vec![super::JigsawPoolAliasBindingModel::Direct {
                alias: "minecraft:village/common/houses",
                target: "minecraft:village/plains/houses",
            }],
            super::DimensionPaddingModel { bottom: 4, top: 8 },
            super::LiquidSettingsModel::IgnoreWaterlogging,
            super::TerrainAdjustmentModel::BeardThin,
        )
        .expect("valid jigsaw structure config");
        assert_eq!(structure.verify_range(), Ok(()));

        let point = structure.find_generation_point(
            ChunkPos { x: -2, z: 3 },
            super::WorldGenerationHeightContext {
                min_y: -64,
                height: 384,
            },
            12345,
            0,
            0,
            0,
        );
        assert_eq!(point.start_pos, (-32, 72, 48));
        assert_eq!(point.max_depth, 7);
        assert!(point.use_expansion_hack);
        assert_eq!(point.project_start_to_heightmap, Some("WORLD_SURFACE_WG"));
        assert_eq!(
            point
                .pool_alias_lookup
                .lookup("minecraft:village/common/houses"),
            "minecraft:village/plains/houses"
        );
        assert_eq!(
            point.max_distance_from_center,
            super::JigsawMaxDistanceModel {
                horizontal: 80,
                vertical: 128,
            }
        );
        assert_eq!(
            point.dimension_padding,
            super::DimensionPaddingModel { bottom: 4, top: 8 }
        );
        assert_eq!(
            point.liquid_settings,
            super::LiquidSettingsModel::IgnoreWaterlogging
        );

        assert_eq!(
            super::JigsawStructureModel::new(
                "minecraft:empty",
                -1,
                start_height,
                false,
                super::TerrainAdjustmentModel::None,
            ),
            Err("jigsaw structure size must be in 0..=20".to_string())
        );
        assert_eq!(
            super::JigsawStructureModel::new_full(
                "minecraft:empty",
                None,
                0,
                start_height,
                false,
                None,
                super::JigsawMaxDistanceModel::new(128, 80).expect("valid max distance"),
                Vec::new(),
                super::DimensionPaddingModel::ZERO,
                super::LiquidSettingsModel::ApplyWaterlogging,
                super::TerrainAdjustmentModel::Bury,
            ),
            Err(
                "Horizontal structure size including terrain adaptation must not exceed 128"
                    .to_string()
            )
        );

        let compact = super::JigsawStructureModel::new(
            "minecraft:bastion/starts",
            3,
            start_height,
            false,
            super::TerrainAdjustmentModel::None,
        )
        .expect("compact constructor fills vanilla defaults");
        assert_eq!(compact.start_jigsaw_name, None);
        assert_eq!(
            compact.max_distance_from_center,
            super::JigsawMaxDistanceModel::DEFAULT
        );
        assert_eq!(
            compact.dimension_padding,
            super::DimensionPaddingModel::ZERO
        );
        assert_eq!(
            compact.liquid_settings,
            super::LiquidSettingsModel::ApplyWaterlogging
        );
    }

    #[test]
    fn structure_check_presence_and_lookup_branches_match_vanilla() {
        assert_eq!(
            super::structure_check_result_from_cached_references(None, false),
            super::StructureCheckResultModel::StartNotPresent
        );
        assert_eq!(
            super::structure_check_result_from_cached_references(Some(0), true),
            super::StructureCheckResultModel::StartPresent
        );
        assert_eq!(
            super::structure_check_result_from_cached_references(Some(1), true),
            super::StructureCheckResultModel::StartNotPresent
        );
        assert_eq!(
            super::structure_check_result_from_cached_references(Some(2), false),
            super::StructureCheckResultModel::StartPresent
        );

        assert!(!super::structure_fast_check_allows_lookup(
            super::StructureCheckResultModel::StartNotPresent
        ));
        assert!(super::structure_fast_check_allows_lookup(
            super::StructureCheckResultModel::ChunkLoadNeeded
        ));
        assert!(super::structure_locate_can_return_fast(
            super::StructureCheckResultModel::StartPresent,
            false
        ));
        assert!(!super::structure_locate_can_return_fast(
            super::StructureCheckResultModel::StartPresent,
            true
        ));
        assert!(!super::structure_locate_can_return_fast(
            super::StructureCheckResultModel::ChunkLoadNeeded,
            false
        ));

        let piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 20,
                min_z: 0,
                max_x: 15,
                max_y: 30,
                max_z: 15,
            },
        };
        let mut start = super::StructureStartModel {
            structure: Some("minecraft:desert_pyramid"),
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: vec![piece],
        };
        assert!(super::structure_start_can_satisfy_lookup(&start, false));
        assert!(super::structure_start_can_satisfy_lookup(&start, true));
        assert!(super::structure_try_add_reference(&mut start));
        assert_eq!(start.references, 1);
        assert!(super::structure_start_can_satisfy_lookup(&start, false));
        assert!(!super::structure_start_can_satisfy_lookup(&start, true));
        assert!(!super::structure_try_add_reference(&mut start));

        let invalid = super::StructureStartModel::invalid();
        assert!(!super::structure_start_can_satisfy_lookup(&invalid, false));
    }

    #[test]
    fn terrain_adjustment_ids_and_bounding_boxes_match_vanilla() {
        let ids = [
            (super::TerrainAdjustmentModel::None, "none"),
            (super::TerrainAdjustmentModel::Bury, "bury"),
            (super::TerrainAdjustmentModel::BeardThin, "beard_thin"),
            (super::TerrainAdjustmentModel::BeardBox, "beard_box"),
            (super::TerrainAdjustmentModel::Encapsulate, "encapsulate"),
        ];
        for (adjustment, id) in ids {
            assert_eq!(adjustment.id(), id);
            assert_eq!(super::TerrainAdjustmentModel::from_id(id), Some(adjustment));
        }
        assert_eq!(super::TerrainAdjustmentModel::from_id("beard"), None);

        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 40,
            max_y: 50,
            max_z: 60,
        };
        assert_eq!(
            super::structure_adjust_bounding_box(super::TerrainAdjustmentModel::None, bounding_box),
            bounding_box
        );
        assert_eq!(
            super::structure_adjust_bounding_box(super::TerrainAdjustmentModel::Bury, bounding_box),
            super::StructureBoundingBoxModel {
                min_x: -2,
                min_y: 8,
                min_z: 18,
                max_x: 52,
                max_y: 62,
                max_z: 72,
            }
        );

        assert!(super::jigsaw_max_distance_with_terrain_is_valid(
            128,
            super::TerrainAdjustmentModel::None
        ));
        assert!(!super::jigsaw_max_distance_with_terrain_is_valid(
            128,
            super::TerrainAdjustmentModel::Bury
        ));
        assert!(super::jigsaw_max_distance_with_terrain_is_valid(
            116,
            super::TerrainAdjustmentModel::Encapsulate
        ));
        assert!(!super::jigsaw_max_distance_with_terrain_is_valid(
            117,
            super::TerrainAdjustmentModel::BeardBox
        ));
    }

    #[test]
    fn structure_access_stores_starts_and_reference_sets_like_chunks() {
        let piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 64,
                min_z: 48,
                max_x: 47,
                max_y: 80,
                max_z: 63,
            },
        };
        let start = super::StructureStartModel {
            structure: Some("minecraft:shipwreck"),
            chunk_pos: ChunkPos { x: 2, z: 3 },
            references: 0,
            pieces: vec![piece],
        };
        let invalid = super::StructureStartModel::invalid();
        let mut access = super::StructureAccessModel::default();

        assert!(access
            .get_start_for_structure("minecraft:shipwreck")
            .is_none());
        assert_eq!(
            access.get_references_for_structure("minecraft:shipwreck"),
            &[] as &[i64]
        );
        assert!(!access.has_any_structure_references());
        assert!(!access.unsaved);

        access.set_start_for_structure("minecraft:shipwreck", start.clone());
        access.set_start_for_structure("minecraft:mineshaft", invalid.clone());
        assert_eq!(
            access.get_start_for_structure("minecraft:shipwreck"),
            Some(&start)
        );
        assert!(access.unsaved);

        access.add_reference_for_structure("minecraft:shipwreck", 0x0000_0002_0000_0003);
        access.add_reference_for_structure("minecraft:shipwreck", 0x0000_0002_0000_0003);
        assert_eq!(
            access.get_references_for_structure("minecraft:shipwreck"),
            &[0x0000_0002_0000_0003]
        );
        assert!(access.has_any_structure_references());

        let mut replacement_starts = BTreeMap::new();
        replacement_starts.insert("minecraft:mineshaft", invalid.clone());
        access.set_all_starts(replacement_starts);
        assert!(access
            .get_start_for_structure("minecraft:shipwreck")
            .is_none());
        assert_eq!(
            access.get_start_for_structure("minecraft:mineshaft"),
            Some(&invalid)
        );

        let mut replacement_references = BTreeMap::new();
        replacement_references.insert("minecraft:mineshaft", vec![7]);
        access.set_all_references(replacement_references);
        assert_eq!(
            access.get_references_for_structure("minecraft:mineshaft"),
            &[7]
        );
    }

    #[test]
    fn structure_access_resolves_only_valid_referenced_starts() {
        let valid_start = super::StructureStartModel {
            structure: Some("minecraft:buried_treasure"),
            chunk_pos: ChunkPos { x: 1, z: 1 },
            references: 0,
            pieces: vec![super::StructurePieceModel {
                bounding_box: super::StructureBoundingBoxModel {
                    min_x: 16,
                    min_y: 45,
                    min_z: 16,
                    max_x: 31,
                    max_y: 55,
                    max_z: 31,
                },
            }],
        };
        let mut valid_access = super::StructureAccessModel::default();
        valid_access.set_start_for_structure("minecraft:buried_treasure", valid_start.clone());

        let mut invalid_access = super::StructureAccessModel::default();
        invalid_access.set_start_for_structure(
            "minecraft:buried_treasure",
            super::StructureStartModel::invalid(),
        );

        let mut chunks = BTreeMap::new();
        chunks.insert(11, valid_access);
        chunks.insert(12, invalid_access);
        let starts = super::structure_access_valid_starts_for_references(
            &chunks,
            "minecraft:buried_treasure",
            &[11, 12, 13],
        );
        assert_eq!(starts, vec![valid_start]);
    }

    #[test]
    fn chunk_generator_create_references_scans_nearby_valid_intersections() {
        let shipwreck_start = super::StructureStartModel {
            structure: Some("minecraft:shipwreck"),
            chunk_pos: ChunkPos { x: 2, z: -1 },
            references: 0,
            pieces: vec![super::StructurePieceModel {
                bounding_box: super::StructureBoundingBoxModel {
                    min_x: 31,
                    min_y: 50,
                    min_z: -16,
                    max_x: 48,
                    max_y: 70,
                    max_z: -1,
                },
            }],
        };
        let out_of_range_start = super::StructureStartModel {
            structure: Some("minecraft:mineshaft"),
            chunk_pos: ChunkPos { x: 9, z: 0 },
            references: 0,
            pieces: vec![super::StructurePieceModel {
                bounding_box: super::StructureBoundingBoxModel {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 15,
                    max_y: 10,
                    max_z: 15,
                },
            }],
        };
        let non_intersecting_start = super::StructureStartModel {
            structure: Some("minecraft:village_plains"),
            chunk_pos: ChunkPos { x: -1, z: 0 },
            references: 0,
            pieces: vec![super::StructurePieceModel {
                bounding_box: super::StructureBoundingBoxModel {
                    min_x: -32,
                    min_y: 60,
                    min_z: 0,
                    max_x: -17,
                    max_y: 80,
                    max_z: 15,
                },
            }],
        };

        assert_eq!(
            super::chunk_pos_key(ChunkPos { x: -1, z: 2 }),
            0x0000_0002_ffff_ffff
        );
        assert_eq!(
            super::structure_reference_writable_area(ChunkPos { x: 2, z: -1 }),
            super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: i32::MIN,
                min_z: -16,
                max_x: 47,
                max_y: i32::MAX,
                max_z: -1,
            }
        );

        let mut source_access = super::StructureAccessModel::default();
        source_access.set_start_for_structure("minecraft:shipwreck", shipwreck_start);
        source_access.set_start_for_structure(
            "minecraft:stronghold",
            super::StructureStartModel::invalid(),
        );

        let mut out_of_range_access = super::StructureAccessModel::default();
        out_of_range_access.set_start_for_structure("minecraft:mineshaft", out_of_range_start);

        let mut non_intersecting_access = super::StructureAccessModel::default();
        non_intersecting_access
            .set_start_for_structure("minecraft:village_plains", non_intersecting_start);

        let mut chunks = BTreeMap::new();
        chunks.insert(
            super::chunk_pos_key(ChunkPos { x: 2, z: -1 }),
            source_access,
        );
        chunks.insert(
            super::chunk_pos_key(ChunkPos { x: 9, z: 0 }),
            out_of_range_access,
        );
        chunks.insert(
            super::chunk_pos_key(ChunkPos { x: -1, z: 0 }),
            non_intersecting_access,
        );

        let references = super::structure_references_for_chunk(&chunks, ChunkPos { x: 2, z: -1 });
        assert_eq!(
            references,
            vec![super::StructureReferenceModel {
                structure: "minecraft:shipwreck",
                source_chunk_key: super::chunk_pos_key(ChunkPos { x: 2, z: -1 }),
            }]
        );

        let mut target_access = super::StructureAccessModel::default();
        let applied = super::create_structure_references_for_chunk(
            &chunks,
            ChunkPos { x: 2, z: -1 },
            &mut target_access,
        );
        assert_eq!(applied, references);
        assert_eq!(
            target_access.get_references_for_structure("minecraft:shipwreck"),
            &[super::chunk_pos_key(ChunkPos { x: 2, z: -1 })]
        );
        assert!(target_access.unsaved);
    }

    #[test]
    fn structure_access_serializes_chunk_structures_payload() {
        let start = super::StructureStartModel {
            structure: Some("minecraft:shipwreck"),
            chunk_pos: ChunkPos { x: 2, z: -1 },
            references: 1,
            pieces: vec![super::StructurePieceModel {
                bounding_box: super::StructureBoundingBoxModel {
                    min_x: 32,
                    min_y: 50,
                    min_z: -16,
                    max_x: 47,
                    max_y: 70,
                    max_z: -1,
                },
            }],
        };
        let mut access = super::StructureAccessModel::default();
        access.set_start_for_structure("minecraft:shipwreck", start);
        access.set_start_for_structure(
            "minecraft:stronghold",
            super::StructureStartModel::invalid(),
        );
        access.add_reference_for_structure(
            "minecraft:shipwreck",
            super::chunk_pos_key(ChunkPos { x: 2, z: -1 }),
        );

        let Tag::Compound(root) =
            super::structure_access_to_chunk_structures_tag(&access, ChunkPos { x: 2, z: -1 })
        else {
            panic!("structures payload should be a compound");
        };
        let Some((_, Tag::Compound(starts))) = root.iter().find(|(name, _)| name == "starts")
        else {
            panic!("starts should be a compound");
        };
        let Some((_, Tag::Compound(shipwreck))) = starts
            .iter()
            .find(|(name, _)| name == "minecraft:shipwreck")
        else {
            panic!("shipwreck start should be serialized");
        };

        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "id"),
            Some((_, Tag::String(id))) if id == "minecraft:shipwreck"
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "ChunkX"),
            Some((_, Tag::Int(2)))
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "ChunkZ"),
            Some((_, Tag::Int(-1)))
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "references"),
            Some((_, Tag::Int(1)))
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "Children"),
            Some((_, Tag::List(children))) if children.len() == 1
        ));
        assert!(matches!(
            starts.iter().find(|(name, _)| name == "minecraft:stronghold"),
            Some((_, Tag::Compound(invalid))) if matches!(
                invalid.iter().find(|(name, _)| name == "id"),
                Some((_, Tag::String(id))) if id == "INVALID"
            )
        ));

        let Some((_, Tag::Compound(references))) =
            root.iter().find(|(name, _)| name == "References")
        else {
            panic!("References should be a compound");
        };
        assert!(matches!(
            references.iter().find(|(name, _)| name == "minecraft:shipwreck"),
            Some((_, Tag::LongArray(values)))
                if values == &[super::chunk_pos_key(ChunkPos { x: 2, z: -1 })]
        ));
    }

    #[test]
    fn structure_family_coverage_matches_builtin_structure_keys() {
        assert_eq!(STRUCTURE_FAMILIES.len(), 21);
        assert_eq!(
            STRUCTURE_FAMILIES
                .iter()
                .map(|entry| entry.structures.len())
                .sum::<usize>(),
            BUILTIN_STRUCTURES.len()
        );

        let villages = STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::Village)
            .unwrap();
        assert_eq!(villages.structures.len(), 5);

        let ruined_portals = STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::RuinedPortal)
            .unwrap();
        assert_eq!(ruined_portals.structures.len(), 7);
        assert!(ruined_portals
            .structures
            .contains(&"minecraft:ruined_portal_nether"));

        assert!(STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::OceanRuins)
            .unwrap()
            .structures
            .contains(&"minecraft:ocean_ruin_warm"));
        assert!(STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::TrialChambers)
            .unwrap()
            .structures
            .contains(&"minecraft:trial_chambers"));
    }

    #[test]
    fn buried_treasure_piece_generation_and_support_scan_match_vanilla() {
        let piece = super::buried_treasure_generation_piece(ChunkPos { x: -2, z: 3 });
        assert_eq!(
            piece.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: -23,
                min_y: 90,
                min_z: 57,
                max_x: -23,
                max_y: 90,
                max_z: 57,
            }
        );
        assert!(super::buried_treasure_is_support("minecraft:sandstone"));
        assert!(super::buried_treasure_is_support("minecraft:diorite"));
        assert!(!super::buried_treasure_is_support("minecraft:sand"));
        assert_eq!(
            super::buried_treasure_soft_state("minecraft:water"),
            "minecraft:sand"
        );
        assert_eq!(
            super::buried_treasure_soft_state("minecraft:gravel"),
            "minecraft:gravel"
        );

        let placement = super::buried_treasure_place(piece, 75, 60, |pos| {
            if pos.y == 69 && pos.x == -23 && pos.z == 57 {
                "minecraft:stone"
            } else if pos.y == 70 && pos.x == -23 && pos.z == 57 {
                "minecraft:water"
            } else if pos.y == 69 {
                "minecraft:water"
            } else {
                "minecraft:air"
            }
        })
        .unwrap();
        assert_eq!(
            placement.chest_pos,
            BlockPos {
                x: -23,
                y: 70,
                z: 57,
            }
        );
        assert_eq!(placement.bounding_box.min_y, 70);
        assert_eq!(placement.side_fill[0].0, "minecraft:stone");
        assert_eq!(placement.side_fill[1].0, "minecraft:sand");
        assert_eq!(placement.side_fill[2].0, "minecraft:stone");
        assert_eq!(placement.side_fill[3].0, "minecraft:stone");
        assert_eq!(placement.side_fill[4].0, "minecraft:stone");
        assert_eq!(placement.side_fill[5].0, "minecraft:stone");

        assert!(super::buried_treasure_place(piece, 64, 60, |_| "minecraft:sand").is_none());
    }

    #[test]
    fn swamp_hut_piece_layout_height_and_entity_flags_match_vanilla() {
        let piece = super::swamp_hut_generation_piece(
            ChunkPos { x: 0, z: 0 },
            super::HorizontalDirection::South,
        );
        assert_eq!(piece.scattered.width, 7);
        assert_eq!(piece.scattered.height, 7);
        assert_eq!(piece.scattered.depth, 9);
        assert_eq!(piece.scattered.height_position, -1);
        assert_eq!(
            piece.scattered.bounding_box,
            super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 64,
                min_z: 0,
                max_x: 6,
                max_y: 70,
                max_z: 8,
            }
        );
        assert_eq!(
            super::swamp_hut_save_tag(&piece),
            super::SwampHutSaveTagModel {
                width: 7,
                height: 7,
                depth: 9,
                height_position: -1,
                witch: false,
                cat: false,
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
        let processed = super::swamp_hut_post_process(piece.clone(), chunk_bb, |x, z| {
            70 + (x == 6 && z == 8) as i32
        })
        .unwrap();
        assert_eq!(processed.piece.scattered.height_position, 70);
        assert_eq!(processed.piece.scattered.bounding_box.min_y, 70);
        assert_eq!(processed.fill_columns.len(), 4);
        assert_eq!(
            processed.entity_spawns,
            vec![
                super::SwampHutEntitySpawnModel {
                    entity: "minecraft:witch",
                    pos: BlockPos { x: 2, y: 72, z: 5 },
                },
                super::SwampHutEntitySpawnModel {
                    entity: "minecraft:cat",
                    pos: BlockPos { x: 2, y: 72, z: 5 },
                },
            ]
        );
        assert!(processed.piece.spawned_witch);
        assert!(processed.piece.spawned_cat);
        assert!(processed.blocks.iter().any(|block| {
            block.world_pos == BlockPos { x: 4, y: 72, z: 6 } && block.state == "minecraft:cauldron"
        }));
        assert!(processed.blocks.iter().any(|block| {
            block.world_pos == BlockPos { x: 0, y: 74, z: 1 }
                && block.state == "minecraft:spruce_stairs[facing=north,shape=outer_right]"
        }));

        let mut already_spawned = piece.clone();
        already_spawned.spawned_witch = true;
        already_spawned.spawned_cat = true;
        let no_spawns =
            super::swamp_hut_post_process(already_spawned, chunk_bb, |_, _| 70).unwrap();
        assert!(no_spawns.entity_spawns.is_empty());

        let outside_chunk = super::StructureBoundingBoxModel {
            min_x: 100,
            min_y: i32::MIN,
            min_z: 100,
            max_x: 115,
            max_y: i32::MAX,
            max_z: 115,
        };
        assert!(super::swamp_hut_post_process(piece, outside_chunk, |_, _| 70).is_none());
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
