use super::*;

pub fn configured_carver(id: &str) -> Option<&'static ConfiguredCarver> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CONFIGURED_CARVERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

fn parse_cave_carver_shape_json(
    id: &str,
    config: &serde_json::Map<String, serde_json::Value>,
) -> Result<CarverShape, String> {
    Ok(CarverShape::Cave {
        horizontal_radius_multiplier: parse_float_provider_json(
            config
                .get("horizontal_radius_multiplier")
                .ok_or_else(|| format!("configured carver {id} missing horizontal radius"))?,
        )?,
        vertical_radius_multiplier: parse_float_provider_json(
            config
                .get("vertical_radius_multiplier")
                .ok_or_else(|| format!("configured carver {id} missing vertical radius"))?,
        )?,
        floor_level: parse_float_provider_json(
            config
                .get("floor_level")
                .ok_or_else(|| format!("configured carver {id} missing floor_level"))?,
        )?,
    })
}

fn parse_canyon_shape_configuration_json(
    id: &str,
    shape: &serde_json::Map<String, serde_json::Value>,
) -> Result<CanyonShapeConfiguration, String> {
    Ok(CanyonShapeConfiguration {
        distance_factor: parse_float_provider_json(
            shape
                .get("distance_factor")
                .ok_or_else(|| format!("configured carver {id} missing distance_factor"))?,
        )?,
        thickness: parse_float_provider_json(
            shape
                .get("thickness")
                .ok_or_else(|| format!("configured carver {id} missing thickness"))?,
        )?,
        width_smoothness: json_i32(shape, "width_smoothness")?,
        horizontal_radius_factor: parse_float_provider_json(
            shape.get("horizontal_radius_factor").ok_or_else(|| {
                format!("configured carver {id} missing horizontal_radius_factor")
            })?,
        )?,
        vertical_radius_default_factor: json_f32(shape, "vertical_radius_default_factor")?,
        vertical_radius_center_factor: json_f32(shape, "vertical_radius_center_factor")?,
    })
}

fn parse_canyon_carver_shape_json(
    id: &str,
    config: &serde_json::Map<String, serde_json::Value>,
) -> Result<CarverShape, String> {
    let shape = config
        .get("shape")
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("configured carver {id} missing canyon shape"))?;
    Ok(CarverShape::Canyon {
        vertical_rotation: parse_float_provider_json(
            config
                .get("vertical_rotation")
                .ok_or_else(|| format!("configured carver {id} missing vertical_rotation"))?,
        )?,
        shape: parse_canyon_shape_configuration_json(id, shape)?,
    })
}

fn parse_carver_shape_json(
    id: &str,
    carver_type: WorldCarverType,
    config: &serde_json::Map<String, serde_json::Value>,
) -> Result<CarverShape, String> {
    match carver_type {
        WorldCarverType::Cave | WorldCarverType::NetherCave => {
            parse_cave_carver_shape_json(id, config)
        }
        WorldCarverType::Canyon => parse_canyon_carver_shape_json(id, config),
    }
}

pub fn parse_configured_carver_from_json(
    id: &'static str,
    value: &serde_json::Value,
) -> Result<ConfiguredCarver, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "configured carver must be a JSON object".to_string())?;
    let carver_type = object
        .get("type")
        .and_then(|v| v.as_str())
        .and_then(world_carver_type)
        .ok_or_else(|| format!("configured carver {id} has unknown type"))?;
    let config = object
        .get("config")
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("configured carver {id} missing config object"))?;
    let probability = json_f32(config, "probability")?;
    let y = parse_height_range_json(
        config
            .get("y")
            .ok_or_else(|| format!("configured carver {id} missing y provider"))?,
    )?;
    let y_scale = parse_float_provider_json(
        config
            .get("yScale")
            .ok_or_else(|| format!("configured carver {id} missing yScale provider"))?,
    )?;
    let lava_level = parse_vertical_anchor_from_json(
        config
            .get("lava_level")
            .ok_or_else(|| format!("configured carver {id} missing lava_level"))?,
    )?;
    let replaceable_tag = config
        .get("replaceable")
        .and_then(|v| v.as_str())
        .and_then(vanilla_carver_replaceable_tag)
        .ok_or_else(|| format!("configured carver {id} has unknown replaceable tag"))?;
    let debug = parse_carver_debug_settings_json(config.get("debug_settings"))?;
    let shape = parse_carver_shape_json(id, carver_type, config)?;

    Ok(ConfiguredCarver {
        id,
        carver_type,
        probability,
        y,
        y_scale,
        lava_level,
        debug,
        replaceable_tag,
        shape,
    })
}

