use super::*;

const SERVERBOUND_SET_BEACON_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSetBeaconPacket.java");
const SERVERBOUND_SET_COMMAND_BLOCK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSetCommandBlockPacket.java");
const SERVERBOUND_SET_COMMAND_MINECART_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSetCommandMinecartPacket.java");
const SERVERBOUND_SET_STRUCTURE_BLOCK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSetStructureBlockPacket.java");
const SERVERBOUND_SIGN_UPDATE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSignUpdatePacket.java");

#[test]
fn serverbound_block_edit_packets_match_java_sources() {
    assert_java_contains(
        SERVERBOUND_SET_BEACON_JAVA,
        &[
            "MobEffect.STREAM_CODEC.apply(ByteBufCodecs::optional)",
            "ServerboundSetBeaconPacket::primary",
            "ServerboundSetBeaconPacket::secondary",
            "return GamePacketTypes.SERVERBOUND_SET_BEACON;",
            "listener.handleSetBeaconPacket(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SET_COMMAND_BLOCK_JAVA,
        &[
            "this.pos = input.readBlockPos();",
            "this.command = input.readUtf();",
            "this.mode = input.readEnum(CommandBlockEntity.Mode.class);",
            "this.trackOutput = (flags & 1) != 0;",
            "this.conditional = (flags & 2) != 0;",
            "this.automatic = (flags & 4) != 0;",
            "return GamePacketTypes.SERVERBOUND_SET_COMMAND_BLOCK;",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SET_COMMAND_MINECART_JAVA,
        &[
            "this.entity = input.readVarInt();",
            "this.command = input.readUtf();",
            "this.trackOutput = input.readBoolean();",
            "return GamePacketTypes.SERVERBOUND_SET_COMMAND_MINECART;",
            "listener.handleSetCommandMinecart(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SET_STRUCTURE_BLOCK_JAVA,
        &[
            "this.pos = input.readBlockPos();",
            "this.updateType = input.readEnum(StructureBlockEntity.UpdateType.class);",
            "this.mode = input.readEnum(StructureMode.class);",
            "this.name = input.readUtf();",
            "this.offset = new BlockPos(Mth.clamp(input.readByte(), -48, 48)",
            "this.size = new Vec3i(Mth.clamp(input.readByte(), 0, 48)",
            "this.integrity = Mth.clamp(input.readFloat(), 0.0F, 1.0F);",
            "this.seed = input.readVarLong();",
            "this.ignoreEntities = (flags & 1) != 0;",
            "this.strict = (flags & 8) != 0;",
            "return GamePacketTypes.SERVERBOUND_SET_STRUCTURE_BLOCK;",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SIGN_UPDATE_JAVA,
        &[
            "private static final int MAX_STRING_LENGTH = 384;",
            "this.pos = input.readBlockPos();",
            "this.isFrontText = input.readBoolean();",
            "this.lines[i] = input.readUtf(384);",
            "return GamePacketTypes.SERVERBOUND_SIGN_UPDATE;",
            "listener.handleSignUpdate(this);",
        ],
    );

    assert_packet_registry_names();
    assert_set_beacon_codec();
    assert_set_command_block_codec();
    assert_set_command_minecart_codec();
    assert_set_structure_block_codec();
    assert_sign_update_codec();
}

fn assert_packet_registry_names() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(SERVERBOUND_SET_BEACON_PACKET_ID, 52);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_BEACON_PACKET_ID),
        Some("set_beacon")
    );
    assert_eq!(SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID, 54);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID),
        Some("set_command_block")
    );
    assert_eq!(SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID, 55);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID),
        Some("set_command_minecart")
    );
    assert_eq!(SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID, 59);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID),
        Some("set_structure_block")
    );
    assert_eq!(SERVERBOUND_SIGN_UPDATE_PACKET_ID, 61);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SIGN_UPDATE_PACKET_ID),
        Some("sign_update")
    );
}

