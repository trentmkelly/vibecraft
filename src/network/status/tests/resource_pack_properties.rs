use super::super::{
    read_expected_configuration_resource_pack_response, server_resource_pack_push_packet,
    write_framed_packet, SERVERBOUND_CONFIGURATION_RESOURCE_PACK_PACKET_ID,
};
use crate::network::codec::{ComponentJson, Uuid};
use crate::network::common::{
    ClientboundResourcePackPushPacket, ResourcePackAction, ServerboundResourcePackPacket,
};
use crate::network::compression::CompressionState;
use crate::network::rate_limit::PacketRateLimiter;
use crate::server_properties::ServerProperties;
use std::path::Path;
use std::time::Instant;

#[test]
pub fn server_resource_pack_properties_match_java_server_pack_info_rules() {
    let mut properties =
        ServerProperties::load_or_default(Path::new("definitely-missing-test-server.properties"))
            .unwrap();
    assert_eq!(server_resource_pack_push_packet(&properties), None);

    properties.set("resource-pack", "https://fallback.example/pack.zip");
    properties.set("resource-pack-sha1", "");
    properties.set("resource-pack-hash", "legacyhash");
    assert_eq!(
        server_resource_pack_push_packet(&properties).map(|packet| (packet.id, packet.hash)),
        Some((
            Uuid([
                0x92, 0xeb, 0xd2, 0x68, 0x95, 0x1e, 0x32, 0xd9, 0x9a, 0xcf, 0x3a, 0x76, 0xd2,
                0x35, 0xa8, 0x6d,
            ]),
            "legacyhash".to_string(),
        ))
    );

    properties.set("resource-pack-id", "00000000-0000-0000-0000-000000000001");
    properties.set("resource-pack-sha1", "0123456789abcdef0123456789abcdef01234567");
    properties.set("require-resource-pack", "true");
    properties.set("resource-pack-prompt", "{\"text\":\"Use pack?\"}");
    assert_eq!(
        server_resource_pack_push_packet(&properties),
        Some(ClientboundResourcePackPushPacket {
            id: Uuid([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
            url: "https://fallback.example/pack.zip".to_string(),
            hash: "0123456789abcdef0123456789abcdef01234567".to_string(),
            required: true,
            prompt: Some(ComponentJson("{\"text\":\"Use pack?\"}".to_string())),
        })
    );

    properties.set("resource-pack-id", "not-a-uuid");
    assert_eq!(server_resource_pack_push_packet(&properties), None);

    properties.set("resource-pack-id", "");
    properties.set("resource-pack-prompt", "{not json");
    assert_eq!(
        server_resource_pack_push_packet(&properties).and_then(|packet| packet.prompt),
        None
    );
}

#[test]
pub fn configuration_resource_pack_wait_finishes_on_terminal_status_like_java() {
    let (mut server, mut client) = super::loopback_pair();
    let registry = super::ActiveLoginRegistry::default();
    let alex = crate::player_access::NameAndId::create_offline("Alex");
    let (guard, _) = registry
        .register_replacing(&alex.uuid, "Alex", &client)
        .unwrap();
    write_resource_pack_response(&mut client, ResourcePackAction::Accepted);
    write_resource_pack_response(&mut client, ResourcePackAction::Downloaded);
    write_resource_pack_response(&mut client, ResourcePackAction::SuccessfullyLoaded);

    let mut rate_limiter = PacketRateLimiter::new(0, Instant::now());
    let response = read_expected_configuration_resource_pack_response(
        &mut server,
        CompressionState::disabled(),
        &mut rate_limiter,
        &guard,
    )
    .unwrap();

    assert_eq!(response.action, ResourcePackAction::SuccessfullyLoaded);
}

fn write_resource_pack_response(stream: &mut std::net::TcpStream, action: ResourcePackAction) {
    write_framed_packet(stream, SERVERBOUND_CONFIGURATION_RESOURCE_PACK_PACKET_ID, |payload| {
        ServerboundResourcePackPacket {
            id: Uuid([7; 16]),
            action,
        }
        .write(payload)
    })
    .unwrap();
}
