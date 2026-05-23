use super::*;

pub fn materialize_noise_preview_chunk(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
) -> LevelChunk {
    let mut chunk = LevelChunk::empty(pos);
    chunk.status = "minecraft:full".to_string();

    let min_y = settings.noise.min_y;
    let max_y = settings.noise.min_y + settings.noise.height;
    let min_section = min_y.div_euclid(16);
    chunk.min_section_y = min_section;
    let section_count = (settings.noise.height + 15) / 16;
    let mut terrain_heights = [settings.sea_level + 1; 16 * 16];
    for z in 0..16 {
        for x in 0..16 {
            let world_x = pos.x * 16 + x as i32;
            let world_z = pos.z * 16 + z as i32;
            terrain_heights[z * 16 + x] =
                noise_preview_terrain_height(world_x, world_z, settings).clamp(min_y + 1, max_y);
        }
    }
    let biome = noise_preview_biome(biome_source_model, pos);
    let mut overlay_blocks = noise_preview_tree_blocks(pos, 0, settings, biome, &terrain_heights);
    overlay_blocks.extend(noise_preview_ground_cover_blocks(
        pos,
        settings,
        biome,
        &terrain_heights,
    ));

    chunk.sections = (0..section_count)
        .map(|section_offset| {
            let section_y = min_section + section_offset;
            ChunkSection {
                y: section_y as i8,
                block_states: noise_preview_section_block_states(
                    section_y,
                    min_y,
                    settings,
                    &terrain_heights,
                    &overlay_blocks,
                )
                .to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String(biome.to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: Some(vec![-1; 2048]),
            }
        })
        .collect();

    let mut world_surface = terrain_heights.map(|height| height.max(settings.sea_level + 1));
    for block in &overlay_blocks {
        if (0..16).contains(&block.pos.x) && (0..16).contains(&block.pos.z) {
            let index = block.pos.z as usize * 16 + block.pos.x as usize;
            world_surface[index] = world_surface[index].max(block.pos.y + 1);
        }
    }
    chunk.heightmaps = BTreeMap::from([
        (
            HeightmapKind::WorldSurfaceWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap(world_surface)),
        ),
        (
            HeightmapKind::OceanFloorWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap(terrain_heights)),
        ),
    ]);
    chunk
}

