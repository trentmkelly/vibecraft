use super::*;

pub fn replace_block_result(
    current_block: &'static str,
    targets: &[TargetBlockStateModel],
) -> Option<&'static str> {
    targets
        .iter()
        .find(|target| rule_test_matches(target.target, current_block))
        .map(|target| target.state)
}

pub fn configured_ore_configuration(id: &str) -> Option<OreConfigurationModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    let natural_stone = RuleTestModel::BlockTag("minecraft:base_stone_overworld");
    let stone_ore = RuleTestModel::BlockTag("minecraft:stone_ore_replaceables");
    let deepslate_ore = RuleTestModel::BlockTag("minecraft:deepslate_ore_replaceables");
    let netherrack = RuleTestModel::BlockMatch("minecraft:netherrack");
    let nether_ore = RuleTestModel::BlockTag("minecraft:base_stone_nether");

    let single_target =
        |target, state, size, discard_chance_on_air_exposure| OreConfigurationModel {
            target_states: vec![TargetBlockStateModel { target, state }],
            size,
            discard_chance_on_air_exposure,
        };
    let overworld_ore_pair =
        |stone_state, deepslate_state, size, discard_chance_on_air_exposure| {
            OreConfigurationModel {
                target_states: vec![
                    TargetBlockStateModel {
                        target: stone_ore,
                        state: stone_state,
                    },
                    TargetBlockStateModel {
                        target: deepslate_ore,
                        state: deepslate_state,
                    },
                ],
                size,
                discard_chance_on_air_exposure,
            }
        };

    Some(match name {
        "ore_magma" => single_target(netherrack, "minecraft:magma_block", 33, 0.0),
        "ore_soul_sand" => single_target(netherrack, "minecraft:soul_sand", 12, 0.0),
        "ore_nether_gold" => single_target(netherrack, "minecraft:nether_gold_ore", 10, 0.0),
        "ore_quartz" => single_target(netherrack, "minecraft:nether_quartz_ore", 14, 0.0),
        "ore_gravel_nether" => single_target(netherrack, "minecraft:gravel", 33, 0.0),
        "ore_blackstone" => single_target(netherrack, "minecraft:blackstone", 33, 0.0),
        "ore_dirt" => single_target(natural_stone, "minecraft:dirt", 33, 0.0),
        "ore_gravel" => single_target(natural_stone, "minecraft:gravel", 33, 0.0),
        "ore_granite" => single_target(natural_stone, "minecraft:granite", 64, 0.0),
        "ore_diorite" => single_target(natural_stone, "minecraft:diorite", 64, 0.0),
        "ore_andesite" => single_target(natural_stone, "minecraft:andesite", 64, 0.0),
        "ore_tuff" => single_target(natural_stone, "minecraft:tuff", 64, 0.0),
        "ore_coal" => overworld_ore_pair(
            "minecraft:coal_ore",
            "minecraft:deepslate_coal_ore",
            17,
            0.0,
        ),
        "ore_coal_buried" => overworld_ore_pair(
            "minecraft:coal_ore",
            "minecraft:deepslate_coal_ore",
            17,
            0.5,
        ),
        "ore_iron" => {
            overworld_ore_pair("minecraft:iron_ore", "minecraft:deepslate_iron_ore", 9, 0.0)
        }
        "ore_iron_small" => {
            overworld_ore_pair("minecraft:iron_ore", "minecraft:deepslate_iron_ore", 4, 0.0)
        }
        "ore_gold" => {
            overworld_ore_pair("minecraft:gold_ore", "minecraft:deepslate_gold_ore", 9, 0.0)
        }
        "ore_gold_buried" => {
            overworld_ore_pair("minecraft:gold_ore", "minecraft:deepslate_gold_ore", 9, 0.5)
        }
        "ore_redstone" => overworld_ore_pair(
            "minecraft:redstone_ore",
            "minecraft:deepslate_redstone_ore",
            8,
            0.0,
        ),
        "ore_diamond_small" => overworld_ore_pair(
            "minecraft:diamond_ore",
            "minecraft:deepslate_diamond_ore",
            4,
            0.5,
        ),
        "ore_diamond_large" => overworld_ore_pair(
            "minecraft:diamond_ore",
            "minecraft:deepslate_diamond_ore",
            12,
            0.7,
        ),
        "ore_diamond_buried" => overworld_ore_pair(
            "minecraft:diamond_ore",
            "minecraft:deepslate_diamond_ore",
            8,
            1.0,
        ),
        "ore_diamond_medium" => overworld_ore_pair(
            "minecraft:diamond_ore",
            "minecraft:deepslate_diamond_ore",
            8,
            0.5,
        ),
        "ore_lapis" => overworld_ore_pair(
            "minecraft:lapis_ore",
            "minecraft:deepslate_lapis_ore",
            7,
            0.0,
        ),
        "ore_lapis_buried" => overworld_ore_pair(
            "minecraft:lapis_ore",
            "minecraft:deepslate_lapis_ore",
            7,
            1.0,
        ),
        "ore_infested" => overworld_ore_pair(
            "minecraft:infested_stone",
            "minecraft:infested_deepslate",
            9,
            0.0,
        ),
        "ore_emerald" => overworld_ore_pair(
            "minecraft:emerald_ore",
            "minecraft:deepslate_emerald_ore",
            3,
            0.0,
        ),
        "ore_ancient_debris_large" => single_target(nether_ore, "minecraft:ancient_debris", 3, 1.0),
        "ore_ancient_debris_small" => single_target(nether_ore, "minecraft:ancient_debris", 2, 1.0),
        "ore_copper_small" => overworld_ore_pair(
            "minecraft:copper_ore",
            "minecraft:deepslate_copper_ore",
            10,
            0.0,
        ),
        "ore_copper_large" => overworld_ore_pair(
            "minecraft:copper_ore",
            "minecraft:deepslate_copper_ore",
            20,
            0.0,
        ),
        "ore_clay" => single_target(natural_stone, "minecraft:clay", 33, 0.0),
        _ => return None,
    })
}

