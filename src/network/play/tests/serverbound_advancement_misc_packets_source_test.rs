use super::*;
use crate::network::codec::{write_identifier, write_string};
use crate::network::varint::write_var_i32;

const SERVERBOUND_SEEN_ADVANCEMENTS_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSeenAdvancementsPacket.java"
);
const SERVERBOUND_SELECT_BUNDLE_ITEM_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSelectBundleItemPacket.java"
);
const SERVERBOUND_SET_GAME_RULE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetGameRulePacket.java"
);
const SERVERBOUND_SPECTATE_ENTITY_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSpectateEntityPacket.java"
);
const SERVERBOUND_TELEPORT_TO_ENTITY_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundTeleportToEntityPacket.java"
);
const VEC_DELTA_CODEC_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/VecDeltaCodec.java"
);

#[test]
fn serverbound_advancement_misc_packets_and_vec_delta_match_java_sources() {
    assert_java_contains(
        SERVERBOUND_SEEN_ADVANCEMENTS_JAVA,
        &[
            "this.action = input.readEnum(ServerboundSeenAdvancementsPacket.Action.class);",
            "if (this.action == ServerboundSeenAdvancementsPacket.Action.OPENED_TAB)",
            "this.tab = input.readIdentifier();",
            "output.writeEnum(this.action);",
            "output.writeIdentifier(this.tab);",
            "return GamePacketTypes.SERVERBOUND_SEEN_ADVANCEMENTS;",
            "listener.handleSeenAdvancements(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SELECT_BUNDLE_ITEM_JAVA,
        &[
            "this(input.readVarInt(), input.readVarInt());",
            "this.selectedItemIndex < 0 && this.selectedItemIndex != -1",
            "throw new IllegalArgumentException(\"Invalid selectedItemIndex: \" + this.selectedItemIndex);",
            "output.writeVarInt(this.slotId);",
            "output.writeVarInt(this.selectedItemIndex);",
            "return GamePacketTypes.SERVERBOUND_BUNDLE_ITEM_SELECTED;",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SET_GAME_RULE_JAVA,
        &[
            "ServerboundSetGameRulePacket.Entry.STREAM_CODEC.apply(ByteBufCodecs.list())",
            "ResourceKey.streamCodec(Registries.GAME_RULE)",
            "ByteBufCodecs.STRING_UTF8",
            "return GamePacketTypes.SERVERBOUND_SET_GAME_RULE;",
            "listener.handleSetGameRule(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SPECTATE_ENTITY_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "ServerboundSpectateEntityPacket::entityId",
            "return GamePacketTypes.SERVERBOUND_SPECTATE_ENTITY;",
            "listener.handleSpectateEntity(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_TELEPORT_TO_ENTITY_JAVA,
        &[
            "this.uuid = input.readUUID();",
            "output.writeUUID(this.uuid);",
            "return GamePacketTypes.SERVERBOUND_TELEPORT_TO_ENTITY;",
            "listener.handleTeleportToEntityPacket(this);",
            "return level.getEntity(this.uuid);",
        ],
    );
    assert_java_contains(
        VEC_DELTA_CODEC_JAVA,
        &[
            "private static final double TRUNCATION_STEPS = 4096.0;",
            "return Math.round(input * 4096.0);",
            "return v / 4096.0;",
            "if (xa == 0L && ya == 0L && za == 0L)",
            "return encode(pos.x) - encode(this.base.x);",
            "return pos.subtract(this.base);",
        ],
    );

    assert_packet_registry_names();
    assert_seen_advancements_codec();
    assert_select_bundle_item_codec();
    assert_set_game_rule_codec();
    assert_spectate_and_teleport_codecs();
    assert_vec_delta_codec_matches_java_rounding();
}

fn assert_packet_registry_names() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(SERVERBOUND_SELECT_BUNDLE_ITEM_PACKET_ID, 3);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SELECT_BUNDLE_ITEM_PACKET_ID),
        Some("bundle_item_selected")
    );
    assert_eq!(SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID, 50);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID),
        Some("seen_advancements")
    );
    assert_eq!(SERVERBOUND_SET_GAME_RULE_PACKET_ID, 57);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_GAME_RULE_PACKET_ID),
        Some("set_game_rule")
    );
    assert_eq!(SERVERBOUND_SPECTATE_ENTITY_PACKET_ID, 62);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SPECTATE_ENTITY_PACKET_ID),
        Some("spectate_entity")
    );
    assert_eq!(SERVERBOUND_TELEPORT_TO_ENTITY_PACKET_ID, 64);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_TELEPORT_TO_ENTITY_PACKET_ID),
        Some("teleport_to_entity")
    );
}

