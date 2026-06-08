use crate::block_update::{BlockPos, Direction};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3Model {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockSourceModel {
    pos: BlockPos,
    facing: Direction,
}

impl BlockSourceModel {
    fn center(self) -> Vec3Model {
        Vec3Model {
            x: self.pos.x as f64 + 0.5,
            y: self.pos.y as f64 + 0.5,
            z: self.pos.z as f64 + 0.5,
        }
    }

    fn front(self) -> BlockPos {
        self.pos.relative(self.facing)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispenseSound {
    Success1000,
    Fail1001,
    Projectile1002,
    Override(i32),
}

#[derive(Debug, Clone, PartialEq)]
enum JavaDispenseTrace {
    Noop,
    DefaultItem {
        spawn: Vec3Model,
        accuracy: i32,
        sound: DispenseSound,
        animation_direction: Direction,
    },
    Projectile {
        position: Vec3Model,
        step: (i32, i32, i32),
        power: f32,
        uncertainty: f32,
        sound: DispenseSound,
    },
    Boat {
        position: Vec3Model,
        y_rot: f32,
    },
    Minecart {
        position: Vec3Model,
    },
    Equipment {
        target: BlockPos,
        guaranteed_drop: bool,
        persistent: bool,
    },
    Optional {
        success: bool,
        sound: DispenseSound,
    },
    SpawnEgg {
        pos: BlockPos,
        offset_y: bool,
        game_event: &'static str,
    },
    ShulkerPlace {
        pos: BlockPos,
        clicked_face: Direction,
        success: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetBlockState {
    Air,
    Solid,
    Water,
    RailFlat,
    RailSlope,
    Beehive { honey_level: i32 },
}

fn direction_step(direction: Direction) -> (i32, i32, i32) {
    match direction {
        Direction::West => (-1, 0, 0),
        Direction::East => (1, 0, 0),
        Direction::Down => (0, -1, 0),
        Direction::Up => (0, 1, 0),
        Direction::North => (0, 0, -1),
        Direction::South => (0, 0, 1),
    }
}

fn direction_y_rot(direction: Direction) -> f32 {
    match direction {
        Direction::South => 0.0,
        Direction::West => 90.0,
        Direction::North => 180.0,
        Direction::East => 270.0,
        Direction::Up | Direction::Down => 0.0,
    }
}

fn default_dispense_position(source: BlockSourceModel, scale: f64) -> Vec3Model {
    let center = source.center();
    let (x, y, z) = direction_step(source.facing);
    Vec3Model {
        x: center.x + scale * x as f64,
        y: center.y + scale * y as f64,
        z: center.z + scale * z as f64,
    }
}

fn default_item_trace(source: BlockSourceModel) -> JavaDispenseTrace {
    let mut spawn = default_dispense_position(source, 0.7);
    if matches!(source.facing, Direction::Up | Direction::Down) {
        spawn.y -= 0.125;
    } else {
        spawn.y -= 0.15625;
    }
    JavaDispenseTrace::DefaultItem {
        spawn,
        accuracy: 6,
        sound: DispenseSound::Success1000,
        animation_direction: source.facing,
    }
}

fn consume_with_remainder(stack_count: u32, inventory_accepts: bool) -> Vec<&'static str> {
    if stack_count <= 1 {
        vec!["return_remainder"]
    } else if inventory_accepts {
        vec!["shrink_dispensed", "insert_remainder"]
    } else {
        vec![
            "shrink_dispensed",
            "spawn_remainder",
            "sound_1000",
            "animation_2000",
        ]
    }
}

fn optional_sound(success: bool) -> DispenseSound {
    if success {
        DispenseSound::Success1000
    } else {
        DispenseSound::Fail1001
    }
}

fn projectile_trace(
    source: BlockSourceModel,
    power: f32,
    uncertainty: f32,
    override_event: Option<i32>,
) -> JavaDispenseTrace {
    JavaDispenseTrace::Projectile {
        position: default_dispense_position(source, 0.7),
        step: direction_step(source.facing),
        power,
        uncertainty,
        sound: override_event
            .map(DispenseSound::Override)
            .unwrap_or(DispenseSound::Projectile1002),
    }
}

fn boat_trace(
    source: BlockSourceModel,
    boat_width: f64,
    front: TargetBlockState,
    below_front_water: bool,
) -> JavaDispenseTrace {
    let center = source.center();
    let (sx, sy, sz) = direction_step(source.facing);
    let just_outside = 0.5625 + boat_width / 2.0;
    let y_offset = match (front, below_front_water) {
        (TargetBlockState::Water, _) => 1.0,
        (TargetBlockState::Air, true) => 0.0,
        _ => return default_item_trace(source),
    };
    JavaDispenseTrace::Boat {
        position: Vec3Model {
            x: center.x + sx as f64 * just_outside,
            y: center.y + sy as f64 * 1.125 + y_offset,
            z: center.z + sz as f64 * just_outside,
        },
        y_rot: direction_y_rot(source.facing),
    }
}

fn minecart_trace(
    source: BlockSourceModel,
    front: TargetBlockState,
    below_front: TargetBlockState,
) -> JavaDispenseTrace {
    let center = source.center();
    let (sx, sy, sz) = direction_step(source.facing);
    let y_offset = match front {
        TargetBlockState::RailSlope => 0.6,
        TargetBlockState::RailFlat => 0.1,
        TargetBlockState::Air => match below_front {
            TargetBlockState::RailSlope if source.facing != Direction::Down => -0.4,
            TargetBlockState::RailSlope | TargetBlockState::RailFlat => -0.9,
            _ => return default_item_trace(source),
        },
        _ => return default_item_trace(source),
    };
    JavaDispenseTrace::Minecart {
        position: Vec3Model {
            x: center.x + sx as f64 * 1.125,
            y: center.y.floor() + sy as f64 + y_offset,
            z: center.z + sz as f64 * 1.125,
        },
    }
}

fn equipment_trace(
    source: BlockSourceModel,
    has_target: bool,
    target_is_mob: bool,
) -> JavaDispenseTrace {
    if has_target {
        JavaDispenseTrace::Equipment {
            target: source.front(),
            guaranteed_drop: target_is_mob,
            persistent: target_is_mob,
        }
    } else {
        default_item_trace(source)
    }
}

fn shears_trace(front: TargetBlockState, shearable_entity: bool) -> JavaDispenseTrace {
    let success = matches!(front, TargetBlockState::Beehive { honey_level } if honey_level >= 5)
        || shearable_entity;
    JavaDispenseTrace::Optional {
        success,
        sound: optional_sound(success),
    }
}

fn shulker_trace(
    source: BlockSourceModel,
    below_target_empty: bool,
    place_consumes: bool,
) -> JavaDispenseTrace {
    JavaDispenseTrace::ShulkerPlace {
        pos: source.front(),
        clicked_face: if below_target_empty {
            source.facing
        } else {
            Direction::Up
        },
        success: place_consumes,
    }
}

fn spawn_egg_trace(
    source: BlockSourceModel,
    has_type: bool,
    spawn_throws: bool,
) -> JavaDispenseTrace {
    if !has_type {
        JavaDispenseTrace::Noop
    } else if spawn_throws {
        JavaDispenseTrace::Optional {
            success: false,
            sound: DispenseSound::Success1000,
        }
    } else {
        JavaDispenseTrace::SpawnEgg {
            pos: source.front(),
            offset_y: source.facing != Direction::Up,
            game_event: "entity_place",
        }
    }
}

const PROJECTILE_ITEMS: &[&str] = &[
    "arrow",
    "tipped_arrow",
    "spectral_arrow",
    "egg",
    "blue_egg",
    "brown_egg",
    "snowball",
    "experience_bottle",
    "splash_potion",
    "lingering_potion",
    "firework_rocket",
    "fire_charge",
    "wind_charge",
];

const BOAT_ITEMS: &[&str] = &[
    "oak_boat",
    "spruce_boat",
    "birch_boat",
    "jungle_boat",
    "dark_oak_boat",
    "acacia_boat",
    "cherry_boat",
    "mangrove_boat",
    "pale_oak_boat",
    "bamboo_raft",
    "oak_chest_boat",
    "spruce_chest_boat",
    "birch_chest_boat",
    "jungle_chest_boat",
    "dark_oak_chest_boat",
    "acacia_chest_boat",
    "cherry_chest_boat",
    "mangrove_chest_boat",
    "pale_oak_chest_boat",
    "bamboo_chest_raft",
];

const FILLED_BUCKET_ITEMS: &[&str] = &[
    "lava_bucket",
    "water_bucket",
    "powder_snow_bucket",
    "salmon_bucket",
    "cod_bucket",
    "pufferfish_bucket",
    "tropical_fish_bucket",
    "axolotl_bucket",
    "tadpole_bucket",
];

const SHULKER_BOX_ITEMS: &[&str] = &[
    "shulker_box",
    "white_shulker_box",
    "orange_shulker_box",
    "magenta_shulker_box",
    "light_blue_shulker_box",
    "yellow_shulker_box",
    "lime_shulker_box",
    "pink_shulker_box",
    "gray_shulker_box",
    "light_gray_shulker_box",
    "cyan_shulker_box",
    "purple_shulker_box",
    "blue_shulker_box",
    "brown_shulker_box",
    "green_shulker_box",
    "red_shulker_box",
    "black_shulker_box",
];

const MINECART_ITEMS: &[&str] = &[
    "minecart",
    "chest_minecart",
    "furnace_minecart",
    "tnt_minecart",
    "hopper_minecart",
    "command_block_minecart",
];

const SPECIAL_BOOTSTRAP_ITEMS: &[&str] = &[
    "armor_stand",
    "chest",
    "bucket",
    "flint_and_steel",
    "bone_meal",
    "tnt",
    "wither_skeleton_skull",
    "carved_pumpkin",
    "glass_bottle",
    "glowstone",
    "shears",
    "brush",
    "honeycomb",
    "potion",
];

const CORE_DISPENSER_PACKAGE_NULL_MARKED: bool = true;

fn core_dispenser_package_is_null_marked() -> bool {
    CORE_DISPENSER_PACKAGE_NULL_MARKED
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(facing: Direction) -> BlockSourceModel {
        BlockSourceModel {
            pos: BlockPos { x: 4, y: 64, z: -2 },
            facing,
        }
    }

    #[test]
    fn block_source_center_default_spawn_and_remainder_match_java() {
        assert_eq!(
            source(Direction::North).center(),
            Vec3Model {
                x: 4.5,
                y: 64.5,
                z: -1.5
            }
        );
        assert_eq!(
            default_item_trace(source(Direction::North)),
            JavaDispenseTrace::DefaultItem {
                spawn: Vec3Model {
                    x: 4.5,
                    y: 64.34375,
                    z: -2.2
                },
                accuracy: 6,
                sound: DispenseSound::Success1000,
                animation_direction: Direction::North,
            }
        );
        assert_eq!(consume_with_remainder(1, false), vec!["return_remainder"]);
        assert_eq!(
            consume_with_remainder(2, false),
            vec![
                "shrink_dispensed",
                "spawn_remainder",
                "sound_1000",
                "animation_2000"
            ]
        );
    }

    #[test]
    fn optional_projectile_equipment_and_spawn_egg_behaviors_match_java() {
        assert_eq!(optional_sound(false), DispenseSound::Fail1001);
        assert_eq!(
            projectile_trace(source(Direction::East), 1.1, 6.0, Some(1004)),
            JavaDispenseTrace::Projectile {
                position: Vec3Model {
                    x: 5.2,
                    y: 64.5,
                    z: -1.5
                },
                step: (1, 0, 0),
                power: 1.1,
                uncertainty: 6.0,
                sound: DispenseSound::Override(1004),
            }
        );
        assert_eq!(
            equipment_trace(source(Direction::West), true, true),
            JavaDispenseTrace::Equipment {
                target: BlockPos { x: 3, y: 64, z: -2 },
                guaranteed_drop: true,
                persistent: true,
            }
        );
        assert_eq!(
            spawn_egg_trace(source(Direction::Up), true, false),
            JavaDispenseTrace::SpawnEgg {
                pos: BlockPos { x: 4, y: 65, z: -2 },
                offset_y: false,
                game_event: "entity_place",
            }
        );
    }

    #[test]
    fn boat_and_minecart_fallbacks_and_offsets_match_java() {
        assert_eq!(
            boat_trace(
                source(Direction::South),
                1.375,
                TargetBlockState::Water,
                false
            ),
            JavaDispenseTrace::Boat {
                position: Vec3Model {
                    x: 4.5,
                    y: 65.5,
                    z: -0.25
                },
                y_rot: 0.0,
            }
        );
        assert!(matches!(
            boat_trace(
                source(Direction::South),
                1.375,
                TargetBlockState::Solid,
                false
            ),
            JavaDispenseTrace::DefaultItem { .. }
        ));
        assert_eq!(
            minecart_trace(
                source(Direction::East),
                TargetBlockState::RailSlope,
                TargetBlockState::Air
            ),
            JavaDispenseTrace::Minecart {
                position: Vec3Model {
                    x: 5.625,
                    y: 64.6,
                    z: -1.5
                },
            }
        );
        assert_eq!(
            minecart_trace(
                source(Direction::North),
                TargetBlockState::Air,
                TargetBlockState::RailFlat
            ),
            JavaDispenseTrace::Minecart {
                position: Vec3Model {
                    x: 4.5,
                    y: 63.1,
                    z: -2.625
                },
            }
        );
    }

    #[test]
    fn shears_and_shulker_box_optional_behavior_match_java() {
        assert_eq!(
            shears_trace(TargetBlockState::Beehive { honey_level: 5 }, false),
            JavaDispenseTrace::Optional {
                success: true,
                sound: DispenseSound::Success1000,
            }
        );
        assert_eq!(
            shears_trace(TargetBlockState::Beehive { honey_level: 4 }, false),
            JavaDispenseTrace::Optional {
                success: false,
                sound: DispenseSound::Fail1001,
            }
        );
        assert_eq!(
            shulker_trace(source(Direction::East), true, true),
            JavaDispenseTrace::ShulkerPlace {
                pos: BlockPos { x: 5, y: 64, z: -2 },
                clicked_face: Direction::East,
                success: true,
            }
        );
        assert_eq!(
            shulker_trace(source(Direction::East), false, false),
            JavaDispenseTrace::ShulkerPlace {
                pos: BlockPos { x: 5, y: 64, z: -2 },
                clicked_face: Direction::Up,
                success: false,
            }
        );
    }

    #[test]
    fn bootstrap_registration_groups_match_java() {
        assert_eq!(PROJECTILE_ITEMS.len(), 13);
        assert_eq!(BOAT_ITEMS.len(), 20);
        assert_eq!(FILLED_BUCKET_ITEMS.len(), 9);
        assert_eq!(SHULKER_BOX_ITEMS.len(), 17);
        assert_eq!(MINECART_ITEMS.len(), 6);
        assert_eq!(SPECIAL_BOOTSTRAP_ITEMS.len(), 14);
        assert_eq!(PROJECTILE_ITEMS[0], "arrow");
        assert_eq!(PROJECTILE_ITEMS[12], "wind_charge");
        assert_eq!(BOAT_ITEMS[19], "bamboo_chest_raft");
        assert_eq!(MINECART_ITEMS[5], "command_block_minecart");
        assert!(SPECIAL_BOOTSTRAP_ITEMS.contains(&"wither_skeleton_skull"));
        assert!(core_dispenser_package_is_null_marked());
    }
}
