use super::*;

const CLIENTBOUND_MOVE_VEHICLE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundMoveVehiclePacket.java"
);

#[test]
fn clientbound_move_vehicle_packet_matches_java_codec() {
    for sentinel in [
        "public record ClientboundMoveVehiclePacket(Vec3 position, float yRot, float xRot)",
        "Vec3.STREAM_CODEC",
        "ClientboundMoveVehiclePacket::position",
        "ByteBufCodecs.FLOAT",
        "ClientboundMoveVehiclePacket::yRot",
        "ClientboundMoveVehiclePacket::xRot",
        "entity.position(), entity.getYRot(), entity.getXRot()",
        "return GamePacketTypes.CLIENTBOUND_MOVE_VEHICLE;",
        "listener.handleMoveVehicle(this);",
    ] {
        assert!(
            CLIENTBOUND_MOVE_VEHICLE_JAVA.contains(sentinel),
            "missing ClientboundMoveVehiclePacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_MOVE_VEHICLE_PACKET_ID, 57);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_VEHICLE_PACKET_ID),
        Some("move_vehicle")
    );

    let packet = ClientboundMoveVehiclePacket {
        position: Vec3 {
            x: 1.25,
            y: 65.0,
            z: -2.5,
        },
        y_rot: 90.0,
        x_rot: -30.5,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            1.25_f64.to_be_bytes().to_vec(),
            65.0_f64.to_be_bytes().to_vec(),
            (-2.5_f64).to_be_bytes().to_vec(),
            90.0_f32.to_be_bytes().to_vec(),
            (-30.5_f32).to_be_bytes().to_vec(),
        ]
        .concat()
    );
    assert_eq!(payload.len(), 32);
    assert_eq!(
        ClientboundMoveVehiclePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_move_vehicle_packet_rejects_malformed_payloads() {
    let mut payload = Vec::new();
    ClientboundMoveVehiclePacket {
        position: Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        y_rot: 4.0,
        x_rot: 5.0,
    }
    .write(&mut payload)
    .unwrap();

    let mut truncated = payload.clone();
    truncated.pop();
    assert!(ClientboundMoveVehiclePacket::read(&mut cursor(truncated)).is_err());

    payload.push(0);
    assert!(ClientboundMoveVehiclePacket::read(&mut cursor(payload)).is_err());
}
