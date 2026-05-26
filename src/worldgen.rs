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
#[cfg(test)] use self::biome_sampling::biome_manager_fiddle;
use self::biome_sampling::{
    biome_manager_get_biome, biome_manager_get_biome_cached, biome_manager_obfuscate_seed,
    ChunkNoiseBiomeCache,
};
pub use self::biome_sampling::{get_biome, ClimateSampler};

mod surface_rules;
pub use self::surface_rules::*;

mod terrain_models;
pub use self::terrain_models::*;

mod decoration_models;
pub use self::decoration_models::*;

mod flat_generation;
pub use self::flat_generation::*;

mod feature_sorting;
use self::feature_sorting::possible_biome_feature_steps_for_source;
pub use self::feature_sorting::{
    biome_decoration_feature_plan, build_features_per_step,
};
#[cfg(test)] use self::feature_sorting::{biome_decoration_structure_calls, build_features_per_step_with_source_ids};

mod tree_decoration_context;
use self::tree_decoration_context::*;
mod tree_decoration_heights;
use self::tree_decoration_heights::*;

mod block_state_support;
use self::block_state_support::*;

mod noise_generator_settings;
pub use self::noise_generator_settings::{
    builtin_density_function, builtin_noise_router, density_function_type,
    noise_router_id_for_settings, BUILTIN_NOISE_GENERATOR_SETTINGS, END_ISLANDS_DENSITY,
    WORLD_PRESETS,
};
#[cfg(test)] use self::noise_generator_settings::{
    cave_generation_family, EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS, OVERWORLD_SPAWN_TARGET,
    TEST_CACHE_ALL_IN_CELL_DENSITY, TEST_NEGATIVE_DENSITY, TEST_POSITIVE_DENSITY,
    TEST_RANGE_CHOICE_DENSITY,
};

mod underground_ore_decoration;
use self::underground_ore_decoration::{
    apply_underground_ore_decoration_from_source_into_region,
    apply_underground_ore_decoration_to_chunk, apply_underground_ore_decoration_to_chunk_with_context,
    clamped_map, MaterialRuleCalculationInput, NoiseMaterialRuleList,
};
#[cfg(test)] use self::underground_ore_decoration::{
    ore_vein_decision, ore_vein_decision_after_toggle, ore_vein_decision_at, ore_vein_richness,
};

mod noise_preview_chunk;
use self::noise_preview_chunk::{live_tree_decoration_blocks, LiveTreeDecorationInput};
#[cfg(test)] use self::noise_preview_chunk::{
    live_tree_count, noise_preview_ground_cover_blocks, noise_preview_terrain_height,
    noise_preview_tree_blocks, NoisePreviewTreeCountKind,
};
pub use self::noise_preview_chunk::materialize_noise_preview_chunk;

mod simple_vegetation;
use self::simple_vegetation::{
    apply_initial_simple_vegetation_decoration_to_chunk, block_predicate_test_in_region,
    configured_simple_vegetation_block, placed_simple_vegetation_feature, region_static_block_name,
    sample_triangle_int, seedless_noise_salt, simple_vegetation_phase, simple_vegetation_source_height,
    vegetation_flower_noise, PlacedSimpleVegetationFeature, SimpleVegetationDecorationInput,
    SimpleVegetationPhase,
};
#[cfg(test)] use self::simple_vegetation::{
    block_predicate_test_in_chunk, place_configured_simple_vegetation_in_target_chunk,
};

mod live_tree_selection;
#[cfg(test)] use self::live_tree_selection::{
    live_birch_tree_config, live_oak_bees_005_tree_config, live_oak_leaf_litter_tree_config,
};
use self::live_tree_selection::{
    live_fallen_tree_placement_plan, live_tree_feature_selection,
    live_tree_sapling_for_tree_config, live_tree_sapling_for_trunk_provider,
    live_tree_sapling_survives_at, live_tree_selector_trace, live_tree_state_with_planned_blocks,
    tree_placement_filter_sapling, LiveTreeDecoratorSet, LiveTreeFeatureConfig,
    LiveTreeFeatureSelection,
};

