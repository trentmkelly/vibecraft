use super::*;

pub fn block_state_provider_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:block_state_provider_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn block_state_provider_sample(
    provider: &BlockStateProviderModel,
    random_roll: i32,
) -> Option<&'static str> {
    block_state_provider_sample_with_context(provider, random_roll, None, 0, "minecraft:air")
}

pub fn block_state_provider_sample_in_context(
    provider: &BlockStateProviderModel,
    random_roll: i32,
    context: BlockPredicateContext,
    origin_y: i32,
    current_block: &'static str,
) -> Option<&'static str> {
    block_state_provider_sample_with_context(
        provider,
        random_roll,
        Some(context),
        origin_y,
        current_block,
    )
}

pub(super) fn block_state_provider_sample_in_context_with_random(
    provider: &BlockStateProviderModel,
    random: &mut RandomSourceKind,
    context: BlockPredicateContext,
    origin_y: i32,
    current_block: &'static str,
) -> Option<&'static str> {
    match provider {
        BlockStateProviderModel::Simple(state) => Some(*state),
        BlockStateProviderModel::RotatedBlock(block) => Some(rotated_pillar_state(
            block,
            match feature_random_next_i32_bound(random, 3) {
                0 => Axis::X,
                1 => Axis::Y,
                _ => Axis::Z,
            },
        )),
        BlockStateProviderModel::Weighted(entries) => {
            let total_weight = entries.iter().try_fold(0_i32, |total, entry| {
                (entry.weight > 0).then_some(total + entry.weight)
            })?;
            let mut roll = feature_random_next_i32_bound(random, total_weight);
            entries.iter().find_map(|entry| {
                roll -= entry.weight;
                (roll < 0).then_some(entry.state)
            })
        }
        BlockStateProviderModel::RandomizedInt {
            source,
            property,
            min_inclusive,
            max_inclusive,
        } => {
            let state = block_state_provider_sample_in_context_with_random(
                source,
                random,
                context,
                origin_y,
                current_block,
            )?;
            let value = *min_inclusive
                + feature_random_next_i32_bound(random, max_inclusive - min_inclusive + 1);
            randomized_int_state_provider_apply(state, property, value)
        }
        BlockStateProviderModel::RuleBased { fallback, rules } => {
            if let Some(rule) = rules
                .iter()
                .find(|rule| block_predicate_test(rule.if_true, context, origin_y))
            {
                return block_state_provider_sample_in_context_with_random(
                    &rule.then,
                    random,
                    context,
                    origin_y,
                    current_block,
                );
            }
            match fallback {
                Some(fallback) => block_state_provider_sample_in_context_with_random(
                    fallback,
                    random,
                    context,
                    origin_y,
                    current_block,
                ),
                None => Some(current_block),
            }
        }
        BlockStateProviderModel::Noise { .. }
        | BlockStateProviderModel::NoiseThreshold { .. }
        | BlockStateProviderModel::DualNoise { .. } => block_state_provider_sample_in_context(
            provider,
            feature_random_next_i32_bound(random, i32::MAX),
            context,
            origin_y,
            current_block,
        ),
    }
}

pub fn block_state_provider_sample_with_noise_value(
    provider: &BlockStateProviderModel,
    random_roll: i32,
    noise_value: f64,
) -> Option<&'static str> {
    block_state_provider_sample_with_context_and_noise(
        provider,
        random_roll,
        None,
        0,
        "minecraft:air",
        Some(noise_value),
    )
}

pub fn block_state_provider_sample_dual_noise_values(
    provider: &BlockStateProviderModel,
    variety_noise: f64,
    candidate_noise_values: &[f64],
    final_noise_value: f64,
) -> Option<&'static str> {
    match provider {
        BlockStateProviderModel::DualNoise {
            variety_min,
            variety_max,
            states,
        } => dual_noise_provider_select_state(
            states,
            *variety_min,
            *variety_max,
            variety_noise,
            candidate_noise_values,
            final_noise_value,
        ),
        _ => block_state_provider_sample_with_noise_value(provider, 0, final_noise_value),
    }
}

fn block_state_provider_sample_with_context(
    provider: &BlockStateProviderModel,
    random_roll: i32,
    context: Option<BlockPredicateContext>,
    origin_y: i32,
    current_block: &'static str,
) -> Option<&'static str> {
    block_state_provider_sample_with_context_and_noise(
        provider,
        random_roll,
        context,
        origin_y,
        current_block,
        None,
    )
}

