//! `BlockStateProvider.CODEC` and the noise/rule based providers.

use crate::registry_pipeline::codec::{
    double_codec, float_range, int_codec, lazy, list, long_codec, opt, positive_float, record,
    req, string_codec, Codec, Field,
};

use super::block_state::block_state;
use super::predicates::block_predicate;
use super::values::{
    any_int_provider, inclusive_int_range, non_empty_list, non_empty_weighted_list,
};
use super::{cached_codec, typed, Variant};

/// `NormalNoise.NoiseParameters.DIRECT_CODEC`: `{firstOctave, amplitudes}`.
pub fn noise_parameters() -> Codec {
    record(vec![
        req("firstOctave", int_codec()),
        req("amplitudes", list(double_codec())),
    ])
}

/// `NoiseBasedStateProvider.noiseCodec`: `seed`, `noise` and `scale`.
fn noise_fields() -> Vec<Field> {
    vec![
        req("seed", long_codec()),
        req("noise", noise_parameters()),
        req("scale", positive_float()),
    ]
}

/// `NoiseProvider.noiseProviderCodec`: the noise fields and the `states`.
fn noise_provider_fields() -> Vec<Field> {
    let mut fields = noise_fields();
    fields.push(req("states", non_empty_list(block_state())));
    fields
}

/// `RuleBasedStateProvider.Rule.CODEC`.
fn rule() -> Codec {
    record(vec![
        req("if_true", block_predicate()),
        req("then", lazy(block_state_provider)),
    ])
}

const BLOCK_STATE_PROVIDER_TYPES: &[Variant] = &[
    ("simple_state_provider", || {
        record(vec![req("state", block_state())])
    }),
    ("weighted_state_provider", || {
        record(vec![req("entries", non_empty_weighted_list(block_state()))])
    }),
    ("noise_threshold_provider", || {
        let mut fields = noise_fields();
        fields.extend([
            req("threshold", float_range(-1.0, 1.0)),
            req("high_chance", float_range(0.0, 1.0)),
            req("default_state", block_state()),
            req("low_states", non_empty_list(block_state())),
            req("high_states", non_empty_list(block_state())),
        ]);
        record(fields)
    }),
    ("noise_provider", || record(noise_provider_fields())),
    ("dual_noise_provider", || {
        let mut fields = vec![
            req("variety", inclusive_int_range(1, 64)),
            req("slow_noise", noise_parameters()),
            req("slow_scale", positive_float()),
        ];
        fields.extend(noise_provider_fields());
        record(fields)
    }),
    ("rotated_block_provider", || {
        // `BlockState.CODEC` xmapped to its block and back to the default state, so the
        // written state is the block's default state.
        record(vec![req("state", block_state().map_tag(default_state_of))])
    }),
    ("randomized_int_state_provider", || {
        record(vec![
            req("source", lazy(block_state_provider)),
            req("property", string_codec()),
            req("values", any_int_provider()),
        ])
    }),
    ("rule_based_state_provider", || {
        record(vec![
            opt("fallback", lazy(block_state_provider)),
            req("rules", list(rule())),
        ])
    }),
];

/// `Block::defaultBlockState` of an encoded block state (`RotatedBlockProvider`).
fn default_state_of(tag: crate::storage::nbt::Tag) -> Result<crate::storage::nbt::Tag, String> {
    use crate::block_states::default_state_properties;
    use crate::storage::nbt::Tag;
    let Tag::Compound(fields) = tag else {
        return Ok(tag);
    };
    let Some(Tag::String(name)) = fields
        .iter()
        .find_map(|(key, value)| (key == "Name").then_some(value.clone()))
    else {
        return Ok(Tag::Compound(fields));
    };
    let mut entries = vec![("Name".to_string(), Tag::String(name.clone()))];
    let mut properties: Vec<(String, Tag)> = default_state_properties(&name)
        .unwrap_or_default()
        .into_iter()
        .map(|(key, value)| (key.to_string(), Tag::String(value.to_string())))
        .collect();
    if !properties.is_empty() {
        properties.sort_by(|a, b| a.0.cmp(&b.0));
        entries.push(("Properties".to_string(), Tag::Compound(properties)));
    }
    Ok(Tag::Compound(entries))
}

cached_codec! {
    /// `BlockStateProvider.CODEC`.
    pub fn block_state_provider() -> Codec {
        typed(
            "type",
            "minecraft:block_state_provider_type",
            BLOCK_STATE_PROVIDER_TYPES,
        )
    }
}
