use super::*;
use crate::network::codec::write_string;

const CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS_JAVA: &str = vibecraft_java_source!(
    "/net/minecraft/network/protocol/game/ClientboundTestInstanceBlockStatus.java"
);
const VEC3I_JAVA: &str = vibecraft_java_source!("/net/minecraft/core/Vec3i.java");
const COMPONENT_SERIALIZATION_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/chat/ComponentSerialization.java");

#[test]
fn clientbound_test_instance_block_status_packet_matches_java_codec() {
    assert_java_contains(
        CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS_JAVA,
        &[
            "public record ClientboundTestInstanceBlockStatus(Component status, Optional<Vec3i> size)",
            "ComponentSerialization.STREAM_CODEC",
            "ByteBufCodecs.optional(Vec3i.STREAM_CODEC)",
            "return GamePacketTypes.CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS;",
            "listener.handleTestInstanceBlockStatus(this);",
        ],
        "ClientboundTestInstanceBlockStatus",
    );
    assert_java_contains(
        VEC3I_JAVA,
        &[
            "ByteBufCodecs.VAR_INT, Vec3i::getX",
            "ByteBufCodecs.VAR_INT, Vec3i::getY",
            "ByteBufCodecs.VAR_INT, Vec3i::getZ",
        ],
        "Vec3i",
    );
    assert_java_contains(
        COMPONENT_SERIALIZATION_JAVA,
        &["public static final StreamCodec<RegistryFriendlyByteBuf, Component> STREAM_CODEC"],
        "ComponentSerialization",
    );
    assert_eq!(CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS_PACKET_ID, 126);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS_PACKET_ID),
        Some("test_instance_block_status")
    );

    let mut present = Vec::new();
    ClientboundTestInstanceBlockStatus {
        status: ComponentJson("{\"text\":\"Ready\"}".to_string()),
        size: Some(Vec3iData {
            x: 12,
            y: -3,
            z: 64,
        }),
    }
    .write(&mut present)
    .unwrap();
    let mut expected_present = component_json_bytes("{\"text\":\"Ready\"}");
    expected_present.push(1);
    write_var_i32(&mut expected_present, 12).unwrap();
    write_var_i32(&mut expected_present, -3).unwrap();
    write_var_i32(&mut expected_present, 64).unwrap();
    assert_eq!(present, expected_present);

    let mut absent = Vec::new();
    ClientboundTestInstanceBlockStatus {
        status: ComponentJson("{\"text\":\"Idle\"}".to_string()),
        size: None,
    }
    .write(&mut absent)
    .unwrap();
    let mut expected_absent = component_json_bytes("{\"text\":\"Idle\"}");
    expected_absent.push(0);
    assert_eq!(absent, expected_absent);
}

fn component_json_bytes(json: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_string(&mut bytes, json, 262144).unwrap();
    bytes
}

fn assert_java_contains(source: &str, sentinels: &[&str], class_name: &str) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing {class_name} sentinel {sentinel}"
        );
    }
}
