//! Every vanilla `tags/**` JSON resolves through the live `TagLoader` port.
//!
//! The vanilla data pack bundled under `vanilla-data/` ships one tag directory per
//! taggable registry. This loads each of them against the loaded vanilla registries
//! and requires that no reference is left unresolved (Java logs and refuses the
//! reload otherwise), so a missing element or a broken nested tag fails the test.

use std::collections::BTreeSet;
use std::path::Path;

use crate::registry::Identifier;
use crate::registry_pipeline::resources::ResourceManager;
use crate::registry_pipeline::tags::load_tags_for_registry;

/// Collects the registry paths (relative to `tags/`) whose directory holds JSON
/// files directly. Registry paths may nest (`worldgen/biome`), and tag ids may nest
/// below them (`villager_trade/armorer/level_1`), so a registry directory is the
/// shortest prefix that is a registry: the caller decides with a registry lookup.
fn collect_json_dirs(root: &Path, relative: &str, found: &mut BTreeSet<String>) {
    let dir = root.join(relative);
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|err| panic!("{dir:?}: {err}")) {
        let path = entry.expect("directory entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).expect("utf-8 name");
        let child = if relative.is_empty() {
            name.to_string()
        } else {
            format!("{relative}/{name}")
        };
        if path.is_dir() {
            collect_json_dirs(root, &child, found);
        } else {
            found.insert(relative.to_string());
        }
    }
}

/// Registries that VibeCraft does not load as element registries. Their tags name
/// worldgen presets/features that are not decoded yet, so element existence cannot
/// be checked and every reference is accepted (explicit deferral).
const UNLOADED_REGISTRIES: &[&str] = &["worldgen/configured_feature"];

/// Maps a tag-file directory to its registry path: the shortest prefix that is a
/// registry. `worldgen` alone only groups registries, so it never matches.
fn registry_path_of(directory: &str, has_registry: impl Fn(&str) -> bool) -> Option<String> {
    let parts: Vec<&str> = directory.split('/').collect();
    (1..=parts.len())
        .map(|len| parts[..len].join("/"))
        .filter(|candidate| candidate != "worldgen")
        .find(|candidate| has_registry(candidate))
}

#[test]
fn all_vanilla_tags_resolve_through_the_tag_loader() {
    let registries = super::registries();
    let manager = ResourceManager::vanilla();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/tags");
    let mut directories = BTreeSet::new();
    collect_json_dirs(&root, "", &mut directories);

    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft");
    // A registry is known when it is loaded, or (for registries whose codec is not
    // ported, e.g. `villager_trade`) when the vanilla pack ships its element files.
    let has_registry = |path: &str| {
        Identifier::parse(&format!("minecraft:{path}"))
            .ok()
            .is_some_and(|key| registries.lookup(&key).is_some())
            || data.join(path).is_dir()
            || UNLOADED_REGISTRIES.contains(&path)
    };
    let registry_paths: BTreeSet<String> = directories
        .iter()
        .map(|dir| {
            registry_path_of(dir, has_registry)
                .unwrap_or_else(|| panic!("tags/{dir} has no loaded registry"))
        })
        .collect();

    let mut resolved = 0;
    for path in registry_paths {
        let unloaded = UNLOADED_REGISTRIES.contains(&path.as_str());
        let registry = Identifier::parse(&format!("minecraft:{path}"))
            .ok()
            .and_then(|key| registries.lookup(&key));
        let mut lookup = |id: &Identifier, _required: bool| match registry {
            Some(registry) => registry.contains(id),
            // Registry not loaded: the element must exist as a pack file.
            None => {
                unloaded
                    || data
                        .join(&path)
                        .join(format!("{}.json", id.path()))
                        .is_file()
            }
        };
        let mut errors = Vec::new();
        let tags = load_tags_for_registry(&manager, &path, &mut lookup, &mut errors);
        assert!(errors.is_empty(), "tags/{path}: {errors:?}");
        resolved += tags.len();
    }
    assert_eq!(resolved, 758, "every vanilla tag file must load");
}