fn block_state_provider_sample_with_context_and_noise(
    provider: &BlockStateProviderModel,
    random_roll: i32,
    context: Option<BlockPredicateContext>,
    origin_y: i32,
    current_block: &'static str,
    noise_value: Option<f64>,
) -> Option<&'static str> {
    match provider {
        BlockStateProviderModel::Simple(state) => Some(*state),
        BlockStateProviderModel::RotatedBlock(block) => Some(rotated_pillar_state(
            block,
            match random_roll.rem_euclid(3) {
                0 => Axis::X,
                1 => Axis::Y,
                _ => Axis::Z,
            },
        )),
        BlockStateProviderModel::Weighted(entries) => {
            let total_weight = entries.iter().try_fold(0_i32, |total, entry| {
                (entry.weight > 0).then_some(total + entry.weight)
            })?;
            let mut roll = random_roll.rem_euclid(total_weight);
            entries.iter().find_map(|entry| {
                roll -= entry.weight;
                (roll < 0).then_some(entry.state)
            })
        }
        BlockStateProviderModel::RandomizedInt {
            source,
            property,
            min_inclusive,
            max_inclusive,
        } => {
            let state = block_state_provider_sample(source, random_roll)?;
            let value = sample_inclusive_i32(*min_inclusive, *max_inclusive, random_roll);
            randomized_int_state_provider_apply(state, property, value)
        }
        BlockStateProviderModel::RuleBased { fallback, rules } => {
            if let Some(context) = context {
                if let Some(rule) = rules
                    .iter()
                    .find(|rule| block_predicate_test(rule.if_true, context, origin_y))
                {
                    return block_state_provider_sample_with_context(
                        &rule.then,
                        random_roll,
                        Some(context),
                        origin_y,
                        current_block,
                    );
                }
            }
            match fallback {
                Some(fallback) => block_state_provider_sample_with_context(
                    fallback,
                    random_roll,
                    context,
                    origin_y,
                    current_block,
                ),
                None => Some(current_block),
            }
        }
        BlockStateProviderModel::Noise { states } => {
            noise_provider_select_state(states, noise_value.unwrap_or(0.0))
        }
        BlockStateProviderModel::NoiseThreshold {
            threshold,
            high_chance,
            default_state,
            low_states,
            high_states,
        } => {
            let noise_value = noise_value.unwrap_or(0.0);
            if noise_value < f64::from(*threshold) {
                weighted_noise_state_pick(low_states, random_roll)
            } else if random_float_from_roll(random_roll) < *high_chance {
                weighted_noise_state_pick(high_states, random_roll)
            } else {
                Some(*default_state)
            }
        }
        BlockStateProviderModel::DualNoise {
            variety_min,
            variety_max,
            states,
        } => dual_noise_provider_select_state(
            states,
            *variety_min,
            *variety_max,
            0.0,
            &[0.0],
            noise_value.unwrap_or(0.0),
        ),
    }
}

