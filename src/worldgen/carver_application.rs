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

pub(super) struct SourceChunkCarverInput<'a> {
    pub chunk: &'a mut LevelChunk,
    pub height_context: WorldGenerationHeightContext,
    pub carver: &'a ConfiguredCarver,
    pub source_chunk_x: i32,
    pub source_chunk_z: i32,
    pub target_chunk_min_x: i32,
    pub target_chunk_min_z: i32,
    pub random: &'a mut LegacyRandom,
    pub mask: &'a mut Vec<usize>,
    pub settings: &'a NoiseGeneratorSettings,
    pub noise_chunk: &'a NoiseChunk,
    pub aquifer: Option<&'a mut NoiseBasedAquifer>,
}

pub(super) fn carve_configured_carver_from_source_chunk(
    input: SourceChunkCarverInput<'_>,
) -> usize {
    match input.carver.shape {
        CarverShape::Cave {
            horizontal_radius_multiplier,
            vertical_radius_multiplier,
            floor_level,
        } => carve_cave_configured_carver_from_source_chunk(
            input,
            horizontal_radius_multiplier,
            vertical_radius_multiplier,
            floor_level,
        ),
        CarverShape::Canyon {
            vertical_rotation,
            shape,
        } => carve_canyon_configured_carver_from_source_chunk(input, vertical_rotation, shape),
    }
}

fn carve_cave_configured_carver_from_source_chunk(
    mut input: SourceChunkCarverInput<'_>,
    horizontal_radius_multiplier: FloatProvider,
    vertical_radius_multiplier: FloatProvider,
    floor_level: FloatProvider,
) -> usize {
    let cave_count = sample_cave_carver_cave_count(input.carver.carver_type, input.random);
    let max_distance = (4 * 2 - 1) << 4;
    let mut carved = 0;
    for _ in 0..cave_count {
        let x = f64::from(input.source_chunk_x * 16 + input.random.next_i32_bound(16));
        let y = f64::from(sample_carver_y(
            input.carver.y,
            input.height_context,
            input.random,
        ));
        let z = f64::from(input.source_chunk_z * 16 + input.random.next_i32_bound(16));
        let horizontal_radius_multiplier = f64::from(sample_float_provider(
            horizontal_radius_multiplier,
            input.random,
        ));
        let vertical_radius_multiplier = f64::from(sample_float_provider(
            vertical_radius_multiplier,
            input.random,
        ));
        let floor_level = f64::from(sample_float_provider(floor_level, input.random));
        let mut tunnels = 1;
        if input.random.next_i32_bound(4) == 0 {
            let y_scale = sample_float_provider(input.carver.y_scale, input.random) as f64;
            let thickness = 1.0 + input.random.next_f32() * 6.0;
            let (base_horizontal_radius, base_vertical_radius) =
                cave_room_radii(thickness, y_scale);
            carved += carve_ellipsoid_into_chunk(CarveEllipsoidChunkInput {
                chunk: &mut *input.chunk,
                height_context: input.height_context,
                carver: input.carver,
                chunk_min_x: input.target_chunk_min_x,
                chunk_min_z: input.target_chunk_min_z,
                x: x + 1.0,
                y,
                z,
                horizontal_radius: base_horizontal_radius,
                vertical_radius: base_vertical_radius,
                skip_model: CarverSkipModel::Cave { floor_level },
                mask: &mut *input.mask,
                settings: input.settings,
                noise_chunk: input.noise_chunk,
                aquifer: input.aquifer.as_deref_mut(),
            });
            tunnels += input.random.next_i32_bound(4);
        }
        carved += carve_cave_tunnels_from_source_position(
            &mut input,
            CaveSourceTunnelBatch {
                x,
                y,
                z,
                horizontal_radius_multiplier,
                vertical_radius_multiplier,
                floor_level,
                tunnels,
                max_distance,
            },
        );
    }
    carved
}

