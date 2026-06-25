use super::*;

const CLIENTBOUND_LEVEL_CHUNK_PACKET_DATA_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLevelChunkPacketData.java");
const CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundLevelChunkWithLightPacket.java");

#[test]
fn clientbound_level_chunk_with_light_packet_matches_java_codecs() {
    assert_java_contains(
        CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT_JAVA,
        &[
            "this.x = input.readInt();",
            "this.z = input.readInt();",
            "this.chunkData = new ClientboundLevelChunkPacketData(input, this.x, this.z);",
            "this.lightData = new ClientboundLightUpdatePacketData(input, this.x, this.z);",
            "output.writeInt(this.x);",
            "output.writeInt(this.z);",
            "this.chunkData.write(output);",
            "this.lightData.write(output);",
            "return GamePacketTypes.CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT;",
            "listener.handleLevelChunkWithLight(this);",
        ],
        "ClientboundLevelChunkWithLightPacket",
    );
    assert_java_contains(
        CLIENTBOUND_LEVEL_CHUNK_PACKET_DATA_JAVA,
        &[
            "private static final StreamCodec<ByteBuf, Map<Heightmap.Types, long[]>> HEIGHTMAPS_STREAM_CODEC = ByteBufCodecs.map",
            "private static final int TWO_MEGABYTES = 2097152;",
            ".filter(entryx -> ((Heightmap.Types)entryx.getKey()).sendToClient())",
            "this.buffer = new byte[calculateChunkSize(levelChunk)];",
            "extractChunkData(new FriendlyByteBuf(this.getWriteBuffer()), levelChunk);",
            "if (size > 2097152)",
            "HEIGHTMAPS_STREAM_CODEC.encode(output, this.heightmaps);",
            "output.writeVarInt(this.buffer.length);",
            "output.writeBytes(this.buffer);",
            "ClientboundLevelChunkPacketData.BlockEntityInfo.LIST_STREAM_CODEC.encode(output, this.blockEntitiesData);",
            "for (LevelChunkSection section : chunk.getSections())",
            "section.write(buffer);",
            "if (buffer.writerIndex() != buffer.capacity())",
            "this.packedXZ = input.readByte();",
            "this.y = input.readShort();",
            "ByteBufCodecs.registry(Registries.BLOCK_ENTITY_TYPE).encode(output, this.type);",
            "output.writeNbt(this.tag);",
            "tag.isEmpty() ? null : tag",
        ],
        "ClientboundLevelChunkPacketData",
    );

    assert_eq!(CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID, 45);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID),
        Some("level_chunk_with_light")
    );

    let mut payload = Vec::new();
    level_chunk_with_light_packet().write(&mut payload).unwrap();
    assert_eq!(payload, expected_level_chunk_with_light_payload());
}

#[test]
fn level_chunk_packet_rejects_buffers_over_java_two_megabyte_guard() {
    let err = ClientboundLevelChunkPacketData {
        heightmaps: BTreeMap::new(),
        buffer: vec![0; ClientboundLevelChunkPacketData::MAX_BUFFER_SIZE + 1],
        block_entity_count: 0,
        block_entities: Vec::new(),
    }
    .write(&mut Vec::new())
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}

fn level_chunk_with_light_packet() -> ClientboundLevelChunkWithLightPacket {
    let mut heightmaps = BTreeMap::new();
    heightmaps.insert(
        "WORLD_SURFACE".to_string(),
        vec![0x0102_0304_0506_0708_i64],
    );
    ClientboundLevelChunkWithLightPacket {
        pos: ChunkPos { x: 1, z: -1 },
        chunk_data: Some(ClientboundLevelChunkPacketData {
            heightmaps,
            buffer: vec![0xaa, 0xbb],
            block_entity_count: 2,
            block_entities: vec![
                LevelChunkBlockEntityInfo {
                    packed_xz: 0x5a,
                    y: 64,
                    block_entity_type_id: 1,
                    tag: Some(Tag::Compound(vec![("note".to_string(), Tag::Int(7))])),
                },
                LevelChunkBlockEntityInfo {
                    packed_xz: 0xc3,
                    y: -5,
                    block_entity_type_id: 0,
                    tag: None,
                },
            ],
        }),
        light_data: Some(ClientboundLightUpdatePacketData {
            sky_y_mask: Vec::new(),
            block_y_mask: Vec::new(),
            empty_sky_y_mask: Vec::new(),
            empty_block_y_mask: Vec::new(),
            sky_updates: Vec::new(),
            block_updates: Vec::new(),
        }),
    }
}

fn expected_level_chunk_with_light_payload() -> Vec<u8> {
    vec![
        0x00, 0x00, 0x00, 0x01, // x
        0xff, 0xff, 0xff, 0xff, // z
        0x01, // heightmap count
        0x01, // WORLD_SURFACE heightmap type id
        0x01, // long-array length
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x02, // chunk buffer length
        0xaa, 0xbb, // chunk buffer
        0x02, // block entity count
        0x5a, // packed XZ
        0x00, 0x40, // y
        0x01, // block entity type id
        0x0a, // non-null compound network NBT tag
        0x03, 0x00, 0x04, b'n', b'o', b't', b'e', 0x00, 0x00, 0x00, 0x07, 0x00,
        0xc3, // packed XZ
        0xff, 0xfb, // y
        0x00, // block entity type id
        0x00, // null NBT tag
        0x00, // skyYMask long-array length
        0x00, // blockYMask long-array length
        0x00, // emptySkyYMask long-array length
        0x00, // emptyBlockYMask long-array length
        0x00, // skyUpdates list length
        0x00, // blockUpdates list length
    ]
}

fn assert_java_contains(source: &str, sentinels: &[&str], class_name: &str) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing {class_name} sentinel {sentinel}"
        );
    }
}
