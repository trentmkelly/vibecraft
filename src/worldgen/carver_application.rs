use super::*;

pub fn pack_carving_mask_indices(indices: &[usize]) -> Vec<i64> {
    let Some(max_index) = indices.iter().copied().max() else {
        return Vec::new();
    };
    let mut words = vec![0_i64; max_index / 64 + 1];
    for index in indices {
        let word_index = index / 64;
        let bit_index = index % 64;
        words[word_index] |= (1_i64) << bit_index;
    }
    words
}

pub(super) fn carve_configured_carver_from_source_chunk(
    chunk: &mut LevelChunk,
    height_context: WorldGenerationHeightContext,
    carver: &ConfiguredCarver,
    source_chunk_x: i32,
    source_chunk_z: i32,
    target_chunk_min_x: i32,
    target_chunk_min_z: i32,
    random: &mut LegacyRandom,
    mask: &mut Vec<usize>,
    settings: &NoiseGeneratorSettings,
    noise_chunk: &NoiseChunk,
    mut aquifer: Option<&mut NoiseBasedAquifer>,
) -> usize {
    match carver.shape {
        CarverShape::Cave {
            horizontal_radius_multiplier,
            vertical_radius_multiplier,
            floor_level,
        } => {
            let cave_count = sample_cave_carver_cave_count(carver.carver_type, random);
            let max_distance = (4 * 2 - 1) << 4;
            let mut carved = 0;
            for _ in 0..cave_count {
                let x = f64::from(source_chunk_x * 16 + random.next_i32_bound(16));
                let y = f64::from(sample_carver_y(carver.y, height_context, random));
                let z = f64::from(source_chunk_z * 16 + random.next_i32_bound(16));
                let horizontal_radius_multiplier =
                    f64::from(sample_float_provider(horizontal_radius_multiplier, random));
                let vertical_radius_multiplier =
                    f64::from(sample_float_provider(vertical_radius_multiplier, random));
                let floor_level = f64::from(sample_float_provider(floor_level, random));
                let mut tunnels = 1;
                if random.next_i32_bound(4) == 0 {
                    let y_scale = sample_float_provider(carver.y_scale, random) as f64;
                    let thickness = 1.0 + random.next_f32() * 6.0;
                    let (base_horizontal_radius, base_vertical_radius) =
                        cave_room_radii(thickness, y_scale);
                    carved += carve_ellipsoid_into_chunk(
                        chunk,
                        height_context,
                        carver,
                        target_chunk_min_x,
                        target_chunk_min_z,
                        x + 1.0,
                        y,
                        z,
                        base_horizontal_radius,
                        base_vertical_radius,
                        CarverSkipModel::Cave { floor_level },
                        mask,
                        settings,
                        noise_chunk,
                        aquifer.as_deref_mut(),
                    );
                    tunnels += random.next_i32_bound(4);
                }
                for _ in 0..tunnels {
                    let horizontal_rotation = random.next_f32() * std::f32::consts::TAU;
                    let vertical_rotation = (random.next_f32() - 0.5) / 4.0;
                    let thickness = match carver.carver_type {
                        WorldCarverType::NetherCave => {
                            nether_carver_thickness(random.next_f32(), random.next_f32())
                        }
                        _ => {
                            let mut thickness = random.next_f32() * 2.0 + random.next_f32();
                            if random.next_i32_bound(10) == 0 {
                                thickness *= random.next_f32() * random.next_f32() * 3.0 + 1.0;
                            }
                            thickness
                        }
                    };
                    let distance = max_distance - random.next_i32_bound(max_distance / 4);
                    carved += carve_cave_tunnel_into_chunk(
                        chunk,
                        height_context,
                        carver,
                        target_chunk_min_x,
                        target_chunk_min_z,
                        random.next_i64(),
                        x,
                        y,
                        z,
                        horizontal_radius_multiplier,
                        vertical_radius_multiplier,
                        thickness,
                        horizontal_rotation,
                        vertical_rotation,
                        0,
                        distance,
                        carver_tunnel_y_scale(carver.carver_type),
                        floor_level,
                        mask,
                        0,
                        settings,
                        noise_chunk,
                        aquifer.as_deref_mut(),
                    );
                }
            }
            carved
        }
        CarverShape::Canyon {
            vertical_rotation,
            ref shape,
        } => {
            let max_distance = (4 * 2 - 1) << 4;
            let x = f64::from(source_chunk_x * 16 + random.next_i32_bound(16));
            let y = f64::from(sample_carver_y(carver.y, height_context, random));
            let z = f64::from(source_chunk_z * 16 + random.next_i32_bound(16));
            let horizontal_rotation = random.next_f32() * std::f32::consts::TAU;
            let vertical_rotation = sample_float_provider(vertical_rotation, random);
            let y_scale = sample_float_provider(carver.y_scale, random) as f64;
            let thickness = sample_float_provider(shape.thickness, random);
            let distance =
                (max_distance as f32 * sample_float_provider(shape.distance_factor, random)) as i32;
            carve_canyon_tunnel_into_chunk(
                chunk,
                height_context,
                carver,
                target_chunk_min_x,
                target_chunk_min_z,
                random.next_i64(),
                x,
                y,
                z,
                thickness,
                horizontal_rotation,
                vertical_rotation,
                distance,
                y_scale,
                shape,
                mask,
                settings,
                noise_chunk,
                aquifer.as_deref_mut(),
            )
        }
    }
}