struct CaveSourceTunnelBatch {
    x: f64,
    y: f64,
    z: f64,
    horizontal_radius_multiplier: f64,
    vertical_radius_multiplier: f64,
    floor_level: f64,
    tunnels: i32,
    max_distance: i32,
}

fn carve_cave_tunnels_from_source_position(
    input: &mut SourceChunkCarverInput<'_>,
    batch: CaveSourceTunnelBatch,
) -> usize {
    let mut carved = 0;
    for _ in 0..batch.tunnels {
        let horizontal_rotation = input.random.next_f32() * std::f32::consts::TAU;
        let vertical_rotation = (input.random.next_f32() - 0.5) / 4.0;
        let thickness = match input.carver.carver_type {
            WorldCarverType::NetherCave => {
                nether_carver_thickness(input.random.next_f32(), input.random.next_f32())
            }
            _ => {
                let mut thickness = input.random.next_f32() * 2.0 + input.random.next_f32();
                if input.random.next_i32_bound(10) == 0 {
                    thickness *= input.random.next_f32() * input.random.next_f32() * 3.0 + 1.0;
                }
                thickness
            }
        };
        let distance = batch.max_distance - input.random.next_i32_bound(batch.max_distance / 4);
        carved += carve_cave_tunnel_into_chunk(CaveTunnelChunkInput {
            chunk: input.chunk,
            height_context: input.height_context,
            carver: input.carver,
            chunk_min_x: input.target_chunk_min_x,
            chunk_min_z: input.target_chunk_min_z,
            tunnel_seed: input.random.next_i64(),
            x: batch.x,
            y: batch.y,
            z: batch.z,
            horizontal_radius_multiplier: batch.horizontal_radius_multiplier,
            vertical_radius_multiplier: batch.vertical_radius_multiplier,
            thickness,
            horizontal_rotation,
            vertical_rotation,
            start_step: 0,
            distance,
            y_scale: carver_tunnel_y_scale(input.carver.carver_type),
            floor_level: batch.floor_level,
            mask: input.mask,
            depth: 0,
            settings: input.settings,
            noise_chunk: input.noise_chunk,
            aquifer: input.aquifer.as_deref_mut(),
        });
    }
    carved
}

fn carve_canyon_configured_carver_from_source_chunk(
    input: SourceChunkCarverInput<'_>,
    vertical_rotation: FloatProvider,
    shape: CanyonShapeConfiguration,
) -> usize {
    let max_distance = (4 * 2 - 1) << 4;
    let x = f64::from(input.source_chunk_x * 16 + input.random.next_i32_bound(16));
    let y = f64::from(sample_carver_y(
        input.carver.y,
        input.height_context,
        input.random,
    ));
    let z = f64::from(input.source_chunk_z * 16 + input.random.next_i32_bound(16));
    let horizontal_rotation = input.random.next_f32() * std::f32::consts::TAU;
    let vertical_rotation = sample_float_provider(vertical_rotation, input.random);
    let y_scale = sample_float_provider(input.carver.y_scale, input.random) as f64;
    let thickness = sample_float_provider(shape.thickness, input.random);
    let distance =
        (max_distance as f32 * sample_float_provider(shape.distance_factor, input.random)) as i32;
    carve_canyon_tunnel_into_chunk(CanyonTunnelChunkInput {
        chunk: input.chunk,
        height_context: input.height_context,
        carver: input.carver,
        chunk_min_x: input.target_chunk_min_x,
        chunk_min_z: input.target_chunk_min_z,
        tunnel_seed: input.random.next_i64(),
        x,
        y,
        z,
        thickness,
        horizontal_rotation,
        vertical_rotation,
        distance,
        y_scale,
        shape: &shape,
        mask: input.mask,
        settings: input.settings,
        noise_chunk: input.noise_chunk,
        aquifer: input.aquifer,
    })
}