fn parse_height_range_json(value: &serde_json::Value) -> Result<HeightRange, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "height range provider must be an object".to_string())?;
    let provider_type = object
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("minecraft:constant")
        .strip_prefix("minecraft:")
        .unwrap_or_else(|| {
            object
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("constant")
        });
    match provider_type {
        "uniform" => Ok(HeightRange {
            min: parse_vertical_anchor_from_json(
                object
                    .get("min_inclusive")
                    .ok_or_else(|| "uniform height range missing min_inclusive".to_string())?,
            )?,
            max: parse_vertical_anchor_from_json(
                object
                    .get("max_inclusive")
                    .ok_or_else(|| "uniform height range missing max_inclusive".to_string())?,
            )?,
        }),
        "constant" => {
            let value = parse_vertical_anchor_from_json(
                object
                    .get("value")
                    .ok_or_else(|| "constant height range missing value".to_string())?,
            )?;
            Ok(HeightRange {
                min: value,
                max: value,
            })
        }
        other => Err(format!("unsupported carver height provider {other}")),
    }
}

fn parse_float_provider_json(value: &serde_json::Value) -> Result<FloatProvider, String> {
    if let Some(number) = value.as_f64() {
        return Ok(FloatProvider::Constant(number as f32));
    }
    let object = value
        .as_object()
        .ok_or_else(|| "float provider must be a number or object".to_string())?;
    let raw_provider_type = object
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "float provider object missing type".to_string())?;
    let provider_type = raw_provider_type
        .strip_prefix("minecraft:")
        .unwrap_or(raw_provider_type);
    match provider_type {
        "uniform" => Ok(FloatProvider::Uniform {
            min: json_f32(object, "min_inclusive")?,
            max: json_f32(object, "max_exclusive")?,
        }),
        "trapezoid" => Ok(FloatProvider::Trapezoid {
            min: json_f32(object, "min")?,
            max: json_f32(object, "max")?,
            plateau: json_f32(object, "plateau")?,
        }),
        other => Err(format!("unsupported float provider {other}")),
    }
}

fn parse_carver_debug_settings_json(
    value: Option<&serde_json::Value>,
) -> Result<CarverDebugSettings, String> {
    let Some(object) = value.and_then(|v| v.as_object()) else {
        return Ok(CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:air",
        });
    };
    let barrier_state = object
        .get("air_state")
        .and_then(|state| state.get("Name"))
        .and_then(|name| name.as_str())
        .and_then(vanilla_carver_debug_state)
        .ok_or_else(|| "carver debug settings missing air_state.Name".to_string())?;
    Ok(CarverDebugSettings {
        enabled: false,
        barrier_state,
    })
}

fn vanilla_carver_replaceable_tag(tag: &str) -> Option<&'static str> {
    match tag {
        "#minecraft:overworld_carver_replaceables" => {
            Some("#minecraft:overworld_carver_replaceables")
        }
        "#minecraft:nether_carver_replaceables" => Some("#minecraft:nether_carver_replaceables"),
        _ => None,
    }
}

fn vanilla_carver_debug_state(name: &str) -> Option<&'static str> {
    match name {
        "minecraft:crimson_button" => Some("minecraft:crimson_button"),
        "minecraft:oak_button" => Some("minecraft:oak_button"),
        "minecraft:warped_button" => Some("minecraft:warped_button"),
        _ => None,
    }
}

