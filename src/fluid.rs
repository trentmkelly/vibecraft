#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_update::{BlockPos, Direction};
use crate::block_metadata::{representative_state_definition, ShapeKind};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidTickResult {
    pub changes: Vec<(BlockPos, BlockStateModel)>,
    pub schedule: Vec<BlockPos>,
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
        let level = if self.is_source() {
            0
        } else if self.falling {
            8
        } else {
            8_u8.saturating_sub(self.level).clamp(1, 7)
        };
        BlockStateModel::new(self.kind.registry_id()).with_property("level", level.to_string())
    }
}

pub fn fluid_state_for_block(state: &BlockStateModel) -> Option<FluidState> {
    if state.property("waterlogged") == Some("true") {
        return Some(FluidKind::Water.source());
    }

    match state.registry_id.as_str() {
        "minecraft:water" => Some(fluid_state_from_legacy_level(
            FluidKind::Water,
            state.property("level"),
        )),
        "minecraft:lava" => Some(fluid_state_from_legacy_level(
            FluidKind::Lava,
            state.property("level"),
        )),
        _ => None,
    }
}

pub fn block_state_model_from_name(name: &str) -> BlockStateModel {
    let Some((registry_id, properties)) = name.split_once('[') else {
        return BlockStateModel::new(name);
    };
    let mut state = BlockStateModel::new(registry_id);
    for property in properties.trim_end_matches(']').split(',') {
        if let Some((key, value)) = property.split_once('=') {
            state = state.with_property(key, value);
        }
    }
    state
}

pub fn block_state_model_name(state: &BlockStateModel) -> String {
    if state.properties.is_empty() {
        return state.registry_id.clone();
    }
    let properties = state
        .properties
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}[{properties}]", state.registry_id)
}

