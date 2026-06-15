use super::*;

const CLIENTBOUND_PLAYER_COMBAT_END_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerCombatEndPacket.java");
const CLIENTBOUND_PLAYER_COMBAT_ENTER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerCombatEnterPacket.java");
const CLIENTBOUND_PLAYER_COMBAT_KILL_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerCombatKillPacket.java");

#[test]
fn clientbound_player_combat_packets_match_java_codecs() {
    assert_combat_end_java_sentinels();
    assert_combat_enter_java_sentinels();
    assert_combat_kill_java_sentinels();

    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_PLAYER_COMBAT_END_PACKET_ID, 66);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_COMBAT_END_PACKET_ID),
        Some("player_combat_end")
    );
    assert_eq!(CLIENTBOUND_PLAYER_COMBAT_ENTER_PACKET_ID, 67);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_COMBAT_ENTER_PACKET_ID),
        Some("player_combat_enter")
    );
    assert_eq!(CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID, 68);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID),
        Some("player_combat_kill")
    );

    let end = ClientboundPlayerCombatEndPacket { duration: 300 };
    let mut end_payload = Vec::new();
    end.write(&mut end_payload).unwrap();
    assert_eq!(end_payload, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundPlayerCombatEndPacket::read(&mut cursor(end_payload)).unwrap(),
        end
    );

    let enter = ClientboundPlayerCombatEnterPacket;
    let mut enter_payload = Vec::new();
    enter.write(&mut enter_payload).unwrap();
    assert!(enter_payload.is_empty());
    assert_eq!(
        ClientboundPlayerCombatEnterPacket::read(&mut cursor(enter_payload)).unwrap(),
        enter
    );

    let kill = ClientboundPlayerCombatKillPacket {
        player_id: 300,
        message: "{\"text\":\"Game over\"}".to_string(),
    };
    let mut kill_payload = Vec::new();
    kill.write(&mut kill_payload).unwrap();
    assert_eq!(
        kill_payload,
        vec![
            0xac, 0x02, // player id
            8, // network NBT string tag
            0, 9, b'G', b'a', b'm', b'e', b' ', b'o', b'v', b'e', b'r',
        ]
    );
    assert_eq!(
        ClientboundPlayerCombatKillPacket::read(&mut cursor(kill_payload)).unwrap(),
        kill
    );
}

#[test]
fn clientbound_player_combat_packets_reject_trailing_payloads() {
    assert!(ClientboundPlayerCombatEndPacket::read(&mut cursor(vec![1, 0])).is_err());
    assert!(ClientboundPlayerCombatEnterPacket::read(&mut cursor(vec![0])).is_err());

    let mut payload = Vec::new();
    ClientboundPlayerCombatKillPacket {
        player_id: 1,
        message: "{\"text\":\"x\"}".to_string(),
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundPlayerCombatKillPacket::read(&mut cursor(payload)).is_err());
}

fn assert_combat_end_java_sentinels() {
    for sentinel in [
        "ClientboundPlayerCombatEndPacket::write",
        "this.duration = input.readVarInt();",
        "output.writeVarInt(this.duration);",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_COMBAT_END;",
        "listener.handlePlayerCombatEnd(this);",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_COMBAT_END_JAVA.contains(sentinel),
            "missing ClientboundPlayerCombatEndPacket sentinel {sentinel}"
        );
    }
}

fn assert_combat_enter_java_sentinels() {
    for sentinel in [
        "public static final ClientboundPlayerCombatEnterPacket INSTANCE",
        "StreamCodec.unit(INSTANCE)",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_COMBAT_ENTER;",
        "listener.handlePlayerCombatEnter(this);",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_COMBAT_ENTER_JAVA.contains(sentinel),
            "missing ClientboundPlayerCombatEnterPacket sentinel {sentinel}"
        );
    }
}

fn assert_combat_kill_java_sentinels() {
    for sentinel in [
        "ByteBufCodecs.VAR_INT",
        "ClientboundPlayerCombatKillPacket::playerId",
        "ComponentSerialization.TRUSTED_STREAM_CODEC",
        "ClientboundPlayerCombatKillPacket::message",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_COMBAT_KILL;",
        "listener.handlePlayerCombatKill(this);",
        "public boolean isSkippable()",
        "return true;",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_COMBAT_KILL_JAVA.contains(sentinel),
            "missing ClientboundPlayerCombatKillPacket sentinel {sentinel}"
        );
    }
}
