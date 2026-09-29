//! `BlockPredicate.CODEC` and `RuleTest.CODEC`.

use serde_json::json;

use super::block_state::{block_by_name, block_set, block_state, block_tag};
use super::values::{direction, offset_codec, offset_field};
use super::{cached_codec, typed, Variant};
use crate::registry_pipeline::codec::{
    float_codec, holder_set, lazy, list, opt_default, record, req, Codec,
};

const FLUID_REGISTRY: &str = "minecraft:fluid";

/// `StateTestingPredicate.stateTestingCodec` fields: only the `offset`.
fn state_testing() -> Vec<crate::registry_pipeline::codec::Field> {
    vec![offset_field()]
}

/// `CombiningPredicate.codec`: `{predicates: [BlockPredicate]}`.
fn combining() -> Codec {
    record(vec![req("predicates", list(lazy(block_predicate)))])
}

const BLOCK_PREDICATE_TYPES: &[Variant] = &[
    ("matching_blocks", || {
        let mut fields = state_testing();
        fields.push(req("blocks", block_set()));
        record(fields)
    }),
    ("matching_block_tag", || {
        let mut fields = state_testing();
        fields.push(req("tag", block_tag()));
        record(fields)
    }),
    ("matching_fluids", || {
        let mut fields = state_testing();
        fields.push(req("fluids", holder_set(FLUID_REGISTRY, false)));
        record(fields)
    }),
    ("has_sturdy_face", || {
        record(vec![offset_field(), req("direction", direction())])
    }),
    ("solid", || record(state_testing())),
    ("replaceable", || record(state_testing())),
    ("would_survive", || {
        record(vec![offset_field(), req("state", block_state())])
    }),
    ("inside_world_bounds", || {
        record(vec![opt_default(
            "offset",
            offset_codec(16),
            json!([0, 0, 0]),
        )])
    }),
    ("any_of", combining),
    ("all_of", combining),
    ("not", || record(vec![req("predicate", lazy(block_predicate))])),
    ("true", || record(vec![])),
    ("unobstructed", || {
        // `Vec3i.CODEC` without the offset range check.
        record(vec![opt_default(
            "offset",
            super::values::block_pos(),
            json!([0, 0, 0]),
        )])
    }),
];

cached_codec! {
    /// `BlockPredicate.CODEC`.
    pub fn block_predicate() -> Codec {
        typed("type", "minecraft:block_predicate_type", BLOCK_PREDICATE_TYPES)
    }
}

const RULE_TEST_TYPES: &[Variant] = &[
    ("always_true", || record(vec![])),
    ("block_match", || record(vec![req("block", block_by_name())])),
    ("blockstate_match", || {
        record(vec![req("block_state", block_state())])
    }),
    ("tag_match", || record(vec![req("tag", block_tag())])),
    ("random_block_match", || {
        record(vec![
            req("block", block_by_name()),
            req("probability", float_codec()),
        ])
    }),
    ("random_blockstate_match", || {
        record(vec![
            req("block_state", block_state()),
            req("probability", float_codec()),
        ])
    }),
];

cached_codec! {
    /// `RuleTest.CODEC`: dispatches on `predicate_type`.
    pub fn rule_test() -> Codec {
        typed("predicate_type", "minecraft:rule_test", RULE_TEST_TYPES)
    }
}
