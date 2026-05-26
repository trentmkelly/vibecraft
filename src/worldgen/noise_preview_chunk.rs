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
        let Ok(plan) = simple_tree_placement_plan(SimpleTreePlacementInput {
            origin: BlockPos {
                x: local_x as i32,
                y: surface_height,
                z: local_z as i32,
            },
            trunk: TrunkPlacerModel {
                base_height,
                height_rand_a: 2,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            foliage: FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Blob { height: 3 },
            },
            trunk_state,
            foliage_state: leaves_state,
            below_trunk_state: "minecraft:dirt",
            rand_a: (tree_seed & 0xffff) as i32,
            rand_b: ((tree_seed >> 16) & 0xffff) as i32,
        }) else {
            continue;
        };
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

#[derive(Clone, Copy)]
pub(super) struct LiveTreeDecorationInput<'a> {
    pub(super) chunk_pos: ChunkPos,
    pub(super) world_seed: i64,
    pub(super) settings: &'a NoiseGeneratorSettings,
    pub(super) biome_source_model: &'a BiomeSourceModel,
    pub(super) climate_sampler: &'a ClimateSampler,
    pub(super) global_features_per_step: Option<&'a [StepFeatureDataModel]>,
    pub(super) region_biome_steps: &'a [&'static [&'static [&'static str]]],
    pub(super) block_context: &'a TreeDecorationBlockContext<'a>,
    pub(super) terrain_heights: SourceTerrainHeights<'a>,
}

#[derive(Clone, Copy)]
struct LiveTreeTraceOptions {
    trees: bool,
    rejects: bool,
    attempts: bool,
}

#[derive(Clone, Copy)]
struct LiveTreeAttemptCandidate {
    feature: &'static str,
    attempt_index: i32,
    local_x: usize,
    local_z: usize,
    surface_height: i32,
    world_x: i32,
    world_z: i32,
    origin: BlockPos,
    candidate_biome: &'static str,
}

struct ValidStandingTreePlacement {
    candidate: LiveTreeAttemptCandidate,
    tree_config: LiveTreeFeatureConfig,
    rand_a: i32,
    rand_b: i32,
    clipped_tree_height: i32,
    prior_log_collision: bool,
    selector_trace: Option<String>,
}

struct TraceAttemptOriginSkip<'a> {
    call: &'a BiomeDecorationFeatureCall,
    attempt_index: i32,
    world_x: i32,
    surface_height: i32,
    world_z: i32,
    reason: &'static str,
    candidate_biome: Option<&'static str>,
}

struct StandingTreeTrace<'a> {
    candidate: LiveTreeAttemptCandidate,
    tree_config: LiveTreeFeatureConfig,
    plan: &'a TreePlacementPlan,
    rand_a: i32,
    rand_b: i32,
    prior_log_collision: bool,
    selector_trace: &'a str,
}

struct LiveTreeDecorationState<'a, 'b> {
    input: LiveTreeDecorationInput<'a>,
    diagnostics: &'b mut TreeDecorationDiagnostics,
    source_started: Instant,
    trace: LiveTreeTraceOptions,
    biome_zoom_seed: i64,
    biome_filter_cache: HashMap<(i32, i32, i32), &'static str>,
    blocks: Vec<TreePlacementBlock>,
    block_overlay: TreeBlockOverlay,
    accepted_log_positions: Option<HashSet<(i32, i32, i32)>>,
}

pub(super) fn live_tree_decoration_blocks(
    input: LiveTreeDecorationInput<'_>,
    diagnostics: &mut TreeDecorationDiagnostics,
) -> Vec<TreePlacementBlock> {
    if input.settings.id != "minecraft:overworld" && input.settings.id != "minecraft:large_biomes" {
        return Vec::new();
    }

    let source_started = Instant::now();
    diagnostics.sources_evaluated += 1;
    let Some(plan) = live_tree_decoration_feature_plan(input, diagnostics, source_started) else {
        return Vec::new();
    };
    let mut state = LiveTreeDecorationState::new(input, diagnostics, source_started);
    state.apply_plan(&plan);
    state.finish()
}

