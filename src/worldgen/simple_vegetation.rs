use super::*;

#[derive(Debug, Clone)]
pub(super) struct PlacedSimpleVegetationFeature {
    pub(super) configured_feature: &'static str,
    pub(super) placement: Vec<PlacementModifier>,
}

pub(super) fn placed_simple_vegetation_feature(id: &str) -> Option<PlacedSimpleVegetationFeature> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    placed_grass_patch_feature(name)
        .or_else(|| placed_tall_grass_or_bush_feature(name))
        .or_else(|| placed_flower_patch_feature(name))
}

fn simple_vegetation_feature(
    configured_feature: &'static str,
    placement: Vec<PlacementModifier>,
) -> PlacedSimpleVegetationFeature {
    PlacedSimpleVegetationFeature {
        configured_feature,
        placement,
    }
}

fn simple_air_filter() -> PlacementModifier {
    PlacementModifier::BlockPredicateFilter {
        predicate: BlockPredicate::MatchingBlockTag {
            tag: "minecraft:air",
        },
    }
}

fn leaf_litter_filter() -> PlacementModifier {
    PlacementModifier::BlockPredicateFilter {
        predicate: BlockPredicate::AllOf {
            predicates: &[
                BlockPredicate::MatchingBlockTag {
                    tag: "minecraft:air",
                },
                BlockPredicate::MatchingBlocksAt {
                    offset_y: -1,
                    blocks: &["minecraft:grass_block"],
                },
            ],
        },
    }
}

fn heightmap_modifier(heightmap: HeightmapKind) -> PlacementModifier {
    PlacementModifier::Heightmap { heightmap }
}

fn random_offset_modifier(xz_spread: i32, y_spread: i32) -> PlacementModifier {
    PlacementModifier::RandomOffset {
        xz_spread,
        y_spread,
    }
}

fn count_modifier(count: i32) -> PlacementModifier {
    PlacementModifier::Count { count }
}

fn rarity_modifier(chance: i32) -> PlacementModifier {
    PlacementModifier::RarityFilter { chance }
}

fn noise_threshold_modifier(
    noise_level: f64,
    below_noise: i32,
    above_noise: i32,
) -> PlacementModifier {
    PlacementModifier::NoiseThresholdCount {
        noise_level,
        below_noise,
        above_noise,
        sampled_noise: 0.0,
    }
}

fn placed_grass_patch_feature(name: &str) -> Option<PlacedSimpleVegetationFeature> {
    let world_surface_wg = heightmap_modifier(HeightmapKind::WorldSurfaceWg);
    let feature = match name {
        "patch_grass_plain" => simple_vegetation_feature(
            "minecraft:grass",
            vec![
                noise_threshold_modifier(-0.8, 5, 10),
                PlacementModifier::InSquare,
                world_surface_wg,
                PlacementModifier::BiomeFilter,
                count_modifier(32),
                random_offset_modifier(7, 3),
                simple_air_filter(),
            ],
        ),
        "patch_grass_meadow" => simple_vegetation_feature(
            "minecraft:grass",
            vec![
                noise_threshold_modifier(-0.8, 5, 10),
                PlacementModifier::InSquare,
                world_surface_wg,
                PlacementModifier::BiomeFilter,
                count_modifier(16),
                random_offset_modifier(7, 3),
                simple_air_filter(),
            ],
        ),
        "patch_grass_forest" => grass_patch_with_initial_count(2, 32),
        "patch_grass_savanna" => grass_patch_with_initial_count(20, 32),
        "patch_grass_normal" => grass_patch_with_initial_count(5, 32),
        "patch_grass_badlands" => simple_vegetation_feature(
            "minecraft:grass",
            vec![
                PlacementModifier::InSquare,
                world_surface_wg,
                PlacementModifier::BiomeFilter,
                count_modifier(32),
                random_offset_modifier(7, 3),
                simple_air_filter(),
            ],
        ),
        "patch_leaf_litter" => simple_vegetation_feature(
            "minecraft:leaf_litter",
            vec![
                count_modifier(2),
                PlacementModifier::InSquare,
                heightmap_modifier(HeightmapKind::WorldSurface),
                PlacementModifier::BiomeFilter,
                count_modifier(32),
                random_offset_modifier(7, 3),
                leaf_litter_filter(),
            ],
        ),
        _ => return None,
    };
    Some(feature)
}

