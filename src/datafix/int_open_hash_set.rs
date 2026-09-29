//! A faithful model of fastutil's `IntOpenHashSet` (load factor 0.75).
//!
//! `LeavesFix` and `TrappedChestBlockEntityFix` iterate `IntOpenHashSet`s, and the
//! iteration order is observable (it decides the order in which new palette
//! entries are appended). The order depends on the hash function, the linear
//! probing and the rehash points, so those are reproduced exactly.

/// fastutil's default expected size (`DEFAULT_INITIAL_SIZE`).
const DEFAULT_INITIAL_SIZE: usize = 16;
/// fastutil's default load factor (`DEFAULT_LOAD_FACTOR`).
const LOAD_FACTOR: f64 = 0.75;

/// `HashCommon.arraySize(expected, f)`: the table size for `expected` entries.
fn array_size(expected: usize) -> usize {
    let needed = (expected as f64 / LOAD_FACTOR).ceil() as usize;
    needed.next_power_of_two().max(2)
}

/// `HashCommon.maxFill(n, f)`.
fn max_fill(n: usize) -> usize {
    ((n as f64 * LOAD_FACTOR).ceil() as usize).min(n - 1)
}

/// `HashCommon.mix(int)`.
fn mix(value: i32) -> u32 {
    let h = (value as u32).wrapping_mul(0x9E37_79B9);
    h ^ (h >> 16)
}

/// `it.unimi.dsi.fastutil.ints.IntOpenHashSet`.
#[derive(Debug, Clone)]
pub struct IntOpenHashSet {
    /// The open-addressing table; `0` marks an empty slot.
    keys: Vec<i32>,
    mask: usize,
    contains_null: bool,
    max_fill: usize,
    size: usize,
}

impl Default for IntOpenHashSet {
    fn default() -> Self {
        Self::new()
    }
}

impl IntOpenHashSet {
    /// `new IntOpenHashSet()`.
    pub fn new() -> Self {
        Self::with_expected(DEFAULT_INITIAL_SIZE)
    }

    /// `new IntOpenHashSet(expected)`; also the initial table of
    /// `new Int2ObjectOpenHashMap(expected)`, whose key iteration order is the
    /// same.
    pub fn with_expected(expected: usize) -> Self {
        let n = array_size(expected);
        Self {
            keys: vec![0; n],
            mask: n - 1,
            contains_null: false,
            max_fill: max_fill(n),
            size: 0,
        }
    }

    /// `size()`.
    pub fn len(&self) -> usize {
        self.size
    }

    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn slot(&self, key: i32) -> usize {
        mix(key) as usize & self.mask
    }

    /// `contains(key)`.
    pub fn contains(&self, key: i32) -> bool {
        if key == 0 {
            return self.contains_null;
        }
        let mut pos = self.slot(key);
        loop {
            let current = self.keys[pos];
            if current == 0 {
                return false;
            }
            if current == key {
                return true;
            }
            pos = (pos + 1) & self.mask;
        }
    }

    /// `add(key)`; returns whether the set changed.
    pub fn add(&mut self, key: i32) -> bool {
        if key == 0 {
            if self.contains_null {
                return false;
            }
            self.contains_null = true;
        } else {
            let mut pos = self.slot(key);
            loop {
                let current = self.keys[pos];
                if current == 0 {
                    break;
                }
                if current == key {
                    return false;
                }
                pos = (pos + 1) & self.mask;
            }
            self.keys[pos] = key;
        }
        let grow = self.size >= self.max_fill;
        self.size += 1;
        if grow {
            self.rehash(array_size(self.size + 1));
        }
        true
    }

    /// `rehash(newN)`: re-inserts the keys from the highest slot down.
    fn rehash(&mut self, new_n: usize) {
        let old = std::mem::replace(&mut self.keys, vec![0; new_n]);
        self.mask = new_n - 1;
        self.max_fill = max_fill(new_n);
        for key in old.into_iter().rev().filter(|key| *key != 0) {
            let mut pos = self.slot(key);
            while self.keys[pos] != 0 {
                pos = (pos + 1) & self.mask;
            }
            self.keys[pos] = key;
        }
    }

    /// The `iterator()` order: the null key first, then the table from the last
    /// slot down to the first.
    pub fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        self.contains_null
            .then_some(0)
            .into_iter()
            .chain(self.keys.iter().rev().copied().filter(|key| *key != 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_and_growth_keep_every_key() {
        let mut set = IntOpenHashSet::new();
        for key in 0..1000 {
            assert!(set.add(key * 7));
            assert!(!set.add(key * 7));
        }
        assert_eq!(set.len(), 1000);
        assert!((0..1000).all(|key| set.contains(key * 7)));
        assert!(!set.contains(3));
        assert_eq!(set.iter().count(), 1000);
    }

    #[test]
    fn iteration_starts_with_zero_then_walks_the_table_backwards() {
        let mut set = IntOpenHashSet::new();
        set.add(5);
        set.add(0);
        set.add(1);
        let order: Vec<i32> = set.iter().collect();
        assert_eq!(order[0], 0);
        let slot = |key: i32| mix(key) as usize & 31;
        // Higher slots come first.
        assert_eq!(order[1] == 5, slot(5) > slot(1));
    }
}
