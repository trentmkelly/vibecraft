//! Tests for the registry pipeline.
//!
//! - `order_fixtures`: element order pinned against the official transcript.
//! - `vanilla_payloads`: contents of the packets generated for the vanilla data.
//! - `codec_engine`: the JSON-to-NBT codec combinators.
//! - `loading`: `RegistryDataLoader`/`TagLoader` behaviour on synthetic packs.
//! - `synchronization`: known-pack elision and the tags payload.
//! - `datapack_content`: recipes, loot tables, advancements and functions from packs.
//! - `world_packs`: world data packs, overlays/filters, overrides and `/reload`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::network::configuration::KnownPack;
use crate::registry::Identifier;
use crate::registry_pipeline::builtin::BuiltinRegistries;
use crate::registry_pipeline::resources::{DirectoryPack, ResourceManager};
use crate::registry_pipeline::store::Registries;
use crate::registry_pipeline::sync::pack_registries;
use crate::registry_pipeline::vanilla_registries;
use crate::storage::nbt::Tag;

mod codec_engine;
mod datapack_content;
mod gametest_codecs;
mod loading;
mod model_tables;
mod official_transcript;
mod order_fixtures;
mod synchronization;
mod trade_rebalance;
mod vanilla_data_coverage;
mod vanilla_content;
mod vanilla_fields;
mod vanilla_payloads;
mod vanilla_tags;
mod world_packs;
mod worldgen_codecs;
mod worldgen_data;

/// The vanilla registries, loaded once for the whole test run.
pub(super) fn registries() -> &'static Registries {
    vanilla_registries().unwrap_or_else(|err| panic!("vanilla registries failed to load: {err}"))
}

pub(super) fn builtin() -> &'static BuiltinRegistries {
    BuiltinRegistries::vanilla().unwrap_or_else(|err| panic!("builtin registries: {err}"))
}

/// Element ids and full network contents of a synchronised registry, exactly as a
/// client without any known pack receives them.
pub(super) fn full_entries(registry: &str) -> Vec<(String, Tag)> {
    let key = Identifier::parse(registry).unwrap_or_else(|err| panic!("{registry}: {err}"));
    let packets = pack_registries(registries(), builtin(), &[])
        .unwrap_or_else(|err| panic!("pack_registries: {err}"));
    let packet = packets
        .into_iter()
        .find(|packet| packet.registry == key)
        .unwrap_or_else(|| panic!("{registry} is not synchronised"));
    packet
        .entries
        .into_iter()
        .map(|entry| {
            let data = entry
                .data
                .unwrap_or_else(|| panic!("{} was sent without contents", entry.id));
            (entry.id.to_string(), data)
        })
        .collect()
}

/// [`full_entries`] keyed by id.
pub(super) fn entry_map(registry: &str) -> BTreeMap<String, Tag> {
    full_entries(registry).into_iter().collect()
}

/// Asserts the wire order of a registry's element ids (bare `minecraft:` paths).
pub(super) fn assert_registry_order(registry: &str, expected: &[&str]) {
    let expected: Vec<String> = expected
        .iter()
        .map(|id| format!("minecraft:{id}"))
        .collect();
    let actual: Vec<String> = full_entries(registry)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(actual, expected, "{registry}");
}

pub(super) fn field<'a>(tag: &'a Tag, name: &str) -> Option<&'a Tag> {
    match tag {
        Tag::Compound(fields) => fields
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value)),
        _ => None,
    }
}

pub(super) fn compound<'a>(tag: &'a Tag, name: &str) -> &'a Tag {
    match field(tag, name) {
        Some(value @ Tag::Compound(_)) => value,
        other => panic!("expected compound field {name}, got {other:?}"),
    }
}

pub(super) fn string(value: &str) -> Tag {
    Tag::String(value.to_string())
}

/// A scratch data pack directory removed on drop.
pub(super) struct TestPackDir {
    root: PathBuf,
}

impl TestPackDir {
    pub(super) fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vibecraft-registry-pipeline-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap_or_else(|err| panic!("create {}: {err}", root.display()));
        Self { root }
    }

    /// Writes `data/<relative>` inside the pack.
    pub(super) fn write(&self, relative: &str, content: &str) -> &Self {
        let path = self.root.join("data").join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| panic!("mkdir: {err}"));
        }
        fs::write(&path, content).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
        self
    }

    pub(super) fn pack(&self, id: &str, known_pack: Option<KnownPack>) -> DirectoryPack {
        DirectoryPack::new(id, self.root.clone(), known_pack)
    }
}

impl Drop for TestPackDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The vanilla pack with `overlay` stacked on top (higher priority).
pub(super) fn vanilla_with(overlay: DirectoryPack) -> ResourceManager {
    ResourceManager::new(vec![Box::new(DirectoryPack::vanilla()), Box::new(overlay)])
}
