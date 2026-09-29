//! Data-driven registry pipeline: the port of `RegistryDataLoader`,
//! `RegistrySynchronization` and `TagLoader`.
//!
//! The pipeline bootstraps the frozen [`Registries`] the server keeps for its whole
//! lifetime from the selected data packs (the built-in vanilla pack, the bundled
//! feature packs and the packs of the world `datapacks` folder) and the static
//! registry report, then serves the configuration-phase `registry_data` and
//! `update_tags` packets from it (see [`crate::registry_pipeline::sync`]).
//!
//! Layout:
//! - [`resources`], [`zip_pack`] — packs (folders, archives, overlays, filters) and
//!   the `ResourceManager` stack (`data/<ns>/...`).
//! - [`tags`] — `TagLoader` (merge, `replace`, optional entries, dependency order).
//! - [`codec`], [`shared`], [`attributes`], [`element_codecs`], [`gametest_codecs`] —
//!   the JSON-to-network codecs for registry elements.
//! - [`worldgen_feature`] — the typed `configured_feature`/`placed_feature` codecs.
//! - [`loader`] — `RegistryDataLoader.load`, freezing, validators and error reports,
//!   plus the tag reload `/reload` performs.
//! - [`store`] — `MappedRegistry` and the layered [`Registries`] access.
//! - [`server_resources`] — the live pack repository, world data configuration and
//!   the registries in force (`WorldLoader.load`, `MinecraftServer.reloadResources`).
//! - [`datapack_content`] — the reloadable pack content besides registries: recipes,
//!   loot tables, advancements and functions (`ReloadableServerResources`).
//! - [`sync`] — packing registries and tags for clients.
//!
//! Registries not loaded yet (`TODO(registry-pipeline-worldgen-<name>)`) are listed
//! by [`registry_data`].

pub mod attributes;
pub mod builtin;
pub mod codec;
pub mod datapack_content;
pub mod element_codecs;
pub mod gametest_codecs;
pub mod loader;
pub mod registry_data;
pub mod resources;
pub mod server_resources;
pub mod shared;
pub mod store;
pub mod sync;
pub mod tags;
pub mod worldgen_feature;
pub mod zip_pack;

use std::sync::OnceLock;

use crate::registry_pipeline::builtin::BuiltinRegistries;
use crate::registry_pipeline::resources::ResourceManager;
use crate::registry_pipeline::store::Registries;

#[cfg(test)]
mod tests;

/// Loads the vanilla registries from `manager` (`ReloadableServerResources` +
/// `RegistryLayer` assembly for the vanilla pack).
pub fn load_registries(
    manager: &ResourceManager,
    builtin: &BuiltinRegistries,
) -> Result<Registries, String> {
    let tasks = registry_data::worldgen_load_tasks();
    let loaded = loader::load(manager, builtin, &tasks)
        .map_err(|err| format!("{err}\n{}", err.full_details()))?;
    let mut logged = loaded.logged;
    let static_layer = loader::build_static_layer(manager, builtin, &mut logged);
    for line in logged {
        crate::log::log_warn(&line);
    }
    let worldgen = loaded.registries;
    Ok(Registries::new(static_layer, worldgen))
}

/// The server-lifetime vanilla registries, loaded on first use.
pub fn vanilla_registries() -> Result<&'static Registries, String> {
    static REGISTRIES: OnceLock<Result<Registries, String>> = OnceLock::new();
    REGISTRIES
        .get_or_init(|| {
            let builtin = BuiltinRegistries::vanilla()?;
            load_registries(&ResourceManager::vanilla(), builtin)
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// `Registry.getId` for `element` (a bare path in the `minecraft` namespace or a
/// full identifier) in the active registry `registry`, looking through both the
/// data-driven and the static layers.
pub fn registry_element_id(registry: &str, element: &str) -> Option<usize> {
    let resources = server_resources::active_resources().ok()?;
    let registry = crate::registry::Identifier::parse(registry).ok()?;
    let element = crate::registry::Identifier::parse(element).ok()?;
    resources.registries.lookup(&registry)?.id_of(&element)
}