fn assert_seen_advancements_codec() {
    let tab = Identifier::parse("minecraft:story/root").unwrap();
    let opened = ServerboundSeenAdvancementsPacket {
        action: ServerboundSeenAdvancementsAction::OpenedTab,
        tab: Some(tab.clone()),
    };
    let mut payload = Vec::new();
    opened.write(&mut payload).unwrap();
    assert_eq!(
        ServerboundSeenAdvancementsPacket::read(&mut cursor(payload)).unwrap(),
        opened
    );

    let closed = ServerboundSeenAdvancementsPacket {
        action: ServerboundSeenAdvancementsAction::ClosedScreen,
        tab: Some(tab),
    };
    let mut payload = Vec::new();
    closed.write(&mut payload).unwrap();
    assert_eq!(payload, vec![1]);
    assert_eq!(
        ServerboundSeenAdvancementsPacket::read(&mut cursor(payload)).unwrap(),
        ServerboundSeenAdvancementsPacket {
            action: ServerboundSeenAdvancementsAction::ClosedScreen,
            tab: None
        }
    );
    assert!(ServerboundSeenAdvancementsPacket::read(&mut cursor(vec![2])).is_err());
    assert!(ServerboundSeenAdvancementsPacket {
        action: ServerboundSeenAdvancementsAction::OpenedTab,
        tab: None,
    }
    .write(&mut Vec::new())
    .is_err());
}

fn assert_select_bundle_item_codec() {
    let packet = ServerboundSelectBundleItemPacket {
        slot_id: 300,
        selected_item_index: -1,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        ServerboundSelectBundleItemPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );

    let selected = ServerboundSelectBundleItemPacket {
        slot_id: 37,
        selected_item_index: 4,
    };
    let mut payload = Vec::new();
    selected.write(&mut payload).unwrap();
    assert_eq!(
        ServerboundSelectBundleItemPacket::read(&mut cursor(payload)).unwrap(),
        selected
    );

    let mut invalid = Vec::new();
    write_var_i32(&mut invalid, 37).unwrap();
    write_var_i32(&mut invalid, -2).unwrap();
    assert!(ServerboundSelectBundleItemPacket::read(&mut cursor(invalid)).is_err());
    assert!(ServerboundSelectBundleItemPacket {
        slot_id: 37,
        selected_item_index: -2,
    }
    .write(&mut Vec::new())
    .is_err());
}

fn assert_set_game_rule_codec() {
    let packet = ServerboundSetGameRulePacket {
        entries: vec![
            ServerboundSetGameRuleEntry {
                game_rule_key: Identifier::parse("minecraft:do_daylight_cycle").unwrap(),
                value: "false".to_string(),
            },
            ServerboundSetGameRuleEntry {
                game_rule_key: Identifier::parse("minecraft:random_tick_speed").unwrap(),
                value: "3".to_string(),
            },
        ],
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(ServerboundSetGameRulePacket::read(&mut cursor(payload)).unwrap(), packet);

    let mut manual = Vec::new();
    write_var_i32(&mut manual, 1).unwrap();
    write_identifier(
        &mut manual,
        &Identifier::parse("minecraft:do_mob_spawning").unwrap(),
    )
    .unwrap();
    write_string(&mut manual, "true", 32767).unwrap();
    assert_eq!(
        ServerboundSetGameRulePacket::read(&mut cursor(manual))
            .unwrap()
            .entries[0]
            .value,
        "true"
    );
}

fn assert_spectate_and_teleport_codecs() {
    let spectate = ServerboundSpectateEntityPacket { entity_id: 300 };
    let mut payload = Vec::new();
    spectate.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ServerboundSpectateEntityPacket::read(&mut cursor(payload)).unwrap(),
        spectate
    );

    let teleport = ServerboundTeleportToEntityPacket {
        uuid: Uuid([
            0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65,
            0x43, 0x21,
        ]),
    };
    let mut payload = Vec::new();
    teleport.write(&mut payload).unwrap();
    assert_eq!(payload, teleport.uuid.0);
    assert_eq!(
        ServerboundTeleportToEntityPacket::read(&mut cursor(payload)).unwrap(),
        teleport
    );
}

fn assert_vec_delta_codec_matches_java_rounding() {
    assert_eq!(VecDeltaCodec::encode(1.25), 5120);
    assert_eq!(VecDeltaCodec::encode(-1.5), -6144);
    assert_eq!(VecDeltaCodec::decode_component(5120), 1.25);

    let mut codec = VecDeltaCodec::new();
    codec.set_base(Vec3 {
        x: 10.25,
        y: -2.0,
        z: 0.125,
    });
    assert_eq!(
        codec.decode(0, 0, 0),
        Vec3 {
            x: 10.25,
            y: -2.0,
            z: 0.125
        }
    );

    let pos = Vec3 {
        x: 10.5,
        y: -1.75,
        z: -0.125,
    };
    assert_eq!(codec.encode_x(pos), 1024);
    assert_eq!(codec.encode_y(pos), 1024);
    assert_eq!(codec.encode_z(pos), -1024);
    assert_eq!(codec.decode(1024, 1024, -1024), pos);
    assert_eq!(
        codec.delta(pos),
        Vec3 {
            x: 0.25,
            y: 0.25,
            z: -0.25
        }
    );
    assert_eq!(codec.base().x, 10.25);
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java source sentinel {sentinel}"
        );
    }
}
