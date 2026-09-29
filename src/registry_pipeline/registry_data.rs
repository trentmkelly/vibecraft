//! Maps `RegistryDataLoader` table entries to their element codecs.
//!
//! The ordered tables themselves (`WORLDGEN_REGISTRIES`, `DIMENSION_REGISTRIES`,
//! `SYNCHRONIZED_REGISTRIES`) live in [`crate::resource_registry_data_loader`]; this
//! module resolves the `elementCodec` each entry names.

use crate::registry_pipeline::element_codecs as codecs;
use crate::registry_pipeline::gametest_codecs as gametest;
use crate::registry_pipeline::loader::{ElementCodecs, LoadTask};
use crate::resource_registry_data_loader::{RegistryDataLoaderRegistryData, WORLDGEN_REGISTRIES};

/// Resolves the direct (pack-decoding) and network (sync-encoding) codecs of a
/// registry, or `None` when the registry is not loaded by this pipeline yet.
///
/// Registries missing here are the `WORLDGEN_REGISTRIES` entries whose element codecs
/// are not ported yet. None of them is in `SYNCHRONIZED_REGISTRIES`, so clients
/// never receive them, but the server still needs them for worldgen:
///
/// - TODO(registry-pipeline-worldgen-features): `worldgen/configured_feature`
/// - TODO(registry-pipeline-worldgen-structures): `worldgen/structure_set`,
///   `worldgen/processor_list`, `worldgen/template_pool`
/// - TODO(registry-pipeline-worldgen-noise): `worldgen/noise_settings`,
///   `worldgen/noise`, `worldgen/density_function`,
///   `worldgen/multi_noise_biome_source_parameter_list`
/// - TODO(registry-pipeline-worldgen-presets): `worldgen/world_preset`,
///   `worldgen/flat_level_generator_preset`
/// - TODO(registry-pipeline-trial-spawner): `trial_spawner_config`
/// - TODO(registry-pipeline-enchantment-provider): `enchantment_provider`
/// - TODO(registry-pipeline-trades): `villager_trade`, `trade_set`
///
/// `DIMENSION_REGISTRIES` (`level_stem`) is loaded per world and not part of this
/// pipeline yet (TODO(registry-pipeline-level-stem)).
pub fn element_codecs(key: &str) -> Option<ElementCodecs> {
    let same = |codec: crate::registry_pipeline::codec::Codec| ElementCodecs {
        direct: codec.clone(),
        network: codec,
    };
    let pair = |direct, network| ElementCodecs { direct, network };
    Some(match key {
        "minecraft:worldgen/biome" => pair(codecs::biome_direct(), codecs::biome_network()),
        "minecraft:chat_type" => same(codecs::chat_type()),
        "minecraft:trim_pattern" => same(codecs::trim_pattern()),
        "minecraft:trim_material" => same(codecs::trim_material()),
        "minecraft:wolf_variant" => pair(
            codecs::wolf_variant_direct(),
            codecs::wolf_variant_network(),
        ),
        "minecraft:wolf_sound_variant" => same(codecs::wolf_sound_variant()),
        "minecraft:pig_variant" => {
            pair(codecs::pig_variant_direct(), codecs::pig_variant_network())
        }
        "minecraft:pig_sound_variant" => same(codecs::pig_sound_variant()),
        "minecraft:frog_variant" => pair(
            codecs::frog_variant_direct(),
            codecs::frog_variant_network(),
        ),
        "minecraft:cat_variant" => {
            pair(codecs::cat_variant_direct(), codecs::cat_variant_network())
        }
        "minecraft:cat_sound_variant" => same(codecs::cat_sound_variant()),
        "minecraft:cow_variant" => {
            pair(codecs::cow_variant_direct(), codecs::cow_variant_network())
        }
        "minecraft:cow_sound_variant" => same(codecs::cow_sound_variant()),
        "minecraft:chicken_variant" => pair(
            codecs::chicken_variant_direct(),
            codecs::chicken_variant_network(),
        ),
        "minecraft:chicken_sound_variant" => same(codecs::chicken_sound_variant()),
        "minecraft:zombie_nautilus_variant" => pair(
            codecs::zombie_nautilus_variant_direct(),
            codecs::zombie_nautilus_variant_network(),
        ),
        "minecraft:painting_variant" => same(codecs::painting_variant()),
        "minecraft:dimension_type" => pair(
            codecs::dimension_type_direct(),
            codecs::dimension_type_network(),
        ),
        "minecraft:damage_type" => same(codecs::damage_type()),
        "minecraft:banner_pattern" => same(codecs::banner_pattern()),
        "minecraft:test_environment" => same(gametest::test_environment()),
        "minecraft:test_instance" => same(gametest::test_instance()),
        "minecraft:enchantment" | "minecraft:dialog" => same(codecs::unported()),
        "minecraft:worldgen/configured_carver"
        | "minecraft:worldgen/placed_feature"
        | "minecraft:worldgen/structure" => same(codecs::worldgen_reference_target()),
        "minecraft:jukebox_song" => same(codecs::jukebox_song()),
        "minecraft:instrument" => same(codecs::instrument()),
        "minecraft:world_clock" => same(codecs::world_clock()),
        "minecraft:timeline" => pair(codecs::timeline_direct(), codecs::timeline_network()),
        _ => return None,
    })
}

/// The `WORLDGEN_REGISTRIES` entries this pipeline can load, in Java order.
pub fn worldgen_load_tasks() -> Vec<LoadTask<'static>> {
    tasks_for(WORLDGEN_REGISTRIES)
}

fn tasks_for(table: &'static [RegistryDataLoaderRegistryData]) -> Vec<LoadTask<'static>> {
    table
        .iter()
        .filter_map(|data| element_codecs(data.key).map(|codecs| LoadTask { data, codecs }))
        .collect()
}

/// Registry keys in `WORLDGEN_REGISTRIES` that this pipeline does not load yet.
#[cfg(test)]
pub fn unloaded_worldgen_registries() -> Vec<&'static str> {
    WORLDGEN_REGISTRIES
        .iter()
        .filter(|data| element_codecs(data.key).is_none())
        .map(|data| data.key)
        .collect()
}
