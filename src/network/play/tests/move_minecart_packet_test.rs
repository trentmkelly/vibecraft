use super::*;
use crate::network::play::chunk_move_minecart::{
    ClientboundMoveMinecartPacket, MinecartLerpStep,
};

const CLIENTBOUND_MOVE_MINECART_JAVA: &str = vibecraft_java_source!(
    "/net/minecraft/network/protocol/game/ClientboundMoveMinecartPacket.java"
);
const NEW_MINECART_BEHAVIOR_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/entity/vehicle/minecart/NewMinecartBehavior.java");
const BYTE_BUF_CODECS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/codec/ByteBufCodecs.java");
const MTH_JAVA: &str = vibecraft_java_source!("/net/minecraft/util/Mth.java");

#[test]
fn clientbound_move_minecart_packet_matches_java_codec_shape() {
    for sentinel in [
        "public record ClientboundMoveMinecartPacket(int entityId, List<NewMinecartBehavior.MinecartStep> lerpSteps)",
        "ByteBufCodecs.VAR_INT",
        "ClientboundMoveMinecartPacket::entityId",
        "NewMinecartBehavior.MinecartStep.STREAM_CODEC.apply(ByteBufCodecs.list())",
        "ClientboundMoveMinecartPacket::lerpSteps",
        "return GamePacketTypes.CLIENTBOUND_MOVE_MINECART_ALONG_TRACK;",
        "listener.handleMinecartAlongTrack(this);",
    ] {
        assert!(
            CLIENTBOUND_MOVE_MINECART_JAVA.contains(sentinel),
            "missing ClientboundMoveMinecartPacket sentinel {sentinel}"
        );
    }

    for sentinel in [
        "public record MinecartStep(Vec3 position, Vec3 movement, float yRot, float xRot, float weight)",
        "Vec3.STREAM_CODEC",
        "ByteBufCodecs.ROTATION_BYTE",
        "ByteBufCodecs.FLOAT",
        "public static final NewMinecartBehavior.MinecartStep ZERO",
    ] {
        assert!(
            NEW_MINECART_BEHAVIOR_JAVA.contains(sentinel),
            "missing MinecartStep sentinel {sentinel}"
        );
    }

    assert!(
        BYTE_BUF_CODECS_JAVA
            .contains("ROTATION_BYTE = BYTE.map(Mth::unpackDegrees, Mth::packDegrees)")
    );
    assert!(MTH_JAVA.contains("public static byte packDegrees(final float angle)"));
    assert!(MTH_JAVA.contains("public static float unpackDegrees(final byte rot)"));

    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_MOVE_MINECART_PACKET_ID, 55);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_MINECART_PACKET_ID),
        Some("move_minecart_along_track")
    );
}

#[test]
fn clientbound_move_minecart_packet_reads_and_writes_java_wire_shape() {
    let packet = ClientboundMoveMinecartPacket {
        entity_id: 300,
        lerp_steps: vec![
            MinecartLerpStep {
                position: Vec3 {
                    x: 1.25,
                    y: -2.5,
                    z: 3.75,
                },
                movement: Vec3 {
                    x: 0.125,
                    y: 0.25,
                    z: 0.5,
                },
                y_rot: 90.0,
                x_rot: -45.0,
                weight: 1.5,
            },
            MinecartLerpStep {
                position: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                movement: Vec3 {
                    x: -1.0,
                    y: 2.0,
                    z: -3.0,
                },
                y_rot: -180.0,
                x_rot: 45.0,
                weight: 0.25,
            },
        ],
    };

    let mut expected = vec![0xac, 0x02, 0x02];
    push_vec3(&mut expected, 1.25, -2.5, 3.75);
    push_vec3(&mut expected, 0.125, 0.25, 0.5);
    expected.extend([0x40, 0xe0]);
    expected.extend_from_slice(&1.5_f32.to_be_bytes());
    push_vec3(&mut expected, 0.0, 0.0, 0.0);
    push_vec3(&mut expected, -1.0, 2.0, -3.0);
    expected.extend([0x80, 0x20]);
    expected.extend_from_slice(&0.25_f32.to_be_bytes());

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, expected);
    assert_eq!(
        ClientboundMoveMinecartPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_move_minecart_packet_rejects_malformed_payloads() {
    assert!(ClientboundMoveMinecartPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundMoveMinecartPacket::read(&mut cursor(vec![1, 1, 0])).is_err());

    let mut payload = Vec::new();
    ClientboundMoveMinecartPacket {
        entity_id: 1,
        lerp_steps: Vec::new(),
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundMoveMinecartPacket::read(&mut cursor(payload)).is_err());
}

fn push_vec3(bytes: &mut Vec<u8>, x: f64, y: f64, z: f64) {
    bytes.extend_from_slice(&x.to_be_bytes());
    bytes.extend_from_slice(&y.to_be_bytes());
    bytes.extend_from_slice(&z.to_be_bytes());
}
