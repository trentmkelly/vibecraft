use super::*;

const CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetDefaultSpawnPositionPacket.java");
const LEVEL_DATA_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/level/storage/LevelData.java");
const GLOBAL_POS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/core/GlobalPos.java");

#[test]
fn clientbound_set_default_spawn_position_packet_matches_java_codec() {
    for sentinel in [
        "LevelData.RespawnData.STREAM_CODEC",
        "ClientboundSetDefaultSpawnPositionPacket::respawnData",
        "return GamePacketTypes.CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION;",
        "listener.handleSetSpawn(this);",
    ] {
        assert!(
            CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_JAVA.contains(sentinel),
            "missing ClientboundSetDefaultSpawnPositionPacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "GlobalPos.STREAM_CODEC",
        "LevelData.RespawnData::globalPos",
        "ByteBufCodecs.FLOAT",
        "LevelData.RespawnData::yaw",
        "LevelData.RespawnData::pitch",
        "Mth.wrapDegrees(yaw)",
        "Mth.clamp(pitch, -90.0F, 90.0F)",
    ] {
        assert!(
            LEVEL_DATA_JAVA.contains(sentinel),
            "missing LevelData.RespawnData sentinel {sentinel}"
        );
    }
    for sentinel in [
        "ResourceKey.streamCodec(Registries.DIMENSION)",
        "GlobalPos::dimension",
        "BlockPos.STREAM_CODEC",
        "GlobalPos::pos",
    ] {
        assert!(
            GLOBAL_POS_JAVA.contains(sentinel),
            "missing GlobalPos sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID, 97);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID),
        Some("set_default_spawn_position")
    );

    let packet = ClientboundSetDefaultSpawnPositionPacket {
        respawn_data: ClientboundSetDefaultSpawnPositionData {
            dimension: Identifier::parse("minecraft:the_end").unwrap(),
            x: 1,
            y: 2,
            z: 3,
            yaw: 45.0,
            pitch: -23.5,
        },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let mut expected = Vec::new();
    expected.push("minecraft:the_end".len() as u8);
    expected.extend_from_slice(b"minecraft:the_end");
    expected.extend_from_slice(&pack_block_position(1, 2, 3).to_be_bytes());
    expected.extend_from_slice(&45.0_f32.to_be_bytes());
    expected.extend_from_slice(&(-23.5_f32).to_be_bytes());
    assert_eq!(payload, expected);
    assert_eq!(
        ClientboundSetDefaultSpawnPositionPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_set_default_spawn_position_packet_rejects_malformed_payloads() {
    assert!(ClientboundSetDefaultSpawnPositionPacket::read(&mut cursor(vec![0])).is_err());

    let mut payload = Vec::new();
    ClientboundSetDefaultSpawnPositionPacket {
        respawn_data: ClientboundSetDefaultSpawnPositionData {
            dimension: Identifier::parse("minecraft:overworld").unwrap(),
            x: 0,
            y: 64,
            z: 0,
            yaw: 0.0,
            pitch: 0.0,
        },
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundSetDefaultSpawnPositionPacket::read(&mut cursor(payload)).is_err());
}
