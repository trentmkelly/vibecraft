//! `FeatureConfiguration` codecs, part one: trees, blocks, ores, fluids and patches.
//!
//! Each function is the `CODEC` of the Java class named in its documentation.

use serde_json::json;

use super::block_state::{
    block_set, block_state, block_tag_hashed, fluid_state, multiface_block,
};
use super::placement::placed_feature_ref;
use super::predicates::{block_predicate, rule_test};
use super::state_providers::block_state_provider;
use super::tree::{feature_size, foliage_placer, root_placer, trunk_placer, tree_decorators};
use super::values::{
    any_int_provider, block_pos, direction, probability,
    Y_SIZE,
};
use crate::registry_pipeline::codec::{
    bool_codec, identifier_codec, int_codec, int_range, list,
    opt, opt_default, or_else, positive_int, record, req, Codec,
};
use crate::registry_pipeline::shared::int_provider;

const PROCESSOR_LIST_REGISTRY: &str = "minecraft:worldgen/processor_list";
const CAVE_SURFACES: &[&str] = &["ceiling", "floor"];

/// `NoneFeatureConfiguration.CODEC` (`MapCodec.unitCodec`): any map, nothing written.
pub fn none() -> Codec {
    record(vec![])
}

/// `ProbabilityFeatureConfiguration.CODEC`.
pub fn probability_config() -> Codec {
    record(vec![req("probability", probability())])
}

/// `CountConfiguration.CODEC`.
pub fn count_config() -> Codec {
    record(vec![req("count", int_provider(0, 256))])
}

/// `BlockStateConfiguration.CODEC`.
pub fn block_state_config() -> Codec {
    record(vec![req("state", block_state())])
}

/// `BlockPileConfiguration.CODEC`.
pub fn block_pile() -> Codec {
    record(vec![req("state_provider", block_state_provider())])
}

/// `PLACE_BELOW_OVERWORLD_TRUNKS` as the JSON `TreeConfiguration` falls back to.
fn place_below_overworld_trunks() -> serde_json::Value {
    json!({
        "type": "minecraft:rule_based_state_provider",
        "rules": [{
            "if_true": {
                "type": "minecraft:not",
                "predicate": {
                    "type": "minecraft:matching_block_tag",
                    "tag": "minecraft:cannot_replace_below_tree_trunk"
                }
            },
            "then": {
                "type": "minecraft:simple_state_provider",
                "state": { "Name": "minecraft:dirt" }
            }
        }]
    })
}

/// `TreeConfiguration.CODEC`.
pub fn tree() -> Codec {
    record(vec![
        req("trunk_provider", block_state_provider()),
        req("trunk_placer", trunk_placer()),
        req("foliage_provider", block_state_provider()),
        req("foliage_placer", foliage_placer()),
        opt("root_placer", root_placer()),
        req("minimum_size", feature_size()),
        req("decorators", tree_decorators()),
        or_else("ignore_vines", bool_codec(), json!(false)),
        or_else(
            "below_trunk_provider",
            block_state_provider(),
            place_below_overworld_trunks(),
        ),
    ])
}

/// `FallenTreeConfiguration.CODEC`.
pub fn fallen_tree() -> Codec {
    record(vec![
        req("trunk_provider", block_state_provider()),
        req("log_length", int_provider(0, 16)),
        req("stump_decorators", tree_decorators()),
        req("log_decorators", tree_decorators()),
    ])
}

/// `SpringConfiguration.CODEC`.
pub fn spring() -> Codec {
    record(vec![
        req("state", fluid_state()),
        or_else("requires_block_below", bool_codec(), json!(true)),
        or_else("rock_count", int_codec(), json!(4)),
        or_else("hole_count", int_codec(), json!(1)),
        req("valid_blocks", block_set()),
    ])
}

/// `OreConfiguration.TargetBlockState.CODEC`.
fn target_block_state() -> Codec {
    record(vec![
        req("target", rule_test()),
        req("state", block_state()),
    ])
}

/// `OreConfiguration.CODEC` (also used by the scattered ore feature).
pub fn ore() -> Codec {
    record(vec![
        req("targets", list(target_block_state())),
        req("size", int_range(0, 64)),
        req("discard_chance_on_air_exposure", probability()),
    ])
}

