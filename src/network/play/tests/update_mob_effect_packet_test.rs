use super::*;

#[test]
fn clientbound_update_mob_effect_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID, 132);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID),
        Some("update_mob_effect")
    );

    assert_eq!(
        MobEffectFlags::from_parts(true, true, true, true),
        MobEffectFlags(0x0f)
    );
    assert_eq!(
        MobEffectFlags::from_parts(false, true, true, true),
        MobEffectFlags(0x0e)
    );
    assert_eq!(
        MobEffectFlags::from_parts(true, false, true, true),
        MobEffectFlags(0x0d)
    );

    let packet = ClientboundUpdateMobEffectPacket {
        entity_id: 129,
        effect_id: 5,
        amplifier: 2,
        duration_ticks: 600,
        flags: MobEffectFlags::from_parts(true, false, true, true),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(payload, vec![0x81, 0x01, 5, 2, 0xd8, 0x04, 0x0d]);
}
