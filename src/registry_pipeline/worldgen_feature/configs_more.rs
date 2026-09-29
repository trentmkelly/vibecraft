//! `FeatureConfiguration` codecs, part two: geodes, dripstone, sculk and the random
//! selector features.

use serde_json::json;

use super::block_state::{block_state, block_tag_hashed};
use super::placement::{placed_feature_ref, placed_feature_set};
use super::state_providers::block_state_provider;
use super::values::{any_int_provider, float_provider, non_empty_list, probability};
use crate::registry_pipeline::codec::{
    bool_codec, double_range, float_range, int_codec, int_range, list, or_else, record, req,
    Codec,
};
use crate::registry_pipeline::shared::int_provider;

/// `Codec.doubleRange(0.0, 1.0)` (`GeodeConfiguration.CHANCE_RANGE`).
fn chance_range() -> Codec {
    double_range(0.0, 1.0)
}

/// `GeodeBlockSettings.CODEC`.
fn geode_block_settings() -> Codec {
    record(vec![
        req("filling_provider", block_state_provider()),
        req("inner_layer_provider", block_state_provider()),
        req("alternate_inner_layer_provider", block_state_provider()),
        req("middle_layer_provider", block_state_provider()),
        req("outer_layer_provider", block_state_provider()),
        req("inner_placements", non_empty_list(block_state())),
        req("cannot_replace", block_tag_hashed()),
        req("invalid_blocks", block_tag_hashed()),
    ])
}

/// `GeodeLayerSettings.CODEC`.
fn geode_layer_settings() -> Codec {
    let range = || double_range(0.01, 50.0);
    record(vec![
        or_else("filling", range(), json!(1.7)),
        or_else("inner_layer", range(), json!(2.2)),
        or_else("middle_layer", range(), json!(3.2)),
        or_else("outer_layer", range(), json!(4.2)),
    ])
}

/// `GeodeCrackSettings.CODEC`.
fn geode_crack_settings() -> Codec {
    record(vec![
        or_else("generate_crack_chance", chance_range(), json!(1.0)),
        or_else("base_crack_size", double_range(0.0, 5.0), json!(2.0)),
        or_else("crack_point_offset", int_range(0, 10), json!(2)),
    ])
}

/// `UniformInt.of(min, max)` as the JSON `orElse` defaults are encoded to.
fn uniform(min: i32, max: i32) -> serde_json::Value {
    json!({ "type": "minecraft:uniform", "min_inclusive": min, "max_inclusive": max })
}

/// `GeodeConfiguration.CODEC`.
pub fn geode() -> Codec {
    record(vec![
        req("blocks", geode_block_settings()),
        req("layers", geode_layer_settings()),
        req("crack", geode_crack_settings()),
        or_else("use_potential_placements_chance", chance_range(), json!(0.35)),
        or_else("use_alternate_layer0_chance", chance_range(), json!(0.0)),
        or_else("placements_require_layer0_alternate", bool_codec(), json!(true)),
        or_else("outer_wall_distance", int_provider(1, 20), uniform(4, 5)),
        or_else("distribution_points", int_provider(1, 20), uniform(3, 4)),
        or_else("point_offset", int_provider(0, 10), uniform(1, 2)),
        or_else("min_gen_offset", int_codec(), json!(-16)),
        or_else("max_gen_offset", int_codec(), json!(16)),
        or_else("noise_multiplier", chance_range(), json!(0.05)),
        req("invalid_blocks_threshold", int_codec()),
    ])
}

/// `DripstoneClusterConfiguration.CODEC`.
pub fn dripstone_cluster() -> Codec {
    record(vec![
        req("floor_to_ceiling_search_range", int_range(1, 512)),
        req("height", int_provider(1, 128)),
        req("radius", int_provider(1, 128)),
        req("max_stalagmite_stalactite_height_diff", int_range(0, 64)),
        req("height_deviation", int_range(1, 64)),
        req("dripstone_block_layer_thickness", int_provider(0, 128)),
        req("density", float_provider(0.0, 2.0)),
        req("wetness", float_provider(0.0, 2.0)),
        req(
            "chance_of_dripstone_column_at_max_distance_from_center",
            probability(),
        ),
        req(
            "max_distance_from_edge_affecting_chance_of_dripstone_column",
            int_range(1, 64),
        ),
        req(
            "max_distance_from_center_affecting_height_bias",
            int_range(1, 64),
        ),
    ])
}

/// `LargeDripstoneConfiguration.CODEC`.
pub fn large_dripstone() -> Codec {
    record(vec![
        or_else("floor_to_ceiling_search_range", int_range(1, 512), json!(30)),
        req("column_radius", int_provider(1, 60)),
        req("height_scale", float_provider(0.0, 20.0)),
        req(
            "max_column_radius_to_cave_height_ratio",
            float_range(0.1, 1.0),
        ),
        req("stalactite_bluntness", float_provider(0.1, 10.0)),
        req("stalagmite_bluntness", float_provider(0.1, 10.0)),
        req("wind_speed", float_provider(0.0, 2.0)),
        req("min_radius_for_wind", int_range(0, 100)),
        req("min_bluntness_for_wind", float_range(0.0, 5.0)),
    ])
}

/// `PointedDripstoneConfiguration.CODEC`.
pub fn pointed_dripstone() -> Codec {
    record(vec![
        or_else("chance_of_taller_dripstone", probability(), json!(0.2)),
        or_else("chance_of_directional_spread", probability(), json!(0.7)),
        or_else("chance_of_spread_radius2", probability(), json!(0.5)),
        or_else("chance_of_spread_radius3", probability(), json!(0.5)),
    ])
}

/// `SculkPatchConfiguration.CODEC`.
pub fn sculk_patch() -> Codec {
    record(vec![
        req("charge_count", int_range(1, 32)),
        req("amount_per_charge", int_range(1, 500)),
        req("spread_attempts", int_range(1, 64)),
        req("growth_rounds", int_range(0, 8)),
        req("spread_rounds", int_range(0, 8)),
        req("extra_rare_growths", any_int_provider()),
        req("catalyst_chance", probability()),
    ])
}

/// `RandomFeatureConfiguration.CODEC`.
pub fn random_selector() -> Codec {
    let weighted = record(vec![
        req("feature", placed_feature_ref()),
        req("chance", probability()),
    ]);
    record(vec![
        req("features", list(weighted)),
        req("default", placed_feature_ref()),
    ])
}

/// `SimpleRandomFeatureConfiguration.CODEC`: a non-empty set of placed features.
pub fn simple_random_selector() -> Codec {
    record(vec![req("features", placed_feature_set())])
}

/// `RandomBooleanFeatureConfiguration.CODEC`.
pub fn random_boolean_selector() -> Codec {
    record(vec![
        req("feature_true", placed_feature_ref()),
        req("feature_false", placed_feature_ref()),
    ])
}
