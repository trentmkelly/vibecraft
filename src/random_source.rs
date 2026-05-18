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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionalRandomFactory {
    Legacy { seed: i64 },
    Xoroshiro { seed_lo: i64, seed_hi: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomStateSeedFactories {
    pub base: PositionalRandomFactory,
    pub aquifer: PositionalRandomFactory,
    pub ore: PositionalRandomFactory,
    pub terrain: RandomSourceKind,
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

    pub fn fork(&mut self) -> Self {
        Self::new(self.next_i64())
    }

    pub fn fork_positional(&mut self) -> PositionalRandomFactory {
        PositionalRandomFactory::Legacy {
            seed: self.next_i64(),
        }
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

    pub fn fork(&mut self) -> Self {
        Self::from_seed128(Seed128 {
            lo: self.next_i64(),
            hi: self.next_i64(),
        })
    }

    pub fn fork_positional(&mut self) -> PositionalRandomFactory {
        PositionalRandomFactory::Xoroshiro {
            seed_lo: self.next_i64(),
            seed_hi: self.next_i64(),
        }
    }
}

impl PositionalRandomFactory {
    pub fn from_seed(self, seed: i64) -> RandomSourceKind {
        match self {
            PositionalRandomFactory::Legacy { .. } => {
                RandomSourceKind::Legacy(LegacyRandom::new(seed))
            }
            PositionalRandomFactory::Xoroshiro { seed_lo, seed_hi } => {
                RandomSourceKind::Xoroshiro(Xoroshiro128PlusPlus::from_seed128(Seed128 {
                    lo: seed ^ seed_lo,
                    hi: seed ^ seed_hi,
                }))
            }
        }
    }

    pub fn at(self, x: i32, y: i32, z: i32) -> RandomSourceKind {
        let positional_seed = block_pos_seed(x, y, z);
        match self {
            PositionalRandomFactory::Legacy { seed } => {
                RandomSourceKind::Legacy(LegacyRandom::new(positional_seed ^ seed))
            }
            PositionalRandomFactory::Xoroshiro { seed_lo, seed_hi } => {
                RandomSourceKind::Xoroshiro(Xoroshiro128PlusPlus::from_seed128(Seed128 {
                    lo: positional_seed ^ seed_lo,
                    hi: seed_hi,
                }))
            }
        }
    }

    pub fn from_hash_of_legacy(self, name: &str) -> Option<LegacyRandom> {
        match self {
            PositionalRandomFactory::Legacy { seed } => {
                Some(LegacyRandom::new(java_string_hash(name) as i64 ^ seed))
            }
            PositionalRandomFactory::Xoroshiro { .. } => None,
        }
    }

    pub fn from_hash_of(self, name: &str) -> RandomSourceKind {
        match self {
            PositionalRandomFactory::Legacy { seed } => {
                RandomSourceKind::Legacy(LegacyRandom::new(java_string_hash(name) as i64 ^ seed))
            }
            PositionalRandomFactory::Xoroshiro { seed_lo, seed_hi } => {
                let seed = seed128_from_hash_of(name);
                RandomSourceKind::Xoroshiro(Xoroshiro128PlusPlus::from_seed128(Seed128 {
                    lo: seed.lo ^ seed_lo,
                    hi: seed.hi ^ seed_hi,
                }))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomSourceKind {
    Legacy(LegacyRandom),
    Xoroshiro(Xoroshiro128PlusPlus),
}

impl RandomSourceKind {
    pub fn new(seed: i64, algorithm: RandomAlgorithm) -> Self {
        match algorithm {
            RandomAlgorithm::Legacy => Self::Legacy(LegacyRandom::new(seed)),
            RandomAlgorithm::Xoroshiro => {
                Self::Xoroshiro(Xoroshiro128PlusPlus::from_i64_seed(seed))
            }
        }
    }

    pub fn fork_positional(&mut self) -> PositionalRandomFactory {
        match self {
            Self::Legacy(random) => random.fork_positional(),
            Self::Xoroshiro(random) => random.fork_positional(),
        }
    }
}

pub fn random_state_seed_factories(
    seed: i64,
    algorithm: RandomAlgorithm,
) -> RandomStateSeedFactories {
    let mut random = RandomSourceKind::new(seed, algorithm);
    let base = random.fork_positional();
    let aquifer = base.from_hash_of("minecraft:aquifer").fork_positional();
    let ore = base.from_hash_of("minecraft:ore").fork_positional();
    let terrain = match algorithm {
        RandomAlgorithm::Legacy => RandomSourceKind::Legacy(LegacyRandom::new(seed)),
        RandomAlgorithm::Xoroshiro => base.from_hash_of("minecraft:terrain"),
    };
    RandomStateSeedFactories {
        base,
        aquifer,
        ore,
        terrain,
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

pub fn seed128_from_md5_digest(digest: [u8; 16]) -> Seed128 {
    Seed128 {
        lo: i64::from_be_bytes(digest[0..8].try_into().expect("fixed MD5 low half")),
        hi: i64::from_be_bytes(digest[8..16].try_into().expect("fixed MD5 high half")),
    }
}

pub fn seed128_from_hash_of(input: &str) -> Seed128 {
    seed128_from_md5_digest(md5_digest(input.as_bytes()))
}

pub fn java_string_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0_i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(unit as i32)
    })
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

fn md5_digest(input: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let mut message = input.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_le_bytes());

    let mut a0 = 0x67452301u32;
    let mut b0 = 0xefcdab89u32;
    let mut c0 = 0x98badcfeu32;
    let mut d0 = 0x10325476u32;

    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 16];
        for (index, word) in words.iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_le_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let next = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(words[g])
                    .rotate_left(S[i]),
            );
            a = d;
            d = c;
            c = b;
            b = next;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut digest = [0u8; 16];
    digest[0..4].copy_from_slice(&a0.to_le_bytes());
    digest[4..8].copy_from_slice(&b0.to_le_bytes());
    digest[8..12].copy_from_slice(&c0.to_le_bytes());
    digest[12..16].copy_from_slice(&d0.to_le_bytes());
    digest
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
        block_pos_seed, decoration_seed, feature_seed, java_string_hash, large_feature_seed,
        large_feature_seed_with_salt, linear_congruential_next, mix_stafford_13,
        random_state_seed_factories, slime_chunk_seed, upgrade_seed_to_128bit,
        upgrade_seed_to_128bit_unmixed, LegacyRandom, PositionalRandomFactory, RandomAlgorithm,
        RandomSourceKind, Seed128, Xoroshiro128PlusPlus, GOLDEN_RATIO_64, SILVER_RATIO_64,
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
        assert_eq!(
            super::seed128_from_md5_digest([
                0x95, 0x74, 0x12, 0x41, 0xc4, 0x91, 0x68, 0x95, 0x53, 0x8c, 0x2e, 0x10, 0x05, 0xbc,
                0x91, 0xb7,
            ]),
            Seed128 {
                lo: -7_677_491_391_079_815_019,
                hi: 6_020_237_448_238_109_111
            }
        );
        assert_eq!(
            super::seed128_from_hash_of("minecraft:terrain"),
            Seed128 {
                lo: 2_226_279_196_109_926_164,
                hi: -2_108_001_439_914_377_933
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
        assert_eq!(java_string_hash("minecraft:terrain"), 1_657_813_608);
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

    #[test]
    fn positional_random_factories_match_vanilla_fork_rules() {
        let mut legacy = LegacyRandom::new(12345);
        let mut fork = legacy.fork();
        assert_eq!(fork.next_i32(), -1_511_962_450);
        let factory = legacy.fork_positional();
        assert_eq!(
            factory,
            PositionalRandomFactory::Legacy {
                seed: -1_236_052_134_575_208_584
            }
        );
        match factory.at(1, 2, 3) {
            RandomSourceKind::Legacy(mut random) => {
                assert_eq!(random.next_i32(), -879_662_638);
            }
            _ => panic!("legacy positional factory should create legacy randoms"),
        }
        match factory.from_seed(99) {
            RandomSourceKind::Legacy(mut random) => {
                assert_eq!(random.next_i32(), -1_192_035_722);
            }
            _ => panic!("legacy from_seed should create legacy randoms"),
        }
        let mut hashed = factory
            .from_hash_of_legacy("minecraft:terrain")
            .expect("legacy factory supports Java String.hashCode hashing");
        assert_eq!(hashed.next_i32(), 1_947_910_319);
        match factory.from_hash_of("minecraft:terrain") {
            RandomSourceKind::Legacy(mut random) => {
                assert_eq!(random.next_i32(), 1_947_910_319);
            }
            _ => panic!("legacy from_hash_of should create legacy randoms"),
        }

        let mut xoroshiro = Xoroshiro128PlusPlus::from_i64_seed(12345);
        let mut fork = xoroshiro.fork();
        assert_eq!(fork.next_i64(), 782_221_843_147_428_965);
        let factory = xoroshiro.fork_positional();
        assert_eq!(
            factory,
            PositionalRandomFactory::Xoroshiro {
                seed_lo: 4_143_755_034_716_878_659,
                seed_hi: 1_226_499_899_398_695_337
            }
        );
        match factory.at(1, 2, 3) {
            RandomSourceKind::Xoroshiro(mut random) => {
                assert_eq!(random.next_i64(), 8_759_782_289_353_588_162);
            }
            _ => panic!("xoroshiro positional factory should create xoroshiro randoms"),
        }
        match factory.from_seed(99) {
            RandomSourceKind::Xoroshiro(mut random) => {
                assert_eq!(random.next_i64(), 3_338_114_822_160_895_021);
            }
            _ => panic!("xoroshiro from_seed should create xoroshiro randoms"),
        }
        assert_eq!(factory.from_hash_of_legacy("minecraft:terrain"), None);
        match factory.from_hash_of("minecraft:terrain") {
            RandomSourceKind::Xoroshiro(mut random) => {
                assert_eq!(random.next_i64(), 2_703_920_793_147_051_671);
            }
            _ => panic!("xoroshiro from_hash_of should create xoroshiro randoms"),
        }
    }

    #[test]
    fn random_state_seed_factories_match_vanilla_randomstate_wiring() {
        let legacy = random_state_seed_factories(12345, RandomAlgorithm::Legacy);
        assert_eq!(
            legacy.base,
            PositionalRandomFactory::Legacy {
                seed: 6_674_089_274_190_705_457
            }
        );
        assert_eq!(
            legacy.aquifer,
            PositionalRandomFactory::Legacy {
                seed: -2_943_771_310_165_987_104
            }
        );
        assert_eq!(
            legacy.ore,
            PositionalRandomFactory::Legacy {
                seed: 7_663_849_966_042_850_292
            }
        );
        match legacy.terrain {
            RandomSourceKind::Legacy(mut random) => {
                assert_eq!(random.next_i64(), 6_674_089_274_190_705_457);
            }
            _ => panic!("legacy terrain random should use the raw world seed"),
        }

        let xoroshiro = random_state_seed_factories(12345, RandomAlgorithm::Xoroshiro);
        assert_eq!(
            xoroshiro.base,
            PositionalRandomFactory::Xoroshiro {
                seed_lo: -8_118_485_274_630_516_485,
                seed_hi: 8_241_557_746_459_281_790
            }
        );
        assert_eq!(
            xoroshiro.aquifer,
            PositionalRandomFactory::Xoroshiro {
                seed_lo: 7_280_243_795_428_841_706,
                seed_hi: -8_497_267_231_399_644_069
            }
        );
        assert_eq!(
            xoroshiro.ore,
            PositionalRandomFactory::Xoroshiro {
                seed_lo: 8_451_019_449_222_520_003,
                seed_hi: -7_319_213_125_637_571_227
            }
        );
        match xoroshiro.terrain {
            RandomSourceKind::Xoroshiro(mut random) => {
                assert_eq!(random.next_i64(), 2_469_905_110_261_187_857);
            }
            _ => panic!("xoroshiro terrain random should hash minecraft:terrain"),
        }
    }
}
