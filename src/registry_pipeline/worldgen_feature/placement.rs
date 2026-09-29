//! `PlacementModifier.CODEC`, `PlacedFeature` and `ConfiguredFeature` (plus the
//! `Feature` registry that selects the configuration codec).

use serde_json::{json, Value as Json};

use super::predicates::block_predicate;
use super::values::{
    block_pos, height_provider, heightmap_type, vertical_direction,
};
use super::{
    configs as c, configs_more as m, cached_codec, typed, Variant, CONFIGURED_FEATURE_REGISTRY,
    PLACED_FEATURE_REGISTRY,
};
use crate::registry_pipeline::codec::{
    describe, double_codec, holder_file, holder_set, int_codec, int_range, lazy, list,
    opt_default, or_else, positive_int, record, req, Codec,
};
use crate::registry_pipeline::shared::int_provider;
use crate::storage::nbt::Tag;

// ---------------------------------------------------------------------------
// Placement modifiers
// ---------------------------------------------------------------------------

const PLACEMENT_MODIFIER_TYPES: &[Variant] = &[
    ("block_predicate_filter", || {
        record(vec![req("predicate", block_predicate())])
    }),
    ("rarity_filter", || record(vec![req("chance", positive_int())])),
    ("surface_relative_threshold_filter", || {
        record(vec![
            req("heightmap", heightmap_type()),
            opt_default("min_inclusive", int_codec(), json!(i32::MIN)),
            opt_default("max_inclusive", int_codec(), json!(i32::MAX)),
        ])
    }),
    ("surface_water_depth_filter", || {
        record(vec![req("max_water_depth", int_codec())])
    }),
    ("biome", || record(vec![])),
    ("count", || record(vec![req("count", int_provider(0, 4096))])),
    ("noise_based_count", || {
        record(vec![
            req("noise_to_count_ratio", int_codec()),
            req("noise_factor", double_codec()),
            or_else("noise_offset", double_codec(), json!(0.0)),
        ])
    }),
    ("noise_threshold_count", || {
        record(vec![
            req("noise_level", double_codec()),
            req("below_noise", int_codec()),
            req("above_noise", int_codec()),
        ])
    }),
    ("count_on_every_layer", || {
        record(vec![req("count", int_provider(0, 256))])
    }),
    ("environment_scan", || {
        record(vec![
            req("direction_of_search", vertical_direction()),
            req("target_condition", block_predicate()),
            opt_default(
                "allowed_search_condition",
                block_predicate(),
                json!({ "type": "minecraft:true" }),
            ),
            req("max_steps", int_range(1, 32)),
        ])
    }),
    ("heightmap", || record(vec![req("heightmap", heightmap_type())])),
    ("height_range", || record(vec![req("height", height_provider())])),
    ("in_square", || record(vec![])),
    ("random_offset", || {
        record(vec![
            req("xz_spread", int_provider(-16, 16)),
            req("y_spread", int_provider(-16, 16)),
        ])
    }),
    ("fixed_placement", || {
        record(vec![req("positions", list(block_pos()))])
    }),
];

cached_codec! {
    /// `PlacementModifier.CODEC`.
    pub fn placement_modifier() -> Codec {
        typed("type", "minecraft:placement_modifier_type", PLACEMENT_MODIFIER_TYPES)
    }
}

// ---------------------------------------------------------------------------
// Placed features
// ---------------------------------------------------------------------------

cached_codec! {
    /// `PlacedFeature.DIRECT_CODEC`: `{feature, placement}`.
    pub fn placed_feature() -> Codec {
        record(vec![
            req("feature", lazy(configured_feature_ref)),
            req("placement", list(placement_modifier())),
        ])
    }
}

cached_codec! {
    /// `PlacedFeature.CODEC`: a registry reference or an inline placed feature.
    pub fn placed_feature_ref() -> Codec {
        holder_file(PLACED_FEATURE_REGISTRY, lazy(placed_feature))
    }
}

cached_codec! {
    /// `ExtraCodecs.nonEmptyHolderSet(PlacedFeature.LIST_CODEC)`.
    pub fn placed_feature_set() -> Codec {
        holder_file_set(PLACED_FEATURE_REGISTRY, lazy(placed_feature))
            .validate(|tag| match tag {
                Tag::List(items) if items.is_empty() => Err("List must have contents".to_string()),
                _ => Ok(()),
            })
    }
}