fn json_f32(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<f32, String> {
    object
        .get(field)
        .and_then(|value| value.as_f64())
        .map(|value| value as f32)
        .ok_or_else(|| format!("missing numeric field {field}"))
}

fn json_i32(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<i32, String> {
    object
        .get(field)
        .and_then(|value| value.as_i64())
        .map(|value| value as i32)
        .ok_or_else(|| format!("missing integer field {field}"))
}

pub fn world_carver_type(id: &str) -> Option<WorldCarverType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLD_CARVER_TYPES
        .iter()
        .copied()
        .find(|carver| carver.id().strip_prefix("minecraft:") == Some(name))
}

pub fn carver_is_start_chunk(carver: &ConfiguredCarver, random_next_float: f32) -> bool {
    random_next_float <= carver.probability
}

pub fn carver_mask_index(x: i32, y: i32, z: i32, min_y: i32) -> Option<usize> {
    if y < min_y {
        return None;
    }
    Some(((x & 15) | ((z & 15) << 4) | ((y - min_y) << 8)) as usize)
}

pub fn carver_mask_position(
    index: usize,
    chunk_min_x: i32,
    chunk_min_z: i32,
    min_y: i32,
) -> BlockPos {
    BlockPos {
        x: chunk_min_x + (index as i32 & 15),
        y: min_y + (index as i32 >> 8),
        z: chunk_min_z + ((index as i32 >> 4) & 15),
    }
}

pub fn carver_can_replace_block(carver: &ConfiguredCarver, block: &str) -> bool {
    match carver.replaceable_tag {
        "#minecraft:overworld_carver_replaceables" => matches!(
            block,
            "minecraft:stone"
                | "minecraft:granite"
                | "minecraft:diorite"
                | "minecraft:andesite"
                | "minecraft:tuff"
                | "minecraft:calcite"
                | "minecraft:dirt"
                | "minecraft:grass_block"
                | "minecraft:podzol"
                | "minecraft:mycelium"
                | "minecraft:coarse_dirt"
                | "minecraft:rooted_dirt"
                | "minecraft:deepslate"
                | "minecraft:sandstone"
                | "minecraft:red_sandstone"
                | "minecraft:sand"
                | "minecraft:red_sand"
                | "minecraft:clay"
                | "minecraft:gravel"
                | "minecraft:water"
                | "minecraft:ice"
                | "minecraft:packed_ice"
                | "minecraft:snow_block"
        ),
        "#minecraft:nether_carver_replaceables" => matches!(
            block,
            "minecraft:netherrack"
                | "minecraft:basalt"
                | "minecraft:blackstone"
                | "minecraft:soul_sand"
                | "minecraft:soul_soil"
                | "minecraft:crimson_nylium"
                | "minecraft:warped_nylium"
                | "minecraft:nether_wart_block"
                | "minecraft:warped_wart_block"
        ),
        _ => false,
    }
}

pub fn carver_effective_lava_y(
    carver: &ConfiguredCarver,
    height_context: WorldGenerationHeightContext,
) -> i32 {
    match carver.carver_type {
        WorldCarverType::NetherCave => height_context.min_y + 31,
        _ => carver.lava_level.resolve_y(height_context),
    }
}

pub fn nether_carver_thickness(first_float: f32, second_float: f32) -> f32 {
    (first_float * 2.0 + second_float) * 2.0
}

pub fn carver_tunnel_y_scale(carver_type: WorldCarverType) -> f64 {
    match carver_type {
        WorldCarverType::NetherCave => 5.0,
        WorldCarverType::Cave | WorldCarverType::Canyon => 1.0,
    }
}

pub fn carver_cave_bound(carver_type: WorldCarverType) -> i32 {
    match carver_type {
        WorldCarverType::NetherCave => 10,
        WorldCarverType::Cave | WorldCarverType::Canyon => 15,
    }
}

pub fn carver_carve_block(
    carver: &ConfiguredCarver,
    height_context: WorldGenerationHeightContext,
    input: CarverBlockInput,
) -> Option<CarverBlockOutcome> {
    if input.was_masked && !input.debug_enabled {
        return None;
    }
    if !carver_can_replace_block(carver, input.block) && !input.debug_enabled {
        return None;
    }
    if carver.carver_type == WorldCarverType::NetherCave && !input.debug_enabled {
        return Some(CarverBlockOutcome {
            pos: input.pos,
            state: if input.pos.y <= carver_effective_lava_y(carver, height_context) {
                "minecraft:lava"
            } else {
                "minecraft:cave_air"
            },
            mask_index: carver_mask_index(
                input.pos.x,
                input.pos.y,
                input.pos.z,
                height_context.min_y,
            )?,
            mark_postprocessing: false,
        });
    }
    let state = if input.pos.y <= carver_effective_lava_y(carver, height_context) {
        "minecraft:lava"
    } else {
        match input.aquifer_state {
            Some("minecraft:air") if input.debug_enabled => "minecraft:orange_stained_glass",
            Some("minecraft:water") if input.debug_enabled => "minecraft:blue_stained_glass",
            Some("minecraft:lava") if input.debug_enabled => "minecraft:red_stained_glass",
            Some(state) => state,
            None if input.debug_enabled => carver.debug.barrier_state,
            None => return None,
        }
    };
    Some(CarverBlockOutcome {
        pos: input.pos,
        state,
        mask_index: carver_mask_index(input.pos.x, input.pos.y, input.pos.z, height_context.min_y)?,
        mark_postprocessing: input.should_schedule_fluid_update
            && matches!(state, "minecraft:water" | "minecraft:lava"),
    })
}

pub struct CarverEllipsoidInput<'a> {
    pub chunk_min_x: i32,
    pub chunk_min_z: i32,
    pub height_context: WorldGenerationHeightContext,
    pub upgrading: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub horizontal_radius: f64,
    pub vertical_radius: f64,
    pub existing_mask_indices: &'a [usize],
    pub debug_enabled: bool,
    pub skip_model: CarverSkipModel<'a>,
}