/// `ReplaceBlockConfiguration.CODEC`.
pub fn replace_block() -> Codec {
    record(vec![req("targets", list(target_block_state()))])
}

/// `StructureProcessorType.LIST_CODEC` reference (a `processor_list` id).
///
/// TODO(registry-pipeline-worldgen-structures): inline processor lists need the
/// `processor_list` codecs; vanilla only references registered lists.
fn processor_list_ref() -> Codec {
    Codec::new(|json, ctx| {
        let id = crate::registry_pipeline::codec::parse_identifier(json).map_err(|_| {
            "Inline processor lists are not supported (TODO(registry-pipeline-worldgen-structures))"
                .to_string()
        })?;
        crate::registry_pipeline::codec::holder_fixed(PROCESSOR_LIST_REGISTRY)
            .parse(&serde_json::Value::String(id.to_string()), ctx)
    })
}

/// `FossilFeatureConfiguration.CODEC`.
pub fn fossil() -> Codec {
    record(vec![
        req("fossil_structures", list(identifier_codec())),
        req("overlay_structures", list(identifier_codec())),
        req("fossil_processors", processor_list_ref()),
        req("overlay_processors", processor_list_ref()),
        req("max_empty_corners_allowed", int_range(0, 7)),
    ])
}

/// `HugeMushroomFeatureConfiguration.CODEC`.
pub fn huge_mushroom() -> Codec {
    record(vec![
        req("cap_provider", block_state_provider()),
        req("stem_provider", block_state_provider()),
        or_else("foliage_radius", int_codec(), json!(2)),
        req("can_place_on", block_predicate()),
    ])
}

/// `SpikeConfiguration.CODEC`.
pub fn spike() -> Codec {
    record(vec![
        req("state", block_state()),
        req("can_place_on", block_predicate()),
        req("can_replace", block_predicate()),
    ])
}

/// `BlockBlobConfiguration.CODEC`.
pub fn block_blob() -> Codec {
    record(vec![
        req("state", block_state()),
        req("can_place_on", block_predicate()),
    ])
}

/// `BlockColumnConfiguration.CODEC`.
pub fn block_column() -> Codec {
    let layer = record(vec![
        req("height", super::values::non_negative_int_provider()),
        req("provider", block_state_provider()),
    ]);
    record(vec![
        req("layers", list(layer)),
        req("direction", direction()),
        req("allowed_placement", block_predicate()),
        req("prioritize_tip", bool_codec()),
    ])
}

/// `VegetationPatchConfiguration.CODEC`.
pub fn vegetation_patch() -> Codec {
    record(vec![
        req("replaceable", block_tag_hashed()),
        req("ground_state", block_state_provider()),
        req("vegetation_feature", placed_feature_ref()),
        req("surface", crate::registry_pipeline::codec::enum_codec(CAVE_SURFACES)),
        req("depth", int_provider(1, 128)),
        req("extra_bottom_block_chance", probability()),
        req("vertical_range", int_range(1, 256)),
        req("vegetation_chance", probability()),
        req("xz_radius", any_int_provider()),
        req("extra_edge_column_chance", probability()),
    ])
}

/// `RootSystemConfiguration.CODEC`.
pub fn root_system() -> Codec {
    record(vec![
        req("feature", placed_feature_ref()),
        req("required_vertical_space_for_tree", int_range(1, 64)),
        req("root_radius", int_range(1, 64)),
        req("root_replaceable", block_tag_hashed()),
        req("root_state_provider", block_state_provider()),
        req("root_placement_attempts", int_range(1, 256)),
        req("root_column_max_height", int_range(1, 4096)),
        req("hanging_root_radius", int_range(1, 64)),
        req("hanging_roots_vertical_span", int_range(1, 16)),
        req("hanging_root_state_provider", block_state_provider()),
        req("hanging_root_placement_attempts", int_range(1, 256)),
        req("allowed_vertical_water_for_tree", int_range(1, 64)),
        req("allowed_tree_position", block_predicate()),
    ])
}