/// `RegistryCodecs.homogeneousList(registry, directCodec)`: a `#tag`, or a list of
/// references / inline elements (a lone element is accepted and a one element list is
/// written bare). A list may not mix references and inline elements.
fn holder_file_set(registry: &'static str, direct: Codec) -> Codec {
    Codec::new(move |json, ctx| {
        if matches!(json, Json::String(text) if text.starts_with('#')) {
            return holder_set(registry, false).parse(json, ctx);
        }
        let element = holder_file(registry, direct.clone());
        let mut items = match json {
            Json::Array(values) => values
                .iter()
                .map(|value| element.parse(value, ctx))
                .collect::<Result<Vec<_>, _>>()?,
            other => vec![element.parse(other, ctx)?],
        };
        let is_reference = |tag: &Tag| matches!(tag, Tag::String(_));
        if let Some(first) = items.first().map(is_reference) {
            if items.iter().any(|item| is_reference(item) != first) {
                return Err(format!(
                    "Mixed type list: references and direct holders in {}",
                    describe(json)
                ));
            }
        }
        Ok(if items.len() == 1 {
            items.remove(0)
        } else {
            Tag::List(items)
        })
    })
}

// ---------------------------------------------------------------------------
// Configured features
// ---------------------------------------------------------------------------

/// `Feature.configuredCodec`: the `config` field decoded by the feature's codec.
macro_rules! feature {
    ($name:literal, $config:expr) => {
        ($name, || record(vec![req("config", $config)]))
    };
}

/// The `BuiltInRegistries.FEATURE` entries with their `FeatureConfiguration` codec.
const FEATURE_TYPES: &[Variant] = &[
    feature!("no_op", c::none()),
    feature!("tree", c::tree()),
    feature!("fallen_tree", c::fallen_tree()),
    feature!("block_pile", c::block_pile()),
    feature!("spring_feature", c::spring()),
    feature!("chorus_plant", c::none()),
    feature!("replace_single_block", c::replace_block()),
    feature!("void_start_platform", c::none()),
    feature!("desert_well", c::none()),
    feature!("fossil", c::fossil()),
    feature!("huge_red_mushroom", c::huge_mushroom()),
    feature!("huge_brown_mushroom", c::huge_mushroom()),
    feature!("spike", c::spike()),
    feature!("glowstone_blob", c::none()),
    feature!("freeze_top_layer", c::none()),
    feature!("vines", c::none()),
    feature!("block_column", c::block_column()),
    feature!("vegetation_patch", c::vegetation_patch()),
    feature!("waterlogged_vegetation_patch", c::vegetation_patch()),
    feature!("root_system", c::root_system()),
    feature!("multiface_growth", c::multiface_growth()),
    feature!("underwater_magma", c::underwater_magma()),
    feature!("monster_room", c::none()),
    feature!("blue_ice", c::none()),
    feature!("iceberg", c::block_state_config()),
    feature!("block_blob", c::block_blob()),
    feature!("disk", c::disk()),
    feature!("lake", c::lake()),
    feature!("ore", c::ore()),
    feature!("end_platform", c::none()),
    feature!("end_spike", c::end_spike()),
    feature!("end_island", c::none()),
    feature!("end_gateway", c::end_gateway()),
    feature!("seagrass", c::probability_config()),
    feature!("kelp", c::none()),
    feature!("coral_tree", c::none()),
    feature!("coral_mushroom", c::none()),
    feature!("coral_claw", c::none()),
    feature!("sea_pickle", c::count_config()),
    feature!("simple_block", c::simple_block()),
    feature!("bamboo", c::probability_config()),
    feature!("huge_fungus", c::huge_fungus()),
    feature!("nether_forest_vegetation", c::nether_forest_vegetation()),
    feature!("weeping_vines", c::none()),
    feature!("twisting_vines", c::twisting_vines()),
    feature!("basalt_columns", c::column()),
    feature!("delta_feature", c::delta()),
    feature!("netherrack_replace_blobs", c::replace_sphere()),
    feature!("fill_layer", c::layer()),
    feature!("bonus_chest", c::none()),
    feature!("basalt_pillar", c::none()),
    feature!("scattered_ore", c::ore()),
    feature!("random_selector", m::random_selector()),
    feature!("simple_random_selector", m::simple_random_selector()),
    feature!("random_boolean_selector", m::random_boolean_selector()),
    feature!("geode", m::geode()),
    feature!("dripstone_cluster", m::dripstone_cluster()),
    feature!("large_dripstone", m::large_dripstone()),
    feature!("pointed_dripstone", m::pointed_dripstone()),
    feature!("sculk_patch", m::sculk_patch()),
];

cached_codec! {
    /// `ConfiguredFeature.DIRECT_CODEC`: `{type, config}` dispatched on the feature.
    pub fn configured_feature() -> Codec {
        typed("type", "minecraft:worldgen/feature", FEATURE_TYPES)
    }
}

cached_codec! {
    /// `ConfiguredFeature.CODEC`: a registry reference or an inline configured feature.
    pub fn configured_feature_ref() -> Codec {
        holder_file(CONFIGURED_FEATURE_REGISTRY, lazy(configured_feature))
    }
}
