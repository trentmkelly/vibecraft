use super::*;
use super::super::chunk_types_c::{
    ChunkBiomeData, ClientboundChunksBiomesPacket, CLIENTBOUND_CHUNKS_BIOMES_MAX_BYTES,
};

const CLIENTBOUND_CHUNKS_BIOMES_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundChunksBiomesPacket.java");

#[test]
fn clientbound_chunks_biomes_packet_matches_java_codec() {
    for sentinel in [
        "public record ClientboundChunksBiomesPacket(List<ClientboundChunksBiomesPacket.ChunkBiomeData> chunkBiomeData)",
        "private static final int TWO_MEGABYTES = 2097152;",
        "this(input.readList(ClientboundChunksBiomesPacket.ChunkBiomeData::new));",
        "output.writeCollection(this.chunkBiomeData, (o, c) -> c.write(o));",
        "return GamePacketTypes.CLIENTBOUND_CHUNKS_BIOMES;",
        "listener.handleChunksBiomes(this);",
        "public record ChunkBiomeData(ChunkPos pos, byte[] buffer)",
        "this(input.readChunkPos(), input.readByteArray(2097152));",
        "output.writeChunkPos(this.pos);",
        "output.writeByteArray(this.buffer);",
    ] {
        assert!(
            CLIENTBOUND_CHUNKS_BIOMES_JAVA.contains(sentinel),
            "missing ClientboundChunksBiomesPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_CHUNKS_BIOMES_PACKET_ID, 13);
    assert_eq!(CLIENTBOUND_CHUNKS_BIOMES_MAX_BYTES, 2_097_152);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CHUNKS_BIOMES_PACKET_ID),
        Some("chunks_biomes")
    );

    let packet = ClientboundChunksBiomesPacket {
        chunk_biome_data: vec![
            ChunkBiomeData {
                pos: ChunkPos { x: 4, z: -2 },
                buffer: vec![0x01, 0x02, 0x03],
            },
            ChunkBiomeData {
                pos: ChunkPos { x: -1, z: 7 },
                buffer: Vec::new(),
            },
        ],
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            2, // collection length
            0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x00, 0x04, // ChunkPos(4, -2)
            3, 0x01, 0x02, 0x03, // byte array
            0x00, 0x00, 0x00, 0x07, 0xff, 0xff, 0xff, 0xff, // ChunkPos(-1, 7)
            0, // empty byte array
        ]
    );
    assert_eq!(
        ClientboundChunksBiomesPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_chunks_biomes_packet_rejects_malformed_payloads() {
    assert!(ClientboundChunksBiomesPacket::read(&mut cursor(Vec::new())).is_err());

    let oversized_len = CLIENTBOUND_CHUNKS_BIOMES_MAX_BYTES + 1;
    let mut oversized = Vec::new();
    write_var_i32(&mut oversized, 1).unwrap();
    write_i64(&mut oversized, 0).unwrap();
    write_var_i32(&mut oversized, oversized_len as i32).unwrap();
    assert!(ClientboundChunksBiomesPacket::read(&mut cursor(oversized)).is_err());

    let mut trailing = Vec::new();
    ClientboundChunksBiomesPacket {
        chunk_biome_data: vec![ChunkBiomeData {
            pos: ChunkPos { x: 0, z: 0 },
            buffer: vec![0],
        }],
    }
    .write(&mut trailing)
    .unwrap();
    trailing.push(0);
    assert!(ClientboundChunksBiomesPacket::read(&mut cursor(trailing)).is_err());
}
