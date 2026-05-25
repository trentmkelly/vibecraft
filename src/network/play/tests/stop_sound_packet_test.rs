use super::*;

#[test]
fn clientbound_stop_sound_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_STOP_SOUND_PACKET_ID, 119);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_STOP_SOUND_PACKET_ID),
        Some("stop_sound")
    );

    let mut without_filters = Vec::new();
    ClientboundStopSoundPacket {
        source: None,
        name: None,
    }
    .write(&mut without_filters)
    .unwrap();
    assert_eq!(without_filters, [0x00]);

    let mut source_only = Vec::new();
    ClientboundStopSoundPacket {
        source: Some(SoundSource::Voice),
        name: None,
    }
    .write(&mut source_only)
    .unwrap();
    assert_eq!(source_only, [0x01, SoundSource::Voice as u8]);

    let sound_name = Identifier::parse("minecraft:block.note_block.harp").unwrap();
    let mut name_only = Vec::new();
    ClientboundStopSoundPacket {
        source: None,
        name: Some(sound_name.clone()),
    }
    .write(&mut name_only)
    .unwrap();
    assert_eq!(
        name_only,
        [
            vec![0x02, 0x1f],
            b"minecraft:block.note_block.harp".to_vec()
        ]
        .concat()
    );

    let mut source_and_name = Vec::new();
    ClientboundStopSoundPacket {
        source: Some(SoundSource::Blocks),
        name: Some(sound_name),
    }
    .write(&mut source_and_name)
    .unwrap();
    assert_eq!(
        source_and_name,
        [
            vec![0x03, SoundSource::Blocks as u8, 0x1f],
            b"minecraft:block.note_block.harp".to_vec()
        ]
        .concat()
    );
}
