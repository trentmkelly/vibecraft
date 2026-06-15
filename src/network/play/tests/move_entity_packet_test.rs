use super::*;

const CLIENTBOUND_MOVE_ENTITY_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundMoveEntityPacket.java");

#[test]
fn clientbound_move_entity_packet_matches_java_subclass_codecs() {
    for sentinel in [
        "protected final int entityId;",
        "protected final short xa;",
        "protected final short ya;",
        "protected final short za;",
        "protected final byte yRot;",
        "protected final byte xRot;",
        "protected final boolean onGround;",
        "protected final boolean hasRot;",
        "protected final boolean hasPos;",
        "listener.handleMoveEntity(this);",
        "return GamePacketTypes.CLIENTBOUND_MOVE_ENTITY_POS;",
        "return GamePacketTypes.CLIENTBOUND_MOVE_ENTITY_POS_ROT;",
        "return GamePacketTypes.CLIENTBOUND_MOVE_ENTITY_ROT;",
    ] {
        assert!(
            CLIENTBOUND_MOVE_ENTITY_JAVA.contains(sentinel),
            "missing ClientboundMoveEntityPacket sentinel {sentinel}"
        );
    }

    assert_java_contains_in_order(&[
        "int entityId = input.readVarInt();\n         short xa = input.readShort();\n         short ya = input.readShort();\n         short za = input.readShort();\n         boolean onGround = input.readBoolean();",
        "output.writeVarInt(this.entityId);\n         output.writeShort(this.xa);\n         output.writeShort(this.ya);\n         output.writeShort(this.za);\n         output.writeBoolean(this.onGround);",
        "int entityId = input.readVarInt();\n         short xa = input.readShort();\n         short ya = input.readShort();\n         short za = input.readShort();\n         byte yRot = input.readByte();\n         byte xRot = input.readByte();\n         boolean onGround = input.readBoolean();",
        "output.writeVarInt(this.entityId);\n         output.writeShort(this.xa);\n         output.writeShort(this.ya);\n         output.writeShort(this.za);\n         output.writeByte(this.yRot);\n         output.writeByte(this.xRot);\n         output.writeBoolean(this.onGround);",
        "int entityId = input.readVarInt();\n         byte yRot = input.readByte();\n         byte xRot = input.readByte();\n         boolean onGround = input.readBoolean();",
        "output.writeVarInt(this.entityId);\n         output.writeByte(this.yRot);\n         output.writeByte(this.xRot);\n         output.writeBoolean(this.onGround);",
    ]);

    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID),
        Some("move_entity_pos")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID),
        Some("move_entity_pos_rot")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID),
        Some("move_entity_rot")
    );
}

#[test]
fn clientbound_move_entity_packet_reads_and_writes_all_java_variants() {
    let pos = ClientboundMoveEntityPacket::pos(300, [32_767, -32_768, -2], false);
    let mut payload = Vec::new();
    pos.write_pos(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![0xac, 0x02, 0x7f, 0xff, 0x80, 0x00, 0xff, 0xfe, 0x00]
    );
    assert_eq!(
        ClientboundMoveEntityPacket::read_pos(&mut cursor(payload)).unwrap(),
        pos
    );

    let pos_rot = ClientboundMoveEntityPacket::pos_rot(300, [1, -2, 3], 90.0, -45.0, true);
    let mut payload = Vec::new();
    pos_rot.write_pos_rot(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![0xac, 0x02, 0x00, 0x01, 0xff, 0xfe, 0x00, 0x03, 0x40, 0xe0, 0x01]
    );
    assert_eq!(
        ClientboundMoveEntityPacket::read_pos_rot(&mut cursor(payload)).unwrap(),
        pos_rot
    );

    let rot = ClientboundMoveEntityPacket::rot(300, 180.0, -45.0, false);
    let mut payload = Vec::new();
    rot.write_rot(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 0x80, 0xe0, 0x00]);
    assert_eq!(
        ClientboundMoveEntityPacket::read_rot(&mut cursor(payload)).unwrap(),
        rot
    );
}

#[test]
fn clientbound_move_entity_packet_rejects_malformed_payloads() {
    assert!(ClientboundMoveEntityPacket::read_pos(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundMoveEntityPacket::read_pos_rot(&mut cursor(vec![1, 0, 0])).is_err());
    assert!(ClientboundMoveEntityPacket::read_rot(&mut cursor(vec![1, 0, 0, 1, 0])).is_err());
}

fn assert_java_contains_in_order(sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            CLIENTBOUND_MOVE_ENTITY_JAVA.contains(sentinel),
            "missing ClientboundMoveEntityPacket codec sentinel {sentinel}"
        );
    }
}
