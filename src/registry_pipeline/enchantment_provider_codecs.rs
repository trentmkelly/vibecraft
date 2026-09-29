//! Codec for `enchantment_provider` (`EnchantmentProvider.DIRECT_CODEC` and the
//! `EnchantmentProviderTypes`).

use crate::registry_pipeline::codec::{holder_file, holder_set, int_range, record, req, Codec};
use crate::registry_pipeline::element_codecs::unported;
use crate::registry_pipeline::shared::int_provider;
use crate::registry_pipeline::worldgen_common::{registry_dispatch, Variant};

const ENCHANTMENT_REGISTRY: &str = "minecraft:enchantment";

/// `EnchantmentProvider.DIRECT_CODEC`.
pub fn enchantment_provider() -> Codec {
    registry_dispatch(
        "type",
        "minecraft:enchantment_provider_type",
        ENCHANTMENT_PROVIDER_TYPES,
    )
}

/// `EnchantmentProviderTypes.bootstrap` registrations.
const ENCHANTMENT_PROVIDER_TYPES: &[Variant] = &[
    ("by_cost", by_cost),
    ("by_cost_with_difficulty", by_cost_with_difficulty),
    ("single", single),
];

/// `IntProviders.CODEC`: any bounds.
fn any_int_provider() -> Codec {
    int_provider(i32::MIN, i32::MAX)
}

/// `EnchantmentsByCost.CODEC`.
fn by_cost() -> Codec {
    record(vec![
        req("enchantments", holder_set(ENCHANTMENT_REGISTRY, false)),
        req("cost", any_int_provider()),
    ])
}

/// `EnchantmentsByCostWithDifficulty.CODEC`.
fn by_cost_with_difficulty() -> Codec {
    record(vec![
        req("enchantments", holder_set(ENCHANTMENT_REGISTRY, false)),
        req("min_cost", int_range(1, 10000)),
        req("max_cost_span", int_range(0, 10000)),
    ])
}

/// `SingleEnchantment.CODEC`: `Enchantment.CODEC` accepts a reference or an inline
/// enchantment.
fn single() -> Codec {
    record(vec![
        req("enchantment", holder_file(ENCHANTMENT_REGISTRY, unported())),
        req("level", any_int_provider()),
    ])
}
