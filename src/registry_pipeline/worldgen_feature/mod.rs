//! Typed codecs for `worldgen/configured_feature` and `worldgen/placed_feature`.
//!
//! Every class reachable from `ConfiguredFeature.DIRECT_CODEC` and
//! `PlacedFeature.DIRECT_CODEC` is modelled here, one function per Java codec:
//!
//! - [`values`]: value/height providers, vertical anchors, positions, directions.
//! - [`block_state`]: `BlockState.CODEC`, `FluidState.CODEC` and block holder sets.
//! - [`predicates`]: `BlockPredicate.CODEC` and `RuleTest.CODEC`.
//! - [`state_providers`]: `BlockStateProvider.CODEC` and `RuleBasedStateProvider`.
//! - [`tree`]: trunk/foliage/root placers, tree decorators and feature sizes.
//! - [`configs`], [`configs_more`]: the `FeatureConfiguration` codecs.
//! - [`placement`]: `PlacementModifier.CODEC`, `PlacedFeature`, `ConfiguredFeature`.
//!
//! The codecs follow the conventions of [`crate::registry_pipeline::codec`]: input is
//! JSON, output the NBT the Java encoder would have produced. Neither registry is
//! synchronised to clients, so the output only has to round-trip (see
//! [`tag_to_json`]). Registry `type` dispatch fails for unknown types like Java's
//! `Registry.byNameCodec()`; unknown record fields are ignored like
//! `RecordCodecBuilder` does.

use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(test)]
use serde_json::{Map, Number, Value as Json};

use crate::registry_pipeline::codec::{self, Codec};
#[cfg(test)]
use crate::storage::nbt::Tag;

pub mod block_state;
pub mod configs;
pub mod configs_more;
pub mod placement;
pub mod predicates;
pub mod state_providers;
pub mod tree;
pub mod values;

pub use placement::{configured_feature, placed_feature};

/// `ConfiguredFeature`'s registry key.
pub(crate) const CONFIGURED_FEATURE_REGISTRY: &str = "minecraft:worldgen/configured_feature";
/// `PlacedFeature`'s registry key.
pub(crate) const PLACED_FEATURE_REGISTRY: &str = "minecraft:worldgen/placed_feature";
/// The static block registry.
pub(crate) const BLOCK_REGISTRY: &str = "minecraft:block";

/// One entry of a `Registry`-dispatched codec table: the registered path in the
/// `minecraft` namespace and the builder of its `MapCodec`.
pub(crate) type Variant = (&'static str, fn() -> Codec);

/// Defines a codec function whose (potentially large) codec tree is built once per
/// thread. Recursive definitions refer to themselves through [`codec::lazy`].
macro_rules! cached_codec {
    ($(#[$meta:meta])* $vis:vis fn $name:ident() -> Codec $body:block) => {
        $(#[$meta])*
        $vis fn $name() -> $crate::registry_pipeline::codec::Codec {
            thread_local! {
                static CODEC: $crate::registry_pipeline::codec::Codec = $body;
            }
            CODEC.with(Clone::clone)
        }
    };
}
pub(crate) use cached_codec;

/// `registry.byNameCodec().dispatch(typeKey, ...)`: selects the variant codec by the
/// `type_key` identifier. Variant codecs are built on first use and cached.
pub(crate) fn typed(
    type_key: &'static str,
    registry: &'static str,
    table: &'static [Variant],
) -> Codec {
    let built: RefCell<HashMap<&'static str, Codec>> = RefCell::new(HashMap::new());
    codec::dispatch(type_key, move |id| {
        let variant = table
            .iter()
            .find(|(path, _)| id.namespace() == "minecraft" && id.path() == *path)
            .ok_or_else(|| {
                format!("Unknown registry key in ResourceKey[minecraft:root / {registry}]: {id}")
            })?;
        Ok(built
            .borrow_mut()
            .entry(variant.0)
            .or_insert_with(variant.1)
            .clone())
    })
}

#[cfg(test)]
/// Converts an encoded element back to the JSON a data pack would hold.
///
/// Byte tags are booleans (`Codec.BOOL`, the only byte-valued codec of these
/// registries); floats print their shortest `f32` representation like Java's
/// `JsonPrimitive(Float)`.
pub fn tag_to_json(tag: &Tag) -> Json {
    match tag {
        Tag::End => Json::Null,
        Tag::Byte(v) => Json::Bool(*v != 0),
        Tag::Short(v) => Json::from(*v),
        Tag::Int(v) => Json::from(*v),
        Tag::Long(v) => Json::from(*v),
        Tag::Float(v) => v
            .to_string()
            .parse::<f64>()
            .ok()
            .and_then(Number::from_f64)
            .map_or(Json::Null, Json::Number),
        Tag::Double(v) => Number::from_f64(*v).map_or(Json::Null, Json::Number),
        Tag::String(v) => Json::String(v.clone()),
        Tag::List(items) => Json::Array(items.iter().map(tag_to_json).collect()),
        Tag::Compound(fields) => Json::Object(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), tag_to_json(value)))
                .collect::<Map<_, _>>(),
        ),
        Tag::ByteArray(v) => Json::from(v.clone()),
        Tag::IntArray(v) => Json::from(v.clone()),
        Tag::LongArray(v) => Json::from(v.clone()),
    }
}

#[cfg(test)]
/// Structural JSON equality where numbers compare by value (`1` equals `1.0`).
pub fn json_semantically_equal(left: &Json, right: &Json) -> bool {
    match (left, right) {
        (Json::Number(a), Json::Number(b)) => a.as_f64() == b.as_f64(),
        (Json::Array(a), Json::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| json_semantically_equal(x, y))
        }
        (Json::Object(a), Json::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, x)| {
                    b.get(key)
                        .is_some_and(|y| json_semantically_equal(x, y))
                })
        }
        _ => left == right,
    }
}
