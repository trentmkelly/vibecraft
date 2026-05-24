use super::*;

pub(super) fn block_state_tag(block: &'static str) -> Tag {
    BlockStateEntry::new(block).to_nbt()
}

pub(super) fn pack_palette_indices(indices: &[u64], bits_per_entry: usize) -> Vec<i64> {
    let values_per_long = 64 / bits_per_entry;
    let mut packed = vec![0_u64; indices.len().div_ceil(values_per_long)];
    for (index, value) in indices.iter().copied().enumerate() {
        let word_index = index / values_per_long;
        let bit_index = (index - word_index * values_per_long) * bits_per_entry;
        packed[word_index] |= value << bit_index;
    }
    packed.into_iter().map(|word| word as i64).collect()
}

pub(super) fn pack_heightmap(values: [i32; 16 * 16]) -> Vec<i64> {
    const BITS_PER_ENTRY: usize = 9;
    let mut packed = vec![0_u64; (values.len() * BITS_PER_ENTRY).div_ceil(64)];
    for (index, value) in values.into_iter().enumerate() {
        let bit_offset = index * BITS_PER_ENTRY;
        let word_index = bit_offset / 64;
        let bit_index = bit_offset % 64;
        let value = value.max(0) as u64 & ((1 << BITS_PER_ENTRY) - 1);
        packed[word_index] |= value << bit_index;
        let spill = bit_index + BITS_PER_ENTRY;
        if spill > 64 {
            packed[word_index + 1] |= value >> (64 - bit_index);
        }
    }
    packed.into_iter().map(|word| word as i64).collect()
}

pub(super) fn bits_for_palette(palette_len: u64) -> usize {
    let needed = 64 - (palette_len.saturating_sub(1)).leading_zeros() as usize;
    needed.max(4)
}

pub(super) fn heightmap_opaque(heightmap: HeightmapKind, block: &str) -> bool {
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => !matches!(
            block,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ),
        HeightmapKind::OceanFloor | HeightmapKind::OceanFloorWg => block_blocks_motion(block),
        HeightmapKind::MotionBlocking => block_blocks_motion(block) || block_has_fluid(block),
        HeightmapKind::MotionBlockingNoLeaves => {
            (block_blocks_motion(block) || block_has_fluid(block)) && !block_is_leaves(block)
        }
    }
}

pub(super) fn block_blocks_motion(block: &str) -> bool {
    !matches!(
        block_state_id(block),
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:snow"
    )
}

pub(super) fn block_has_fluid(block: &str) -> bool {
    matches!(block_state_id(block), "minecraft:water" | "minecraft:lava")
        || block.contains("waterlogged=true")
}

pub(super) fn block_is_leaves(block: &str) -> bool {
    block_state_id(block).ends_with("_leaves")
}

pub(super) fn block_state_id(block: &str) -> &str {
    block.split_once('[').map_or(block, |(id, _)| id)
}
