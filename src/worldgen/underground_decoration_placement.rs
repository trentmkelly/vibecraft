use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct OrePlacementReport {
    pub(super) placed: usize,
    pub(super) configured_calls: usize,
    pub(super) total_us: u128,
    pub(super) candidate_count: usize,
    pub(super) in_chunk_candidates: usize,
    pub(super) origin_us: u128,
    pub(super) candidate_us: u128,
    pub(super) block_us: u128,
}

impl std::ops::AddAssign for OrePlacementReport {
    fn add_assign(&mut self, rhs: Self) {
        self.placed += rhs.placed;
        self.configured_calls += rhs.configured_calls;
        self.total_us += rhs.total_us;
        self.candidate_count += rhs.candidate_count;
        self.in_chunk_candidates += rhs.in_chunk_candidates;
        self.origin_us += rhs.origin_us;
        self.candidate_us += rhs.candidate_us;
        self.block_us += rhs.block_us;
    }
}

pub(super) fn place_ore_feature_in_chunk(
    chunk: &LevelChunk,
    block_cache: &mut OreBlockCache,
    source_pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    climate_sampler: &ClimateSampler,
    placed_feature_id: &'static str,
    feature: &PlacedOreFeatureModel,
    config: &OreConfigurationModel,
    feature_seed_value: i64,
    skip_biome_filter: bool,
) -> OrePlacementReport {
    let context = WorldGenerationHeightContext {
        min_y: settings.noise.min_y,
        height: settings.noise.height,
    };
    let mut random = RandomSourceKind::new(feature_seed_value, RandomAlgorithm::Xoroshiro);
    let origin = BlockPos {
        x: source_pos.x * 16,
        y: settings.noise.min_y,
        z: source_pos.z * 16,
    };
    if let Some(report) = place_vanilla_ore_feature_fast(
        chunk,
        block_cache,
        biome_source_model,
        settings,
        climate_sampler,
        placed_feature_id,
        &feature.placement,
        context,
        config,
        origin,
        &mut random,
        skip_biome_filter,
    ) {
        let _ = seed;
        return report;
    }
    let report = place_ore_feature_positions_depth_first(
        chunk,
        block_cache,
        biome_source_model,
        settings,
        climate_sampler,
        placed_feature_id,
        &feature.placement,
        context,
        config,
        origin,
        &mut random,
        skip_biome_filter,
    );
    let _ = seed;
    report
}

#[allow(clippy::too_many_arguments)]
fn place_vanilla_ore_feature_fast(
    chunk: &LevelChunk,
    block_cache: &mut OreBlockCache,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    placed_feature_id: &str,
    modifiers: &[PlacementModifier],
    context: WorldGenerationHeightContext,
    config: &OreConfigurationModel,
    origin: BlockPos,
    random: &mut RandomSourceKind,
    skip_biome_filter: bool,
) -> Option<OrePlacementReport> {
    let [frequency, PlacementModifier::InSquare, PlacementModifier::HeightRange { height }, PlacementModifier::BiomeFilter] =
        modifiers
    else {
        return None;
    };
    let count = match *frequency {
        PlacementModifier::Count { count } => count.max(0),
        PlacementModifier::CountProvider { provider, .. } => {
            sample_int_provider(provider, random).max(0)
        }
        PlacementModifier::RarityFilter { chance } => {
            if chance > 0 && feature_random_next_f32(random) < 1.0 / chance as f32 {
                1
            } else {
                0
            }
        }
        _ => return None,
    };

    let mut report = OrePlacementReport::default();
    for _ in 0..count {
        let x = origin.x + feature_random_next_i32_bound(random, 16);
        let z = origin.z + feature_random_next_i32_bound(random, 16);
        let position = BlockPos {
            x,
            y: height_provider_sample_with_random(*height, context, random),
            z,
        };
        if !skip_biome_filter
            && !biome_allows_feature_at(
                biome_source_model,
                settings,
                climate_sampler,
                position,
                placed_feature_id,
            )
        {
            continue;
        }
        report +=
            place_configured_ore_in_chunk(chunk, block_cache, settings, config, position, random);
    }
    Some(report)
}