fn live_tree_decoration_feature_plan(
    input: LiveTreeDecorationInput<'_>,
    diagnostics: &mut TreeDecorationDiagnostics,
    source_started: Instant,
) -> Option<BiomeDecorationFeaturePlan> {
    let started = Instant::now();
    let biome_steps = input.region_biome_steps;
    diagnostics.source_biome_steps_ms += started.elapsed().as_millis();
    if biome_steps.is_empty() {
        diagnostics.source_total_us += source_started.elapsed().as_micros();
        return None;
    }

    let started = Instant::now();
    let local_features_per_step;
    let features_per_step = if let Some(features_per_step) = input.global_features_per_step {
        features_per_step
    } else {
        local_features_per_step = build_features_per_step(biome_steps, true).ok()?;
        &local_features_per_step
    };
    diagnostics.source_feature_sort_ms += started.elapsed().as_millis();

    let started = Instant::now();
    let plan = biome_decoration_feature_plan(
        input.world_seed,
        input.chunk_pos.x,
        input.chunk_pos.z,
        input.settings.noise.min_y.div_euclid(16),
        features_per_step,
        biome_steps,
    );
    diagnostics.source_plan_ms += started.elapsed().as_millis();
    diagnostics.feature_calls_total += plan.feature_calls.len();
    Some(plan)
}

impl<'a, 'b> LiveTreeDecorationState<'a, 'b> {
    fn new(
        input: LiveTreeDecorationInput<'a>,
        diagnostics: &'b mut TreeDecorationDiagnostics,
        source_started: Instant,
    ) -> Self {
        let trace = LiveTreeTraceOptions {
            trees: std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_TRACE").is_some(),
            rejects: std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_TRACE_REJECTS").is_some(),
            attempts: std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_TRACE_ATTEMPTS").is_some(),
        };
        Self {
            input,
            diagnostics,
            source_started,
            trace,
            biome_zoom_seed: biome_manager_obfuscate_seed(input.world_seed),
            biome_filter_cache: HashMap::new(),
            blocks: Vec::new(),
            block_overlay: TreeBlockOverlay::default(),
            accepted_log_positions: trace.trees.then(HashSet::new),
        }
    }

    fn apply_plan(&mut self, plan: &BiomeDecorationFeaturePlan) {
        for (call, count_kind) in plan.feature_calls.iter().filter_map(live_tree_feature_call) {
            self.apply_feature_call(call, count_kind);
        }
    }

    fn apply_feature_call(
        &mut self,
        call: &BiomeDecorationFeatureCall,
        count_kind: NoisePreviewTreeCountKind,
    ) {
        self.diagnostics.tree_feature_calls += 1;
        let mut random = RandomSourceKind::new(call.seed, RandomAlgorithm::Xoroshiro);
        let count = live_tree_count(count_kind, &mut random);
        self.diagnostics.tree_attempts += count as usize;
        for attempt_index in 0..count {
            self.apply_attempt(call, attempt_index, &mut random);
        }
    }

    fn apply_attempt(
        &mut self,
        call: &BiomeDecorationFeatureCall,
        attempt_index: i32,
        random: &mut RandomSourceKind,
    ) {
        let local_x = feature_random_next_i32_bound(random, 16) as usize;
        let local_z = feature_random_next_i32_bound(random, 16) as usize;
        let Some(candidate) = self.candidate_for_attempt(call, attempt_index, local_x, local_z)
        else {
            return;
        };
        if !self.passes_placement_sapling_filter(candidate) {
            return;
        }
        let selector_trace = self
            .trace
            .trees
            .then(|| live_tree_selector_trace(call.feature, *random))
            .flatten();
        let Some(selection) = live_tree_feature_selection(call.feature, random) else {
            return;
        };
        match selection {
            LiveTreeFeatureSelection::Fallen(config) => {
                self.place_fallen_tree(candidate, config, random, selector_trace);
            }
            LiveTreeFeatureSelection::Tree(tree_config) => {
                self.place_standing_tree(candidate, tree_config, random, selector_trace);
            }
        }
    }