/// `MultifaceGrowthConfiguration.CODEC`.
pub fn multiface_growth() -> Codec {
    record(vec![
        or_else("block", multiface_block(), json!("minecraft:glow_lichen")),
        or_else("search_range", int_range(1, 64), json!(10)),
        or_else("can_place_on_floor", bool_codec(), json!(false)),
        or_else("can_place_on_ceiling", bool_codec(), json!(false)),
        or_else("can_place_on_wall", bool_codec(), json!(false)),
        or_else("chance_of_spreading", probability(), json!(0.5)),
        req("can_be_placed_on", block_set()),
    ])
}

/// `UnderwaterMagmaConfiguration.CODEC`.
pub fn underwater_magma() -> Codec {
    record(vec![
        req("floor_search_range", int_range(0, 512)),
        req("placement_radius_around_floor", int_range(0, 64)),
        req("placement_probability_per_valid_position", probability()),
    ])
}

/// `NetherForestVegetationConfig.CODEC`.
pub fn nether_forest_vegetation() -> Codec {
    record(vec![
        req("state_provider", block_state_provider()),
        req("spread_width", positive_int()),
        req("spread_height", positive_int()),
    ])
}

/// `TwistingVinesConfig.CODEC`.
pub fn twisting_vines() -> Codec {
    record(vec![
        req("spread_width", positive_int()),
        req("spread_height", positive_int()),
        req("max_height", positive_int()),
    ])
}

/// `ColumnFeatureConfiguration.CODEC`.
pub fn column() -> Codec {
    record(vec![
        req("reach", int_provider(0, 3)),
        req("height", int_provider(1, 10)),
    ])
}

/// `DeltaFeatureConfiguration.CODEC`.
pub fn delta() -> Codec {
    record(vec![
        req("contents", block_state()),
        req("rim", block_state()),
        req("size", int_provider(0, 16)),
        req("rim_size", int_provider(0, 16)),
    ])
}

/// `ReplaceSphereConfiguration.CODEC`.
pub fn replace_sphere() -> Codec {
    record(vec![
        req("target", block_state()),
        req("state", block_state()),
        req("radius", int_provider(0, 12)),
    ])
}

/// `LayerConfiguration.CODEC`.
pub fn layer() -> Codec {
    record(vec![
        req("height", int_range(0, Y_SIZE)),
        req("state", block_state()),
    ])
}

/// `DiskConfiguration.CODEC`.
pub fn disk() -> Codec {
    record(vec![
        req("state_provider", block_state_provider()),
        req("target", block_predicate()),
        req("radius", int_provider(0, 8)),
        req("half_height", int_range(0, 4)),
    ])
}

/// `LakeFeature.Configuration.CODEC`.
pub fn lake() -> Codec {
    record(vec![
        req("fluid", block_state_provider()),
        req("barrier", block_state_provider()),
    ])
}

/// `EndSpikeFeature.EndSpike.CODEC`.
fn end_spike_definition() -> Codec {
    record(vec![
        or_else("centerX", int_codec(), json!(0)),
        or_else("centerZ", int_codec(), json!(0)),
        or_else("radius", int_codec(), json!(0)),
        or_else("height", int_codec(), json!(0)),
        or_else("guarded", bool_codec(), json!(false)),
    ])
}

/// `EndSpikeConfiguration.CODEC`.
pub fn end_spike() -> Codec {
    record(vec![
        or_else("crystal_invulnerable", bool_codec(), json!(false)),
        req("spikes", list(end_spike_definition())),
        opt("crystal_beam_target", block_pos()),
    ])
}

/// `EndGatewayConfiguration.CODEC`.
pub fn end_gateway() -> Codec {
    record(vec![
        opt("exit", block_pos()),
        req("exact", bool_codec()),
    ])
}

/// `SimpleBlockConfiguration.CODEC`.
pub fn simple_block() -> Codec {
    record(vec![
        req("to_place", block_state_provider()),
        opt_default("schedule_tick", bool_codec(), json!(false)),
    ])
}

/// `HugeFungusConfiguration.CODEC`.
pub fn huge_fungus() -> Codec {
    record(vec![
        req("valid_base_block", block_state()),
        req("stem_state", block_state()),
        req("hat_state", block_state()),
        req("decor_state", block_state()),
        req("replaceable_blocks", block_predicate()),
        or_else("planted", bool_codec(), json!(false)),
    ])
}
