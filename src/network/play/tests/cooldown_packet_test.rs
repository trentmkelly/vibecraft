use super::*;

#[test]
fn clientbound_cooldown_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_COOLDOWN_PACKET_ID, 22);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_COOLDOWN_PACKET_ID),
        Some("cooldown")
    );

    let packet = ClientboundCooldownPacket {
        cooldown_group: Identifier::parse("minecraft:ender_pearl").unwrap(),
        duration: 300,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        [
            vec![21],
            b"minecraft:ender_pearl".to_vec(),
            vec![0xac, 0x02]
        ]
        .concat()
    );
}
