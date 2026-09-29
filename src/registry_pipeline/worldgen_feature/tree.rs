//! Tree building blocks: trunk, foliage and root placers, tree decorators and
//! feature sizes.

use serde_json::json;

use super::block_state::block_set;
use super::state_providers::block_state_provider;
use super::values::{
    direction, non_empty_list, non_negative_int_provider, positive_int_provider, probability,
};
use super::{cached_codec, typed, Variant};
use crate::registry_pipeline::codec::{
    int_codec, int_range, list, non_negative_int, opt, opt_default, or_else,
    positive_int, record, req, Codec, Field,
};
use crate::registry_pipeline::shared::int_provider;
use crate::storage::nbt::Tag;

// ---------------------------------------------------------------------------
// Trunk placers
// ---------------------------------------------------------------------------

/// `TrunkPlacer.trunkPlacerParts`: `base_height`, `height_rand_a`, `height_rand_b`.
fn trunk_placer_parts() -> Vec<Field> {
    vec![
        req("base_height", int_range(0, 32)),
        req("height_rand_a", int_range(0, 24)),
        req("height_rand_b", int_range(0, 24)),
    ]
}

/// `CherryTrunkPlacer.BRANCH_START_CODEC`: a bare uniform int provider map (no `type`)
/// spanning at least two values, validated to `[-16, 0]`.
fn branch_start_offset() -> Codec {
    record(vec![
        req("min_inclusive", int_codec()),
        req("max_inclusive", int_codec()),
    ])
    .validate(|tag| {
        let Tag::Compound(fields) = tag else {
            return Ok(());
        };
        let value = |name: &str| {
            fields.iter().find_map(|(key, tag)| match tag {
                Tag::Int(v) if key == name => Some(*v),
                _ => None,
            })
        };
        let (Some(min), Some(max)) = (value("min_inclusive"), value("max_inclusive")) else {
            return Ok(());
        };
        if max < min {
            Err(format!(
                "Max must be at least min, min_inclusive: {min}, max_inclusive: {max}"
            ))
        } else if max - min < 1 {
            Err("Need at least 2 blocks variation for the branch starts to fit both branches"
                .to_string())
        } else if min < -16 {
            Err(format!("Value provider too low: -16 [{min}-{max}]"))
        } else if max > 0 {
            Err(format!("Value provider too high: 0 [{min}-{max}]"))
        } else {
            Ok(())
        }
    })
}

const TRUNK_PLACER_TYPES: &[Variant] = &[
    ("straight_trunk_placer", || record(trunk_placer_parts())),
    ("forking_trunk_placer", || record(trunk_placer_parts())),
    ("giant_trunk_placer", || record(trunk_placer_parts())),
    ("mega_jungle_trunk_placer", || record(trunk_placer_parts())),
    ("dark_oak_trunk_placer", || record(trunk_placer_parts())),
    ("fancy_trunk_placer", || record(trunk_placer_parts())),
    ("bending_trunk_placer", || {
        let mut fields = trunk_placer_parts();
        fields.push(opt_default("min_height_for_leaves", positive_int(), json!(1)));
        fields.push(req("bend_length", int_provider(1, 64)));
        record(fields)
    }),
    ("upwards_branching_trunk_placer", || {
        let mut fields = trunk_placer_parts();
        fields.extend([
            req("extra_branch_steps", positive_int_provider()),
            req("place_branch_per_log_probability", probability()),
            req("extra_branch_length", non_negative_int_provider()),
            req("can_grow_through", block_set()),
        ]);
        record(fields)
    }),
    ("cherry_trunk_placer", || {
        let mut fields = trunk_placer_parts();
        fields.extend([
            req("branch_count", int_provider(1, 3)),
            req("branch_horizontal_length", int_provider(2, 16)),
            req("branch_start_offset_from_top", branch_start_offset()),
            req("branch_end_offset_from_top", int_provider(-16, 16)),
        ]);
        record(fields)
    }),
];

cached_codec! {
    /// `TrunkPlacer.CODEC`.
    pub fn trunk_placer() -> Codec {
        typed("type", "minecraft:worldgen/trunk_placer_type", TRUNK_PLACER_TYPES)
    }
}

