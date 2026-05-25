use super::*;

pub fn feature_size_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:feature_size_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn trunk_placer_type(id: &str) -> Option<&'static str> {
    worldgen_type_registry_entry("minecraft:trunk_placer_type", id)
}

pub fn foliage_placer_type(id: &str) -> Option<&'static str> {
    worldgen_type_registry_entry("minecraft:foliage_placer_type", id)
}

pub fn root_placer_type(id: &str) -> Option<&'static str> {
    worldgen_type_registry_entry("minecraft:root_placer_type", id)
}

fn worldgen_type_registry_entry(registry_id: &str, id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == registry_id)?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn validate_trunk_placer(placer: TrunkPlacerModel) -> Result<TrunkPlacerModel, String> {
    if !(0..=32).contains(&placer.base_height)
        || !(0..=24).contains(&placer.height_rand_a)
        || !(0..=24).contains(&placer.height_rand_b)
    {
        return Err("trunk placer base fields are outside vanilla codec ranges".to_string());
    }

    let valid = match placer.kind {
        TrunkPlacerKind::Straight
        | TrunkPlacerKind::Forking
        | TrunkPlacerKind::Giant
        | TrunkPlacerKind::MegaJungle
        | TrunkPlacerKind::DarkOak
        | TrunkPlacerKind::Fancy => true,
        TrunkPlacerKind::Bending {
            min_height_for_leaves,
            bend_length_min,
            bend_length_max,
        } => {
            min_height_for_leaves > 0
                && (1..=64).contains(&bend_length_min)
                && (1..=64).contains(&bend_length_max)
                && bend_length_min <= bend_length_max
        }
        TrunkPlacerKind::UpwardsBranching {
            place_branch_per_log_probability,
            extra_branch_steps_min,
            extra_branch_length_min,
        } => {
            (0.0..=1.0).contains(&place_branch_per_log_probability)
                && extra_branch_steps_min > 0
                && extra_branch_length_min >= 0
        }
        TrunkPlacerKind::Cherry {
            branch_count_min,
            branch_count_max,
            branch_horizontal_length_min,
            branch_horizontal_length_max,
            branch_start_offset_from_top_min,
            branch_start_offset_from_top_max,
            branch_end_offset_from_top_min,
            branch_end_offset_from_top_max,
        } => {
            (1..=3).contains(&branch_count_min)
                && (1..=3).contains(&branch_count_max)
                && branch_count_min <= branch_count_max
                && (2..=16).contains(&branch_horizontal_length_min)
                && (2..=16).contains(&branch_horizontal_length_max)
                && branch_horizontal_length_min <= branch_horizontal_length_max
                && (-16..=0).contains(&branch_start_offset_from_top_min)
                && (-16..=0).contains(&branch_start_offset_from_top_max)
                && branch_start_offset_from_top_min <= branch_start_offset_from_top_max
                && branch_start_offset_from_top_max - branch_start_offset_from_top_min >= 1
                && (-16..=16).contains(&branch_end_offset_from_top_min)
                && (-16..=16).contains(&branch_end_offset_from_top_max)
                && branch_end_offset_from_top_min <= branch_end_offset_from_top_max
        }
    };
    if valid {
        Ok(placer)
    } else {
        Err("trunk placer variant fields are outside vanilla codec ranges".to_string())
    }
}

pub fn trunk_placer_height(placer: TrunkPlacerModel, rand_a: i32, rand_b: i32) -> i32 {
    placer.base_height
        + rand_a.rem_euclid(placer.height_rand_a + 1)
        + rand_b.rem_euclid(placer.height_rand_b + 1)
}

pub fn tree_valid_pos(state: &str) -> bool {
    let state = block_state_id(state);
    block_matches_tag(state, "minecraft:air")
        || block_matches_tag(state, "minecraft:replaceable_by_trees")
}

pub fn tree_trunk_free_pos(state: &str) -> bool {
    let state = block_state_id(state);
    tree_valid_pos(state) || block_matches_tag(state, "minecraft:logs")
}

pub fn tree_max_free_height(
    tree_height: i32,
    min_size: FeatureSizeModel,
    rows: &[&[&str]],
    ignore_vines: bool,
) -> i32 {
    for y in 0..=tree_height + 1 {
        let radius = feature_size_at_height(min_size, tree_height, y);
        let radius_width = radius * 2 + 1;
        let expected_width = (radius_width * radius_width).max(0) as usize;
        let row = rows.get(y as usize).copied().unwrap_or(&[]);
        if row.len() < expected_width
            || row.iter().take(expected_width).any(|state| {
                !tree_trunk_free_pos(state) || (!ignore_vines && *state == "minecraft:vine")
            })
        {
            return y - 2;
        }
    }
    tree_height
}

pub struct TreeCanPlaceInput<'a> {
    pub origin: BlockPos,
    pub trunk_origin: BlockPos,
    pub tree_height: i32,
    pub min_size: FeatureSizeModel,
    pub min_clipped_height: Option<i32>,
    pub build_min_y: i32,
    pub build_max_y: i32,
    pub rows: &'a [&'a [&'a str]],
    pub ignore_vines: bool,
}

pub fn tree_can_place(input: TreeCanPlaceInput<'_>) -> bool {
    let min_y = input.origin.y.min(input.trunk_origin.y);
    let max_y = input.origin.y.max(input.trunk_origin.y) + input.tree_height + 1;
    if min_y < input.build_min_y + 1 || max_y > input.build_max_y + 1 {
        return false;
    }
    let clipped_tree_height = tree_max_free_height(
        input.tree_height,
        input.min_size,
        input.rows,
        input.ignore_vines,
    );
    clipped_tree_height >= input.tree_height
        || input
            .min_clipped_height
            .is_some_and(|min| clipped_tree_height >= min)
}

