use super::*;

#[test]
fn clientbound_entity_position_sync_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID, 35);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID),
        Some("entity_position_sync")
    );

    let packet = ClientboundEntityPositionSyncPacket {
        id: 300,
        position: Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        movement: Vec3 {
            x: 0.125,
            y: -0.25,
            z: 0.5,
        },
        y_rot: 90.0,
        x_rot: -30.0,
        on_ground: false,
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let mut expected = vec![0xac, 0x02];
    expected.extend_from_slice(&1.25_f64.to_be_bytes());
    expected.extend_from_slice(&64.0_f64.to_be_bytes());
    expected.extend_from_slice(&(-2.5_f64).to_be_bytes());
    expected.extend_from_slice(&0.125_f64.to_be_bytes());
    expected.extend_from_slice(&(-0.25_f64).to_be_bytes());
    expected.extend_from_slice(&0.5_f64.to_be_bytes());
    expected.extend_from_slice(&90.0_f32.to_be_bytes());
    expected.extend_from_slice(&(-30.0_f32).to_be_bytes());
    expected.push(0);

    assert_eq!(payload, expected);
}
