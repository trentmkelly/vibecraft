//! `ChunkPalettedStorageFix.Section`: one 16x16x16 chunk section being palettised.

use std::collections::HashSet;

use crate::datafix::dynamic::{as_byte_buffer, get, get_i32_or, remove, set};
use crate::datafix::fixes::block_state_data::{block_state_to_tag, get_state, group_state};
use crate::datafix::packed_bit_storage::{ceil_log2, PackedBitStorage};
use crate::storage::nbt::Tag;

use super::mapping_constants::{MappingConstants, StateId, FIX, VIRTUAL};

const SIZE: usize = 4096;
const FILTER_ME: &str = "%%FILTER_ME%%";

/// The name of a block state (`ChunkPalettedStorageFix.getName`).
pub fn state_name(state: StateId, constants: &MappingConstants) -> String {
    resolve(state, constants).0.clone()
}

/// A property of a block state, `""` when absent (`ChunkPalettedStorageFix.getProperty`).
pub fn state_property(state: StateId, property: &str, constants: &MappingConstants) -> String {
    resolve(state, constants)
        .1
        .iter()
        .find(|(key, _)| key == property)
        .map(|(_, value)| value.clone())
        .unwrap_or_default()
}

fn resolve(
    state: StateId,
    constants: &MappingConstants,
) -> &'static (String, Vec<(String, String)>) {
    if constants.is_constant(state) {
        // Constants live as long as the process.
        let constants: &'static MappingConstants = MappingConstants::get();
        constants.state(state)
    } else {
        group_state(state)
    }
}

/// `ChunkPalettedStorageFix.DataLayer`: a 2048 byte nibble array.
struct DataLayer(Vec<i8>);

impl DataLayer {
    fn empty() -> Self {
        Self(vec![0; 2048])
    }

    /// `new DataLayer(byte[])`; `None` where Java throws for a wrong length.
    fn from_bytes(bytes: Vec<i8>) -> Option<Self> {
        (bytes.len() == 2048).then_some(Self(bytes))
    }

    fn get(&self, x: usize, y: usize, z: usize) -> i32 {
        let index = y << 8 | z << 4 | x;
        let byte = i32::from(self.0[index >> 1]);
        if index & 1 == 0 {
            byte & 15
        } else {
            (byte >> 4) & 15
        }
    }
}

/// `ChunkPalettedStorageFix.Section`.
pub struct Section {
    /// The identity palette (`CrudeIncrementalIntIdentityHashBiMap`).
    palette: Vec<StateId>,
    list_tag: Vec<StateId>,
    section: Tag,
    has_data: bool,
    to_fix: Vec<(usize, Vec<i32>)>,
    update: Vec<i32>,
    /// The section's `Y`.
    pub y: i32,
    seen: HashSet<StateId>,
    buffer: Vec<i32>,
}

impl Section {
    /// `new Section(section)`.
    pub fn new(section: &Tag) -> Self {
        Self {
            palette: Vec::new(),
            list_tag: Vec::new(),
            section: section.clone(),
            has_data: get(section, "Blocks").is_some(),
            to_fix: Vec::new(),
            update: Vec::new(),
            y: get_i32_or(section, "Y", 0),
            seen: HashSet::new(),
            buffer: vec![0; SIZE],
        }
    }

    /// The `toFix` lists in insertion order.
    pub fn to_fix(&self) -> &[(usize, Vec<i32>)] {
        &self.to_fix
    }

    /// The positions whose neighbours are in other chunks (`update`).
    pub fn update(&self) -> &[i32] {
        &self.update
    }

    fn id_for(&mut self, state: StateId) -> i32 {
        match self.palette.iter().position(|existing| *existing == state) {
            Some(index) => index as i32,
            None => {
                self.palette.push(state);
                (self.palette.len() - 1) as i32
            }
        }
    }

    /// `Section.getBlock`.
    pub fn get_block(&self, pos: i32, constants: &MappingConstants) -> StateId {
        usize::try_from(pos)
            .ok()
            .filter(|pos| *pos < SIZE)
            .and_then(|pos| self.palette.get(self.buffer[pos] as usize))
            .copied()
            .unwrap_or(constants.air)
    }

    /// `Section.setBlock`.
    pub fn set_block(&mut self, index: i32, state: StateId, constants: &MappingConstants) {
        if self.seen.insert(state) {
            let listed = if state_name(state, constants) == FILTER_ME {
                constants.air
            } else {
                state
            };
            self.list_tag.push(listed);
        }
        self.buffer[index as usize] = self.id_for(state);
    }

    fn add_fix(&mut self, id: usize, position: i32) {
        match self.to_fix.iter_mut().find(|(existing, _)| *existing == id) {
            Some((_, positions)) => positions.push(position),
            None => self.to_fix.push((id, vec![position])),
        }
    }

    /// `Section.upgrade`: converts the legacy arrays; returns the updated side
    /// mask, or `None` where the Java code would throw.
    pub fn upgrade(&mut self, mut sides: i32) -> Option<i32> {
        if !self.has_data {
            return Some(sides);
        }
        let constants = MappingConstants::get();
        let blocks = get(&self.section, "Blocks").and_then(as_byte_buffer)?;
        if blocks.len() < SIZE {
            return None;
        }
        let data = self.data_layer("Data")?;
        let add_blocks = self.data_layer("Add")?;
        self.seen.insert(constants.air);
        self.id_for(constants.air);
        self.list_tag.push(constants.air);
        for (idx, block_byte) in blocks.iter().take(SIZE).enumerate() {
            let xx = idx & 15;
            let yy = (idx >> 8) & 15;
            let zz = (idx >> 4) & 15;
            let id = (add_blocks.get(xx, yy, zz) << 12)
                | ((i32::from(*block_byte) & 255) << 4)
                | data.get(xx, yy, zz);
            let block = (id >> 4) as usize;
            if FIX.contains(&block) {
                self.add_fix(block, idx as i32);
            }
            if VIRTUAL.contains(&block) {
                let s = super::get_side_mask(xx == 0, xx == 15, zz == 0, zz == 15);
                if s == 0 {
                    self.update.push(idx as i32);
                } else {
                    sides |= s;
                }
            }
            let (group, _) = get_state(id);
            self.set_block(idx as i32, group, constants);
        }
        Some(sides)
    }

    /// A nibble array field; absent or unreadable gives an empty layer, a wrong
    /// length gives `None` (Java throws).
    fn data_layer(&self, field: &str) -> Option<DataLayer> {
        match get(&self.section, field).and_then(as_byte_buffer) {
            Some(bytes) => DataLayer::from_bytes(bytes),
            None => Some(DataLayer::empty()),
        }
    }

    /// `Section.write`: the section with `Palette` / `BlockStates` in place of the
    /// legacy arrays. `None` where the Java code would throw.
    pub fn write(&self, constants: &MappingConstants) -> Option<Tag> {
        let mut section = self.section.clone();
        if !self.has_data {
            return Some(section);
        }
        let palette = self
            .list_tag
            .iter()
            .map(|state| block_state_to_tag(resolve(*state, constants)))
            .collect();
        set(&mut section, "Palette", Tag::List(palette));
        let size = ceil_log2(self.seen.len()).max(4);
        let mut storage = PackedBitStorage::new(size, SIZE);
        for (index, value) in self.buffer.iter().enumerate() {
            storage.set(index, i64::from(*value));
        }
        set(
            &mut section,
            "BlockStates",
            Tag::LongArray(storage.raw().to_vec()),
        );
        remove(&mut section, "Blocks");
        remove(&mut section, "Data");
        remove(&mut section, "Add");
        Some(section)
    }
}
