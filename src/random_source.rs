#![allow(dead_code)]

const LEGACY_MULTIPLIER: u64 = 25_214_903_917;
const LEGACY_INCREMENT: u64 = 11;
const LEGACY_MASK: u64 = (1_u64 << 48) - 1;
pub const GOLDEN_RATIO_64: i64 = -7_046_029_254_386_353_131;
pub const SILVER_RATIO_64: i64 = 7_640_891_576_956_012_809;
const DOUBLE_UNIT: f64 = 1.0 / ((1_u64 << 53) as f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed128 {
    pub lo: i64,
    pub hi: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyRandom {
    seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xoroshiro128PlusPlus {
    seed_lo: u64,
    seed_hi: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomAlgorithm {
    Legacy,
    Xoroshiro,
}

impl LegacyRandom {
    pub fn new(seed: i64) -> Self {
        let mut random = Self { seed: 0 };
        random.set_seed(seed);
        random
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = ((seed as u64) ^ LEGACY_MULTIPLIER) & LEGACY_MASK;
    }

    pub fn next_bits(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(LEGACY_MULTIPLIER)
            .wrapping_add(LEGACY_INCREMENT)
            & LEGACY_MASK;
        (self.seed >> (48 - bits)) as i32
    }

    pub fn next_i32(&mut self) -> i32 {
        self.next_bits(32)
    }

    pub fn next_i32_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");
        if (bound & (bound - 1)) == 0 {
            return (((bound as i64) * (self.next_bits(31) as i64)) >> 31) as i32;
        }
        loop {
            let sample = self.next_bits(31);
            let modulo = sample % bound;
            if sample.wrapping_sub(modulo).wrapping_add(bound - 1) >= 0 {
                return modulo;
            }
        }
    }

    pub fn next_i64(&mut self) -> i64 {
        let upper = self.next_bits(32) as i64;
        let lower = self.next_bits(32) as i64;
        (upper << 32).wrapping_add(lower)
    }

    pub fn next_f64(&mut self) -> f64 {
        let upper = self.next_bits(26) as i64;
        let lower = self.next_bits(27) as i64;
        (((upper << 27) + lower) as f64) * DOUBLE_UNIT
    }
}

impl Xoroshiro128PlusPlus {
    pub fn from_i64_seed(seed: i64) -> Self {
        Self::from_seed128(upgrade_seed_to_128bit(seed))
    }

    pub fn from_seed128(seed: Seed128) -> Self {
        let mut seed_lo = seed.lo as u64;
        let mut seed_hi = seed.hi as u64;
        if (seed_lo | seed_hi) == 0 {
            seed_lo = GOLDEN_RATIO_64 as u64;
            seed_hi = SILVER_RATIO_64 as u64;
        }
        Self { seed_lo, seed_hi }
    }

    pub fn next_i64(&mut self) -> i64 {
        let s0 = self.seed_lo;
        let mut s1 = self.seed_hi;
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        s1 ^= s0;
        self.seed_lo = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.seed_hi = s1.rotate_left(28);
        result as i64
    }

    pub fn next_i32(&mut self) -> i32 {
        self.next_i64() as i32
    }

    pub fn next_i32_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");
        let mut random_bits = self.next_i32() as u32 as u64;
        let mut multiplied = random_bits * bound as u64;
        let mut fraction = multiplied & 0xFFFF_FFFF;
        if fraction < bound as u64 {
            let threshold = (0_u32.wrapping_sub(bound as u32) % bound as u32) as u64;
            while fraction < threshold {
                random_bits = self.next_i32() as u32 as u64;
                multiplied = random_bits * bound as u64;
                fraction = multiplied & 0xFFFF_FFFF;
            }
        }
        (multiplied >> 32) as i32
    }
}

pub fn mix_stafford_13(mut z: i64) -> i64 {
    z = (z ^ unsigned_shift_right(z, 30)).wrapping_mul(-4_658_895_280_553_007_687);
    z = (z ^ unsigned_shift_right(z, 27)).wrapping_mul(-7_723_592_293_110_705_685);
    z ^ unsigned_shift_right(z, 31)
}

pub fn upgrade_seed_to_128bit_unmixed(seed: i64) -> Seed128 {
    let lo = seed ^ SILVER_RATIO_64;
    let hi = lo.wrapping_add(GOLDEN_RATIO_64);
    Seed128 { lo, hi }
}

pub fn upgrade_seed_to_128bit(seed: i64) -> Seed128 {
    let unmixed = upgrade_seed_to_128bit_unmixed(seed);
    Seed128 {
        lo: mix_stafford_13(unmixed.lo),
        hi: mix_stafford_13(unmixed.hi),
    }
}

pub fn block_pos_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut seed =
        i64::from(x.wrapping_mul(3_129_871)) ^ (z as i64).wrapping_mul(116_129_781) ^ y as i64;
    seed = seed
        .wrapping_mul(seed)
        .wrapping_mul(42_317_861)
        .wrapping_add(seed.wrapping_mul(11));
    seed >> 16
}

pub fn linear_congruential_next(rval: i64, c: i64) -> i64 {
    rval.wrapping_mul(
        rval.wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407),
    )
    .wrapping_add(c)
}