fn grass_patch_with_initial_count(
    initial_count: i32,
    placement_count: i32,
) -> PlacedSimpleVegetationFeature {
    simple_vegetation_feature(
        "minecraft:grass",
        vec![
            count_modifier(initial_count),
            PlacementModifier::InSquare,
            heightmap_modifier(HeightmapKind::WorldSurfaceWg),
            PlacementModifier::BiomeFilter,
            count_modifier(placement_count),
            random_offset_modifier(7, 3),
            simple_air_filter(),
        ],
    )
}

fn placed_tall_grass_or_bush_feature(name: &str) -> Option<PlacedSimpleVegetationFeature> {
    let heightmap = heightmap_modifier(HeightmapKind::MotionBlocking);
    let feature = match name {
        "patch_tall_grass_2" => simple_vegetation_feature(
            "minecraft:tall_grass",
            vec![
                noise_threshold_modifier(-0.8, 0, 7),
                rarity_modifier(32),
                PlacementModifier::InSquare,
                heightmap,
                PlacementModifier::BiomeFilter,
                count_modifier(96),
                random_offset_modifier(7, 3),
                simple_air_filter(),
            ],
        ),
        "patch_tall_grass" => simple_vegetation_feature(
            "minecraft:tall_grass",
            vec![
                rarity_modifier(5),
                PlacementModifier::InSquare,
                heightmap,
                PlacementModifier::BiomeFilter,
                count_modifier(96),
                random_offset_modifier(7, 3),
                simple_air_filter(),
            ],
        ),
        "patch_bush" => simple_vegetation_feature(
            "minecraft:bush",
            vec![
                rarity_modifier(4),
                PlacementModifier::InSquare,
                heightmap,
                PlacementModifier::BiomeFilter,
                count_modifier(24),
                random_offset_modifier(5, 3),
                simple_air_filter(),
            ],
        ),
        _ => return None,
    };
    Some(feature)
}

fn placed_flower_patch_feature(name: &str) -> Option<PlacedSimpleVegetationFeature> {
    let heightmap = heightmap_modifier(HeightmapKind::MotionBlocking);
    let feature = match name {
        "flower_plains" => simple_vegetation_feature(
            "minecraft:flower_plain",
            vec![
                noise_threshold_modifier(-0.8, 15, 4),
                rarity_modifier(32),
                PlacementModifier::InSquare,
                heightmap,
                PlacementModifier::BiomeFilter,
                count_modifier(64),
                random_offset_modifier(6, 2),
                simple_air_filter(),
            ],
        ),
        "flower_default" => simple_vegetation_feature(
            "minecraft:flower_default",
            vec![
                rarity_modifier(32),
                PlacementModifier::InSquare,
                heightmap,
                PlacementModifier::BiomeFilter,
            ],
        ),
        "forest_flowers" => simple_vegetation_feature(
            "minecraft:forest_flowers",
            vec![
                rarity_modifier(7),
                PlacementModifier::InSquare,
                heightmap,
                PlacementModifier::CountProvider {
                    provider: IntProviderModel::Uniform {
                        min_inclusive: -3,
                        max_inclusive: 1,
                    },
                    sampled_count: 0,
                },
                PlacementModifier::BiomeFilter,
            ],
        ),
        "patch_sunflower" => simple_vegetation_feature(
            "minecraft:sunflower",
            vec![
                rarity_modifier(3),
                PlacementModifier::InSquare,
                heightmap,
                PlacementModifier::BiomeFilter,
                count_modifier(96),
                random_offset_modifier(7, 3),
                simple_air_filter(),
            ],
        ),
        _ => return None,
    };
    Some(feature)
}

