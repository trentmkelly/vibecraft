//! Mirrors `net.minecraft.world.level.chunk.DataLayer` from the 26.1.2 decomp.
//!
//! A `DataLayer` stores 16×16×16 = 4096 nibbles (4-bit values) packed two per
//! byte, totalling 2048 bytes. It models the on-disk vanilla format used for
//! both block-light and sky-light section payloads.
//!
//! Layout invariants (kept identical to Java so chunk serialization round-trips
//! byte-for-byte):
//! - Index math: `index = (y << 8) | (z << 4) | x` for `0 <= x, y, z < 16`.
//! - Byte index: `index >> 1`; nibble index: `index & 1`; shift: `4 * nibble`.
//! - Default-value optimisation: while every nibble equals `default_value`,
//!   the backing buffer stays `None`; the first mutating `set` materialises it
//!   to a 2048-byte `Vec` pre-filled with `default_value` packed into both
//!   nibbles of each byte. This matches Java's lazy `data` allocation.

use crate::storage::chunk::LIGHT_DATA_LAYER_LENGTH;

/// Side length (and width/height) of a single light section, in blocks.
pub const SECTION_SIZE: usize = 16;
/// Total number of nibbles stored per section.
pub const SECTION_VOLUME: usize = SECTION_SIZE * SECTION_SIZE * SECTION_SIZE;
/// Maximum representable light level (4-bit nibble ceiling).
pub const MAX_LIGHT_LEVEL: u8 = 15;

/// Java: `DataLayer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayer {
    data: Option<Vec<u8>>,
    default_value: u8,
}

impl DataLayer {
    /// Java: `new DataLayer()` — default value 0, no backing buffer.
    pub fn empty() -> Self {
        Self::with_default(0)
    }

    /// Java: `new DataLayer(int defaultValue)`.
    pub fn with_default(default_value: u8) -> Self {
        Self {
            data: None,
            default_value: default_value & MAX_LIGHT_LEVEL,
        }
    }

    /// Java: `new DataLayer(byte[] data)` — caller-supplied 2048-byte buffer.
    pub fn from_bytes(data: Vec<u8>) -> Self {
        assert_eq!(
            data.len(),
            LIGHT_DATA_LAYER_LENGTH,
            "DataLayer requires exactly {} bytes",
            LIGHT_DATA_LAYER_LENGTH,
        );
        Self {
            data: Some(data),
            default_value: 0,
        }
    }

    /// Read a 4-bit value at the given local section coordinate.
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        self.get_index(Self::index(x, y, z))
    }

    /// Write a 4-bit value at the given local section coordinate.
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: u8) {
        self.set_index(Self::index(x, y, z), value);
    }

    /// Java: `fill(int value)` — wipes the backing buffer and stores a new
    /// homogenous default.
    pub fn fill(&mut self, value: u8) {
        self.default_value = value & MAX_LIGHT_LEVEL;
        self.data = None;
    }

    /// Java: `getData()` — lazily allocates a packed 2048-byte buffer the first
    /// time mutable bytes are needed.
    pub fn data_mut(&mut self) -> &mut [u8] {
        let default_value = self.default_value;
        self.data.get_or_insert_with(|| {
            let packed = Self::pack_filled(default_value);
            vec![packed; LIGHT_DATA_LAYER_LENGTH]
        })
    }

    /// View the materialised buffer if one exists; absent layers report `None`.
    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    /// Java: `copy()` — deep clones the backing buffer if any.
    pub fn copy(&self) -> Self {
        match &self.data {
            Some(buffer) => Self {
                data: Some(buffer.clone()),
                default_value: 0,
            },
            None => Self {
                data: None,
                default_value: self.default_value,
            },
        }
    }

    /// Java: `isDefinitelyHomogenous()` — true while the buffer is still the
    /// lazy default.
    pub fn is_definitely_homogenous(&self) -> bool {
        self.data.is_none()
    }

    /// Java: `isDefinitelyFilledWith(int value)`.
    pub fn is_definitely_filled_with(&self, value: u8) -> bool {
        self.data.is_none() && self.default_value == (value & MAX_LIGHT_LEVEL)
    }

    /// Java: `isEmpty()` — `data == null && defaultValue == 0`.
    pub fn is_empty(&self) -> bool {
        self.data.is_none() && self.default_value == 0
    }

    /// Convert the layer into the byte vector expected by chunk packets and
    /// region NBT. Lazy layers materialise on the way out.
    pub fn into_bytes(mut self) -> Vec<u8> {
        // `data_mut` materialises the buffer in place; `data` is therefore
        // guaranteed to be `Some` immediately after.
        self.data_mut();
        self.data.unwrap_or_default()
    }

    /// Borrow the contents as the 2048-byte buffer expected on the network,
    /// materialising it if necessary.
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self.data {
            Some(buffer) => buffer.clone(),
            None => vec![Self::pack_filled(self.default_value); LIGHT_DATA_LAYER_LENGTH],
        }
    }

    /// The current homogenous default value (only meaningful when
    /// `is_definitely_homogenous`).
    pub fn default_value(&self) -> u8 {
        self.default_value
    }

    /// Java: nibble index inside its byte (`index & 1`).
    fn nibble_index(index: usize) -> usize {
        index & 1
    }

    /// Java: byte index in the packed buffer (`index >> 1`).
    fn byte_index(index: usize) -> usize {
        index >> 1
    }

    /// Java: `getIndex(x, y, z)` — `(y << 8) | (z << 4) | x`.
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < SECTION_SIZE && y < SECTION_SIZE && z < SECTION_SIZE);
        (y << 8) | (z << 4) | x
    }

    fn get_index(&self, index: usize) -> u8 {
        match &self.data {
            Some(data) => {
                let byte = data[Self::byte_index(index)];
                (byte >> (4 * Self::nibble_index(index))) & MAX_LIGHT_LEVEL
            }
            None => self.default_value,
        }
    }

    fn set_index(&mut self, index: usize, value: u8) {
        let value = value & MAX_LIGHT_LEVEL;
        let byte_index = Self::byte_index(index);
        let shift = 4 * Self::nibble_index(index);
        let mask = !(MAX_LIGHT_LEVEL << shift);
        let data = self.data_mut();
        data[byte_index] = (data[byte_index] & mask) | (value << shift);
    }

    fn pack_filled(value: u8) -> u8 {
        let value = value & MAX_LIGHT_LEVEL;
        value | (value << 4)
    }
}

/// Storage-ready 2048-byte byte vector representing a section filled with
/// sky-light level 15 — the same byte pattern Java emits when serialising
/// a homogenous `DataLayer(15)`. Used by tests that need a parity fixture.
pub fn fullbright_sky_layer_bytes() -> Vec<i8> {
    DataLayer::with_default(MAX_LIGHT_LEVEL)
        .into_bytes()
        .into_iter()
        .map(|b| b as i8)
        .collect()
}
