#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_update::{BlockPos, Direction};

pub const MAX_CROP_AGE: u8 = 7;
pub const MAX_GROWING_PLANT_AGE: u8 = 25;
pub const LEAVES_DECAY_DISTANCE: u8 = 7;
pub const SCULK_SENSOR_ACTIVE_TICKS: i32 = 30;
pub const SCULK_SENSOR_COOLDOWN_TICKS: i32 = 10;
pub const CORAL_DIE_TICKS: i32 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantFamily {
    Crop,
    Sapling,
    Fungus,
    Vine,
    Leaves,
    Moss,
    Sculk,
    Coral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantAction {
    Noop,
    Age(BlockStateModel),
    GrowTree {
        feature: String,
    },
    GrowHugeFungus {
        feature: String,
    },
    Spread {
        placements: Vec<(BlockPos, BlockStateModel)>,
    },
    Decay,
    Schedule {
        delay: i32,
    },
    Transform(BlockStateModel),
}

pub fn plant_family(state: &BlockStateModel) -> Option<PlantFamily> {
    let id = state.registry_id.as_str();
    if matches!(
        id,
        "minecraft:wheat"
            | "minecraft:carrots"
            | "minecraft:potatoes"
            | "minecraft:beetroots"
            | "minecraft:torchflower_crop"
            | "minecraft:nether_wart"
    ) {
        Some(PlantFamily::Crop)
    } else if id.ends_with("_sapling") || id == "minecraft:bamboo_sapling" {
        Some(PlantFamily::Sapling)
    } else if id.ends_with("_fungus") || id == "minecraft:mushroom" {
        Some(PlantFamily::Fungus)
    } else if id == "minecraft:vine" || id.ends_with("_vines") || id == "minecraft:glow_lichen" {
        Some(PlantFamily::Vine)
    } else if id.ends_with("_leaves") {
        Some(PlantFamily::Leaves)
    } else if id == "minecraft:moss_block"
        || id == "minecraft:pale_moss_block"
        || id == "minecraft:hanging_moss"
    {
        Some(PlantFamily::Moss)
    } else if id.starts_with("minecraft:sculk") {
        Some(PlantFamily::Sculk)
    } else if id.contains("coral") && !id.contains("dead_") {
        Some(PlantFamily::Coral)
    } else {
        None
    }
}

pub fn age_of(state: &BlockStateModel) -> u8 {
    state
        .property("age")
        .and_then(|age| age.parse::<u8>().ok())
        .unwrap_or(0)
}

pub fn with_age(state: &BlockStateModel, age: u8) -> BlockStateModel {
    state.clone().with_property("age", age.to_string())
}

pub fn random_tick_crop(
    state: &BlockStateModel,
    light_above: u8,
    growth_roll_succeeds: bool,
) -> PlantAction {
    if plant_family(state) != Some(PlantFamily::Crop) || light_above < 9 || !growth_roll_succeeds {
        return PlantAction::Noop;
    }
    let age = age_of(state);
    if age >= MAX_CROP_AGE {
        PlantAction::Noop
    } else {
        PlantAction::Age(with_age(state, age + 1))
    }
}

pub fn bonemeal(state: &BlockStateModel, pos: BlockPos) -> PlantAction {
    match plant_family(state) {
        Some(PlantFamily::Crop) => {
            let next = (age_of(state) + 2).min(MAX_CROP_AGE);
            PlantAction::Age(with_age(state, next))
        }
        Some(PlantFamily::Sapling) => {
            if state.property("stage") == Some("0") {
                PlantAction::Age(state.clone().with_property("stage", "1"))
            } else {
                PlantAction::GrowTree {
                    feature: tree_feature_for_sapling(&state.registry_id).to_string(),
                }
            }
        }
        Some(PlantFamily::Fungus) => PlantAction::GrowHugeFungus {
            feature: format!(
                "{}_huge_fungus",
                state.registry_id.trim_start_matches("minecraft:")
            ),
        },
        Some(PlantFamily::Vine) => PlantAction::Spread {
            placements: vec![(pos.relative(Direction::Up), state.clone())],
        },
        Some(PlantFamily::Moss) => moss_spread(pos),
        Some(PlantFamily::Sculk) => PlantAction::Spread {
            placements: vec![
                (
                    pos.relative(Direction::North),
                    BlockStateModel::new("minecraft:sculk_vein"),
                ),
                (
                    pos.relative(Direction::South),
                    BlockStateModel::new("minecraft:sculk"),
                ),
            ],
        },
        Some(PlantFamily::Coral) | Some(PlantFamily::Leaves) | None => PlantAction::Noop,
    }
}

pub fn growing_plant_tick(
    state: &BlockStateModel,
    pos: BlockPos,
    growth_direction: Direction,
    target_free: bool,
    roll_succeeds: bool,
) -> PlantAction {
    let age = age_of(state);
    if age >= MAX_GROWING_PLANT_AGE || !target_free || !roll_succeeds {
        return PlantAction::Noop;
    }
    PlantAction::Spread {
        placements: vec![(
            pos.relative(growth_direction),
            with_age(state, (age + 1).min(MAX_GROWING_PLANT_AGE)),
        )],
    }
}

pub fn leaves_decay(state: &BlockStateModel) -> PlantAction {
    let persistent = state.property("persistent") == Some("true");
    let distance = state
        .property("distance")
        .and_then(|distance| distance.parse::<u8>().ok())
        .unwrap_or(LEAVES_DECAY_DISTANCE);
    if !persistent && distance >= LEAVES_DECAY_DISTANCE {
        PlantAction::Decay
    } else {
        PlantAction::Noop
    }
}

pub fn update_leaf_distance(neighbor_distances: &[u8]) -> u8 {
    neighbor_distances
        .iter()
        .copied()
        .min()
        .map(|distance| (distance + 1).min(LEAVES_DECAY_DISTANCE))
        .unwrap_or(LEAVES_DECAY_DISTANCE)
}

pub fn coral_tick(state: &BlockStateModel, has_water: bool) -> PlantAction {
    if plant_family(state) != Some(PlantFamily::Coral) {
        return PlantAction::Noop;
    }
    if has_water || state.registry_id.starts_with("minecraft:dead_") {
        PlantAction::Noop
    } else {
        PlantAction::Transform(BlockStateModel::new(format!(
            "minecraft:dead_{}",
            state.registry_id.trim_start_matches("minecraft:")
        )))
    }
}

pub fn coral_schedule_if_dry(state: &BlockStateModel, has_water: bool) -> PlantAction {
    if plant_family(state) == Some(PlantFamily::Coral) && !has_water {
        PlantAction::Schedule {
            delay: CORAL_DIE_TICKS,
        }
    } else {
        PlantAction::Noop
    }
}

pub fn sculk_sensor_transition(
    state: &BlockStateModel,
    vibration_power: Option<u8>,
) -> PlantAction {
    match state.property("sculk_sensor_phase").unwrap_or("inactive") {
        "inactive" => match vibration_power {
            Some(power) => PlantAction::Transform(
                state
                    .clone()
                    .with_property("sculk_sensor_phase", "active")
                    .with_property("power", power.min(15).to_string()),
            ),
            None => PlantAction::Noop,
        },
        "active" => PlantAction::Transform(
            state
                .clone()
                .with_property("sculk_sensor_phase", "cooldown")
                .with_property("power", "0"),
        ),
        "cooldown" => PlantAction::Transform(
            state
                .clone()
                .with_property("sculk_sensor_phase", "inactive")
                .with_property("power", "0"),
        ),
        _ => PlantAction::Noop,
    }
}

fn moss_spread(pos: BlockPos) -> PlantAction {
    PlantAction::Spread {
        placements: vec![
            (
                pos.relative(Direction::North),
                BlockStateModel::new("minecraft:moss_carpet"),
            ),
            (
                pos.relative(Direction::East),
                BlockStateModel::new("minecraft:azalea"),
            ),
            (
                pos.relative(Direction::South),
                BlockStateModel::new("minecraft:flowering_azalea"),
            ),
            (
                pos.relative(Direction::West),
                BlockStateModel::new("minecraft:moss_block"),
            ),
        ],
    }
}

fn tree_feature_for_sapling(registry_id: &str) -> &'static str {
    match registry_id {
        "minecraft:oak_sapling" => "minecraft:oak",
        "minecraft:spruce_sapling" => "minecraft:spruce",
        "minecraft:birch_sapling" => "minecraft:birch",
        "minecraft:jungle_sapling" => "minecraft:jungle",
        "minecraft:acacia_sapling" => "minecraft:acacia",
        "minecraft:dark_oak_sapling" => "minecraft:dark_oak",
        "minecraft:cherry_sapling" => "minecraft:cherry",
        "minecraft:mangrove_propagule" => "minecraft:mangrove",
        _ => "minecraft:tree",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        age_of, bonemeal, coral_schedule_if_dry, coral_tick, growing_plant_tick, leaves_decay,
        plant_family, random_tick_crop, sculk_sensor_transition, update_leaf_distance, with_age,
        PlantAction, PlantFamily, CORAL_DIE_TICKS, LEAVES_DECAY_DISTANCE, MAX_CROP_AGE,
    };
    use crate::block_behavior::BlockStateModel;
    use crate::block_update::{BlockPos, Direction};

    #[test]
    fn classifies_required_plant_families() {
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:wheat")),
            Some(PlantFamily::Crop)
        );
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:oak_sapling")),
            Some(PlantFamily::Sapling)
        );
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:crimson_fungus")),
            Some(PlantFamily::Fungus)
        );
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:vine")),
            Some(PlantFamily::Vine)
        );
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:oak_leaves")),
            Some(PlantFamily::Leaves)
        );
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:moss_block")),
            Some(PlantFamily::Moss)
        );
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:sculk")),
            Some(PlantFamily::Sculk)
        );
        assert_eq!(
            plant_family(&BlockStateModel::new("minecraft:tube_coral")),
            Some(PlantFamily::Coral)
        );
    }

    #[test]
    fn crop_random_tick_and_bonemeal_advance_age_with_caps() {
        let wheat = BlockStateModel::new("minecraft:wheat").with_property("age", "2");
        assert_eq!(
            random_tick_crop(&wheat, 9, true),
            PlantAction::Age(with_age(&wheat, 3))
        );
        assert_eq!(random_tick_crop(&wheat, 8, true), PlantAction::Noop);
        assert_eq!(
            bonemeal(&wheat, BlockPos { x: 0, y: 64, z: 0 }),
            PlantAction::Age(with_age(&wheat, 4))
        );
        assert_eq!(
            bonemeal(&with_age(&wheat, 6), BlockPos { x: 0, y: 64, z: 0 }),
            PlantAction::Age(with_age(&wheat, MAX_CROP_AGE))
        );
        assert_eq!(age_of(&with_age(&wheat, 5)), 5);
    }

    #[test]
    fn saplings_stage_before_tree_generation() {
        let sapling = BlockStateModel::new("minecraft:oak_sapling").with_property("stage", "0");
        assert_eq!(
            bonemeal(&sapling, BlockPos { x: 0, y: 64, z: 0 }),
            PlantAction::Age(sapling.clone().with_property("stage", "1"))
        );
        assert_eq!(
            bonemeal(
                &sapling.with_property("stage", "1"),
                BlockPos { x: 0, y: 64, z: 0 }
            ),
            PlantAction::GrowTree {
                feature: "minecraft:oak".to_string()
            }
        );
    }

    #[test]
    fn fungi_vines_moss_and_sculk_use_spread_or_feature_actions() {
        let pos = BlockPos { x: 1, y: 64, z: 1 };
        assert_eq!(
            bonemeal(&BlockStateModel::new("minecraft:warped_fungus"), pos),
            PlantAction::GrowHugeFungus {
                feature: "warped_fungus_huge_fungus".to_string()
            }
        );
        assert!(matches!(
            bonemeal(&BlockStateModel::new("minecraft:vine"), pos),
            PlantAction::Spread { placements } if placements[0].0 == pos.relative(Direction::Up)
        ));
        assert!(matches!(
            bonemeal(&BlockStateModel::new("minecraft:moss_block"), pos),
            PlantAction::Spread { placements } if placements.len() == 4
        ));
        assert!(matches!(
            bonemeal(&BlockStateModel::new("minecraft:sculk"), pos),
            PlantAction::Spread { placements } if placements.iter().any(|(_, state)| state.registry_id == "minecraft:sculk_vein")
        ));
    }

    #[test]
    fn growing_plant_heads_extend_until_max_age() {
        let vine = BlockStateModel::new("minecraft:cave_vines").with_property("age", "3");
        let pos = BlockPos { x: 0, y: 70, z: 0 };
        assert!(matches!(
            growing_plant_tick(&vine, pos, Direction::Down, true, true),
            PlantAction::Spread { placements } if placements[0].0 == pos.relative(Direction::Down)
                && placements[0].1.property("age") == Some("4")
        ));
        assert_eq!(
            growing_plant_tick(
                &vine.with_property("age", "25"),
                pos,
                Direction::Down,
                true,
                true
            ),
            PlantAction::Noop
        );
    }

    #[test]
    fn leaves_decay_depends_on_persistence_and_log_distance() {
        let leaves = BlockStateModel::new("minecraft:oak_leaves")
            .with_property("persistent", "false")
            .with_property("distance", LEAVES_DECAY_DISTANCE.to_string());
        assert_eq!(leaves_decay(&leaves), PlantAction::Decay);
        assert_eq!(
            leaves_decay(&leaves.with_property("persistent", "true")),
            PlantAction::Noop
        );
        assert_eq!(update_leaf_distance(&[1, 3, 6]), 2);
        assert_eq!(update_leaf_distance(&[]), LEAVES_DECAY_DISTANCE);
    }

    #[test]
    fn coral_schedules_dry_death_and_transforms_to_dead_variant() {
        let coral = BlockStateModel::new("minecraft:tube_coral");
        assert_eq!(
            coral_schedule_if_dry(&coral, false),
            PlantAction::Schedule {
                delay: CORAL_DIE_TICKS
            }
        );
        assert_eq!(
            coral_tick(&coral, false),
            PlantAction::Transform(BlockStateModel::new("minecraft:dead_tube_coral"))
        );
        assert_eq!(coral_tick(&coral, true), PlantAction::Noop);
    }

    #[test]
    fn sculk_sensor_cycles_inactive_active_cooldown() {
        let sensor = BlockStateModel::new("minecraft:sculk_sensor")
            .with_property("sculk_sensor_phase", "inactive")
            .with_property("power", "0");
        assert_eq!(
            sculk_sensor_transition(&sensor, Some(21)),
            PlantAction::Transform(
                sensor
                    .clone()
                    .with_property("sculk_sensor_phase", "active")
                    .with_property("power", "15")
            )
        );
        let active = sensor
            .clone()
            .with_property("sculk_sensor_phase", "active")
            .with_property("power", "12");
        assert_eq!(
            sculk_sensor_transition(&active, None),
            PlantAction::Transform(
                active
                    .clone()
                    .with_property("sculk_sensor_phase", "cooldown")
                    .with_property("power", "0")
            )
        );
        let cooldown = active.with_property("sculk_sensor_phase", "cooldown");
        assert_eq!(
            sculk_sensor_transition(&cooldown, None),
            PlantAction::Transform(
                cooldown
                    .with_property("sculk_sensor_phase", "inactive")
                    .with_property("power", "0")
            )
        );
    }
}