// ---------------------------------------------------------------------------
// Foliage placers
// ---------------------------------------------------------------------------

/// `FoliagePlacer.foliagePlacerParts`: `radius` and `offset`.
fn foliage_placer_parts() -> Vec<Field> {
    vec![
        req("radius", int_provider(0, 16)),
        req("offset", int_provider(0, 16)),
    ]
}

/// `foliagePlacerParts` plus `field`.
fn foliage_with(field: Field) -> Codec {
    let mut fields = foliage_placer_parts();
    fields.push(field);
    record(fields)
}

/// `BlobFoliagePlacer.blobParts`: the parts and `height` in `[0, 16]`.
fn blob_parts() -> Codec {
    foliage_with(req("height", int_range(0, 16)))
}

const FOLIAGE_PLACER_TYPES: &[Variant] = &[
    ("blob_foliage_placer", blob_parts),
    ("spruce_foliage_placer", || {
        foliage_with(req("trunk_height", int_provider(0, 24)))
    }),
    ("pine_foliage_placer", || {
        foliage_with(req("height", int_provider(0, 24)))
    }),
    ("acacia_foliage_placer", || record(foliage_placer_parts())),
    ("bush_foliage_placer", blob_parts),
    ("fancy_foliage_placer", blob_parts),
    ("jungle_foliage_placer", || {
        foliage_with(req("height", int_range(0, 16)))
    }),
    ("mega_pine_foliage_placer", || {
        foliage_with(req("crown_height", int_provider(0, 24)))
    }),
    ("dark_oak_foliage_placer", || record(foliage_placer_parts())),
    ("random_spread_foliage_placer", || {
        let mut fields = foliage_placer_parts();
        fields.extend([
            req("foliage_height", int_provider(1, 512)),
            req("leaf_placement_attempts", int_range(0, 256)),
        ]);
        record(fields)
    }),
    ("cherry_foliage_placer", || {
        let mut fields = foliage_placer_parts();
        fields.extend([
            req("height", int_provider(4, 16)),
            req("wide_bottom_layer_hole_chance", probability()),
            req("corner_hole_chance", probability()),
            req("hanging_leaves_chance", probability()),
            req("hanging_leaves_extension_chance", probability()),
        ]);
        // Java's encoder reads `corner_hole_chance` from `wideBottomLayerHoleChance`
        // (`CherryFoliagePlacer.CODEC` forGetter), so both fields are written with the
        // wide bottom layer value.
        record(fields).map_tag(mirror_corner_hole_chance)
    }),
];

/// Reproduces the `CherryFoliagePlacer` encoder quirk described above.
fn mirror_corner_hole_chance(tag: Tag) -> Result<Tag, String> {
    let Tag::Compound(mut fields) = tag else {
        return Ok(tag);
    };
    let wide = fields
        .iter()
        .find_map(|(key, value)| (key == "wide_bottom_layer_hole_chance").then(|| value.clone()));
    if let Some(wide) = wide {
        for (key, value) in &mut fields {
            if key == "corner_hole_chance" {
                *value = wide.clone();
            }
        }
    }
    Ok(Tag::Compound(fields))
}

cached_codec! {
    /// `FoliagePlacer.CODEC`.
    pub fn foliage_placer() -> Codec {
        typed("type", "minecraft:worldgen/foliage_placer_type", FOLIAGE_PLACER_TYPES)
    }
}

// ---------------------------------------------------------------------------
// Root placers
// ---------------------------------------------------------------------------

/// `AboveRootPlacement.CODEC`.
fn above_root_placement() -> Codec {
    record(vec![
        req("above_root_provider", block_state_provider()),
        req("above_root_placement_chance", probability()),
    ])
}

/// `MangroveRootPlacement.CODEC`.
fn mangrove_root_placement() -> Codec {
    record(vec![
        req("can_grow_through", block_set()),
        req("muddy_roots_in", block_set()),
        req("muddy_roots_provider", block_state_provider()),
        req("max_root_width", int_range(1, 12)),
        req("max_root_length", int_range(1, 64)),
        req("random_skew_chance", probability()),
    ])
}

