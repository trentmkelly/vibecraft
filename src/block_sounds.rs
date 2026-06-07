#![allow(dead_code)]

use crate::block_update::{BlockPos, Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientDesertSound {
    SandIdle,
    DryGrass,
    DeadBushIdle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientDesertSoundEmission {
    Local {
        sound: AmbientDesertSound,
        pos: BlockPos,
    },
    Player {
        sound: AmbientDesertSound,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmbientDesertRandom {
    pub idle_sand_roll: i32,
    pub dry_grass_roll: i32,
    pub dead_bush_roll: i32,
    pub badlands_dead_bush_skip_roll: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmbientDesertColumnProbe {
    pub pos: BlockPos,
    pub surface_y: i32,
}

impl AmbientDesertSound {
    pub fn id(self) -> &'static str {
        match self {
            Self::SandIdle => "minecraft:block.sand.idle",
            Self::DryGrass => "minecraft:block.dry_grass.ambient",
            Self::DeadBushIdle => "minecraft:block.deadbush.idle",
        }
    }
}

pub fn ambient_desert_sounds_for_tick(
    pos: BlockPos,
    random: AmbientDesertRandom,
    block_at: impl Fn(BlockPos) -> &'static str,
    surface_y_at: impl Fn(BlockPos) -> i32,
) -> Vec<AmbientDesertSoundEmission> {
    let mut emissions = Vec::new();
    if block_at(pos.relative(Direction::Up)) == "minecraft:air"
        && random.idle_sand_roll == 0
        && should_play_ambient_sand_sound(pos, &block_at, &surface_y_at)
    {
        emissions.push(AmbientDesertSoundEmission::Local {
            sound: AmbientDesertSound::SandIdle,
            pos,
        });
    }

    if random.dry_grass_roll == 0
        && should_play_desert_dry_vegetation_block_sounds(pos.relative(Direction::Down), &block_at)
    {
        emissions.push(AmbientDesertSoundEmission::Player {
            sound: AmbientDesertSound::DryGrass,
        });
    }

    if random.dead_bush_roll == 0 {
        let below_pos = pos.relative(Direction::Down);
        let below_block = block_at(below_pos);
        let badlands_block =
            below_block == "minecraft:red_sand" || block_matches_terracotta_tag(below_block);
        if !(badlands_block && random.badlands_dead_bush_skip_roll != 0)
            && should_play_desert_dry_vegetation_block_sounds(below_pos, &block_at)
        {
            emissions.push(AmbientDesertSoundEmission::Local {
                sound: AmbientDesertSound::DeadBushIdle,
                pos,
            });
        }
    }
    emissions
}

pub fn should_play_desert_dry_vegetation_block_sounds(
    below_pos: BlockPos,
    block_at: impl Fn(BlockPos) -> &'static str,
) -> bool {
    block_triggers_ambient_desert_dry_vegetation_sounds(block_at(below_pos))
        && block_triggers_ambient_desert_dry_vegetation_sounds(block_at(
            below_pos.relative(Direction::Down),
        ))
}

pub fn should_play_ambient_sand_sound(
    pos: BlockPos,
    block_at: impl Fn(BlockPos) -> &'static str,
    surface_y_at: impl Fn(BlockPos) -> i32,
) -> bool {
    let mut matching_blocks_found = 0;
    let mut sides_checked = 0;
    for dir in [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        let probe_pos = offset_horizontal(pos, dir, 8);
        let probe = AmbientDesertColumnProbe {
            pos: probe_pos,
            surface_y: surface_y_at(probe_pos),
        };
        if column_contains_ambient_sand_trigger(probe, &block_at) {
            if matching_blocks_found >= 3 {
                return true;
            }
            matching_blocks_found += 1;
        }
        sides_checked += 1;
        if (4 - sides_checked) + matching_blocks_found < 3 {
            return false;
        }
    }
    false
}

pub fn column_contains_ambient_sand_trigger(
    probe: AmbientDesertColumnProbe,
    block_at: impl Fn(BlockPos) -> &'static str,
) -> bool {
    if (probe.surface_y - probe.pos.y).abs() > 5 {
        let mut current = BlockPos {
            y: probe.pos.y + 5,
            ..probe.pos
        };
        let mut above_block = block_at(BlockPos {
            y: probe.pos.y + 6,
            ..probe.pos
        });
        for _ in 0..10 {
            let current_block = block_at(current);
            if above_block == "minecraft:air"
                && block_triggers_ambient_desert_sand_sounds(current_block)
            {
                return true;
            }
            above_block = current_block;
            current = current.relative(Direction::Down);
        }
        false
    } else {
        let above = BlockPos {
            y: probe.surface_y + 1,
            ..probe.pos
        };
        let surface = BlockPos {
            y: probe.surface_y,
            ..probe.pos
        };
        block_at(above) == "minecraft:air"
            && block_triggers_ambient_desert_sand_sounds(block_at(surface))
    }
}

pub fn block_triggers_ambient_desert_sand_sounds(block: &str) -> bool {
    matches!(block, "minecraft:sand" | "minecraft:red_sand")
}

pub fn block_triggers_ambient_desert_dry_vegetation_sounds(block: &str) -> bool {
    block_triggers_ambient_desert_sand_sounds(block) || block_matches_terracotta_tag(block)
}

fn block_matches_terracotta_tag(block: &str) -> bool {
    matches!(
        block,
        "minecraft:terracotta"
            | "minecraft:white_terracotta"
            | "minecraft:orange_terracotta"
            | "minecraft:magenta_terracotta"
            | "minecraft:light_blue_terracotta"
            | "minecraft:yellow_terracotta"
            | "minecraft:lime_terracotta"
            | "minecraft:pink_terracotta"
            | "minecraft:gray_terracotta"
            | "minecraft:light_gray_terracotta"
            | "minecraft:cyan_terracotta"
            | "minecraft:purple_terracotta"
            | "minecraft:blue_terracotta"
            | "minecraft:brown_terracotta"
            | "minecraft:green_terracotta"
            | "minecraft:red_terracotta"
            | "minecraft:black_terracotta"
    )
}

fn offset_horizontal(pos: BlockPos, direction: Direction, distance: i32) -> BlockPos {
    match direction {
        Direction::North => BlockPos {
            z: pos.z - distance,
            ..pos
        },
        Direction::South => BlockPos {
            z: pos.z + distance,
            ..pos
        },
        Direction::West => BlockPos {
            x: pos.x - distance,
            ..pos
        },
        Direction::East => BlockPos {
            x: pos.x + distance,
            ..pos
        },
        Direction::Down | Direction::Up => pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos() -> BlockPos {
        BlockPos { x: 0, y: 64, z: 0 }
    }

    #[test]
    fn ambient_desert_sound_ids_match_java_sound_events() {
        assert_eq!(
            AmbientDesertSound::SandIdle.id(),
            "minecraft:block.sand.idle"
        );
        assert_eq!(
            AmbientDesertSound::DryGrass.id(),
            "minecraft:block.dry_grass.ambient"
        );
        assert_eq!(
            AmbientDesertSound::DeadBushIdle.id(),
            "minecraft:block.deadbush.idle"
        );
    }

    #[test]
    fn desert_sound_block_tags_match_vanilla_resources() {
        assert!(block_triggers_ambient_desert_sand_sounds("minecraft:sand"));
        assert!(block_triggers_ambient_desert_sand_sounds(
            "minecraft:red_sand"
        ));
        assert!(!block_triggers_ambient_desert_sand_sounds(
            "minecraft:terracotta"
        ));
        assert!(block_triggers_ambient_desert_dry_vegetation_sounds(
            "minecraft:blue_terracotta"
        ));
        assert!(block_triggers_ambient_desert_dry_vegetation_sounds(
            "minecraft:red_sand"
        ));
        assert!(!block_triggers_ambient_desert_dry_vegetation_sounds(
            "minecraft:dirt"
        ));
    }

    #[test]
    fn dry_vegetation_requires_two_triggering_blocks_below() {
        assert!(should_play_desert_dry_vegetation_block_sounds(pos(), |p| {
            if p.y == 64 || p.y == 63 {
                "minecraft:sand"
            } else {
                "minecraft:air"
            }
        }));
        assert!(!should_play_desert_dry_vegetation_block_sounds(
            pos(),
            |p| {
                if p.y == 64 {
                    "minecraft:sand"
                } else {
                    "minecraft:air"
                }
            }
        ));
    }

    #[test]
    fn ambient_sand_column_scan_matches_surface_and_far_scan_paths() {
        let near = AmbientDesertColumnProbe {
            pos: pos(),
            surface_y: 64,
        };
        assert!(column_contains_ambient_sand_trigger(near, |p| match p.y {
            65 => "minecraft:air",
            64 => "minecraft:red_sand",
            _ => "minecraft:stone",
        }));

        let far = AmbientDesertColumnProbe {
            pos: pos(),
            surface_y: 90,
        };
        assert!(column_contains_ambient_sand_trigger(far, |p| match p.y {
            70 => "minecraft:air",
            69 => "minecraft:sand",
            _ => "minecraft:stone",
        }));
        assert!(!column_contains_ambient_sand_trigger(far, |p| match p.y {
            70 => "minecraft:stone",
            69 => "minecraft:sand",
            _ => "minecraft:stone",
        }));
    }

    #[test]
    fn ambient_sand_requires_four_matching_horizontal_columns_like_java_counter() {
        let origin = pos();
        assert!(should_play_ambient_sand_sound(
            origin,
            |p| {
                if p.y == 65 {
                    "minecraft:air"
                } else if p.y == 64 && (p.x.abs() == 8 || p.z.abs() == 8) {
                    "minecraft:sand"
                } else {
                    "minecraft:stone"
                }
            },
            |_| 64,
        ));
        assert!(!should_play_ambient_sand_sound(
            origin,
            |p| {
                if p.y == 65 {
                    "minecraft:air"
                } else if p.y == 64 && p.z.abs() == 8 {
                    "minecraft:sand"
                } else {
                    "minecraft:stone"
                }
            },
            |_| 64,
        ));
    }

    #[test]
    fn ambient_desert_tick_respects_random_gates_and_badlands_dead_bush_skip() {
        let origin = pos();
        let random = AmbientDesertRandom {
            idle_sand_roll: 0,
            dry_grass_roll: 0,
            dead_bush_roll: 0,
            badlands_dead_bush_skip_roll: 0,
        };
        let emissions = ambient_desert_sounds_for_tick(
            origin,
            random,
            |p| {
                if p == origin.relative(Direction::Up) || p.y == 65 {
                    "minecraft:air"
                } else if p.y == 63 || p.y == 62 || p.x.abs() == 8 || p.z.abs() == 8 {
                    "minecraft:sand"
                } else {
                    "minecraft:stone"
                }
            },
            |_| 64,
        );
        assert!(emissions.contains(&AmbientDesertSoundEmission::Local {
            sound: AmbientDesertSound::SandIdle,
            pos: origin
        }));
        assert!(emissions.contains(&AmbientDesertSoundEmission::Player {
            sound: AmbientDesertSound::DryGrass
        }));
        assert!(emissions.contains(&AmbientDesertSoundEmission::Local {
            sound: AmbientDesertSound::DeadBushIdle,
            pos: origin
        }));

        let skipped = ambient_desert_sounds_for_tick(
            origin,
            AmbientDesertRandom {
                badlands_dead_bush_skip_roll: 1,
                ..random
            },
            |p| {
                if p == origin.relative(Direction::Up) || p.y == 65 {
                    "minecraft:air"
                } else if p.y == 63 || p.y == 62 {
                    "minecraft:red_sand"
                } else if p.x.abs() == 8 || p.z.abs() == 8 {
                    "minecraft:sand"
                } else {
                    "minecraft:stone"
                }
            },
            |_| 64,
        );
        assert!(!skipped.contains(&AmbientDesertSoundEmission::Local {
            sound: AmbientDesertSound::DeadBushIdle,
            pos: origin
        }));
    }
}