fn fluid_state_from_legacy_level(kind: FluidKind, level: Option<&str>) -> FluidState {
    let legacy = level.and_then(|value| value.parse::<u8>().ok()).unwrap_or(0);
    if legacy == 0 {
        kind.source()
    } else if legacy >= 8 {
        FluidState {
            kind,
            level: 8,
            falling: true,
        }
    } else {
        FluidState {
            kind,
            level: 8 - legacy,
            falling: false,
        }
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

/// Returns whether a normal `BlockItem` placement may use this position as its
/// clicked placement target instead of offsetting into the clicked face.
///
/// Java: `BlockPlaceContext.canPlace()` delegates to `BlockState.canBeReplaced`.
/// Liquid blocks are replaceable by block items, while solid blocks are not.
pub fn block_item_can_replace(state: &BlockStateModel) -> bool {
    if state.is_air() || fluid_state_for_block(state).is_some() {
        return true;
    }
    representative_state_definition(state.registry_id.as_str())
        .map(|definition| definition.physical.destroy_time == 0.0)
        .unwrap_or(false)
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

/// Executes the server-side flowing-fluid tick.
///
/// Java parity target:
/// * `FlowingFluid.tick` recomputes non-source liquid state, writes air when it dries up,
///   writes the new legacy `LiquidBlock.LEVEL` when its amount changes, then calls `spread`.
/// * `FlowingFluid.spread` tries downward flow first, then lateral spread selected by
///   `getSpread`/`getSlopeDistance`.
/// * `LiquidBlock.onPlace` and neighbor shape updates schedule fluid ticks for changed fluid
///   blocks; callers should schedule every returned `schedule` position with the fluid delay.
pub fn tick_fluid(
    pos: BlockPos,
    current_state: &BlockStateModel,
    get_block: impl Fn(BlockPos) -> BlockStateModel,
) -> FluidTickResult {
    let Some(mut fluid) = fluid_state_for_block(current_state) else {
        return FluidTickResult {
            changes: Vec::new(),
            schedule: Vec::new(),
        };
    };
    let mut block_state = current_state.clone();
    let mut changes = Vec::new();
    let mut schedule = Vec::new();

    if fluid.kind == FluidKind::Lava && lava_should_convert_to_solid(pos, fluid, &get_block) {
        changes.push((
            pos,
            BlockStateModel::new(if fluid.is_source() {
                "minecraft:obsidian"
            } else {
                "minecraft:cobblestone"
            }),
        ));
        return FluidTickResult { changes, schedule };
    }

    if !fluid.is_source() {
        let new_fluid = get_new_liquid(pos, &block_state, fluid.kind, &get_block);
        if new_fluid != Some(fluid) {
            match new_fluid {
                Some(next) => {
                    fluid = next;
                    block_state = next.block_state();
                    changes.push((pos, block_state.clone()));
                    schedule.push(pos);
                }
                None => {
                    changes.push((pos, BlockStateModel::air()));
                    return FluidTickResult { changes, schedule };
                }
            }
        }
    }

    spread(pos, &block_state, fluid, &get_block, &mut changes, &mut schedule);
    FluidTickResult { changes, schedule }
}

fn spread(
    pos: BlockPos,
    state: &BlockStateModel,
    fluid: FluidState,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
    changes: &mut Vec<(BlockPos, BlockStateModel)>,
    schedule: &mut Vec<BlockPos>,
) {
    let below_pos = pos.relative(Direction::Down);
    let below_state = get_block(below_pos);
    let below_fluid = fluid_state_for_block(&below_state);
    if can_maybe_pass_through(
        pos,
        state,
        Direction::Down,
        below_pos,
        &below_state,
        below_fluid,
        fluid.kind,
    )
        && can_replace_fluid(below_fluid, fluid.kind, Direction::Down)
        && can_hold_specific_fluid(&below_state, fluid.kind)
    {
        let new_below = get_new_liquid(below_pos, &below_state, fluid.kind, get_block)
            .unwrap_or(FluidState {
                kind: fluid.kind,
                level: 8,
                falling: true,
            });
        spread_to(below_pos, &below_state, Direction::Down, new_below, changes, schedule);
        if source_neighbor_count(pos, fluid.kind, get_block) >= 3 {
            spread_to_sides(pos, state, fluid, get_block, changes, schedule);
        }
        return;
    }

    if fluid.is_source() || !is_water_hole(pos, state, below_pos, &below_state, fluid.kind) {
        spread_to_sides(pos, state, fluid, get_block, changes, schedule);
    }
}

fn spread_to_sides(
    pos: BlockPos,
    state: &BlockStateModel,
    fluid: FluidState,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
    changes: &mut Vec<(BlockPos, BlockStateModel)>,
    schedule: &mut Vec<BlockPos>,
) {
    let mut neighbor = i32::from(fluid.level) - drop_off(fluid.kind);
    if fluid.falling {
        neighbor = 7;
    }
    if neighbor <= 0 {
        return;
    }
    for (direction, new_fluid) in get_spread(pos, state, fluid.kind, get_block) {
        let target = pos.relative(direction);
        let target_state = get_block(target);
        spread_to(target, &target_state, direction, new_fluid, changes, schedule);
    }
}

fn spread_to(
    pos: BlockPos,
    state: &BlockStateModel,
    direction: Direction,
    target: FluidState,
    changes: &mut Vec<(BlockPos, BlockStateModel)>,
    schedule: &mut Vec<BlockPos>,
) {
    if direction == Direction::Down
        && target.kind == FluidKind::Lava
        && fluid_state_for_block(state).is_some_and(|fluid| fluid.kind == FluidKind::Water)
    {
        changes.push((pos, BlockStateModel::new("minecraft:stone")));
        schedule.push(pos);
        return;
    }
    changes.push((pos, target.block_state()));
    schedule.push(pos);
}

fn get_new_liquid(
    pos: BlockPos,
    state: &BlockStateModel,
    kind: FluidKind,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
) -> Option<FluidState> {
    let mut highest_neighbor = 0_u8;
    let mut source_neighbors = 0;
    for direction in horizontal_directions() {
        let relative_pos = pos.relative(direction);
        let block_state = get_block(relative_pos);
        let fluid_state = fluid_state_for_block(&block_state);
        if fluid_state.is_some_and(|fluid| fluid.kind == kind)
            && can_pass_through_wall(direction, state, &block_state)
        {
            let fluid = fluid_state.unwrap();
            if fluid.is_source() {
                source_neighbors += 1;
            }
            highest_neighbor = highest_neighbor.max(fluid.level);
        }
    }

    if source_neighbors >= 2 && kind == FluidKind::Water {
        let below = get_block(pos.relative(Direction::Down));
        let below_fluid = fluid_state_for_block(&below);
        if block_is_solid(&below) || below_fluid.is_some_and(|fluid| fluid.kind == kind && fluid.is_source()) {
            return Some(kind.source());
        }
    }

    let above_pos = pos.relative(Direction::Up);
    let above = get_block(above_pos);
    let above_fluid = fluid_state_for_block(&above);
    if above_fluid.is_some_and(|fluid| fluid.kind == kind)
        && can_pass_through_wall(Direction::Up, state, &above)
    {
        return Some(FluidState {
            kind,
            level: 8,
            falling: true,
        });
    }

    let amount = i32::from(highest_neighbor) - drop_off(kind);
    (amount > 0).then_some(FluidState {
        kind,
        level: amount as u8,
        falling: false,
    })
}

fn get_spread(
    pos: BlockPos,
    state: &BlockStateModel,
    kind: FluidKind,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
) -> Vec<(Direction, FluidState)> {
    let mut lowest = 1000;
    let mut result = Vec::new();
    for direction in horizontal_directions() {
        let test_pos = pos.relative(direction);
        let test_state = get_block(test_pos);
        let test_fluid = fluid_state_for_block(&test_state);
        if can_maybe_pass_through(
            pos,
            state,
            direction,
            test_pos,
            &test_state,
            test_fluid,
            kind,
        ) {
            let Some(new_fluid) = get_new_liquid(test_pos, &test_state, kind, get_block) else {
                continue;
            };
            if !can_hold_specific_fluid(&test_state, new_fluid.kind) {
                continue;
            }
            let distance = if is_hole(test_pos, &test_state, kind, get_block) {
                0
            } else {
                slope_distance(test_pos, 1, direction.opposite(), &test_state, kind, get_block)
            };
            if distance < lowest {
                result.clear();
            }
            if distance <= lowest && can_replace_fluid(test_fluid, new_fluid.kind, direction) {
                result.push((direction, new_fluid));
                lowest = distance;
            }
        }
    }
    result
}

fn slope_distance(
    pos: BlockPos,
    pass: i32,
    from: Direction,
    state: &BlockStateModel,
    kind: FluidKind,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
) -> i32 {
    let mut lowest = 1000;
    for direction in horizontal_directions() {
        if direction == from {
            continue;
        }
        let test_pos = pos.relative(direction);
        let test_state = get_block(test_pos);
        let test_fluid = fluid_state_for_block(&test_state);
        if can_pass_through(pos, state, direction, test_pos, &test_state, test_fluid, kind) {
            if is_hole(test_pos, &test_state, kind, get_block) {
                return pass;
            }
            if pass < slope_find_distance(kind) {
                lowest = lowest.min(slope_distance(
                    test_pos,
                    pass + 1,
                    direction.opposite(),
                    &test_state,
                    kind,
                    get_block,
                ));
            }
        }
    }
    lowest
}

fn can_pass_through(
    source_pos: BlockPos,
    source_state: &BlockStateModel,
    direction: Direction,
    test_pos: BlockPos,
    test_state: &BlockStateModel,
    test_fluid: Option<FluidState>,
    kind: FluidKind,
) -> bool {
    can_maybe_pass_through(
        source_pos,
        source_state,
        direction,
        test_pos,
        test_state,
        test_fluid,
        kind,
    ) && can_hold_specific_fluid(test_state, kind)
}

fn can_maybe_pass_through(
    _source_pos: BlockPos,
    source_state: &BlockStateModel,
    direction: Direction,
    _test_pos: BlockPos,
    test_state: &BlockStateModel,
    test_fluid: Option<FluidState>,
    kind: FluidKind,
) -> bool {
    !test_fluid.is_some_and(|fluid| fluid.kind == kind && fluid.is_source())
        && can_hold_any_fluid(test_state)
        && can_pass_through_wall(direction, source_state, test_state)
}

fn can_pass_through_wall(
    _direction: Direction,
    source_state: &BlockStateModel,
    target_state: &BlockStateModel,
) -> bool {
    !block_is_full_cube(source_state) && !block_is_full_cube(target_state)
}

fn can_hold_any_fluid(state: &BlockStateModel) -> bool {
    if state.property("waterlogged").is_some() {
        return true;
    }
    if state.is_air() || fluid_state_for_block(state).is_some() {
        return true;
    }
    if block_is_solid(state) {
        return false;
    }
    if representative_state_definition(state.registry_id.as_str()).is_none() {
        return false;
    }
    let id = state.registry_id.as_str();
    !matches!(
        id,
        "minecraft:ladder"
            | "minecraft:sugar_cane"
            | "minecraft:bubble_column"
            | "minecraft:nether_portal"
            | "minecraft:end_portal"
            | "minecraft:end_gateway"
            | "minecraft:structure_void"
    ) && !id.ends_with("_door")
        && !id.ends_with("_sign")
        && !id.ends_with("_wall_sign")
        && !id.ends_with("_hanging_sign")
        && !id.ends_with("_wall_hanging_sign")
}

fn can_hold_specific_fluid(state: &BlockStateModel, kind: FluidKind) -> bool {
    state.property("waterlogged").is_none() || kind == FluidKind::Water
}

fn can_replace_fluid(existing: Option<FluidState>, new_kind: FluidKind, direction: Direction) -> bool {
    match existing {
        None => true,
        Some(fluid) if fluid.kind == new_kind && !fluid.is_source() => true,
        Some(fluid) if fluid.kind == FluidKind::Water && new_kind == FluidKind::Lava => direction == Direction::Down,
        Some(fluid) if fluid.kind == FluidKind::Lava && new_kind == FluidKind::Water => fluid_height(fluid) >= 0.44444445,
        _ => false,
    }
}

fn is_water_hole(
    top_pos: BlockPos,
    top_state: &BlockStateModel,
    bottom_pos: BlockPos,
    bottom_state: &BlockStateModel,
    kind: FluidKind,
) -> bool {
    if !can_pass_through_wall(Direction::Down, top_state, bottom_state) {
        return false;
    }
    let bottom_fluid = fluid_state_for_block(bottom_state);
    bottom_fluid.is_some_and(|fluid| fluid.kind == kind)
        || (bottom_pos.y < top_pos.y
            && can_hold_any_fluid(bottom_state)
            && can_hold_specific_fluid(bottom_state, kind))
}

fn is_hole(
    pos: BlockPos,
    state: &BlockStateModel,
    kind: FluidKind,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
) -> bool {
    let below_pos = pos.relative(Direction::Down);
    let below_state = get_block(below_pos);
    is_water_hole(pos, state, below_pos, &below_state, kind)
}

fn source_neighbor_count(
    pos: BlockPos,
    kind: FluidKind,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
) -> i32 {
    horizontal_directions()
        .into_iter()
        .filter(|direction| {
            fluid_state_for_block(&get_block(pos.relative(*direction)))
                .is_some_and(|fluid| fluid.kind == kind && fluid.is_source())
        })
        .count() as i32
}

fn lava_should_convert_to_solid(
    pos: BlockPos,
    lava: FluidState,
    get_block: &impl Fn(BlockPos) -> BlockStateModel,
) -> bool {
    if lava.kind != FluidKind::Lava {
        return false;
    }
    fluid_neighbor_order().into_iter().any(|direction| {
        let neighbor = pos.relative(direction.opposite());
        fluid_state_for_block(&get_block(neighbor)).is_some_and(|fluid| fluid.kind == FluidKind::Water)
    })
}

fn block_is_solid(state: &BlockStateModel) -> bool {
    if state.is_air() || fluid_state_for_block(state).is_some() {
        return false;
    }
    representative_state_definition(state.registry_id.as_str())
        .map(|definition| definition.physical.has_collision)
        .unwrap_or(true)
}

fn block_is_full_cube(state: &BlockStateModel) -> bool {
    if state.is_air() || fluid_state_for_block(state).is_some() {
        return false;
    }
    representative_state_definition(state.registry_id.as_str())
        .map(|definition| definition.collision_shape == ShapeKind::FullCube)
        .unwrap_or(true)
}

fn drop_off(kind: FluidKind) -> i32 {
    match kind {
        FluidKind::Water => 1,
        FluidKind::Lava => 2,
    }
}

fn slope_find_distance(kind: FluidKind) -> i32 {
    match kind {
        FluidKind::Water => 4,
        FluidKind::Lava => 2,
    }
}

fn fluid_height(fluid: FluidState) -> f32 {
    f32::from(fluid.level) / 9.0
}

fn horizontal_directions() -> [Direction; 4] {
    [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ]
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
        block_item_can_replace, block_state_model_name, can_hold_water, can_place_liquid,
        fluid_neighbor_order, fluid_state_for_block, pickup_liquid, place_liquid,
        plan_fluid_update, save_fluid_ticks, tick_fluid, FluidKind, LiquidPickupResult,
        LiquidPlaceResult,
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
    fn normal_block_items_can_replace_liquid_blocks_but_not_solid_blocks() {
        assert!(block_item_can_replace(&BlockStateModel::air()));
        assert!(block_item_can_replace(&BlockStateModel::new("minecraft:water")));
        assert!(block_item_can_replace(
            &BlockStateModel::new("minecraft:lava").with_property("level", "8")
        ));
        assert!(!block_item_can_replace(&BlockStateModel::new("minecraft:stone")));
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

    #[test]
    fn source_water_tick_flows_down_with_falling_legacy_level() {
        let source = BlockStateModel::new("minecraft:water").with_property("level", "0");
        let result = tick_fluid(
            BlockPos { x: 0, y: 64, z: 0 },
            &source,
            |pos| match (pos.x, pos.y, pos.z) {
                (0, 63, 0) => BlockStateModel::air(),
                _ => BlockStateModel::new("minecraft:stone"),
            },
        );
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].0, BlockPos { x: 0, y: 63, z: 0 });
        assert_eq!(block_state_model_name(&result.changes[0].1), "minecraft:water[level=8]");
    }

    #[test]
    fn flowing_water_tick_drains_without_source_neighbors() {
        let flowing = BlockStateModel::new("minecraft:water").with_property("level", "4");
        let result = tick_fluid(BlockPos { x: 0, y: 64, z: 0 }, &flowing, |_| {
            BlockStateModel::air()
        });
        assert_eq!(result.changes[0].0, BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(result.changes[0].1, BlockStateModel::air());
    }

    #[test]
    fn two_water_sources_regenerate_middle_source_like_vanilla_gamerule_default() {
        let flowing = BlockStateModel::new("minecraft:water").with_property("level", "4");
        let result = tick_fluid(
            BlockPos { x: 0, y: 64, z: 0 },
            &flowing,
            |pos| match (pos.x, pos.y, pos.z) {
                (-1, 64, 0) | (1, 64, 0) => {
                    BlockStateModel::new("minecraft:water").with_property("level", "0")
                }
                (0, 63, 0) => BlockStateModel::new("minecraft:stone"),
                _ => BlockStateModel::air(),
            },
        );
        assert_eq!(result.changes[0].0, BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(block_state_model_name(&result.changes[0].1), "minecraft:water[level=0]");
    }
}
