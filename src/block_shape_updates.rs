//! Java `BlockBehaviour.updateShape` ports: the per-block reaction when a
//! neighbour changes, dispatched on the official block-type key like
//! [`crate::block_survival`] and [`crate::block_placement`].
//!
//! Java's contract: `Level.setBlock` triggers `updateShapeAtEdge` on the six
//! neighbours; each neighbour returns its replacement state (often itself,
//! `Blocks.AIR` when its support broke, or a state with one connection
//! property recomputed) and may schedule fluid/block ticks through
//! `ScheduledTickAccess`. [`ShapeUpdate`] carries all three effects; callers
//! apply the state change (recursively re-running neighbour updates, capped by
//! Java's 512 chain limit in `block_update`) and schedule the requested ticks.
//!
//! Coverage mirrors the other catalogs: [`update_shape`] returns `None` for
//! the few overrides blocked on other subsystems (`UNPORTED_SHAPE_UPDATES`),
//! `Some(unchanged)` for blocks without an override, and the exact ported
//! behavior otherwise.

#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_placement::{
    axis_name, bool_str, clockwise, default_state, direction_by_name, direction_name,
    is_horizontal, set, stairs_shape, PlacementWorld,
};
use crate::block_properties::{state_physics_by_name, StateFluid, SupportType};
use crate::block_states::block_state_entry;
use crate::block_survival::{can_survive, multiface_can_attach_to, SurvivalWorld};
use crate::block_tags::block_tag_contains;
use crate::block_update::{BlockPos, Direction};

/// Java `Fluids.WATER.getTickDelay(level)` = 5 for the overworld.
pub const WATER_TICK_DELAY: i32 = 5;

/// The effects of one `updateShape` call.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeUpdate {
    /// The replacement state (`Blocks.AIR` pops the block off).
    pub state: BlockStateModel,
    /// Java `ticks.scheduleTick(pos, Fluids.WATER, ...)` — waterlogged drain.
    pub schedule_fluid_tick: bool,
    /// Java `ticks.scheduleTick(pos, this, delay)`.
    pub schedule_block_tick: Option<i32>,
}

impl ShapeUpdate {
    fn keep(state: &BlockStateModel) -> Self {
        Self {
            state: state.clone(),
            schedule_fluid_tick: false,
            schedule_block_tick: None,
        }
    }

    fn air() -> Self {
        Self {
            state: BlockStateModel::air(),
            schedule_fluid_tick: false,
            schedule_block_tick: None,
        }
    }

    fn with_fluid_tick(mut self, waterlogged: bool) -> Self {
        self.schedule_fluid_tick |= waterlogged;
        self
    }

    fn with_block_tick(mut self, delay: i32) -> Self {
        self.schedule_block_tick = Some(delay);
        self
    }
}

/// Overrides blocked on other subsystems: redstone wire (signal graph re-walk
/// on UP/sides), note blocks (instrument sampling), nether portals (frame
/// validation), liquids/bubble columns (fluid engine), beehives (bee
/// entities), copper chests (weathering pairing), and observers' pulse
/// scheduling beyond the simple tick.
pub const UNPORTED_SHAPE_UPDATES: &[&str] = &[
    "redstone_wire",
    "note",
    "nether_portal",
    "liquid",
    "bubble_column",
    "beehive",
    "copper_chest",
    "weathering_copper_chest",
];

fn waterlogged(state: &BlockStateModel) -> bool {
    state.property("waterlogged") == Some("true")
}

fn unsupported(state: &BlockStateModel, pos: BlockPos, world: &impl SurvivalWorld) -> bool {
    !can_survive(state, pos, world)
}

fn facing_of(state: &BlockStateModel) -> Direction {
    state
        .property("facing")
        .and_then(direction_by_name)
        .unwrap_or(Direction::North)
}

fn is_type(state: &BlockStateModel, types: &[&str]) -> bool {
    block_state_entry(&state.registry_id).is_some_and(|entry| types.contains(&entry.block_type))
}

