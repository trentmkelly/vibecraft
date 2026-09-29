//! Loads every vendored vanilla advancement through the live loader and codec.
//!
//! The corpus is `vanilla-data/data/minecraft/advancement/**` (git tracked copy of the 26.1.2
//! data-generator output), so nothing here needs the optional decompiled sources.
//!
//! The corpus test asserts, per file: (a) the live `AdvancementDefinition::from_json` loader
//! and the strict `Advancement.CODEC` mirror accept it with every trigger and predicate known,
//! (b) the whole set forms a valid advancement tree that `TreeNodePosition` can lay out, and
//! (c) re-encoding the decoded document gives semantically equal JSON. The verified path list
//! is pinned in `vanilla-data/reports/vanilla_data_verified_advancements.txt`, which
//! `tools/tick_vanilla_data_items.py` consumes.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::advancement_codec::AdvancementCodec;
use crate::advancement_condition_schema::{BuiltinLookup, Reg, RegistryLookup};
use crate::advancement_system::AdvancementDefinition;
use crate::advancement_tree::AdvancementTreeModel;
use crate::advancement_tree_position::{AdvancementLayoutInput, TreeNodePositionLayout};
use crate::registry::Identifier;
use crate::server_advancement_manager::ServerAdvancementManagerModel;

const DATA_ROOT: &str = "vanilla-data/data/minecraft";
const VERIFIED_LIST: &str = "vanilla-data/reports/vanilla_data_verified_advancements.txt";
const CHECKLIST: &str = "CHECKLIST_VANILLA_DATA_RESOURCES.md";
const CHECKLIST_PREFIX: &str = "decompiled-server-26.1.2/data/minecraft/";

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Resolves data-driven registries and tags from the vendored `vanilla-data` tree.
struct VendoredLookup {
    builtin: BuiltinLookup,
}

impl VendoredLookup {
    /// Directory (under `data/minecraft`) holding a data-driven registry's entries.
    fn directory(registry: Reg) -> Option<&'static str> {
        match registry {
            Reg::Biome => Some("worldgen/biome"),
            Reg::Structure => Some("worldgen/structure"),
            Reg::Enchantment => Some("enchantment"),
            Reg::JukeboxSong => Some("jukebox_song"),
            Reg::DamageType => Some("damage_type"),
            _ => None,
        }
    }

    /// Directory of tag files for a registry (`data/minecraft/tags/<dir>`).
    fn tag_directory(registry: Reg) -> &'static str {
        match registry {
            Reg::Item => "item",
            Reg::Block => "block",
            Reg::EntityType => "entity_type",
            Reg::Fluid => "fluid",
            Reg::Biome => "worldgen/biome",
            Reg::Structure => "worldgen/structure",
            Reg::Enchantment => "enchantment",
            Reg::JukeboxSong => "jukebox_song",
            Reg::DamageType => "damage_type",
            _ => "unsupported",
        }
    }

    fn file_exists(directory: &str, id: &Identifier) -> bool {
        if id.namespace() != "minecraft" {
            return false;
        }
        manifest_path(&format!("{DATA_ROOT}/{directory}/{}.json", id.path())).is_file()
    }
}

impl RegistryLookup for VendoredLookup {
    fn contains(&self, registry: Reg, id: &Identifier) -> bool {
        match Self::directory(registry) {
            Some(directory) => Self::file_exists(directory, id),
            None => self.builtin.contains(registry, id),
        }
    }

    fn contains_tag(&self, registry: Reg, id: &Identifier) -> bool {
        Self::file_exists(&format!("tags/{}", Self::tag_directory(registry)), id)
    }
}

/// Every advancement JSON as `(checklist path, advancement id, parsed document, raw text)`.
struct Corpus {
    entries: Vec<CorpusEntry>,
}

struct CorpusEntry {
    checklist_path: String,
    id: Identifier,
    raw: String,
    document: Value,
}

fn collect_json(directory: &Path, files: &mut Vec<PathBuf>) {
    let mut children = fs::read_dir(directory)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", directory.display()))
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|err| panic!("cannot list {}: {err}", directory.display()));
    children.sort();
    for path in children {
        if path.is_dir() {
            collect_json(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "json") {
            files.push(path);
        }
    }
}