pub fn carver_ellipsoid_candidate_positions(input: CarverEllipsoidInput<'_>) -> Vec<BlockPos> {
    let chunk_middle_x = input.chunk_min_x + 8;
    let chunk_middle_z = input.chunk_min_z + 8;
    let max_delta = 16.0 + input.horizontal_radius * 2.0;
    if (input.x - f64::from(chunk_middle_x)).abs() > max_delta
        || (input.z - f64::from(chunk_middle_z)).abs() > max_delta
    {
        return Vec::new();
    }

    let min_x_index =
        ((input.x - input.horizontal_radius).floor() as i32 - input.chunk_min_x - 1).max(0);
    let max_x_index =
        ((input.x + input.horizontal_radius).floor() as i32 - input.chunk_min_x).min(15);
    let min_y =
        ((input.y - input.vertical_radius).floor() as i32 - 1).max(input.height_context.min_y + 1);
    let protected_blocks_on_top = if input.upgrading { 0 } else { 7 };
    let max_y = ((input.y + input.vertical_radius).floor() as i32 + 1).min(
        input.height_context.min_y + input.height_context.height - 1 - protected_blocks_on_top,
    );
    let min_z_index =
        ((input.z - input.horizontal_radius).floor() as i32 - input.chunk_min_z - 1).max(0);
    let max_z_index =
        ((input.z + input.horizontal_radius).floor() as i32 - input.chunk_min_z).min(15);
    let mut positions = Vec::new();

    for x_index in min_x_index..=max_x_index {
        let world_x = input.chunk_min_x + x_index;
        let xd = (f64::from(world_x) + 0.5 - input.x) / input.horizontal_radius;
        for z_index in min_z_index..=max_z_index {
            let world_z = input.chunk_min_z + z_index;
            let zd = (f64::from(world_z) + 0.5 - input.z) / input.horizontal_radius;
            if xd * xd + zd * zd >= 1.0 {
                continue;
            }
            for world_y in (min_y + 1..=max_y).rev() {
                let yd = (f64::from(world_y) - 0.5 - input.y) / input.vertical_radius;
                if carver_should_skip_ellipsoid_cell(
                    input.skip_model,
                    input.height_context,
                    xd,
                    yd,
                    zd,
                    world_y,
                ) {
                    continue;
                }
                let Some(mask_index) =
                    carver_mask_index(world_x, world_y, world_z, input.height_context.min_y)
                else {
                    continue;
                };
                if input.debug_enabled || !input.existing_mask_indices.contains(&mask_index) {
                    positions.push(BlockPos {
                        x: world_x,
                        y: world_y,
                        z: world_z,
                    });
                }
            }
        }
    }
    positions
}

pub fn carver_should_skip_ellipsoid_cell(
    skip_model: CarverSkipModel<'_>,
    height_context: WorldGenerationHeightContext,
    xd: f64,
    yd: f64,
    zd: f64,
    y: i32,
) -> bool {
    match skip_model {
        CarverSkipModel::None => false,
        CarverSkipModel::Cave { floor_level } => {
            yd <= floor_level || xd * xd + yd * yd + zd * zd >= 1.0
        }
        CarverSkipModel::Canyon { width_factors } => {
            let y_index = y - height_context.min_y;
            let width_factor = width_factors
                .get((y_index - 1).max(0) as usize)
                .copied()
                .unwrap_or(1.0);
            (xd * xd + zd * zd) * f64::from(width_factor) + yd * yd / 6.0 >= 1.0
        }
    }
}

