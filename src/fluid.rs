#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_update::{BlockPos, Direction};
use crate::scheduled_tick::{SavedTick, ScheduledTick, TickPriority};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidKind {
    Water,
    Lava,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidState {
    pub kind: FluidKind,
    pub level: u8,
    pub falling: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiquidPlaceResult {
    Rejected(BlockStateModel),
    Replaced(BlockStateModel),
    Waterlogged(BlockStateModel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiquidPickupResult {
    Empty,
    PickedUp {
        fluid: FluidState,
        replacement: BlockStateModel,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidUpdatePlan {
    pub pos: BlockPos,
    pub state: FluidState,
    pub self_tick: ScheduledTick,
    pub neighbor_ticks: Vec<ScheduledTick>,
}

impl FluidKind {
    pub fn registry_id(self) -> &'static str {
        match self {
            Self::Water => "minecraft:water",
            Self::Lava => "minecraft:lava",
        }
    }

    pub const fn tick_delay(self) -> i32 {
        match self {
            Self::Water => 5,
            Self::Lava => 30,
        }
    }

    pub const fn source(self) -> FluidState {
        FluidState {
            kind: self,
            level: 8,
            falling: false,
        }
    }
}

impl FluidState {
    pub const fn empty() -> Option<Self> {
        None
    }

    pub const fn is_source(self) -> bool {
        self.level == 8 && !self.falling
    }

    pub fn block_state(self) -> BlockStateModel {
        BlockStateModel::new(self.kind.registry_id())
            .with_property("level", if self.is_source() { "0" } else { "1" })
    }
}

pub fn fluid_state_for_block(state: &BlockStateModel) -> Option<FluidState> {
    if state.property("waterlogged") == Some("true") {
        return Some(FluidKind::Water.source());
    }

    match state.registry_id.as_str() {
        "minecraft:water" => Some(FluidKind::Water.source()),
        "minecraft:lava" => Some(FluidKind::Lava.source()),
        _ => None,
    }
}

pub fn can_hold_water(state: &BlockStateModel) -> bool {
    state.properties.contains_key("waterlogged")
}

pub fn can_place_liquid(state: &BlockStateModel, fluid: FluidKind) -> bool {
    if state.is_air() {
        return true;
    }
    fluid == FluidKind::Water
        && can_hold_water(state)
        && state.property("waterlogged") != Some("true")
}

pub fn place_liquid(state: &BlockStateModel, fluid: FluidKind) -> LiquidPlaceResult {
    if state.is_air()
        || state.registry_id == "minecraft:water"
        || state.registry_id == "minecraft:lava"
    {
        return LiquidPlaceResult::Replaced(fluid.source().block_state());
    }

    if fluid == FluidKind::Water && can_place_liquid(state, fluid) {
        return LiquidPlaceResult::Waterlogged(state.clone().with_property("waterlogged", "true"));
    }

    LiquidPlaceResult::Rejected(state.clone())
}

pub fn pickup_liquid(state: &BlockStateModel) -> LiquidPickupResult {
    if state.property("waterlogged") == Some("true") {
        return LiquidPickupResult::PickedUp {
            fluid: FluidKind::Water.source(),
            replacement: state.clone().with_property("waterlogged", "false"),
        };
    }

    match state.registry_id.as_str() {
        "minecraft:water" => LiquidPickupResult::PickedUp {
            fluid: FluidKind::Water.source(),
            replacement: BlockStateModel::air(),
        },
        "minecraft:lava" => LiquidPickupResult::PickedUp {
            fluid: FluidKind::Lava.source(),
            replacement: BlockStateModel::air(),
        },
        _ => LiquidPickupResult::Empty,
    }
}

pub fn plan_fluid_update(
    pos: BlockPos,
    state: FluidState,
    game_time: i64,
    first_sub_tick_order: i64,
) -> FluidUpdatePlan {
    let self_tick = ScheduledTick::create(
        game_time,
        first_sub_tick_order,
        pos,
        state.kind.registry_id(),
        state.kind.tick_delay(),
        TickPriority::Normal,
    );
    let neighbor_ticks = fluid_neighbor_order()
        .into_iter()
        .enumerate()
        .map(|(offset, direction)| {
            ScheduledTick::create(
                game_time,
                first_sub_tick_order + 1 + offset as i64,
                pos.relative(direction),
                state.kind.registry_id(),
                state.kind.tick_delay(),
                TickPriority::Normal,
            )
        })
        .collect();

    FluidUpdatePlan {
        pos,
        state,
        self_tick,
        neighbor_ticks,
    }
}

pub fn save_fluid_ticks(plan: &FluidUpdatePlan, current_tick: i64) -> Vec<SavedTick> {
    std::iter::once(&plan.self_tick)
        .chain(plan.neighbor_ticks.iter())
        .map(|tick| tick.to_saved_tick(current_tick))
        .collect()
}

pub const fn fluid_neighbor_order() -> [Direction; 5] {
    [
        Direction::Down,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        can_hold_water, can_place_liquid, fluid_neighbor_order, fluid_state_for_block,
        pickup_liquid, place_liquid, plan_fluid_update, save_fluid_ticks, FluidKind,
        LiquidPickupResult, LiquidPlaceResult,
    };
    use crate::block_behavior::BlockStateModel;
    use crate::block_update::{BlockPos, Direction};
    use crate::scheduled_tick::TickPriority;

    #[test]
    fn waterlogged_property_exposes_source_water_fluid_state() {
        let stairs = BlockStateModel::default_for("minecraft:oak_stairs")
            .unwrap()
            .with_property("waterlogged", "true");
        let fluid = fluid_state_for_block(&stairs).unwrap();
        assert_eq!(fluid.kind, FluidKind::Water);
        assert!(fluid.is_source());

        assert_eq!(
            fluid_state_for_block(&BlockStateModel::new("minecraft:lava"))
                .unwrap()
                .kind,
            FluidKind::Lava
        );
        assert_eq!(
            fluid_state_for_block(&BlockStateModel::new("minecraft:stone")),
            None
        );
    }

    #[test]
    fn only_water_can_fill_waterloggable_non_fluid_blocks() {
        let chest = BlockStateModel::default_for("minecraft:chest").unwrap();
        assert!(can_hold_water(&chest));
        assert!(can_place_liquid(&chest, FluidKind::Water));
        assert!(!can_place_liquid(&chest, FluidKind::Lava));
        assert!(!can_place_liquid(
            &chest.with_property("waterlogged", "true"),
            FluidKind::Water
        ));
    }

    #[test]
    fn placing_liquid_replaces_air_or_fluid_and_waterlogs_supported_blocks() {
        assert_eq!(
            place_liquid(&BlockStateModel::air(), FluidKind::Water),
            LiquidPlaceResult::Replaced(
                BlockStateModel::new("minecraft:water").with_property("level", "0")
            )
        );

        let stairs = BlockStateModel::default_for("minecraft:oak_stairs").unwrap();
        assert_eq!(
            place_liquid(&stairs, FluidKind::Water),
            LiquidPlaceResult::Waterlogged(stairs.clone().with_property("waterlogged", "true"))
        );
        assert_eq!(
            place_liquid(&stairs, FluidKind::Lava),
            LiquidPlaceResult::Rejected(stairs)
        );
    }

    #[test]
    fn picking_up_liquid_clears_waterlogged_flag_or_replaces_fluid_block_with_air() {
        let chest = BlockStateModel::default_for("minecraft:chest")
            .unwrap()
            .with_property("waterlogged", "true");
        assert_eq!(
            pickup_liquid(&chest),
            LiquidPickupResult::PickedUp {
                fluid: FluidKind::Water.source(),
                replacement: chest.with_property("waterlogged", "false"),
            }
        );

        assert_eq!(
            pickup_liquid(&BlockStateModel::new("minecraft:lava")),
            LiquidPickupResult::PickedUp {
                fluid: FluidKind::Lava.source(),
                replacement: BlockStateModel::air(),
            }
        );
        assert_eq!(
            pickup_liquid(&BlockStateModel::new("minecraft:stone")),
            LiquidPickupResult::Empty
        );
    }

    #[test]
    fn fluid_updates_schedule_self_and_neighbor_ticks_with_vanilla_delays() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let plan = plan_fluid_update(pos, FluidKind::Water.source(), 100, 7);
        assert_eq!(plan.self_tick.ty, "minecraft:water");
        assert_eq!(plan.self_tick.trigger_tick, 105);
        assert_eq!(plan.self_tick.sub_tick_order, 7);
        assert_eq!(plan.self_tick.priority, TickPriority::Normal);

        assert_eq!(plan.neighbor_ticks.len(), 5);
        assert_eq!(
            plan.neighbor_ticks
                .iter()
                .map(|tick| tick.pos)
                .collect::<Vec<_>>(),
            fluid_neighbor_order()
                .into_iter()
                .map(|direction| pos.relative(direction))
                .collect::<Vec<_>>()
        );

        let lava = plan_fluid_update(pos, FluidKind::Lava.source(), 100, 20);
        assert_eq!(lava.self_tick.trigger_tick, 130);
    }

    #[test]
    fn saved_fluid_ticks_store_delay_relative_to_current_tick() {
        let plan = plan_fluid_update(
            BlockPos { x: 1, y: 2, z: 3 },
            FluidKind::Water.source(),
            200,
            0,
        );
        let saved = save_fluid_ticks(&plan, 202);
        assert_eq!(saved.len(), 6);
        assert!(saved.iter().all(|tick| tick.ty == "minecraft:water"));
        assert!(saved.iter().all(|tick| tick.delay == 3));
        assert!(saved
            .iter()
            .all(|tick| tick.priority == TickPriority::Normal));
    }

    #[test]
    fn fluid_neighbor_order_flows_down_before_horizontal_spread() {
        assert_eq!(
            fluid_neighbor_order(),
            [
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ]
        );
    }
}
