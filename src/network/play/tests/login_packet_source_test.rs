use super::*;
use crate::network::codec::write_identifier;
use crate::network::varint::write_var_i32;

const CLIENTBOUND_LOGIN_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLoginPacket.java");
const COMMON_PLAYER_SPAWN_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/CommonPlayerSpawnInfo.java");
const PLAYER_LIST_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/players/PlayerList.java");
const MINECRAFT_SERVER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/MinecraftServer.java");

#[test]
fn clientbound_login_packet_matches_java_record_codec() {
    assert_java_contains(
        CLIENTBOUND_LOGIN_JAVA,
        &[
            "Set<ResourceKey<Level>> levels",
            "input.readInt()",
            "input.readBoolean()",
            "input.readCollection(Sets::newHashSetWithExpectedSize, buf -> buf.readResourceKey(Registries.DIMENSION))",
            "input.readVarInt()",
            "new CommonPlayerSpawnInfo(input)",
            "output.writeInt(this.playerId);",
            "output.writeBoolean(this.hardcore);",
            "output.writeCollection(this.levels, FriendlyByteBuf::writeResourceKey);",
            "this.commonPlayerSpawnInfo.write(output);",
            "output.writeBoolean(this.enforcesSecureChat);",
            "return GamePacketTypes.CLIENTBOUND_LOGIN;",
            "listener.handleLogin(this);",
        ],
        "ClientboundLoginPacket",
    );
    assert_java_contains(
        COMMON_PLAYER_SPAWN_INFO_JAVA,
        &[
            "DimensionType.STREAM_CODEC.decode(input)",
            "input.readResourceKey(Registries.DIMENSION)",
            "GameType.byNullableId(input.readByte())",
            "input.readOptional(FriendlyByteBuf::readGlobalPos)",
            "DimensionType.STREAM_CODEC.encode(output, this.dimensionType);",
            "output.writeResourceKey(this.dimension);",
            "output.writeByte(GameType.getNullableId(this.previousGameType));",
            "output.writeOptional(this.lastDeathLocation, FriendlyByteBuf::writeGlobalPos);",
        ],
        "CommonPlayerSpawnInfo",
    );
    assert_java_contains(
        PLAYER_LIST_JAVA,
        &[
            "new ClientboundLoginPacket(",
            "this.server.levelKeys()",
            "player.createCommonSpawnInfo(level)",
            "this.server.enforceSecureProfile()",
        ],
        "PlayerList",
    );
    assert_java_contains(
        MINECRAFT_SERVER_JAVA,
        &[
            "public Set<ResourceKey<Level>> levelKeys()",
            "return this.levels.keySet();",
        ],
        "MinecraftServer",
    );
    assert_eq!(CLIENTBOUND_LOGIN_PACKET_ID, 49);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LOGIN_PACKET_ID),
        Some("login")
    );

    let mut payload = Vec::new();
    login_packet().write(&mut payload).unwrap();
    assert_eq!(payload, expected_login_payload());
}

fn login_packet() -> ClientboundLoginPacket {
    ClientboundLoginPacket {
        player_id: 42,
        hardcore: true,
        levels: BTreeSet::from([
            Identifier::parse("minecraft:the_nether").unwrap(),
            Identifier::parse("minecraft:overworld").unwrap(),
        ]),
        max_players: 20,
        chunk_radius: 10,
        simulation_distance: 8,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo {
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
        },
        enforces_secure_chat: true,
    }
}

fn expected_login_payload() -> Vec<u8> {
    let mut expected = Vec::new();
    expected.extend_from_slice(&42_i32.to_be_bytes());
    expected.push(1);
    write_var_i32(&mut expected, 2).unwrap();
    write_identifier(
        &mut expected,
        &Identifier::parse("minecraft:overworld").unwrap(),
    )
    .unwrap();
    write_identifier(
        &mut expected,
        &Identifier::parse("minecraft:the_nether").unwrap(),
    )
    .unwrap();
    write_var_i32(&mut expected, 20).unwrap();
    write_var_i32(&mut expected, 10).unwrap();
    write_var_i32(&mut expected, 8).unwrap();
    expected.push(0);
    expected.push(1);
    expected.push(0);
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
    expected.push(1);
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
