use super::super::*;
use crate::network::varint::read_var_i32;
use std::io::{self, Cursor, Read};

#[test]
pub fn live_spawn_chunk_packet_uses_generated_level_chunk_serialization() {
    let mut payload = Vec::new();
    let world_root = std::env::temp_dir().join(format!(
        "vibecraft-missing-world-root-{}",
        std::process::id()
    ));

    super::super::write_generated_spawn_chunk_packet(&mut payload, 0, 0, &world_root, 0)
        .expect("missing region files should fall back to generated terrain");

    let mut input = Cursor::new(payload.as_slice());
    assert_eq!(read_i32_be(&mut input).unwrap(), 0);
    assert_eq!(read_i32_be(&mut input).unwrap(), 0);
    let heightmap_count = read_var_i32(&mut input).unwrap();
    assert!(
        heightmap_count >= 3,
        "live chunk packets should serialize generated LevelChunk heightmaps, not the legacy zero-heightmap superflat packet"
    );
}

#[test]
pub fn live_spawn_chunk_packet_carries_terrain_sections_and_sky_light() {
    let mut payload = Vec::new();
    let world_root = std::env::temp_dir().join(format!(
        "vibecraft-missing-world-root-{}",
        std::process::id()
    ));

    super::super::write_generated_spawn_chunk_packet(&mut payload, 0, 0, &world_root, 0)
        .expect("missing region files should fall back to generated terrain");

    let decoded = decode_live_level_chunk_payload(&payload);
    assert_eq!(decoded.chunk_x, 0);
    assert_eq!(decoded.chunk_z, 0);
    assert!(
        decoded.non_empty_sections > 0,
        "initial chunk must contain terrain sections, not a void-only payload"
    );
    assert!(
        decoded.non_empty_blocks > 0,
        "initial chunk terrain sections must report non-air block counts"
    );
    assert!(
        decoded.sky_light_updates > 0,
        "initial chunk must carry sky-light update layers for vanilla rendering"
    );
    assert_eq!(
        decoded.trailing_bytes, 0,
        "decoder should consume exactly the Java chunk-with-light payload"
    );
}

#[test]
pub fn live_spawn_chunk_is_framed_as_level_chunk_with_light_not_light_update() {
    let chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    let mut framed = Vec::new();

    super::super::write_generated_spawn_chunk_packets_from_chunk(
        &mut framed,
        CompressionState::disabled(),
        &chunk,
    )
    .unwrap();

    let mut input = Cursor::new(framed.as_slice());
    let frame_len = read_var_i32(&mut input).unwrap();
    assert!(frame_len > 0);
    let packet_id = read_var_i32(&mut input).unwrap();
    assert_eq!(packet_id, CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID);
    assert_ne!(packet_id, crate::network::play::CLIENTBOUND_LIGHT_UPDATE_PACKET_ID);
}

#[derive(Debug)]
struct DecodedLiveLevelChunkPayload {
    chunk_x: i32,
    chunk_z: i32,
    non_empty_sections: usize,
    non_empty_blocks: i32,
    sky_light_updates: usize,
    trailing_bytes: usize,
}

fn decode_live_level_chunk_payload(payload: &[u8]) -> DecodedLiveLevelChunkPayload {
    let mut input = Cursor::new(payload);
    let chunk_x = read_i32_be(&mut input).unwrap();
    let chunk_z = read_i32_be(&mut input).unwrap();

    let heightmap_count = read_var_i32(&mut input).unwrap();
    for _ in 0..heightmap_count {
        let _heightmap_type = read_var_i32(&mut input).unwrap();
        let word_count = read_var_i32(&mut input).unwrap();
        input.set_position(input.position() + u64::try_from(word_count).unwrap() * 8);
    }

    let section_data_len = read_var_i32(&mut input).unwrap();
    let section_data_end = input.position() + u64::try_from(section_data_len).unwrap();
    let mut non_empty_sections = 0_usize;
    let mut non_empty_blocks = 0_i32;
    while input.position() < section_data_end {
        let section_non_empty = read_i16_be(&mut input).unwrap();
        let _fluid_count = read_i16_be(&mut input).unwrap();
        if section_non_empty > 0 {
            non_empty_sections += 1;
            non_empty_blocks += i32::from(section_non_empty);
        }
        skip_network_paletted_container(&mut input, 4096, 8).unwrap();
        skip_network_paletted_container(&mut input, 64, 3).unwrap();
    }
    assert_eq!(input.position(), section_data_end);

    let block_entity_count = read_var_i32(&mut input).unwrap();
    assert_eq!(
        block_entity_count, 0,
        "generated spawn terrain should not require block entity payload parsing"
    );

    let _sky_y_mask = crate::network::codec::read_bitset(&mut input).unwrap();
    let _block_y_mask = crate::network::codec::read_bitset(&mut input).unwrap();
    let _empty_sky_y_mask = crate::network::codec::read_bitset(&mut input).unwrap();
    let _empty_block_y_mask = crate::network::codec::read_bitset(&mut input).unwrap();
    let sky_light_updates = read_light_update_layers(&mut input).unwrap();
    let _block_light_updates = read_light_update_layers(&mut input).unwrap();

    DecodedLiveLevelChunkPayload {
        chunk_x,
        chunk_z,
        non_empty_sections,
        non_empty_blocks,
        sky_light_updates,
        trailing_bytes: payload.len() - input.position() as usize,
    }
}

fn skip_network_paletted_container(
    input: &mut Cursor<&[u8]>,
    entries: usize,
    max_indirect_bits: usize,
) -> io::Result<()> {
    let mut bits = [0_u8; 1];
    input.read_exact(&mut bits)?;
    let bits_per_entry = usize::from(bits[0]);
    if bits_per_entry == 0 {
        let _single_value = read_var_i32(input)?;
        return Ok(());
    } else if bits_per_entry <= max_indirect_bits {
        let palette_len = read_var_i32(input)?;
        for _ in 0..palette_len {
            let _palette_id = read_var_i32(input)?;
        }
    }

    let values_per_word = 64 / bits_per_entry;
    let word_count = entries.div_ceil(values_per_word);
    input.set_position(input.position() + u64::try_from(word_count).unwrap() * 8);
    Ok(())
}

fn read_light_update_layers(input: &mut Cursor<&[u8]>) -> io::Result<usize> {
    let layer_count = read_var_i32(input)?;
    for _ in 0..layer_count {
        let layer_len = read_var_i32(input)?;
        assert_eq!(layer_len, 2048);
        input.set_position(input.position() + u64::try_from(layer_len).unwrap());
    }
    Ok(usize::try_from(layer_count).unwrap())
}

fn read_i16_be(input: &mut Cursor<&[u8]>) -> io::Result<i16> {
    let mut bytes = [0_u8; 2];
    input.read_exact(&mut bytes)?;
    Ok(i16::from_be_bytes(bytes))
}

fn read_i32_be(input: &mut Cursor<&[u8]>) -> io::Result<i32> {
    let mut bytes = [0_u8; 4];
    input.read_exact(&mut bytes)?;
    Ok(i32::from_be_bytes(bytes))
}
