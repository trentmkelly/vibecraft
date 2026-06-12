use super::*;

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

fn text_component_bytes(text: &[u8]) -> Vec<u8> {
    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        vec![0, text.len() as u8],
        text.to_vec(),
        vec![0],
    ]
    .concat()
}

fn boss_prefix(event_id: Uuid, operation: i32) -> Vec<u8> {
    let mut payload = Vec::new();
    write_uuid(&mut payload, event_id).unwrap();
    write_var_i32(&mut payload, operation).unwrap();
    payload
}

fn assert_clientbound_boss_event_java_source() {
    const CLIENTBOUND_BOSS_EVENT_JAVA: &str = include_str!(
        "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBossEventPacket.java"
    );
    for sentinel in [
        "public class ClientboundBossEventPacket implements Packet<ClientGamePacketListener>",
        "private static final int FLAG_DARKEN = 1;",
        "private static final int FLAG_MUSIC = 2;",
        "private static final int FLAG_FOG = 4;",
        "this.id = input.readUUID();",
        "input.readEnum(ClientboundBossEventPacket.OperationType.class)",
        "output.writeUUID(this.id);",
        "output.writeEnum(this.operation.getType());",
        "this.operation.write(output);",
        "GamePacketTypes.CLIENTBOUND_BOSS_EVENT",
        "listener.handleBossUpdate(this);",
        "ComponentSerialization.TRUSTED_STREAM_CODEC.encode(output, this.name);",
        "output.writeFloat(this.progress);",
        "output.writeEnum(this.color);",
        "output.writeEnum(this.overlay);",
        "output.writeByte(ClientboundBossEventPacket.encodeProperties(this.darkenScreen, this.playMusic, this.createWorldFog));",
        "ADD(ClientboundBossEventPacket.AddOperation::new)",
        "REMOVE(input -> ClientboundBossEventPacket.REMOVE_OPERATION)",
        "UPDATE_PROGRESS(ClientboundBossEventPacket.UpdateProgressOperation::new)",
        "UPDATE_NAME(ClientboundBossEventPacket.UpdateNameOperation::new)",
        "UPDATE_STYLE(ClientboundBossEventPacket.UpdateStyleOperation::new)",
        "UPDATE_PROPERTIES(ClientboundBossEventPacket.UpdatePropertiesOperation::new)",
    ] {
        assert!(
            CLIENTBOUND_BOSS_EVENT_JAVA.contains(sentinel),
            "missing ClientboundBossEventPacket sentinel {sentinel}"
        );
    }
}

#[test]
fn clientbound_boss_event_packet_matches_java_operation_codecs() {
    assert_clientbound_boss_event_java_source();
    assert_eq!(CLIENTBOUND_BOSS_EVENT_PACKET_ID, 9);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_BOSS_EVENT_PACKET_ID),
        Some("boss_event")
    );

    let event_id = Uuid([
        0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43,
        0x21,
    ]);

    let mut add = Vec::new();
    ClientboundBossEventPacket {
        event_id,
        operation: BossEventOperation::Add {
            name: text_component("Boss"),
            progress: 0.75,
            color: BossBarColor::Purple,
            overlay: BossBarOverlay::Notched10,
            flags: BossEventFlags {
                darken_screen: true,
                play_music: false,
                create_world_fog: true,
            },
        },
    }
    .write(&mut add)
    .unwrap();
    assert_eq!(
        add,
        [
            boss_prefix(event_id, 0),
            text_component_bytes(b"Boss"),
            0.75_f32.to_be_bytes().to_vec(),
            vec![5, 2, 5],
        ]
        .concat()
    );

    let mut remove = Vec::new();
    ClientboundBossEventPacket {
        event_id,
        operation: BossEventOperation::Remove,
    }
    .write(&mut remove)
    .unwrap();
    assert_eq!(remove, boss_prefix(event_id, 1));

    let mut progress = Vec::new();
    ClientboundBossEventPacket {
        event_id,
        operation: BossEventOperation::UpdateProgress { progress: 0.25 },
    }
    .write(&mut progress)
    .unwrap();
    assert_eq!(
        progress,
        [boss_prefix(event_id, 2), 0.25_f32.to_be_bytes().to_vec()].concat()
    );

    let mut name = Vec::new();
    ClientboundBossEventPacket {
        event_id,
        operation: BossEventOperation::UpdateName {
            name: text_component("Renamed"),
        },
    }
    .write(&mut name)
    .unwrap();
    assert_eq!(
        name,
        [boss_prefix(event_id, 3), text_component_bytes(b"Renamed")].concat()
    );

    let mut style = Vec::new();
    ClientboundBossEventPacket {
        event_id,
        operation: BossEventOperation::UpdateStyle {
            color: BossBarColor::Red,
            overlay: BossBarOverlay::Notched20,
        },
    }
    .write(&mut style)
    .unwrap();
    assert_eq!(style, [boss_prefix(event_id, 4), vec![2, 4]].concat());

    let mut properties = Vec::new();
    ClientboundBossEventPacket {
        event_id,
        operation: BossEventOperation::UpdateProperties {
            flags: BossEventFlags {
                darken_screen: false,
                play_music: true,
                create_world_fog: true,
            },
        },
    }
    .write(&mut properties)
    .unwrap();
    assert_eq!(properties, [boss_prefix(event_id, 5), vec![6]].concat());
}
