use super::*;

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
    pos: BlockPos,
    vein_toggle: f64,
    vein_ridged: f64,
    vein_gap: f64,
    debug_ore_veins: bool,
) -> Option<&'static str> {
    ore_vein_decision_after_toggle(
        ore_factory,
        pos,
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
                BlockPos { x, y, z },
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
                BlockPos { x, y, z },
                vein_toggle,
                || noise_chunk.vein_ridged_at(x, y, z),
                || noise_chunk.vein_gap_at(x, y, z),
                self.debug_ore_veins,
            )
        }
    }
}

pub(super) struct NoiseMaterialRuleList {
    default_block: &'static str,
    ore_vein_rule: Option<OreVeinMaterialRule>,
}

pub(super) struct MaterialRuleCalculationInput<'a> {
    pub(super) aquifer: Option<&'a mut NoiseBasedAquifer>,
    pub(super) noise_chunk: &'a NoiseChunk,
    pub(super) settings: &'a NoiseGeneratorSettings,
    pub(super) pos: BlockPos,
    pub(super) density: f64,
    pub(super) timings: &'a mut LiveTerrainTimings,
    pub(super) detailed_timing: bool,
}

impl NoiseMaterialRuleList {
    pub(super) fn new(
        settings: &NoiseGeneratorSettings,
        ore_factory: PositionalRandomFactory,
    ) -> Self {
        Self {
            default_block: settings.default_block,
            ore_vein_rule: settings.ore_veins_enabled.then_some(OreVeinMaterialRule {
                ore_factory,
                debug_ore_veins: false,
            }),
        }
    }

    pub(super) fn calculate(&self, input: MaterialRuleCalculationInput<'_>) -> &'static str {
        // Mirrors Java's `MaterialRuleList`: the aquifer filler runs first and
        // may return a fluid/air block. Returning `None` means the slot remains
        // solid, so the next filler (OreVeinifier) gets a chance before the
        // generator falls back to the default block.
        let aquifer_substance = if input.density > 0.0 {
            None
        } else if let Some(aquifer) = input.aquifer {
            input.timings.aquifer_calls += 1;
            if input.detailed_timing {
                let aquifer_started = Instant::now();
                let substance = aquifer.compute_substance(
                    input.noise_chunk,
                    input.pos.x,
                    input.pos.y,
                    input.pos.z,
                    input.density,
                );
                input.timings.fill_aquifer_compute_us += aquifer_started.elapsed().as_micros();
                substance
            } else {
                aquifer.compute_substance(
                    input.noise_chunk,
                    input.pos.x,
                    input.pos.y,
                    input.pos.z,
                    input.density,
                )
            }
        } else {
            Some(
                global_fluid_status(
                    input.pos.y,
                    input.settings.sea_level,
                    input.settings.default_fluid,
                )
                .at(input.pos.y),
            )
        };

        if let Some(block) = aquifer_substance {
            return block;
        }

        if let Some(rule) = &self.ore_vein_rule {
            if let Some(block) = rule.try_apply(
                input.noise_chunk,
                input.pos.x,
                input.pos.y,
                input.pos.z,
                input.timings,
                input.detailed_timing,
            ) {
                return block;
            }
        }

        self.default_block
    }
}

pub(super) fn ore_vein_decision_after_toggle<R, G>(
    ore_factory: PositionalRandomFactory,
    pos: BlockPos,
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
    let distance_from_top = vein_type.max_y - pos.y;
    let distance_from_bottom = pos.y - vein_type.min_y;
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
    let mut positional_random = ore_factory.at(pos.x, pos.y, pos.z);
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

pub(super) fn clamped_map(
    value: f64,
    from_min: f64,
    from_max: f64,
    to_min: f64,
    to_max: f64,
) -> f64 {
    let clamped = value.clamp(from_min, from_max);
    let progress = (clamped - from_min) / (from_max - from_min);
    to_min + progress * (to_max - to_min)
}

pub(super) fn apply_underground_ore_decoration_to_chunk(
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

pub(super) fn apply_underground_ore_decoration_to_chunk_with_context(
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

pub(super) fn apply_underground_ore_decoration_from_source_into_region(
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