fn load_corpus() -> Corpus {
    let root = manifest_path(&format!("{DATA_ROOT}/advancement"));
    let mut files = Vec::new();
    collect_json(&root, &mut files);
    let entries = files
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(manifest_path(DATA_ROOT))
                .unwrap_or_else(|err| panic!("{}: {err}", path.display()))
                .to_string_lossy()
                .replace('\\', "/");
            let raw = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
            let document = serde_json::from_str(&raw)
                .unwrap_or_else(|err| panic!("{relative} is not JSON: {err}"));
            let id_path = relative
                .strip_prefix("advancement/")
                .and_then(|rest| rest.strip_suffix(".json"))
                .unwrap_or_else(|| panic!("unexpected corpus path {relative}"));
            CorpusEntry {
                checklist_path: format!("{CHECKLIST_PREFIX}{relative}"),
                id: Identifier::parse(&format!("minecraft:{id_path}"))
                    .unwrap_or_else(|err| panic!("{relative}: {err}")),
                raw,
                document,
            }
        })
        .collect();
    Corpus { entries }
}

/// Per-file verification: live loader, strict codec, and semantic round trip.
fn verify_entry(codec: &AdvancementCodec<'_>, entry: &CorpusEntry) -> Result<AdvancementDefinition, String> {
    let definition = AdvancementDefinition::from_json(&entry.id.to_string(), &entry.raw)?;
    let encoded = codec.round_trip(&entry.document)?;
    if encoded != entry.document {
        return Err("re-encoded JSON differs from the source document".to_string());
    }
    Ok(definition)
}

/// Builds the `TreeNodePosition` input for `id`, mirroring `TreeNodePosition.run`.
fn layout_input(
    tree: &AdvancementTreeModel,
    manager: &ServerAdvancementManagerModel,
    id: &Identifier,
) -> AdvancementLayoutInput {
    let node = tree.get(id).unwrap_or_else(|| panic!("{id} missing from tree"));
    let children = node.children().map(|child| layout_input(tree, manager, child)).collect();
    let has_display = manager
        .get(id)
        .is_some_and(|holder| holder.value().display.is_some());
    AdvancementLayoutInput { id: id.to_string(), has_display, children }
}

#[test]
fn every_vendored_vanilla_advancement_loads_validates_and_round_trips() {
    let corpus = load_corpus();
    assert_eq!(corpus.entries.len(), 1617, "vendored advancement corpus size");
    let lookup = VendoredLookup { builtin: BuiltinLookup };
    let codec = AdvancementCodec::new(&lookup);

    let mut failures = Vec::new();
    let mut verified = Vec::new();
    let mut definitions = BTreeMap::new();
    for entry in &corpus.entries {
        match verify_entry(&codec, entry) {
            Ok(definition) => {
                verified.push(entry.checklist_path.clone());
                definitions.insert(entry.id.clone(), definition);
            }
            Err(err) => failures.push(format!("{}: {err}", entry.checklist_path)),
        }
    }
    assert!(failures.is_empty(), "{} advancement(s) failed:\n{}", failures.len(), failures.join("\n"));

    // (b) tree validation: manager warnings, parents, roots, and layout.
    let mut manager = ServerAdvancementManagerModel::default();
    manager.apply(definitions.clone());
    assert_eq!(manager.validation_warnings(), &[] as &[String]);
    for (id, definition) in &definitions {
        if let Some(parent) = &definition.parent {
            assert!(definitions.contains_key(parent), "{id} has unknown parent {parent}");
        }
    }
    let tree_size = manager.tree().nodes().count();
    assert_eq!(tree_size, definitions.len(), "every advancement must be inserted in the tree");
    let roots = manager.tree().roots().map(|root| root.id().clone()).collect::<Vec<_>>();
    let orphans = definitions.values().filter(|definition| definition.parent.is_none()).count();
    assert_eq!(roots.len(), orphans, "roots are exactly the parentless advancements");
    for root in manager.positioned_visible_roots() {
        let input = layout_input(manager.tree(), &manager, root);
        let positions = TreeNodePositionLayout::run(&input)
            .unwrap_or_else(|err| panic!("layout of {root} failed: {err}"));
        assert!(!positions.is_empty(), "layout of {root} produced no positions");
        assert!(positions.values().all(|position| position.y.is_finite() && position.y >= 0.0));
    }

    check_verified_list(&verified);
}

/// The committed verified list must equal the freshly computed one (regenerate with
/// `VIBECRAFT_UPDATE_VERIFIED_LIST=1`).
fn check_verified_list(verified: &[String]) {
    let mut sorted = verified.to_vec();
    sorted.sort();
    let expected = format!("{}\n", sorted.join("\n"));
    let path = manifest_path(VERIFIED_LIST);
    if std::env::var_os("VIBECRAFT_UPDATE_VERIFIED_LIST").is_some() {
        fs::write(&path, &expected).unwrap_or_else(|err| panic!("cannot write {}: {err}", path.display()));
    }
    let committed = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
    assert_eq!(committed, expected, "verified list is stale; rerun with VIBECRAFT_UPDATE_VERIFIED_LIST=1");
}