pub(super) fn configured_simple_vegetation_block(
    id: &str,
    random: &mut RandomSourceKind,
    pos: BlockPos,
) -> Option<SimpleBlockConfigurationModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    let mut random_roll = || feature_random_next_i32_bound(random, i32::MAX);
    let to_place = match name {
        "grass" => BlockStateProviderModel::Simple("minecraft:short_grass"),
        "tall_grass" => BlockStateProviderModel::Simple("minecraft:tall_grass"),
        "bush" => BlockStateProviderModel::Simple("minecraft:bush"),
        "sunflower" => BlockStateProviderModel::Simple("minecraft:sunflower"),
        "leaf_litter" => {
            let mut entries = Vec::with_capacity(12);
            for amount in 1..=3 {
                for facing in ["north", "east", "south", "west"] {
                    let state = match (amount, facing) {
                        (1, "north") => "minecraft:leaf_litter[amount=1,facing=north]",
                        (1, "east") => "minecraft:leaf_litter[amount=1,facing=east]",
                        (1, "south") => "minecraft:leaf_litter[amount=1,facing=south]",
                        (1, "west") => "minecraft:leaf_litter[amount=1,facing=west]",
                        (2, "north") => "minecraft:leaf_litter[amount=2,facing=north]",
                        (2, "east") => "minecraft:leaf_litter[amount=2,facing=east]",
                        (2, "south") => "minecraft:leaf_litter[amount=2,facing=south]",
                        (2, "west") => "minecraft:leaf_litter[amount=2,facing=west]",
                        (3, "north") => "minecraft:leaf_litter[amount=3,facing=north]",
                        (3, "east") => "minecraft:leaf_litter[amount=3,facing=east]",
                        (3, "south") => "minecraft:leaf_litter[amount=3,facing=south]",
                        (3, "west") => "minecraft:leaf_litter[amount=3,facing=west]",
                        _ => unreachable!(),
                    };
                    entries.push(WeightedBlockState { state, weight: 1 });
                }
            }
            BlockStateProviderModel::Weighted(entries)
        }
        "flower_default" => BlockStateProviderModel::Weighted(vec![
            WeightedBlockState {
                state: "minecraft:poppy",
                weight: 2,
            },
            WeightedBlockState {
                state: "minecraft:dandelion",
                weight: 1,
            },
        ]),
        "flower_plain" => {
            let provider = BlockStateProviderModel::NoiseThreshold {
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
            let noise_value = vegetation_flower_noise(pos.x, pos.z, 2345, 0.005);
            let state = block_state_provider_sample_with_noise_value(
                &provider,
                random_roll(),
                noise_value,
            )?;
            BlockStateProviderModel::Simple(state)
        }
        "forest_flowers" => {
            let states = [
                "minecraft:lilac",
                "minecraft:rose_bush",
                "minecraft:peony",
                "minecraft:lily_of_the_valley",
            ];
            BlockStateProviderModel::Simple(
                states[feature_random_next_i32_bound(random, states.len() as i32) as usize],
            )
        }
        _ => return None,
    };
    Some(SimpleBlockConfigurationModel {
        to_place,
        schedule_tick: false,
    })
}