    fn candidate_for_attempt(
        &mut self,
        call: &BiomeDecorationFeatureCall,
        attempt_index: i32,
        local_x: usize,
        local_z: usize,
    ) -> Option<LiveTreeAttemptCandidate> {
        let surface_height =
            self.input
                .terrain_heights
                .local_height(HeightmapKind::OceanFloor, local_x, local_z);
        self.trace_attempt_surface(call, attempt_index, local_x, local_z, surface_height);
        if surface_height <= self.input.settings.noise.min_y {
            self.trace_attempt_skip_no_ocean_floor(
                call,
                attempt_index,
                local_x,
                local_z,
                surface_height,
            );
            return None;
        }
        let world_surface =
            self.input
                .terrain_heights
                .local_height(HeightmapKind::WorldSurface, local_x, local_z);
        if world_surface - surface_height > 0 {
            self.trace_attempt_skip_surface_water_depth(
                call,
                attempt_index,
                local_x,
                local_z,
                surface_height,
                world_surface,
            );
            return None;
        }
        self.candidate_with_biome(call, attempt_index, local_x, local_z, surface_height)
    }

    fn candidate_with_biome(
        &mut self,
        call: &BiomeDecorationFeatureCall,
        attempt_index: i32,
        local_x: usize,
        local_z: usize,
        surface_height: i32,
    ) -> Option<LiveTreeAttemptCandidate> {
        let world_x = self.input.chunk_pos.x * 16 + local_x as i32;
        let world_z = self.input.chunk_pos.z * 16 + local_z as i32;
        let started = Instant::now();
        let candidate_biome = biome_manager_get_biome_cached(
            self.input.biome_source_model,
            self.biome_zoom_seed,
            BlockPos {
                x: world_x,
                y: surface_height,
                z: world_z,
            },
            self.input.climate_sampler,
            None,
            &mut self.biome_filter_cache,
        );
        self.diagnostics.candidate_biome_ms += started.elapsed().as_millis();
        let Some(candidate_biome) = candidate_biome else {
            self.trace_attempt_skip_origin(TraceAttemptOriginSkip {
                call,
                attempt_index,
                world_x,
                surface_height,
                world_z,
                reason: "no_biome",
                candidate_biome: None,
            });
            return None;
        };
        let Some(candidate_generation) = biome_generation_settings(candidate_biome) else {
            self.trace_attempt_skip_origin(TraceAttemptOriginSkip {
                call,
                attempt_index,
                world_x,
                surface_height,
                world_z,
                reason: "no_generation_settings",
                candidate_biome: Some(candidate_biome),
            });
            return None;
        };
        if !biome_has_placed_feature(candidate_generation, call.feature) {
            self.trace_attempt_skip_origin(TraceAttemptOriginSkip {
                call,
                attempt_index,
                world_x,
                surface_height,
                world_z,
                reason: "biome_filter",
                candidate_biome: Some(candidate_biome),
            });
            return None;
        }
        Some(LiveTreeAttemptCandidate {
            feature: call.feature,
            attempt_index,
            local_x,
            local_z,
            surface_height,
            world_x,
            world_z,
            origin: BlockPos {
                x: local_x as i32,
                y: surface_height,
                z: local_z as i32,
            },
            candidate_biome,
        })
    }

    fn passes_placement_sapling_filter(&self, candidate: LiveTreeAttemptCandidate) -> bool {
        let Some(sapling) = tree_placement_filter_sapling(candidate.feature) else {
            return true;
        };
        let survives = live_tree_sapling_survives_at(
            self.input.chunk_pos,
            self.input.block_context,
            &self.block_overlay,
            candidate.origin,
            sapling,
        );
        if !survives {
            self.trace_attempt_skip_candidate(candidate, "placement_sapling_filter", false);
        }
        survives
    }

