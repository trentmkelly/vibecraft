//! The vendored worldgen data files (`template_pool`, `processor_list`, `noise`,
//! `density_function`, `noise_settings`, `structure`, `structure_set`, world presets,
//! `enchantment_provider`, `trial_spawner`, ...) decode with their typed Java codec
//! mirrors, load through the live registry pipeline without problems and re-encode to
//! semantically equal JSON.
//!
//! `tools/vanilla_data_verified_pools.txt` lists the checklist paths this covers;
//! `tools/tick_vanilla_data_items.py` ticks exactly those lines of
//! `CHECKLIST_VANILLA_DATA_RESOURCES.md`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use super::{builtin, registries};
use crate::registry::Identifier;
use crate::registry_pipeline::codec::{Codec, CodecContext};
use crate::registry_pipeline::registry_data::element_codecs;
use crate::registry_pipeline::resources::ResourceManager;
use crate::registry_pipeline::tags::load_tags_for_registry;
use crate::storage::nbt::Tag;

/// Registries covered by this suite: `(registry key, data directory)`.
const FAMILIES: &[(&str, &str)] = &[
    ("minecraft:worldgen/template_pool", "worldgen/template_pool"),
    (
        "minecraft:worldgen/processor_list",
        "worldgen/processor_list",
    ),
    ("minecraft:worldgen/noise", "worldgen/noise"),
    (
        "minecraft:worldgen/density_function",
        "worldgen/density_function",
    ),
    (
        "minecraft:worldgen/noise_settings",
        "worldgen/noise_settings",
    ),
    ("minecraft:worldgen/structure", "worldgen/structure"),
    (
        "minecraft:worldgen/configured_carver",
        "worldgen/configured_carver",
    ),
    ("minecraft:worldgen/structure_set", "worldgen/structure_set"),
    (
        "minecraft:worldgen/multi_noise_biome_source_parameter_list",
        "worldgen/multi_noise_biome_source_parameter_list",
    ),
    ("minecraft:worldgen/world_preset", "worldgen/world_preset"),
    (
        "minecraft:worldgen/flat_level_generator_preset",
        "worldgen/flat_level_generator_preset",
    ),
    ("minecraft:enchantment_provider", "enchantment_provider"),
    ("minecraft:villager_trade", "villager_trade"),
    ("minecraft:trade_set", "trade_set"),
    ("minecraft:trial_spawner", "trial_spawner"),
];

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Extension-less paths of every `.json` below `dir`, relative to `root`.
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

fn vendored(dir: &str) -> Vec<String> {
    let base = manifest_path("vanilla-data/data/minecraft").join(dir);
    let mut files = Vec::new();
    collect(&base, &base, &mut files);
    files.sort();
    files
}

fn read_json(dir: &str, file: &str) -> Value {
    let path = manifest_path("vanilla-data/data/minecraft")
        .join(dir)
        .join(format!("{file}.json"));
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

/// Every registry key `codec` may reference while loading.
fn loading_registries() -> BTreeSet<Identifier> {
    crate::resource_registry_data_loader::WORLDGEN_REGISTRIES
        .iter()
        .map(|data| Identifier::parse(data.key).expect("registry key"))
        .collect()
}

fn decode(codec: &Codec, json: &Value) -> Result<Tag, String> {
    let loading = loading_registries();
    codec.parse(json, &CodecContext::new(builtin(), &loading))
}

/// `NbtOps` -> `JsonOps`: numbers become JSON numbers (floats through their shortest
/// decimal form), the bytes 0 and 1 become booleans.
fn tag_to_json(tag: &Tag) -> Value {
    match tag {
        Tag::Byte(v) if matches!(*v, 0 | 1) => json!(*v == 1),
        Tag::Byte(v) => json!(*v),
        Tag::Short(v) => json!(*v),
        Tag::Int(v) => json!(*v),
        Tag::Long(v) => json!(*v),
        Tag::Float(v) => v
            .to_string()
            .parse::<f64>()
            .map_or(Value::Null, |double| json!(double)),
        Tag::Double(v) => json!(*v),
        Tag::String(v) => json!(v),
        Tag::List(items) => Value::Array(items.iter().map(tag_to_json).collect()),
        Tag::Compound(fields) => {
            let mut map = Map::new();
            for (key, value) in fields {
                map.insert(key.clone(), tag_to_json(value));
            }
            Value::Object(map)
        }
        other => panic!("unexpected tag {other:?}"),
    }
}

/// Whether `output` (the re-encoded JSON) carries everything `input` said.
///
/// Encoded output may contain more than the input (defaults written by `orElse`
/// fields, full block-state property sets) but never less, except for values the Java
/// codec drops because they equal the default. Numbers compare as `f32` (float
/// fields) and identifiers ignore the `minecraft:` namespace.
fn covers(input: &Value, output: &Value, defaults_dropped: &mut Vec<String>, path: &str) -> bool {
    match (input, output) {
        (Value::Object(a), Value::Object(b)) => a.iter().all(|(key, value)| {
            let child_path = format!("{path}/{key}");
            match b.get(key) {
                Some(other) => covers(value, other, defaults_dropped, &child_path),
                None => {
                    // The encoder omits `optionalFieldOf(name, default)` values equal to
                    // the default; those are recorded so tests can see them.
                    defaults_dropped.push(child_path);
                    true
                }
            }
        }),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .enumerate()
                    .all(|(i, (x, y))| covers(x, y, defaults_dropped, &format!("{path}/{i}")))
        }
        (Value::Number(a), Value::Number(b)) => {
            let (x, y) = (
                a.as_f64().unwrap_or(f64::NAN),
                b.as_f64().unwrap_or(f64::NAN),
            );
            x == y || x as f32 == y as f32
        }
        (Value::Bool(a), Value::Number(b)) | (Value::Number(b), Value::Bool(a)) => {
            b.as_f64() == Some(if *a { 1.0 } else { 0.0 })
        }
        (Value::String(a), Value::String(b)) => {
            a == b || format!("minecraft:{a}") == *b || format!("minecraft:{b}") == *a
        }
        _ => input == output,
    }
}

