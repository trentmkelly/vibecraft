//! Every vendored vanilla data file of a registry with a fully ported element codec
//! must be decoded by the pipeline (no skipped file, no missing file).
//!
//! `tools/vanilla_data_verified_paths.txt` lists the checklist paths this covers;
//! `tools/tick_vanilla_data_items.py` ticks exactly those lines of
//! `CHECKLIST_VANILLA_DATA_RESOURCES.md`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::registries;
use crate::registry::Identifier;

/// Registries whose element codec is a typed port of the Java codec (not the
/// generic pass-through or a reference-only target). Value: data directory.
///
/// Not listed (still unticked): `enchantment` and `dialog` (generic codec,
/// TODO(registry-pipeline-enchantment), TODO(registry-pipeline-dialog)),
/// `worldgen/configured_carver|placed_feature|structure` (reference-only targets),
/// and every registry the pipeline does not load yet (see `registry_data`).
const VERIFIED_REGISTRIES: &[(&str, &str)] = &[
    ("minecraft:worldgen/biome", "worldgen/biome"),
    ("minecraft:chat_type", "chat_type"),
    ("minecraft:trim_pattern", "trim_pattern"),
    ("minecraft:trim_material", "trim_material"),
    ("minecraft:wolf_variant", "wolf_variant"),
    ("minecraft:wolf_sound_variant", "wolf_sound_variant"),
    ("minecraft:pig_variant", "pig_variant"),
    ("minecraft:pig_sound_variant", "pig_sound_variant"),
    ("minecraft:frog_variant", "frog_variant"),
    ("minecraft:cat_variant", "cat_variant"),
    ("minecraft:cat_sound_variant", "cat_sound_variant"),
    ("minecraft:cow_variant", "cow_variant"),
    ("minecraft:cow_sound_variant", "cow_sound_variant"),
    ("minecraft:chicken_variant", "chicken_variant"),
    ("minecraft:chicken_sound_variant", "chicken_sound_variant"),
    ("minecraft:zombie_nautilus_variant", "zombie_nautilus_variant"),
    ("minecraft:painting_variant", "painting_variant"),
    ("minecraft:dimension_type", "dimension_type"),
    ("minecraft:damage_type", "damage_type"),
    ("minecraft:banner_pattern", "banner_pattern"),
    ("minecraft:test_environment", "test_environment"),
    ("minecraft:test_instance", "test_instance"),
    ("minecraft:jukebox_song", "jukebox_song"),
    ("minecraft:instrument", "instrument"),
    ("minecraft:world_clock", "world_clock"),
    ("minecraft:timeline", "timeline"),
];

/// Collects the extension-less paths of every `.json` under `dir`, relative to `root`.
fn collect(dir: &Path, root: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
        let path = entry.unwrap_or_else(|err| panic!("{err}")).path();
        if path.is_dir() {
            collect(&path, root, out);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let text = relative.to_string_lossy().replace('\\', "/");
            out.push(text.trim_end_matches(".json").to_string());
        }
    }
}

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Asserts each verified registry holds exactly the vendored files and returns the
/// checklist paths of them.
fn verified_paths() -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for (key, dir) in VERIFIED_REGISTRIES {
        let registry = registries()
            .lookup(&Identifier::parse(key).unwrap_or_else(|err| panic!("{key}: {err}")))
            .unwrap_or_else(|| panic!("{key} is not loaded"));
        let base = manifest_path("vanilla-data/data/minecraft").join(dir);
        let mut files = Vec::new();
        collect(&base, &base, &mut files);
        assert!(!files.is_empty(), "{key}: no vendored files");
        let loaded: BTreeSet<String> = registry
            .elements()
            .iter()
            .map(|element| element.key.path().to_string())
            .collect();
        let vendored: BTreeSet<String> = files.iter().cloned().collect();
        assert_eq!(loaded, vendored, "{key}: loaded elements != vendored files");
        for file in files {
            paths.insert(format!(
                "decompiled-server-26.1.2/data/minecraft/{dir}/{file}.json"
            ));
        }
    }
    paths
}

#[test]
fn every_vendored_file_of_a_ported_registry_is_loaded() {
    let paths = verified_paths();
    let list = manifest_path("tools/vanilla_data_verified_paths.txt");
    let text: String = paths.iter().map(|path| format!("{path}\n")).collect();
    if std::env::var_os("VIBECRAFT_WRITE_VERIFIED_PATHS").is_some() {
        fs::write(&list, &text).unwrap_or_else(|err| panic!("write list: {err}"));
    }
    let committed = fs::read_to_string(&list).unwrap_or_else(|err| panic!("read list: {err}"));
    assert_eq!(
        committed, text,
        "regenerate with VIBECRAFT_WRITE_VERIFIED_PATHS=1"
    );
}

#[test]
fn ticked_checklist_lines_of_ported_registries_are_all_verified() {
    let paths = verified_paths();
    let checklist = fs::read_to_string(manifest_path("CHECKLIST_VANILLA_DATA_RESOURCES.md"))
        .unwrap_or_else(|err| panic!("read checklist: {err}"));
    for line in checklist
        .lines()
        .filter(|line| line.starts_with("- [x] Audit vanilla data resource `"))
    {
        let path = line.split('`').nth(1).unwrap_or_default();
        let in_ported_dir = VERIFIED_REGISTRIES.iter().any(|(_, dir)| {
            path.starts_with(&format!("decompiled-server-26.1.2/data/minecraft/{dir}/"))
        });
        if in_ported_dir {
            assert!(paths.contains(path), "ticked but not verified: {path}");
        }
    }
}