struct CarveEllipsoidChunkInput<'a> {
    chunk: &'a mut LevelChunk,
    height_context: WorldGenerationHeightContext,
    carver: &'a ConfiguredCarver,
    chunk_min_x: i32,
    chunk_min_z: i32,
    x: f64,
    y: f64,
    z: f64,
    horizontal_radius: f64,
    vertical_radius: f64,
    skip_model: CarverSkipModel<'a>,
    mask: &'a mut Vec<usize>,
    settings: &'a NoiseGeneratorSettings,
    noise_chunk: &'a NoiseChunk,
    aquifer: Option<&'a mut NoiseBasedAquifer>,
}

fn carve_ellipsoid_into_chunk(mut input: CarveEllipsoidChunkInput<'_>) -> usize {
    let positions = carver_ellipsoid_candidate_positions(CarverEllipsoidInput {
        chunk_min_x: input.chunk_min_x,
        chunk_min_z: input.chunk_min_z,
        height_context: input.height_context,
        upgrading: false,
        x: input.x,
        y: input.y,
        z: input.z,
        horizontal_radius: input.horizontal_radius,
        vertical_radius: input.vertical_radius,
        existing_mask_indices: input.mask,
        debug_enabled: false,
        skip_model: input.skip_model,
    });
    let mut carved = 0;
    for pos in positions {
        let Some(block) = input.chunk.get_block_state_name(pos.x, pos.y, pos.z) else {
            continue;
        };
        let Some(block) = carver_static_block_name(block) else {
            continue;
        };
        let (aquifer_state, should_schedule_fluid_update) = carver_aquifer_carve_state(
            input.carver,
            input.height_context,
            input.settings,
            input.noise_chunk,
            input.aquifer.as_deref_mut(),
            pos,
        );
        let block_input = CarverBlockInput {
            pos,
            block,
            was_masked: carver_mask_index(pos.x, pos.y, pos.z, input.height_context.min_y)
                .is_some_and(|index| input.mask.contains(&index)),
            aquifer_state,
            should_schedule_fluid_update,
            debug_enabled: false,
        };
        let outcome = carver_carve_block(input.carver, input.height_context, block_input);
        if carver_trace_matches(pos) {
            eprintln!(
                "[carver-trace] carver={} pos=({},{},{}) block={} was_masked={} aquifer_state={:?} outcome={:?}",
                input.carver.id,
                pos.x,
                pos.y,
                pos.z,
                block,
                block_input.was_masked,
                block_input.aquifer_state,
                outcome.as_ref().map(|outcome| outcome.state)
            );
        }
        let Some(outcome) = outcome else {
            continue;
        };
        input
            .chunk
            .set_block_state(pos.x, pos.y, pos.z, outcome.state);
        input.mask.push(outcome.mask_index);
        carved += 1;
    }
    carved
}