    fn place_fallen_tree(
        &mut self,
        candidate: LiveTreeAttemptCandidate,
        config: FallenTreeConfigurationModel,
        random: &mut RandomSourceKind,
        selector_trace: Option<String>,
    ) {
        self.diagnostics.tree_candidates += 1;
        if !live_tree_sapling_survives_at(
            self.input.chunk_pos,
            self.input.block_context,
            &self.block_overlay,
            candidate.origin,
            live_tree_sapling_for_trunk_provider(&config.trunk_provider),
        ) {
            self.trace_attempt_skip_candidate(candidate, "sapling_survival", true);
            return;
        }
        let started = Instant::now();
        let plan = live_fallen_tree_placement_plan(
            self.input.chunk_pos,
            self.input.block_context,
            &self.block_overlay,
            candidate.origin,
            &config,
            random,
        );
        self.diagnostics.placement_plan_ms += started.elapsed().as_millis();
        let Some(plan) = plan else {
            return;
        };
        self.diagnostics.placement_plan_blocks += plan.blocks.len();
        self.trace_fallen_tree(candidate, &plan, selector_trace.as_deref().unwrap_or("n/a"));
        self.accept_plan_blocks(plan);
    }

    fn place_standing_tree(
        &mut self,
        candidate: LiveTreeAttemptCandidate,
        tree_config: LiveTreeFeatureConfig,
        random: &mut RandomSourceKind,
        selector_trace: Option<String>,
    ) {
        if !live_tree_sapling_survives_at(
            self.input.chunk_pos,
            self.input.block_context,
            &self.block_overlay,
            candidate.origin,
            live_tree_sapling_for_tree_config(tree_config),
        ) {
            self.trace_attempt_skip_candidate(candidate, "sapling_survival", false);
            return;
        }
        let rand_a = feature_random_next_i32_bound(random, tree_config.rand_a_bound);
        let rand_b = feature_random_next_i32_bound(random, tree_config.rand_b_bound);
        let prior_log_collision =
            self.prior_log_collision(candidate.origin, tree_config, rand_a, rand_b);
        self.diagnostics.tree_candidates += 1;
        let started = Instant::now();
        let clipped_tree_height = live_tree_clipped_height_with_previous_blocks(
            self.input.block_context,
            &self.block_overlay,
            candidate.origin,
            tree_config,
            rand_a,
            rand_b,
            self.input.settings,
        );
        self.diagnostics.validation_ms += started.elapsed().as_millis();
        let Some(clipped_tree_height) = clipped_tree_height else {
            self.diagnostics.validation_rejects += 1;
            self.trace_tree_validation_reject(candidate, tree_config, rand_a, rand_b);
            return;
        };
        self.diagnostics.validation_accepts += 1;
        self.place_valid_standing_tree(
            ValidStandingTreePlacement {
                candidate,
                tree_config,
                rand_a,
                rand_b,
                clipped_tree_height,
                prior_log_collision,
                selector_trace,
            },
            random,
        );
    }

    fn place_valid_standing_tree(
        &mut self,
        placement: ValidStandingTreePlacement,
        random: &mut RandomSourceKind,
    ) {
        let started = Instant::now();
        let Ok(mut plan) = live_tree_placement_plan(
            LiveTreePlacementInput {
                block_context: self.input.block_context,
                previous_source_blocks: &self.block_overlay,
                origin: placement.candidate.origin,
                config: placement.tree_config,
                rand_a: placement.rand_a,
                rand_b: placement.rand_b,
                clipped_tree_height: placement.clipped_tree_height,
            },
            random,
        ) else {
            return;
        };
        append_live_tree_decorators(
            LiveTreeDecoratorInput {
                source_pos: self.input.chunk_pos,
                block_context: self.input.block_context,
                source_terrain_heights: self.input.terrain_heights,
                settings: self.input.settings,
                previous_source_blocks: &self.block_overlay,
                decorators: placement.tree_config.decorators,
            },
            &mut plan,
            random,
        );
        self.diagnostics.placement_plan_ms += started.elapsed().as_millis();
        self.diagnostics.placement_plan_blocks += plan.blocks.len();
        self.trace_standing_tree(StandingTreeTrace {
            candidate: placement.candidate,
            tree_config: placement.tree_config,
            plan: &plan,
            rand_a: placement.rand_a,
            rand_b: placement.rand_b,
            prior_log_collision: placement.prior_log_collision,
            selector_trace: placement.selector_trace.as_deref().unwrap_or("n/a"),
        });
        self.accept_plan_logs(&plan);
        self.accept_plan_blocks(plan);
    }

