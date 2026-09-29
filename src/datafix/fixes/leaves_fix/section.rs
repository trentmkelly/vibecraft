//! Port of `LeavesFix.Section`: a legacy (1.13 pre-flattening-finished) chunk
//! section with a block state palette and packed block states, shared by
//! [`LeavesFix`](super) and `TrappedChestBlockEntityFix`.

use crate::datafix::dynamic::{as_i64, get, get_i32_or, list_items};
use crate::datafix::packed_bit_storage::{ceil_log2, PackedBitStorage};
use crate::datafix::typed::{set_typed, typed};
use crate::storage::nbt::Tag;

/// Blocks per section (`SIZE`).
pub const SIZE: usize = 4096;

/// `Section`'s state: the palette, the section's `Y` and the packed states.
/// `storage` is `None` for a section the fix can skip.
pub struct Section {
    /// The palette entries (`Name` / `Properties` compounds).
    pub palette: Vec<Tag>,
    index: i32,
    /// `None` when `skippable()` held while reading.
    pub storage: Option<PackedBitStorage>,
}

/// `Dynamic.get("BlockStates").asLongStream().toArray()`.
fn read_long_stream(tag: &Tag) -> Vec<i64> {
    match get(tag, "BlockStates") {
        Some(Tag::LongArray(values)) => values.clone(),
        Some(other) => list_items(other)
            .map(|items| items.iter().filter_map(as_i64).collect())
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

impl Section {
    /// `new Section(section, inputSchema)`. `skippable` is the subclass hook
    /// (`Section.skippable()`), which may also rewrite the palette. Returns
    /// `None` where the Java constructor throws (a `BlockStates` array of the
    /// wrong length).
    pub fn read(section: &Tag, skippable: impl FnOnce(&mut Vec<Tag>) -> bool) -> Option<Self> {
        let mut palette = typed(section, "Palette")
            .and_then(list_items)
            .unwrap_or_default();
        let index = get_i32_or(section, "Y", 0);
        let storage = if skippable(&mut palette) {
            None
        } else {
            let states = read_long_stream(section);
            let bits = ceil_log2(palette.len()).max(4);
            let required = (SIZE * bits as usize).div_ceil(64);
            if states.len() != required {
                return None;
            }
            Some(PackedBitStorage::with_data(bits, SIZE, states))
        };
        Some(Self {
            palette,
            index,
            storage,
        })
    }

    /// `getIndex()`: the section's `Y`.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// `isSkippable()`.
    pub fn is_skippable(&self) -> bool {
        self.storage.is_none()
    }

    /// `getBlock(pos)`: the palette index at `pos`; only valid on a non-skippable
    /// section.
    pub fn block(&self, pos: usize) -> usize {
        self.storage.as_ref().map_or(0, |storage| storage.get(pos)) as usize
    }

    /// `write(section)`: stores the (possibly grown) palette and states.
    pub fn write(&self, section: &mut Tag) {
        let Some(storage) = &self.storage else {
            return;
        };
        crate::datafix::dynamic::set(
            section,
            "BlockStates",
            Tag::LongArray(storage.raw().to_vec()),
        );
        set_typed(section, "Palette", Tag::List(self.palette.clone()));
    }
}
