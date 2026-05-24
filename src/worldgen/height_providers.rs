use super::*;

impl VerticalAnchor {
    pub fn resolve_y(self, context: WorldGenerationHeightContext) -> i32 {
        match self {
            VerticalAnchor::Absolute(y) => y,
            VerticalAnchor::AboveBottom(offset) => context.min_y + offset,
            VerticalAnchor::BelowTop(offset) => context.min_y + context.height - 1 - offset,
        }
    }
}

pub fn height_provider_type(id: &str) -> Option<&'static HeightProviderType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    HEIGHT_PROVIDER_TYPES.iter().find(|provider_type| {
        provider_type
            .id
            .strip_prefix("minecraft:")
            .unwrap_or(provider_type.id)
            == name
    })
}

pub fn height_provider_sample_bounds(
    provider: HeightProvider,
    context: WorldGenerationHeightContext,
) -> (i32, i32) {
    match provider {
        HeightProvider::Constant { value } => {
            let y = value.resolve_y(context);
            (y, y)
        }
        HeightProvider::Uniform {
            min_inclusive,
            max_inclusive,
        }
        | HeightProvider::Trapezoid {
            min_inclusive,
            max_inclusive,
            ..
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if min > max {
                (min, min)
            } else {
                (min, max)
            }
        }
        HeightProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        }
        | HeightProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if max - min - inner + 1 <= 0 {
                (min, min)
            } else {
                (min, max - 1)
            }
        }
        HeightProvider::WeightedList { distribution } => distribution
            .iter()
            .map(|entry| height_provider_sample_bounds(entry.provider, context))
            .reduce(|(min_a, max_a), (min_b, max_b)| (min_a.min(min_b), max_a.max(max_b)))
            .unwrap_or((context.min_y, context.min_y)),
    }
}

pub fn height_provider_sample_with_rolls(
    provider: HeightProvider,
    context: WorldGenerationHeightContext,
    first_roll: i32,
    second_roll: i32,
    third_roll: i32,
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
                min + first_roll.rem_euclid(max - min + 1)
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
                let limit = first_roll.rem_euclid(outer_bound);
                min + second_roll.rem_euclid(limit + inner)
            }
        }
        HeightProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if max - min - inner + 1 <= 0 {
                min
            } else {
                let upper = min + inner + first_roll.rem_euclid(max - (min + inner) + 1);
                let biased_upper = min + second_roll.rem_euclid(upper - min);
                min + third_roll.rem_euclid(biased_upper - min + inner)
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
                return min + first_roll.rem_euclid(range + 1);
            }

            let plateau_start = (range - plateau) / 2;
            let plateau_end = range - plateau_start;
            min + first_roll.rem_euclid(plateau_end + 1) + second_roll.rem_euclid(plateau_start + 1)
        }
        HeightProvider::WeightedList { distribution } => {
            let positive_weight_total = distribution
                .iter()
                .map(|entry| entry.weight.max(0))
                .sum::<i32>();
            if positive_weight_total <= 0 {
                return context.min_y;
            }

            let mut choice = first_roll.rem_euclid(positive_weight_total);
            let selected = distribution
                .iter()
                .find(|entry| {
                    let weight = entry.weight.max(0);
                    if choice < weight {
                        true
                    } else {
                        choice -= weight;
                        false
                    }
                })
                .expect("positive total weight must select a provider");
            height_provider_sample_with_rolls(
                selected.provider,
                context,
                second_roll,
                third_roll,
                0,
            )
        }
    }
}

pub const HEIGHT_PROVIDER_TYPES: &[HeightProviderType] = &[
    HeightProviderType {
        id: "minecraft:constant",
    },
    HeightProviderType {
        id: "minecraft:uniform",
    },
    HeightProviderType {
        id: "minecraft:biased_to_bottom",
    },
    HeightProviderType {
        id: "minecraft:very_biased_to_bottom",
    },
    HeightProviderType {
        id: "minecraft:trapezoid",
    },
    HeightProviderType {
        id: "minecraft:weighted_list",
    },
];