pub fn cave_carver_cave_count(
    cave_bound: i32,
    first_roll: i32,
    second_roll: i32,
    third_roll: i32,
) -> i32 {
    if cave_bound <= 0 {
        return 0;
    }
    let first = first_roll.rem_euclid(cave_bound);
    let second = second_roll.rem_euclid(first + 1);
    third_roll.rem_euclid(second + 1)
}

pub fn sample_cave_carver_cave_count(
    carver_type: WorldCarverType,
    random: &mut LegacyRandom,
) -> i32 {
    let cave_bound = carver_cave_bound(carver_type);
    if cave_bound <= 0 {
        return 0;
    }
    let first = random.next_i32_bound(cave_bound);
    let second = random.next_i32_bound(first + 1);
    random.next_i32_bound(second + 1)
}

pub fn cave_carver_thickness(
    first_float: f32,
    second_float: f32,
    rare_roll: i32,
    rare_first_float: f32,
    rare_second_float: f32,
) -> f32 {
    let mut thickness = first_float * 2.0 + second_float;
    if rare_roll.rem_euclid(10) == 0 {
        thickness *= rare_first_float * rare_second_float * 3.0 + 1.0;
    }
    thickness
}

pub fn cave_room_radii(thickness: f32, y_scale: f64) -> (f64, f64) {
    let horizontal_radius = 1.5 + f64::from(thickness);
    (horizontal_radius, horizontal_radius * y_scale)
}

pub struct CaveTunnelInput<'a> {
    pub chunk_middle_x: f64,
    pub chunk_middle_z: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub thickness: f32,
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
    pub distance: i32,
    pub y_scale: f64,
    pub horizontal_radius_multiplier: f64,
    pub vertical_radius_multiplier: f64,
    pub random_quarter_skip_rolls: &'a [i32],
    pub rotation_rolls: &'a [(f32, f32, f32, f32, f32, f32)],
}

pub fn cave_tunnel_steps(input: CaveTunnelInput<'_>) -> Vec<CaveTunnelStep> {
    let mut steps = Vec::new();
    let mut y_rota = 0.0_f32;
    let mut x_rota = 0.0_f32;
    let mut x = input.x;
    let mut y = input.y;
    let mut z = input.z;
    let mut horizontal_rotation = input.horizontal_rotation;
    let mut vertical_rotation = input.vertical_rotation;
    for current_step in 0..input.distance {
        let horizontal_radius = 1.5
            + f64::from((std::f32::consts::PI * current_step as f32 / input.distance as f32).sin())
                * f64::from(input.thickness);
        let vertical_radius = horizontal_radius * input.y_scale;
        let cos_x = vertical_rotation.cos();
        x += f64::from(horizontal_rotation.cos() * cos_x);
        y += f64::from(vertical_rotation.sin());
        z += f64::from(horizontal_rotation.sin() * cos_x);
        vertical_rotation *= 0.7;
        vertical_rotation += x_rota * 0.1;
        horizontal_rotation += y_rota * 0.1;
        x_rota *= 0.9;
        y_rota *= 0.75;
        let (xr_a, xr_b, xr_c, yr_a, yr_b, yr_c) = input
            .rotation_rolls
            .get(current_step as usize)
            .copied()
            .unwrap_or((0.5, 0.5, 0.0, 0.5, 0.5, 0.0));
        x_rota += (xr_a - xr_b) * xr_c * 2.0;
        y_rota += (yr_a - yr_b) * yr_c * 4.0;
        let carve = input
            .random_quarter_skip_rolls
            .get(current_step as usize)
            .copied()
            .unwrap_or(1)
            .rem_euclid(4)
            != 0;
        let can_reach = (x - input.chunk_middle_x) * (x - input.chunk_middle_x)
            + (z - input.chunk_middle_z) * (z - input.chunk_middle_z)
            - f64::from(input.distance - current_step).powi(2)
            <= f64::from(input.thickness + 18.0).powi(2);
        steps.push(CaveTunnelStep {
            step: current_step,
            x,
            y,
            z,
            horizontal_radius: horizontal_radius * input.horizontal_radius_multiplier,
            vertical_radius: vertical_radius * input.vertical_radius_multiplier,
            can_reach,
            carve,
        });
        if carve && !can_reach {
            break;
        }
    }
    steps
}