fn carver_trace_matches(pos: BlockPos) -> bool {
    let Ok(raw) = std::env::var("VIBECRAFT_WORLDGEN_CARVER_TRACE") else {
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

pub(super) struct CaveTunnelChunkInput<'a> {
    pub chunk: &'a mut LevelChunk,
    pub height_context: WorldGenerationHeightContext,
    pub carver: &'a ConfiguredCarver,
    pub chunk_min_x: i32,
    pub chunk_min_z: i32,
    pub tunnel_seed: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub horizontal_radius_multiplier: f64,
    pub vertical_radius_multiplier: f64,
    pub thickness: f32,
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
    pub start_step: i32,
    pub distance: i32,
    pub y_scale: f64,
    pub floor_level: f64,
    pub mask: &'a mut Vec<usize>,
    pub depth: usize,
    pub settings: &'a NoiseGeneratorSettings,
    pub noise_chunk: &'a NoiseChunk,
    pub aquifer: Option<&'a mut NoiseBasedAquifer>,
}

struct CaveTunnelSplitState {
    x: f64,
    y: f64,
    z: f64,
    horizontal_rotation: f32,
    vertical_rotation: f32,
    current_step: i32,
}

fn carve_cave_tunnel_split_branches(
    input: &mut CaveTunnelChunkInput<'_>,
    random: &mut LegacyRandom,
    state: CaveTunnelSplitState,
) -> usize {
    let left_seed = random.next_i64();
    let left_thickness = random.next_f32() * 0.5 + 0.5;
    let mut carved = carve_cave_tunnel_into_chunk(CaveTunnelChunkInput {
        chunk: &mut *input.chunk,
        height_context: input.height_context,
        carver: input.carver,
        chunk_min_x: input.chunk_min_x,
        chunk_min_z: input.chunk_min_z,
        tunnel_seed: left_seed,
        x: state.x,
        y: state.y,
        z: state.z,
        horizontal_radius_multiplier: input.horizontal_radius_multiplier,
        vertical_radius_multiplier: input.vertical_radius_multiplier,
        thickness: left_thickness,
        horizontal_rotation: state.horizontal_rotation - std::f32::consts::FRAC_PI_2,
        vertical_rotation: state.vertical_rotation / 3.0,
        start_step: state.current_step,
        distance: input.distance,
        y_scale: 1.0,
        floor_level: input.floor_level,
        mask: &mut *input.mask,
        depth: input.depth + 1,
        settings: input.settings,
        noise_chunk: input.noise_chunk,
        aquifer: input.aquifer.as_deref_mut(),
    });
    let right_seed = random.next_i64();
    let right_thickness = random.next_f32() * 0.5 + 0.5;
    carved += carve_cave_tunnel_into_chunk(CaveTunnelChunkInput {
        chunk: &mut *input.chunk,
        height_context: input.height_context,
        carver: input.carver,
        chunk_min_x: input.chunk_min_x,
        chunk_min_z: input.chunk_min_z,
        tunnel_seed: right_seed,
        x: state.x,
        y: state.y,
        z: state.z,
        horizontal_radius_multiplier: input.horizontal_radius_multiplier,
        vertical_radius_multiplier: input.vertical_radius_multiplier,
        thickness: right_thickness,
        horizontal_rotation: state.horizontal_rotation + std::f32::consts::FRAC_PI_2,
        vertical_rotation: state.vertical_rotation / 3.0,
        start_step: state.current_step,
        distance: input.distance,
        y_scale: 1.0,
        floor_level: input.floor_level,
        mask: &mut *input.mask,
        depth: input.depth + 1,
        settings: input.settings,
        noise_chunk: input.noise_chunk,
        aquifer: input.aquifer.as_deref_mut(),
    });
    carved
}

pub(super) fn carve_cave_tunnel_into_chunk(mut input: CaveTunnelChunkInput<'_>) -> usize {
    if input.distance < 2 || input.depth > 8 {
        return 0;
    }
    let mut random = LegacyRandom::new(input.tunnel_seed);
    let split_point = random.next_i32_bound(input.distance / 2) + input.distance / 4;
    let steep = random.next_i32_bound(6) == 0;
    let mut y_rota = 0.0_f32;
    let mut x_rota = 0.0_f32;
    let chunk_middle_x = f64::from(input.chunk_min_x + 8);
    let chunk_middle_z = f64::from(input.chunk_min_z + 8);
    let mut x = input.x;
    let mut y = input.y;
    let mut z = input.z;
    let mut horizontal_rotation = input.horizontal_rotation;
    let mut vertical_rotation = input.vertical_rotation;
    let mut carved = 0;

    for current_step in input.start_step..input.distance {
        let horizontal_radius = 1.5
            + f64::from((std::f32::consts::PI * current_step as f32 / input.distance as f32).sin())
                * f64::from(input.thickness);
        let vertical_radius = horizontal_radius * input.y_scale;
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

        if current_step == split_point && input.thickness > 1.0 {
            carved += carve_cave_tunnel_split_branches(
                &mut input,
                &mut random,
                CaveTunnelSplitState {
                    x,
                    y,
                    z,
                    horizontal_rotation,
                    vertical_rotation,
                    current_step,
                },
            );
            return carved;
        }

        if random.next_i32_bound(4) == 0 {
            continue;
        }
        let can_reach = (x - chunk_middle_x) * (x - chunk_middle_x)
            + (z - chunk_middle_z) * (z - chunk_middle_z)
            - f64::from(input.distance - current_step).powi(2)
            <= f64::from(input.thickness + 18.0).powi(2);
        if !can_reach {
            return carved;
        }
        carved += carve_ellipsoid_into_chunk(CarveEllipsoidChunkInput {
            chunk: input.chunk,
            height_context: input.height_context,
            carver: input.carver,
            chunk_min_x: input.chunk_min_x,
            chunk_min_z: input.chunk_min_z,
            x,
            y,
            z,
            horizontal_radius: horizontal_radius * input.horizontal_radius_multiplier,
            vertical_radius: vertical_radius * input.vertical_radius_multiplier,
            skip_model: CarverSkipModel::Cave {
                floor_level: input.floor_level,
            },
            mask: input.mask,
            settings: input.settings,
            noise_chunk: input.noise_chunk,
            aquifer: input.aquifer.as_deref_mut(),
        });
    }

    carved
}

pub(super) struct CanyonTunnelChunkInput<'a> {
    pub chunk: &'a mut LevelChunk,
    pub height_context: WorldGenerationHeightContext,
    pub carver: &'a ConfiguredCarver,
    pub chunk_min_x: i32,
    pub chunk_min_z: i32,
    pub tunnel_seed: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub thickness: f32,
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
    pub distance: i32,
    pub y_scale: f64,
    pub shape: &'a CanyonShapeConfiguration,
    pub mask: &'a mut Vec<usize>,
    pub settings: &'a NoiseGeneratorSettings,
    pub noise_chunk: &'a NoiseChunk,
    pub aquifer: Option<&'a mut NoiseBasedAquifer>,
}