pub(super) fn vegetation_flower_noise(world_x: i32, world_z: i32, seed: i64, scale: f64) -> f64 {
    let x = f64::from(world_x) * scale;
    let z = f64::from(world_z) * scale;
    let seed_offset = (seed as f64).sin() * 31.415_926_535_897_93;
    ((x + seed_offset).sin() * 0.55 + (z - seed_offset).cos() * 0.45).clamp(-1.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SimpleVegetationPhase {
    BeforeTrees,
    AfterTrees,
}

pub(super) fn simple_vegetation_phase(feature: &str) -> SimpleVegetationPhase {
    match feature.strip_prefix("minecraft:").unwrap_or(feature) {
        // In forest-like vanilla biome JSON these flower patches are ordered
        // before the tree placed feature in the vegetal-decoration step, so
        // they must be visible to tree validation and tree decorators.
        "forest_flowers" | "flower_forest_flowers" | "wildflowers_birch_forest" => {
            SimpleVegetationPhase::BeforeTrees
        }
        _ => SimpleVegetationPhase::AfterTrees,
    }
}

pub(super) struct SimpleVegetationDecorationInput<'a> {
    pub chunk: &'a mut LevelChunk,
    pub biome_source_model: &'a BiomeSourceModel,
    pub settings: &'a NoiseGeneratorSettings,
    pub seed: i64,
    pub decoration_region_biome_steps: &'a [&'static [&'static [&'static str]]],
    pub source_region_biome_steps: &'a DecorationBiomeStepsByChunk,
    pub target_terrain_heights: &'a TreeDecorationHeights,
    pub context_cache: &'a TreeDecorationContextCache,
    pub phase: SimpleVegetationPhase,
}

#[derive(Default)]
struct SimpleVegetationDecorationStats {
    plan_ms: u128,
    placement_ms: u128,
    calls: usize,
}

impl SimpleVegetationDecorationStats {
    fn print_debug_report(&self, total_started: Instant, feature_sort_ms: u128, placed: usize) {
        if std::env::var_os("VIBECRAFT_WORLDGEN_TREE_DEBUG").is_none() {
            return;
        }
        eprintln!(
            "[simple-vegetation-debug] total={}ms feature_sort={}ms plan={}ms placement={}ms calls={} placed={}",
            total_started.elapsed().as_millis(),
            feature_sort_ms,
            self.plan_ms,
            self.placement_ms,
            self.calls,
            placed
        );
    }
}

pub(super) fn apply_initial_simple_vegetation_decoration_to_chunk(
    mut input: SimpleVegetationDecorationInput<'_>,
) -> usize {
    let total_started = Instant::now();
    if input.settings.id != "minecraft:overworld" && input.settings.id != "minecraft:large_biomes" {
        return 0;
    }
    if input.decoration_region_biome_steps.is_empty() {
        return 0;
    }

    let Some(router) = builtin_noise_router(noise_router_id_for_settings(*input.settings))
        .map(|entry| entry.router)
    else {
        return 0;
    };
    let climate_sampler = ClimateSampler::from_noise_router(&router, input.seed, *input.settings);
    let global_biome_steps = possible_biome_feature_steps_for_source(input.biome_source_model);
    let feature_source_steps = if global_biome_steps.is_empty() {
        input.decoration_region_biome_steps
    } else {
        &global_biome_steps
    };
    let feature_sort_started = Instant::now();
    let features_per_step = match build_features_per_step(feature_source_steps, true) {
        Ok(features) => features,
        Err(_) => return 0,
    };
    let feature_sort_ms = feature_sort_started.elapsed().as_millis();

    let mut stats = SimpleVegetationDecorationStats::default();
    let placed = place_initial_simple_vegetation_sources(
        &mut input,
        &climate_sampler,
        &features_per_step,
        &mut stats,
    );
    stats.print_debug_report(total_started, feature_sort_ms, placed);
    placed
}

