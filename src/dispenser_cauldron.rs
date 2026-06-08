#![allow(dead_code)]

use crate::block_update::{BlockPos, Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispensedItemKind {
    DefaultItem,
    Equippable,
    SpawnEggWithEntityData,
    Projectile,
    Boat,
    Minecart,
    ContainerBucket,
    EmptyBucket,
    FlintAndSteel,
    BoneMeal,
    Tnt,
    Shears,
    ShulkerBox,
    SaddleOrChest,
    Glowstone,
    GlassBottle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispenseBehaviorKind {
    DefaultDrop,
    Equipment,
    SpawnEgg,
    Projectile,
    Boat,
    Minecart,
    EmptyContainer,
    FillContainer,
    IgniteOrCharge,
    BoneMeal,
    PrimeTnt,
    Shears,
    PlaceShulkerBox,
    EquipAnimal,
    FillBottle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DispenseAction {
    Noop,
    LevelEvent(u16),
    ScheduleTick {
        delay: i32,
        triggered: bool,
    },
    SpawnItem {
        item: &'static str,
        count: u32,
        pos: DispensePosition,
        velocity: DispenseVelocity,
    },
    SpawnEntity {
        entity: &'static str,
        pos: DispensePosition,
        yaw_degrees: f32,
    },
    UseOnBlock {
        target: BlockPos,
        behavior: DispenseBehaviorKind,
    },
    ChangeStack {
        consumed: u32,
        remainder: Option<&'static str>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DispensePosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DispenseVelocity {
    pub x_base: f64,
    pub y_base: f64,
    pub z_base: f64,
    pub accuracy: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronContent {
    Empty,
    Water { level: u8 },
    Lava,
    PowderSnow { level: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronItem {
    WaterBucket,
    LavaBucket,
    PowderSnowBucket,
    Bucket,
    WaterPotion,
    NonWaterPotion,
    GlassBottle,
    DyedArmor,
    UndyedArmor,
    PatternedBanner,
    PlainBanner,
    DyedShulkerBox,
    PlainShulkerBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronDispatcherKind {
    Empty,
    Water,
    Lava,
    PowderSnow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronInteractionKind {
    Default,
    FillWater,
    FillLava,
    FillPowderSnow,
    EmptyWaterPotion,
    FillBucketWater,
    FillGlassBottle,
    AddWaterPotion,
    CleanDyedItem,
    CleanBanner,
    CleanShulkerBox,
    FillBucketLava,
    FillBucketPowderSnow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronItemKey {
    WaterBucket,
    LavaBucket,
    PowderSnowBucket,
    Bucket,
    Potion,
    GlassBottle,
    Banner(&'static str),
    ShulkerBox(&'static str),
    LeatherArmor,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronItemTag {
    CanRemoveDye,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CauldronItemStackModel {
    pub item: CauldronItemKey,
    pub tags: Vec<CauldronItemTag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CauldronDispatcherModel {
    pub name: &'static str,
    pub kind: CauldronDispatcherKind,
    tags: Vec<(CauldronItemTag, CauldronInteractionKind)>,
    items: Vec<(CauldronItemKey, CauldronInteractionKind)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CauldronBootstrapModel {
    pub dispatchers: Vec<CauldronDispatcherModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronResult {
    Success,
    Consume,
    TryWithEmptyHand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CauldronInteractionTrace {
    pub result: CauldronResult,
    pub server_action: Option<CauldronAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CauldronAction {
    Success {
        new_content: CauldronContent,
        returned_item: &'static str,
        stat: &'static str,
        sound: &'static str,
        game_event: &'static str,
    },
    CleanItem {
        new_content: CauldronContent,
        item: &'static str,
        stat: &'static str,
    },
    ConsumeOnly,
    TryWithEmptyHand,
}

pub fn dispenser_neighbor_change(
    powered: bool,
    powered_above: bool,
    triggered: bool,
) -> DispenseAction {
    let should_trigger = powered || powered_above;
    match (should_trigger, triggered) {
        (true, false) => DispenseAction::ScheduleTick {
            delay: 4,
            triggered: true,
        },
        (false, true) => DispenseAction::ScheduleTick {
            delay: 0,
            triggered: false,
        },
        _ => DispenseAction::Noop,
    }
}

pub fn select_dispense_behavior(item: DispensedItemKind, enabled: bool) -> DispenseBehaviorKind {
    if !enabled {
        return DispenseBehaviorKind::DefaultDrop;
    }

    match item {
        DispensedItemKind::Equippable => DispenseBehaviorKind::Equipment,
        DispensedItemKind::SpawnEggWithEntityData => DispenseBehaviorKind::SpawnEgg,
        DispensedItemKind::Projectile => DispenseBehaviorKind::Projectile,
        DispensedItemKind::Boat => DispenseBehaviorKind::Boat,
        DispensedItemKind::Minecart => DispenseBehaviorKind::Minecart,
        DispensedItemKind::ContainerBucket => DispenseBehaviorKind::EmptyContainer,
        DispensedItemKind::EmptyBucket => DispenseBehaviorKind::FillContainer,
        DispensedItemKind::FlintAndSteel | DispensedItemKind::Glowstone => {
            DispenseBehaviorKind::IgniteOrCharge
        }
        DispensedItemKind::BoneMeal => DispenseBehaviorKind::BoneMeal,
        DispensedItemKind::Tnt => DispenseBehaviorKind::PrimeTnt,
        DispensedItemKind::Shears => DispenseBehaviorKind::Shears,
        DispensedItemKind::ShulkerBox => DispenseBehaviorKind::PlaceShulkerBox,
        DispensedItemKind::SaddleOrChest => DispenseBehaviorKind::EquipAnimal,
        DispensedItemKind::GlassBottle => DispenseBehaviorKind::FillBottle,
        DispensedItemKind::DefaultItem => DispenseBehaviorKind::DefaultDrop,
    }
}

pub fn dispense_position(source: BlockPos, facing: Direction, scale: f64) -> DispensePosition {
    let center = DispensePosition {
        x: source.x as f64 + 0.5,
        y: source.y as f64 + 0.5,
        z: source.z as f64 + 0.5,
    };
    let (dx, dy, dz) = direction_steps(facing);
    DispensePosition {
        x: center.x + scale * dx as f64,
        y: center.y + scale * dy as f64,
        z: center.z + scale * dz as f64,
    }
}

pub fn default_dispense_item(
    source: BlockPos,
    facing: Direction,
    item: &'static str,
) -> DispenseAction {
    let mut pos = dispense_position(source, facing, 0.7);
    if matches!(facing, Direction::Up | Direction::Down) {
        pos.y -= 0.125;
    } else {
        pos.y -= 0.15625;
    }
    let (dx, _, dz) = direction_steps(facing);
    DispenseAction::SpawnItem {
        item,
        count: 1,
        pos,
        velocity: DispenseVelocity {
            x_base: dx as f64,
            y_base: 0.2,
            z_base: dz as f64,
            accuracy: 6,
        },
    }
}

pub fn execute_dispense(
    source: BlockPos,
    facing: Direction,
    item: DispensedItemKind,
    item_name: &'static str,
    enabled: bool,
    target_available: bool,
) -> Vec<DispenseAction> {
    if !target_available {
        return vec![DispenseAction::LevelEvent(1001)];
    }

    let behavior = select_dispense_behavior(item, enabled);
    let target = source.relative(facing);
    match behavior {
        DispenseBehaviorKind::DefaultDrop => vec![
            default_dispense_item(source, facing, item_name),
            DispenseAction::LevelEvent(1000),
            DispenseAction::LevelEvent(2000),
        ],
        DispenseBehaviorKind::Projectile => vec![
            DispenseAction::SpawnEntity {
                entity: item_name,
                pos: dispense_position(source, facing, 0.7),
                yaw_degrees: direction_yaw(facing),
            },
            DispenseAction::ChangeStack {
                consumed: 1,
                remainder: None,
            },
            DispenseAction::LevelEvent(1002),
        ],
        DispenseBehaviorKind::Boat | DispenseBehaviorKind::Minecart => vec![
            DispenseAction::SpawnEntity {
                entity: item_name,
                pos: dispense_position(source, facing, 1.125),
                yaw_degrees: direction_yaw(facing),
            },
            DispenseAction::ChangeStack {
                consumed: 1,
                remainder: None,
            },
            DispenseAction::LevelEvent(1000),
        ],
        DispenseBehaviorKind::EmptyContainer
        | DispenseBehaviorKind::FillContainer
        | DispenseBehaviorKind::IgniteOrCharge
        | DispenseBehaviorKind::BoneMeal
        | DispenseBehaviorKind::Shears
        | DispenseBehaviorKind::FillBottle => vec![DispenseAction::UseOnBlock { target, behavior }],
        DispenseBehaviorKind::PrimeTnt => vec![
            DispenseAction::SpawnEntity {
                entity: "minecraft:tnt",
                pos: dispense_position(source, facing, 0.7),
                yaw_degrees: 0.0,
            },
            DispenseAction::ChangeStack {
                consumed: 1,
                remainder: None,
            },
        ],
        DispenseBehaviorKind::Equipment
        | DispenseBehaviorKind::SpawnEgg
        | DispenseBehaviorKind::PlaceShulkerBox
        | DispenseBehaviorKind::EquipAnimal => {
            vec![DispenseAction::UseOnBlock { target, behavior }]
        }
    }
}

pub const CAULDRON_DISPATCHER_CODEC_KEYS: [&str; 4] = ["empty", "water", "lava", "powder_snow"];

pub const JAVA_NULL_MARKED_CAULDRON_PACKAGE: bool = true;

pub const JAVA_CAULDRON_COLOURS: [&str; 16] = [
    "white",
    "gray",
    "black",
    "blue",
    "brown",
    "cyan",
    "green",
    "light_blue",
    "light_gray",
    "lime",
    "magenta",
    "orange",
    "pink",
    "purple",
    "red",
    "yellow",
];

pub fn cauldron_package_is_null_marked() -> bool {
    JAVA_NULL_MARKED_CAULDRON_PACKAGE
}

impl CauldronDispatcherModel {
    pub fn new(name: &'static str, kind: CauldronDispatcherKind) -> Self {
        Self {
            name,
            kind,
            tags: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn put_item(&mut self, item: CauldronItemKey, interaction: CauldronInteractionKind) {
        self.items.push((item, interaction));
    }

    pub fn put_tag(&mut self, tag: CauldronItemTag, interaction: CauldronInteractionKind) {
        self.tags.push((tag, interaction));
    }

    pub fn get(&self, stack: &CauldronItemStackModel) -> CauldronInteractionKind {
        self.tags
            .iter()
            .find_map(|(tag, interaction)| stack.tags.contains(tag).then_some(*interaction))
            .or_else(|| {
                self.items
                    .iter()
                    .find_map(|(item, interaction)| (*item == stack.item).then_some(*interaction))
            })
            .unwrap_or(CauldronInteractionKind::Default)
    }

    pub fn item_mappings(&self) -> &[(CauldronItemKey, CauldronInteractionKind)] {
        &self.items
    }

    pub fn tag_mappings(&self) -> &[(CauldronItemTag, CauldronInteractionKind)] {
        &self.tags
    }
}

pub fn cauldron_bootstrap_model() -> CauldronBootstrapModel {
    let mut empty = new_cauldron_dispatcher("empty", CauldronDispatcherKind::Empty);
    add_default_cauldron_interactions(&mut empty);
    empty.put_item(
        CauldronItemKey::Potion,
        CauldronInteractionKind::EmptyWaterPotion,
    );

    let mut water = new_cauldron_dispatcher("water", CauldronDispatcherKind::Water);
    add_default_cauldron_interactions(&mut water);
    water.put_item(
        CauldronItemKey::Bucket,
        CauldronInteractionKind::FillBucketWater,
    );
    water.put_item(
        CauldronItemKey::GlassBottle,
        CauldronInteractionKind::FillGlassBottle,
    );
    water.put_item(
        CauldronItemKey::Potion,
        CauldronInteractionKind::AddWaterPotion,
    );
    water.put_tag(
        CauldronItemTag::CanRemoveDye,
        CauldronInteractionKind::CleanDyedItem,
    );
    for colour in JAVA_CAULDRON_COLOURS {
        water.put_item(
            CauldronItemKey::Banner(colour),
            CauldronInteractionKind::CleanBanner,
        );
    }
    for colour in JAVA_CAULDRON_COLOURS {
        water.put_item(
            CauldronItemKey::ShulkerBox(colour),
            CauldronInteractionKind::CleanShulkerBox,
        );
    }

    let mut lava = new_cauldron_dispatcher("lava", CauldronDispatcherKind::Lava);
    lava.put_item(
        CauldronItemKey::Bucket,
        CauldronInteractionKind::FillBucketLava,
    );
    add_default_cauldron_interactions(&mut lava);

    let mut powder_snow =
        new_cauldron_dispatcher("powder_snow", CauldronDispatcherKind::PowderSnow);
    powder_snow.put_item(
        CauldronItemKey::Bucket,
        CauldronInteractionKind::FillBucketPowderSnow,
    );
    add_default_cauldron_interactions(&mut powder_snow);

    CauldronBootstrapModel {
        dispatchers: vec![empty, water, lava, powder_snow],
    }
}

fn new_cauldron_dispatcher(
    name: &'static str,
    kind: CauldronDispatcherKind,
) -> CauldronDispatcherModel {
    CauldronDispatcherModel::new(name, kind)
}

fn add_default_cauldron_interactions(dispatcher: &mut CauldronDispatcherModel) {
    dispatcher.put_item(
        CauldronItemKey::LavaBucket,
        CauldronInteractionKind::FillLava,
    );
    dispatcher.put_item(
        CauldronItemKey::WaterBucket,
        CauldronInteractionKind::FillWater,
    );
    dispatcher.put_item(
        CauldronItemKey::PowderSnowBucket,
        CauldronInteractionKind::FillPowderSnow,
    );
}

pub fn cauldron_interaction_trace(
    content: CauldronContent,
    item: CauldronItem,
    under_water: bool,
    client_side: bool,
) -> CauldronInteractionTrace {
    let server_action = cauldron_interaction(content, item, under_water);
    let result = match &server_action {
        CauldronAction::Success { .. } | CauldronAction::CleanItem { .. } => {
            CauldronResult::Success
        }
        CauldronAction::ConsumeOnly => CauldronResult::Consume,
        CauldronAction::TryWithEmptyHand => CauldronResult::TryWithEmptyHand,
    };
    CauldronInteractionTrace {
        result,
        server_action: (!client_side
            && !matches!(
                server_action,
                CauldronAction::TryWithEmptyHand | CauldronAction::ConsumeOnly
            ))
        .then_some(server_action),
    }
}

pub fn cauldron_interaction(
    content: CauldronContent,
    item: CauldronItem,
    under_water: bool,
) -> CauldronAction {
    match (content, item) {
        (_, CauldronItem::WaterBucket) => CauldronAction::Success {
            new_content: CauldronContent::Water { level: 3 },
            returned_item: "minecraft:bucket",
            stat: "fill_cauldron",
            sound: "bucket_empty",
            game_event: "fluid_place",
        },
        (_, CauldronItem::LavaBucket) if under_water => CauldronAction::ConsumeOnly,
        (_, CauldronItem::LavaBucket) => CauldronAction::Success {
            new_content: CauldronContent::Lava,
            returned_item: "minecraft:bucket",
            stat: "fill_cauldron",
            sound: "bucket_empty_lava",
            game_event: "fluid_place",
        },
        (_, CauldronItem::PowderSnowBucket) if under_water => CauldronAction::ConsumeOnly,
        (_, CauldronItem::PowderSnowBucket) => CauldronAction::Success {
            new_content: CauldronContent::PowderSnow { level: 3 },
            returned_item: "minecraft:bucket",
            stat: "fill_cauldron",
            sound: "bucket_empty_powder_snow",
            game_event: "fluid_place",
        },
        (CauldronContent::Empty, CauldronItem::WaterPotion) => CauldronAction::Success {
            new_content: CauldronContent::Water { level: 1 },
            returned_item: "minecraft:glass_bottle",
            stat: "use_cauldron",
            sound: "bottle_empty",
            game_event: "fluid_place",
        },
        (CauldronContent::Water { level }, CauldronItem::WaterPotion) if level < 3 => {
            CauldronAction::Success {
                new_content: CauldronContent::Water { level: level + 1 },
                returned_item: "minecraft:glass_bottle",
                stat: "use_cauldron",
                sound: "bottle_empty",
                game_event: "fluid_place",
            }
        }
        (CauldronContent::Water { level: 3 }, CauldronItem::WaterPotion) => {
            CauldronAction::TryWithEmptyHand
        }
        (CauldronContent::Water { level }, CauldronItem::GlassBottle) => CauldronAction::Success {
            new_content: lower_layered_cauldron(level),
            returned_item: "minecraft:potion{water}",
            stat: "use_cauldron",
            sound: "bottle_fill",
            game_event: "fluid_pickup",
        },
        (CauldronContent::Water { level: 3 }, CauldronItem::Bucket) => CauldronAction::Success {
            new_content: CauldronContent::Empty,
            returned_item: "minecraft:water_bucket",
            stat: "use_cauldron",
            sound: "bucket_fill",
            game_event: "fluid_pickup",
        },
        (CauldronContent::Lava, CauldronItem::Bucket) => CauldronAction::Success {
            new_content: CauldronContent::Empty,
            returned_item: "minecraft:lava_bucket",
            stat: "use_cauldron",
            sound: "bucket_fill_lava",
            game_event: "fluid_pickup",
        },
        (CauldronContent::PowderSnow { level: 3 }, CauldronItem::Bucket) => {
            CauldronAction::Success {
                new_content: CauldronContent::Empty,
                returned_item: "minecraft:powder_snow_bucket",
                stat: "use_cauldron",
                sound: "bucket_fill_powder_snow",
                game_event: "fluid_pickup",
            }
        }
        (CauldronContent::Water { level }, CauldronItem::DyedArmor) => CauldronAction::CleanItem {
            new_content: lower_layered_cauldron(level),
            item: "remove_dyed_color",
            stat: "clean_armor",
        },
        (CauldronContent::Water { level }, CauldronItem::PatternedBanner) => {
            CauldronAction::CleanItem {
                new_content: lower_layered_cauldron(level),
                item: "remove_last_banner_pattern",
                stat: "clean_banner",
            }
        }
        (CauldronContent::Water { level }, CauldronItem::DyedShulkerBox) => {
            CauldronAction::CleanItem {
                new_content: lower_layered_cauldron(level),
                item: "minecraft:shulker_box",
                stat: "clean_shulker_box",
            }
        }
        (_, CauldronItem::NonWaterPotion)
        | (_, CauldronItem::UndyedArmor)
        | (_, CauldronItem::PlainBanner)
        | (_, CauldronItem::PlainShulkerBox)
        | (_, CauldronItem::GlassBottle)
        | (_, CauldronItem::Bucket) => CauldronAction::TryWithEmptyHand,
        _ => CauldronAction::TryWithEmptyHand,
    }
}

pub fn lower_layered_cauldron(level: u8) -> CauldronContent {
    if level <= 1 {
        CauldronContent::Empty
    } else {
        CauldronContent::Water { level: level - 1 }
    }
}

fn direction_steps(direction: Direction) -> (i32, i32, i32) {
    match direction {
        Direction::West => (-1, 0, 0),
        Direction::East => (1, 0, 0),
        Direction::Down => (0, -1, 0),
        Direction::Up => (0, 1, 0),
        Direction::North => (0, 0, -1),
        Direction::South => (0, 0, 1),
    }
}

fn direction_yaw(direction: Direction) -> f32 {
    match direction {
        Direction::South => 0.0,
        Direction::West => 90.0,
        Direction::North => 180.0,
        Direction::East => 270.0,
        Direction::Up | Direction::Down => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> BlockPos {
        BlockPos {
            x: 10,
            y: 64,
            z: -5,
        }
    }

    #[test]
    fn dispenser_trigger_state_schedules_vanilla_four_tick_pulse() {
        assert_eq!(
            dispenser_neighbor_change(true, false, false),
            DispenseAction::ScheduleTick {
                delay: 4,
                triggered: true
            }
        );
        assert_eq!(
            dispenser_neighbor_change(false, false, true),
            DispenseAction::ScheduleTick {
                delay: 0,
                triggered: false
            }
        );
        assert_eq!(
            dispenser_neighbor_change(true, true, true),
            DispenseAction::Noop
        );
    }

    #[test]
    fn dispenser_behavior_selection_matches_vanilla_fallbacks() {
        assert_eq!(
            select_dispense_behavior(DispensedItemKind::Equippable, true),
            DispenseBehaviorKind::Equipment
        );
        assert_eq!(
            select_dispense_behavior(DispensedItemKind::SpawnEggWithEntityData, true),
            DispenseBehaviorKind::SpawnEgg
        );
        assert_eq!(
            select_dispense_behavior(DispensedItemKind::SpawnEggWithEntityData, false),
            DispenseBehaviorKind::DefaultDrop
        );
        assert_eq!(
            select_dispense_behavior(DispensedItemKind::DefaultItem, true),
            DispenseBehaviorKind::DefaultDrop
        );
    }

    #[test]
    fn default_dispense_spawns_item_at_offset_with_default_events() {
        let actions = execute_dispense(
            source(),
            Direction::North,
            DispensedItemKind::DefaultItem,
            "minecraft:cobblestone",
            true,
            true,
        );
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[1], DispenseAction::LevelEvent(1000));
        assert_eq!(actions[2], DispenseAction::LevelEvent(2000));
        assert!(matches!(
            actions[0],
            DispenseAction::SpawnItem {
                item: "minecraft:cobblestone",
                count: 1,
                pos: DispensePosition { z, .. },
                velocity: DispenseVelocity { z_base: -1.0, accuracy: 6, .. }
            } if z < -5.0
        ));
    }

    #[test]
    fn projectile_boat_tnt_and_empty_slots_follow_special_paths() {
        assert!(matches!(
            execute_dispense(
                source(),
                Direction::East,
                DispensedItemKind::Projectile,
                "minecraft:arrow",
                true,
                true
            )[0],
            DispenseAction::SpawnEntity {
                entity: "minecraft:arrow",
                yaw_degrees: 270.0,
                ..
            }
        ));
        assert!(matches!(
            execute_dispense(
                source(),
                Direction::South,
                DispensedItemKind::Boat,
                "minecraft:oak_boat",
                true,
                true
            )[0],
            DispenseAction::SpawnEntity {
                entity: "minecraft:oak_boat",
                yaw_degrees: 0.0,
                ..
            }
        ));
        assert!(matches!(
            execute_dispense(
                source(),
                Direction::Up,
                DispensedItemKind::Tnt,
                "minecraft:tnt",
                true,
                true
            )[0],
            DispenseAction::SpawnEntity {
                entity: "minecraft:tnt",
                ..
            }
        ));
        assert_eq!(
            execute_dispense(
                source(),
                Direction::North,
                DispensedItemKind::DefaultItem,
                "minecraft:air",
                true,
                false
            ),
            vec![DispenseAction::LevelEvent(1001)]
        );
    }

    #[test]
    fn use_on_block_dispense_behaviors_target_facing_block() {
        assert_eq!(
            execute_dispense(
                source(),
                Direction::West,
                DispensedItemKind::ContainerBucket,
                "minecraft:water_bucket",
                true,
                true
            ),
            vec![DispenseAction::UseOnBlock {
                target: BlockPos { x: 9, y: 64, z: -5 },
                behavior: DispenseBehaviorKind::EmptyContainer
            }]
        );
        assert_eq!(
            execute_dispense(
                source(),
                Direction::West,
                DispensedItemKind::EmptyBucket,
                "minecraft:bucket",
                true,
                true
            ),
            vec![DispenseAction::UseOnBlock {
                target: BlockPos { x: 9, y: 64, z: -5 },
                behavior: DispenseBehaviorKind::FillContainer
            }]
        );
    }

    #[test]
    fn cauldron_dispatchers_match_java_bootstrap_registration() {
        let bootstrap = cauldron_bootstrap_model();
        let names: Vec<_> = bootstrap
            .dispatchers
            .iter()
            .map(|dispatcher| dispatcher.name)
            .collect();
        assert_eq!(names, CAULDRON_DISPATCHER_CODEC_KEYS);
        assert!(cauldron_package_is_null_marked());

        let empty = &bootstrap.dispatchers[0];
        assert_eq!(
            empty.item_mappings(),
            &[
                (
                    CauldronItemKey::LavaBucket,
                    CauldronInteractionKind::FillLava
                ),
                (
                    CauldronItemKey::WaterBucket,
                    CauldronInteractionKind::FillWater
                ),
                (
                    CauldronItemKey::PowderSnowBucket,
                    CauldronInteractionKind::FillPowderSnow
                ),
                (
                    CauldronItemKey::Potion,
                    CauldronInteractionKind::EmptyWaterPotion
                )
            ]
        );

        let lava = &bootstrap.dispatchers[2];
        assert_eq!(
            lava.item_mappings()[0],
            (
                CauldronItemKey::Bucket,
                CauldronInteractionKind::FillBucketLava
            )
        );
        assert_eq!(
            lava.get(&CauldronItemStackModel {
                item: CauldronItemKey::Other,
                tags: vec![]
            }),
            CauldronInteractionKind::Default
        );
    }

    #[test]
    fn water_dispatcher_registers_java_cleaning_items_and_tag_precedence() {
        let bootstrap = cauldron_bootstrap_model();
        let water = &bootstrap.dispatchers[1];

        assert_eq!(
            water.tag_mappings(),
            &[(
                CauldronItemTag::CanRemoveDye,
                CauldronInteractionKind::CleanDyedItem
            )]
        );
        assert_eq!(
            water.get(&CauldronItemStackModel {
                item: CauldronItemKey::Potion,
                tags: vec![CauldronItemTag::CanRemoveDye]
            }),
            CauldronInteractionKind::CleanDyedItem
        );

        let banner_items: Vec<_> = water
            .item_mappings()
            .iter()
            .filter_map(|(item, interaction)| {
                matches!(interaction, CauldronInteractionKind::CleanBanner).then_some(*item)
            })
            .collect();
        let shulker_items: Vec<_> = water
            .item_mappings()
            .iter()
            .filter_map(|(item, interaction)| {
                matches!(interaction, CauldronInteractionKind::CleanShulkerBox).then_some(*item)
            })
            .collect();
        assert_eq!(banner_items.len(), 16);
        assert_eq!(shulker_items.len(), 16);
        assert_eq!(banner_items[0], CauldronItemKey::Banner("white"));
        assert_eq!(banner_items[15], CauldronItemKey::Banner("yellow"));
        assert_eq!(shulker_items[0], CauldronItemKey::ShulkerBox("white"));
        assert_eq!(shulker_items[15], CauldronItemKey::ShulkerBox("yellow"));
    }

    #[test]
    fn cauldron_trace_keeps_java_result_and_server_side_effects_separate() {
        assert_eq!(
            cauldron_interaction_trace(
                CauldronContent::Water { level: 2 },
                CauldronItem::GlassBottle,
                false,
                true
            ),
            CauldronInteractionTrace {
                result: CauldronResult::Success,
                server_action: None
            }
        );
        assert_eq!(
            cauldron_interaction_trace(
                CauldronContent::Water { level: 2 },
                CauldronItem::GlassBottle,
                false,
                false
            ),
            CauldronInteractionTrace {
                result: CauldronResult::Success,
                server_action: Some(CauldronAction::Success {
                    new_content: CauldronContent::Water { level: 1 },
                    returned_item: "minecraft:potion{water}",
                    stat: "use_cauldron",
                    sound: "bottle_fill",
                    game_event: "fluid_pickup"
                })
            }
        );
        assert_eq!(
            cauldron_interaction_trace(
                CauldronContent::Empty,
                CauldronItem::PowderSnowBucket,
                true,
                false
            ),
            CauldronInteractionTrace {
                result: CauldronResult::Consume,
                server_action: None
            }
        );
    }

    #[test]
    fn cauldron_bucket_interactions_fill_and_pickup_fluids() {
        assert_eq!(
            cauldron_interaction(CauldronContent::Empty, CauldronItem::WaterBucket, false),
            CauldronAction::Success {
                new_content: CauldronContent::Water { level: 3 },
                returned_item: "minecraft:bucket",
                stat: "fill_cauldron",
                sound: "bucket_empty",
                game_event: "fluid_place"
            }
        );
        assert_eq!(
            cauldron_interaction(
                CauldronContent::Water { level: 3 },
                CauldronItem::Bucket,
                false
            ),
            CauldronAction::Success {
                new_content: CauldronContent::Empty,
                returned_item: "minecraft:water_bucket",
                stat: "use_cauldron",
                sound: "bucket_fill",
                game_event: "fluid_pickup"
            }
        );
        assert_eq!(
            cauldron_interaction(CauldronContent::Empty, CauldronItem::LavaBucket, true),
            CauldronAction::ConsumeOnly
        );
    }

    #[test]
    fn cauldron_bottle_and_potion_interactions_adjust_layers() {
        assert_eq!(
            cauldron_interaction(CauldronContent::Empty, CauldronItem::WaterPotion, false),
            CauldronAction::Success {
                new_content: CauldronContent::Water { level: 1 },
                returned_item: "minecraft:glass_bottle",
                stat: "use_cauldron",
                sound: "bottle_empty",
                game_event: "fluid_place"
            }
        );
        assert_eq!(
            cauldron_interaction(
                CauldronContent::Water { level: 2 },
                CauldronItem::GlassBottle,
                false
            ),
            CauldronAction::Success {
                new_content: CauldronContent::Water { level: 1 },
                returned_item: "minecraft:potion{water}",
                stat: "use_cauldron",
                sound: "bottle_fill",
                game_event: "fluid_pickup"
            }
        );
        assert_eq!(
            cauldron_interaction(
                CauldronContent::Water { level: 3 },
                CauldronItem::WaterPotion,
                false
            ),
            CauldronAction::TryWithEmptyHand
        );
    }

    #[test]
    fn cauldron_cleaning_requires_water_and_lowers_fill_level() {
        assert_eq!(
            cauldron_interaction(
                CauldronContent::Water { level: 1 },
                CauldronItem::DyedArmor,
                false
            ),
            CauldronAction::CleanItem {
                new_content: CauldronContent::Empty,
                item: "remove_dyed_color",
                stat: "clean_armor"
            }
        );
        assert_eq!(
            cauldron_interaction(
                CauldronContent::Water { level: 2 },
                CauldronItem::PatternedBanner,
                false
            ),
            CauldronAction::CleanItem {
                new_content: CauldronContent::Water { level: 1 },
                item: "remove_last_banner_pattern",
                stat: "clean_banner"
            }
        );
        assert_eq!(
            cauldron_interaction(CauldronContent::Lava, CauldronItem::DyedShulkerBox, false),
            CauldronAction::TryWithEmptyHand
        );
    }
}
