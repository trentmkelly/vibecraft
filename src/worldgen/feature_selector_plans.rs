use super::*;

pub fn validate_weighted_placed_feature(
    feature: WeightedPlacedFeatureModel,
) -> Result<WeightedPlacedFeatureModel, &'static str> {
    if feature.feature.is_empty() {
        Err("weighted placed feature must reference a feature")
    } else if !(0.0..=1.0).contains(&feature.chance) {
        Err("weighted placed feature chance must be in 0.0..=1.0")
    } else {
        Ok(feature)
    }
}

pub fn random_selector_feature<'a>(
    config: &'a RandomFeatureConfigurationModel,
    chance_rolls: &[f32],
) -> Option<&'a str> {
    for (index, feature) in config.features.iter().enumerate() {
        validate_weighted_placed_feature(*feature).ok()?;
        let roll = chance_rolls.get(index).copied().unwrap_or(1.0);
        if roll < feature.chance {
            return Some(feature.feature);
        }
    }
    Some(config.default_feature)
}

pub fn simple_random_selector_feature(
    config: &SimpleRandomFeatureConfigurationModel,
    index_roll: i32,
) -> Option<&str> {
    if config.features.is_empty() {
        None
    } else {
        Some(config.features[index_roll.rem_euclid(config.features.len() as i32) as usize])
    }
}

pub fn random_boolean_selector_feature(
    config: RandomBooleanFeatureConfigurationModel,
    roll: bool,
) -> &'static str {
    if roll {
        config.feature_true
    } else {
        config.feature_false
    }
}