fn carve_ellipsoid_into_chunk(
    chunk: &mut LevelChunk,
    height_context: WorldGenerationHeightContext,
    carver: &ConfiguredCarver,
    chunk_min_x: i32,
    chunk_min_z: i32,
    x: f64,
    y: f64,
    z: f64,
    horizontal_radius: f64,
    vertical_radius: f64,
    skip_model: CarverSkipModel<'_>,
    mask: &mut Vec<usize>,
    settings: &NoiseGeneratorSettings,
    noise_chunk: &NoiseChunk,
    mut aquifer: Option<&mut NoiseBasedAquifer>,
) -> usize {
    let positions = carver_ellipsoid_candidate_positions(
        chunk_min_x,
        chunk_min_z,
        height_context,
        false,
        x,
        y,
        z,
        horizontal_radius,
        vertical_radius,
        mask,
        false,
        skip_model,
    );
    let mut carved = 0;
    for pos in positions {
        let Some(block) = chunk.get_block_state_name(pos.x, pos.y, pos.z) else {
            continue;
        };
        let Some(block) = carver_static_block_name(block) else {
            continue;
        };
        let (aquifer_state, should_schedule_fluid_update) = carver_aquifer_carve_state(
            carver,
            height_context,
            settings,
            noise_chunk,
            aquifer.as_deref_mut(),
            pos,
        );
        let input = CarverBlockInput {
            pos,
            block,
            was_masked: carver_mask_index(pos.x, pos.y, pos.z, height_context.min_y)
                .is_some_and(|index| mask.contains(&index)),
            aquifer_state,
            should_schedule_fluid_update,
            debug_enabled: false,
        };
        let outcome = carver_carve_block(carver, height_context, input);
        if carver_trace_matches(pos) {
            eprintln!(
                "[carver-trace] carver={} pos=({},{},{}) block={} was_masked={} aquifer_state={:?} outcome={:?}",
                carver.id,
                pos.x,
                pos.y,
                pos.z,
                block,
                input.was_masked,
                input.aquifer_state,
                outcome.as_ref().map(|outcome| outcome.state)
            );
        }
        let Some(outcome) = outcome else {
            continue;
        };
        chunk.set_block_state(pos.x, pos.y, pos.z, outcome.state);
        mask.push(outcome.mask_index);
        carved += 1;
    }
    carved
}