/// Java `BlockState.updateShape(...)` toward one changed neighbour. Returns
/// `None` for unported overrides (callers keep their previous behavior).
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)] // one arm per Java class catalog
pub fn update_shape(
    state: &BlockStateModel,
    pos: BlockPos,
    direction_to_neighbour: Direction,
    _neighbour_pos: BlockPos,
    neighbour_state: &BlockStateModel,
    random_roll: i32,
    world: &impl PlacementWorld,
) -> Option<ShapeUpdate> {
    let entry = block_state_entry(&state.registry_id)?;
    let block_type = entry.block_type;
    if UNPORTED_SHAPE_UPDATES.contains(&block_type) {
        return None;
    }
    let direction = direction_to_neighbour;

    let update = match block_type {
        // ---- canSurvive from any direction -> AIR ----
        // VegetationBlock family (TallGrassBlock keys as `tall_grass`),
        // carpets, snow layers, frogspawn, mushrooms.
        "tall_grass" | "bush" | "flower" | "flower_bed" | "cactus_flower" | "eyeblossom"
        | "firefly_bush" | "sapling" | "azalea" | "lily_pad" | "sweet_berry_bush"
        | "dry_vegetation" | "short_dry_grass" | "tall_dry_grass" | "nether_roots"
        | "nether_sprouts" | "nether_fungus" | "nether_wart" | "wither_rose" | "mushroom"
        | "carpet" | "wool_carpet" | "snow_layer" | "frogspawn" | "leaf_litter" => {
            if unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state)
            }
        }
        // Seagrass keeps the fluid tick alive when it survives.
        "seagrass" => {
            if unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(true)
            }
        }
        "sea_pickle" => {
            if unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        // MossyCarpetBlock recomputes its arms (and dies without any).
        "mossy_carpet" => {
            if unsupported(state, pos, world) {
                return Some(ShapeUpdate::air());
            }
            let context = mossy_recompute_context(pos);
            match crate::block_placement::connecting::mossy_carpet_placement(
                state.clone(),
                &context,
                world,
            ) {
                crate::block_placement::PlacementOutcome::Place(updated) => {
                    let has_face = updated.property("base") == Some("true")
                        || ["north", "south", "west", "east"]
                            .iter()
                            .any(|face| updated.property(face).is_some_and(|side| side != "none"));
                    if has_face {
                        ShapeUpdate::keep(&updated)
                    } else {
                        ShapeUpdate::air()
                    }
                }
                crate::block_placement::PlacementOutcome::Reject => ShapeUpdate::air(),
            }
        }

        // ---- canSurvive only when the support neighbour changed ----
        // DOWN-supported blocks.
        "banner"
        | "standing_sign"
        | "cake"
        | "candle_cake"
        | "pressure_plate"
        | "weighted_pressure_plate"
        | "torch"
        | "redstone_torch"
        | "flower_pot" => {
            if direction == Direction::Down && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state)
            }
        }
        // DiodeBlock: below must stay RIGID-sturdy.
        "repeater" | "comparator" => {
            let below_rigid =
                state_physics_by_name(&neighbour_state.state_name()).is_some_and(|physics| {
                    crate::block_properties::is_face_sturdy(physics, 1, SupportType::Rigid)
                });
            if direction == Direction::Down && !below_rigid {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state)
            }
        }
        // Coral plants/fans: DOWN support + waterlogged tick (the die tick is
        // handled with the coral block arm below).
        "base_coral_plant" | "base_coral_fan" => {
            if direction == Direction::Down && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "coral_plant" | "coral_fan" => {
            if direction == Direction::Down && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                let update = ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state));
                coral_die_tick(state, pos, world, random_roll, update)
            }
        }
        "coral" => {
            let update = ShapeUpdate::keep(state);
            coral_die_tick(state, pos, world, random_roll, update)
        }
        "coral_wall_fan" => {
            if direction.opposite() == facing_of(state) && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                let update = ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state));
                coral_die_tick(state, pos, world, random_roll, update)
            }
        }
        "base_coral_wall_fan" => {
            if direction.opposite() == facing_of(state) && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }

        // Facing-attached blocks pop when their wall breaks.
        "wall_torch"
        | "redstone_wall_torch"
        | "wall_banner"
        | "wall_sign"
        | "ladder"
        | "trip_wire_hook"
        | "piston_head" => {
            let pops = direction.opposite() == facing_of(state) && unsupported(state, pos, world);
            if pops {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "cocoa" => {
            if direction == facing_of(state) && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state)
            }
        }
        // FaceAttachedHorizontalDirectionalBlock (buttons, levers).
        "button" | "lever" => {
            let connected = match state.property("face") {
                Some("ceiling") => Direction::Down,
                Some("floor") => Direction::Up,
                _ => facing_of(state),
            };
            if connected.opposite() == direction && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "amethyst_cluster" => {
            if direction == facing_of(state).opposite() && unsupported(state, pos, world) {
                ShapeUpdate::air().with_fluid_tick(waterlogged(state))
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "hanging_roots" => {
            if direction == Direction::Up && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "ceiling_hanging_sign" => {
            if direction == Direction::Up && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "wall_hanging_sign" => {
            if axis_name(direction) == axis_name(clockwise(facing_of(state)))
                && unsupported(state, pos, world)
            {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "mangrove_propagule" => {
            if direction == Direction::Up && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "spore_blossom" => {
            if direction == Direction::Up && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "lantern" | "weathering_lantern" => {
            let connected = if state.property("hanging") == Some("true") {
                Direction::Up
            } else {
                Direction::Down
            };
            if connected == direction && unsupported(state, pos, world) {
                ShapeUpdate::air().with_fluid_tick(waterlogged(state))
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "bell" => {
            let attachment = state.property("attachment").unwrap_or("floor");
            let connected = match attachment {
                "floor" => Direction::Up,
                "ceiling" => Direction::Down,
                _ => facing_of(state),
            };
            if connected.opposite() == direction
                && unsupported(state, pos, world)
                && attachment != "double_wall"
            {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state)
            }
        }

        // ---- canSurvive failure schedules a deferred break tick ----
        "cactus" | "sugar_cane" | "chorus_plant" => {
            let update = if block_type == "chorus_plant" && !unsupported(state, pos, world) {
                // Recompute the touched connection like getStateWithConnections.
                let connects = neighbour_state.registry_id == state.registry_id
                    || neighbour_state.registry_id == "minecraft:chorus_flower"
                    || direction == Direction::Down
                        && block_tag_contains(
                            "supports_chorus_plant",
                            &neighbour_state.registry_id,
                        );
                ShapeUpdate::keep(&set(
                    state.clone(),
                    direction_name(direction),
                    bool_str(connects),
                ))
            } else {
                ShapeUpdate::keep(state)
            };
            if unsupported(state, pos, world) {
                update.with_block_tick(1)
            } else {
                update
            }
        }
        "chorus_flower" => {
            if direction != Direction::Up && unsupported(state, pos, world) {
                ShapeUpdate::keep(state).with_block_tick(1)
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "dirt_path" | "farmland" => {
            if direction == Direction::Up && unsupported(state, pos, world) {
                ShapeUpdate::keep(state).with_block_tick(1)
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "bamboo_stalk" => {
            let mut update = if direction == Direction::Up
                && neighbour_state.registry_id == "minecraft:bamboo"
                && age_of(neighbour_state) > age_of(state)
            {
                // Java state.cycle(AGE): 0 -> 1.
                ShapeUpdate::keep(&set(state.clone(), "age", "1"))
            } else {
                ShapeUpdate::keep(state)
            };
            if unsupported(state, pos, world) {
                update = update.with_block_tick(1);
            }
            update
        }
        "bamboo_sapling" => {
            if unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else if direction == Direction::Up
                && neighbour_state.registry_id == "minecraft:bamboo"
            {
                ShapeUpdate::keep(&default_state("minecraft:bamboo"))
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "big_dripleaf" => {
            if direction == Direction::Down && unsupported(state, pos, world) {
                ShapeUpdate::air()
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "big_dripleaf_stem" => {
            let mut update = ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state));
            if matches!(direction, Direction::Down | Direction::Up)
                && unsupported(state, pos, world)
            {
                update = update.with_block_tick(1);
            }
            update
        }
        "small_dripleaf" | "pitcher_crop" | "tall_seagrass" | "double_plant" | "tall_flower" => {
            double_half_update(block_type, state, pos, direction, neighbour_state, world)
        }
        "door" | "weathering_copper_door" => {
            door_half_update(state, pos, direction, neighbour_state, world)
        }
        "bed" => bed_part_update(state, direction, neighbour_state),

        // ---- fluid-tick only ----
        "barrier"
        | "rail"
        | "powered_rail"
        | "detector_rail"
        | "candle"
        | "chain"
        | "weathering_copper_chain"
        | "conduit"
        | "copper_golem_statue"
        | "weathering_copper_golem_statue"
        | "decorated_pot"
        | "dried_ghast"
        | "ender_chest"
        | "heavy_core"
        | "light"
        | "lightning_rod"
        | "weathering_lightning_rod"
        | "mangrove_roots"
        | "sculk_sensor"
        | "calibrated_sculk_sensor"
        | "sculk_shrieker"
        | "shelf"
        | "sign"
        | "slab"
        | "weathering_copper_slab"
        | "trapdoor"
        | "weathering_copper_trap_door"
        | "waterlogged_transparent"
        | "iron_chain" => ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state)),

        // ---- connection recomputes ----
        "fence" => {
            if is_horizontal(direction) {
                let connects = crate::block_placement::connecting::fence_connects_to(
                    &state.registry_id,
                    neighbour_state,
                    direction.opposite(),
                    world,
                );
                ShapeUpdate::keep(&set(
                    state.clone(),
                    direction_name(direction),
                    bool_str(connects),
                ))
                .with_fluid_tick(waterlogged(state))
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "iron_bars" | "stained_glass_pane" | "weathering_copper_bar" => {
            if is_horizontal(direction) {
                let attaches = crate::block_placement::connecting::pane_attaches_to(
                    neighbour_state,
                    direction.opposite(),
                );
                ShapeUpdate::keep(&set(
                    state.clone(),
                    direction_name(direction),
                    bool_str(attaches),
                ))
                .with_fluid_tick(waterlogged(state))
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "wall" => {
            // Java updates the touched side + UP; a full recompute via the
            // placement routine yields the same result.
            if direction == Direction::Down {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            } else {
                let context = mossy_recompute_context(pos);
                match crate::block_placement::connecting::wall_placement(
                    state.clone(),
                    &context,
                    world,
                ) {
                    crate::block_placement::PlacementOutcome::Place(mut updated) => {
                        if let Some(value) = state.property("waterlogged") {
                            updated = set(updated, "waterlogged", value);
                        }
                        ShapeUpdate::keep(&updated).with_fluid_tick(waterlogged(state))
                    }
                    crate::block_placement::PlacementOutcome::Reject => {
                        ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
                    }
                }
            }
        }
        "fence_gate" => {
            if axis_name(direction) != axis_name(clockwise(facing_of(state))) {
                ShapeUpdate::keep(state)
            } else {
                let in_wall = block_tag_contains("walls", &neighbour_state.registry_id)
                    || block_tag_contains(
                        "walls",
                        &world
                            .state_at(pos.relative(direction.opposite()))
                            .registry_id,
                    );
                ShapeUpdate::keep(&set(state.clone(), "in_wall", bool_str(in_wall)))
            }
        }
        "tripwire" => {
            if is_horizontal(direction) {
                let connects = crate::block_placement::connecting::tripwire_connects_to(
                    neighbour_state,
                    direction,
                );
                ShapeUpdate::keep(&set(
                    state.clone(),
                    direction_name(direction),
                    bool_str(connects),
                ))
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "stair" | "weathering_copper_stair" => {
            if is_horizontal(direction) {
                let shape = stairs_shape(state, pos, world);
                ShapeUpdate::keep(&set(state.clone(), "shape", shape))
                    .with_fluid_tick(waterlogged(state))
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "chest" | "trapped_chest" => {
            chest_pair_update(state, direction, neighbour_state).with_fluid_tick(waterlogged(state))
        }
        "huge_mushroom" => {
            if neighbour_state.registry_id == state.registry_id {
                ShapeUpdate::keep(&set(state.clone(), direction_name(direction), "false"))
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "campfire" => {
            if direction == Direction::Down {
                let signal = neighbour_state.registry_id == "minecraft:hay_block";
                ShapeUpdate::keep(&set(state.clone(), "signal_fire", bool_str(signal)))
                    .with_fluid_tick(waterlogged(state))
            } else {
                ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
            }
        }
        "snowy_dirt" | "grass" | "mycelium" | "nylium" => {
            if direction == Direction::Up && state.has_property("snowy") {
                let snowy = block_tag_contains("snow", &neighbour_state.registry_id);
                ShapeUpdate::keep(&set(state.clone(), "snowy", bool_str(snowy)))
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "attached_stem" => {
            // Back to a plain stem when the fruit disappears.
            let fruit = if state.registry_id.contains("melon") {
                "minecraft:melon"
            } else {
                "minecraft:pumpkin"
            };
            if direction == facing_of(state) && neighbour_state.registry_id != fruit {
                let stem = if state.registry_id.contains("melon") {
                    "minecraft:melon_stem"
                } else {
                    "minecraft:pumpkin_stem"
                };
                ShapeUpdate::keep(&set(default_state(stem), "age", "7"))
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "leaves" | "mangrove_leaves" | "tinted_particle_leaves" | "untinted_particle_leaves" => {
            let neighbour_distance =
                crate::block_placement::connecting::leaves_distance_at(neighbour_state) + 1;
            let own_distance = state
                .property("distance")
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(7);
            let update = ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state));
            if neighbour_distance != 1 || own_distance != neighbour_distance {
                update.with_block_tick(1)
            } else {
                update
            }
        }
        "concrete_powder" => {
            // ConcretePowderBlock extends FallingBlock: the super chain also
            // schedules the 2-tick fall check.
            if crate::block_placement::connecting::concrete_should_solidify(world, pos) {
                ShapeUpdate::keep(&default_state(&state.registry_id.replace("_powder", "")))
                    .with_block_tick(2)
            } else {
                ShapeUpdate::keep(state).with_block_tick(2)
            }
        }
        "multiface" | "glow_lichen" | "sculk_vein" => {
            let update = multiface_face_update(state, pos, direction, neighbour_state, world);
            update.with_fluid_tick(waterlogged(state))
        }
        "vine" => {
            if direction == Direction::Down {
                ShapeUpdate::keep(state)
            } else {
                let face = direction_name(direction);
                if state.property(face) == Some("true")
                    && !multiface_can_attach_to(neighbour_state, direction)
                    && !vine_face_held_from_above(state, pos, direction, world)
                {
                    let updated = set(state.clone(), face, "false");
                    if vine_has_any_face(&updated) {
                        ShapeUpdate::keep(&updated)
                    } else {
                        ShapeUpdate::air()
                    }
                } else {
                    ShapeUpdate::keep(state)
                }
            }
        }
        "hanging_moss" => {
            let mut update = ShapeUpdate::keep(&set(
                state.clone(),
                "tip",
                bool_str(
                    world.state_at(pos.relative(Direction::Down)).registry_id != state.registry_id,
                ),
            ));
            if unsupported(state, pos, world) {
                update = update.with_block_tick(1);
            }
            update
        }
        "kelp" | "twisting_vines" | "weeping_vines" | "cave_vines" => {
            growing_head_update(block_type, state, pos, direction, neighbour_state, world)
        }
        "kelp_plant" | "twisting_vines_plant" | "weeping_vines_plant" | "cave_vines_plant" => {
            growing_body_update(
                block_type,
                state,
                pos,
                direction,
                neighbour_state,
                random_roll,
                world,
            )
        }
        "pointed_dripstone" => {
            let mut update = ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state));
            if matches!(direction, Direction::Up | Direction::Down)
                && unsupported(state, pos, world)
            {
                update = update.with_block_tick(
                    if state.property("vertical_direction") == Some("down") {
                        2
                    } else {
                        1
                    },
                );
            }
            update
        }

        // ---- deferred-tick blocks ----
        "scaffolding" => ShapeUpdate::keep(state)
            .with_fluid_tick(waterlogged(state))
            .with_block_tick(1),
        "brushable" => ShapeUpdate::keep(state).with_block_tick(2),
        "creaking_heart" => ShapeUpdate::keep(state).with_block_tick(1),
        // FallingBlock family: getDelayAfterPlace() = 2 (anvils and dragon
        // eggs fall too; brushables add it on their own arm above).
        "colored_falling" | "sand" | "anvil" | "dragon_egg" => {
            ShapeUpdate::keep(state).with_block_tick(2)
        }
        "observer" => {
            if facing_of(state) == direction && state.property("powered") != Some("true") {
                ShapeUpdate::keep(state).with_block_tick(2)
            } else {
                ShapeUpdate::keep(state)
            }
        }
        "fire" => {
            if can_survive(state, pos, world) {
                // getStateWithAge re-derives the face set at the same age.
                let context = mossy_recompute_context(pos);
                match crate::block_placement::state_for_placement("minecraft:fire", &context, world)
                {
                    Some(crate::block_placement::PlacementOutcome::Place(updated)) => {
                        let age = state.property("age").unwrap_or("0");
                        ShapeUpdate::keep(&set(updated, "age", age))
                    }
                    _ => ShapeUpdate::keep(state),
                }
            } else {
                ShapeUpdate::air()
            }
        }
        "soul_fire" => {
            if can_survive(state, pos, world) {
                ShapeUpdate::keep(state)
            } else {
                ShapeUpdate::air()
            }
        }

        _ => ShapeUpdate::keep(state),
    };
    Some(update)
}

fn age_of(state: &BlockStateModel) -> i32 {
    state
        .property("age")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0)
}

/// Coral family `tryScheduleDieTick`: without adjacent water the coral dies
/// after `60 + nextInt(40)` ticks.
fn coral_die_tick(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
    random_roll: i32,
    update: ShapeUpdate,
) -> ShapeUpdate {
    if waterlogged(state) {
        return update;
    }
    let has_water = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ]
    .iter()
    .any(|direction| {
        let neighbour = world.state_at(pos.relative(*direction));
        matches!(
            state_physics_by_name(&neighbour.state_name())
                .map_or(StateFluid::Empty, |physics| physics.fluid),
            StateFluid::Water { .. }
        )
    });
    if has_water {
        update
    } else {
        update.with_block_tick(60 + random_roll.rem_euclid(40))
    }
}

/// Shared DoublePlantBlock.updateShape (also small dripleaf, pitcher crop,
/// tall seagrass): vertical updates sync or break the two halves.
fn double_half_update(
    block_type: &str,
    state: &BlockStateModel,
    pos: BlockPos,
    direction: Direction,
    neighbour_state: &BlockStateModel,
    world: &impl PlacementWorld,
) -> ShapeUpdate {
    // PitcherCropBlock only acts double once grown (age >= 3 per isDouble).
    if block_type == "pitcher_crop" && age_of(state) < 3 {
        return if can_survive(state, pos, world) {
            ShapeUpdate::keep(state)
        } else {
            ShapeUpdate::air()
        };
    }
    let half = state.property("half").unwrap_or("lower");
    let toward_other = if half == "lower" {
        Direction::Up
    } else {
        Direction::Down
    };
    if direction == toward_other {
        let neighbour_is_other_half = neighbour_state.registry_id == state.registry_id
            && neighbour_state.property("half") != Some(half);
        if !neighbour_is_other_half {
            return ShapeUpdate::air();
        }
        return ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state));
    }
    if half == "lower" && direction == Direction::Down && unsupported(state, pos, world) {
        return ShapeUpdate::air();
    }
    ShapeUpdate::keep(state).with_fluid_tick(waterlogged(state))
}

/// Java `DoorBlock.updateShape`.
fn door_half_update(
    state: &BlockStateModel,
    pos: BlockPos,
    direction: Direction,
    neighbour_state: &BlockStateModel,
    world: &impl PlacementWorld,
) -> ShapeUpdate {
    let half = state.property("half").unwrap_or("lower");
    let toward_other = if half == "lower" {
        Direction::Up
    } else {
        Direction::Down
    };
    if direction == toward_other {
        let neighbour_is_door = is_type(neighbour_state, &["door", "weathering_copper_door"]);
        if neighbour_is_door && neighbour_state.property("half") != Some(half) {
            // Copy the counterpart's state onto this half.
            return ShapeUpdate::keep(&set(neighbour_state.clone(), "half", half));
        }
        return ShapeUpdate::air();
    }
    if half == "lower" && direction == Direction::Down && unsupported(state, pos, world) {
        return ShapeUpdate::air();
    }
    ShapeUpdate::keep(state)
}

/// Java `BedBlock.updateShape`.
fn bed_part_update(
    state: &BlockStateModel,
    direction: Direction,
    neighbour_state: &BlockStateModel,
) -> ShapeUpdate {
    let facing = facing_of(state);
    let part = state.property("part").unwrap_or("foot");
    // getNeighbourDirection: HEAD looks opposite the facing, FOOT along it.
    let neighbour_direction = if part == "head" {
        facing.opposite()
    } else {
        facing
    };
    if direction != neighbour_direction {
        return ShapeUpdate::keep(state);
    }
    if neighbour_state.registry_id == state.registry_id
        && neighbour_state.property("part") != Some(part)
    {
        let occupied = neighbour_state.property("occupied").unwrap_or("false");
        ShapeUpdate::keep(&set(state.clone(), "occupied", occupied))
    } else {
        ShapeUpdate::air()
    }
}

/// Java `ChestBlock.updateShape` pairing maintenance.
fn chest_pair_update(
    state: &BlockStateModel,
    direction: Direction,
    neighbour_state: &BlockStateModel,
) -> ShapeUpdate {
    let connected_direction = |chest: &BlockStateModel| {
        let facing = facing_of(chest);
        match chest.property("type") {
            Some("left") => clockwise(facing),
            Some("right") => crate::block_placement::counter_clockwise(facing),
            _ => facing,
        }
    };
    if neighbour_state.registry_id == state.registry_id && is_horizontal(direction) {
        let neighbour_type = neighbour_state.property("type").unwrap_or("single");
        if state.property("type") == Some("single")
            && neighbour_type != "single"
            && state.property("facing") == neighbour_state.property("facing")
            && connected_direction(neighbour_state) == direction.opposite()
        {
            let opposite = if neighbour_type == "left" {
                "right"
            } else {
                "left"
            };
            return ShapeUpdate::keep(&set(state.clone(), "type", opposite));
        }
    } else if connected_direction(state) == direction {
        return ShapeUpdate::keep(&set(state.clone(), "type", "single"));
    }
    ShapeUpdate::keep(state)
}

/// Java `MultifaceBlock.updateShape` face removal.
fn multiface_face_update(
    state: &BlockStateModel,
    _pos: BlockPos,
    direction: Direction,
    neighbour_state: &BlockStateModel,
    _world: &impl PlacementWorld,
) -> ShapeUpdate {
    let any_face = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ]
    .iter()
    .any(|face| state.property(direction_name(*face)) == Some("true"));
    if !any_face {
        return ShapeUpdate::air();
    }
    let face = direction_name(direction);
    if state.property(face) == Some("true") && !multiface_can_attach_to(neighbour_state, direction)
    {
        let updated = set(state.clone(), face, "false");
        let still_attached = [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ]
        .iter()
        .any(|other| updated.property(direction_name(*other)) == Some("true"));
        if still_attached {
            ShapeUpdate::keep(&updated)
        } else {
            ShapeUpdate::air()
        }
    } else {
        ShapeUpdate::keep(state)
    }
}

fn vine_has_any_face(state: &BlockStateModel) -> bool {
    ["up", "north", "south", "west", "east"]
        .iter()
        .any(|face| state.property(face) == Some("true"))
}

/// VineBlock side faces survive when the vine above holds the same face.
fn vine_face_held_from_above(
    state: &BlockStateModel,
    pos: BlockPos,
    direction: Direction,
    world: &impl PlacementWorld,
) -> bool {
    if direction == Direction::Up {
        return false;
    }
    let above = world.state_at(pos.relative(Direction::Up));
    above.registry_id == state.registry_id
        && above.property(direction_name(direction)) == Some("true")
}

/// Java `GrowingPlantHeadBlock.updateShape`.
fn growing_head_update(
    block_type: &str,
    state: &BlockStateModel,
    pos: BlockPos,
    direction: Direction,
    neighbour_state: &BlockStateModel,
    world: &impl PlacementWorld,
) -> ShapeUpdate {
    let grows_up = !block_type.starts_with("weeping");
    let growth = if grows_up {
        Direction::Up
    } else {
        Direction::Down
    };
    let (head, body) = growing_family(block_type);
    if direction == growth.opposite() {
        if unsupported(state, pos, world) {
            return ShapeUpdate::keep(state).with_block_tick(1);
        }
        let ahead = world.state_at(pos.relative(growth));
        if ahead.registry_id == head || ahead.registry_id == body {
            return ShapeUpdate::keep(&default_state(body));
        }
    }
    if direction == growth
        && (neighbour_state.registry_id == head || neighbour_state.registry_id == body)
    {
        return ShapeUpdate::keep(&default_state(body));
    }
    // Kelp schedules fluid ticks (underwater plant).
    ShapeUpdate::keep(state).with_fluid_tick(block_type == "kelp")
}

/// Java `GrowingPlantBodyBlock.updateShape`.
fn growing_body_update(
    block_type: &str,
    state: &BlockStateModel,
    pos: BlockPos,
    direction: Direction,
    neighbour_state: &BlockStateModel,
    random_roll: i32,
    world: &impl PlacementWorld,
) -> ShapeUpdate {
    let head_type = block_type.trim_end_matches("_plant");
    let grows_up = !head_type.starts_with("weeping");
    let growth = if grows_up {
        Direction::Up
    } else {
        Direction::Down
    };
    let (head, body) = growing_family(head_type);
    let mut update = ShapeUpdate::keep(state).with_fluid_tick(head_type == "kelp");
    if direction == growth.opposite() && unsupported(state, pos, world) {
        update = update.with_block_tick(1);
    }
    if direction == growth
        && neighbour_state.registry_id != head
        && neighbour_state.registry_id != body
    {
        // Convert back into a head with a fresh random age.
        return ShapeUpdate::keep(&set(
            default_state(head),
            "age",
            (random_roll.rem_euclid(25)).to_string(),
        ))
        .with_fluid_tick(head_type == "kelp");
    }
    update
}

fn growing_family(head_type: &str) -> (&'static str, &'static str) {
    match head_type {
        "kelp" => ("minecraft:kelp", "minecraft:kelp_plant"),
        "twisting_vines" => ("minecraft:twisting_vines", "minecraft:twisting_vines_plant"),
        "cave_vines" => ("minecraft:cave_vines", "minecraft:cave_vines_plant"),
        _ => ("minecraft:weeping_vines", "minecraft:weeping_vines_plant"),
    }
}

/// Connection recomputes reuse placement routines that read a `PlaceContext`;
/// only the position matters for them.
fn mossy_recompute_context(pos: BlockPos) -> crate::block_placement::PlaceContext {
    crate::block_placement::PlaceContext {
        clicked_pos: pos,
        clicked_face: Direction::Up,
        click_location: [
            f64::from(pos.x) + 0.5,
            f64::from(pos.y),
            f64::from(pos.z) + 0.5,
        ],
        replacing_clicked_on_block: false,
        player_yaw: 0.0,
        player_pitch: 0.0,
        secondary_use_active: false,
        random_age_roll: 0,
    }
}

#[cfg(test)]
mod tests;