const ROOT_PLACER_TYPES: &[Variant] = &[("mangrove_root_placer", || {
    // `RootPlacer.rootPlacerParts` plus the mangrove placement.
    record(vec![
        req("trunk_offset_y", super::values::any_int_provider()),
        req("root_provider", block_state_provider()),
        opt("above_root_placement", above_root_placement()),
        req("mangrove_root_placement", mangrove_root_placement()),
    ])
})];

cached_codec! {
    /// `RootPlacer.CODEC`.
    pub fn root_placer() -> Codec {
        typed("type", "minecraft:worldgen/root_placer_type", ROOT_PLACER_TYPES)
    }
}

// ---------------------------------------------------------------------------
// Tree decorators
// ---------------------------------------------------------------------------

/// `{probability: floatRange(0, 1)}`.
fn probability_only() -> Codec {
    record(vec![req("probability", probability())])
}

const TREE_DECORATOR_TYPES: &[Variant] = &[
    ("trunk_vine", || record(vec![])),
    ("leave_vine", probability_only),
    ("pale_moss", || {
        record(vec![
            req("leaves_probability", probability()),
            req("trunk_probability", probability()),
            req("ground_probability", probability()),
        ])
    }),
    ("creaking_heart", probability_only),
    ("cocoa", probability_only),
    ("beehive", probability_only),
    ("alter_ground", || {
        record(vec![req("provider", block_state_provider())])
    }),
    ("attached_to_leaves", || {
        record(vec![
            req("probability", probability()),
            req("exclusion_radius_xz", int_range(0, 16)),
            req("exclusion_radius_y", int_range(0, 16)),
            req("block_provider", block_state_provider()),
            req("required_empty_blocks", int_range(1, 16)),
            req("directions", non_empty_list(direction())),
        ])
    }),
    ("place_on_ground", || {
        record(vec![
            or_else("tries", positive_int(), json!(128)),
            or_else("radius", non_negative_int(), json!(2)),
            or_else("height", non_negative_int(), json!(1)),
            req("block_state_provider", block_state_provider()),
        ])
    }),
    ("attached_to_logs", || {
        record(vec![
            req("probability", probability()),
            req("block_provider", block_state_provider()),
            req("directions", non_empty_list(direction())),
        ])
    }),
];

cached_codec! {
    /// `TreeDecorator.CODEC`.
    pub fn tree_decorator() -> Codec {
        typed("type", "minecraft:worldgen/tree_decorator_type", TREE_DECORATOR_TYPES)
    }
}

/// `TreeDecorator.CODEC.listOf()`.
pub fn tree_decorators() -> Codec {
    list(tree_decorator())
}

// ---------------------------------------------------------------------------
// Feature sizes
// ---------------------------------------------------------------------------

/// `FeatureSize.minClippedHeightCodec`: optional `min_clipped_height` in `[0, 80]`.
fn min_clipped_height() -> Field {
    opt("min_clipped_height", int_range(0, 80))
}

const FEATURE_SIZE_TYPES: &[Variant] = &[
    ("two_layers_feature_size", || {
        record(vec![
            or_else("limit", int_range(0, 81), json!(1)),
            or_else("lower_size", int_range(0, 16), json!(0)),
            or_else("upper_size", int_range(0, 16), json!(1)),
            min_clipped_height(),
        ])
    }),
    ("three_layers_feature_size", || {
        record(vec![
            or_else("limit", int_range(0, 80), json!(1)),
            or_else("upper_limit", int_range(0, 80), json!(1)),
            or_else("lower_size", int_range(0, 16), json!(0)),
            or_else("middle_size", int_range(0, 16), json!(1)),
            or_else("upper_size", int_range(0, 16), json!(1)),
            min_clipped_height(),
        ])
    }),
];

cached_codec! {
    /// `FeatureSize.CODEC`.
    pub fn feature_size() -> Codec {
        typed("type", "minecraft:worldgen/feature_size_type", FEATURE_SIZE_TYPES)
    }
}