pub(super) fn carve_canyon_tunnel_into_chunk(mut input: CanyonTunnelChunkInput<'_>) -> usize {
    if input.distance < 1 {
        return 0;
    }
    let mut random = LegacyRandom::new(input.tunnel_seed);
    let width_factors = canyon_width_factors_from_random(
        input.height_context.height,
        input.shape.width_smoothness,
        &mut random,
    );
    let chunk_middle_x = f64::from(input.chunk_min_x + 8);
    let chunk_middle_z = f64::from(input.chunk_min_z + 8);
    let mut y_rota = 0.0_f32;
    let mut x_rota = 0.0_f32;
    let mut x = input.x;
    let mut y = input.y;
    let mut z = input.z;
    let mut horizontal_rotation = input.horizontal_rotation;
    let mut vertical_rotation = input.vertical_rotation;
    let mut carved = 0;

    for current_step in 0..input.distance {
        let base_horizontal_radius = 1.5
            + f64::from((current_step as f32 * std::f32::consts::PI / input.distance as f32).sin())
                * f64::from(input.thickness);
        let horizontal_radius = base_horizontal_radius
            * f64::from(sample_float_provider(
                input.shape.horizontal_radius_factor,
                &mut random,
            ));
        let vertical_radius = canyon_vertical_radius(
            input.shape.vertical_radius_default_factor,
            input.shape.vertical_radius_center_factor,
            base_horizontal_radius * input.y_scale,
            input.distance,
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
            - f64::from(input.distance - current_step).powi(2)
            <= f64::from(input.thickness + 18.0).powi(2);
        if !can_reach {
            return carved;
        }
        carved += carve_ellipsoid_into_chunk(CarveEllipsoidChunkInput {
            chunk: input.chunk,
            height_context: input.height_context,
            carver: input.carver,
            chunk_min_x: input.chunk_min_x,
            chunk_min_z: input.chunk_min_z,
            x,
            y,
            z,
            horizontal_radius,
            vertical_radius,
            skip_model: CarverSkipModel::Canyon {
                width_factors: &width_factors,
            },
            mask: input.mask,
            settings: input.settings,
            noise_chunk: input.noise_chunk,
            aquifer: input.aquifer.as_deref_mut(),
        });
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