fn place_initial_simple_vegetation_sources(
    input: &mut SimpleVegetationDecorationInput<'_>,
    climate_sampler: &ClimateSampler,
    features_per_step: &[StepFeatureDataModel],
    stats: &mut SimpleVegetationDecorationStats,
) -> usize {
    let target_pos = input.chunk.pos;
    let mut placed = 0;
    for source_z in target_pos.z - 1..=target_pos.z + 1 {
        for source_x in target_pos.x - 1..=target_pos.x + 1 {
            let source_pos = ChunkPos {
                x: source_x,
                z: source_z,
            };
            let Some(source_terrain_heights) = (if source_pos == target_pos {
                Some(SourceTerrainHeights::Full(input.target_terrain_heights))
            } else {
                input
                    .context_cache
                    .cached_region_chunks
                    .get(&source_pos)
                    .map(SourceTerrainHeights::Lazy)
            }) else {
                continue;
            };
            let possible_steps = input
                .source_region_biome_steps
                .get(&source_pos)
                .cloned()
                .unwrap_or_else(|| input.decoration_region_biome_steps.to_vec());
            if possible_steps.is_empty() {
                continue;
            }
            let plan_started = Instant::now();
            let plan = biome_decoration_feature_plan(
                input.seed,
                source_pos.x,
                source_pos.z,
                input.settings.noise.min_y.div_euclid(16),
                features_per_step,
                &possible_steps,
            );
            stats.plan_ms += plan_started.elapsed().as_millis();
            for call in plan.feature_calls.iter().filter(|call| {
                call.step_index == GenerationDecorationStep::VegetalDecoration as usize
                    && placed_simple_vegetation_feature(call.feature).is_some()
                    && simple_vegetation_phase(call.feature) == input.phase
            }) {
                let Some(feature) = placed_simple_vegetation_feature(call.feature) else {
                    continue;
                };
                stats.calls += 1;
                let mut random = RandomSourceKind::new(call.seed, RandomAlgorithm::Xoroshiro);
                let placement_started = Instant::now();
                placed += place_simple_vegetation_feature_positions_depth_first(
                    SimpleVegetationPlacementInput {
                        target_chunk: input.chunk,
                        source_pos,
                        biome_source_model: input.biome_source_model,
                        settings: input.settings,
                        climate_sampler,
                        placed_feature_id: call.feature,
                        feature: &feature,
                        modifiers: &feature.placement,
                        position: BlockPos {
                            x: source_pos.x * 16,
                            y: input.settings.noise.min_y,
                            z: source_pos.z * 16,
                        },
                        source_terrain_heights,
                        random: &mut random,
                    },
                );
                stats.placement_ms += placement_started.elapsed().as_millis();
            }
        }
    }
    placed
}

pub(super) struct SimpleVegetationPlacementInput<'a> {
    pub target_chunk: &'a mut LevelChunk,
    pub source_pos: ChunkPos,
    pub biome_source_model: &'a BiomeSourceModel,
    pub settings: &'a NoiseGeneratorSettings,
    pub climate_sampler: &'a ClimateSampler,
    pub placed_feature_id: &'a str,
    pub feature: &'a PlacedSimpleVegetationFeature,
    pub modifiers: &'a [PlacementModifier],
    pub position: BlockPos,
    pub source_terrain_heights: SourceTerrainHeights<'a>,
    pub random: &'a mut RandomSourceKind,
}

pub(super) fn place_simple_vegetation_feature_positions_depth_first(
    input: SimpleVegetationPlacementInput<'_>,
) -> usize {
    let mut walker = SimpleVegetationPlacementWalker {
        target_chunk: input.target_chunk,
        source_pos: input.source_pos,
        biome_source_model: input.biome_source_model,
        settings: input.settings,
        climate_sampler: input.climate_sampler,
        placed_feature_id: input.placed_feature_id,
        feature: input.feature,
        source_terrain_heights: input.source_terrain_heights,
        random: input.random,
    };
    walker.place(input.modifiers, input.position)
}

struct SimpleVegetationPlacementWalker<'a> {
    target_chunk: &'a mut LevelChunk,
    source_pos: ChunkPos,
    biome_source_model: &'a BiomeSourceModel,
    settings: &'a NoiseGeneratorSettings,
    climate_sampler: &'a ClimateSampler,
    placed_feature_id: &'a str,
    feature: &'a PlacedSimpleVegetationFeature,
    source_terrain_heights: SourceTerrainHeights<'a>,
    random: &'a mut RandomSourceKind,
}