fn carver_trace_matches(pos: BlockPos) -> bool {
    let Ok(raw) = std::env::var("RUSTCRAFT_WORLDGEN_CARVER_TRACE") else {
        return false;
    };
    raw.split(';').any(|entry| {
        let mut parts = entry.split(',');
        let Some(x) = parts
            .next()
            .and_then(|part| part.trim().parse::<i32>().ok())
        else {
            return false;
        };
        let Some(y) = parts
            .next()
            .and_then(|part| part.trim().parse::<i32>().ok())
        else {
            return false;
        };
        let Some(z) = parts
            .next()
            .and_then(|part| part.trim().parse::<i32>().ok())
        else {
            return false;
        };
        parts.next().is_none() && pos.x == x && pos.y == y && pos.z == z
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn carve_cave_tunnel_into_chunk(
    chunk: &mut LevelChunk,
    height_context: WorldGenerationHeightContext,
    carver: &ConfiguredCarver,
    chunk_min_x: i32,
    chunk_min_z: i32,
    tunnel_seed: i64,
    mut x: f64,
    mut y: f64,
    mut z: f64,
    horizontal_radius_multiplier: f64,
    vertical_radius_multiplier: f64,
    thickness: f32,
    mut horizontal_rotation: f32,
    mut vertical_rotation: f32,
    start_step: i32,
    distance: i32,
    y_scale: f64,
    floor_level: f64,
    mask: &mut Vec<usize>,
    depth: usize,
    settings: &NoiseGeneratorSettings,
    noise_chunk: &NoiseChunk,
    mut aquifer: Option<&mut NoiseBasedAquifer>,
) -> usize {
    if distance < 2 || depth > 8 {
        return 0;
    }
    let mut random = LegacyRandom::new(tunnel_seed);
    let split_point = random.next_i32_bound(distance / 2) + distance / 4;
    let steep = random.next_i32_bound(6) == 0;
    let mut y_rota = 0.0_f32;
    let mut x_rota = 0.0_f32;
    let chunk_middle_x = f64::from(chunk_min_x + 8);
    let chunk_middle_z = f64::from(chunk_min_z + 8);
    let mut carved = 0;

    for current_step in start_step..distance {
        let horizontal_radius = 1.5
            + f64::from((std::f32::consts::PI * current_step as f32 / distance as f32).sin())
                * f64::from(thickness);
        let vertical_radius = horizontal_radius * y_scale;
        let cos_x = vertical_rotation.cos();
        x += f64::from(horizontal_rotation.cos() * cos_x);
        y += f64::from(vertical_rotation.sin());
        z += f64::from(horizontal_rotation.sin() * cos_x);
        vertical_rotation *= if steep { 0.92 } else { 0.7 };
        vertical_rotation += x_rota * 0.1;
        horizontal_rotation += y_rota * 0.1;
        x_rota *= 0.9;
        y_rota *= 0.75;
        x_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
        y_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;

        if current_step == split_point && thickness > 1.0 {
            let left_seed = random.next_i64();
            let left_thickness = random.next_f32() * 0.5 + 0.5;
            carved += carve_cave_tunnel_into_chunk(
                chunk,
                height_context,
                carver,
                chunk_min_x,
                chunk_min_z,
                left_seed,
                x,
                y,
                z,
                horizontal_radius_multiplier,
                vertical_radius_multiplier,
                left_thickness,
                horizontal_rotation - std::f32::consts::FRAC_PI_2,
                vertical_rotation / 3.0,
                current_step,
                distance,
                1.0,
                floor_level,
                mask,
                depth + 1,
                settings,
                noise_chunk,
                aquifer.as_deref_mut(),
            );
            let right_seed = random.next_i64();
            let right_thickness = random.next_f32() * 0.5 + 0.5;
            carved += carve_cave_tunnel_into_chunk(
                chunk,
                height_context,
                carver,
                chunk_min_x,
                chunk_min_z,
                right_seed,
                x,
                y,
                z,
                horizontal_radius_multiplier,
                vertical_radius_multiplier,
                right_thickness,
                horizontal_rotation + std::f32::consts::FRAC_PI_2,
                vertical_rotation / 3.0,
                current_step,
                distance,
                1.0,
                floor_level,
                mask,
                depth + 1,
                settings,
                noise_chunk,
                aquifer.as_deref_mut(),
            );
            return carved;
        }

        if random.next_i32_bound(4) == 0 {
            continue;
        }
        let can_reach = (x - chunk_middle_x) * (x - chunk_middle_x)
            + (z - chunk_middle_z) * (z - chunk_middle_z)
            - f64::from(distance - current_step).powi(2)
            <= f64::from(thickness + 18.0).powi(2);
        if !can_reach {
            return carved;
        }
        carved += carve_ellipsoid_into_chunk(
            chunk,
            height_context,
            carver,
            chunk_min_x,
            chunk_min_z,
            x,
            y,
            z,
            horizontal_radius * horizontal_radius_multiplier,
            vertical_radius * vertical_radius_multiplier,
            CarverSkipModel::Cave { floor_level },
            mask,
            settings,
            noise_chunk,
            aquifer.as_deref_mut(),
        );
    }

    carved
}

#[allow(clippy::too_many_arguments)]
pub(super) fn carve_canyon_tunnel_into_chunk(
    chunk: &mut LevelChunk,
    height_context: WorldGenerationHeightContext,
    carver: &ConfiguredCarver,
    chunk_min_x: i32,
    chunk_min_z: i32,
    tunnel_seed: i64,
    mut x: f64,
    mut y: f64,
    mut z: f64,
    thickness: f32,
    mut horizontal_rotation: f32,
    mut vertical_rotation: f32,
    distance: i32,
    y_scale: f64,
    shape: &CanyonShapeConfiguration,
    mask: &mut Vec<usize>,
    settings: &NoiseGeneratorSettings,
    noise_chunk: &NoiseChunk,
    mut aquifer: Option<&mut NoiseBasedAquifer>,
) -> usize {
    if distance < 1 {
        return 0;
    }
    let mut random = LegacyRandom::new(tunnel_seed);
    let width_factors = canyon_width_factors_from_random(
        height_context.height,
        shape.width_smoothness,
        &mut random,
    );
    let chunk_middle_x = f64::from(chunk_min_x + 8);
    let chunk_middle_z = f64::from(chunk_min_z + 8);
    let mut y_rota = 0.0_f32;
    let mut x_rota = 0.0_f32;
    let mut carved = 0;

    for current_step in 0..distance {
        let base_horizontal_radius = 1.5
            + f64::from((current_step as f32 * std::f32::consts::PI / distance as f32).sin())
                * f64::from(thickness);
        let horizontal_radius = base_horizontal_radius
            * f64::from(sample_float_provider(
                shape.horizontal_radius_factor,
                &mut random,
            ));
        let vertical_radius = canyon_vertical_radius(
            shape.vertical_radius_default_factor,
            shape.vertical_radius_center_factor,
            base_horizontal_radius * y_scale,
            distance,
            current_step,
            random.next_f32(),
        );
        let xc = vertical_rotation.cos();
        x += f64::from(horizontal_rotation.cos() * xc);
        y += f64::from(vertical_rotation.sin());
        z += f64::from(horizontal_rotation.sin() * xc);
        vertical_rotation *= 0.7;
        vertical_rotation += x_rota * 0.05;
        horizontal_rotation += y_rota * 0.05;
        x_rota *= 0.8;
        y_rota *= 0.5;
        x_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
        y_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;

        if random.next_i32_bound(4) == 0 {
            continue;
        }
        let can_reach = (x - chunk_middle_x) * (x - chunk_middle_x)
            + (z - chunk_middle_z) * (z - chunk_middle_z)
            - f64::from(distance - current_step).powi(2)
            <= f64::from(thickness + 18.0).powi(2);
        if !can_reach {
            return carved;
        }
        carved += carve_ellipsoid_into_chunk(
            chunk,
            height_context,
            carver,
            chunk_min_x,
            chunk_min_z,
            x,
            y,
            z,
            horizontal_radius,
            vertical_radius,
            CarverSkipModel::Canyon {
                width_factors: &width_factors,
            },
            mask,
            settings,
            noise_chunk,
            aquifer.as_deref_mut(),
        );
    }

    carved
}

fn carver_aquifer_carve_state(
    carver: &ConfiguredCarver,
    height_context: WorldGenerationHeightContext,
    settings: &NoiseGeneratorSettings,
    noise_chunk: &NoiseChunk,
    aquifer: Option<&mut NoiseBasedAquifer>,
    pos: BlockPos,
) -> (Option<&'static str>, bool) {
    if pos.y <= carver_effective_lava_y(carver, height_context) {
        return (Some("minecraft:lava"), false);
    }
    if let Some(aquifer) = aquifer {
        let state = aquifer.compute_substance(noise_chunk, pos.x, pos.y, pos.z, 0.0);
        return (state, aquifer.should_schedule_fluid_update);
    }
    let fluid = global_fluid_status(pos.y, settings.sea_level, settings.default_fluid);
    (disabled_aquifer_substance(0.0, fluid, pos.y), false)
}

fn canyon_width_factors_from_random(
    depth: i32,
    width_smoothness: i32,
    random: &mut LegacyRandom,
) -> Vec<f32> {
    let mut factors = Vec::new();
    let mut width_factor = 1.0_f32;
    for y_index in 0..depth.max(0) {
        if y_index == 0 || width_smoothness <= 0 || random.next_i32_bound(width_smoothness) == 0 {
            width_factor = 1.0 + random.next_f32() * random.next_f32();
        }
        factors.push(width_factor * width_factor);
    }
    factors
}

fn sample_carver_y(
    range: HeightRange,
    context: WorldGenerationHeightContext,
    random: &mut LegacyRandom,
) -> i32 {
    let min = range.min.resolve_y(context);
    let max = range.max.resolve_y(context);
    if max <= min {
        min
    } else {
        min + random.next_i32_bound(max - min + 1)
    }
}

fn sample_float_provider(provider: FloatProvider, random: &mut LegacyRandom) -> f32 {
    match provider {
        FloatProvider::Constant(value) => value,
        FloatProvider::Uniform { min, max } => min + (max - min) * random.next_f32(),
        FloatProvider::Trapezoid { min, max, plateau } => {
            if max <= min {
                return min;
            }
            let span = max - min;
            let plateau = plateau.clamp(0.0, span);
            let slope = (span - plateau) * 0.5;
            let first = random.next_f32() * (slope + plateau);
            let second = random.next_f32() * slope;
            min + first + second
        }
    }
}

pub(super) fn carver_static_block_name(block: &str) -> Option<&'static str> {
    match block_state_id(block) {
        "minecraft:stone" => Some("minecraft:stone"),
        "minecraft:granite" => Some("minecraft:granite"),
        "minecraft:diorite" => Some("minecraft:diorite"),
        "minecraft:andesite" => Some("minecraft:andesite"),
        "minecraft:tuff" => Some("minecraft:tuff"),
        "minecraft:calcite" => Some("minecraft:calcite"),
        "minecraft:dirt" => Some("minecraft:dirt"),
        "minecraft:grass_block" => Some("minecraft:grass_block"),
        "minecraft:podzol" => Some("minecraft:podzol"),
        "minecraft:mycelium" => Some("minecraft:mycelium"),
        "minecraft:coarse_dirt" => Some("minecraft:coarse_dirt"),
        "minecraft:rooted_dirt" => Some("minecraft:rooted_dirt"),
        "minecraft:deepslate" => Some("minecraft:deepslate"),
        "minecraft:sandstone" => Some("minecraft:sandstone"),
        "minecraft:red_sandstone" => Some("minecraft:red_sandstone"),
        "minecraft:sand" => Some("minecraft:sand"),
        "minecraft:red_sand" => Some("minecraft:red_sand"),
        "minecraft:clay" => Some("minecraft:clay"),
        "minecraft:gravel" => Some("minecraft:gravel"),
        "minecraft:water" => Some("minecraft:water"),
        "minecraft:ice" => Some("minecraft:ice"),
        "minecraft:packed_ice" => Some("minecraft:packed_ice"),
        "minecraft:snow_block" => Some("minecraft:snow_block"),
        "minecraft:netherrack" => Some("minecraft:netherrack"),
        "minecraft:basalt" => Some("minecraft:basalt"),
        "minecraft:blackstone" => Some("minecraft:blackstone"),
        "minecraft:soul_sand" => Some("minecraft:soul_sand"),
        "minecraft:soul_soil" => Some("minecraft:soul_soil"),
        "minecraft:crimson_nylium" => Some("minecraft:crimson_nylium"),
        "minecraft:warped_nylium" => Some("minecraft:warped_nylium"),
        "minecraft:nether_wart_block" => Some("minecraft:nether_wart_block"),
        "minecraft:warped_wart_block" => Some("minecraft:warped_wart_block"),
        "minecraft:oak_log" => Some("minecraft:oak_log"),
        "minecraft:spruce_log" => Some("minecraft:spruce_log"),
        "minecraft:birch_log" => Some("minecraft:birch_log"),
        "minecraft:jungle_log" => Some("minecraft:jungle_log"),
        "minecraft:acacia_log" => Some("minecraft:acacia_log"),
        "minecraft:dark_oak_log" => Some("minecraft:dark_oak_log"),
        "minecraft:mangrove_log" => Some("minecraft:mangrove_log"),
        "minecraft:cherry_log" => Some("minecraft:cherry_log"),
        "minecraft:pale_oak_log" => Some("minecraft:pale_oak_log"),
        "minecraft:oak_leaves" => Some("minecraft:oak_leaves"),
        "minecraft:spruce_leaves" => Some("minecraft:spruce_leaves"),
        "minecraft:birch_leaves" => Some("minecraft:birch_leaves"),
        "minecraft:jungle_leaves" => Some("minecraft:jungle_leaves"),
        "minecraft:acacia_leaves" => Some("minecraft:acacia_leaves"),
        "minecraft:dark_oak_leaves" => Some("minecraft:dark_oak_leaves"),
        "minecraft:mangrove_leaves" => Some("minecraft:mangrove_leaves"),
        "minecraft:cherry_leaves" => Some("minecraft:cherry_leaves"),
        "minecraft:pale_oak_leaves" => Some("minecraft:pale_oak_leaves"),
        "minecraft:short_grass" => Some("minecraft:short_grass"),
        "minecraft:tall_grass" => Some("minecraft:tall_grass"),
        "minecraft:fern" => Some("minecraft:fern"),
        "minecraft:large_fern" => Some("minecraft:large_fern"),
        "minecraft:bush" => Some("minecraft:bush"),
        "minecraft:leaf_litter" => Some("minecraft:leaf_litter"),
        _ => None,
    }
}
