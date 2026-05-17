#![allow(dead_code)]

use crate::block_update::BlockPos;
use crate::storage::region::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomTickBlock {
    pub block_randomly_ticking: bool,
    pub fluid_randomly_ticking: bool,
    pub air: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomTickSection {
    pub section_y: i32,
    blocks: Vec<RandomTickBlock>,
    ticking_block_count: usize,
    ticking_fluid_count: usize,
    non_empty_block_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomTickTarget {
    Precipitation,
    Block,
    Fluid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomTickEvent {
    pub target: RandomTickTarget,
    pub pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelRandom {
    rand_value: i32,
}

pub const SECTION_SIZE: usize = 16 * 16 * 16;

impl RandomTickBlock {
    pub const AIR: Self = Self {
        block_randomly_ticking: false,
        fluid_randomly_ticking: false,
        air: true,
    };

    pub const fn block() -> Self {
        Self {
            block_randomly_ticking: true,
            fluid_randomly_ticking: false,
            air: false,
        }
    }

    pub const fn fluid() -> Self {
        Self {
            block_randomly_ticking: false,
            fluid_randomly_ticking: true,
            air: false,
        }
    }

    pub const fn block_and_fluid() -> Self {
        Self {
            block_randomly_ticking: true,
            fluid_randomly_ticking: true,
            air: false,
        }
    }
}

impl RandomTickSection {
    pub fn air(section_y: i32) -> Self {
        Self {
            section_y,
            blocks: vec![RandomTickBlock::AIR; SECTION_SIZE],
            ticking_block_count: 0,
            ticking_fluid_count: 0,
            non_empty_block_count: 0,
        }
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block: RandomTickBlock) {
        let idx = index(x, y, z);
        let previous = self.blocks[idx];
        self.remove_counts(previous);
        self.add_counts(block);
        self.blocks[idx] = block;
    }

    pub fn block(&self, x: u8, y: u8, z: u8) -> RandomTickBlock {
        self.blocks[index(x, y, z)]
    }

    pub fn is_randomly_ticking(&self) -> bool {
        self.ticking_block_count > 0 || self.ticking_fluid_count > 0
    }

    pub fn is_randomly_ticking_blocks(&self) -> bool {
        self.ticking_block_count > 0
    }

    pub fn is_randomly_ticking_fluids(&self) -> bool {
        self.ticking_fluid_count > 0
    }

    pub fn has_only_air(&self) -> bool {
        self.non_empty_block_count == 0
    }

    fn remove_counts(&mut self, block: RandomTickBlock) {
        if !block.air {
            self.non_empty_block_count -= 1;
        }
        if block.block_randomly_ticking {
            self.ticking_block_count -= 1;
        }
        if block.fluid_randomly_ticking {
            self.ticking_fluid_count -= 1;
        }
    }

    fn add_counts(&mut self, block: RandomTickBlock) {
        if !block.air {
            self.non_empty_block_count += 1;
        }
        if block.block_randomly_ticking {
            self.ticking_block_count += 1;
        }
        if block.fluid_randomly_ticking {
            self.ticking_fluid_count += 1;
        }
    }
}

impl LevelRandom {
    pub const fn new(rand_value: i32) -> Self {
        Self { rand_value }
    }

    pub fn block_random_pos(
        &mut self,
        x_origin: i32,
        y_origin: i32,
        z_origin: i32,
        y_mask: i32,
    ) -> BlockPos {
        self.rand_value = self.rand_value.wrapping_mul(3).wrapping_add(1_013_904_223);
        let value = self.rand_value >> 2;
        BlockPos {
            x: x_origin + (value & 15),
            y: y_origin + ((value >> 16) & y_mask),
            z: z_origin + ((value >> 8) & 15),
        }
    }
}

pub fn plan_random_ticks_for_chunk(
    chunk_pos: ChunkPos,
    sections: &[RandomTickSection],
    tick_speed: i32,
    random: &mut LevelRandom,
    precipitation_rolls: impl IntoIterator<Item = i32>,
) -> Vec<RandomTickEvent> {
    let mut events = Vec::new();
    let min_x = chunk_pos.x * 16;
    let min_z = chunk_pos.z * 16;

    for roll in precipitation_rolls
        .into_iter()
        .take(tick_speed.max(0) as usize)
    {
        if roll == 0 {
            events.push(RandomTickEvent {
                target: RandomTickTarget::Precipitation,
                pos: random.block_random_pos(min_x, 0, min_z, 15),
            });
        }
    }

    if tick_speed <= 0 {
        return events;
    }

    for section in sections {
        if !section.is_randomly_ticking() {
            continue;
        }
        let min_y = section.section_y * 16;
        for _ in 0..tick_speed {
            let pos = random.block_random_pos(min_x, min_y, min_z, 15);
            let local_x = (pos.x - min_x) as u8;
            let local_y = (pos.y - min_y) as u8;
            let local_z = (pos.z - min_z) as u8;
            let block = section.block(local_x, local_y, local_z);
            if block.block_randomly_ticking {
                events.push(RandomTickEvent {
                    target: RandomTickTarget::Block,
                    pos,
                });
            }
            if block.fluid_randomly_ticking {
                events.push(RandomTickEvent {
                    target: RandomTickTarget::Fluid,
                    pos,
                });
            }
        }
    }

    events
}

fn index(x: u8, y: u8, z: u8) -> usize {
    y as usize * 16 * 16 + z as usize * 16 + x as usize
}

#[cfg(test)]
mod tests {
    use super::{
        plan_random_ticks_for_chunk, LevelRandom, RandomTickBlock, RandomTickSection,
        RandomTickTarget,
    };
    use crate::block_update::BlockPos;
    use crate::storage::region::ChunkPos;

    #[test]
    fn get_block_random_pos_matches_vanilla_bit_layout() {
        let mut random = LevelRandom::new(0);
        assert_eq!(
            random.block_random_pos(32, 64, -16, 15),
            BlockPos {
                x: 39,
                y: 75,
                z: -4
            }
        );
        assert_eq!(
            random.block_random_pos(32, 64, -16, 15),
            BlockPos {
                x: 47,
                y: 78,
                z: -13
            }
        );
    }

    #[test]
    fn section_counts_track_randomly_ticking_blocks_and_fluids() {
        let mut section = RandomTickSection::air(4);
        assert!(section.has_only_air());
        assert!(!section.is_randomly_ticking());

        section.set_block(1, 2, 3, RandomTickBlock::block());
        assert!(!section.has_only_air());
        assert!(section.is_randomly_ticking_blocks());
        assert!(!section.is_randomly_ticking_fluids());

        section.set_block(1, 2, 3, RandomTickBlock::fluid());
        assert!(!section.is_randomly_ticking_blocks());
        assert!(section.is_randomly_ticking_fluids());

        section.set_block(1, 2, 3, RandomTickBlock::AIR);
        assert!(section.has_only_air());
        assert!(!section.is_randomly_ticking());
    }

    #[test]
    fn random_tick_plan_uses_tick_speed_per_randomly_ticking_section() {
        let chunk_pos = ChunkPos { x: 2, z: -1 };
        let mut section = RandomTickSection::air(4);
        let mut random = LevelRandom::new(0);
        let first = random.block_random_pos(32, 64, -16, 15);
        let second = random.block_random_pos(32, 64, -16, 15);
        section.set_block(
            (first.x - 32) as u8,
            (first.y - 64) as u8,
            (first.z + 16) as u8,
            RandomTickBlock::block(),
        );
        section.set_block(
            (second.x - 32) as u8,
            (second.y - 64) as u8,
            (second.z + 16) as u8,
            RandomTickBlock::block_and_fluid(),
        );

        let mut random = LevelRandom::new(0);
        let events = plan_random_ticks_for_chunk(chunk_pos, &[section], 2, &mut random, [1, 1]);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].target, RandomTickTarget::Block);
        assert_eq!(events[0].pos, first);
        assert_eq!(events[1].target, RandomTickTarget::Block);
        assert_eq!(events[1].pos, second);
        assert_eq!(events[2].target, RandomTickTarget::Fluid);
        assert_eq!(events[2].pos, second);
    }

    #[test]
    fn random_ticks_skip_sections_without_ticking_blocks_or_fluids() {
        let chunk_pos = ChunkPos { x: 0, z: 0 };
        let section = RandomTickSection::air(0);
        let mut random = LevelRandom::new(0);
        let events = plan_random_ticks_for_chunk(chunk_pos, &[section], 12, &mut random, [1; 12]);

        assert!(events.is_empty());
    }

    #[test]
    fn precipitation_rolls_run_before_block_random_ticks() {
        let chunk_pos = ChunkPos { x: 0, z: 0 };
        let mut section = RandomTickSection::air(0);
        let mut probe = LevelRandom::new(0);
        let precipitation_pos = probe.block_random_pos(0, 0, 0, 15);
        let block_pos = probe.block_random_pos(0, 0, 0, 15);
        section.set_block(
            block_pos.x as u8,
            block_pos.y as u8,
            block_pos.z as u8,
            RandomTickBlock::block(),
        );

        let mut random = LevelRandom::new(0);
        let events = plan_random_ticks_for_chunk(chunk_pos, &[section], 1, &mut random, [0]);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].target, RandomTickTarget::Precipitation);
        assert_eq!(events[0].pos, precipitation_pos);
        assert_eq!(events[1].target, RandomTickTarget::Block);
        assert_eq!(events[1].pos, block_pos);
    }
}
