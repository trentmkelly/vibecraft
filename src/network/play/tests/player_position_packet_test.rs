use super::*;

const CLIENTBOUND_PLAYER_POSITION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerPositionPacket.java");
const POSITION_MOVE_ROTATION_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/entity/PositionMoveRotation.java");
const RELATIVE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/entity/Relative.java");

#[test]
fn clientbound_player_position_packet_matches_java_codec() {
    assert_java_player_position_sentinels();

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
    assert_eq!(
        ClientboundPlayerPositionPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_player_position_packet_masks_unknown_relative_bits_like_java() {
    assert_java_player_position_sentinels();

    let packet = ClientboundPlayerPositionPacket {
        id: 1,
        position: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 2.0,
        },
        movement: Vec3 {
            x: 3.0,
            y: 4.0,
            z: 5.0,
        },
        y_rot: 6.0,
        x_rot: 7.0,
        relative_flags: u32::MAX,
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(&payload[payload.len() - 4..], &0x1ff_i32.to_be_bytes());

    let decoded = ClientboundPlayerPositionPacket::read(&mut cursor(payload)).unwrap();
    assert_eq!(decoded.relative_flags, 0x1ff);
}

#[test]
fn clientbound_player_position_packet_rejects_malformed_payloads() {
    assert!(ClientboundPlayerPositionPacket::read(&mut cursor(vec![1; 60])).is_err());

    let mut payload = Vec::new();
    ClientboundPlayerPositionPacket {
        id: 1,
        position: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        movement: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        y_rot: 0.0,
        x_rot: 0.0,
        relative_flags: 0,
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundPlayerPositionPacket::read(&mut cursor(payload)).is_err());
}

fn assert_java_player_position_sentinels() {
    for sentinel in [
        "ByteBufCodecs.VAR_INT",
        "ClientboundPlayerPositionPacket::id",
        "PositionMoveRotation.STREAM_CODEC",
        "ClientboundPlayerPositionPacket::change",
        "Relative.SET_STREAM_CODEC",
        "ClientboundPlayerPositionPacket::relatives",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_POSITION;",
        "listener.handleMovePlayer(this);",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_POSITION_JAVA.contains(sentinel),
            "missing ClientboundPlayerPositionPacket sentinel {sentinel}"
        );
    }

    for sentinel in [
        "Vec3.STREAM_CODEC",
        "PositionMoveRotation::position",
        "PositionMoveRotation::deltaMovement",
        "ByteBufCodecs.FLOAT",
        "PositionMoveRotation::yRot",
        "PositionMoveRotation::xRot",
    ] {
        assert!(
            POSITION_MOVE_ROTATION_JAVA.contains(sentinel),
            "missing PositionMoveRotation sentinel {sentinel}"
        );
    }

    for sentinel in [
        "X(0)",
        "Y(1)",
        "Z(2)",
        "Y_ROT(3)",
        "X_ROT(4)",
        "DELTA_X(5)",
        "DELTA_Y(6)",
        "DELTA_Z(7)",
        "ROTATE_DELTA(8)",
        "ByteBufCodecs.INT.map(Relative::unpack, Relative::pack)",
        "for (Relative argument : values())",
        "result |= argument.getMask();",
    ] {
        assert!(
            RELATIVE_JAVA.contains(sentinel),
            "missing Relative sentinel {sentinel}"
        );
    }
}
