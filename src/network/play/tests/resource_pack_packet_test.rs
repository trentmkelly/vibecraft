use super::*;
use crate::network::common::ResourcePackAction;

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