pub struct CaveTunnelSplitInput<'a> {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub thickness: f32,
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
    pub distance: i32,
    pub split_roll: i32,
    pub steep_roll: i32,
    pub left_thickness_roll: f32,
    pub right_thickness_roll: f32,
    pub rotation_rolls: &'a [(f32, f32, f32, f32, f32, f32)],
}

pub fn cave_tunnel_split_branch(input: CaveTunnelSplitInput<'_>) -> Option<CaveTunnelBranch> {
    if input.distance < 2 || input.thickness <= 1.0 {
        return None;
    }
    let split_point = input.split_roll.rem_euclid(input.distance / 2) + input.distance / 4;
    let steep = input.steep_roll.rem_euclid(6) == 0;
    let mut y_rota = 0.0_f32;
    let mut x_rota = 0.0_f32;
    let mut x = input.x;
    let mut y = input.y;
    let mut z = input.z;
    let mut horizontal_rotation = input.horizontal_rotation;
    let mut vertical_rotation = input.vertical_rotation;

    for current_step in 0..input.distance {
        let cos_x = vertical_rotation.cos();
        x += f64::from(horizontal_rotation.cos() * cos_x);
        y += f64::from(vertical_rotation.sin());
        z += f64::from(horizontal_rotation.sin() * cos_x);
        vertical_rotation *= if steep { 0.92 } else { 0.7 };
        vertical_rotation += x_rota * 0.1;
        horizontal_rotation += y_rota * 0.1;
        x_rota *= 0.9;
        y_rota *= 0.75;
        let (xr_a, xr_b, xr_c, yr_a, yr_b, yr_c) = input
            .rotation_rolls
            .get(current_step as usize)
            .copied()
            .unwrap_or((0.5, 0.5, 0.0, 0.5, 0.5, 0.0));
        x_rota += (xr_a - xr_b) * xr_c * 2.0;
        y_rota += (yr_a - yr_b) * yr_c * 4.0;

        if current_step == split_point {
            return Some(CaveTunnelBranch {
                split_step: current_step,
                x,
                y,
                z,
                left_thickness: input.left_thickness_roll * 0.5 + 0.5,
                right_thickness: input.right_thickness_roll * 0.5 + 0.5,
                left_horizontal_rotation: horizontal_rotation - std::f32::consts::FRAC_PI_2,
                right_horizontal_rotation: horizontal_rotation + std::f32::consts::FRAC_PI_2,
                vertical_rotation: vertical_rotation / 3.0,
                distance: input.distance,
            });
        }
    }

    None
}

pub struct CanyonTunnelInput<'a> {
    pub chunk_middle_x: f64,
    pub chunk_middle_z: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub thickness: f32,
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
    pub distance: i32,
    pub y_scale: f64,
    pub default_vertical_factor: f32,
    pub center_vertical_factor: f32,
    pub random_quarter_skip_rolls: &'a [i32],
    pub horizontal_radius_factor_rolls: &'a [f32],
    pub vertical_radius_rolls: &'a [f32],
    pub rotation_rolls: &'a [(f32, f32, f32, f32, f32, f32)],
}