fn feature_size_min_clipped_height(size: FeatureSizeModel) -> Option<i32> {
    match size {
        FeatureSizeModel::TwoLayers {
            min_clipped_height, ..
        }
        | FeatureSizeModel::ThreeLayers {
            min_clipped_height, ..
        } => min_clipped_height,
    }
}

pub fn validate_tree_configuration(config: &TreeConfigurationModel) -> Result<(), String> {
    validate_trunk_placer(config.trunk_placer)?;
    validate_foliage_placer(config.foliage_placer)?;
    validate_feature_size(config.minimum_size)?;
    if let Some(root_placer) = config.root_placer {
        validate_root_placer(root_placer)?;
    }
    for decorator in config.decorators.iter().copied() {
        validate_tree_decorator(decorator)?;
    }
    if block_state_provider_sample(&config.trunk_provider, 0).is_none()
        || block_state_provider_sample(&config.foliage_provider, 0).is_none()
        || block_state_provider_sample(&config.dirt_provider, 0).is_none()
    {
        return Err("tree configuration providers must sample block states".to_string());
    }
    Ok(())
}

pub fn configured_tree_placement_plan(
    origin: BlockPos,
    config: &TreeConfigurationModel,
    build_min_y: i32,
    build_max_y: i32,
    replaceable_rows: &[&[&str]],
    rand_a: i32,
    rand_b: i32,
) -> Result<Option<TreePlacementPlan>, String> {
    validate_tree_configuration(config)?;
    let tree_height = trunk_placer_height(config.trunk_placer, rand_a, rand_b);
    if !tree_can_place(TreeCanPlaceInput {
        origin,
        trunk_origin: origin,
        tree_height,
        min_size: config.minimum_size,
        min_clipped_height: feature_size_min_clipped_height(config.minimum_size),
        build_min_y,
        build_max_y,
        rows: replaceable_rows,
        ignore_vines: config.ignore_vines,
    }) {
        return Ok(None);
    }

    let trunk_provider = block_state_provider_sample(&config.trunk_provider, rand_a)
        .ok_or_else(|| "validated tree trunk provider produced no block state".to_string())?;
    let foliage_provider = block_state_provider_sample(&config.foliage_provider, rand_b)
        .ok_or_else(|| "validated tree foliage provider produced no block state".to_string())?;
    let dirt_provider =
        block_state_provider_sample(&config.dirt_provider, rand_a.wrapping_add(rand_b))
            .ok_or_else(|| "validated tree dirt provider produced no block state".to_string())?;

    Ok(Some(simple_tree_placement_plan(
        origin,
        config.trunk_placer,
        config.foliage_placer,
        trunk_provider,
        foliage_provider,
        dirt_provider,
        rand_a,
        rand_b,
    )?))
}

pub fn validate_foliage_placer(placer: FoliagePlacerModel) -> Result<FoliagePlacerModel, String> {
    if !(0..=16).contains(&placer.radius_min)
        || !(0..=16).contains(&placer.radius_max)
        || placer.radius_min > placer.radius_max
        || !(0..=16).contains(&placer.offset_min)
        || !(0..=16).contains(&placer.offset_max)
        || placer.offset_min > placer.offset_max
    {
        return Err("foliage placer base providers are outside vanilla codec ranges".to_string());
    }

    let valid = match placer.kind {
        FoliagePlacerKind::Blob { height }
        | FoliagePlacerKind::Bush { height }
        | FoliagePlacerKind::Fancy { height }
        | FoliagePlacerKind::Jungle { height } => (0..=16).contains(&height),
        FoliagePlacerKind::Cherry { height, .. } => (4..=16).contains(&height),
        FoliagePlacerKind::Spruce {
            height_min,
            height_max,
        }
        | FoliagePlacerKind::Pine {
            height_min,
            height_max,
        }
        | FoliagePlacerKind::MegaPine {
            height_min,
            height_max,
        } => {
            (0..=24).contains(&height_min)
                && (0..=24).contains(&height_max)
                && height_min <= height_max
        }
        FoliagePlacerKind::Acacia | FoliagePlacerKind::DarkOak => true,
        FoliagePlacerKind::RandomSpread {
            foliage_height_min,
            foliage_height_max,
            leaf_placement_attempts,
        } => {
            (1..=512).contains(&foliage_height_min)
                && (1..=512).contains(&foliage_height_max)
                && foliage_height_min <= foliage_height_max
                && (0..=256).contains(&leaf_placement_attempts)
        }
    };

    let chance_valid = match placer.kind {
        FoliagePlacerKind::Cherry {
            wide_bottom_layer_hole_chance,
            corner_hole_chance,
            hanging_leaves_chance,
            hanging_leaves_extension_chance,
            ..
        } => [
            wide_bottom_layer_hole_chance,
            corner_hole_chance,
            hanging_leaves_chance,
            hanging_leaves_extension_chance,
        ]
        .into_iter()
        .all(|chance| (0.0..=1.0).contains(&chance)),
        _ => true,
    };

    if valid && chance_valid {
        Ok(placer)
    } else {
        Err("foliage placer variant fields are outside vanilla codec ranges".to_string())
    }
}

pub fn validate_root_placer(placer: RootPlacerModel) -> Result<RootPlacerModel, String> {
    if placer
        .above_root_placement_chance
        .is_some_and(|chance| !(0.0..=1.0).contains(&chance))
        || !(1..=12).contains(&placer.mangrove_root_placement.max_root_width)
        || !(1..=64).contains(&placer.mangrove_root_placement.max_root_length)
        || !(0.0..=1.0).contains(&placer.mangrove_root_placement.random_skew_chance)
    {
        Err("root placer fields are outside vanilla codec ranges".to_string())
    } else {
        Ok(placer)
    }
}
