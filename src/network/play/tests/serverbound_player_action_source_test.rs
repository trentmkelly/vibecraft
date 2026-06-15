use super::*;

const SERVERBOUND_PADDLE_BOAT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundPaddleBoatPacket.java");
const SERVERBOUND_PLAYER_INPUT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundPlayerInputPacket.java");
const SERVERBOUND_PLAYER_COMMAND_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundPlayerCommandPacket.java");
const SERVERBOUND_PLAYER_ACTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundPlayerActionPacket.java");

#[test]
fn serverbound_player_action_family_matches_java_sources() {
    assert_java_contains(
        SERVERBOUND_PADDLE_BOAT_JAVA,
        &[
            "this.left = input.readBoolean();",
            "this.right = input.readBoolean();",
            "output.writeBoolean(this.left);",
            "output.writeBoolean(this.right);",
            "listener.handlePaddleBoat(this);",
            "return GamePacketTypes.SERVERBOUND_PADDLE_BOAT;",
        ],
    );
    assert_java_contains(
        SERVERBOUND_PLAYER_INPUT_JAVA,
        &[
            "Input.STREAM_CODEC, ServerboundPlayerInputPacket::input, ServerboundPlayerInputPacket::new",
            "return GamePacketTypes.SERVERBOUND_PLAYER_INPUT;",
            "listener.handlePlayerInput(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_PLAYER_COMMAND_JAVA,
        &[
            "this.id = input.readVarInt();",
            "this.action = input.readEnum(ServerboundPlayerCommandPacket.Action.class);",
            "this.data = input.readVarInt();",
            "output.writeVarInt(this.id);",
            "output.writeEnum(this.action);",
            "output.writeVarInt(this.data);",
            "START_RIDING_JUMP",
            "START_FALL_FLYING",
            "return GamePacketTypes.SERVERBOUND_PLAYER_COMMAND;",
            "listener.handlePlayerCommand(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_PLAYER_ACTION_JAVA,
        &[
            "this.action = input.readEnum(ServerboundPlayerActionPacket.Action.class);",
            "this.pos = input.readBlockPos();",
            "this.direction = Direction.from3DDataValue(input.readUnsignedByte());",
            "this.sequence = input.readVarInt();",
            "output.writeEnum(this.action);",
            "output.writeBlockPos(this.pos);",
            "output.writeByte(this.direction.get3DDataValue());",
            "output.writeVarInt(this.sequence);",
            "SWAP_ITEM_WITH_OFFHAND",
            "STAB;",
            "return GamePacketTypes.SERVERBOUND_PLAYER_ACTION;",
            "listener.handlePlayerAction(this);",
        ],
    );

    let registry = PlayProtocolRegistry::new();
    assert_eq!(SERVERBOUND_PADDLE_BOAT_PACKET_ID, 35);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PADDLE_BOAT_PACKET_ID),
        Some("paddle_boat")
    );
    assert_eq!(SERVERBOUND_PLAYER_ACTION_PACKET_ID, 41);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PLAYER_ACTION_PACKET_ID),
        Some("player_action")
    );
    assert_eq!(SERVERBOUND_PLAYER_COMMAND_PACKET_ID, 42);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PLAYER_COMMAND_PACKET_ID),
        Some("player_command")
    );
    assert_eq!(SERVERBOUND_PLAYER_INPUT_PACKET_ID, 43);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PLAYER_INPUT_PACKET_ID),
        Some("player_input")
    );
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }
}
