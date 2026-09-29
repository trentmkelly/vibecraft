//! Configuration-phase registry synchronisation (`SynchronizeRegistriesTask`).
//!
//! Java order: `ClientboundSelectKnownPacks` -> wait for the client's answer ->
//! one `ClientboundRegistryDataPacket` per synchronised registry -> one
//! `ClientboundUpdateTagsPacket`. The payloads are generated from the frozen
//! registries in [`crate::registry_pipeline`], not from static tables.

use super::*;
use crate::network::configuration::{
    ClientboundRegistryDataPacket, ClientboundSelectKnownPacks, KnownPack,
    ServerboundSelectKnownPacks,
};
use crate::registry_pipeline::builtin::BuiltinRegistries;
use crate::registry_pipeline::resources::ResourceManager;
use crate::registry_pipeline::sync::{
    pack_registries, serialize_tags_to_network, write_update_tags_packet,
};
use crate::registry_pipeline::vanilla_registries;

fn registry_error(message: String) -> io::Error {
    io::Error::other(message)
}

/// Runs `SynchronizeRegistriesTask` on the configuration connection.
pub(super) fn run_synchronize_registries_task(
    stream: &mut TcpStream,
    compression: CompressionState,
    rate_limiter: &mut PacketRateLimiter,
    active_login: &ActiveLoginGuard,
) -> io::Result<()> {
    let registries = vanilla_registries().map_err(registry_error)?;
    let builtin = BuiltinRegistries::vanilla().map_err(registry_error)?;

    // `SynchronizeRegistriesTask.start`: offer every pack the server can describe.
    let requested = ResourceManager::vanilla().known_packs();
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        |payload| {
            ClientboundSelectKnownPacks {
                known_packs: requested.clone(),
            }
            .write(payload)
        },
    )?;
    let accepted = match wait_for_configuration_packet_body_with_rate_limit(
        stream,
        compression,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
        rate_limiter,
        Some(active_login),
    ) {
        Ok(body) => ServerboundSelectKnownPacks::read(&mut Cursor::new(body))?.known_packs,
        Err(err) if is_rate_limit_disconnect_error(&err) => {
            return write_configuration_rate_limit_disconnect(
                stream,
                compression,
                &err.to_string(),
            );
        }
        Err(err) => return Err(err),
    };

    // `handleResponse`: contents may only be elided when the client accepted
    // exactly the offered packs.
    let negotiated: &[KnownPack] = if accepted == requested {
        &requested
    } else {
        &[]
    };
    for packet in pack_registries(registries, builtin, negotiated).map_err(registry_error)? {
        write_registry_data_packet(stream, compression, &packet)?;
    }
    let tags = serialize_tags_to_network(registries);
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID,
        |payload| write_update_tags_packet(payload, &tags),
    )
}

fn write_registry_data_packet(
    stream: &mut TcpStream,
    compression: CompressionState,
    packet: &ClientboundRegistryDataPacket,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        |payload| packet.write(payload),
    )
}
