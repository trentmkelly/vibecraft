//! Bit-storage contracts and the zero-bit implementation from vanilla.

#![allow(dead_code)]

pub trait BitStorage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32;
    fn set(&mut self, index: i32, value: i32);
    fn get(&self, index: i32) -> i32;
    fn get_raw(&self) -> &'static [u64];
    fn get_size(&self) -> i32;
    fn get_bits(&self) -> i32;
    fn get_all(&self, output: &mut dyn FnMut(i32));
    fn unpack(&self, output: &mut [i32]);
    fn copy(&self) -> Self
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZeroBitStorage {
    size: i32,
}

impl ZeroBitStorage {
    pub const RAW: &'static [u64] = &[];

    pub const fn new(size: i32) -> Self {
        Self { size }
    }

    fn validate_index(&self, index: i32) {
        assert!(
            index >= 0 && index < self.size,
            "index {} outside [0, {})",
            index,
            self.size
        );
    }

    fn validate_value(value: i32) {
        assert_eq!(value, 0, "value {} outside [0, 0]", value);
    }
}

impl BitStorage for ZeroBitStorage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32 {
        self.validate_index(index);
        Self::validate_value(value);
        0
    }

    fn set(&mut self, index: i32, value: i32) {
        self.validate_index(index);
        Self::validate_value(value);
    }

    fn get(&self, index: i32) -> i32 {
        self.validate_index(index);
        0
    }

    fn get_raw(&self) -> &'static [u64] {
        Self::RAW
    }

    fn get_size(&self) -> i32 {
        self.size
    }

    fn get_bits(&self) -> i32 {
        0
    }

    fn get_all(&self, output: &mut dyn FnMut(i32)) {
        for _ in 0..self.size {
            output(0);
        }
    }

    fn unpack(&self, output: &mut [i32]) {
        for value in &mut output[..self.size as usize] {
            *value = 0;
        }
    }

    fn copy(&self) -> Self {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::{BitStorage, ZeroBitStorage};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn bit_storage_and_zero_storage_match_java_sources() {
        const BIT_STORAGE: &str = vibecraft_java_source!("/net/minecraft/util/BitStorage.java");
        const ZERO_STORAGE: &str =
            vibecraft_java_source!("/net/minecraft/util/ZeroBitStorage.java");
        assert_eq!(BIT_STORAGE.lines().count(), 23);
        assert_eq!(ZERO_STORAGE.lines().count(), 65);
        for fragment in [
            "int getAndSet(int index, int value)",
            "void set(int index, int value)",
            "int get(int index)",
            "long[] getRaw()",
            "void getAll(IntConsumer output)",
            "void unpack(int[] output)",
            "BitStorage copy()",
        ] {
            assert!(BIT_STORAGE.contains(fragment), "missing BitStorage source fragment: {fragment}");
        }
        for fragment in [
            "public static final long[] RAW = new long[0]",
            "Validate.inclusiveBetween(0L, this.size - 1, index)",
            "Validate.inclusiveBetween(0L, 0L, value)",
            "return 0",
            "return RAW",
            "return this",
        ] {
            assert!(
                ZERO_STORAGE.contains(fragment),
                "missing ZeroBitStorage source fragment: {fragment}"
            );
        }
    }

    #[test]
    fn zero_bit_storage_validates_and_returns_zero_like_vanilla() {
        let mut storage = ZeroBitStorage::new(3);
        assert_eq!(storage.get_size(), 3);
        assert_eq!(storage.get_bits(), 0);
        assert_eq!(storage.get_raw(), &[] as &[u64]);
        assert_eq!(storage.get(0), 0);
        assert_eq!(storage.get_and_set(1, 0), 0);
        storage.set(2, 0);

        let mut all = Vec::new();
        storage.get_all(&mut |value| all.push(value));
        assert_eq!(all, vec![0, 0, 0]);
        let mut unpacked = vec![7, 7, 7, 7];
        storage.unpack(&mut unpacked);
        assert_eq!(unpacked, vec![0, 0, 0, 7]);
        assert_eq!(storage.copy(), storage);

        assert!(std::panic::catch_unwind(|| storage.get(3)).is_err());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            storage.set(0, 1);
        }))
        .is_err());
    }
}