fn assert_set_beacon_codec() {
    let packet = ServerboundSetBeaconPacket {
        primary_effect_id: Some(1),
        secondary_effect_id: Some(39),
    };
    let payload = round_trip(&packet, ServerboundSetBeaconPacket::write);
    assert_eq!(payload, vec![1, 1, 1, 39]);
    assert_eq!(
        ServerboundSetBeaconPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundSetBeaconPacket::read);

    let none = ServerboundSetBeaconPacket {
        primary_effect_id: None,
        secondary_effect_id: None,
    };
    let payload = round_trip(&none, ServerboundSetBeaconPacket::write);
    assert_eq!(payload, vec![0, 0]);
    assert_eq!(ServerboundSetBeaconPacket::read(&mut cursor(payload)).unwrap(), none);
}

fn assert_set_command_block_codec() {
    let packet = ServerboundSetCommandBlockPacket {
        x: -12,
        y: 64,
        z: 34,
        command: "say hi".to_string(),
        mode: CommandBlockMode::Redstone,
        track_output: true,
        conditional: false,
        automatic: true,
    };
    let payload = round_trip(&packet, ServerboundSetCommandBlockPacket::write);
    assert_eq!(&payload[8..], &[6, b's', b'a', b'y', b' ', b'h', b'i', 2, 5]);
    assert_eq!(
        ServerboundSetCommandBlockPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundSetCommandBlockPacket::read);
}

fn assert_set_command_minecart_codec() {
    let packet = ServerboundSetCommandMinecartPacket {
        entity_id: 128,
        command: "say hi".to_string(),
        track_output: true,
    };
    let payload = round_trip(&packet, ServerboundSetCommandMinecartPacket::write);
    assert_eq!(payload, vec![0x80, 0x01, 6, b's', b'a', b'y', b' ', b'h', b'i', 1]);
    assert_eq!(
        ServerboundSetCommandMinecartPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundSetCommandMinecartPacket::read);
}

fn assert_set_structure_block_codec() {
    let packet = ServerboundSetStructureBlockPacket {
        x: -12,
        y: 64,
        z: 34,
        update_type: StructureBlockUpdateType::LoadArea,
        mode: StructureBlockMode::Load,
        name: "demo:house".to_string(),
        offset: [-2, 3, 4],
        size: [5, 6, 7],
        mirror: StructureMirror::FrontBack,
        rotation: StructureRotation::Counterclockwise90,
        data: "metadata".to_string(),
        integrity: 0.75,
        seed: 128,
        ignore_entities: true,
        strict: true,
        show_air: false,
        show_bounding_box: true,
    };
    let payload = round_trip(&packet, ServerboundSetStructureBlockPacket::write);
    assert_eq!(
        ServerboundSetStructureBlockPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundSetStructureBlockPacket::read);

    let clamped = ServerboundSetStructureBlockPacket::read(&mut cursor(vec![
        0, 0, 0, 0, 0, 0, 0, 0, // pos
        2, 1, // update type, mode
        0, // name
        0x80, 0x7f, 0xff, // offset clamps to -48, 48, -1
        0xff, 0x7f, 10, // size clamps to 48, 48, 10
        0, 0, // mirror, rotation
        0, // data
        0x3f, 0x80, 0x00, 0x00, // integrity clamps to 1.0
        0, // seed
        0x0f, // flags
    ]))
    .unwrap();
    assert_eq!(clamped.offset, [-48, 48, -1]);
    assert_eq!(clamped.size, [0, 48, 10]);
    assert_eq!(clamped.integrity, 1.0);
    assert!(clamped.ignore_entities);
    assert!(clamped.show_air);
    assert!(clamped.show_bounding_box);
    assert!(clamped.strict);
}

fn assert_sign_update_codec() {
    let packet = ServerboundSignUpdatePacket {
        x: -12,
        y: 64,
        z: 34,
        is_front_text: false,
        lines: [
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
            "four".to_string(),
        ],
    };
    let payload = round_trip(&packet, ServerboundSignUpdatePacket::write);
    assert_eq!(payload[8], 0);
    assert_eq!(
        ServerboundSignUpdatePacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_rejects_trailing(payload, ServerboundSignUpdatePacket::read);
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
