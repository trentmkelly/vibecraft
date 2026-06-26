use super::*;

const CLIENTBOUND_LIGHT_UPDATE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLightUpdatePacket.java");
const CLIENTBOUND_LIGHT_UPDATE_DATA_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLightUpdatePacketData.java");
const BYTE_BUF_CODECS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/codec/ByteBufCodecs.java");
const FRIENDLY_BYTE_BUF_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/FriendlyByteBuf.java");

#[test]
fn clientbound_light_update_packet_matches_java_data_codec() {
    assert_java_contains(
        CLIENTBOUND_LIGHT_UPDATE_JAVA,
        &[
            "this.x = input.readVarInt();",
            "this.z = input.readVarInt();",
            "this.lightData = new ClientboundLightUpdatePacketData(input, this.x, this.z);",
            "output.writeVarInt(this.x);",
            "output.writeVarInt(this.z);",
            "this.lightData.write(output);",
            "return GamePacketTypes.CLIENTBOUND_LIGHT_UPDATE;",
            "listener.handleLightUpdatePacket(this);",
        ],
        "ClientboundLightUpdatePacket",
    );
    assert_java_contains(
        CLIENTBOUND_LIGHT_UPDATE_DATA_JAVA,
        &[
            "private static final StreamCodec<ByteBuf, byte[]> DATA_LAYER_STREAM_CODEC = ByteBufCodecs.byteArray(2048);",
            "this.skyYMask = input.readBitSet();",
            "this.blockYMask = input.readBitSet();",
            "this.emptySkyYMask = input.readBitSet();",
            "this.emptyBlockYMask = input.readBitSet();",
            "this.skyUpdates = input.readList(DATA_LAYER_STREAM_CODEC);",
            "this.blockUpdates = input.readList(DATA_LAYER_STREAM_CODEC);",
            "output.writeBitSet(this.skyYMask);",
            "output.writeBitSet(this.blockYMask);",
            "output.writeBitSet(this.emptySkyYMask);",
            "output.writeBitSet(this.emptyBlockYMask);",
            "output.writeCollection(this.skyUpdates, DATA_LAYER_STREAM_CODEC);",
            "output.writeCollection(this.blockUpdates, DATA_LAYER_STREAM_CODEC);",
            "if (data.isEmpty())",
            "emptyMask.set(sectionIndex);",
            "mask.set(sectionIndex);",
            "updates.add(data.copy().getData());",
        ],
        "ClientboundLightUpdatePacketData",
    );
    assert_java_contains(
        BYTE_BUF_CODECS_JAVA,
        &[
            "static StreamCodec<ByteBuf, byte[]> byteArray(final int maxSize)",
            "FriendlyByteBuf.readByteArray(input, maxSize)",
            "FriendlyByteBuf.writeByteArray(output, value);",
        ],
        "ByteBufCodecs.byteArray",
    );
    assert_java_contains(
        FRIENDLY_BYTE_BUF_JAVA,
        &[
            "return BitSet.valueOf(this.readLongArray());",
            "this.writeLongArray(bitSet.toLongArray());",
            "VarInt.write(output, bytes.length);",
            "output.writeBytes(bytes);",
        ],
        "FriendlyByteBuf bitset/byte-array helpers",
    );

    assert_eq!(CLIENTBOUND_LIGHT_UPDATE_PACKET_ID, 48);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LIGHT_UPDATE_PACKET_ID),
        Some("light_update")
    );

    let mut payload = Vec::new();
    light_update_packet().write(&mut payload).unwrap();
    assert_eq!(payload, expected_light_update_payload());
}

#[test]
fn clientbound_light_update_rejects_non_data_layer_lengths() {
    let err = ClientboundLightUpdatePacketData {
        sky_y_mask: vec![1],
        block_y_mask: Vec::new(),
        empty_sky_y_mask: Vec::new(),
        empty_block_y_mask: Vec::new(),
        sky_updates: vec![vec![0; ClientboundLightUpdatePacketData::DATA_LAYER_SIZE - 1]],
        block_updates: Vec::new(),
    }
    .write(&mut Vec::new())
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}

fn light_update_packet() -> ClientboundLightUpdatePacket {
    ClientboundLightUpdatePacket {
        pos: ChunkPos { x: 128, z: -2 },
        light_data: ClientboundLightUpdatePacketData {
            sky_y_mask: vec![0b10],
            block_y_mask: vec![0b100],
            empty_sky_y_mask: vec![0b1000],
            empty_block_y_mask: vec![0b1_0000],
            sky_updates: vec![vec![-1; ClientboundLightUpdatePacketData::DATA_LAYER_SIZE]],
            block_updates: vec![vec![1; ClientboundLightUpdatePacketData::DATA_LAYER_SIZE]],
        },
    }
}

fn expected_light_update_payload() -> Vec<u8> {
    let mut expected = vec![
        0x80, 0x01, // x = 128
        0xfe, 0xff, 0xff, 0xff, 0x0f, // z = -2
        0x01, // skyYMask long-array length
    ];
    expected.extend_from_slice(&0b10_u64.to_be_bytes());
    expected.push(0x01);
    expected.extend_from_slice(&0b100_u64.to_be_bytes());
    expected.push(0x01);
    expected.extend_from_slice(&0b1000_u64.to_be_bytes());
    expected.push(0x01);
    expected.extend_from_slice(&0b1_0000_u64.to_be_bytes());
    expected.push(0x01);
    expected.extend_from_slice(&[0x80, 0x10]);
    expected.extend(std::iter::repeat_n(
        0xff,
        ClientboundLightUpdatePacketData::DATA_LAYER_SIZE,
    ));
    expected.push(0x01);
    expected.extend_from_slice(&[0x80, 0x10]);
    expected.extend(std::iter::repeat_n(
        0x01,
        ClientboundLightUpdatePacketData::DATA_LAYER_SIZE,
    ));
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
