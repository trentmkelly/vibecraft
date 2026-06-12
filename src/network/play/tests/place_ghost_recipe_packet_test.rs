use super::*;

const CLIENTBOUND_PLACE_GHOST_RECIPE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlaceGhostRecipePacket.java"
);
const BYTE_BUF_CODECS_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/network/codec/ByteBufCodecs.java");
const FRIENDLY_BYTE_BUF_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/network/FriendlyByteBuf.java");

#[test]
fn clientbound_place_ghost_recipe_packet_matches_java_codec() {
    for sentinel in [
        "ByteBufCodecs.CONTAINER_ID",
        "ClientboundPlaceGhostRecipePacket::containerId",
        "RecipeDisplay.STREAM_CODEC",
        "ClientboundPlaceGhostRecipePacket::recipeDisplay",
        "return GamePacketTypes.CLIENTBOUND_PLACE_GHOST_RECIPE;",
        "listener.handlePlaceRecipe(this);",
    ] {
        assert!(
            CLIENTBOUND_PLACE_GHOST_RECIPE_JAVA.contains(sentinel),
            "missing ClientboundPlaceGhostRecipePacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "StreamCodec<ByteBuf, Integer> CONTAINER_ID",
        "FriendlyByteBuf.writeContainerId(output, value);",
    ] {
        assert!(
            BYTE_BUF_CODECS_JAVA.contains(sentinel),
            "missing ByteBufCodecs container-id sentinel {sentinel}"
        );
    }
    for sentinel in [
        "public static int readContainerId(final ByteBuf input)",
        "return VarInt.read(input);",
        "public static void writeContainerId(final ByteBuf output, final int id)",
        "VarInt.write(output, id);",
    ] {
        assert!(
            FRIENDLY_BYTE_BUF_JAVA.contains(sentinel),
            "missing FriendlyByteBuf container-id sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID, 63);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID),
        Some("place_ghost_recipe")
    );

    let packet = ClientboundPlaceGhostRecipePacket {
        container_id: 5,
        recipe_display: RecipeDisplayData::Stonecutter {
            ingredient: SlotDisplayData::Item { item_id: 5 },
            result: SlotDisplayData::Item { item_id: 6 },
            crafting_station: SlotDisplayData::Empty,
        },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(payload, vec![5, 3, 4, 5, 4, 6, 0]);
}
