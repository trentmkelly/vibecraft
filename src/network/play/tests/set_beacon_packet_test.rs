use super::*;

#[test]
fn serverbound_set_beacon_packet_matches_java_codec() {
    assert_eq!(SERVERBOUND_SET_BEACON_PACKET_ID, 52);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_BEACON_PACKET_ID),
        Some("set_beacon")
    );

    let cases = [
        (
            ServerboundSetBeaconPacket {
                primary_effect_id: None,
                secondary_effect_id: None,
            },
            vec![0, 0],
        ),
        (
            ServerboundSetBeaconPacket {
                primary_effect_id: Some(1),
                secondary_effect_id: Some(39),
            },
            vec![1, 1, 1, 39],
        ),
    ];

    for (packet, expected_payload) in cases {
        let mut payload = Vec::new();
        packet.write(&mut payload).unwrap();
        assert_eq!(payload, expected_payload);
        assert_eq!(
            ServerboundSetBeaconPacket::read(&mut cursor(payload.clone())).unwrap(),
            packet
        );

        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_SET_BEACON_PACKET_ID, payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_set_beacon, Some(packet));
    }
}

#[test]
fn serverbound_set_beacon_packet_rejects_malformed_payloads() {
    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SET_BEACON_PACKET_ID, vec![0])),
        DispatchOutcome::Disconnect(reason) if reason.starts_with("bad set beacon packet:")
    ));

    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SET_BEACON_PACKET_ID, vec![1, 40, 0])),
        DispatchOutcome::Disconnect(reason) if reason.starts_with("bad set beacon packet:")
    ));

    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SET_BEACON_PACKET_ID, vec![0, 0, 0])),
        DispatchOutcome::Disconnect(reason) if reason == "bad set beacon packet: trailing payload"
    ));

    assert!(ServerboundSetBeaconPacket {
        primary_effect_id: Some(40),
        secondary_effect_id: None,
    }
    .write(&mut Vec::new())
    .is_err());
}
