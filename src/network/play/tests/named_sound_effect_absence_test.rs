use super::*;

#[test]
fn clientbound_named_sound_effect_packet_is_absent_in_java_26_1_2() {
    let registry = PlayProtocolRegistry::new();

    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SOUND_ENTITY_PACKET_ID),
        Some("sound_entity")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SOUND_PACKET_ID),
        Some("sound")
    );
    assert_eq!(registry.clientbound_name(119), Some("stop_sound"));
    assert!(!registry.clientbound().contains(&"named_sound"));
    assert!(!registry.clientbound().contains(&"named_sound_effect"));
}
