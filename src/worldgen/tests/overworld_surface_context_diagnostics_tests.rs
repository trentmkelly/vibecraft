use super::*;

#[test]
#[ignore = "diagnostic for surface/water column mismatches against the vanilla fixture"]
fn normal_overworld_surface_water_column_diagnostic() {
    let fixture_json = include_str!(
        "../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    );
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

    let preset =
        super::super::resolve_world_preset("normal").expect("normal preset should resolve");
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &preset.overworld.generator
    else {
        panic!("normal overworld should use a noise generator");
    };
    let router_id = super::super::noise_router_id_for_settings(**noise_settings);
    let noise_router = super::super::builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .expect("normal overworld should have a built-in noise router");
    let climate_sampler =
        super::super::ClimateSampler::from_noise_router(&noise_router, seed, **noise_settings);
    let algorithm = if noise_settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let biome_zoom_seed = super::super::biome_manager_obfuscate_seed(seed);
    let surface_noise = super::super::random_state_normal_noise_snapshot(
        seed,
        **noise_settings,
        "minecraft:surface",
    );
    let base_rng = crate::random_source::random_state_seed_factories(seed, algorithm).base;

    fn fixture_block_type(block: &serde_json::Value) -> &str {
        block
            .as_str()
            .and_then(|raw| raw.split_once('[').map(|(name, _)| name).or(Some(raw)))
            .expect("fixture block should be a string")
    }

    fn is_surface_water_material(block: &str) -> bool {
        matches!(
            block,
            "minecraft:water"
                | "minecraft:sand"
                | "minecraft:dirt"
                | "minecraft:grass_block"
                | "minecraft:stone"
                | "minecraft:deepslate"
                | "minecraft:gravel"
                | "minecraft:air"
        )
    }

    fn column_stack_from_fixture(
        y_column: &[serde_json::Value],
        y_min: i32,
        local_z: usize,
        y0: i32,
        y1: i32,
    ) -> String {
        (y0..=y1)
            .map(|y| {
                let block = y_column
                    .get((y - y_min) as usize)
                    .and_then(serde_json::Value::as_array)
                    .and_then(|z_column| z_column.get(local_z))
                    .map(fixture_block_type)
                    .unwrap_or("minecraft:air");
                format!("{y}:{block}")
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn column_stack_from_chunk(
        chunk: &LevelChunk,
        world_x: i32,
        world_z: i32,
        y0: i32,
        y1: i32,
    ) -> String {
        (y0..=y1)
            .map(|y| {
                let block = chunk
                    .get_block_state(world_x, y, world_z)
                    .unwrap_or_else(|| "minecraft:air".to_string());
                format!("{y}:{block}")
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn density_and_aquifer_at(
        chunk_pos: ChunkPos,
        x: i32,
        y: i32,
        z: i32,
        seed: i64,
        settings: &super::super::NoiseGeneratorSettings,
        noise_router: super::super::NoiseRouter,
    ) -> (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        Option<&'static str>,
    ) {
        let chunk_min_x = chunk_pos.x * 16;
        let chunk_min_z = chunk_pos.z * 16;
        let mut noise_chunk =
            super::super::NoiseChunk::new(chunk_min_x, chunk_min_z, *settings, seed, noise_router);
        let algorithm = if settings.legacy_random_source {
            crate::random_source::RandomAlgorithm::Legacy
        } else {
            crate::random_source::RandomAlgorithm::Xoroshiro
        };
        let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
        let mut aquifer = settings.aquifers_enabled.then(|| {
            super::super::NoiseBasedAquifer::new(
                &mut noise_chunk,
                super::super::NoiseBasedAquiferBounds {
                    chunk_min_x,
                    chunk_max_x: chunk_min_x + 15,
                    chunk_min_z,
                    chunk_max_z: chunk_min_z + 15,
                    min_block_y: settings.noise.min_y,
                    y_block_size: settings.noise.height,
                },
                seed,
                *settings,
                noise_router,
                factories.aquifer,
            )
        });

        let cell_width = settings.noise.cell_width();
        let cell_height = settings.noise.cell_height();
        let cell_x_index = (x - chunk_min_x).div_euclid(cell_width);
        let cell_z_index = (z - chunk_min_z).div_euclid(cell_width);
        let cell_y_index = y.div_euclid(cell_height) - noise_chunk.cell_noise_min_y;
        noise_chunk.advance_cell_x(cell_x_index);
        noise_chunk.select_cell_yz(cell_y_index, cell_z_index);
        noise_chunk.update_for_y(
            y,
            f64::from(y.rem_euclid(cell_height)) / f64::from(cell_height),
        );
        noise_chunk.update_for_x(
            x,
            f64::from((x - chunk_min_x).rem_euclid(cell_width)) / f64::from(cell_width),
        );
        noise_chunk.update_for_z(
            z,
            f64::from((z - chunk_min_z).rem_euclid(cell_width)) / f64::from(cell_width),
        );
        let interpolated_density = noise_chunk.interpolated_density(x, y, z);
        let direct_density = noise_router
            .final_density
            .compute_with_noise(seed, *settings, x, y, z);
        let direct_depth = noise_router
            .depth
            .compute_with_noise(seed, *settings, x, y, z);
        let direct_base3d = super::super::BASE_3D_NOISE_OVERWORLD_DENSITY
            .compute_with_noise(seed, *settings, x, y, z);
        let direct_sloped = super::super::OVERWORLD_SLOPED_CHEESE_DENSITY
            .compute_with_noise(seed, *settings, x, y, z);
        let direct_continents = noise_router
            .continents
            .compute_with_noise(seed, *settings, x, y, z);
        let direct_erosion = noise_router
            .erosion
            .compute_with_noise(seed, *settings, x, y, z);
        let direct_ridges = noise_router
            .ridges
            .compute_with_noise(seed, *settings, x, y, z);
        let direct_prelim = noise_router
            .preliminary_surface_level
            .compute_with_noise(seed, *settings, x, y, z);
        let aquifer_state = aquifer.as_mut().and_then(|aquifer| {
            aquifer.compute_substance(&noise_chunk, x, y, z, interpolated_density)
        });
        (
            interpolated_density,
            direct_density,
            direct_base3d,
            direct_sloped,
            direct_depth,
            direct_continents,
            direct_erosion,
            direct_ridges,
            direct_prelim,
            aquifer_state,
        )
    }

    let mut pair_counts = BTreeMap::<(String, String), usize>::new();
    let mut printed = 0_usize;
    for fixture_chunk in fixture_chunks {
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
        let y_max = y_min
            + fixture_chunk
                .get("blocks")
                .and_then(serde_json::Value::as_array)
                .and_then(|x_columns| x_columns.first())
                .and_then(serde_json::Value::as_array)
                .expect("fixture chunk should include x/y block arrays")
                .len() as i32;
        let x_columns = fixture_chunk
            .get("blocks")
            .and_then(serde_json::Value::as_array)
            .expect("fixture chunk should include blocks");
        let pos = ChunkPos {
            x: chunk_x,
            z: chunk_z,
        };
        let (base, _, _) = super::super::generate_real_surface_base_chunk(
            pos,
            biome_source_model,
            noise_settings,
            seed,
        )
        .expect("real-surface base generation should succeed");
        let mut carver = base.clone();
        super::super::apply_configured_carvers_for_biome_source(
            &mut carver,
            biome_source_model,
            noise_settings,
            seed,
        );
        let mut ore = carver.clone();
        super::super::apply_mineshaft_underground_structures_to_chunk(&mut ore, seed);
        super::super::apply_underground_ore_decoration_to_chunk(
            &mut ore,
            biome_source_model,
            noise_settings,
            seed,
            None,
        );

        for (local_x, y_column_value) in x_columns.iter().enumerate() {
            let y_column = y_column_value
                .as_array()
                .expect("fixture x column should include y array");
            for (y_offset, z_column_value) in y_column.iter().enumerate() {
                let world_y = y_min + y_offset as i32;
                if !(48..=80).contains(&world_y) {
                    continue;
                }
                let z_column = z_column_value
                    .as_array()
                    .expect("fixture y column should include z array");
                for (local_z, expected_value) in z_column.iter().enumerate() {
                    let expected = fixture_block_type(expected_value);
                    let world_x = chunk_x * 16 + local_x as i32;
                    let world_z = chunk_z * 16 + local_z as i32;
                    let actual = ore
                        .get_block_state(world_x, world_y, world_z)
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    if expected == actual {
                        continue;
                    }
                    if !is_surface_water_material(expected)
                        && !is_surface_water_material(actual.as_str())
                    {
                        continue;
                    }
                    *pair_counts
                        .entry((expected.to_string(), actual.clone()))
                        .or_default() += 1;
                    if printed >= 16 {
                        continue;
                    }

                    let local_x_i32 = local_x as i32;
                    let local_z_i32 = local_z as i32;
                    let start_height =
                        super::super::read_world_surface_wg(&base, local_x, local_z) + 1;
                    let biome_y = if noise_settings.legacy_random_source {
                        0
                    } else {
                        start_height
                    };
                    let biome = super::super::biome_manager_get_biome(
                        biome_source_model,
                        biome_zoom_seed,
                        world_x,
                        biome_y,
                        world_z,
                        &climate_sampler,
                    )
                    .unwrap_or("minecraft:plains");
                    let rule_biome_y = if noise_settings.legacy_random_source {
                        0
                    } else {
                        world_y
                    };
                    let rule_biome = super::super::biome_manager_get_biome(
                        biome_source_model,
                        biome_zoom_seed,
                        world_x,
                        rule_biome_y,
                        world_z,
                        &climate_sampler,
                    )
                    .unwrap_or("minecraft:plains");
                    let surface_noise_value = surface_noise
                        .as_ref()
                        .map(|snap| {
                            super::super::normal_noise_sample(
                                snap,
                                world_x as f64,
                                0.0,
                                world_z as f64,
                            )
                        })
                        .unwrap_or(0.0);
                    let surface_depth = {
                        let mut at_rng = base_rng.at(world_x, 0, world_z);
                        let jitter = super::super::random_next_f64(&mut at_rng) * 0.25;
                        (surface_noise_value * 2.75 + 3.0 + jitter) as i32
                    };
                    let (
                        density,
                        direct_density,
                        direct_base3d,
                        direct_sloped,
                        direct_depth,
                        direct_continents,
                        direct_erosion,
                        direct_ridges,
                        direct_prelim,
                        aquifer_state,
                    ) = density_and_aquifer_at(
                        pos,
                        world_x,
                        world_y,
                        world_z,
                        seed,
                        noise_settings,
                        noise_router,
                    );
                    let actual_top = (y_min..y_max)
                        .rev()
                        .find(|y| {
                            ore.get_block_state(world_x, *y, world_z)
                                .is_some_and(|block| block != "minecraft:air")
                        })
                        .map(|y| y + 1)
                        .unwrap_or(y_min);
                    let expected_top = (0..y_column.len())
                        .rev()
                        .find(|offset| {
                            y_column[*offset]
                                .as_array()
                                .and_then(|z_column| z_column.get(local_z))
                                .map(fixture_block_type)
                                .is_some_and(|block| block != "minecraft:air")
                        })
                        .map(|offset| y_min + offset as i32 + 1)
                        .unwrap_or(y_min);

                    eprintln!(
                            "[worldgen-surface-water-sample] chunk=({chunk_x},{chunk_z}) local=({local_x_i32},{local_z_i32}) world=({world_x},{world_z}) y={world_y} expected={expected} actual={actual} expected_top={expected_top} actual_top={actual_top} base_world_surface_wg={start_height} surface_biome={biome}@{biome_y} rule_biome={rule_biome}@{rule_biome_y} surface_noise={surface_noise_value:.6} surface_depth={surface_depth} density={density:.6} direct_density={direct_density:.6} base3d={direct_base3d:.6} sloped={direct_sloped:.6} depth={direct_depth:.6} continents={direct_continents:.6} erosion={direct_erosion:.6} ridges={direct_ridges:.6} prelim={direct_prelim:.6} aquifer={aquifer_state:?}"
                        );
                    eprintln!(
                        "[worldgen-surface-water-column] expected {}",
                        column_stack_from_fixture(
                            y_column,
                            y_min,
                            local_z,
                            world_y - 6,
                            world_y + 8
                        )
                    );
                    eprintln!(
                        "[worldgen-surface-water-column] actual   {}",
                        column_stack_from_chunk(&ore, world_x, world_z, world_y - 6, world_y + 8)
                    );
                    printed += 1;
                }
            }
        }
    }

    eprintln!("[worldgen-surface-water-summary] top expected->actual mismatches in y=48..80");
    let mut sorted_pairs = pair_counts.into_iter().collect::<Vec<_>>();
    sorted_pairs.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for ((expected, actual), count) in sorted_pairs.into_iter().take(20) {
        eprintln!(
            "[worldgen-surface-water-pair] count={count} expected={expected} actual={actual}"
        );
    }
}

#[test]
#[ignore = "diagnostic for lightweight tree context heightmap drift after surface building"]
fn normal_overworld_tree_context_height_delta_diagnostic() {
    let fixture_json = include_str!(
        "../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    );
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

    let preset =
        super::super::resolve_world_preset("normal").expect("normal preset should resolve");
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &preset.overworld.generator
    else {
        panic!("normal overworld should use a noise generator");
    };
    let router = super::super::builtin_noise_router(super::super::noise_router_id_for_settings(
        **noise_settings,
    ))
    .expect("normal overworld should have a built-in noise router")
    .router;

    let mut total_world_surface_delta = 0_usize;
    let mut total_ocean_floor_delta = 0_usize;
    let mut total_motion_blocking_delta = 0_usize;
    let mut total_columns = 0_usize;
    for fixture_chunk in fixture_chunks {
        let chunk_x = fixture_chunk
            .get("chunkX")
            .and_then(serde_json::Value::as_i64)
            .expect("fixture chunk should include chunkX") as i32;
        let chunk_z = fixture_chunk
            .get("chunkZ")
            .and_then(serde_json::Value::as_i64)
            .expect("fixture chunk should include chunkZ") as i32;
        let pos = ChunkPos {
            x: chunk_x,
            z: chunk_z,
        };
        let (surface_chunk, _, _) = super::super::generate_real_surface_base_chunk(
            pos,
            biome_source_model,
            noise_settings,
            seed,
        )
        .expect("real-surface base generation should succeed");
        let live_heights =
            super::super::tree_decoration_terrain_heights(&surface_chunk, noise_settings);
        let lightweight_heights =
            super::super::noise_tree_context_heights(pos, noise_settings, seed, router);

        let mut chunk_world_surface_delta = 0_usize;
        let mut chunk_ocean_floor_delta = 0_usize;
        let mut chunk_motion_blocking_delta = 0_usize;
        let mut examples = Vec::new();
        for local_z in 0..16_usize {
            for local_x in 0..16_usize {
                let index = local_z * 16 + local_x;
                let live_world_surface = live_heights.world_surface[index];
                let lightweight_world_surface = lightweight_heights.world_surface[index];
                let live_ocean_floor = live_heights.ocean_floor[index];
                let lightweight_ocean_floor = lightweight_heights.ocean_floor[index];
                let live_motion_blocking = live_heights.motion_blocking[index];
                let lightweight_motion_blocking = lightweight_heights.motion_blocking[index];
                let differs = live_world_surface != lightweight_world_surface
                    || live_ocean_floor != lightweight_ocean_floor
                    || live_motion_blocking != lightweight_motion_blocking;
                if !differs {
                    continue;
                }
                if live_world_surface != lightweight_world_surface {
                    chunk_world_surface_delta += 1;
                }
                if live_ocean_floor != lightweight_ocean_floor {
                    chunk_ocean_floor_delta += 1;
                }
                if live_motion_blocking != lightweight_motion_blocking {
                    chunk_motion_blocking_delta += 1;
                }
                if examples.len() < 8 {
                    examples.push(format!(
                            "local=({local_x},{local_z}) world=({},{}) live_ws={} light_ws={} live_of={} light_of={} live_mb={} light_mb={}",
                            chunk_x * 16 + local_x as i32,
                            chunk_z * 16 + local_z as i32,
                            live_world_surface,
                            lightweight_world_surface,
                            live_ocean_floor,
                            lightweight_ocean_floor,
                            live_motion_blocking,
                            lightweight_motion_blocking
                        ));
                }
            }
        }
        total_world_surface_delta += chunk_world_surface_delta;
        total_ocean_floor_delta += chunk_ocean_floor_delta;
        total_motion_blocking_delta += chunk_motion_blocking_delta;
        total_columns += 16 * 16;
        eprintln!(
                "[tree-context-height-delta] chunk=({chunk_x},{chunk_z}) world_surface={} ocean_floor={} motion_blocking={} examples={}",
                chunk_world_surface_delta,
                chunk_ocean_floor_delta,
                chunk_motion_blocking_delta,
                examples.join(" | ")
            );
    }
    eprintln!(
            "[tree-context-height-delta-summary] chunks={} columns={} world_surface={} ocean_floor={} motion_blocking={}",
            fixture_chunks.len(),
            total_columns,
            total_world_surface_delta,
            total_ocean_floor_delta,
            total_motion_blocking_delta
        );
}

#[test]
#[ignore = "diagnostic for lightweight tree context synthetic block drift"]
fn normal_overworld_tree_context_block_delta_diagnostic() {
    let fixture_json = include_str!(
        "../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    );
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

    let preset =
        super::super::resolve_world_preset("normal").expect("normal preset should resolve");
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &preset.overworld.generator
    else {
        panic!("normal overworld should use a noise generator");
    };
    let router = super::super::builtin_noise_router(super::super::noise_router_id_for_settings(
        **noise_settings,
    ))
    .expect("normal overworld should have a built-in noise router")
    .router;

    let mut total_counts = BTreeMap::<(String, String), usize>::new();
    for fixture_chunk in fixture_chunks {
        let chunk_x = fixture_chunk
            .get("chunkX")
            .and_then(serde_json::Value::as_i64)
            .expect("fixture chunk should include chunkX") as i32;
        let chunk_z = fixture_chunk
            .get("chunkZ")
            .and_then(serde_json::Value::as_i64)
            .expect("fixture chunk should include chunkZ") as i32;
        let pos = ChunkPos {
            x: chunk_x,
            z: chunk_z,
        };
        let (surface_chunk, _, _) = super::super::generate_real_surface_base_chunk(
            pos,
            biome_source_model,
            noise_settings,
            seed,
        )
        .expect("real-surface base generation should succeed");
        let lightweight = super::super::LightweightTreeContextChunk {
            terrain_heights: super::super::noise_tree_context_heights(
                pos,
                noise_settings,
                seed,
                router,
            ),
            min_y: noise_settings.noise.min_y,
            max_y: noise_settings.noise.min_y + noise_settings.noise.height,
        };

        let mut chunk_counts = BTreeMap::<(String, String), usize>::new();
        let mut examples = Vec::new();
        for local_z in 0..16_usize {
            for local_x in 0..16_usize {
                let index = local_z * 16 + local_x;
                let ocean_floor = lightweight.terrain_heights.ocean_floor[index];
                let world_surface = lightweight.terrain_heights.world_surface[index];
                let min_y = (ocean_floor.min(world_surface) - 6).max(noise_settings.noise.min_y);
                let max_y = (world_surface + 8)
                    .min(noise_settings.noise.min_y + noise_settings.noise.height - 1);
                let world_x = chunk_x * 16 + local_x as i32;
                let world_z = chunk_z * 16 + local_z as i32;
                for y in min_y..=max_y {
                    let live = surface_chunk
                        .get_block_state_name(world_x, y, world_z)
                        .unwrap_or("minecraft:air");
                    let synthetic = lightweight.synthetic_block_state(world_x, y, world_z);
                    if live == synthetic {
                        continue;
                    }
                    *chunk_counts
                        .entry((live.to_string(), synthetic.to_string()))
                        .or_default() += 1;
                    *total_counts
                        .entry((live.to_string(), synthetic.to_string()))
                        .or_default() += 1;
                    if examples.len() < 8 {
                        examples.push(format!(
                                "world=({world_x},{y},{world_z}) live={live} synthetic={synthetic} of={ocean_floor} ws={world_surface}"
                            ));
                    }
                }
            }
        }

        let mut sorted = chunk_counts.into_iter().collect::<Vec<_>>();
        sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        let summary = sorted
            .into_iter()
            .take(8)
            .map(|((live, synthetic), count)| format!("{count}:{live}->{synthetic}"))
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!(
            "[tree-context-block-delta] chunk=({chunk_x},{chunk_z}) top_pairs=[{}] examples={}",
            summary,
            examples.join(" | ")
        );
    }

    eprintln!("[tree-context-block-delta-summary] top live->synthetic mismatches");
    let mut sorted = total_counts.into_iter().collect::<Vec<_>>();
    sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for ((live, synthetic), count) in sorted.into_iter().take(20) {
        eprintln!(
            "[tree-context-block-delta-pair] count={count} live={live} synthetic={synthetic}"
        );
    }
}
