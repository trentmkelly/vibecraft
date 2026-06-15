use super::*;
use crate::network::codec::{write_identifier, ComponentJson};
use crate::network::varint::write_var_i32;

const SERVERBOUND_SET_JIGSAW_BLOCK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSetJigsawBlockPacket.java");
const SERVERBOUND_SET_TEST_BLOCK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSetTestBlockPacket.java");
const SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundTestInstanceBlockActionPacket.java");
const TEST_INSTANCE_BLOCK_ENTITY_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/level/block/entity/TestInstanceBlockEntity.java");
const TEST_BLOCK_MODE_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/level/block/state/properties/TestBlockMode.java");
const JIGSAW_BLOCK_ENTITY_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/level/block/entity/JigsawBlockEntity.java");

#[test]
fn serverbound_jigsaw_and_test_packets_match_java_sources() {
    assert_java_contains(
        SERVERBOUND_SET_JIGSAW_BLOCK_JAVA,
        &[
            "this.pos = input.readBlockPos();",
            "this.name = input.readIdentifier();",
            "this.target = input.readIdentifier();",
            "this.pool = input.readIdentifier();",
            "this.finalState = input.readUtf();",
            "JigsawBlockEntity.JointType.CODEC.byName(input.readUtf(), JigsawBlockEntity.JointType.ALIGNED)",
            "this.selectionPriority = input.readVarInt();",
            "this.placementPriority = input.readVarInt();",
            "return GamePacketTypes.SERVERBOUND_SET_JIGSAW_BLOCK;",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SET_TEST_BLOCK_JAVA,
        &[
            "BlockPos.STREAM_CODEC",
            "TestBlockMode.STREAM_CODEC",
            "ByteBufCodecs.STRING_UTF8",
            "return GamePacketTypes.SERVERBOUND_SET_TEST_BLOCK;",
            "listener.handleSetTestBlock(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_JAVA,
        &[
            "ServerboundTestInstanceBlockActionPacket.Action.STREAM_CODEC",
            "TestInstanceBlockEntity.Data.STREAM_CODEC",
            "INIT(0)",
            "QUERY(1)",
            "SET(2)",
            "RESET(3)",
            "SAVE(4)",
            "EXPORT(5)",
            "RUN(6)",
            "ByIdMap.OutOfBoundsStrategy.ZERO",
            "return GamePacketTypes.SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION;",
        ],
    );
    assert_java_contains(
        TEST_INSTANCE_BLOCK_ENTITY_JAVA,
        &[
            "ByteBufCodecs.optional(ResourceKey.streamCodec(Registries.TEST_INSTANCE))",
            "Vec3i.STREAM_CODEC",
            "Rotation.STREAM_CODEC",
            "ByteBufCodecs.BOOL",
            "TestInstanceBlockEntity.Status.STREAM_CODEC",
            "ByteBufCodecs.optional(ComponentSerialization.STREAM_CODEC)",
            "CLEARED(\"cleared\", 0)",
            "RUNNING(\"running\", 1)",
            "FINISHED(\"finished\", 2)",
        ],
    );
    assert_java_contains(
        TEST_BLOCK_MODE_JAVA,
        &[
            "START(0, \"start\")",
            "LOG(1, \"log\")",
            "FAIL(2, \"fail\")",
            "ACCEPT(3, \"accept\")",
            "ByIdMap.OutOfBoundsStrategy.ZERO",
            "ByteBufCodecs.idMapper(BY_ID, mode -> mode.id)",
        ],
    );
    assert_java_contains(
        JIGSAW_BLOCK_ENTITY_JAVA,
        &[
            "ROLLABLE(\"rollable\")",
            "ALIGNED(\"aligned\")",
            "StringRepresentable.fromEnum(JigsawBlockEntity.JointType::values)",
            "return this.name;",
        ],
    );

    assert_packet_registry_names();
    assert_set_jigsaw_block_codec();
    assert_set_test_block_codec();
    assert_test_instance_block_action_codec();
}

fn assert_packet_registry_names() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(SERVERBOUND_SET_JIGSAW_BLOCK_PACKET_ID, 58);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_JIGSAW_BLOCK_PACKET_ID),
        Some("set_jigsaw_block")
    );
    assert_eq!(SERVERBOUND_SET_TEST_BLOCK_PACKET_ID, 60);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_TEST_BLOCK_PACKET_ID),
        Some("set_test_block")
    );
    assert_eq!(SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_PACKET_ID, 65);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_PACKET_ID),
        Some("test_instance_block_action")
    );
}

