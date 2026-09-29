//! Comparison against the registry sync of the official 26.1.2 server.
//!
//! The fixtures were captured from an unmodified vanilla `server.jar` by
//! `tools/capture_official_registry_sync.py` with a client that offers no known
//! packs, so every registry entry carries its full network NBT.

use std::collections::BTreeMap;
use std::io::Cursor;

use super::{builtin, registries};
use crate::network::codec::read_identifier;
use crate::network::configuration::ClientboundRegistryDataPacket;
use crate::network::varint::read_var_i32;
use crate::registry::Identifier;
use crate::registry_pipeline::sync::{pack_registries, serialize_tags_to_network};
use crate::storage::nbt::Tag;

const OFFICIAL_REGISTRY_DATA: &[u8] = include_bytes!("fixtures/official_registry_data_26_1_2.bin");
const OFFICIAL_UPDATE_TAGS: &[u8] = include_bytes!("fixtures/official_update_tags_26_1_2.bin");

/// Java's `CompoundTag` is hash-ordered, so compounds compare as unordered maps.
fn canonical(tag: &Tag) -> Tag {
    match tag {
        Tag::Compound(fields) => {
            let mut sorted: Vec<(String, Tag)> = fields
                .iter()
                .map(|(key, value)| (key.clone(), canonical(value)))
                .collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            Tag::Compound(sorted)
        }
        Tag::List(items) => Tag::List(items.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

/// Paths at which two canonical tags differ (for readable failure output).
fn first_differences(a: &Tag, b: &Tag, path: String, out: &mut Vec<String>) {
    match (a, b) {
        (Tag::Compound(x), Tag::Compound(y)) => {
            let keys: std::collections::BTreeSet<&String> =
                x.iter().chain(y.iter()).map(|(key, _)| key).collect();
            for key in keys {
                let left = x.iter().find(|(k, _)| k == key).map(|(_, v)| v);
                let right = y.iter().find(|(k, _)| k == key).map(|(_, v)| v);
                match (left, right) {
                    (Some(l), Some(r)) => first_differences(l, r, format!("{path}/{key}"), out),
                    (l, r) => out.push(format!("{path}/{key}: official {l:?} ours {r:?}")),
                }
            }
        }
        (Tag::List(x), Tag::List(y)) if x.len() == y.len() => {
            for (index, (l, r)) in x.iter().zip(y).enumerate() {
                first_differences(l, r, format!("{path}[{index}]"), out);
            }
        }
        (l, r) if l != r => out.push(format!("{path}: official {l:?} ours {r:?}")),
        _ => {}
    }
}

fn official_registry_packets() -> Vec<ClientboundRegistryDataPacket> {
    let mut cursor = Cursor::new(OFFICIAL_REGISTRY_DATA);
    let mut packets = Vec::new();
    while (cursor.position() as usize) < OFFICIAL_REGISTRY_DATA.len() {
        let length = read_var_i32(&mut cursor).expect("packet length") as usize;
        let start = cursor.position() as usize;
        let body = &OFFICIAL_REGISTRY_DATA[start..start + length];
        packets.push(
            ClientboundRegistryDataPacket::read(&mut Cursor::new(body)).expect("registry packet"),
        );
        cursor.set_position((start + length) as u64);
    }
    packets
}

/// Elements whose generic (schema-less) conversion still differs from Java's typed
/// `Enchantment.DIRECT_CODEC` encoding: `BlockPos` offsets are `IntArrayTag`s and
/// `Vec3`/`MinMaxBounds.Doubles` leaves are `DoubleTag`s, which the schema-less
/// converter cannot know. The client decodes both forms identically.
///
/// TODO(registry-pipeline-enchantment): port the typed enchantment codec, then
/// remove this allow-list so every element must match exactly.
const KNOWN_INEXACT: &[&str] = &[
    "minecraft:enchantment/minecraft:frost_walker",
    "minecraft:enchantment/minecraft:lunge",
    "minecraft:enchantment/minecraft:wind_burst",
];

#[test]
fn registry_data_matches_the_official_server_entry_for_entry() {
    let official = official_registry_packets();
    let ours = pack_registries(registries(), builtin(), &[]).expect("pack");

    let official_order: Vec<String> = official.iter().map(|p| p.registry.to_string()).collect();
    let our_order: Vec<String> = ours.iter().map(|p| p.registry.to_string()).collect();
    assert_eq!(our_order, official_order, "registry packet order");

    let mut mismatches = Vec::new();
    for (theirs, mine) in official.iter().zip(&ours) {
        let their_ids: Vec<String> = theirs.entries.iter().map(|e| e.id.to_string()).collect();
        let my_ids: Vec<String> = mine.entries.iter().map(|e| e.id.to_string()).collect();
        assert_eq!(my_ids, their_ids, "{} element order", mine.registry);
        for (their_entry, my_entry) in theirs.entries.iter().zip(&mine.entries) {
            let their_data = their_entry.data.as_ref().map(canonical);
            let my_data = my_entry.data.as_ref().map(canonical);
            let label = format!("{}/{}", mine.registry, my_entry.id);
            if their_data != my_data && !KNOWN_INEXACT.contains(&label.as_str()) {
                let mut diffs = Vec::new();
                if let (Some(a), Some(b)) = (&their_data, &my_data) {
                    first_differences(a, b, String::new(), &mut diffs);
                }
                mismatches.push(format!("{label}: {}", diffs.join("; ")));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} entries differ from the official server:\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Parses an `update_tags` body into registry -> tag -> ids.
fn parse_update_tags(body: &[u8]) -> BTreeMap<Identifier, BTreeMap<Identifier, Vec<i32>>> {
    let mut cursor = Cursor::new(body);
    let mut registries = BTreeMap::new();
    let registry_count = read_var_i32(&mut cursor).expect("registry count");
    for _ in 0..registry_count {
        let registry = read_identifier(&mut cursor).expect("registry");
        let tag_count = read_var_i32(&mut cursor).expect("tag count");
        let mut tags = BTreeMap::new();
        for _ in 0..tag_count {
            let tag = read_identifier(&mut cursor).expect("tag");
            let id_count = read_var_i32(&mut cursor).expect("id count");
            let ids = (0..id_count)
                .map(|_| read_var_i32(&mut cursor).expect("id"))
                .collect();
            tags.insert(tag, ids);
        }
        registries.insert(registry, tags);
    }
    assert_eq!(cursor.position() as usize, body.len(), "trailing bytes");
    registries
}

#[test]
fn update_tags_match_the_official_server_for_every_registry_and_tag() {
    let official = parse_update_tags(OFFICIAL_UPDATE_TAGS);
    let ours: BTreeMap<Identifier, BTreeMap<Identifier, Vec<i32>>> =
        serialize_tags_to_network(registries())
            .into_iter()
            .map(|(registry, tags)| (registry, tags.into_iter().collect()))
            .collect();

    let official_registries: Vec<String> = official.keys().map(ToString::to_string).collect();
    let our_registries: Vec<String> = ours.keys().map(ToString::to_string).collect();
    assert_eq!(our_registries, official_registries, "tag registries");

    for (registry, their_tags) in &official {
        let my_tags = &ours[registry];
        let their_names: Vec<String> = their_tags.keys().map(ToString::to_string).collect();
        let my_names: Vec<String> = my_tags.keys().map(ToString::to_string).collect();
        assert_eq!(my_names, their_names, "{registry} tag names");
        for (name, their_ids) in their_tags {
            assert_eq!(&my_tags[name], their_ids, "{registry} #{name}");
        }
    }
}

#[test]
fn known_inexact_entries_still_differ_so_the_allow_list_stays_minimal() {
    // If a listed element starts matching, remove it from KNOWN_INEXACT.
    let official = official_registry_packets();
    let ours = pack_registries(registries(), builtin(), &[]).expect("pack");
    for (theirs, mine) in official.iter().zip(&ours) {
        for (their_entry, my_entry) in theirs.entries.iter().zip(&mine.entries) {
            let label = format!("{}/{}", mine.registry, my_entry.id);
            if KNOWN_INEXACT.contains(&label.as_str()) {
                let their_data = their_entry.data.as_ref().map(canonical);
                let my_data = my_entry.data.as_ref().map(canonical);
                assert_ne!(
                    their_data, my_data,
                    "{label} now matches; drop it from the list"
                );
            }
        }
    }
}