/// Decodes one file and checks the re-encoded form. Returns the paths of input fields
/// the encoder dropped, or a problem description.
fn check_file(codec: &Codec, dir: &str, file: &str) -> Result<Vec<String>, String> {
    let input = read_json(dir, file);
    let tag = decode(codec, &input).map_err(|err| format!("{dir}/{file}: decode: {err}"))?;
    let output = tag_to_json(&tag);
    let again = decode(codec, &output)
        .map_err(|err| format!("{dir}/{file}: re-decode of the encoded form: {err}"))?;
    if again != tag {
        return Err(format!("{dir}/{file}: encoding is not a fixed point"));
    }
    let mut dropped = Vec::new();
    if !covers(&input, &output, &mut dropped, "") {
        return Err(format!(
            "{dir}/{file}: re-encoded JSON differs from the input"
        ));
    }
    Ok(dropped)
}

/// Files of `dir` that fail [`check_file`], with the reasons.
fn problems(registry: &str, dir: &str) -> Vec<String> {
    let codec = element_codecs(registry)
        .unwrap_or_else(|| panic!("{registry} has no codec"))
        .direct;
    vendored(dir)
        .iter()
        .filter_map(|file| check_file(&codec, dir, file).err())
        .collect()
}

#[test]
fn every_vendored_worldgen_file_decodes_and_round_trips() {
    let mut all = Vec::new();
    for (registry, dir) in FAMILIES {
        all.extend(problems(registry, dir));
    }
    assert!(
        all.is_empty(),
        "{} problems:\n{}",
        all.len(),
        all.join("\n")
    );
}

#[test]
fn vendored_file_counts_match_the_vanilla_data_report() {
    let expected = [
        ("worldgen/template_pool", 188),
        ("worldgen/processor_list", 40),
        ("worldgen/noise", 62),
        ("worldgen/density_function", 35),
        ("worldgen/noise_settings", 7),
        ("worldgen/structure", 34),
        ("worldgen/configured_carver", 4),
        ("worldgen/structure_set", 20),
        ("worldgen/multi_noise_biome_source_parameter_list", 2),
        ("worldgen/world_preset", 6),
        ("worldgen/flat_level_generator_preset", 9),
        ("enchantment_provider", 7),
        ("villager_trade", 387),
        ("trade_set", 68),
        ("trial_spawner", 28),
    ];
    for (dir, count) in expected {
        assert_eq!(vendored(dir).len(), count, "{dir}");
    }
}

#[test]
fn live_registries_load_every_vendored_worldgen_file() {
    for (key, dir) in FAMILIES {
        let registry = registries()
            .lookup(&Identifier::parse(key).expect("registry key"))
            .unwrap_or_else(|| panic!("{key} is not loaded"));
        let loaded: BTreeSet<String> = registry
            .elements()
            .iter()
            .map(|element| element.key.path().to_string())
            .collect();
        let files: BTreeSet<String> = vendored(dir).into_iter().collect();
        assert_eq!(loaded, files, "{key}");
    }
}

