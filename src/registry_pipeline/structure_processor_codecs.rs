//! Codecs for `worldgen/processor_list` (`StructureProcessorType.DIRECT_CODEC`) and
//! the template system pieces a processor list is made of: `StructureProcessor`
//! kinds, `ProcessorRule`, `RuleTest`, `PosRuleTest` and `RuleBlockEntityModifier`.

use serde_json::{json, Value as Json};

use crate::registry_pipeline::codec::{
    describe, either, enum_codec, float_codec, float_range, holder_file, holder_set,
    identifier_codec, int_codec, lazy, lenient_opt_default, list, opt, or_else, record, req,
    tag_key_hashed, Codec, Field,
};
use crate::registry_pipeline::shared::int_provider;
use crate::registry_pipeline::worldgen_common::{
    block_id, block_state, compound_tag, heightmap_types, registry_dispatch, unit, Variant,
};
use crate::storage::nbt::Tag;

/// `Registries.PROCESSOR_LIST`.
pub const PROCESSOR_LIST_REGISTRY: &str = "minecraft:worldgen/processor_list";
/// `Registries.BLOCK`.
const BLOCK_REGISTRY: &str = "minecraft:block";

/// `StructureProcessorType.DIRECT_CODEC`: `{processors: [...]}` or a bare list.
pub fn processor_list() -> Codec {
    let processors = || list(lazy(structure_processor));
    either(record(vec![req("processors", processors())]), processors())
        // `Codec.withAlternative` decodes either form but always encodes the first one.
        .map_tag(|tag| {
            Ok(match tag {
                Tag::List(_) => Tag::Compound(vec![("processors".to_string(), tag)]),
                other => other,
            })
        })
}

/// `StructureProcessorType.LIST_CODEC`: a `worldgen/processor_list` reference or an
/// inline list.
pub fn processor_list_holder() -> Codec {
    holder_file(PROCESSOR_LIST_REGISTRY, processor_list())
}

/// `StructureProcessorType.SINGLE_CODEC`.
pub fn structure_processor() -> Codec {
    registry_dispatch(
        "processor_type",
        "minecraft:worldgen/structure_processor",
        STRUCTURE_PROCESSORS,
    )
}

/// `StructureProcessorType` registrations.
const STRUCTURE_PROCESSORS: &[Variant] = &[
    ("block_ignore", block_ignore),
    ("block_rot", block_rot),
    ("gravity", gravity),
    ("jigsaw_replacement", unit),
    ("rule", rule_processor),
    ("nop", unit),
    ("block_age", block_age),
    ("blackstone_replace", unit),
    ("lava_submerged_block", unit),
    ("protected_blocks", protected_blocks),
    ("capped", capped),
];

/// `BlockState.CODEC.xmap(getBlock, Block::defaultBlockState)`: only the block is
/// kept, so the encoder writes the block's default state.
fn default_block_state() -> Codec {
    let full = block_state();
    Codec::new(move |json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        // Properties are lenient and always dropped by `getBlock`, so only the name
        // (validated by the full codec) matters.
        let mut name_only = serde_json::Map::new();
        if let Some(name) = object.get("Name") {
            name_only.insert("Name".to_string(), name.clone());
        }
        full.parse(&Json::Object(name_only), ctx)
    })
}

/// `BlockIgnoreProcessor.CODEC`.
fn block_ignore() -> Codec {
    record(vec![req("blocks", list(default_block_state()))])
}

/// `BlockRotProcessor.CODEC`.
fn block_rot() -> Codec {
    record(vec![
        opt("rottable_blocks", holder_set(BLOCK_REGISTRY, false)),
        req("integrity", float_range(0.0, 1.0)),
    ])
}

/// `GravityProcessor.CODEC`.
fn gravity() -> Codec {
    record(vec![
        or_else("heightmap", heightmap_types(), json!("WORLD_SURFACE_WG")),
        or_else("offset", int_codec(), json!(0)),
    ])
}

/// `RuleProcessor.CODEC`.
fn rule_processor() -> Codec {
    record(vec![req("rules", list(processor_rule()))])
}