#[test]
fn corpus_exercises_only_triggers_with_condition_schemas() {
    let corpus = load_corpus();
    let supported = crate::advancement_condition_schema::supported_trigger_implementations();
    let registry = crate::advancement_trigger_registry::CriteriaTriggersModel::java_default_registry();
    let mut used = BTreeSet::new();
    for entry in &corpus.entries {
        let criteria = entry.document["criteria"].as_object().map(|map| map.values().collect::<Vec<_>>());
        for criterion in criteria.unwrap_or_default() {
            let trigger = criterion["trigger"].as_str().unwrap_or_default();
            let name = trigger.strip_prefix("minecraft:").unwrap_or(trigger);
            let known = registry.by_name(name).unwrap_or_else(|| panic!("{} uses unknown trigger {trigger}", entry.id));
            assert!(supported.contains(known.implementation), "{trigger} has no schema");
            used.insert(name.to_string());
        }
    }
    assert!(used.len() >= 50, "corpus should exercise most trigger types, saw {}", used.len());
}

#[test]
fn unknown_triggers_predicates_and_fields_fail_instead_of_being_skipped() {
    let lookup = VendoredLookup { builtin: BuiltinLookup };
    let codec = AdvancementCodec::new(&lookup);
    let attempt = |criterion: &str| {
        let document: Value = serde_json::from_str(&format!(r#"{{"criteria":{{"c":{criterion}}}}}"#))
            .unwrap_or_else(|err| panic!("{err}"));
        codec.round_trip(&document)
    };
    assert!(attempt(r#"{"trigger":"minecraft:not_a_trigger"}"#).unwrap_err().contains("unknown criterion trigger"));
    assert!(attempt(r#"{"trigger":"minecraft:tick","conditions":{"bogus":1}}"#).unwrap_err().contains("unknown field"));
    assert!(attempt(
        r#"{"trigger":"minecraft:tick","conditions":{"player":[{"condition":"minecraft:bogus"}]}}"#
    )
    .is_err());
    assert!(attempt(
        r#"{"trigger":"minecraft:tick","conditions":{"player":{"type":"minecraft:not_an_entity"}}}"#
    )
    .unwrap_err()
    .contains("unknown EntityType entry"));
    assert!(attempt(
        r#"{"trigger":"minecraft:tick","conditions":{"player":{"type_specific":{"type":"minecraft:bogus"}}}}"#
    )
    .is_err());
    assert!(attempt(r#"{"trigger":"minecraft:tick","extra":1}"#).unwrap_err().contains("unknown field"));
    assert!(attempt(r#"{"trigger":"minecraft:recipe_unlocked"}"#).unwrap_err().contains("recipe"));
}

#[test]
fn round_trip_normalizes_defaults_like_java() {
    let lookup = BuiltinLookup;
    let codec = AdvancementCodec::new(&lookup);
    let document: Value = serde_json::from_str(
        r#"{"criteria":{"c":{"trigger":"minecraft:inventory_changed","conditions":{
            "items":[{"items":["minecraft:stone"],"count":{"min":2,"max":2}}],"slots":{}}}},
            "rewards":{"experience":0}}"#,
    )
    .unwrap_or_else(|err| panic!("{err}"));
    let encoded = codec.round_trip(&document).unwrap_or_else(|err| panic!("{err}"));
    let expected: Value = serde_json::from_str(
        r#"{"criteria":{"c":{"trigger":"minecraft:inventory_changed","conditions":{
            "items":[{"items":"minecraft:stone","count":2}]}}},"requirements":[["c"]]}"#,
    )
    .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(encoded, expected);
}

/// Every ticked advancement line in the checklist must be a file the corpus test verified.
#[test]
fn ticked_advancement_checklist_lines_are_all_verified() {
    let verified = fs::read_to_string(manifest_path(VERIFIED_LIST))
        .unwrap_or_else(|err| panic!("cannot read verified list: {err}"))
        .lines()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let checklist = fs::read_to_string(manifest_path(CHECKLIST))
        .unwrap_or_else(|err| panic!("cannot read {CHECKLIST}: {err}"));
    let marker = format!("`{CHECKLIST_PREFIX}advancement/");
    let mut ticked = 0;
    for line in checklist.lines().filter(|line| line.starts_with("- [x]") && line.contains(&marker)) {
        let start = line.find(&marker).map_or(0, |index| index + 1);
        let path = line[start..].split('`').next().unwrap_or_default();
        assert!(verified.contains(path), "ticked line for {path} is not in the verified list");
        ticked += 1;
    }
    assert!(ticked <= verified.len());
}