fn assert_set_jigsaw_block_codec() {
    let packet = ServerboundSetJigsawBlockPacket {
        x: -12,
        y: 64,
        z: 34,
        name: Identifier::parse("minecraft:start").unwrap(),
        target: Identifier::parse("minecraft:target").unwrap(),
        pool: Identifier::parse("minecraft:village/plains/houses").unwrap(),
        final_state: "minecraft:air".to_string(),
        joint: JigsawJointType::Rollable,
        selection_priority: 7,
        placement_priority: -3,
    };
    let payload = round_trip(&packet, ServerboundSetJigsawBlockPacket::write);
    assert_eq!(
        ServerboundSetJigsawBlockPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundSetJigsawBlockPacket::read);

    let mut unknown_joint = Vec::new();
    write_block_position(&mut unknown_joint, 0, 1, 2).unwrap();
    for id in ["minecraft:start", "minecraft:target", "minecraft:pool"] {
        write_identifier(&mut unknown_joint, &Identifier::parse(id).unwrap()).unwrap();
    }
    write_string(&mut unknown_joint, "minecraft:air", 32767).unwrap();
    write_string(&mut unknown_joint, "unknown", 32767).unwrap();
    write_var_i32(&mut unknown_joint, 0).unwrap();
    write_var_i32(&mut unknown_joint, 0).unwrap();
    let decoded = ServerboundSetJigsawBlockPacket::read(&mut cursor(unknown_joint)).unwrap();
    assert_eq!(decoded.joint, JigsawJointType::Aligned);
}

fn assert_set_test_block_codec() {
    let packet = ServerboundSetTestBlockPacket {
        x: 1,
        y: 2,
        z: 3,
        mode: TestBlockMode::Accept,
        message: "ok".to_string(),
    };
    let payload = round_trip(&packet, ServerboundSetTestBlockPacket::write);
    assert_eq!(
        ServerboundSetTestBlockPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundSetTestBlockPacket::read);

    let mut invalid_mode = Vec::new();
    write_block_position(&mut invalid_mode, 0, 0, 0).unwrap();
    write_var_i32(&mut invalid_mode, 99).unwrap();
    write_string(&mut invalid_mode, "fallback", 32767).unwrap();
    let decoded = ServerboundSetTestBlockPacket::read(&mut cursor(invalid_mode)).unwrap();
    assert_eq!(decoded.mode, TestBlockMode::Start);
}

fn assert_test_instance_block_action_codec() {
    let packet = ServerboundTestInstanceBlockActionPacket {
        x: 10,
        y: 64,
        z: -5,
        action: TestInstanceBlockAction::Run,
        data: TestInstanceBlockData {
            test: Some(Identifier::parse("minecraft:always_pass").unwrap()),
            size: [3, 4, 5],
            rotation: StructureRotation::Clockwise90,
            ignore_entities: true,
            status: TestInstanceBlockStatus::Finished,
            error_message: Some(ComponentJson("{\"text\":\"boom\"}".to_string())),
        },
    };
    let payload = round_trip(&packet, ServerboundTestInstanceBlockActionPacket::write);
    assert_eq!(
        ServerboundTestInstanceBlockActionPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundTestInstanceBlockActionPacket::read);

    let mut fallback = Vec::new();
    write_block_position(&mut fallback, 0, 0, 0).unwrap();
    write_var_i32(&mut fallback, 99).unwrap();
    write_bool(&mut fallback, false).unwrap();
    for component in [1, 2, 3] {
        write_var_i32(&mut fallback, component).unwrap();
    }
    write_var_i32(&mut fallback, 5).unwrap();
    write_bool(&mut fallback, false).unwrap();
    write_var_i32(&mut fallback, 99).unwrap();
    write_bool(&mut fallback, false).unwrap();
    let decoded = ServerboundTestInstanceBlockActionPacket::read(&mut cursor(fallback)).unwrap();
    assert_eq!(decoded.action, TestInstanceBlockAction::Init);
    assert_eq!(decoded.data.rotation, StructureRotation::Clockwise90);
    assert_eq!(decoded.data.status, TestInstanceBlockStatus::Cleared);
    assert_eq!(decoded.data.test, None);
    assert_eq!(decoded.data.error_message, None);
}

fn round_trip<T>(packet: &T, write: impl FnOnce(&T, &mut Vec<u8>) -> io::Result<()>) -> Vec<u8> {
    let mut payload = Vec::new();
    write(packet, &mut payload).unwrap();
    payload
}

fn assert_rejects_trailing<T>(
    mut payload: Vec<u8>,
    read: impl FnOnce(&mut std::io::Cursor<Vec<u8>>) -> io::Result<T>,
) {
    payload.push(0);
    assert!(read(&mut cursor(payload)).is_err());
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java source sentinel {sentinel}"
        );
    }
}