pub fn rule_test_matches(test: RuleTestModel, block: &str) -> bool {
    match test {
        RuleTestModel::AlwaysTrue => true,
        RuleTestModel::BlockMatch(expected) => block == expected,
        RuleTestModel::TagMatch(blocks) => blocks.contains(&block),
        RuleTestModel::BlockTag(tag) => block_matches_tag(block, tag),
    }
}

pub fn ore_should_skip_air_check(discard_chance_on_air_exposure: f32, random_roll: f32) -> bool {
    if discard_chance_on_air_exposure <= 0.0 {
        true
    } else if discard_chance_on_air_exposure >= 1.0 {
        false
    } else {
        random_roll >= discard_chance_on_air_exposure
    }
}

pub fn ore_can_place(
    current_block: &'static str,
    adjacent_to_air: bool,
    config: &OreConfigurationModel,
    target: TargetBlockStateModel,
    random_roll: f32,
) -> bool {
    rule_test_matches(target.target, current_block)
        && (ore_should_skip_air_check(config.discard_chance_on_air_exposure, random_roll)
            || !adjacent_to_air)
}

pub fn scattered_ore_offset(
    origin: BlockPos,
    try_index: i32,
    axis_rolls: [(f32, f32); 3],
) -> BlockPos {
    let max_distance = try_index.min(7);
    let axis =
        |(first, second): (f32, f32)| ((first - second) * max_distance as f32).round() as i32;
    BlockPos {
        x: origin.x + axis(axis_rolls[0]),
        y: origin.y + axis(axis_rolls[1]),
        z: origin.z + axis(axis_rolls[2]),
    }
}

pub fn scattered_ore_attempt(
    origin: BlockPos,
    try_index: i32,
    axis_rolls: [(f32, f32); 3],
    current_block: &'static str,
    adjacent_to_air: bool,
    config: &OreConfigurationModel,
    air_check_roll: f32,
) -> Option<ScatteredOreAttempt> {
    let pos = scattered_ore_offset(origin, try_index, axis_rolls);
    config.target_states.iter().copied().find_map(|target| {
        ore_can_place(
            current_block,
            adjacent_to_air,
            config,
            target,
            air_check_roll,
        )
        .then_some(ScatteredOreAttempt {
            pos,
            state: target.state,
        })
    })
}

