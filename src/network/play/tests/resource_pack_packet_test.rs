use super::*;
use crate::network::codec::ComponentJson;
use crate::network::common::{
    ClientboundResourcePackPushPacket as CommonClientboundResourcePackPushPacket,
    ResourcePackAction,
};

#[test]
fn clientbound_resource_pack_pop_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_RESOURCE_PACK_POP_PACKET_ID, 80);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_RESOURCE_PACK_POP_PACKET_ID),
        Some("resource_pack_pop")
    );

    let absent = ClientboundResourcePackPopPacket { id: None };
    let mut absent_payload = Vec::new();
    absent.write(&mut absent_payload).unwrap();
    assert_eq!(absent_payload, vec![0]);

    let id = Uuid([
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f,
    ]);
    let present = ClientboundResourcePackPopPacket { id: Some(id) };
    let mut present_payload = Vec::new();
    present.write(&mut present_payload).unwrap();

    let mut expected_present = vec![1];
    expected_present.extend_from_slice(&id.0);
    assert_eq!(present_payload, expected_present);
}

#[test]
fn clientbound_resource_pack_push_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID, 81);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID),
        Some("resource_pack_push")
    );

    let packet = CommonClientboundResourcePackPushPacket {
        id: Uuid([
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d,
            0x2e, 0x2f,
        ]),
        url: "https://example.invalid/pack.zip".to_string(),
        hash: "0123456789abcdef0123456789abcdef01234567".to_string(),
        required: true,
        prompt: Some(ComponentJson("{\"text\":\"Use pack?\"}".to_string())),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let mut expected = packet.id.0.to_vec();
    expected.push(packet.url.len() as u8);
    expected.extend_from_slice(packet.url.as_bytes());
    expected.push(packet.hash.len() as u8);
    expected.extend_from_slice(packet.hash.as_bytes());
    expected.push(1);
    expected.push(1);
    expected.extend_from_slice(&[8, 0, 9]);
    expected.extend_from_slice(b"Use pack?");
    assert_eq!(payload, expected);
    assert_eq!(
        CommonClientboundResourcePackPushPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn serverbound_resource_pack_packet_matches_java_codec() {
    assert_eq!(SERVERBOUND_RESOURCE_PACK_PACKET_ID, 49);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_RESOURCE_PACK_PACKET_ID),
        Some("resource_pack")
    );

    let id = Uuid([
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ]);
    let actions = [
        ResourcePackAction::SuccessfullyLoaded,
        ResourcePackAction::Declined,
        ResourcePackAction::FailedDownload,
        ResourcePackAction::Accepted,
        ResourcePackAction::Downloaded,
        ResourcePackAction::InvalidUrl,
        ResourcePackAction::FailedReload,
        ResourcePackAction::Discarded,
    ];

    for (ordinal, action) in actions.into_iter().enumerate() {
        let packet = ServerboundResourcePackPacket { id, action };
        let mut payload = Vec::new();
        packet.write(&mut payload).unwrap();

        let mut expected = id.0.to_vec();
        expected.push(ordinal as u8);
        assert_eq!(payload, expected);
        assert_eq!(
            ServerboundResourcePackPacket::read(&mut cursor(payload.clone())).unwrap(),
            packet
        );

        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_RESOURCE_PACK_PACKET_ID, payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_resource_pack_response, Some(packet));
    }
}

#[test]
fn serverbound_resource_pack_packet_rejects_malformed_payloads() {
    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_RESOURCE_PACK_PACKET_ID, vec![0; 16])),
        DispatchOutcome::Disconnect(reason) if reason.starts_with("bad resource pack packet:")
    ));

    let mut invalid_action = vec![0; 16];
    invalid_action.push(8);
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_RESOURCE_PACK_PACKET_ID, invalid_action)),
        DispatchOutcome::Disconnect(reason) if reason.starts_with("bad resource pack packet:")
    ));

    let mut trailing_payload = vec![0; 16];
    trailing_payload.extend_from_slice(&[ResourcePackAction::Accepted as u8, 0x00]);
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_RESOURCE_PACK_PACKET_ID, trailing_payload)),
        DispatchOutcome::Disconnect(reason) if reason == "bad resource pack packet: trailing payload"
    ));
}
