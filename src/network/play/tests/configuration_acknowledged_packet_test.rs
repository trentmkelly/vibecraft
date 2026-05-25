use super::*;

#[test]
fn serverbound_configuration_acknowledged_packet_matches_java_unit_codec() {
    assert_eq!(SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID, 16);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID),
        Some("configuration_acknowledged")
    );

    let mut payload = Vec::new();
    ServerboundConfigurationAcknowledgedPacket
        .write(&mut payload)
        .unwrap();
    assert!(payload.is_empty());
    assert_eq!(
        ServerboundConfigurationAcknowledgedPacket::read(&mut cursor(payload.clone())).unwrap(),
        ServerboundConfigurationAcknowledgedPacket
    );

    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.state, PlayState::Reconfiguring);
}

#[test]
fn serverbound_configuration_acknowledged_packet_rejects_extra_payload() {
    assert!(ServerboundConfigurationAcknowledgedPacket::read(&mut cursor(vec![0x00])).is_err());

    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID,
            vec![0x00]
        )),
        DispatchOutcome::Disconnect(reason)
            if reason.starts_with("bad configuration acknowledged packet:")
    ));
}