    fn prior_log_collision(
        &self,
        origin: BlockPos,
        tree_config: LiveTreeFeatureConfig,
        rand_a: i32,
        rand_b: i32,
    ) -> bool {
        let Some(accepted_log_positions) = &self.accepted_log_positions else {
            return false;
        };
        let tree_height = trunk_placer_height(
            TrunkPlacerModel {
                base_height: tree_config.base_height,
                height_rand_a: tree_config.height_rand_a,
                height_rand_b: tree_config.height_rand_b,
                kind: if matches!(tree_config.foliage.kind, FoliagePlacerKind::Fancy { .. }) {
                    TrunkPlacerKind::Fancy
                } else {
                    TrunkPlacerKind::Straight
                },
            },
            rand_a,
            rand_b,
        );
        tree_validation_volume_intersects_world_positions(
            self.input.chunk_pos,
            origin,
            tree_height,
            tree_config.minimum_size,
            accepted_log_positions,
        )
    }

    fn accept_plan_logs(&mut self, plan: &TreePlacementPlan) {
        let Some(accepted_log_positions) = &mut self.accepted_log_positions else {
            return;
        };
        for block in plan
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::Log)
        {
            accepted_log_positions.insert((
                self.input.chunk_pos.x * 16 + block.pos.x,
                block.pos.y,
                self.input.chunk_pos.z * 16 + block.pos.z,
            ));
        }
    }

    fn accept_plan_blocks(&mut self, plan: TreePlacementPlan) {
        for block in &plan.blocks {
            self.block_overlay.insert(
                local_tree_block_to_world_key(self.input.chunk_pos, block.pos),
                block.state,
            );
        }
        self.blocks.extend(plan.blocks);
    }

    fn finish(self) -> Vec<TreePlacementBlock> {
        let started = Instant::now();
        let filtered = self
            .blocks
            .into_iter()
            .filter(|block| {
                (self.input.settings.noise.min_y
                    ..self.input.settings.noise.min_y + self.input.settings.noise.height)
                    .contains(&block.pos.y)
            })
            .collect::<Vec<_>>();
        self.diagnostics.filter_ms += started.elapsed().as_millis();
        self.diagnostics.filtered_blocks += filtered.len();
        self.diagnostics.source_total_us += self.source_started.elapsed().as_micros();
        filtered
    }

    fn trace_attempt_surface(
        &self,
        call: &BiomeDecorationFeatureCall,
        attempt_index: i32,
        local_x: usize,
        local_z: usize,
        surface_height: i32,
    ) {
        if self.trace.attempts {
            eprintln!(
                "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} local=({}, {}) surface={}",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                call.feature,
                attempt_index,
                local_x,
                local_z,
                surface_height,
            );
        }
    }

    fn trace_attempt_skip_no_ocean_floor(
        &self,
        call: &BiomeDecorationFeatureCall,
        attempt_index: i32,
        local_x: usize,
        local_z: usize,
        surface_height: i32,
    ) {
        if self.trace.attempts {
            eprintln!(
                "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} local=({}, {}) skip=no_ocean_floor surface={}",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                call.feature,
                attempt_index,
                local_x,
                local_z,
                surface_height,
            );
        }
    }

    fn trace_attempt_skip_surface_water_depth(
        &self,
        call: &BiomeDecorationFeatureCall,
        attempt_index: i32,
        local_x: usize,
        local_z: usize,
        surface_height: i32,
        world_surface: i32,
    ) {
        if self.trace.attempts {
            eprintln!(
                "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} local=({}, {}) skip=surface_water_depth surface={} world_surface={} depth={}",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                call.feature,
                attempt_index,
                local_x,
                local_z,
                surface_height,
                world_surface,
                world_surface - surface_height,
            );
        }
    }

    fn trace_attempt_skip_origin(&self, skip: TraceAttemptOriginSkip<'_>) {
        if !self.trace.attempts {
            return;
        }
        if let Some(candidate_biome) = skip.candidate_biome {
            eprintln!(
                "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) candidate_biome={} skip={}",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                skip.call.feature,
                skip.attempt_index,
                skip.world_x,
                skip.surface_height,
                skip.world_z,
                candidate_biome,
                skip.reason,
            );
        } else {
            eprintln!(
                "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) skip={}",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                skip.call.feature,
                skip.attempt_index,
                skip.world_x,
                skip.surface_height,
                skip.world_z,
                skip.reason,
            );
        }
    }

    fn trace_attempt_skip_candidate(
        &self,
        candidate: LiveTreeAttemptCandidate,
        reason: &str,
        fallen: bool,
    ) {
        if self.trace.attempts {
            let fallen_text = if fallen { " fallen=true" } else { "" };
            eprintln!(
                "[tree-trace-attempt] target=({},{}) source=({},{}) feature={} attempt={} origin=({}, {}, {}) candidate_biome={}{} skip={}",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                candidate.feature,
                candidate.attempt_index,
                candidate.world_x,
                candidate.surface_height,
                candidate.world_z,
                candidate.candidate_biome,
                fallen_text,
                reason,
            );
        }
    }

    fn trace_tree_validation_reject(
        &self,
        candidate: LiveTreeAttemptCandidate,
        tree_config: LiveTreeFeatureConfig,
        rand_a: i32,
        rand_b: i32,
    ) {
        if self.trace.rejects {
            eprintln!(
                "[tree-trace-reject] target=({},{}) source=({},{}) feature={} candidate_biome={} origin=({}, {}, {}) local=({}, {}) trunk={} leaves={} rand=({}, {})",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                candidate.feature,
                candidate.candidate_biome,
                candidate.world_x,
                candidate.surface_height,
                candidate.world_z,
                candidate.local_x,
                candidate.local_z,
                tree_config.trunk_state,
                tree_config.leaves_state,
                rand_a,
                rand_b,
            );
        }
    }

    fn trace_fallen_tree(
        &self,
        candidate: LiveTreeAttemptCandidate,
        plan: &TreePlacementPlan,
        selector_trace: &str,
    ) {
        if !self.trace.trees {
            return;
        }
        let blocks_in_target = live_tree_blocks_in_target(
            self.input.chunk_pos,
            self.input.block_context.target_pos,
            &plan.blocks,
            false,
        );
        if blocks_in_target > 0
            || self.input.block_context.source_pos == self.input.block_context.target_pos
        {
            eprintln!(
                "[tree-trace] target=({},{}) source=({},{}) feature={} candidate_biome={} origin=({}, {}, {}) local=({}, {}) fallen=true selector={} blocks_in_target={}",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                candidate.feature,
                candidate.candidate_biome,
                candidate.world_x,
                candidate.surface_height,
                candidate.world_z,
                candidate.local_x,
                candidate.local_z,
                selector_trace,
                blocks_in_target,
            );
        }
    }

    fn trace_standing_tree(&self, trace: StandingTreeTrace<'_>) {
        if !self.trace.trees {
            return;
        }
        let blocks_in_target = live_tree_blocks_in_target(
            self.input.chunk_pos,
            self.input.block_context.target_pos,
            &trace.plan.blocks,
            false,
        );
        let log_blocks_in_target = live_tree_blocks_in_target(
            self.input.chunk_pos,
            self.input.block_context.target_pos,
            &trace.plan.blocks,
            true,
        );
        if blocks_in_target == 0
            && self.input.block_context.source_pos != self.input.block_context.target_pos
        {
            return;
        }
        eprintln!(
            "[tree-trace] target=({},{}) source=({},{}) feature={} candidate_biome={} origin=({}, {}, {}) local=({}, {}) trunk={} leaves={} rand=({}, {}) selector={} blocks_in_target={} logs_in_target={}",
            self.input.block_context.target_pos.x,
            self.input.block_context.target_pos.z,
            self.input.chunk_pos.x,
            self.input.chunk_pos.z,
            trace.candidate.feature,
            trace.candidate.candidate_biome,
            trace.candidate.world_x,
            trace.candidate.surface_height,
            trace.candidate.world_z,
            trace.candidate.local_x,
            trace.candidate.local_z,
            trace.tree_config.trunk_state,
            trace.tree_config.leaves_state,
            trace.rand_a,
            trace.rand_b,
            trace.selector_trace,
            blocks_in_target,
            log_blocks_in_target,
        );
        if trace.prior_log_collision {
            eprintln!(
                "[tree-trace] stale-validation-risk target=({},{}) source=({},{}) feature={} origin=({}, {}, {}) intersects_prior_source_logs=true",
                self.input.block_context.target_pos.x,
                self.input.block_context.target_pos.z,
                self.input.chunk_pos.x,
                self.input.chunk_pos.z,
                trace.candidate.feature,
                trace.candidate.world_x,
                trace.candidate.surface_height,
                trace.candidate.world_z,
            );
        }
    }
}