// ---------------------------------------------------------------------------
// Verified checklist rows
// ---------------------------------------------------------------------------

/// Checklist paths of every vendored file that decoded and round-tripped.
fn verified_paths() -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for (registry, dir) in FAMILIES {
        let codec = element_codecs(registry)
            .unwrap_or_else(|| panic!("{registry} has no codec"))
            .direct;
        for file in vendored(dir) {
            if check_file(&codec, dir, &file).is_ok() {
                paths.insert(format!(
                    "decompiled-server-26.1.2/data/minecraft/{dir}/{file}.json"
                ));
            }
        }
    }
    paths.extend(resolved_preset_tag_paths());
    paths.extend(super::trade_rebalance::verified_paths());
    paths
}

/// Registries whose tag files this suite verifies against the loaded registry.
const TAGGED_PRESET_REGISTRIES: &[&str] = &[
    "worldgen/world_preset",
    "worldgen/flat_level_generator_preset",
];

/// Checklist paths of the `tags/<preset registry>/**` files that resolve against the
/// loaded preset registries (`TagLoader`: every entry must exist).
fn resolved_preset_tag_paths() -> BTreeSet<String> {
    let manager = ResourceManager::vanilla();
    let mut paths = BTreeSet::new();
    for path in TAGGED_PRESET_REGISTRIES {
        let key = Identifier::parse(&format!("minecraft:{path}")).expect("registry key");
        let registry = registries()
            .lookup(&key)
            .unwrap_or_else(|| panic!("{path} is not loaded"));
        let mut lookup = |id: &Identifier, _required: bool| registry.contains(id);
        let mut errors = Vec::new();
        let tags = load_tags_for_registry(&manager, path, &mut lookup, &mut errors);
        assert!(errors.is_empty(), "tags/{path}: {errors:?}");
        for (tag, entries) in tags {
            assert!(!entries.is_empty(), "tags/{path}/{tag} resolved to nothing");
            paths.insert(format!(
                "decompiled-server-26.1.2/data/minecraft/tags/{path}/{}.json",
                tag.path()
            ));
        }
    }
    paths
}

#[test]
fn preset_tags_list_the_vanilla_world_and_flat_presets() {
    let paths = resolved_preset_tag_paths();
    for expected in [
        "tags/worldgen/world_preset/normal.json",
        "tags/worldgen/world_preset/extended.json",
        "tags/worldgen/flat_level_generator_preset/visible.json",
    ] {
        let full = format!("decompiled-server-26.1.2/data/minecraft/{expected}");
        assert!(paths.contains(&full), "{full}");
    }
}

#[test]
fn verified_pool_paths_list_is_current() {
    let paths = verified_paths();
    let list = manifest_path("tools/vanilla_data_verified_pools.txt");
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
fn ticked_checklist_lines_of_worldgen_families_are_all_verified() {
    let paths = verified_paths();
    let checklist = fs::read_to_string(manifest_path("CHECKLIST_VANILLA_DATA_RESOURCES.md"))
        .unwrap_or_else(|err| panic!("read checklist: {err}"));
    for line in checklist
        .lines()
        .filter(|line| line.starts_with("- [x] Audit vanilla data resource `"))
    {
        let path = line.split('`').nth(1).unwrap_or_default();
        let data = "decompiled-server-26.1.2/data/minecraft";
        let in_family = FAMILIES
            .iter()
            .map(|(_, dir)| format!("{data}/{dir}/"))
            .chain([format!("{data}/datapacks/")])
            .chain(
                TAGGED_PRESET_REGISTRIES
                    .iter()
                    .map(|registry| format!("{data}/tags/{registry}/")),
            )
            .any(|prefix| path.starts_with(&prefix));
        if in_family {
            assert!(paths.contains(path), "ticked but not verified: {path}");
        }
    }
}

/// Field names the Java encoder legitimately omits when the value equals the default of
/// an `optionalFieldOf(name, default)` (or `lenientOptionalFieldOf`) component. Any
/// other input field that vanishes on re-encoding means a codec ignores real data.
const DEFAULTED_FIELDS: &[&str] = &[];

#[test]
fn input_fields_are_only_dropped_when_they_equal_a_java_default() {
    let mut names = BTreeSet::new();
    for (registry, dir) in FAMILIES {
        let codec = element_codecs(registry).expect("codec").direct;
        for file in vendored(dir) {
            for path in check_file(&codec, dir, &file).unwrap_or_default() {
                let name = path.rsplit('/').next().unwrap_or_default().to_string();
                names.insert(name);
            }
        }
    }
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    assert_eq!(names, DEFAULTED_FIELDS);
}