fn place_ore_feature_positions_depth_first(
    chunk: &LevelChunk,
    block_cache: &mut OreBlockCache,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    placed_feature_id: &str,
    modifiers: &[PlacementModifier],
    context: WorldGenerationHeightContext,
    config: &OreConfigurationModel,
    position: BlockPos,
    random: &mut RandomSourceKind,
    skip_biome_filter: bool,
) -> OrePlacementReport {
    let Some((modifier, remaining_modifiers)) = modifiers.split_first() else {
        return place_configured_ore_in_chunk(
            chunk,
            block_cache,
            settings,
            config,
            position,
            random,
        );
    };

    match *modifier {
        PlacementModifier::Count { count } => {
            let mut report = OrePlacementReport::default();
            for _ in 0..count.max(0) {
                report += place_ore_feature_positions_depth_first(
                    chunk,
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                );
            }
            report
        }
        PlacementModifier::CountProvider { provider, .. } => {
            let count = sample_int_provider(provider, random);
            let mut report = OrePlacementReport::default();
            for _ in 0..count.max(0) {
                report += place_ore_feature_positions_depth_first(
                    chunk,
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                );
            }
            report
        }
        PlacementModifier::RarityFilter { chance } => {
            if chance > 0 && feature_random_next_f32(random) < 1.0 / chance as f32 {
                place_ore_feature_positions_depth_first(
                    chunk,
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                )
            } else {
                OrePlacementReport::default()
            }
        }
        PlacementModifier::InSquare => place_ore_feature_positions_depth_first(
            chunk,
            block_cache,
            biome_source_model,
            settings,
            climate_sampler,
            placed_feature_id,
            remaining_modifiers,
            context,
            config,
            BlockPos {
                x: position.x + feature_random_next_i32_bound(random, 16),
                y: position.y,
                z: position.z + feature_random_next_i32_bound(random, 16),
            },
            random,
            skip_biome_filter,
        ),
        PlacementModifier::HeightRange { height } => place_ore_feature_positions_depth_first(
            chunk,
            block_cache,
            biome_source_model,
            settings,
            climate_sampler,
            placed_feature_id,
            remaining_modifiers,
            context,
            config,
            BlockPos {
                x: position.x,
                y: height_provider_sample_with_random(height, context, random),
                z: position.z,
            },
            random,
            skip_biome_filter,
        ),
        PlacementModifier::BiomeFilter => {
            if skip_biome_filter
                || biome_allows_feature_at(
                    biome_source_model,
                    settings,
                    climate_sampler,
                    position,
                    placed_feature_id,
                )
            {
                place_ore_feature_positions_depth_first(
                    chunk,
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                )
            } else {
                OrePlacementReport::default()
            }
        }
        _ => {
            let positions = placement_modifier_positions_with_context(
                *modifier,
                position,
                PlacementContextModel {
                    min_y: settings.noise.min_y,
                    world_surface_height: chunk
                        .heightmap_value(
                            HeightmapKind::WorldSurface,
                            position.x.rem_euclid(16) as usize,
                            position.z.rem_euclid(16) as usize,
                        )
                        .unwrap_or(settings.sea_level + 1),
                    ocean_floor_height: chunk
                        .heightmap_value(
                            HeightmapKind::OceanFloor,
                            position.x.rem_euclid(16) as usize,
                            position.z.rem_euclid(16) as usize,
                        )
                        .unwrap_or(settings.sea_level + 1),
                    biome_allows_feature: true,
                    block_predicate: BlockPredicateContext {
                        block: "minecraft:air",
                        fluid: "minecraft:empty",
                        solid: false,
                        replaceable: true,
                        unobstructed: true,
                        min_y: settings.noise.min_y,
                        height: settings.noise.height,
                    },
                },
                feature_random_next_i32_bound(random, i32::MAX),
                feature_random_next_i32_bound(random, i32::MAX),
                feature_random_next_i32_bound(random, i32::MAX),
            );
            positions.into_iter().fold(
                OrePlacementReport::default(),
                |mut report, next_position| {
                    report += place_ore_feature_positions_depth_first(
                        chunk,
                        block_cache,
                        biome_source_model,
                        settings,
                        climate_sampler,
                        placed_feature_id,
                        remaining_modifiers,
                        context,
                        config,
                        next_position,
                        random,
                        skip_biome_filter,
                    );
                    report
                },
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn place_disk_feature_in_chunk(
    block_cache: &mut OreBlockCache,
    source_pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    placed_feature_id: &str,
    feature: &PlacedDiskFeatureModel,
    config: &DiskConfigurationModel,
    feature_seed_value: i64,
    skip_biome_filter: bool,
) -> usize {
    let context = WorldGenerationHeightContext {
        min_y: settings.noise.min_y,
        height: settings.noise.height,
    };
    let mut random = RandomSourceKind::new(feature_seed_value, RandomAlgorithm::Xoroshiro);
    let origin = BlockPos {
        x: source_pos.x * 16,
        y: settings.noise.min_y,
        z: source_pos.z * 16,
    };
    place_disk_feature_positions_depth_first(
        block_cache,
        biome_source_model,
        settings,
        climate_sampler,
        placed_feature_id,
        &feature.placement,
        context,
        config,
        origin,
        &mut random,
        skip_biome_filter,
    )
}

#[allow(clippy::too_many_arguments)]
fn place_disk_feature_positions_depth_first(
    block_cache: &mut OreBlockCache,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    placed_feature_id: &str,
    modifiers: &[PlacementModifier],
    context: WorldGenerationHeightContext,
    config: &DiskConfigurationModel,
    position: BlockPos,
    random: &mut RandomSourceKind,
    skip_biome_filter: bool,
) -> usize {
    let Some((modifier, remaining_modifiers)) = modifiers.split_first() else {
        return place_configured_disk_in_chunk(block_cache, settings, config, position, random);
    };

    match *modifier {
        PlacementModifier::Count { count } => {
            let mut placed = 0;
            for _ in 0..count.max(0) {
                placed += place_disk_feature_positions_depth_first(
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                );
            }
            placed
        }
        PlacementModifier::CountProvider { provider, .. } => {
            let mut placed = 0;
            for _ in 0..sample_int_provider(provider, random).max(0) {
                placed += place_disk_feature_positions_depth_first(
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                );
            }
            placed
        }
        PlacementModifier::RarityFilter { chance } => {
            if chance > 0 && feature_random_next_f32(random) < 1.0 / chance as f32 {
                place_disk_feature_positions_depth_first(
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                )
            } else {
                0
            }
        }
        PlacementModifier::InSquare => place_disk_feature_positions_depth_first(
            block_cache,
            biome_source_model,
            settings,
            climate_sampler,
            placed_feature_id,
            remaining_modifiers,
            context,
            config,
            BlockPos {
                x: position.x + feature_random_next_i32_bound(random, 16),
                y: position.y,
                z: position.z + feature_random_next_i32_bound(random, 16),
            },
            random,
            skip_biome_filter,
        ),
        PlacementModifier::Heightmap { heightmap } => {
            let x = position.x;
            let z = position.z;
            let y = block_cache.heightmap_value_by_scan(heightmap, x, z);
            if y <= settings.noise.min_y {
                0
            } else {
                place_disk_feature_positions_depth_first(
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    BlockPos { x, y, z },
                    random,
                    skip_biome_filter,
                )
            }
        }
        PlacementModifier::HeightRange { height } => place_disk_feature_positions_depth_first(
            block_cache,
            biome_source_model,
            settings,
            climate_sampler,
            placed_feature_id,
            remaining_modifiers,
            context,
            config,
            BlockPos {
                x: position.x,
                y: height_provider_sample_with_random(height, context, random),
                z: position.z,
            },
            random,
            skip_biome_filter,
        ),
        PlacementModifier::BlockPredicateFilter { predicate } => {
            let block = block_cache
                .block_state_name(position.x, position.y, position.z)
                .and_then(carver_static_block_name)
                .unwrap_or("minecraft:air");
            let predicate_context = block_predicate_context_for_state(
                block,
                settings.noise.min_y,
                settings.noise.height,
            );
            if block_predicate_test(predicate, predicate_context, position.y) {
                place_disk_feature_positions_depth_first(
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                )
            } else {
                0
            }
        }
        PlacementModifier::BiomeFilter => {
            if skip_biome_filter
                || biome_allows_feature_at(
                    biome_source_model,
                    settings,
                    climate_sampler,
                    position,
                    placed_feature_id,
                )
            {
                place_disk_feature_positions_depth_first(
                    block_cache,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    remaining_modifiers,
                    context,
                    config,
                    position,
                    random,
                    skip_biome_filter,
                )
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn place_configured_disk_in_chunk(
    block_cache: &mut OreBlockCache,
    settings: &NoiseGeneratorSettings,
    config: &DiskConfigurationModel,
    origin: BlockPos,
    random: &mut RandomSourceKind,
) -> usize {
    let radius = sample_int_provider(config.radius, random).clamp(0, 8);
    let half_height = config.half_height.clamp(0, 4);
    let top = origin.y + half_height;
    let bottom_exclusive = origin.y - half_height - 1;
    let mut placed = 0;
    for z in origin.z - radius..=origin.z + radius {
        for x in origin.x - radius..=origin.x + radius {
            let dx = x - origin.x;
            let dz = z - origin.z;
            if dx * dx + dz * dz > radius * radius {
                continue;
            }
            for y in (bottom_exclusive + 1..=top).rev() {
                let Some(current) = block_cache
                    .block_state_name(x, y, z)
                    .and_then(carver_static_block_name)
                else {
                    continue;
                };
                let context = block_predicate_context_for_state(
                    current,
                    settings.noise.min_y,
                    settings.noise.height,
                );
                if !block_predicate_test(config.target, context, y) {
                    continue;
                }
                let below = block_cache
                    .block_state_name(x, y - 1, z)
                    .and_then(carver_static_block_name)
                    .unwrap_or("minecraft:air");
                let provider_context = block_predicate_context_for_state(
                    below,
                    settings.noise.min_y,
                    settings.noise.height,
                );
                let Some(state) = block_state_provider_sample_in_context_with_random(
                    &config.state_provider,
                    random,
                    provider_context,
                    y,
                    current,
                ) else {
                    continue;
                };
                block_cache.set_block_state(x, y, z, state);
                placed += 1;
            }
        }
    }
    placed
}

pub(super) fn block_predicate_context_for_state(
    block: &'static str,
    min_y: i32,
    height: i32,
) -> BlockPredicateContext {
    BlockPredicateContext {
        min_y,
        height,
        block,
        fluid: if block_has_fluid(block) {
            block_state_id(block)
        } else {
            "minecraft:empty"
        },
        solid: block_blocks_motion(block),
        replaceable: matches!(
            block_state_id(block),
            "minecraft:air"
                | "minecraft:cave_air"
                | "minecraft:void_air"
                | "minecraft:water"
                | "minecraft:lava"
                | "minecraft:short_grass"
                | "minecraft:fern"
                | "minecraft:tall_grass"
                | "minecraft:large_fern"
        ),
        unobstructed: matches!(
            block_state_id(block),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water"
        ),
    }
}

pub(super) fn sample_int_provider(
    provider: IntProviderModel,
    random: &mut RandomSourceKind,
) -> i32 {
    match provider {
        IntProviderModel::Constant(value) => value,
        IntProviderModel::Uniform {
            min_inclusive,
            max_inclusive,
        } => {
            if max_inclusive <= min_inclusive {
                min_inclusive
            } else {
                min_inclusive
                    + feature_random_next_i32_bound(random, max_inclusive - min_inclusive + 1)
            }
        }
    }
}

pub(super) fn sample_int_provider_from_roll(provider: IntProviderModel, roll: i32) -> i32 {
    match provider {
        IntProviderModel::Constant(value) => value,
        IntProviderModel::Uniform {
            min_inclusive,
            max_inclusive,
        } => {
            if max_inclusive <= min_inclusive {
                min_inclusive
            } else {
                min_inclusive + roll.rem_euclid(max_inclusive - min_inclusive + 1)
            }
        }
    }
}

pub(super) fn height_provider_sample_with_random(
    provider: HeightProvider,
    context: WorldGenerationHeightContext,
    random: &mut RandomSourceKind,
) -> i32 {
    match provider {
        HeightProvider::Constant { value } => value.resolve_y(context),
        HeightProvider::Uniform {
            min_inclusive,
            max_inclusive,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if min > max {
                min
            } else {
                random_next_i32_between_inclusive(random, min, max)
            }
        }
        HeightProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            let outer_bound = max - min - inner + 1;
            if outer_bound <= 0 {
                min
            } else {
                let limit = feature_random_next_i32_bound(random, outer_bound);
                min + feature_random_next_i32_bound(random, limit + inner)
            }
        }
        HeightProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if max - min < inner {
                min
            } else {
                let upper = random_next_i32_between_inclusive(random, min + inner, max);
                let biased_upper = random_next_i32_between_inclusive(random, min, upper - 1);
                random_next_i32_between_inclusive(random, min, biased_upper - 1 + inner)
            }
        }
        HeightProvider::Trapezoid {
            min_inclusive,
            max_inclusive,
            plateau,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if min > max {
                return min;
            }

            let range = max - min;
            if plateau >= range {
                return random_next_i32_between_inclusive(random, min, max);
            }

            let plateau_start = (range - plateau) / 2;
            let plateau_end = range - plateau_start;
            min + random_next_i32_between_inclusive(random, 0, plateau_end)
                + random_next_i32_between_inclusive(random, 0, plateau_start)
        }
        HeightProvider::WeightedList { distribution } => {
            let positive_weight_total = distribution
                .iter()
                .map(|entry| entry.weight.max(0))
                .sum::<i32>();
            if positive_weight_total <= 0 {
                return context.min_y;
            }

            let mut choice = feature_random_next_i32_bound(random, positive_weight_total);
            let Some(selected) = distribution.iter().find(|entry| {
                let weight = entry.weight.max(0);
                if choice < weight {
                    true
                } else {
                    choice -= weight;
                    false
                }
            }) else {
                return context.min_y;
            };
            height_provider_sample_with_random(selected.provider, context, random)
        }
    }
}

fn random_next_i32_between_inclusive(
    random: &mut RandomSourceKind,
    min_inclusive: i32,
    max_inclusive: i32,
) -> i32 {
    debug_assert!(min_inclusive <= max_inclusive);
    min_inclusive + feature_random_next_i32_bound(random, max_inclusive - min_inclusive + 1)
}

pub(super) fn biome_allows_feature_at(
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    pos: BlockPos,
    feature: &str,
) -> bool {
    // Java BiomeFilter uses PlacementContext.getLevel().getBiome(origin).
    // During feature generation the level is WorldGenRegion, whose getBiome
    // path goes through BiomeManager with the world's obfuscated biome seed.
    biome_manager_get_biome(
        biome_source_model,
        climate_sampler.biome_zoom_seed,
        pos.x,
        pos.y.clamp(
            settings.noise.min_y,
            settings.noise.min_y + settings.noise.height - 1,
        ),
        pos.z,
        climate_sampler,
    )
    .and_then(biome_generation_settings)
    .is_some_and(|generation| biome_has_placed_feature(generation, feature))
}

fn place_configured_ore_in_chunk(
    _chunk: &LevelChunk,
    block_cache: &mut OreBlockCache,
    settings: &NoiseGeneratorSettings,
    config: &OreConfigurationModel,
    origin: BlockPos,
    random: &mut RandomSourceKind,
) -> OrePlacementReport {
    let total_started = Instant::now();
    let direction = feature_random_next_f32(random);
    let spread_xy = config.size as f32 / 8.0;
    let max_radius = ((config.size as f32 / 16.0 * 2.0 + 1.0) / 2.0).ceil() as i32;
    let spread_ceil = spread_xy.ceil() as i32;
    let x_start = origin.x - spread_ceil - max_radius;
    let y_start = origin.y - 2 - max_radius;
    let z_start = origin.z - spread_ceil - max_radius;
    let size_xz = 2 * (spread_ceil + max_radius);
    let size_y = 2 * (2 + max_radius);
    let y_rolls = [(
        feature_random_next_i32_bound(random, 3),
        feature_random_next_i32_bound(random, 3),
    )];

    let started = Instant::now();
    if !ore_origin_overlaps_ocean_floor_wg(block_cache, x_start, y_start, z_start, size_xz) {
        return OrePlacementReport {
            configured_calls: 1,
            total_us: total_started.elapsed().as_micros(),
            origin_us: started.elapsed().as_micros(),
            ..OrePlacementReport::default()
        };
    }
    let origin_us = started.elapsed().as_micros();

    let started = Instant::now();
    let radius_rolls = (0..config.size.max(0))
        .map(|_| feature_random_next_f64(random))
        .collect::<Vec<_>>();

    let spheres = ore_vein_spheres(origin, config.size, direction, &y_rolls, &radius_rolls);
    let candidates = ore_vein_position_candidates(
        &spheres,
        x_start,
        y_start,
        z_start,
        size_xz,
        size_y,
        settings.noise.min_y..settings.noise.min_y + settings.noise.height,
    );
    let candidate_us = started.elapsed().as_micros();
    let candidate_count = candidates.len();

    let mut placed = 0;
    let mut in_chunk_candidates = 0;
    let block_started = Instant::now();
    for pos in candidates {
        let can_write = block_cache.can_write_world_xz(pos.x, pos.z);
        if can_write {
            in_chunk_candidates += 1;
        }
        let Some(current) = block_cache.block_state_name(pos.x, pos.y, pos.z) else {
            continue;
        };
        if !can_write {
            // Java runs the whole ore feature in a WorldGenRegion. Even when
            // this chunk-local path drops the write, buried ores must still
            // consume their air-exposure roll so later origins keep parity.
            if config
                .target_states
                .iter()
                .any(|target| rule_test_matches(target.target, current))
                && config.discard_chance_on_air_exposure > 0.0
                && config.discard_chance_on_air_exposure < 1.0
            {
                let _ = feature_random_next_f32(random);
            }
            continue;
        }
        let Some(new_block) = config.target_states.iter().find_map(|target| {
            if !rule_test_matches(target.target, current) {
                return None;
            }
            let skip_air_check = match config.discard_chance_on_air_exposure {
                chance if chance <= 0.0 => true,
                chance if chance >= 1.0 => false,
                chance => feature_random_next_f32(random) >= chance,
            };
            (skip_air_check || !is_adjacent_to_ore_air(block_cache, pos)).then_some(target.state)
        }) else {
            continue;
        };
        block_cache.set_block_state(pos.x, pos.y, pos.z, new_block);
        placed += 1;
    }
    let block_us = block_started.elapsed().as_micros();
    log_ore_detail_if_enabled(OreDetailDebug {
        total_started,
        candidate_us,
        block_us,
        candidate_count,
        in_chunk_candidates,
        placed,
        size: config.size,
    });

    OrePlacementReport {
        placed,
        configured_calls: 1,
        total_us: total_started.elapsed().as_micros(),
        candidate_count,
        in_chunk_candidates,
        origin_us,
        candidate_us,
        block_us,
    }
}

struct OreDetailDebug {
    total_started: Instant,
    candidate_us: u128,
    block_us: u128,
    candidate_count: usize,
    in_chunk_candidates: usize,
    placed: usize,
    size: i32,
}

fn log_ore_detail_if_enabled(debug: OreDetailDebug) {
    if std::env::var_os("RUSTCRAFT_WORLDGEN_ORE_DETAIL_DEBUG").is_none() {
        return;
    }
    eprintln!(
        "[ore-detail] total={}ms candidates={}us block={}us candidate_count={} in_chunk={} placed={} size={}",
        debug.total_started.elapsed().as_millis(),
        debug.candidate_us,
        debug.block_us,
        debug.candidate_count,
        debug.in_chunk_candidates,
        debug.placed,
        debug.size
    );
}

pub(super) fn ore_origin_overlaps_ocean_floor_wg(
    block_cache: &OreBlockCache,
    x_start: i32,
    y_start: i32,
    z_start: i32,
    size_xz: i32,
) -> bool {
    for x in x_start..=x_start + size_xz {
        for z in z_start..=z_start + size_xz {
            if block_cache
                .ocean_floor_wg_height(x, z)
                .is_some_and(|height| y_start <= height)
            {
                return true;
            }
        }
    }
    false
}

pub(super) fn is_adjacent_to_ore_air(block_cache: &OreBlockCache, pos: BlockPos) -> bool {
    const OFFSETS: [(i32, i32, i32); 6] = [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];
    OFFSETS.iter().any(|(dx, dy, dz)| {
        block_cache
            .block_state_name(pos.x + dx, pos.y + dy, pos.z + dz)
            .is_some_and(|block| block_matches_tag(block, "minecraft:air"))
    })
}