impl SimpleVegetationPlacementWalker<'_> {
    fn place(&mut self, modifiers: &[PlacementModifier], position: BlockPos) -> usize {
        let Some((modifier, remaining_modifiers)) = modifiers.split_first() else {
            return place_configured_simple_vegetation_in_target_chunk(
                self.target_chunk,
                self.settings,
                self.feature.configured_feature,
                position,
                self.random,
            );
        };

        match *modifier {
            PlacementModifier::Count { count } => {
                self.place_repeated(count.max(0), remaining_modifiers, position)
            }
            PlacementModifier::CountProvider { provider, .. } => {
                let count = sample_int_provider(provider, self.random).clamp(0, i32::MAX);
                self.place_repeated(count, remaining_modifiers, position)
            }
            PlacementModifier::NoiseThresholdCount {
                noise_level,
                below_noise,
                above_noise,
                ..
            } => self.place_noise_threshold_count(
                noise_level,
                below_noise,
                above_noise,
                remaining_modifiers,
                position,
            ),
            PlacementModifier::RarityFilter { chance } => {
                self.place_rarity_filter(chance, remaining_modifiers, position)
            }
            PlacementModifier::InSquare => self.place_in_square(remaining_modifiers, position),
            PlacementModifier::Heightmap { heightmap } => {
                self.place_heightmap(heightmap, remaining_modifiers, position)
            }
            PlacementModifier::RandomOffset {
                xz_spread,
                y_spread,
            } => self.place_random_offset(xz_spread, y_spread, remaining_modifiers, position),
            PlacementModifier::BlockPredicateFilter { predicate } => {
                self.place_block_predicate(predicate, remaining_modifiers, position)
            }
            PlacementModifier::BiomeFilter => {
                self.place_biome_filter(remaining_modifiers, position)
            }
            _ => 0,
        }
    }

    fn place_repeated(
        &mut self,
        count: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let mut placed = 0;
        for _ in 0..count {
            placed += self.place(remaining_modifiers, position);
        }
        placed
    }

    fn place_noise_threshold_count(
        &mut self,
        noise_level: f64,
        below_noise: i32,
        above_noise: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let noise = vegetation_flower_noise(
            position.x,
            position.z,
            seedless_noise_salt(self.placed_feature_id),
            0.005,
        );
        let count = if noise < noise_level {
            below_noise
        } else {
            above_noise
        };
        self.place_repeated(count.max(0), remaining_modifiers, position)
    }

    fn place_rarity_filter(
        &mut self,
        chance: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        if chance > 0 && feature_random_next_i32_bound(self.random, chance) == 0 {
            self.place(remaining_modifiers, position)
        } else {
            0
        }
    }

    fn place_in_square(
        &mut self,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let x_offset = feature_random_next_i32_bound(self.random, 16);
        let z_offset = feature_random_next_i32_bound(self.random, 16);
        self.place(
            remaining_modifiers,
            BlockPos {
                x: position.x + x_offset,
                y: position.y,
                z: position.z + z_offset,
            },
        )
    }

    fn place_heightmap(
        &mut self,
        heightmap: HeightmapKind,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let y = simple_vegetation_source_height(
            self.source_pos,
            self.source_terrain_heights,
            heightmap,
            position.x,
            position.z,
            self.settings,
        );
        if y <= self.settings.noise.min_y {
            0
        } else {
            self.place(remaining_modifiers, BlockPos { y, ..position })
        }
    }

    fn place_random_offset(
        &mut self,
        xz_spread: i32,
        y_spread: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let x_offset = sample_triangle_int(self.random, xz_spread);
        let y_offset = sample_triangle_int(self.random, y_spread);
        let z_offset = sample_triangle_int(self.random, xz_spread);
        self.place(
            remaining_modifiers,
            BlockPos {
                x: position.x + x_offset,
                y: position.y + y_offset,
                z: position.z + z_offset,
            },
        )
    }

    fn place_block_predicate(
        &mut self,
        predicate: BlockPredicate,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        if block_predicate_test_in_chunk(self.target_chunk, self.settings, predicate, position) {
            self.place(remaining_modifiers, position)
        } else {
            0
        }
    }

    fn place_biome_filter(
        &mut self,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        if biome_allows_feature_at(
            self.biome_source_model,
            self.settings,
            self.climate_sampler,
            position,
            self.placed_feature_id,
        ) {
            self.place(remaining_modifiers, position)
        } else {
            0
        }
    }
}