fn noise_provider_select_state(states: &[&'static str], noise_value: f64) -> Option<&'static str> {
    if states.is_empty() {
        return None;
    }
    let placement_value = ((1.0 + noise_value) / 2.0).clamp(0.0, 0.9999);
    states
        .get((placement_value * states.len() as f64) as usize)
        .copied()
}

fn weighted_noise_state_pick(states: &[&'static str], random_roll: i32) -> Option<&'static str> {
    if states.is_empty() {
        return None;
    }
    states
        .get(random_roll.rem_euclid(states.len() as i32) as usize)
        .copied()
}

fn dual_noise_provider_select_state(
    states: &[&'static str],
    variety_min: i32,
    variety_max: i32,
    variety_noise: f64,
    candidate_noise_values: &[f64],
    final_noise_value: f64,
) -> Option<&'static str> {
    if states.is_empty() {
        return None;
    }
    let local_variety = clamped_map(
        variety_noise,
        -1.0,
        1.0,
        f64::from(variety_min),
        f64::from(variety_max + 1),
    ) as usize;
    if local_variety == 0 {
        return None;
    }

    let mut possible_states = Vec::with_capacity(local_variety);
    for index in 0..local_variety {
        let candidate_noise = candidate_noise_values.get(index).copied().unwrap_or(0.0);
        possible_states.push(noise_provider_select_state(states, candidate_noise)?);
    }
    noise_provider_select_state(&possible_states, final_noise_value)
}

fn random_float_from_roll(random_roll: i32) -> f32 {
    (random_roll.rem_euclid(1_000_000) as f32) / 1_000_000.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

fn rotated_pillar_state(block: &'static str, axis: Axis) -> &'static str {
    match (block, axis) {
        ("minecraft:oak_log", Axis::X) => "minecraft:oak_log[axis=x]",
        ("minecraft:oak_log", Axis::Y) => "minecraft:oak_log",
        ("minecraft:oak_log", Axis::Z) => "minecraft:oak_log[axis=z]",
        ("minecraft:birch_log", Axis::X) => "minecraft:birch_log[axis=x]",
        ("minecraft:birch_log", Axis::Y) => "minecraft:birch_log",
        ("minecraft:birch_log", Axis::Z) => "minecraft:birch_log[axis=z]",
        ("minecraft:spruce_log", Axis::X) => "minecraft:spruce_log[axis=x]",
        ("minecraft:spruce_log", Axis::Y) => "minecraft:spruce_log",
        ("minecraft:spruce_log", Axis::Z) => "minecraft:spruce_log[axis=z]",
        ("minecraft:jungle_log", Axis::X) => "minecraft:jungle_log[axis=x]",
        ("minecraft:jungle_log", Axis::Y) => "minecraft:jungle_log",
        ("minecraft:jungle_log", Axis::Z) => "minecraft:jungle_log[axis=z]",
        ("minecraft:acacia_log", Axis::X) => "minecraft:acacia_log[axis=x]",
        ("minecraft:acacia_log", Axis::Y) => "minecraft:acacia_log",
        ("minecraft:acacia_log", Axis::Z) => "minecraft:acacia_log[axis=z]",
        ("minecraft:dark_oak_log", Axis::X) => "minecraft:dark_oak_log[axis=x]",
        ("minecraft:dark_oak_log", Axis::Y) => "minecraft:dark_oak_log",
        ("minecraft:dark_oak_log", Axis::Z) => "minecraft:dark_oak_log[axis=z]",
        ("minecraft:mangrove_log", Axis::X) => "minecraft:mangrove_log[axis=x]",
        ("minecraft:mangrove_log", Axis::Y) => "minecraft:mangrove_log",
        ("minecraft:mangrove_log", Axis::Z) => "minecraft:mangrove_log[axis=z]",
        ("minecraft:cherry_log", Axis::X) => "minecraft:cherry_log[axis=x]",
        ("minecraft:cherry_log", Axis::Y) => "minecraft:cherry_log",
        ("minecraft:cherry_log", Axis::Z) => "minecraft:cherry_log[axis=z]",
        ("minecraft:pale_oak_log", Axis::X) => "minecraft:pale_oak_log[axis=x]",
        ("minecraft:pale_oak_log", Axis::Y) => "minecraft:pale_oak_log",
        ("minecraft:pale_oak_log", Axis::Z) => "minecraft:pale_oak_log[axis=z]",
        ("minecraft:hay_block", Axis::X) => "minecraft:hay_block[axis=x]",
        ("minecraft:hay_block", Axis::Y) => "minecraft:hay_block",
        ("minecraft:hay_block", Axis::Z) => "minecraft:hay_block[axis=z]",
        ("minecraft:basalt", Axis::X) => "minecraft:basalt[axis=x]",
        ("minecraft:basalt", Axis::Y) => "minecraft:basalt",
        ("minecraft:basalt", Axis::Z) => "minecraft:basalt[axis=z]",
        _ => block,
    }
}

fn randomized_int_state_provider_apply(
    state: &'static str,
    property: &str,
    value: i32,
) -> Option<&'static str> {
    match (state, property, value) {
        ("minecraft:cave_vines[age=0,berries=false]", "age", 23) => {
            Some("minecraft:cave_vines[age=23,berries=false]")
        }
        ("minecraft:cave_vines[age=0,berries=false]", "age", 24) => {
            Some("minecraft:cave_vines[age=24,berries=false]")
        }
        ("minecraft:cave_vines[age=0,berries=false]", "age", 25) => {
            Some("minecraft:cave_vines[age=25,berries=false]")
        }
        ("minecraft:cave_vines[age=0,berries=true]", "age", 23) => {
            Some("minecraft:cave_vines[age=23,berries=true]")
        }
        ("minecraft:cave_vines[age=0,berries=true]", "age", 24) => {
            Some("minecraft:cave_vines[age=24,berries=true]")
        }
        ("minecraft:cave_vines[age=0,berries=true]", "age", 25) => {
            Some("minecraft:cave_vines[age=25,berries=true]")
        }
        (
            "minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]",
            "age",
            0,
        ) => Some("minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]"),
        (
            "minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]",
            "age",
            1,
        ) => Some("minecraft:mangrove_propagule[age=1,hanging=true,stage=0,waterlogged=false]"),
        (
            "minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]",
            "age",
            2,
        ) => Some("minecraft:mangrove_propagule[age=2,hanging=true,stage=0,waterlogged=false]"),
        (
            "minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]",
            "age",
            3,
        ) => Some("minecraft:mangrove_propagule[age=3,hanging=true,stage=0,waterlogged=false]"),
        (
            "minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]",
            "age",
            4,
        ) => Some("minecraft:mangrove_propagule[age=4,hanging=true,stage=0,waterlogged=false]"),
        _ => Some(state),
    }
}
