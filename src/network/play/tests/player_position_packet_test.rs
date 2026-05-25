use super::*;

#[test]
fn clientbound_player_position_packet_writes_relative_flags_as_fixed_int() {
    assert_eq!(CLIENTBOUND_PLAYER_POSITION_PACKET_ID, 72);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_POSITION_PACKET_ID),
        Some("player_position")
    );

    let packet = ClientboundPlayerPositionPacket {
        id: 300,
        position: Vec3 {
            x: -8.0,
            y: 70.25,
            z: 3.5,
        },
        movement: Vec3 {
            x: 0.0625,
            y: 0.0,
            z: -0.125,
        },
        y_rot: 180.0,
        x_rot: 45.0,
        relative_flags: 0b1_0010_0011,
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let mut expected = vec![0xac, 0x02];
    expected.extend_from_slice(&(-8.0_f64).to_be_bytes());
    expected.extend_from_slice(&70.25_f64.to_be_bytes());
    expected.extend_from_slice(&3.5_f64.to_be_bytes());
    expected.extend_from_slice(&0.0625_f64.to_be_bytes());
    expected.extend_from_slice(&0.0_f64.to_be_bytes());
    expected.extend_from_slice(&(-0.125_f64).to_be_bytes());
    expected.extend_from_slice(&180.0_f32.to_be_bytes());
    expected.extend_from_slice(&45.0_f32.to_be_bytes());
    expected.extend_from_slice(&0b1_0010_0011_i32.to_be_bytes());

    assert_eq!(payload, expected);
    assert_eq!(&payload[payload.len() - 4..], &[0x00, 0x00, 0x01, 0x23]);
}
