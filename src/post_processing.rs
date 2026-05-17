#![allow(dead_code)]

use std::collections::BTreeSet;

use crate::block_update::BlockPos;
use crate::scheduled_tick::SavedTick;
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostProcessingSections {
    min_section_y: i32,
    sections: Vec<Vec<i16>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction8 {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeData {
    indices: Vec<Vec<i32>>,
    sides: BTreeSet<Direction8>,
    neighbor_block_ticks: Vec<SavedTick>,
    neighbor_fluid_ticks: Vec<SavedTick>,
}

impl PostProcessingSections {
    pub fn new(min_section_y: i32, section_count: usize) -> Self {
        Self {
            min_section_y,
            sections: vec![Vec::new(); section_count],
        }
    }

    pub fn mark_pos(&mut self, pos: BlockPos) -> bool {
        let section_index = pos.y.div_euclid(16) - self.min_section_y;
        if section_index < 0 || section_index as usize >= self.sections.len() {
            return false;
        }
        self.sections[section_index as usize].push(pack_offset_coordinates(pos));
        true
    }

    pub fn add_packed_post_process(&mut self, section_index: usize, packed_offsets: &[i16]) {
        self.sections[section_index].extend_from_slice(packed_offsets);
    }

    pub fn to_nbt(&self) -> Tag {
        pack_offsets(&self.sections)
    }
}

impl UpgradeData {
    pub fn empty(section_count: usize) -> Self {
        Self {
            indices: vec![Vec::new(); section_count],
            sides: BTreeSet::new(),
            neighbor_block_ticks: Vec::new(),
            neighbor_fluid_ticks: Vec::new(),
        }
    }

    pub fn set_indices(&mut self, section_index: usize, indices: Vec<i32>) {
        self.indices[section_index] = indices;
    }

    pub fn add_side(&mut self, side: Direction8) {
        self.sides.insert(side);
    }

    pub fn add_neighbor_block_tick(&mut self, tick: SavedTick) {
        self.neighbor_block_ticks.push(tick);
    }

    pub fn add_neighbor_fluid_tick(&mut self, tick: SavedTick) {
        self.neighbor_fluid_ticks.push(tick);
    }

    pub fn is_empty(&self) -> bool {
        self.indices.iter().all(Vec::is_empty)
            && self.sides.is_empty()
            && self.neighbor_block_ticks.is_empty()
            && self.neighbor_fluid_ticks.is_empty()
    }

    pub fn write(&self) -> Tag {
        let mut fields = Vec::new();
        let indices = self
            .indices
            .iter()
            .enumerate()
            .filter(|(_, values)| !values.is_empty())
            .map(|(index, values)| (index.to_string(), Tag::IntArray(values.clone())))
            .collect::<Vec<_>>();
        if !indices.is_empty() {
            fields.push(("Indices".to_string(), Tag::Compound(indices)));
        }

        fields.push(("Sides".to_string(), Tag::Byte(self.side_mask() as i8)));
        if !self.neighbor_block_ticks.is_empty() {
            fields.push((
                "neighbor_block_ticks".to_string(),
                saved_ticks_to_nbt(&self.neighbor_block_ticks),
            ));
        }
        if !self.neighbor_fluid_ticks.is_empty() {
            fields.push((
                "neighbor_fluid_ticks".to_string(),
                saved_ticks_to_nbt(&self.neighbor_fluid_ticks),
            ));
        }
        Tag::Compound(fields)
    }

    fn side_mask(&self) -> u8 {
        self.sides
            .iter()
            .fold(0u8, |mask, side| mask | (1 << side.ordinal()))
    }
}

impl Direction8 {
    pub const fn ordinal(self) -> u8 {
        match self {
            Direction8::North => 0,
            Direction8::NorthEast => 1,
            Direction8::East => 2,
            Direction8::SouthEast => 3,
            Direction8::South => 4,
            Direction8::SouthWest => 5,
            Direction8::West => 6,
            Direction8::NorthWest => 7,
        }
    }
}

pub fn pack_offset_coordinates(pos: BlockPos) -> i16 {
    let dx = pos.x & 15;
    let dy = pos.y & 15;
    let dz = pos.z & 15;
    (dx | (dy << 4) | (dz << 8)) as i16
}

pub fn unpack_offset_coordinates(packed: i16, section_y: i32, chunk_pos: ChunkPos) -> BlockPos {
    let packed = packed as u16;
    BlockPos {
        x: chunk_pos.x * 16 + (packed & 15) as i32,
        y: section_y * 16 + ((packed >> 4) & 15) as i32,
        z: chunk_pos.z * 16 + ((packed >> 8) & 15) as i32,
    }
}

pub fn pack_offsets(sections: &[Vec<i16>]) -> Tag {
    Tag::List(
        sections
            .iter()
            .map(|offsets| Tag::List(offsets.iter().map(|offset| Tag::Short(*offset)).collect()))
            .collect(),
    )
}

fn saved_ticks_to_nbt(ticks: &[SavedTick]) -> Tag {
    Tag::List(
        ticks
            .iter()
            .map(|tick| {
                Tag::Compound(vec![
                    ("i".to_string(), Tag::String(tick.ty.clone())),
                    ("x".to_string(), Tag::Int(tick.pos.x)),
                    ("y".to_string(), Tag::Int(tick.pos.y)),
                    ("z".to_string(), Tag::Int(tick.pos.z)),
                    ("t".to_string(), Tag::Int(tick.delay)),
                    ("p".to_string(), Tag::Int(tick.priority.value())),
                ])
            })
            .collect(),
    )
}

pub fn upgrade_data_from_nbt(tag: &Tag, section_count: usize) -> Result<UpgradeData, String> {
    let compound = match tag {
        Tag::Compound(fields) => fields,
        other => return Err(format!("expected UpgradeData compound, got {other:?}")),
    };
    let mut upgrade = UpgradeData::empty(section_count);

    if let Some(Tag::Compound(indices)) = field(compound, "Indices") {
        for (key, value) in indices {
            let index = key
                .parse::<usize>()
                .map_err(|err| format!("invalid upgrade section index '{key}': {err}"))?;
            if index >= section_count {
                return Err(format!("upgrade section index {index} out of range"));
            }
            match value {
                Tag::IntArray(values) => upgrade.indices[index] = values.clone(),
                other => return Err(format!("upgrade indices must be int arrays, got {other:?}")),
            }
        }
    }

    let side_mask = match field(compound, "Sides") {
        Some(Tag::Byte(value)) => *value as u8,
        Some(Tag::Int(value)) => *value as u8,
        None => 0,
        Some(other) => return Err(format!("upgrade sides must be byte or int, got {other:?}")),
    };
    for side in [
        Direction8::North,
        Direction8::NorthEast,
        Direction8::East,
        Direction8::SouthEast,
        Direction8::South,
        Direction8::SouthWest,
        Direction8::West,
        Direction8::NorthWest,
    ] {
        if side_mask & (1 << side.ordinal()) != 0 {
            upgrade.add_side(side);
        }
    }

    Ok(upgrade)
}

fn field<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    fields
        .iter()
        .find_map(|(field_name, value)| (field_name == name).then_some(value))
}

#[cfg(test)]
mod tests {
    use super::{
        pack_offset_coordinates, pack_offsets, unpack_offset_coordinates, upgrade_data_from_nbt,
        Direction8, PostProcessingSections, UpgradeData,
    };
    use crate::block_update::BlockPos;
    use crate::scheduled_tick::{SavedTick, TickPriority};
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;

    #[test]
    fn post_processing_offsets_pack_local_x_y_z_nibbles_like_proto_chunk() {
        let pos = BlockPos {
            x: -17,
            y: -1,
            z: 34,
        };
        let packed = pack_offset_coordinates(pos);
        assert_eq!(packed as u16, 0x2FF);
        assert_eq!(
            unpack_offset_coordinates(packed, -1, ChunkPos { x: -2, z: 2 }),
            pos
        );
    }

    #[test]
    fn post_processing_sections_mark_by_build_section_and_pack_empty_lists() {
        let mut post = PostProcessingSections::new(-4, 8);
        assert!(!post.mark_pos(BlockPos { x: 0, y: -80, z: 0 }));
        assert!(post.mark_pos(BlockPos {
            x: 18,
            y: -63,
            z: 35
        }));
        post.add_packed_post_process(1, &[0x123]);

        match post.to_nbt() {
            Tag::List(sections) => {
                assert_eq!(sections.len(), 8);
                assert_eq!(sections[0], Tag::List(vec![Tag::Short(0x312)]));
                assert_eq!(sections[1], Tag::List(vec![Tag::Short(0x123)]));
                assert_eq!(sections[2], Tag::List(vec![]));
            }
            other => panic!("expected post-processing list, got {other:?}"),
        }
    }

    #[test]
    fn pack_offsets_preserves_section_count_and_offset_order() {
        assert_eq!(
            pack_offsets(&[vec![1, 2], vec![], vec![3]]),
            Tag::List(vec![
                Tag::List(vec![Tag::Short(1), Tag::Short(2)]),
                Tag::List(vec![]),
                Tag::List(vec![Tag::Short(3)])
            ])
        );
    }

    #[test]
    fn upgrade_data_empty_only_when_no_indices_sides_or_neighbor_ticks() {
        let mut upgrade = UpgradeData::empty(3);
        assert!(upgrade.is_empty());
        upgrade.set_indices(1, vec![4, 5]);
        assert!(!upgrade.is_empty());
    }

    #[test]
    fn upgrade_data_writes_indices_sides_and_neighbor_ticks() {
        let mut upgrade = UpgradeData::empty(4);
        upgrade.set_indices(2, vec![7, 8, 9]);
        upgrade.add_side(Direction8::NorthEast);
        upgrade.add_side(Direction8::West);
        upgrade.add_neighbor_block_tick(SavedTick {
            ty: "minecraft:air".to_string(),
            pos: BlockPos { x: 1, y: 2, z: 3 },
            delay: 5,
            priority: TickPriority::High,
        });

        match upgrade.write() {
            Tag::Compound(fields) => {
                assert!(fields.iter().any(|(name, value)| {
                    name == "Indices"
                        && *value
                            == Tag::Compound(vec![("2".to_string(), Tag::IntArray(vec![7, 8, 9]))])
                }));
                assert!(fields
                    .iter()
                    .any(|(name, value)| name == "Sides" && *value == Tag::Byte(66)));
                assert!(fields.iter().any(|(name, value)| {
                    name == "neighbor_block_ticks"
                        && matches!(value, Tag::List(values) if values.len() == 1)
                }));
            }
            other => panic!("expected upgrade compound, got {other:?}"),
        }
    }

    #[test]
    fn upgrade_data_reads_indices_and_side_mask() {
        let tag = Tag::Compound(vec![
            (
                "Indices".to_string(),
                Tag::Compound(vec![("0".to_string(), Tag::IntArray(vec![1, 2]))]),
            ),
            ("Sides".to_string(), Tag::Byte(0b1000_0001u8 as i8)),
        ]);

        let parsed = upgrade_data_from_nbt(&tag, 2).expect("upgrade data parses");
        assert!(!parsed.is_empty());
        assert_eq!(parsed.write(), tag);
    }
}