pub fn canyon_tunnel_steps(input: CanyonTunnelInput<'_>) -> Vec<CaveTunnelStep> {
    let mut steps = Vec::new();
    let mut y_rota = 0.0_f32;
    let mut x_rota = 0.0_f32;
    let mut x = input.x;
    let mut y = input.y;
    let mut z = input.z;
    let mut horizontal_rotation = input.horizontal_rotation;
    let mut vertical_rotation = input.vertical_rotation;
    for current_step in 0..input.distance {
        let base_horizontal_radius = 1.5
            + f64::from((current_step as f32 * std::f32::consts::PI / input.distance as f32).sin())
                * f64::from(input.thickness);
        let horizontal_factor = input
            .horizontal_radius_factor_rolls
            .get(current_step as usize)
            .copied()
            .unwrap_or(1.0);
        let vertical_roll = input
            .vertical_radius_rolls
            .get(current_step as usize)
            .copied()
            .unwrap_or(1.0);
        let horizontal_radius = base_horizontal_radius * f64::from(horizontal_factor);
        let vertical_radius = canyon_vertical_radius(
            input.default_vertical_factor,
            input.center_vertical_factor,
            base_horizontal_radius * input.y_scale,
            input.distance,
            current_step,
            vertical_roll,
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
        let (xr_a, xr_b, xr_c, yr_a, yr_b, yr_c) = input
            .rotation_rolls
            .get(current_step as usize)
            .copied()
            .unwrap_or((0.5, 0.5, 0.0, 0.5, 0.5, 0.0));
        x_rota += (xr_a - xr_b) * xr_c * 2.0;
        y_rota += (yr_a - yr_b) * yr_c * 4.0;
        let carve = input
            .random_quarter_skip_rolls
            .get(current_step as usize)
            .copied()
            .unwrap_or(1)
            .rem_euclid(4)
            != 0;
        let can_reach = (x - input.chunk_middle_x) * (x - input.chunk_middle_x)
            + (z - input.chunk_middle_z) * (z - input.chunk_middle_z)
            - f64::from(input.distance - current_step).powi(2)
            <= f64::from(input.thickness + 18.0).powi(2);
        steps.push(CaveTunnelStep {
            step: current_step,
            x,
            y,
            z,
            horizontal_radius,
            vertical_radius,
            can_reach,
            carve,
        });
        if carve && !can_reach {
            break;
        }
    }
    steps
}

pub fn canyon_width_factors(depth: i32, width_smoothness: i32, rolls: &[(i32, f32)]) -> Vec<f32> {
    let mut factors = Vec::new();
    let mut width_factor = 1.0_f32;
    for y_index in 0..depth.max(0) {
        let (reset_roll, random_float) = rolls.get(y_index as usize).copied().unwrap_or((1, 0.0));
        if y_index == 0 || width_smoothness <= 0 || reset_roll.rem_euclid(width_smoothness) == 0 {
            width_factor = 1.0 + random_float * random_float;
        }
        factors.push(width_factor * width_factor);
    }
    factors
}

pub fn canyon_vertical_radius(
    default_factor: f32,
    center_factor: f32,
    vertical_radius: f64,
    distance: i32,
    current_step: i32,
    random_between_roll: f32,
) -> f64 {
    let vertical_multiplier = 1.0 - (0.5 - current_step as f32 / distance as f32).abs() * 2.0;
    let factor = default_factor + center_factor * vertical_multiplier;
    f64::from(factor)
        * vertical_radius
        * f64::from(0.75 + 0.25 * random_between_roll.clamp(0.0, 1.0))
}

pub fn carvers_for_noise_settings(settings_id: &str) -> &'static [&'static str] {
    match strip_minecraft(settings_id) {
        "nether" => NETHER_COMMON_CARVERS,
        "end" => &[],
        _ => OVERWORLD_COMMON_CARVERS,
    }
}

pub fn carvers_for_biome_source_and_noise_settings(
    biome_source_model: &BiomeSourceModel,
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
) -> &'static [&'static str] {
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let sampler = ClimateSampler::from_noise_router(&noise_router, seed, *settings);
    let biome = get_biome(biome_source_model, pos.x * 4, 0, pos.z * 4, &sampler)
        .unwrap_or("minecraft:plains");
    biome_generation_settings(biome)
        .map(|generation| generation.carvers)
        .unwrap_or_else(|| carvers_for_noise_settings(settings.id))
}

