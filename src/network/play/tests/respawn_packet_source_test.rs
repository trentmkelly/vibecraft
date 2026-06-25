use super::*;
use crate::network::codec::write_identifier;
use crate::network::varint::write_var_i32;

const CLIENTBOUND_RESPAWN_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundRespawnPacket.java");
const COMMON_PLAYER_SPAWN_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/CommonPlayerSpawnInfo.java");

#[test]
fn clientbound_respawn_packet_matches_java_common_spawn_info_codec() {
    assert_java_contains(
        CLIENTBOUND_RESPAWN_JAVA,
        &[
            "public record ClientboundRespawnPacket(CommonPlayerSpawnInfo commonPlayerSpawnInfo, byte dataToKeep)",
            "public static final byte KEEP_ATTRIBUTE_MODIFIERS = 1;",
            "public static final byte KEEP_ENTITY_DATA = 2;",
            "public static final byte KEEP_ALL_DATA = 3;",
            "this(new CommonPlayerSpawnInfo(input), input.readByte());",
            "this.commonPlayerSpawnInfo.write(output);",
            "output.writeByte(this.dataToKeep);",
            "return GamePacketTypes.CLIENTBOUND_RESPAWN;",
            "listener.handleRespawn(this);",
            "return (this.dataToKeep & mask) != 0;",
        ],
        "ClientboundRespawnPacket",
    );
    assert_java_contains(
        COMMON_PLAYER_SPAWN_INFO_JAVA,
        &[
            "DimensionType.STREAM_CODEC.decode(input)",
            "input.readResourceKey(Registries.DIMENSION)",
            "input.readLong()",
            "GameType.byId(input.readByte())",
            "GameType.byNullableId(input.readByte())",
            "input.readBoolean()",
            "input.readOptional(FriendlyByteBuf::readGlobalPos)",
            "input.readVarInt()",
            "DimensionType.STREAM_CODEC.encode(output, this.dimensionType);",
            "output.writeResourceKey(this.dimension);",
            "output.writeLong(this.seed);",
            "output.writeByte(this.gameType.getId());",
            "output.writeByte(GameType.getNullableId(this.previousGameType));",
            "output.writeBoolean(this.isDebug);",
            "output.writeBoolean(this.isFlat);",
            "output.writeOptional(this.lastDeathLocation, FriendlyByteBuf::writeGlobalPos);",
            "output.writeVarInt(this.portalCooldown);",
            "output.writeVarInt(this.seaLevel);",
        ],
        "CommonPlayerSpawnInfo",
    );
    assert_eq!(CLIENTBOUND_RESPAWN_PACKET_ID, 82);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_RESPAWN_PACKET_ID),
        Some("respawn")
    );

    let spawn_info = CommonPlayerSpawnInfo {
        dimension_type: Identifier::parse("minecraft:the_nether").unwrap(),
        dimension: Identifier::parse("minecraft:the_nether").unwrap(),
        seed: -7,
        game_mode: GameMode::Creative,
        previous_game_mode: Some(GameMode::Survival),
        is_debug: false,
        is_flat: true,
        last_death_location: Some((
            Identifier::parse("minecraft:overworld").unwrap(),
            [1, 64, -2],
        )),
        portal_cooldown: 20,
        sea_level: 32,
    };
    let mut payload = Vec::new();
    ClientboundRespawnPacket {
        spawn_info,
        data_to_keep: RespawnDataToKeep::KEEP_ALL_DATA,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, expected_respawn_payload());

    assert!(RespawnDataToKeep::KEEP_ALL_DATA.should_keep(RespawnDataToKeep::KEEP_ENTITY_DATA));
    assert!(
        RespawnDataToKeep::KEEP_ALL_DATA.should_keep(RespawnDataToKeep::KEEP_ATTRIBUTE_MODIFIERS)
    );
    assert!(!RespawnDataToKeep::KEEP_ATTRIBUTE_MODIFIERS
        .should_keep(RespawnDataToKeep::KEEP_ENTITY_DATA));
}

fn expected_respawn_payload() -> Vec<u8> {
    let mut expected = Vec::new();
    write_var_i32(&mut expected, 3).unwrap();
    write_identifier(
        &mut expected,
        &Identifier::parse("minecraft:the_nether").unwrap(),
    )
    .unwrap();
    expected.extend_from_slice(&(-7_i64).to_be_bytes());
    expected.push(1);
    expected.push(0);
    expected.push(0);
    expected.push(1);
    expected.push(1);
    write_identifier(
        &mut expected,
        &Identifier::parse("minecraft:overworld").unwrap(),
    )
    .unwrap();
    expected.extend_from_slice(&pack_block_position(1, 64, -2).to_be_bytes());
    write_var_i32(&mut expected, 20).unwrap();
    write_var_i32(&mut expected, 32).unwrap();
    expected.push(RespawnDataToKeep::KEEP_ALL_DATA.bits());
    expected
}

fn assert_java_contains(source: &str, sentinels: &[&str], class_name: &str) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing {class_name} sentinel {sentinel}"
        );
    }
}