pub fn ore_vein_spheres(
    origin: BlockPos,
    size: i32,
    direction_roll: f32,
    y_rolls: &[(i32, i32)],
    radius_rolls: &[f64],
) -> Vec<OreVeinSphere> {
    let direction = direction_roll * std::f32::consts::PI;
    let spread_xy = size as f32 / 8.0;
    let x0 = f64::from(origin.x) + f64::from(direction.sin() * spread_xy);
    let x1 = f64::from(origin.x) - f64::from(direction.sin() * spread_xy);
    let z0 = f64::from(origin.z) + f64::from(direction.cos() * spread_xy);
    let z1 = f64::from(origin.z) - f64::from(direction.cos() * spread_xy);
    let default_y = (0, 0);
    let (y_roll_0, y_roll_1) = y_rolls.first().copied().unwrap_or(default_y);
    let y0 = f64::from(origin.y + y_roll_0 - 2);
    let y1 = f64::from(origin.y + y_roll_1 - 2);

    let mut spheres = Vec::new();
    for i in 0..size.max(0) {
        let step = i as f64 / size as f64;
        let radius_roll = radius_rolls.get(i as usize).copied().unwrap_or(0.0);
        let center_x = x0 + (x1 - x0) * step;
        let center_y = y0 + (y1 - y0) * step;
        let center_z = z0 + (z1 - z0) * step;
        let size_scale = radius_roll * f64::from(size) / 16.0;
        let radius = (((std::f64::consts::PI * step).sin() + 1.0) * size_scale + 1.0) / 2.0;
        spheres.push(OreVeinSphere {
            center_x,
            center_y,
            center_z,
            radius,
        });
    }

    for i in 0..spheres.len().saturating_sub(1) {
        if spheres[i].radius <= 0.0 {
            continue;
        }
        for j in i + 1..spheres.len() {
            if spheres[j].radius <= 0.0 {
                continue;
            }
            let dx = spheres[i].center_x - spheres[j].center_x;
            let dy = spheres[i].center_y - spheres[j].center_y;
            let dz = spheres[i].center_z - spheres[j].center_z;
            let dr = spheres[i].radius - spheres[j].radius;
            if ore_vein_sphere_is_shadowed(dr, dx, dy, dz) {
                if dr > 0.0 {
                    spheres[j].radius = -1.0;
                } else {
                    spheres[i].radius = -1.0;
                }
            }
        }
    }

    spheres
        .into_iter()
        .filter(|sphere| sphere.radius >= 0.0)
        .collect()
}

pub fn ore_vein_position_candidates(
    spheres: &[OreVeinSphere],
    x_start: i32,
    y_start: i32,
    z_start: i32,
    size_xz: i32,
    size_y: i32,
    build_height: std::ops::Range<i32>,
) -> Vec<BlockPos> {
    let mut positions = Vec::new();
    let bitset_len = (size_xz.max(0) as usize)
        .saturating_mul(size_y.max(0) as usize)
        .saturating_mul(size_xz.max(0) as usize);
    let mut visited = vec![false; bitset_len];
    for sphere in spheres {
        let x_min = (sphere.center_x - sphere.radius)
            .floor()
            .max(f64::from(x_start)) as i32;
        let y_min = (sphere.center_y - sphere.radius)
            .floor()
            .max(f64::from(y_start)) as i32;
        let z_min = (sphere.center_z - sphere.radius)
            .floor()
            .max(f64::from(z_start)) as i32;
        let x_max = (sphere.center_x + sphere.radius)
            .floor()
            .max(f64::from(x_min)) as i32;
        let y_max = (sphere.center_y + sphere.radius)
            .floor()
            .max(f64::from(y_min)) as i32;
        let z_max = (sphere.center_z + sphere.radius)
            .floor()
            .max(f64::from(z_min)) as i32;
        for x in x_min..=x_max {
            let xd = (f64::from(x) + 0.5 - sphere.center_x) / sphere.radius;
            if xd * xd >= 1.0 {
                continue;
            }
            for y in y_min..=y_max {
                let yd = (f64::from(y) + 0.5 - sphere.center_y) / sphere.radius;
                if xd * xd + yd * yd >= 1.0 {
                    continue;
                }
                for z in z_min..=z_max {
                    let zd = (f64::from(z) + 0.5 - sphere.center_z) / sphere.radius;
                    if xd * xd + yd * yd + zd * zd >= 1.0 || !build_height.contains(&y) {
                        continue;
                    }
                    let bitset_index =
                        x - x_start + (y - y_start) * size_xz + (z - z_start) * size_xz * size_y;
                    if bitset_index >= 0 {
                        let bitset_index = bitset_index as usize;
                        if bitset_index >= visited.len() {
                            // Java's BitSet grows past the requested initial
                            // size; OreFeature's inclusive coordinate loops can
                            // address boundary slots beyond sizeXZ*sizeY*sizeXZ.
                            visited.resize(bitset_index + 1, false);
                        }
                        if !visited[bitset_index] {
                            visited[bitset_index] = true;
                            positions.push(BlockPos { x, y, z });
                        }
                    }
                }
            }
        }
    }
    positions
}

pub fn ore_placement_plan(
    config: &OreConfigurationModel,
    candidates: &[OrePlacementContext],
) -> Vec<OrePlacementBlock> {
    candidates
        .iter()
        .filter_map(|candidate| {
            config.target_states.iter().copied().find_map(|target| {
                ore_can_place(
                    candidate.current_block,
                    candidate.adjacent_to_air,
                    config,
                    target,
                    candidate.air_check_roll,
                )
                .then_some(OrePlacementBlock {
                    pos: candidate.pos,
                    state: target.state,
                })
            })
        })
        .collect()
}