fn live_tree_feature_call(
    call: &BiomeDecorationFeatureCall,
) -> Option<(&BiomeDecorationFeatureCall, NoisePreviewTreeCountKind)> {
    let count_kind = noise_preview_tree_feature_count_kind(call.feature)?;
    (call.step_index == GenerationDecorationStep::VegetalDecoration as usize)
        .then_some((call, count_kind))
}

fn live_tree_blocks_in_target(
    source_pos: ChunkPos,
    target_pos: ChunkPos,
    blocks: &[TreePlacementBlock],
    logs_only: bool,
) -> usize {
    let target_min_x = target_pos.x * 16;
    let target_min_z = target_pos.z * 16;
    let target_max_x = target_min_x + 15;
    let target_max_z = target_min_z + 15;
    blocks
        .iter()
        .filter(|block| !logs_only || block.kind == TreePlacementBlockKind::Log)
        .filter(|block| {
            let block_world_x = source_pos.x * 16 + block.pos.x;
            let block_world_z = source_pos.z * 16 + block.pos.z;
            block_world_x >= target_min_x
                && block_world_x <= target_max_x
                && block_world_z >= target_min_z
                && block_world_z <= target_max_z
        })
        .count()
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

    for (call, count_kind) in plan.feature_calls.iter().filter_map(|call| {
        let count_kind = noise_preview_tree_feature_count_kind(call.feature)?;
        (call.step_index == GenerationDecorationStep::VegetalDecoration as usize)
            .then_some((call, count_kind))
    }) {
        let mut random = RandomSourceKind::new(call.seed, RandomAlgorithm::Xoroshiro);
        let count = live_tree_count(count_kind, &mut random);

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
pub(super) enum NoisePreviewTreeCountKind {
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

pub(super) fn live_tree_count(
    kind: NoisePreviewTreeCountKind,
    random: &mut RandomSourceKind,
) -> i32 {
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

pub(super) fn noise_preview_ground_cover_blocks(
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
            let state = if sunflowers && roll.is_multiple_of(97) {
                Some("minecraft:sunflower")
            } else if flowers && roll.is_multiple_of(23) {
                Some(if biome == "minecraft:forest" {
                    "minecraft:poppy"
                } else {
                    "minecraft:dandelion"
                })
            } else if grass && roll.is_multiple_of(7) {
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

pub(super) fn noise_preview_terrain_height(
    x: i32,
    z: i32,
    settings: &NoiseGeneratorSettings,
) -> i32 {
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