mod live_tree_placement;
#[cfg(test)] use self::live_tree_placement::tree_decorator_solid_render;
use self::live_tree_placement::{
    append_live_tree_decorators, live_tree_placement_plan, LiveTreeDecoratorInput,
    LiveTreePlacementInput,
};

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
#[cfg(test)] use self::jigsaw_model_impls::*;
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
use self::terrain_splines::*;
mod density_function_evaluation;
pub use self::density_function_evaluation::*;
mod noise_aquifer;
pub use self::noise_aquifer::*;
mod noise_sampling;
pub use self::noise_sampling::*;
mod generated_sections;
use self::generated_sections::*;
mod surface_rule_runtime;
pub use self::surface_rule_runtime::*;
mod surface_generation;
pub use self::surface_generation::*;
mod carver_generation;
pub use self::carver_generation::*;
mod carver_application;
pub use self::carver_application::*;
mod tree_decoration_generation;
use self::tree_decoration_generation::*;
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
#[cfg(test)] use self::aquatic_feature_plans::*;
mod vegetation_patch_plans;
#[cfg(test)] use self::vegetation_patch_plans::*;
mod lake_feature_plans;
#[cfg(test)] use self::lake_feature_plans::*;
mod fossil_feature_plans;
pub use self::fossil_feature_plans::*;
mod geode_feature_plans;
#[cfg(test)] use self::geode_feature_plans::*;
mod iceberg_feature_plans;
#[cfg(test)] use self::iceberg_feature_plans::*;
mod blue_ice_feature_plans;
#[cfg(test)] use self::blue_ice_feature_plans::*;
mod feature_selector_plans;
#[cfg(test)] use self::feature_selector_plans::*;
mod fill_layer_feature_plans;
#[cfg(test)] use self::fill_layer_feature_plans::*;
mod end_island_feature_plans;
#[cfg(test)] use self::end_island_feature_plans::*;
mod replace_sphere_feature_plans;
#[cfg(test)] use self::replace_sphere_feature_plans::*;
mod basalt_pillar_feature_plans;
#[cfg(test)] use self::basalt_pillar_feature_plans::*;
mod basalt_column_feature_plans;
#[cfg(test)] use self::basalt_column_feature_plans::*;
mod delta_feature_plans;
#[cfg(test)] use self::delta_feature_plans::*;
mod glowstone_feature_plans;
#[cfg(test)] use self::glowstone_feature_plans::*;
mod nether_forest_vegetation_plans;
#[cfg(test)] use self::nether_forest_vegetation_plans::*;
mod nether_vine_feature_plans;
#[cfg(test)] use self::nether_vine_feature_plans::*;
mod end_platform_feature_plans;
#[cfg(test)] use self::end_platform_feature_plans::*;
mod end_gateway_feature_plans;
#[cfg(test)] use self::end_gateway_feature_plans::*;
mod chorus_plant_feature_plans;
#[cfg(test)] use self::chorus_plant_feature_plans::*;
mod end_podium_feature_plans;
#[cfg(test)] use self::end_podium_feature_plans::*;
mod end_spike_feature_plans;
#[cfg(test)] use self::end_spike_feature_plans::*;
mod huge_fungus_feature_plans;
#[cfg(test)] use self::huge_fungus_feature_plans::*;
mod block_pile_feature_plans;
#[cfg(test)] use self::block_pile_feature_plans::*;
mod disk_feature_plans;
#[cfg(test)] use self::disk_feature_plans::*;
mod snow_and_freeze_feature_plans;
#[cfg(test)] use self::snow_and_freeze_feature_plans::*;
mod underwater_magma_feature_plans;
#[cfg(test)] use self::underwater_magma_feature_plans::*;
mod dripstone_cluster_feature_plans;
#[cfg(test)] use self::dripstone_cluster_feature_plans::*;
mod pointed_dripstone_feature_plans;
pub use self::pointed_dripstone_feature_plans::*;
mod large_dripstone_feature_plans;
#[cfg(test)] use self::large_dripstone_feature_plans::*;
mod tree_configuration_plans;
pub use self::tree_configuration_plans::*;
mod root_system_feature_plans;
#[cfg(test)] use self::root_system_feature_plans::*;
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
mod jigsaw_placement;
#[cfg(test)] use self::jigsaw_placement::*;
mod structure_template_support;
#[cfg(test)] use self::structure_template_support::*;
mod structure_access_support;
#[cfg(test)] use self::structure_access_support::*;
mod structure_scattered_features;
#[cfg(test)] pub use self::structure_scattered_features::*;
mod structure_template_features;
pub use self::structure_template_features::*;
mod structure_ocean_ruins;
#[cfg(test)] use self::structure_ocean_ruins::*;
mod structure_mineshaft_generation;
pub use self::structure_mineshaft_generation::*;
mod structure_mineshaft_application;
pub use self::structure_mineshaft_application::*;
mod structure_strongholds;
pub use self::structure_strongholds::*;
mod structure_nether_fortress;
#[cfg(test)] use self::structure_nether_fortress::*;
mod structure_ocean_monuments;
#[cfg(test)] use self::structure_ocean_monuments::*;
mod structure_end_city;
#[cfg(test)] use self::structure_end_city::*;
mod structure_woodland_mansion;
#[cfg(test)] use self::structure_woodland_mansion::*;

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
        BlockPos, BlockPredicate, BlockPredicateContext, CarverShape,
        CaveDensityOutput, CaveSurface, ConfiguredFeatureSource, DensityFunction, DensityMarker,
        FeatureConfigurationKind, FeatureFamily, FlatLayerInfo, FloatProvider,
        FluidStatus, FoliagePlacerKind, FoliagePlacerModel, GenerationDecorationStep,
        HeightProvider, HeightRange,
        MappedDensityFunction, MobSpawnerDataModel, NoiseRouterPreset, NoiseSettings,
        OreVeinDecisionInput, OreVeinifierConstants, PlacedFeatureSource, PlacementContextModel,
        PlacementModifier, RandomSpreadType, RandomStateNoiseCache, SpawnBlockKind, SpawnColumnHeights, StructureFamily,
        StructurePlacementKind, SurfaceConditionSource, SurfaceMaterialContext, SurfaceRuleKind,
        SurfaceRulePreset, SurfaceRuleSource,
        TreePlacementBlockKind, TrunkPlacerKind, TrunkPlacerModel, VerticalAnchor, WeightedHeightProvider, WorldCarverType, WorldGenerationHeightContext,
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
        TEST_NEGATIVE_DENSITY, TEST_POSITIVE_DENSITY, UPGRADE_DATA_MODEL,
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

    mod feature_placement_support_environment_tests;

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
        feature_placement_support_decorator_tests::assert_decorator_spring_and_monster_room_support(
        );
    }
}
