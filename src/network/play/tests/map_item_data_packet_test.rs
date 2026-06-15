use super::*;

const CLIENTBOUND_MAP_ITEM_DATA_JAVA: &str = vibecraft_java_source!(
    "/net/minecraft/network/protocol/game/ClientboundMapItemDataPacket.java"
);
const MAP_ID_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/level/saveddata/maps/MapId.java");
const MAP_DECORATION_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/level/saveddata/maps/MapDecoration.java");
const MAP_ITEM_SAVED_DATA_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/level/saveddata/maps/MapItemSavedData.java");

#[test]
fn clientbound_map_item_data_packet_matches_java_codec_shape() {
    for sentinel in [
        "public record ClientboundMapItemDataPacket(",
        "MapId.STREAM_CODEC",
        "ClientboundMapItemDataPacket::mapId",
        "ByteBufCodecs.BYTE",
        "ClientboundMapItemDataPacket::scale",
        "ByteBufCodecs.BOOL",
        "ClientboundMapItemDataPacket::locked",
        "MapDecoration.STREAM_CODEC.apply(ByteBufCodecs.list()).apply(ByteBufCodecs::optional)",
        "MapItemSavedData.MapPatch.STREAM_CODEC",
        "return GamePacketTypes.CLIENTBOUND_MAP_ITEM_DATA;",
        "listener.handleMapItemData(this);",
        "this.decorations.ifPresent(map::addClientSideDecorations);",
        "this.colorPatch.ifPresent(patch -> patch.applyToMap(map));",
    ] {
        assert!(
            CLIENTBOUND_MAP_ITEM_DATA_JAVA.contains(sentinel),
            "missing ClientboundMapItemDataPacket sentinel {sentinel}"
        );
    }

    assert!(MAP_ID_JAVA.contains("STREAM_CODEC = ByteBufCodecs.VAR_INT.map(MapId::new, MapId::id)"));
    assert!(
        MAP_DECORATION_JAVA.contains(
            "public record MapDecoration(Holder<MapDecorationType> type, byte x, byte y, byte rot, Optional<Component> name)"
        )
    );
    assert!(MAP_DECORATION_JAVA.contains("ComponentSerialization.OPTIONAL_STREAM_CODEC"));
    assert!(MAP_DECORATION_JAVA.contains("rot = (byte)(rot & 15);"));
    assert!(MAP_ITEM_SAVED_DATA_JAVA.contains("public record MapPatch(int startX, int startY, int width, int height, byte[] mapColors)"));
    assert!(MAP_ITEM_SAVED_DATA_JAVA.contains("output.writeByte(patch.width);"));
    assert!(MAP_ITEM_SAVED_DATA_JAVA.contains("FriendlyByteBuf.writeByteArray(output, patch.mapColors);"));
    assert!(MAP_ITEM_SAVED_DATA_JAVA.contains("int width = input.readUnsignedByte();"));
    assert!(MAP_ITEM_SAVED_DATA_JAVA.contains("return Optional.empty();"));

    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID, 51);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID),
        Some("map_item_data")
    );
}

#[test]
fn clientbound_map_item_data_packet_reads_and_writes_java_wire_shape() {
    let packet = ClientboundMapItemDataPacket {
        map_id: 300,
        scale: 2,
        locked: true,
        decorations: Some(vec![MapDecorationData {
            decoration_type_id: 7,
            x: -1,
            y: 2,
            rotation: 3,
            name: Some(text_component("Home")),
        }]),
        color_patch: Some(MapPatch {
            width: 2,
            height: 1,
            start_x: 4,
            start_y: 5,
            colors: vec![6, 7],
        }),
    };

    let expected = [
        vec![
            0xac, 0x02, // MapId VarInt.
            2, 1, // scale byte, locked bool.
            1, 1, // optional decorations present, list length.
            7, 0xff, 2, 3, // type id, x, y, rot.
            1, // optional name present.
            10, // network-NBT compound tag id.
            8, 0, 4, b't', b'e', b'x', b't', 0, 4, b'H', b'o', b'm', b'e',
            0, // end compound.
            2, 1, 4, 5, 2, 6, 7, // patch dimensions/start and byte-array colors.
        ],
    ]
    .concat();

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, expected);
    assert_eq!(
        ClientboundMapItemDataPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_map_item_data_packet_handles_absent_optional_fields_and_rotation_mask() {
    let mut masked_payload = Vec::new();
    ClientboundMapItemDataPacket {
        map_id: 1,
        scale: 0,
        locked: false,
        decorations: Some(vec![MapDecorationData {
            decoration_type_id: 2,
            x: 3,
            y: -4,
            rotation: 19,
            name: None,
        }]),
        color_patch: Some(MapPatch {
            width: 0,
            height: 9,
            start_x: 8,
            start_y: 7,
            colors: vec![6],
        }),
    }
    .write(&mut masked_payload)
    .unwrap();
    assert_eq!(masked_payload, vec![1, 0, 0, 1, 1, 2, 3, 0xfc, 3, 0, 0]);
    assert_eq!(
        ClientboundMapItemDataPacket::read(&mut cursor(masked_payload)).unwrap(),
        ClientboundMapItemDataPacket {
            map_id: 1,
            scale: 0,
            locked: false,
            decorations: Some(vec![MapDecorationData {
                decoration_type_id: 2,
                x: 3,
                y: -4,
                rotation: 3,
                name: None,
            }]),
            color_patch: None,
        }
    );

    let mut empty_update = Vec::new();
    ClientboundMapItemDataPacket {
        map_id: 1,
        scale: 0,
        locked: false,
        decorations: None,
        color_patch: None,
    }
    .write(&mut empty_update)
    .unwrap();
    assert_eq!(empty_update, vec![1, 0, 0, 0, 0]);
    assert_eq!(
        ClientboundMapItemDataPacket::read(&mut cursor(empty_update)).unwrap(),
        ClientboundMapItemDataPacket {
            map_id: 1,
            scale: 0,
            locked: false,
            decorations: None,
            color_patch: None,
        }
    );
}

#[test]
fn clientbound_map_item_data_packet_rejects_malformed_payloads() {
    assert!(ClientboundMapItemDataPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundMapItemDataPacket::read(&mut cursor(vec![1, 0, 0, 1, 1])).is_err());
    assert!(ClientboundMapItemDataPacket::read(&mut cursor(vec![1, 0, 0, 0, 2, 1])).is_err());

    let mut trailing = Vec::new();
    ClientboundMapItemDataPacket {
        map_id: 1,
        scale: 0,
        locked: false,
        decorations: None,
        color_patch: None,
    }
    .write(&mut trailing)
    .unwrap();
    trailing.push(0);
    assert!(ClientboundMapItemDataPacket::read(&mut cursor(trailing)).is_err());
}

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}
