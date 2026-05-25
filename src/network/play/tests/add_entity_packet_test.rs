use super::*;

#[test]
fn clientbound_add_entity_packet_matches_java_codec_order() {
    assert_eq!(CLIENTBOUND_ADD_ENTITY_PACKET_ID, 1);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_ADD_ENTITY_PACKET_ID),
        Some("add_entity")
    );

    let mut payload = Vec::new();
    ClientboundAddEntityPacket::new(
        300,
        Uuid([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
        5,
        Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        Vec3 {
            x: 0.3,
            y: 0.1,
            z: -0.3,
        },
        (-45.0, 359.9),
        22.5,
        123,
    )
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            vec![0xac, 0x02],
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            vec![5],
            1.25_f64.to_be_bytes().to_vec(),
            64.0_f64.to_be_bytes().to_vec(),
            (-2.5_f64).to_be_bytes().to_vec(),
            vec![0x91, 0x99, 0x59, 0x99, 0x19, 0x96],
            vec![224, 255, 16, 123],
        ]
        .concat()
    );
}

#[test]
fn clientbound_add_entity_packet_encodes_large_lp_movement_continuation() {
    let mut payload = Vec::new();
    ClientboundAddEntityPacket::new(
        1,
        Uuid([0xff; 16]),
        127,
        Vec3::ZERO,
        Vec3 {
            x: 5.5,
            y: -6.25,
            z: 2.0,
        },
        (0.0, 0.0),
        0.0,
        2048,
    )
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        &payload[..18],
        &[vec![1], vec![0xff; 16], vec![127]].concat()
    );
    assert_eq!(&payload[18..42], &[0; 24]);
    assert_eq!(
        &payload[42..49],
        &[0x3f, 0x92, 0xa4, 0x90, 0x1b, 0x6f, 0x01]
    );
    assert_eq!(&payload[49..], &[0, 0, 0, 0x80, 0x10]);
}
