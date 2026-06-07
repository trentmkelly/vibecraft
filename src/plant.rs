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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeGrowerModel {
    pub name: &'static str,
    pub secondary_chance: f32,
    pub mega_tree: Option<&'static str>,
    pub secondary_mega_tree: Option<&'static str>,
    pub tree: Option<&'static str>,
    pub secondary_tree: Option<&'static str>,
    pub flowers: Option<&'static str>,
    pub secondary_flowers: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeGrowPlan {
    Single {
        feature: &'static str,
        clear_pos: BlockPos,
        restore_pos: BlockPos,
    },
    Mega {
        feature: &'static str,
        origin: BlockPos,
        clear_positions: [BlockPos; 4],
    },
}

pub const TREE_GROWERS: [TreeGrowerModel; 10] = [
    TreeGrowerModel {
        name: "oak",
        secondary_chance: 0.1,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:oak"),
        secondary_tree: Some("minecraft:fancy_oak"),
        flowers: Some("minecraft:oak_bees_005"),
        secondary_flowers: Some("minecraft:fancy_oak_bees_005"),
    },
    TreeGrowerModel {
        name: "spruce",
        secondary_chance: 0.5,
        mega_tree: Some("minecraft:mega_spruce"),
        secondary_mega_tree: Some("minecraft:mega_pine"),
        tree: Some("minecraft:spruce"),
        secondary_tree: None,
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrowerModel {
        name: "mangrove",
        secondary_chance: 0.85,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:mangrove"),
        secondary_tree: Some("minecraft:tall_mangrove"),
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrowerModel::simple("azalea", None, Some("minecraft:azalea_tree"), None),
    TreeGrowerModel::simple(
        "birch",
        None,
        Some("minecraft:birch"),
        Some("minecraft:birch_bees_005"),
    ),
    TreeGrowerModel::simple(
        "jungle",
        Some("minecraft:mega_jungle_tree"),
        Some("minecraft:jungle_tree_no_vine"),
        None,
    ),
    TreeGrowerModel::simple("acacia", None, Some("minecraft:acacia"), None),
    TreeGrowerModel::simple(
        "cherry",
        None,
        Some("minecraft:cherry"),
        Some("minecraft:cherry_bees_005"),
    ),
    TreeGrowerModel::simple("dark_oak", Some("minecraft:dark_oak"), None, None),
    TreeGrowerModel::simple("pale_oak", Some("minecraft:pale_oak_bonemeal"), None, None),
];

impl TreeGrowerModel {
    pub const fn simple(
        name: &'static str,
        mega_tree: Option<&'static str>,
        tree: Option<&'static str>,
        flowers: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            secondary_chance: 0.0,
            mega_tree,
            secondary_mega_tree: None,
            tree,
            secondary_tree: None,
            flowers,
            secondary_flowers: None,
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        TREE_GROWERS
            .iter()
            .copied()
            .find(|grower| grower.name == name)
    }

    pub fn configured_feature(self, random_float: f32, has_flowers: bool) -> Option<&'static str> {
        if random_float < self.secondary_chance {
            if has_flowers {
                if let Some(feature) = self.secondary_flowers {
                    return Some(feature);
                }
            }
            if let Some(feature) = self.secondary_tree {
                return Some(feature);
            }
        }
        if has_flowers {
            self.flowers.or(self.tree)
        } else {
            self.tree
        }
    }

    pub fn configured_mega_feature(self, random_float: f32) -> Option<&'static str> {
        if random_float < self.secondary_chance {
            self.secondary_mega_tree.or(self.mega_tree)
        } else {
            self.mega_tree
        }
    }

    pub fn grow_plan(
        self,
        pos: BlockPos,
        state: &BlockStateModel,
        mega_random_float: f32,
        tree_random_float: f32,
        has_flowers: bool,
        mut is_sapling_at: impl FnMut(BlockPos, &str) -> bool,
    ) -> Option<TreeGrowPlan> {
        // TODO(tree-grower-live-feature): execute these plans through Java-shaped
        // ConfiguredFeature.place once the worldgen feature runtime is wired into live block ticks.
        if let Some(feature) = self.configured_mega_feature(mega_random_float) {
            for dx in [0, -1] {
                for dz in [0, -1] {
                    let origin = offset_pos(pos, dx, 0, dz);
                    let clear_positions = two_by_two_positions(origin);
                    if clear_positions
                        .iter()
                        .all(|candidate| is_sapling_at(*candidate, &state.registry_id))
                    {
                        return Some(TreeGrowPlan::Mega {
                            feature,
                            origin,
                            clear_positions,
                        });
                    }
                }
            }
        }

        self.configured_feature(tree_random_float, has_flowers)
            .map(|feature| TreeGrowPlan::Single {
                feature,
                clear_pos: pos,
                restore_pos: pos,
            })
    }

    pub fn minimum_height(self) -> Option<i32> {
        self.tree.and_then(tree_feature_base_height)
    }
}

pub fn tree_grower_for_sapling(registry_id: &str) -> Option<TreeGrowerModel> {
    let name = match registry_id {
        "minecraft:oak_sapling" => "oak",
        "minecraft:spruce_sapling" => "spruce",
        "minecraft:birch_sapling" => "birch",
        "minecraft:jungle_sapling" => "jungle",
        "minecraft:acacia_sapling" => "acacia",
        "minecraft:dark_oak_sapling" => "dark_oak",
        "minecraft:pale_oak_sapling" => "pale_oak",
        "minecraft:cherry_sapling" => "cherry",
        "minecraft:mangrove_propagule" => "mangrove",
        "minecraft:azalea" | "minecraft:flowering_azalea" => "azalea",
        _ => return None,
    };
    TreeGrowerModel::by_name(name)
}

pub fn has_tree_grower_flowers(
    pos: BlockPos,
    mut is_flower_at: impl FnMut(BlockPos) -> bool,
) -> bool {
    for x in (pos.x - 2)..=(pos.x + 2) {
        for y in (pos.y - 1)..=(pos.y + 1) {
            for z in (pos.z - 2)..=(pos.z + 2) {
                if is_flower_at(BlockPos { x, y, z }) {
                    return true;
                }
            }
        }
    }
    false
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
    tree_grower_for_sapling(registry_id)
        .and_then(|grower| grower.configured_feature(1.0, false))
        .unwrap_or("minecraft:tree")
}

fn offset_pos(pos: BlockPos, x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos {
        x: pos.x + x,
        y: pos.y + y,
        z: pos.z + z,
    }
}

fn two_by_two_positions(origin: BlockPos) -> [BlockPos; 4] {
    [
        origin,
        offset_pos(origin, 1, 0, 0),
        offset_pos(origin, 0, 0, 1),
        offset_pos(origin, 1, 0, 1),
    ]
}

fn tree_feature_base_height(feature: &str) -> Option<i32> {
    match feature {
        "minecraft:oak" | "minecraft:jungle_tree_no_vine" => Some(4),
        "minecraft:birch" | "minecraft:acacia" | "minecraft:spruce" => Some(5),
        "minecraft:cherry" => Some(7),
        "minecraft:azalea_tree" => Some(4),
        "minecraft:mangrove" => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        age_of, bonemeal, coral_schedule_if_dry, coral_tick, growing_plant_tick,
        has_tree_grower_flowers, leaves_decay, plant_family, random_tick_crop,
        sculk_sensor_transition, tree_grower_for_sapling, update_leaf_distance, with_age,
        PlantAction, PlantFamily, TreeGrowPlan, TreeGrowerModel, CORAL_DIE_TICKS,
        LEAVES_DECAY_DISTANCE, MAX_CROP_AGE, TREE_GROWERS,
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
    fn tree_grower_registry_and_feature_selection_match_java() {
        assert_eq!(TREE_GROWERS.len(), 10);
        let oak = TreeGrowerModel::by_name("oak").unwrap();
        assert_eq!(oak.configured_feature(0.2, false), Some("minecraft:oak"));
        assert_eq!(
            oak.configured_feature(0.05, false),
            Some("minecraft:fancy_oak")
        );
        assert_eq!(
            oak.configured_feature(0.2, true),
            Some("minecraft:oak_bees_005")
        );
        assert_eq!(
            oak.configured_feature(0.05, true),
            Some("minecraft:fancy_oak_bees_005")
        );

        let spruce = TreeGrowerModel::by_name("spruce").unwrap();
        assert_eq!(
            spruce.configured_mega_feature(0.6),
            Some("minecraft:mega_spruce")
        );
        assert_eq!(
            spruce.configured_mega_feature(0.4),
            Some("minecraft:mega_pine")
        );

        let mangrove = TreeGrowerModel::by_name("mangrove").unwrap();
        assert_eq!(
            mangrove.configured_feature(0.9, false),
            Some("minecraft:mangrove")
        );
        assert_eq!(
            mangrove.configured_feature(0.8, false),
            Some("minecraft:tall_mangrove")
        );
    }

    #[test]
    fn tree_grower_sapling_mapping_minimum_height_and_flower_scan_match_java() {
        assert_eq!(
            tree_grower_for_sapling("minecraft:pale_oak_sapling")
                .unwrap()
                .configured_mega_feature(1.0),
            Some("minecraft:pale_oak_bonemeal")
        );
        assert_eq!(
            tree_grower_for_sapling("minecraft:flowering_azalea")
                .unwrap()
                .minimum_height(),
            Some(4)
        );
        assert_eq!(
            TreeGrowerModel::by_name("cherry").unwrap().minimum_height(),
            Some(7)
        );
        assert_eq!(
            TreeGrowerModel::by_name("dark_oak")
                .unwrap()
                .minimum_height(),
            None
        );

        let pos = BlockPos {
            x: 10,
            y: 64,
            z: -4,
        };
        assert!(has_tree_grower_flowers(pos, |candidate| candidate
            == BlockPos { x: 8, y: 63, z: -6 }));
        assert!(has_tree_grower_flowers(pos, |candidate| candidate
            == BlockPos {
                x: 12,
                y: 65,
                z: -2
            }));
        assert!(!has_tree_grower_flowers(pos, |candidate| candidate
            == BlockPos {
                x: 13,
                y: 64,
                z: -4
            }));
    }

    #[test]
    fn tree_grow_plan_scans_mega_offsets_and_falls_back_to_single_like_java() {
        let pos = BlockPos { x: 5, y: 70, z: 5 };
        let state = BlockStateModel::new("minecraft:spruce_sapling");
        let spruce = tree_grower_for_sapling(&state.registry_id).unwrap();
        let plan = spruce
            .grow_plan(pos, &state, 0.25, 1.0, false, |candidate, sapling| {
                sapling == "minecraft:spruce_sapling"
                    && matches!(
                        (candidate.x, candidate.z),
                        (4, 4) | (5, 4) | (4, 5) | (5, 5)
                    )
            })
            .unwrap();
        assert_eq!(
            plan,
            TreeGrowPlan::Mega {
                feature: "minecraft:mega_pine",
                origin: BlockPos { x: 4, y: 70, z: 4 },
                clear_positions: [
                    BlockPos { x: 4, y: 70, z: 4 },
                    BlockPos { x: 5, y: 70, z: 4 },
                    BlockPos { x: 4, y: 70, z: 5 },
                    BlockPos { x: 5, y: 70, z: 5 },
                ],
            }
        );

        assert_eq!(
            spruce.grow_plan(pos, &state, 0.75, 1.0, false, |candidate, _| candidate
                == pos),
            Some(TreeGrowPlan::Single {
                feature: "minecraft:spruce",
                clear_pos: pos,
                restore_pos: pos,
            })
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
