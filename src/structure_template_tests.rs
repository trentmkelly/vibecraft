//! Loads every vendored vanilla structure template through [`StructureTemplate`] and
//! round-trips it through `save`. `tools/vanilla_data_verified_structures.txt` lists
//! the checklist rows this proves; only those rows may be ticked.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::*;
use crate::storage::nbt::read_gzip_named_tag;

const CHECKLIST_PREFIX: &str = "decompiled-server-26.1.2/data/minecraft/structure/";

fn structure_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/structure")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "nbt") {
            out.push(path);
        }
    }
}

fn templates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect(&structure_root(), &mut paths);
    paths.sort();
    paths
}

fn checklist_path(path: &Path) -> String {
    let relative = path.strip_prefix(structure_root()).expect("under root");
    format!("{CHECKLIST_PREFIX}{}", relative.to_string_lossy())
}

fn load_file(path: &Path) -> Compound {
    let file = std::fs::File::open(path).expect("open");
    match read_gzip_named_tag(file).expect("gzip nbt").1 {
        Tag::Compound(root) => root,
        other => panic!("{}: root is {other:?}", path.display()),
    }
}

/// Compound key order in files written by Java follows `HashMap` iteration, which the
/// insertion-ordered [`Tag::Compound`] cannot reproduce, so equality is checked on the
/// tree with every compound sorted by key. `save` stamps the current `DataVersion`, so
/// the vendored value is compared separately (it must be present and not newer).
fn canonical(compound: &[(String, Tag)]) -> Vec<(String, Tag)> {
    fn sort(tag: &Tag) -> Tag {
        match tag {
            Tag::Compound(values) => Tag::Compound(canonical(values)),
            Tag::List(values) => Tag::List(values.iter().map(sort).collect()),
            other => other.clone(),
        }
    }
    let mut sorted: Vec<(String, Tag)> = compound
        .iter()
        .filter(|(key, _)| key != "DataVersion")
        .map(|(key, tag)| (key.clone(), sort(tag)))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    sorted
}

/// Reports the files whose load leaves unresolved states or whose round trip does
/// not reproduce the vendored NBT.
fn failures() -> Vec<String> {
    let mut failed = Vec::new();
    for path in templates() {
        let original = load_file(&path);
        let template = StructureTemplate::load(&original);
        let name = checklist_path(&path);
        let data_version = original.iter().find(|(key, _)| key == "DataVersion");
        if !matches!(data_version, Some((_, Tag::Int(v))) if *v > 0 && *v <= CURRENT_DATA_VERSION) {
            failed.push(format!("{name}: missing or newer DataVersion"));
        } else if !template.unresolved_states.is_empty() {
            failed.push(format!(
                "{name}: unresolved {:?}",
                template.unresolved_states
            ));
        } else if canonical(&template.save()) != canonical(&original) {
            failed.push(format!("{name}: round trip differs"));
        }
    }
    failed
}

#[test]
fn every_vendored_template_is_present() {
    assert_eq!(templates().len(), 1202);
}

#[test]
fn every_vendored_template_round_trips_through_load_and_save() {
    let failed = failures();
    assert!(
        failed.is_empty(),
        "{} failures: {:#?}",
        failed.len(),
        &failed[..failed.len().min(20)]
    );
}

fn verified_paths() -> BTreeSet<String> {
    include_str!("../tools/vanilla_data_verified_structures.txt")
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// The verified list is exactly the templates that load and round-trip.
#[test]
fn verified_list_is_exactly_the_round_tripping_templates() {
    let failed: BTreeSet<String> = failures()
        .iter()
        .map(|f| f.split(": ").next().expect("path").to_string())
        .collect();
    let expected: BTreeSet<String> = templates()
        .iter()
        .map(|p| checklist_path(p))
        .filter(|p| !failed.contains(p))
        .collect();
    assert_eq!(verified_paths(), expected);
}

/// Every ticked structure row of the checklist is in the verified list.
#[test]
fn ticked_structure_rows_are_verified() {
    let verified = verified_paths();
    let checklist = include_str!("../CHECKLIST_VANILLA_DATA_RESOURCES.md");
    for line in checklist.lines().filter(|l| l.starts_with("- [x]")) {
        let Some(rest) = line.split('`').nth(1) else {
            continue;
        };
        if rest.starts_with(CHECKLIST_PREFIX) {
            assert!(verified.contains(rest), "ticked but unverified: {rest}");
        }
    }
}

fn state_tag(name: &str, property: Option<(&str, &str)>) -> Tag {
    let mut compound = vec![("Name".to_string(), Tag::String(name.to_string()))];
    if let Some((key, value)) = property {
        compound.push((
            "Properties".to_string(),
            Tag::Compound(vec![(key.to_string(), Tag::String(value.to_string()))]),
        ));
    }
    Tag::Compound(compound)
}

/// Unknown blocks, properties and property values load like Java (air / default
/// value) but are reported as unresolved so the vanilla-data test can fail on them.
#[test]
fn unresolved_block_states_are_reported() {
    let root = vec![(
        "palette".to_string(),
        Tag::List(vec![
            state_tag("minecraft:stone", None),
            state_tag("minecraft:not_a_block", None),
            state_tag("minecraft:oak_log", Some(("axis", "diagonal"))),
            state_tag("minecraft:oak_log", Some(("nope", "x"))),
            state_tag("minecraft:oak_log", Some(("axis", "x"))),
        ]),
    )];
    let template = StructureTemplate::load(&root);
    assert_eq!(template.unresolved_states.len(), 3);
}

/// `buildInfoList`: full blocks, then other blocks, then block entities, each by y, x, z.
#[test]
fn load_orders_full_blocks_then_others_then_block_entities() {
    let block = |x: i32, y: i32, state: i32, nbt: bool| {
        let mut compound = vec![
            ("pos".to_string(), int_list([x, y, 0])),
            ("state".to_string(), Tag::Int(state)),
        ];
        if nbt {
            compound.push(("nbt".to_string(), Tag::Compound(vec![])));
        }
        Tag::Compound(compound)
    };
    let palette = vec![
        state_tag("minecraft:stone", None),
        state_tag("minecraft:torch", None),
    ];
    let blocks = vec![
        block(0, 1, 0, true),
        block(1, 0, 1, false),
        block(2, 0, 0, false),
        block(0, 0, 0, false),
    ];
    let root = vec![
        ("palette".to_string(), Tag::List(palette)),
        ("blocks".to_string(), Tag::List(blocks)),
    ];
    let order: Vec<[i32; 3]> = StructureTemplate::load(&root).palettes[0]
        .iter()
        .map(|info| info.pos)
        .collect();
    assert_eq!(order, vec![[0, 0, 0], [2, 0, 0], [1, 0, 0], [0, 1, 0]]);
}
