//! Codec for `worldgen/template_pool` (`StructureTemplatePool.DIRECT_CODEC`) and every
//! `StructurePoolElement` kind (`StructurePoolElementType`), plus the pool alias
//! bindings and `DimensionPadding` used by jigsaw structures.

use serde_json::json;

use crate::registry_pipeline::codec::{
    either, enum_codec, holder_file, identifier_codec, int_range, lazy, list, non_negative_int,
    opt, record, req, Codec, CodecResult, Field,
};
use crate::registry_pipeline::element_codecs::worldgen_reference_target;
use crate::registry_pipeline::structure_processor_codecs::processor_list_holder;
use crate::registry_pipeline::worldgen_common::{
    non_empty_weighted_list, registry_dispatch, unit, Variant,
};
use crate::storage::nbt::Tag;

/// `Registries.TEMPLATE_POOL`.
pub const TEMPLATE_POOL_REGISTRY: &str = "minecraft:worldgen/template_pool";
/// `Registries.PLACED_FEATURE`.
const PLACED_FEATURE_REGISTRY: &str = "minecraft:worldgen/placed_feature";

/// `StructureTemplatePool.Projection.CODEC`.
fn projection() -> Field {
    req("projection", enum_codec(&["terrain_matching", "rigid"]))
}

/// `LiquidSettings.CODEC`.
pub fn liquid_settings() -> Codec {
    enum_codec(&["ignore_waterlogging", "apply_waterlogging"])
}

/// `StructureTemplatePool.DIRECT_CODEC`: `{fallback, elements: [{element, weight}]}`.
pub fn template_pool() -> Codec {
    record(vec![
        req("fallback", lazy(template_pool_holder)),
        req(
            "elements",
            list(record(vec![
                req("element", lazy(pool_element)),
                req("weight", int_range(1, 150)),
            ])),
        ),
    ])
}

/// `StructureTemplatePool.CODEC`: a `worldgen/template_pool` reference or an inline
/// pool.
pub fn template_pool_holder() -> Codec {
    holder_file(TEMPLATE_POOL_REGISTRY, lazy(template_pool))
}

/// `StructurePoolElement.CODEC`.
pub fn pool_element() -> Codec {
    registry_dispatch(
        "element_type",
        "minecraft:worldgen/structure_pool_element",
        POOL_ELEMENTS,
    )
}

/// `StructurePoolElementType` registrations.
const POOL_ELEMENTS: &[Variant] = &[
    ("single_pool_element", single_pool_element),
    ("list_pool_element", list_pool_element),
    ("feature_pool_element", feature_pool_element),
    ("empty_pool_element", unit),
    ("legacy_single_pool_element", single_pool_element),
];

/// `SinglePoolElement.CODEC` / `LegacySinglePoolElement.CODEC`: the template is an
/// `Identifier` (`TEMPLATE_CODEC` cannot encode a runtime template).
fn single_pool_element() -> Codec {
    record(vec![
        req("location", identifier_codec()),
        req("processors", processor_list_holder()),
        projection(),
        opt("override_liquid_settings", liquid_settings()),
    ])
}

/// `ListPoolElement.CODEC`; the constructor rejects an empty element list.
fn list_pool_element() -> Codec {
    record(vec![
        req("elements", list(lazy(pool_element))),
        projection(),
    ])
    .validate(|tag| match tag {
        Tag::Compound(fields)
            if fields
                .iter()
                .any(|(key, value)| key == "elements" && *value == Tag::List(Vec::new())) =>
        {
            Err("Elements are empty".to_string())
        }
        _ => Ok(()),
    })
}

/// `FeaturePoolElement.CODEC`.
///
/// TODO(registry-pipeline-worldgen-placed-feature): an inline `PlacedFeature` is only
/// decoded generically until `PlacedFeature.DIRECT_CODEC` is ported.
fn feature_pool_element() -> Codec {
    record(vec![
        req(
            "feature",
            holder_file(PLACED_FEATURE_REGISTRY, worldgen_reference_target()),
        ),
        projection(),
    ])
}

// ---------------------------------------------------------------------------
// Jigsaw structure helpers
// ---------------------------------------------------------------------------

/// `PoolAliasBinding.CODEC`.
pub fn pool_alias_binding() -> Codec {
    registry_dispatch(
        "type",
        "minecraft:worldgen/pool_alias_binding",
        POOL_ALIAS_BINDINGS,
    )
}

/// `PoolAliasBindings.bootstrap` registrations.
const POOL_ALIAS_BINDINGS: &[Variant] = &[
    ("random", random_alias),
    ("random_group", random_group_alias),
    ("direct", direct_alias),
];

/// `ResourceKey.codec(Registries.TEMPLATE_POOL)`: a plain identifier.
fn pool_key() -> Codec {
    identifier_codec()
}

fn direct_alias() -> Codec {
    record(vec![req("alias", pool_key()), req("target", pool_key())])
}

fn random_alias() -> Codec {
    record(vec![
        req("alias", pool_key()),
        req("targets", non_empty_weighted_list(pool_key())),
    ])
}

fn random_group_alias() -> Codec {
    record(vec![req(
        "groups",
        non_empty_weighted_list(list(lazy(pool_alias_binding))),
    )])
}

/// `DimensionPadding.CODEC`: a non-negative int (equal top and bottom) or
/// `{bottom, top}` with lenient optional non-negative ints.
pub fn dimension_padding() -> Codec {
    use crate::registry_pipeline::codec::lenient_opt_default;
    let object = record(vec![
        lenient_opt_default("bottom", non_negative_int(), json!(0)),
        lenient_opt_default("top", non_negative_int(), json!(0)),
    ]);
    either(non_negative_int(), object).map_tag(padding_encode)
}

/// `padding.hasEqualTopAndBottom() ? Either.left(bottom) : Either.right(padding)`.
fn padding_encode(tag: Tag) -> CodecResult<Tag> {
    let Tag::Compound(fields) = &tag else {
        return Ok(tag);
    };
    let get = |name: &str| {
        fields
            .iter()
            .find_map(|(key, value)| match value {
                Tag::Int(v) if key == name => Some(*v),
                _ => None,
            })
            .unwrap_or(0)
    };
    let (top, bottom) = (get("top"), get("bottom"));
    Ok(if top == bottom { Tag::Int(bottom) } else { tag })
}
