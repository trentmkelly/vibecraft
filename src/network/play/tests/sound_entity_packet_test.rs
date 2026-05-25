use super::*;

#[test]
fn clientbound_sound_entity_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SOUND_ENTITY_PACKET_ID, 116);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SOUND_ENTITY_PACKET_ID),
        Some("sound_entity")
    );

    let mut registered_payload = Vec::new();
    ClientboundSoundPacket {
        sound: SoundEventHolder::Registered { id: 5 },
        source_id: SoundSource::Blocks as i32,
        position: Vec3::ZERO,
        volume: 0.75,
        pitch: 1.25,
        seed: -9,
        entity_id: Some(300),
    }
    .write_entity(&mut registered_payload)
    .unwrap();

    let mut expected_registered = Vec::new();
    expected_registered.extend_from_slice(&[0x06]);
    expected_registered.extend_from_slice(&[SoundSource::Blocks as u8]);
    expected_registered.extend_from_slice(&[0xac, 0x02]);
    expected_registered.extend_from_slice(&0.75_f32.to_be_bytes());
    expected_registered.extend_from_slice(&1.25_f32.to_be_bytes());
    expected_registered.extend_from_slice(&(-9_i64).to_be_bytes());
    assert_eq!(registered_payload, expected_registered);

    let mut direct_payload = Vec::new();
    ClientboundSoundPacket {
        sound: SoundEventHolder::Direct {
            location: Identifier::parse("minecraft:test.sound").unwrap(),
            fixed_range: Some(16.0),
        },
        source_id: SoundSource::Players as i32,
        position: Vec3::ZERO,
        volume: 1.0,
        pitch: 0.5,
        seed: 42,
        entity_id: Some(1),
    }
    .write_entity(&mut direct_payload)
    .unwrap();

    let mut expected_direct = Vec::new();
    expected_direct.extend_from_slice(&[0x00, 0x14]);
    expected_direct.extend_from_slice(b"minecraft:test.sound");
    expected_direct.extend_from_slice(&[0x01]);
    expected_direct.extend_from_slice(&16.0_f32.to_be_bytes());
    expected_direct.extend_from_slice(&[SoundSource::Players as u8]);
    expected_direct.extend_from_slice(&[0x01]);
    expected_direct.extend_from_slice(&1.0_f32.to_be_bytes());
    expected_direct.extend_from_slice(&0.5_f32.to_be_bytes());
    expected_direct.extend_from_slice(&42_i64.to_be_bytes());
    assert_eq!(direct_payload, expected_direct);
}
