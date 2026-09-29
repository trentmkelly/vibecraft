//! Port of `net.minecraft.util.datafix.PackedBitStorage`: fixed-width values
//! packed into longs, with entries allowed to span two longs (the pre-1.16
//! layout that `ChunkPalettedStorageFix` writes).

/// `DataFixUtils.ceillog2`.
pub fn ceil_log2(value: usize) -> u32 {
    if value == 0 {
        0
    } else {
        usize::BITS - (value - 1).leading_zeros()
    }
}

/// `PackedBitStorage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedBitStorage {
    data: Vec<i64>,
    bits: u32,
    mask: i64,
    size: usize,
}

/// `Mth.roundToward(value, factor)`: `value` rounded up to a multiple of `factor`.
fn round_toward(value: usize, factor: usize) -> usize {
    value.div_ceil(factor) * factor
}

impl PackedBitStorage {
    /// `new PackedBitStorage(bits, size)`.
    pub fn new(bits: u32, size: usize) -> Self {
        let data = vec![0; round_toward(size * bits as usize, 64) / 64];
        Self::with_data(bits, size, data)
    }

    /// `new PackedBitStorage(bits, size, data)`; panics like the Java
    /// constructor throws (`bits` outside 1..=32 or a wrong data length).
    pub fn with_data(bits: u32, size: usize, data: Vec<i64>) -> Self {
        assert!((1..=32).contains(&bits), "bits must be between 1 and 32");
        let required = round_toward(size * bits as usize, 64) / 64;
        assert_eq!(
            data.len(),
            required,
            "Invalid length given for storage, got: {} but expected: {required}",
            data.len()
        );
        Self {
            data,
            bits,
            mask: (1_i64 << bits) - 1,
            size,
        }
    }

    /// `set(index, value)`.
    pub fn set(&mut self, index: usize, value: i64) {
        assert!(index < self.size, "index out of range");
        assert!((0..=self.mask).contains(&value), "value out of range");
        let bits = self.bits as usize;
        let position = index * bits;
        let start_data = position >> 6;
        let end_data = ((index + 1) * bits - 1) >> 6;
        let start_bit = position ^ (start_data << 6);
        self.data[start_data] = (self.data[start_data] & !(self.mask << start_bit))
            | ((value & self.mask) << start_bit);
        if start_data != end_data {
            let shift_bits = 64 - start_bit;
            let wanted_bits = bits - shift_bits;
            self.data[end_data] = (((self.data[end_data] as u64) >> wanted_bits) << wanted_bits)
                as i64
                | ((value & self.mask) >> shift_bits);
        }
    }

    /// `get(index)`.
    pub fn get(&self, index: usize) -> i64 {
        assert!(index < self.size, "index out of range");
        let bits = self.bits as usize;
        let position = index * bits;
        let start_data = position >> 6;
        let end_data = ((index + 1) * bits - 1) >> 6;
        let start_bit = position ^ (start_data << 6);
        if start_data == end_data {
            return (((self.data[start_data] as u64) >> start_bit) as i64) & self.mask;
        }
        let shift_bits = 64 - start_bit;
        ((((self.data[start_data] as u64) >> start_bit) as i64)
            | (self.data[end_data] << shift_bits))
            & self.mask
    }

    /// `getRaw()`.
    pub fn raw(&self) -> &[i64] {
        &self.data
    }

    /// `getBits()`.
    pub fn bits(&self) -> u32 {
        self.bits
    }
}
