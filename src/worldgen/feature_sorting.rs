use super::*;

pub fn build_features_per_step(
    feature_sources: &[&[&[&'static str]]],
    try_reducing_error: bool,
) -> Result<Vec<StepFeatureDataModel>, String> {
    match build_features_per_step_raw(feature_sources) {
        Ok(features) => Ok(features),
        Err(error) if try_reducing_error && error == "Feature order cycle found" => Err(format!(
            "Feature order cycle found, involved sources: {}",
            feature_sources.len()
        )),
        Err(error) => Err(error),
    }
}

pub fn build_features_per_step_with_source_ids(
    feature_sources: &[FeatureSorterSourceModel],
    try_reducing_error: bool,
) -> Result<Vec<StepFeatureDataModel>, String> {
    let source_steps = feature_sources
        .iter()
        .map(|source| source.feature_steps)
        .collect::<Vec<_>>();

    match build_features_per_step_raw(&source_steps) {
        Ok(features) => Ok(features),
        Err(error) if try_reducing_error && error == "Feature order cycle found" => {
            let mut reduced_sources = feature_sources.to_vec();

            loop {
                let last_size = reduced_sources.len();
                let mut index = 0;
                while index < reduced_sources.len() {
                    let removed = reduced_sources.remove(index);
                    let reduced_steps = reduced_sources
                        .iter()
                        .map(|source| source.feature_steps)
                        .collect::<Vec<_>>();

                    if build_features_per_step_raw(&reduced_steps).is_err() {
                        continue;
                    }

                    reduced_sources.insert(index, removed);
                    index += 1;
                }

                if last_size == reduced_sources.len() {
                    break;
                }
            }

            let involved_sources = reduced_sources
                .iter()
                .map(|source| source.id)
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "Feature order cycle found, involved sources: [{}]",
                involved_sources
            ))
        }
        Err(error) => Err(error),
    }
}

fn build_features_per_step_raw(
    feature_sources: &[&[&[&'static str]]],
) -> Result<Vec<StepFeatureDataModel>, String> {
    use std::collections::{BTreeMap, BTreeSet};

    let mut feature_indices = BTreeMap::<&'static str, usize>::new();
    let mut next_feature_index = 0_usize;
    let mut edges = BTreeMap::<FeatureSorterData, BTreeSet<FeatureSorterData>>::new();
    let mut max_step = 0_usize;

    for features_for_step in feature_sources {
        max_step = max_step.max(features_for_step.len());
        let mut feature_list = Vec::new();
        for (step, features) in features_for_step.iter().enumerate() {
            for feature in *features {
                let feature_index = *feature_indices.entry(*feature).or_insert_with(|| {
                    let index = next_feature_index;
                    next_feature_index += 1;
                    index
                });
                feature_list.push(FeatureSorterData {
                    feature_index,
                    step,
                    feature,
                });
            }
        }

        for (index, feature) in feature_list.iter().copied().enumerate() {
            let data = edges.entry(feature).or_default();
            if let Some(next) = feature_list.get(index + 1) {
                data.insert(*next);
            }
        }
    }

    let mut discovered = BTreeSet::new();
    let mut currently_visiting = BTreeSet::new();
    let mut sorted_features = Vec::new();
    for feature in edges.keys().copied().collect::<Vec<_>>() {
        if !currently_visiting.is_empty() {
            return Err(
                "You somehow broke the universe; DFS bork (iteration finished with non-empty in-progress vertex set"
                    .to_string(),
            );
        }
        if !discovered.contains(&feature)
            && feature_sorter_dfs(
                feature,
                &edges,
                &mut discovered,
                &mut currently_visiting,
                &mut sorted_features,
            )
        {
            return Err("Feature order cycle found".to_string());
        }
    }

    sorted_features.reverse();
    Ok((0..max_step)
        .map(|step| StepFeatureDataModel {
            features: sorted_features
                .iter()
                .copied()
                .filter(|feature| feature.step == step)
                .collect(),
        })
        .collect())
}

pub fn biome_decoration_feature_plan(
    world_seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    min_section_y: i32,
    features_per_step: &[StepFeatureDataModel],
    possible_biome_feature_steps: &[&[&[&'static str]]],
) -> BiomeDecorationFeaturePlan {
    use std::collections::BTreeSet;

    let origin = BlockPos {
        x: chunk_x * 16,
        y: min_section_y * 16,
        z: chunk_z * 16,
    };
    let decoration_seed = crate::random_source::decoration_seed(
        world_seed,
        origin.x,
        origin.z,
        RandomAlgorithm::Xoroshiro,
    );
    let mut feature_calls = Vec::new();

    for (step_index, step_feature_data) in features_per_step.iter().enumerate() {
        let mut possible_features_this_step = BTreeSet::new();
        for biome_steps in possible_biome_feature_steps {
            if let Some(features_in_biome_this_step) = biome_steps.get(step_index) {
                for feature in *features_in_biome_this_step {
                    if let Some(index) = step_feature_data.index_mapping(feature) {
                        possible_features_this_step.insert(index);
                    }
                }
            }
        }

        for global_feature_index in possible_features_this_step {
            if let Some(feature) = step_feature_data.features.get(global_feature_index) {
                feature_calls.push(BiomeDecorationFeatureCall {
                    step_index,
                    global_feature_index,
                    feature: feature.feature,
                    seed: crate::random_source::feature_seed(
                        decoration_seed,
                        global_feature_index as i32,
                        step_index as i32,
                    ),
                });
            }
        }
    }

    BiomeDecorationFeaturePlan {
        origin,
        decoration_seed,
        feature_calls,
    }
}

pub fn biome_decoration_structure_calls(
    decoration_seed: i64,
    generation_steps: usize,
    structures_by_step: &[&[&'static str]],
) -> Vec<BiomeDecorationStructureCall> {
    let mut calls = Vec::new();
    for step_index in 0..generation_steps {
        if let Some(structures) = structures_by_step.get(step_index) {
            for (step_structure_index, structure) in structures.iter().copied().enumerate() {
                calls.push(BiomeDecorationStructureCall {
                    step_index,
                    step_structure_index,
                    structure,
                    seed: crate::random_source::feature_seed(
                        decoration_seed,
                        step_structure_index as i32,
                        step_index as i32,
                    ),
                });
            }
        }
    }
    calls
}

pub(super) fn possible_biome_feature_steps_for_source(
    biome_source_model: &BiomeSourceModel,
) -> Vec<&'static [&'static [&'static str]]> {
    match biome_source_model {
        BiomeSourceModel::Fixed { biome } => biome_generation_settings(biome)
            .map(|generation| vec![generation.feature_steps])
            .unwrap_or_default(),
        BiomeSourceModel::Checkerboard { biomes, .. } => {
            let mut seen = Vec::new();
            let mut steps = Vec::new();
            for biome in biomes {
                if seen.contains(biome) {
                    continue;
                }
                if let Some(generation) = biome_generation_settings(biome) {
                    seen.push(*biome);
                    steps.push(generation.feature_steps);
                }
            }
            steps
        }
        BiomeSourceModel::MultiNoisePreset { preset } => multi_noise_parameter_list_preset(preset)
            .map(|preset| {
                let mut seen = Vec::new();
                let mut steps = Vec::new();
                for entry in preset.parameters {
                    if seen.contains(&entry.biome) {
                        continue;
                    }
                    if let Some(generation) = biome_generation_settings(entry.biome) {
                        seen.push(entry.biome);
                        steps.push(generation.feature_steps);
                    }
                }
                steps
            })
            .unwrap_or_default(),
        BiomeSourceModel::TheEnd => ["minecraft:the_end", "minecraft:end_highlands"]
            .into_iter()
            .filter_map(biome_generation_settings)
            .map(|generation| generation.feature_steps)
            .collect(),
    }
}

fn feature_sorter_dfs(
    feature: FeatureSorterData,
    edges: &std::collections::BTreeMap<
        FeatureSorterData,
        std::collections::BTreeSet<FeatureSorterData>,
    >,
    discovered: &mut std::collections::BTreeSet<FeatureSorterData>,
    currently_visiting: &mut std::collections::BTreeSet<FeatureSorterData>,
    sorted_features: &mut Vec<FeatureSorterData>,
) -> bool {
    if discovered.contains(&feature) {
        return false;
    }
    if !currently_visiting.insert(feature) {
        return true;
    }
    for child in edges.get(&feature).into_iter().flatten().copied() {
        if feature_sorter_dfs(
            child,
            edges,
            discovered,
            currently_visiting,
            sorted_features,
        ) {
            return true;
        }
    }
    currently_visiting.remove(&feature);
    discovered.insert(feature);
    sorted_features.push(feature);
    false
}

