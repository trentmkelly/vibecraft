//! `RegistrySynchronization` and `TagNetworkSerialization`: the configuration-phase
//! registry data and tag payloads generated from the frozen [`Registries`].

use std::io::{self, Write};

use crate::network::codec::write_identifier;
use crate::network::configuration::{
    ClientboundRegistryDataPacket, KnownPack, PackedRegistryEntry,
};
use crate::network::varint::write_var_i32;
use crate::registry::Identifier;
use crate::registry_pipeline::builtin::BuiltinRegistries;
use crate::registry_pipeline::codec::CodecContext;
use crate::registry_pipeline::registry_data::element_codecs;
use crate::registry_pipeline::store::Registries;
use crate::resource_registry_data_loader::SYNCHRONIZED_REGISTRIES;

/// `RegistrySynchronization.packRegistries`: one packet per synchronised registry
/// present in the worldgen layer, in `SYNCHRONIZED_REGISTRIES` order.
///
/// Elements whose source pack is one of `client_known_packs` are sent without
/// contents; the client rebuilds them from its own copy of the pack.
pub fn pack_registries(
    registries: &Registries,
    builtin: &BuiltinRegistries,
    client_known_packs: &[KnownPack],
) -> Result<Vec<ClientboundRegistryDataPacket>, String> {
    let loading = std::collections::BTreeSet::new();
    let ctx = CodecContext::new(builtin, &loading);
    let mut packets = Vec::with_capacity(SYNCHRONIZED_REGISTRIES.len());
    for data in SYNCHRONIZED_REGISTRIES {
        let key = Identifier::parse(data.key)
            .map_err(|err| format!("invalid synchronised registry {}: {err}", data.key))?;
        let Some(registry) = registries
            .worldgen_layer()
            .iter()
            .find(|registry| registry.key() == &key)
        else {
            continue;
        };
        let codecs = element_codecs(data.key)
            .ok_or_else(|| format!("no element codec for synchronised registry {}", data.key))?;
        let mut entries = Vec::with_capacity(registry.len());
        for element in registry.elements() {
            let can_skip_contents = element
                .info
                .known_pack
                .as_ref()
                .is_some_and(|pack| client_known_packs.contains(pack));
            let contents = if can_skip_contents {
                None
            } else {
                Some(
                    codecs
                        .network
                        .parse(&element.json, &ctx)
                        .map_err(|err| format!("Failed to serialize {}: {err}", element.key))?,
                )
            };
            entries.push(PackedRegistryEntry {
                id: element.key.clone(),
                data: contents,
            });
        }
        packets.push(ClientboundRegistryDataPacket {
            registry: key,
            entries,
        });
    }
    Ok(packets)
}

/// `TagNetworkSerialization.NetworkPayload` for one registry: tag id to element ids.
pub type TagPayload = Vec<(Identifier, Vec<i32>)>;

/// `TagNetworkSerialization.serializeTagsToNetwork`: the tags of every
/// network-safe registry (synchronised dynamic registries, then all static ones)
/// whose payload is non-empty.
pub fn serialize_tags_to_network(registries: &Registries) -> Vec<(Identifier, TagPayload)> {
    let networked = SYNCHRONIZED_REGISTRIES.iter().filter_map(|data| {
        registries
            .worldgen_layer()
            .iter()
            .find(|registry| registry.key().to_string() == data.key)
    });
    networked
        .chain(registries.static_layer().iter())
        .filter_map(|registry| {
            let payload: TagPayload = registry
                .tags()
                .iter()
                .map(|(tag, elements)| {
                    (tag.clone(), elements.iter().map(|id| *id as i32).collect())
                })
                .collect();
            (!payload.is_empty()).then(|| (registry.key().clone(), payload))
        })
        .collect()
}

/// `ClientboundUpdateTagsPacket.STREAM_CODEC`: `Map<registry, NetworkPayload>`.
pub fn write_update_tags_packet<W: Write>(
    writer: &mut W,
    tags: &[(Identifier, TagPayload)],
) -> io::Result<()> {
    write_var_i32(writer, tags.len() as i32)?;
    for (registry, payload) in tags {
        write_identifier(writer, registry)?;
        write_var_i32(writer, payload.len() as i32)?;
        for (tag, ids) in payload {
            write_identifier(writer, tag)?;
            write_var_i32(writer, ids.len() as i32)?;
            for id in ids {
                write_var_i32(writer, *id)?;
            }
        }
    }
    Ok(())
}