pub fn decoration_seed(
    world_seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    algorithm: RandomAlgorithm,
) -> i64 {
    match algorithm {
        RandomAlgorithm::Legacy => {
            let mut random = LegacyRandom::new(world_seed);
            let x_scale = random.next_i64() | 1;
            let z_scale = random.next_i64() | 1;
            (chunk_x as i64)
                .wrapping_mul(x_scale)
                .wrapping_add((chunk_z as i64).wrapping_mul(z_scale))
                ^ world_seed
        }
        RandomAlgorithm::Xoroshiro => {
            let mut random = Xoroshiro128PlusPlus::from_i64_seed(world_seed);
            let x_scale = random.next_i64() | 1;
            let z_scale = random.next_i64() | 1;
            (chunk_x as i64)
                .wrapping_mul(x_scale)
                .wrapping_add((chunk_z as i64).wrapping_mul(z_scale))
                ^ world_seed
        }
    }
}

pub fn feature_seed(decoration_seed: i64, index: i32, step: i32) -> i64 {
    decoration_seed
        .wrapping_add(index as i64)
        .wrapping_add(10_000_i64.wrapping_mul(step as i64))
}

pub fn large_feature_seed(world_seed: i64, chunk_x: i32, chunk_z: i32) -> i64 {
    let mut random = LegacyRandom::new(world_seed);
    let x_scale = random.next_i64();
    let z_scale = random.next_i64();
    (chunk_x as i64).wrapping_mul(x_scale) ^ (chunk_z as i64).wrapping_mul(z_scale) ^ world_seed
}

pub fn large_feature_seed_with_salt(seed: i64, x: i32, z: i32, salt: i32) -> i64 {
    (x as i64)
        .wrapping_mul(341_873_128_712)
        .wrapping_add((z as i64).wrapping_mul(132_897_987_541))
        .wrapping_add(seed)
        .wrapping_add(salt as i64)
}

pub fn slime_chunk_seed(x: i32, z: i32, seed: i64, salt: i64) -> i64 {
    seed.wrapping_add(i64::from(x.wrapping_mul(x).wrapping_mul(4_987_142)))
        .wrapping_add(i64::from(x.wrapping_mul(5_947_611)))
        .wrapping_add((z as i64).wrapping_mul(z as i64).wrapping_mul(4_392_871))
        .wrapping_add(i64::from(z.wrapping_mul(389_711)))
        ^ salt
}

fn unsigned_shift_right(value: i64, shift: u32) -> i64 {
    ((value as u64) >> shift) as i64
}

#[cfg(test)]
mod tests {
    use super::{
        block_pos_seed, decoration_seed, feature_seed, large_feature_seed,
        large_feature_seed_with_salt, linear_congruential_next, mix_stafford_13, slime_chunk_seed,
        upgrade_seed_to_128bit, upgrade_seed_to_128bit_unmixed, LegacyRandom, RandomAlgorithm,
        Seed128, Xoroshiro128PlusPlus, GOLDEN_RATIO_64, SILVER_RATIO_64,
    };

    #[test]
    fn legacy_random_matches_java_random_outputs() {
        let mut random = LegacyRandom::new(1);
        assert_eq!(random.next_i32(), -1_155_869_325);
        assert_eq!(random.next_i32_bound(10), 8);
        assert_eq!(random.next_i64(), 7_564_655_870_752_979_346);

        let mut random = LegacyRandom::new(12345);
        assert_eq!(random.next_f64(), 0.3618031071604718);
    }

    #[test]
    fn random_support_seed_upgrade_matches_stafford_constants() {
        assert_eq!(GOLDEN_RATIO_64, -7_046_029_254_386_353_131);
        assert_eq!(SILVER_RATIO_64, 7_640_891_576_956_012_809);
        assert_eq!(mix_stafford_13(0), 0);
        assert_eq!(
            upgrade_seed_to_128bit_unmixed(0),
            Seed128 {
                lo: SILVER_RATIO_64,
                hi: 594_862_322_569_659_678
            }
        );
        assert_eq!(
            upgrade_seed_to_128bit(0),
            Seed128 {
                lo: 3_847_398_142_028_685_078,
                hi: 7_192_185_014_346_937_746
            }
        );
    }

    #[test]
    fn xoroshiro128plusplus_matches_vanilla_transition() {
        let mut random = Xoroshiro128PlusPlus::from_seed128(Seed128 { lo: 1, hi: 2 });
        assert_eq!(random.next_i64(), 393_217);
        assert_eq!(random.next_i64(), 669_327_710_093_319);

        let mut zero_seed = Xoroshiro128PlusPlus::from_seed128(Seed128 { lo: 0, hi: 0 });
        assert_eq!(zero_seed.next_i64(), 6_807_859_099_481_836_695);
    }

    #[test]
    fn worldgen_seed_derivations_match_vanilla_formulas() {
        assert_eq!(block_pos_seed(1, 2, 3), -33_674_130_277_896);
        assert_eq!(
            linear_congruential_next(123, 456),
            2_443_659_180_228_472_098
        );

        let legacy_decoration = decoration_seed(12345, 4, -7, RandomAlgorithm::Legacy);
        assert_eq!(legacy_decoration, -1_544_766_108_629_817_268);
        assert_eq!(
            feature_seed(legacy_decoration, 9, 2),
            -1_544_766_108_629_797_259
        );
        assert_eq!(large_feature_seed(12345, 4, -7), 752_069_546_558_437_189);
        assert_eq!(
            large_feature_seed_with_salt(12345, 4, -7, 10_387_313),
            437_217_001_719
        );

        assert_eq!(slime_chunk_seed(4, -7, 12345, 987_234_911), 672_110_732);
        let mut slime = LegacyRandom::new(slime_chunk_seed(4, -7, 12345, 987_234_911));
        assert_ne!(slime.next_i32_bound(10), 0);
    }
}