fn block_predicate_context_for_chunk_pos(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
    position: BlockPos,
) -> BlockPredicateContext {
    let block = chunk
        .get_block_state_name(position.x, position.y, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    block_predicate_context_for_state(block, settings.noise.min_y, settings.noise.height)
}

pub(super) fn block_predicate_test_in_chunk(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
    predicate: BlockPredicate,
    position: BlockPos,
) -> bool {
    let context = block_predicate_context_for_chunk_pos(chunk, settings, position);
    match predicate {
        BlockPredicate::MatchingBlocks { blocks } => blocks.contains(&context.block),
        BlockPredicate::MatchingBlocksAt { offset_y, blocks } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context = block_predicate_context_for_chunk_pos(chunk, settings, offset_pos);
            blocks.contains(&offset_context.block)
        }
        BlockPredicate::MatchingBlockTag { tag } => block_matches_tag(context.block, tag),
        BlockPredicate::MatchingFluids { fluids } => fluids.contains(&context.fluid),
        BlockPredicate::MatchingFluidsAt { offset_y, fluids } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context = block_predicate_context_for_chunk_pos(chunk, settings, offset_pos);
            fluids.contains(&offset_context.fluid)
        }
        BlockPredicate::Solid => context.solid,
        BlockPredicate::SolidAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_chunk_pos(chunk, settings, offset_pos).solid
        }
        BlockPredicate::Replaceable => context.replaceable,
        BlockPredicate::ReplaceableAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_chunk_pos(chunk, settings, offset_pos).replaceable
        }
        BlockPredicate::WouldSurvive {
            offset_y,
            state: _,
            survives,
        }
        | BlockPredicate::HasSturdyFace {
            offset_y,
            direction: _,
            sturdy: survives,
        } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y
                && y < settings.noise.min_y + settings.noise.height
                && survives
        }
        BlockPredicate::InsideWorldBounds { offset_y } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y && y < settings.noise.min_y + settings.noise.height
        }
        BlockPredicate::AnyOf { predicates } => predicates
            .iter()
            .any(|predicate| block_predicate_test_in_chunk(chunk, settings, *predicate, position)),
        BlockPredicate::AllOf { predicates } => predicates
            .iter()
            .all(|predicate| block_predicate_test_in_chunk(chunk, settings, *predicate, position)),
        BlockPredicate::Not { predicate } => {
            !block_predicate_test_in_chunk(chunk, settings, *predicate, position)
        }
        BlockPredicate::True => true,
        BlockPredicate::Unobstructed => context.unobstructed,
    }
}

pub(super) fn region_static_block_name(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    position: BlockPos,
) -> Option<&'static str> {
    let pos = ChunkPos {
        x: position.x.div_euclid(16),
        z: position.z.div_euclid(16),
    };
    chunks
        .get(&pos)?
        .get_block_state_name(position.x, position.y, position.z)
        .and_then(carver_static_block_name)
}

fn block_predicate_context_for_region_pos(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    settings: &NoiseGeneratorSettings,
    position: BlockPos,
) -> BlockPredicateContext {
    let block = region_static_block_name(chunks, position).unwrap_or("minecraft:air");
    block_predicate_context_for_state(block, settings.noise.min_y, settings.noise.height)
}