pub fn apply_configured_carvers_to_chunk(
    chunk: &mut LevelChunk,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    carver_ids: &[&'static str],
) -> usize {
    let height_context = WorldGenerationHeightContext {
        min_y: settings.noise.min_y,
        height: settings.noise.height,
    };
    let target_chunk = chunk.pos;
    let chunk_min_x = target_chunk.x * 16;
    let chunk_min_z = target_chunk.z * 16;
    let mut mask = Vec::new();
    let mut carved_blocks = 0;
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|e| e.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let mut noise_chunk = NoiseChunk::new(chunk_min_x, chunk_min_z, *settings, seed, noise_router);
    let algorithm = if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
    let mut aquifer = settings.aquifers_enabled.then(|| {
        NoiseBasedAquifer::new(
            &mut noise_chunk,
            chunk_min_x,
            chunk_min_x + 15,
            chunk_min_z,
            chunk_min_z + 15,
            settings.noise.min_y,
            settings.noise.height,
            seed,
            *settings,
            noise_router,
            factories.aquifer,
        )
    });

    for (carver_index, carver_id) in carver_ids.iter().enumerate() {
        let Some(carver) = configured_carver(carver_id) else {
            continue;
        };
        for source_chunk_x in target_chunk.x - 8..=target_chunk.x + 8 {
            for source_chunk_z in target_chunk.z - 8..=target_chunk.z + 8 {
                let mut random = LegacyRandom::new(carver_seed(
                    seed,
                    carver_index as i32,
                    source_chunk_x,
                    source_chunk_z,
                ));
                if !carver_is_start_chunk(carver, random.next_f32()) {
                    continue;
                }
                carved_blocks += carve_configured_carver_from_source_chunk(
                    chunk,
                    height_context,
                    carver,
                    source_chunk_x,
                    source_chunk_z,
                    chunk_min_x,
                    chunk_min_z,
                    &mut random,
                    &mut mask,
                    settings,
                    &noise_chunk,
                    aquifer.as_mut(),
                );
            }
        }
    }

    if !mask.is_empty() {
        chunk.carving_mask = Some(pack_carving_mask_indices(&mask));
    }

    carved_blocks
}

pub fn apply_configured_carvers_for_biome_source(
    chunk: &mut LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
) -> usize {
    let mut noise_context = create_live_noise_generation_context(chunk.pos, settings, seed);
    apply_configured_carvers_for_biome_source_with_noise_context(
        chunk,
        biome_source_model,
        settings,
        seed,
        &mut noise_context,
    )
}

fn create_live_noise_generation_context(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
) -> LiveNoiseGenerationContext {
    let chunk_min_x = pos.x * 16;
    let chunk_min_z = pos.z * 16;
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|e| e.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let mut noise_chunk = NoiseChunk::new(chunk_min_x, chunk_min_z, *settings, seed, noise_router);
    let algorithm = if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
    let aquifer = settings.aquifers_enabled.then(|| {
        NoiseBasedAquifer::new(
            &mut noise_chunk,
            chunk_min_x,
            chunk_min_x + 15,
            chunk_min_z,
            chunk_min_z + 15,
            settings.noise.min_y,
            settings.noise.height,
            seed,
            *settings,
            noise_router,
            factories.aquifer,
        )
    });
    LiveNoiseGenerationContext {
        noise_chunk,
        aquifer,
    }
}

pub(super) fn apply_configured_carvers_for_biome_source_with_noise_context(
    chunk: &mut LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_context: &mut LiveNoiseGenerationContext,
) -> usize {
    let height_context = WorldGenerationHeightContext {
        min_y: settings.noise.min_y,
        height: settings.noise.height,
    };
    let target_chunk = chunk.pos;
    let chunk_min_x = target_chunk.x * 16;
    let chunk_min_z = target_chunk.z * 16;
    let mut mask = Vec::new();
    let mut carved_blocks = 0;
    let common_overworld_carvers = matches!(
        biome_source_model,
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld"
        }
    ) && matches!(
        settings.id,
        "minecraft:overworld" | "minecraft:large_biomes" | "minecraft:amplified"
    );

    for source_chunk_x in target_chunk.x - 8..=target_chunk.x + 8 {
        for source_chunk_z in target_chunk.z - 8..=target_chunk.z + 8 {
            let source_pos = ChunkPos {
                x: source_chunk_x,
                z: source_chunk_z,
            };
            let carver_ids = if common_overworld_carvers {
                carvers_for_noise_settings(settings.id)
            } else {
                carvers_for_biome_source_and_noise_settings(
                    biome_source_model,
                    source_pos,
                    settings,
                    seed,
                )
            };
            for (carver_index, carver_id) in carver_ids.iter().enumerate() {
                let Some(carver) = configured_carver(carver_id) else {
                    continue;
                };
                let mut random = LegacyRandom::new(carver_seed(
                    seed,
                    carver_index as i32,
                    source_chunk_x,
                    source_chunk_z,
                ));
                if !carver_is_start_chunk(carver, random.next_f32()) {
                    continue;
                }
                carved_blocks += carve_configured_carver_from_source_chunk(
                    chunk,
                    height_context,
                    carver,
                    source_chunk_x,
                    source_chunk_z,
                    chunk_min_x,
                    chunk_min_z,
                    &mut random,
                    &mut mask,
                    settings,
                    &noise_context.noise_chunk,
                    noise_context.aquifer.as_mut(),
                );
            }
        }
    }

    if !mask.is_empty() {
        chunk.carving_mask = Some(pack_carving_mask_indices(&mask));
    }

    carved_blocks
}
