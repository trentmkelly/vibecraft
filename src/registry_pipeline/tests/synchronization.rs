//! `RegistrySynchronization.packRegistries` and the tags payload.

use super::{builtin, vanilla_with, TestPackDir};
use crate::network::configuration::KnownPack;
use crate::registry::Identifier;
use crate::registry_pipeline::load_registries;
use crate::registry_pipeline::resources::vanilla_known_pack;
use crate::registry_pipeline::sync::{
    pack_registries, serialize_tags_to_network, write_update_tags_packet,
};
use crate::storage::nbt::Tag;

#[test]
fn entries_from_a_known_pack_are_sent_without_contents() {
    let core = vec![vanilla_known_pack()];
    let packets = pack_registries(super::registries(), builtin(), &core).expect("pack");
    assert_eq!(packets.len(), 28);
    for packet in &packets {
        assert!(!packet.entries.is_empty(), "{}", packet.registry);
        assert!(
            packet.entries.iter().all(|entry| entry.data.is_none()),
            "{} sent contents for known-pack entries",
            packet.registry
        );
    }
}

#[test]
fn a_client_without_matching_packs_receives_full_contents() {
    // handleResponse: any answer other than the offered packs means "no packs".
    let packets = pack_registries(super::registries(), builtin(), &[]).expect("pack");
    for packet in &packets {
        assert!(
            packet.entries.iter().all(|entry| entry.data.is_some()),
            "{} elided contents for a client without the pack",
            packet.registry
        );
    }
    // A different version of the same pack is not the offered pack either.
    let other_version = vec![KnownPack::vanilla_with_version("core", "1.0")];
    let stale = pack_registries(super::registries(), builtin(), &other_version).expect("pack");
    assert!(stale
        .iter()
        .all(|packet| packet.entries.iter().all(|entry| entry.data.is_some())));
}

#[test]
fn data_pack_entries_are_sent_in_full_even_when_the_client_knows_core() {
    let pack = TestPackDir::new();
    pack.write(
        "custom/banner_pattern/mine.json",
        r#"{"asset_id":"custom:mine","translation_key":"custom.mine"}"#,
    );
    pack.write(
        "minecraft/banner_pattern/base.json",
        r#"{"asset_id":"minecraft:base","translation_key":"custom.base"}"#,
    );
    let registries =
        load_registries(&vanilla_with(pack.pack("custom_pack", None)), builtin()).expect("load");
    let core = vec![vanilla_known_pack()];
    let packets = pack_registries(&registries, builtin(), &core).expect("pack");
    let banner = packets
        .iter()
        .find(|packet| {
            packet.registry == Identifier::parse("minecraft:banner_pattern").expect("id")
        })
        .expect("banner_pattern packet");
    let by_id = |name: &str| {
        banner
            .entries
            .iter()
            .find(|entry| entry.id.to_string() == name)
            .unwrap_or_else(|| panic!("{name} missing"))
    };
    // A new element and an overridden vanilla element come from a pack the client
    // does not have, so they carry data; untouched vanilla elements do not.
    assert!(by_id("custom:mine").data.is_some());
    assert!(by_id("minecraft:base").data.is_some());
    assert!(by_id("minecraft:border").data.is_none());
    assert_eq!(banner.entries.len(), 44);
}

#[test]
fn update_tags_packet_uses_the_protocol_layout() {
    let payload = serialize_tags_to_network(super::registries());
    let mut bytes = Vec::new();
    write_update_tags_packet(&mut bytes, &payload).expect("write");

    let mut cursor = std::io::Cursor::new(bytes);
    let read_var = |cursor: &mut std::io::Cursor<Vec<u8>>| {
        crate::network::varint::read_var_i32(cursor).expect("varint")
    };
    let registry_count = read_var(&mut cursor);
    assert_eq!(registry_count as usize, payload.len());
    for (registry, tags) in &payload {
        let read_id = crate::network::codec::read_identifier(&mut cursor).expect("registry");
        assert_eq!(&read_id, registry);
        assert_eq!(read_var(&mut cursor) as usize, tags.len());
        for (tag, ids) in tags {
            assert_eq!(
                &crate::network::codec::read_identifier(&mut cursor).expect("tag"),
                tag
            );
            assert_eq!(read_var(&mut cursor) as usize, ids.len());
            for expected in ids {
                assert_eq!(read_var(&mut cursor), *expected);
            }
        }
    }
    assert_eq!(cursor.position() as usize, cursor.get_ref().len());
}

#[test]
fn registry_data_packet_round_trips_full_and_elided_entries() {
    let packets = pack_registries(super::registries(), builtin(), &[]).expect("pack");
    let banner = packets
        .into_iter()
        .find(|packet| packet.registry.to_string() == "minecraft:banner_pattern")
        .expect("packet");
    let mut bytes = Vec::new();
    banner.write(&mut bytes).expect("write");
    let decoded = crate::network::configuration::ClientboundRegistryDataPacket::read(
        &mut std::io::Cursor::new(bytes),
    )
    .expect("read");
    assert_eq!(decoded, banner);
    assert!(matches!(decoded.entries[0].data, Some(Tag::Compound(_))));
}