pub(super) fn block_predicate_test_in_region(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    settings: &NoiseGeneratorSettings,
    predicate: BlockPredicate,
    position: BlockPos,
) -> bool {
    let context = block_predicate_context_for_region_pos(chunks, settings, position);
    match predicate {
        BlockPredicate::MatchingBlocks { blocks } => blocks.contains(&context.block),
        BlockPredicate::MatchingBlocksAt { offset_y, blocks } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context =
                block_predicate_context_for_region_pos(chunks, settings, offset_pos);
            blocks.contains(&offset_context.block)
        }
        BlockPredicate::MatchingBlockTag { tag } => block_matches_tag(context.block, tag),
        BlockPredicate::MatchingFluids { fluids } => fluids.contains(&context.fluid),
        BlockPredicate::MatchingFluidsAt { offset_y, fluids } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context =
                block_predicate_context_for_region_pos(chunks, settings, offset_pos);
            fluids.contains(&offset_context.fluid)
        }
        BlockPredicate::Solid => context.solid,
        BlockPredicate::SolidAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_region_pos(chunks, settings, offset_pos).solid
        }
        BlockPredicate::Replaceable => context.replaceable,
        BlockPredicate::ReplaceableAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_region_pos(chunks, settings, offset_pos).replaceable
        }
        BlockPredicate::WouldSurvive {
            offset_y,
            state: _,
            survives,
        }
        | BlockPredicate::HasSturdyFace {
            offset_y,
            direction: _,
            sturdy: survives,
        } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y
                && y < settings.noise.min_y + settings.noise.height
                && survives
        }
        BlockPredicate::InsideWorldBounds { offset_y } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y && y < settings.noise.min_y + settings.noise.height
        }
        BlockPredicate::AnyOf { predicates } => predicates.iter().any(|predicate| {
            block_predicate_test_in_region(chunks, settings, *predicate, position)
        }),
        BlockPredicate::AllOf { predicates } => predicates.iter().all(|predicate| {
            block_predicate_test_in_region(chunks, settings, *predicate, position)
        }),
        BlockPredicate::Not { predicate } => {
            !block_predicate_test_in_region(chunks, settings, *predicate, position)
        }
        BlockPredicate::True => true,
        BlockPredicate::Unobstructed => context.unobstructed,
    }
}

pub(super) fn seedless_noise_salt(id: &str) -> i64 {
    id.bytes().fold(0_i64, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(i64::from(byte))
    })
}

pub(super) fn simple_vegetation_source_height(
    source_pos: ChunkPos,
    source_terrain_heights: SourceTerrainHeights<'_>,
    heightmap: HeightmapKind,
    world_x: i32,
    world_z: i32,
    settings: &NoiseGeneratorSettings,
) -> i32 {
    source_terrain_heights.world_height(
        source_pos,
        heightmap,
        world_x,
        world_z,
        settings.noise.min_y,
    )
}

pub(super) fn place_configured_simple_vegetation_in_target_chunk(
    chunk: &mut LevelChunk,
    settings: &NoiseGeneratorSettings,
    configured_feature: &'static str,
    position: BlockPos,
    random: &mut RandomSourceKind,
) -> usize {
    if position.x.div_euclid(16) != chunk.pos.x || position.z.div_euclid(16) != chunk.pos.z {
        return 0;
    }
    if !(settings.noise.min_y..settings.noise.min_y + settings.noise.height).contains(&position.y) {
        return 0;
    }
    let current = chunk
        .get_block_state_name(position.x, position.y, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    let below = chunk
        .get_block_state_name(position.x, position.y - 1, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    let above = chunk
        .get_block_state_name(position.x, position.y + 1, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    let Some(config) = configured_simple_vegetation_block(configured_feature, random, position)
    else {
        return 0;
    };
    let Some(plan) = simple_block_placement_plan(
        &config,
        SimpleBlockPlacementContext {
            origin_block: current,
            below_block: below,
            above_block: above,
        },
        random,
    ) else {
        return 0;
    };

    chunk.set_block_state(position.x, position.y, position.z, plan.state);
    let mut placed = 1;
    if let Some(upper_state) = plan.upper_state {
        if position.y + 1 < settings.noise.min_y + settings.noise.height {
            chunk.set_block_state(position.x, position.y + 1, position.z, upper_state);
            placed += 1;
        }
    }
    placed
}

pub(super) fn sample_triangle_int(random: &mut RandomSourceKind, range: i32) -> i32 {
    feature_random_next_i32_bound(random, range + 1)
        - feature_random_next_i32_bound(random, range + 1)
}