fn noise_preview_section_block_states(
    section_y: i32,
    min_y: i32,
    settings: &NoiseGeneratorSettings,
    surface_heights: &[i32; 16 * 16],
    tree_blocks: &[TreePlacementBlock],
) -> PalettedContainer {
    let mut palette: Vec<&'static str> = Vec::new();
    let mut indices = vec![0_u64; SECTION_VOLUME];
    for local_y in 0..16 {
        let world_y = section_y * 16 + local_y as i32;
        for z in 0..16 {
            for x in 0..16 {
                let surface_height = surface_heights[z * 16 + x];
                let block = tree_blocks
                    .iter()
                    .find(|block| {
                        block.pos.x == x as i32 && block.pos.y == world_y && block.pos.z == z as i32
                    })
                    .map(|block| block.state)
                    .unwrap_or_else(|| {
                        noise_preview_block_at(world_y, min_y, surface_height, settings.sea_level)
                    });
                let palette_index = match palette.iter().position(|entry| *entry == block) {
                    Some(index) => index as u64,
                    None => {
                        palette.push(block);
                        (palette.len() - 1) as u64
                    }
                };
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

pub(super) fn noise_preview_block_at(
    world_y: i32,
    min_y: i32,
    surface_height: i32,
    sea_level: i32,
) -> &'static str {
    if world_y <= min_y {
        "minecraft:bedrock"
    } else if world_y >= surface_height {
        if world_y <= sea_level {
            "minecraft:water"
        } else {
            "minecraft:air"
        }
    } else if world_y == surface_height - 1 {
        if surface_height <= sea_level + 1 {
            "minecraft:sand"
        } else {
            "minecraft:grass_block"
        }
    } else if world_y >= surface_height - 4 {
        if surface_height <= sea_level + 1 {
            "minecraft:sandstone"
        } else {
            "minecraft:dirt"
        }
    } else if world_y < min_y + 5 {
        "minecraft:deepslate"
    } else {
        "minecraft:stone"
    }
}

pub(super) fn noise_preview_tree_blocks(
    chunk_pos: ChunkPos,
    world_seed: i64,
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
    let mut blocks = Vec::new();
    for (index, (local_x, local_z, tree_seed)) in noise_preview_tree_origins(
        generation,
        world_seed,
        chunk_pos,
        settings.noise.min_y.div_euclid(16),
    )
    .into_iter()
    .enumerate()
    {
        let surface_height = terrain_heights[local_z * 16 + local_x];
        if surface_height <= settings.sea_level + 2 {
            continue;
        }
        let tree_seed = tree_seed ^ (index as i64).wrapping_mul(10_000) as u64;
        let (trunk_state, leaves_state, base_height) =
            noise_preview_tree_materials(generation, tree_seed);
        let plan = simple_tree_placement_plan(
            BlockPos {
                x: local_x as i32,
                y: surface_height,
                z: local_z as i32,
            },
            TrunkPlacerModel {
                base_height,
                height_rand_a: 2,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Blob { height: 3 },
            },
            trunk_state,
            leaves_state,
            "minecraft:dirt",
            (tree_seed & 0xffff) as i32,
            ((tree_seed >> 16) & 0xffff) as i32,
        )
        .expect("hard-coded preview tree configuration must validate");
        blocks.extend(plan.blocks);
    }
    blocks
        .into_iter()
        .filter(|block| {
            (settings.noise.min_y..settings.noise.min_y + settings.noise.height)
                .contains(&block.pos.y)
        })
        .collect()
}

pub(super) fn live_tree_decoration_blocks(
    chunk_pos: ChunkPos,
    world_seed: i64,
    settings: &NoiseGeneratorSettings,
    biome_source_model: &BiomeSourceModel,
    climate_sampler: &ClimateSampler,
    global_features_per_step: Option<&[StepFeatureDataModel]>,
    region_biome_steps: &[&'static [&'static [&'static str]]],
    block_context: &TreeDecorationBlockContext<'_>,
    terrain_heights: SourceTerrainHeights<'_>,
    diagnostics: &mut TreeDecorationDiagnostics,
) -> Vec<TreePlacementBlock> {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return Vec::new();
    }

    let source_started = Instant::now();
    diagnostics.sources_evaluated += 1;
    let started = Instant::now();
    let biome_steps = region_biome_steps;
    diagnostics.source_biome_steps_ms += started.elapsed().as_millis();
    if biome_steps.is_empty() {
        diagnostics.source_total_us += source_started.elapsed().as_micros();
        return Vec::new();
    }

    let started = Instant::now();
    let local_features_per_step;
    let features_per_step = if let Some(features_per_step) = global_features_per_step {
        features_per_step
    } else {
        local_features_per_step = match build_features_per_step(&biome_steps, true) {
            Ok(features) => features,
            Err(_) => return Vec::new(),
        };
        &local_features_per_step
    };
    diagnostics.source_feature_sort_ms += started.elapsed().as_millis();
    let started = Instant::now();
    let plan = biome_decoration_feature_plan(
        world_seed,
        chunk_pos.x,
        chunk_pos.z,
        settings.noise.min_y.div_euclid(16),
        &features_per_step,
        &biome_steps,
    );
    diagnostics.source_plan_ms += started.elapsed().as_millis();
    diagnostics.feature_calls_total += plan.feature_calls.len();

    let trace_trees = std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_TRACE").is_some();
    let trace_rejects = std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_TRACE_REJECTS").is_some();
    let trace_attempts = std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_TRACE_ATTEMPTS").is_some();
    let biome_zoom_seed = biome_manager_obfuscate_seed(world_seed);
    let mut biome_filter_cache: HashMap<(i32, i32, i32), &'static str> = HashMap::new();
    let mut blocks = Vec::new();
    let mut block_overlay = TreeBlockOverlay::default();
    let mut accepted_log_positions = trace_trees.then(HashSet::new);
    for call in plan.feature_calls.iter().filter(|call| {
        call.step_index == GenerationDecorationStep::VegetalDecoration as usize
            && noise_preview_tree_feature_count_kind(call.feature).is_some()
    }) {
        diagnostics.tree_feature_calls += 1;
        let mut random = RandomSourceKind::new(call.seed, RandomAlgorithm::Xoroshiro);
        let count = live_tree_count(
            noise_preview_tree_feature_count_kind(call.feature).unwrap(),
            &mut random,
        );
        diagnostics.tree_attempts += count as usize;

        for attempt_index in 0..count {
            let local_x = feature_random_next_i32_bound(&mut random, 16) as usize;
            let local_z = feature_random_next_i32_bound(&mut random, 16) as usize;
            let surface_height =
                terrain_heights.local_height(HeightmapKind::OceanFloor, local_x, local_z);
            if trace_attempts {
                eprintln!(
                    "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} local=({}, {}) surface={}",
                    block_context.target_pos.x,
                    block_context.target_pos.z,
                    chunk_pos.x,
                    chunk_pos.z,
                    call.feature,
                    attempt_index,
                    local_x,
                    local_z,
                    surface_height,
                );
            }
            if surface_height <= settings.noise.min_y {
                if trace_attempts {
                    eprintln!(
                        "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} local=({}, {}) skip=no_ocean_floor surface={}",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        attempt_index,
                        local_x,
                        local_z,
                        surface_height,
                    );
                }
                continue;
            }
            let world_surface =
                terrain_heights.local_height(HeightmapKind::WorldSurface, local_x, local_z);
            if world_surface - surface_height > 0 {
                if trace_attempts {
                    eprintln!(
                        "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} local=({}, {}) skip=surface_water_depth surface={} world_surface={} depth={}",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        attempt_index,
                        local_x,
                        local_z,
                        surface_height,
                        world_surface,
                        world_surface - surface_height,
                    );
                }
                continue;
            }

            let world_x = chunk_pos.x * 16 + local_x as i32;
            let world_z = chunk_pos.z * 16 + local_z as i32;
            let started = Instant::now();
            let Some(candidate_biome) = biome_manager_get_biome_cached(
                biome_source_model,
                biome_zoom_seed,
                world_x,
                surface_height,
                world_z,
                climate_sampler,
                None,
                &mut biome_filter_cache,
            ) else {
                diagnostics.candidate_biome_ms += started.elapsed().as_millis();
                if trace_attempts {
                    eprintln!(
                        "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) skip=no_biome",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        attempt_index,
                        world_x,
                        surface_height,
                        world_z,
                    );
                }
                continue;
            };
            diagnostics.candidate_biome_ms += started.elapsed().as_millis();
            let Some(candidate_generation) = biome_generation_settings(candidate_biome) else {
                if trace_attempts {
                    eprintln!(
                        "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) candidate_biome={} skip=no_generation_settings",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        attempt_index,
                        world_x,
                        surface_height,
                        world_z,
                        candidate_biome,
                    );
                }
                continue;
            };
            if !biome_has_placed_feature(candidate_generation, call.feature) {
                if trace_attempts {
                    eprintln!(
                        "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) candidate_biome={} skip=biome_filter",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        attempt_index,
                        world_x,
                        surface_height,
                        world_z,
                        candidate_biome,
                    );
                }
                continue;
            }

            if let Some(sapling) = tree_placement_filter_sapling(call.feature) {
                let origin = BlockPos {
                    x: local_x as i32,
                    y: surface_height,
                    z: local_z as i32,
                };
                if !live_tree_sapling_survives_at(
                    chunk_pos,
                    block_context,
                    &block_overlay,
                    origin,
                    sapling,
                ) {
                    if trace_attempts {
                        eprintln!(
                            "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) candidate_biome={} skip=placement_sapling_filter",
                            block_context.target_pos.x,
                            block_context.target_pos.z,
                            chunk_pos.x,
                            chunk_pos.z,
                            call.feature,
                            attempt_index,
                            world_x,
                            surface_height,
                            world_z,
                            candidate_biome,
                        );
                    }
                    continue;
                }
            }

            let selector_trace = trace_trees
                .then(|| live_tree_selector_trace(call.feature, random))
                .flatten();
            let Some(selection) = live_tree_feature_selection(call.feature, &mut random) else {
                continue;
            };
            let origin = BlockPos {
                x: local_x as i32,
                y: surface_height,
                z: local_z as i32,
            };
            if let LiveTreeFeatureSelection::Fallen(config) = selection {
                diagnostics.tree_candidates += 1;
                if !live_tree_sapling_survives_at(
                    chunk_pos,
                    block_context,
                    &block_overlay,
                    origin,
                    live_tree_sapling_for_trunk_provider(&config.trunk_provider),
                ) {
                    if trace_attempts {
                        eprintln!(
                            "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) candidate_biome={} fallen=true skip=sapling_survival",
                            block_context.target_pos.x,
                            block_context.target_pos.z,
                            chunk_pos.x,
                            chunk_pos.z,
                            call.feature,
                            attempt_index,
                            world_x,
                            surface_height,
                            world_z,
                            candidate_biome,
                        );
                    }
                    continue;
                }
                let started = Instant::now();
                let Some(plan) = live_fallen_tree_placement_plan(
                    chunk_pos,
                    block_context,
                    &block_overlay,
                    origin,
                    &config,
                    &mut random,
                ) else {
                    diagnostics.placement_plan_ms += started.elapsed().as_millis();
                    continue;
                };
                diagnostics.placement_plan_ms += started.elapsed().as_millis();
                diagnostics.placement_plan_blocks += plan.blocks.len();
                if trace_trees {
                    let target_min_x = block_context.target_pos.x * 16;
                    let target_min_z = block_context.target_pos.z * 16;
                    let target_max_x = target_min_x + 15;
                    let target_max_z = target_min_z + 15;
                    let blocks_in_target = plan
                        .blocks
                        .iter()
                        .filter(|block| {
                            let block_world_x = chunk_pos.x * 16 + block.pos.x;
                            let block_world_z = chunk_pos.z * 16 + block.pos.z;
                            block_world_x >= target_min_x
                                && block_world_x <= target_max_x
                                && block_world_z >= target_min_z
                                && block_world_z <= target_max_z
                        })
                        .count();
                    if blocks_in_target > 0 || block_context.source_pos == block_context.target_pos
                    {
                        eprintln!(
                            "[tree-trace] target=({},{}) source=({},{}) feature={} candidate_biome={} origin=({}, {}, {}) local=({}, {}) fallen=true selector={} blocks_in_target={}",
                            block_context.target_pos.x,
                            block_context.target_pos.z,
                            chunk_pos.x,
                            chunk_pos.z,
                            call.feature,
                            candidate_biome,
                            world_x,
                            surface_height,
                            world_z,
                            local_x,
                            local_z,
                            selector_trace.as_deref().unwrap_or("n/a"),
                            blocks_in_target,
                        );
                    }
                }
                for block in &plan.blocks {
                    block_overlay.insert(
                        local_tree_block_to_world_key(chunk_pos, block.pos),
                        block.state,
                    );
                }
                blocks.extend(plan.blocks);
                continue;
            }
            let LiveTreeFeatureSelection::Tree(tree_config) = selection else {
                unreachable!("fallen tree selections are handled above");
            };
            if !live_tree_sapling_survives_at(
                chunk_pos,
                block_context,
                &block_overlay,
                origin,
                live_tree_sapling_for_tree_config(tree_config),
            ) {
                if trace_attempts {
                    eprintln!(
                        "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) candidate_biome={} skip=sapling_survival",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        attempt_index,
                        world_x,
                        surface_height,
                        world_z,
                        candidate_biome,
                    );
                }
                continue;
            }
            let rand_a = feature_random_next_i32_bound(&mut random, tree_config.rand_a_bound);
            let rand_b = feature_random_next_i32_bound(&mut random, tree_config.rand_b_bound);
            let prior_log_collision = if let Some(accepted_log_positions) = &accepted_log_positions
            {
                let tree_height = trunk_placer_height(
                    TrunkPlacerModel {
                        base_height: tree_config.base_height,
                        height_rand_a: tree_config.height_rand_a,
                        height_rand_b: tree_config.height_rand_b,
                        kind: if matches!(tree_config.foliage.kind, FoliagePlacerKind::Fancy { .. })
                        {
                            TrunkPlacerKind::Fancy
                        } else {
                            TrunkPlacerKind::Straight
                        },
                    },
                    rand_a,
                    rand_b,
                );
                tree_validation_volume_intersects_world_positions(
                    chunk_pos,
                    origin,
                    tree_height,
                    tree_config.minimum_size,
                    accepted_log_positions,
                )
            } else {
                false
            };
            diagnostics.tree_candidates += 1;
            let started = Instant::now();
            let Some(clipped_tree_height) = live_tree_clipped_height_with_previous_blocks(
                block_context,
                &block_overlay,
                origin,
                tree_config,
                rand_a,
                rand_b,
                settings,
            ) else {
                diagnostics.validation_ms += started.elapsed().as_millis();
                diagnostics.validation_rejects += 1;
                if trace_rejects {
                    eprintln!(
                        "[tree-trace-reject] target=({},{}) source=({},{}) feature={} candidate_biome={} origin=({}, {}, {}) local=({}, {}) trunk={} leaves={} rand=({}, {})",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        candidate_biome,
                        world_x,
                        surface_height,
                        world_z,
                        local_x,
                        local_z,
                        tree_config.trunk_state,
                        tree_config.leaves_state,
                        rand_a,
                        rand_b,
                    );
                }
                continue;
            };
            diagnostics.validation_ms += started.elapsed().as_millis();
            diagnostics.validation_accepts += 1;
            let started = Instant::now();
            let mut plan = live_tree_placement_plan(
                block_context,
                &block_overlay,
                origin,
                tree_config,
                rand_a,
                rand_b,
                clipped_tree_height,
                &mut random,
            )
            .expect("hard-coded preview tree configuration must validate");
            append_live_tree_decorators(
                chunk_pos,
                block_context,
                terrain_heights,
                settings,
                &block_overlay,
                &mut plan,
                tree_config.decorators,
                &mut random,
            );
            diagnostics.placement_plan_ms += started.elapsed().as_millis();
            diagnostics.placement_plan_blocks += plan.blocks.len();
            if trace_trees {
                let target_min_x = block_context.target_pos.x * 16;
                let target_min_z = block_context.target_pos.z * 16;
                let target_max_x = target_min_x + 15;
                let target_max_z = target_min_z + 15;
                let blocks_in_target = plan
                    .blocks
                    .iter()
                    .filter(|block| {
                        let block_world_x = chunk_pos.x * 16 + block.pos.x;
                        let block_world_z = chunk_pos.z * 16 + block.pos.z;
                        block_world_x >= target_min_x
                            && block_world_x <= target_max_x
                            && block_world_z >= target_min_z
                            && block_world_z <= target_max_z
                    })
                    .count();
                let log_blocks_in_target = plan
                    .blocks
                    .iter()
                    .filter(|block| {
                        if block.kind != TreePlacementBlockKind::Log {
                            return false;
                        }
                        let block_world_x = chunk_pos.x * 16 + block.pos.x;
                        let block_world_z = chunk_pos.z * 16 + block.pos.z;
                        block_world_x >= target_min_x
                            && block_world_x <= target_max_x
                            && block_world_z >= target_min_z
                            && block_world_z <= target_max_z
                    })
                    .count();
                if blocks_in_target > 0 || block_context.source_pos == block_context.target_pos {
                    eprintln!(
                        "[tree-trace] target=({},{}) source=({},{}) feature={} candidate_biome={} origin=({}, {}, {}) local=({}, {}) trunk={} leaves={} rand=({}, {}) selector={} blocks_in_target={} logs_in_target={}",
                        block_context.target_pos.x,
                        block_context.target_pos.z,
                        chunk_pos.x,
                        chunk_pos.z,
                        call.feature,
                        candidate_biome,
                        world_x,
                        surface_height,
                        world_z,
                        local_x,
                        local_z,
                        tree_config.trunk_state,
                        tree_config.leaves_state,
                        rand_a,
                        rand_b,
                        selector_trace.as_deref().unwrap_or("n/a"),
                        blocks_in_target,
                        log_blocks_in_target,
                    );
                    if prior_log_collision {
                        eprintln!(
                            "[tree-trace] stale-validation-risk target=({},{}) source=({},{}) feature={} origin=({}, {}, {}) intersects_prior_source_logs=true",
                            block_context.target_pos.x,
                            block_context.target_pos.z,
                            chunk_pos.x,
                            chunk_pos.z,
                            call.feature,
                            world_x,
                            surface_height,
                            world_z,
                        );
                    }
                }
            }
            if let Some(accepted_log_positions) = &mut accepted_log_positions {
                for block in plan
                    .blocks
                    .iter()
                    .filter(|block| block.kind == TreePlacementBlockKind::Log)
                {
                    accepted_log_positions.insert((
                        chunk_pos.x * 16 + block.pos.x,
                        block.pos.y,
                        chunk_pos.z * 16 + block.pos.z,
                    ));
                }
            }
            for block in &plan.blocks {
                block_overlay.insert(
                    local_tree_block_to_world_key(chunk_pos, block.pos),
                    block.state,
                );
            }
            blocks.extend(plan.blocks);
        }
    }

    let started = Instant::now();
    let filtered = blocks
        .into_iter()
        .filter(|block| {
            (settings.noise.min_y..settings.noise.min_y + settings.noise.height)
                .contains(&block.pos.y)
        })
        .collect::<Vec<_>>();
    diagnostics.filter_ms += started.elapsed().as_millis();
    diagnostics.filtered_blocks += filtered.len();
    diagnostics.source_total_us += source_started.elapsed().as_micros();
    filtered
}

fn tree_validation_volume_intersects_world_positions(
    source_pos: ChunkPos,
    origin: BlockPos,
    tree_height: i32,
    min_size: FeatureSizeModel,
    positions: &HashSet<(i32, i32, i32)>,
) -> bool {
    for y_offset in 0..=tree_height + 1 {
        let radius = feature_size_at_height(min_size, tree_height, y_offset);
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let world_x = source_pos.x * 16 + origin.x + dx;
                let world_y = origin.y + y_offset;
                let world_z = source_pos.z * 16 + origin.z + dz;
                if positions.contains(&(world_x, world_y, world_z)) {
                    return true;
                }
            }
        }
    }
    false
}