/// `BlockAgeProcessor.CODEC`.
fn block_age() -> Codec {
    record(vec![req("mossiness", float_codec())])
}

/// `ProtectedBlockProcessor.CODEC`.
fn protected_blocks() -> Codec {
    record(vec![req("value", tag_key_hashed(BLOCK_REGISTRY))])
}

/// `CappedProcessor.CODEC`.
fn capped() -> Codec {
    record(vec![
        req("delegate", lazy(structure_processor)),
        req("limit", int_provider(1, i32::MAX)),
    ])
}

// ---------------------------------------------------------------------------
// Rules
// ---------------------------------------------------------------------------

/// `ProcessorRule.CODEC`.
fn processor_rule() -> Codec {
    record(vec![
        req("input_predicate", rule_test()),
        req("location_predicate", rule_test()),
        lenient_opt_default(
            "position_predicate",
            pos_rule_test(),
            json!({"predicate_type": "minecraft:always_true"}),
        ),
        req("output_state", block_state()),
        lenient_opt_default(
            "block_entity_modifier",
            rule_block_entity_modifier(),
            json!({"type": "minecraft:passthrough"}),
        ),
    ])
}

/// `RuleTest.CODEC`.
pub fn rule_test() -> Codec {
    registry_dispatch("predicate_type", "minecraft:rule_test", RULE_TESTS)
}

/// `RuleTestType` registrations.
const RULE_TESTS: &[Variant] = &[
    ("always_true", unit),
    ("block_match", block_match),
    ("blockstate_match", block_state_match),
    ("tag_match", tag_match),
    ("random_block_match", random_block_match),
    ("random_blockstate_match", random_block_state_match),
];

fn block_match() -> Codec {
    record(vec![req("block", block_id())])
}

fn block_state_match() -> Codec {
    record(vec![req("block_state", block_state())])
}

/// `TagKey.codec(Registries.BLOCK)`: a plain (unhashed) tag identifier.
fn tag_match() -> Codec {
    record(vec![req("tag", identifier_codec())])
}

fn random_block_match() -> Codec {
    record(vec![
        req("block", block_id()),
        req("probability", float_codec()),
    ])
}

fn random_block_state_match() -> Codec {
    record(vec![
        req("block_state", block_state()),
        req("probability", float_codec()),
    ])
}

/// `PosRuleTest.CODEC`.
pub fn pos_rule_test() -> Codec {
    registry_dispatch("predicate_type", "minecraft:pos_rule_test", POS_RULE_TESTS)
}

/// `PosRuleTestType` registrations.
const POS_RULE_TESTS: &[Variant] = &[
    ("always_true", unit),
    ("linear_pos", linear_pos),
    ("axis_aligned_linear_pos", axis_aligned_linear_pos),
];

fn linear_fields() -> Vec<Field> {
    vec![
        or_else("min_chance", float_codec(), json!(0.0)),
        or_else("max_chance", float_codec(), json!(0.0)),
        or_else("min_dist", int_codec(), json!(0)),
        or_else("max_dist", int_codec(), json!(0)),
    ]
}

fn linear_pos() -> Codec {
    record(linear_fields())
}

fn axis_aligned_linear_pos() -> Codec {
    let mut fields = linear_fields();
    fields.push(or_else("axis", enum_codec(&["x", "y", "z"]), json!("y")));
    record(fields)
}

/// `RuleBlockEntityModifier.CODEC`.
pub fn rule_block_entity_modifier() -> Codec {
    registry_dispatch(
        "type",
        "minecraft:rule_block_entity_modifier",
        BLOCK_ENTITY_MODIFIERS,
    )
}

/// `RuleBlockEntityModifierType` registrations.
const BLOCK_ENTITY_MODIFIERS: &[Variant] = &[
    ("clear", unit),
    ("passthrough", unit),
    ("append_static", append_static),
    ("append_loot", append_loot),
];

fn append_static() -> Codec {
    record(vec![req("data", compound_tag())])
}

/// `AppendLoot.CODEC`: `LootTable.KEY_CODEC` is a plain resource key.
fn append_loot() -> Codec {
    record(vec![req("loot_table", identifier_codec())])
}
