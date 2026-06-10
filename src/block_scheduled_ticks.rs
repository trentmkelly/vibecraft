//! Java `BlockBehaviour.tick` ports for scheduled block ticks — the deferred
//! reactions the `updateShape` catalog queues (support pops, leaf distance,
//! coral death, observer pulses), dispatched on the official block-type key.
//!
//! `None` means the override is blocked on another subsystem
//! (`FallingBlockEntity` for gravity/scaffolding/dripstone falls, the redstone
//! graph for observer signal emission beyond the state flip, fluids for
//! bubble columns); callers keep their previous behavior and the TODOs in the
//! live tick engine mark the seams.

#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_placement::{connecting, set, PlacementWorld};
use crate::block_properties::{state_physics_by_name, StateFluid};
use crate::block_states::block_state_entry;
use crate::block_survival::{can_survive, multiface_can_attach_to, SurvivalWorld};
use crate::block_update::{BlockPos, Direction};

/// The world mutation a scheduled tick performs.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockTickOutcome {
    /// Nothing to do (the condition resolved itself).
    None,
    /// Java `level.setBlock(pos, state, ...)`, optionally rescheduling.
    SetState {
        state: BlockStateModel,
        reschedule: Option<i32>,
    },
    /// Java `level.destroyBlock(pos, dropResources)`.
    Destroy { drop: bool },
}

/// Block types whose scheduled tick is not handled by this catalog. The
/// FallingBlock families are handled by the live engine
/// (`block_placement_live::process_live_block_ticks` intercepts them and
/// spawns `FallingBlockEntity` sims before this catalog runs); the rest are
/// blocked on other subsystems.
pub const UNPORTED_SCHEDULED_TICKS: &[&str] = &[
    // FallingBlockEntity spawns happen in the live engine:
    "sand",
    "colored_falling",
    "anvil",
    "dragon_egg",
    "concrete_powder",
    "brushable",
    // TODO(scaffolding-dripstone-falls): their pre-fall recompute logic:
    "scaffolding",
    "pointed_dripstone",
    // Fluid engine:
    "bubble_column",
    "liquid",
    // EnvironmentAttributes:
    "creaking_heart",
];

/// Java `BlockState.tick(level, pos, random)` for a scheduled tick. Returns
/// `None` for unported overrides.
#[allow(clippy::too_many_lines)] // one arm per Java class catalog
pub fn scheduled_tick(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl PlacementWorld,
) -> Option<BlockTickOutcome> {
    let entry = block_state_entry(&state.registry_id)?;
    let block_type = entry.block_type;
    if UNPORTED_SCHEDULED_TICKS.contains(&block_type) {
        return None;
    }

    let outcome = match block_type {
        // The destroy-if-unsupported family: Java `!canSurvive ->
        // destroyBlock(pos, true)`.
        "cactus"
        | "sugar_cane"
        | "chorus_flower"
        | "chorus_plant"
        | "bamboo_stalk"
        | "big_dripleaf_stem"
        | "kelp"
        | "kelp_plant"
        | "twisting_vines"
        | "twisting_vines_plant"
        | "weeping_vines"
        | "weeping_vines_plant"
        | "cave_vines"
        | "cave_vines_plant" => {
            if can_survive(state, pos, world) {
                BlockTickOutcome::None
            } else {
                BlockTickOutcome::Destroy { drop: true }
            }
        }

        // HangingMossBlock: canStayAtPosition (above attach or same block).
        "hanging_moss" => {
            let above = world.state_at(pos.relative(Direction::Up));
            let stays = above.registry_id == state.registry_id
                || multiface_can_attach_to(&above, Direction::Up);
            if stays {
                BlockTickOutcome::None
            } else {
                BlockTickOutcome::Destroy { drop: true }
            }
        }

        // LeavesBlock.tick: write the recomputed distance.
        "leaves" | "mangrove_leaves" | "tinted_particle_leaves" | "untinted_particle_leaves" => {
            let mut distance = 7;
            for direction in [
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ] {
                let neighbour = world.state_at(pos.relative(direction));
                distance = distance.min(connecting::leaves_distance_at(&neighbour) + 1);
                if distance == 1 {
                    break;
                }
            }
            let updated = set(state.clone(), "distance", distance.to_string());
            if updated == *state {
                BlockTickOutcome::None
            } else {
                BlockTickOutcome::SetState {
                    state: updated,
                    reschedule: None,
                }
            }
        }

        // Coral family death: without adjacent (or contained) water the live
        // block becomes its dead_ counterpart; fans keep their facing and
        // drop the waterlogged flag.
        "coral" | "coral_plant" | "coral_fan" | "coral_wall_fan" => {
            if coral_has_water(state, pos, world) {
                BlockTickOutcome::None
            } else {
                let path = state.registry_id.strip_prefix("minecraft:").unwrap_or("");
                let mut dead =
                    crate::block_placement::default_state(&format!("minecraft:dead_{path}"));
                if let Some(facing) = state.property("facing") {
                    dead = dead.try_set_property("facing", facing);
                }
                if dead.has_property("waterlogged") {
                    dead = dead.try_set_property("waterlogged", "false");
                }
                BlockTickOutcome::SetState {
                    state: dead,
                    reschedule: None,
                }
            }
        }

        // FarmlandBlock.turnToDirt / DirtPathBlock (unconditional in path's
        // tick, guarded in farmland's).
        "dirt_path" => BlockTickOutcome::SetState {
            state: crate::block_placement::default_state("minecraft:dirt"),
            reschedule: None,
        },
        "farmland" => {
            if can_survive(state, pos, world) {
                BlockTickOutcome::None
            } else {
                BlockTickOutcome::SetState {
                    state: crate::block_placement::default_state("minecraft:dirt"),
                    reschedule: None,
                }
            }
        }

        // ObserverBlock.tick: pulse POWERED (signal emission is TODO with the
        // redstone graph; the visible state flip is exact).
        "observer" => {
            if state.property("powered") == Some("true") {
                BlockTickOutcome::SetState {
                    state: set(state.clone(), "powered", "false"),
                    reschedule: None,
                }
            } else {
                BlockTickOutcome::SetState {
                    state: set(state.clone(), "powered", "true"),
                    reschedule: Some(2),
                }
            }
        }

        _ => BlockTickOutcome::None,
    };
    Some(outcome)
}

/// Coral `scanForWater`: contained (waterlogged) or any adjacent water fluid.
fn coral_has_water(state: &BlockStateModel, pos: BlockPos, world: &impl SurvivalWorld) -> bool {
    if state.property("waterlogged") == Some("true") {
        return true;
    }
    [
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestWorld {
        blocks: HashMap<(i32, i32, i32), BlockStateModel>,
    }

    impl TestWorld {
        fn with(mut self, pos: (i32, i32, i32), state: BlockStateModel) -> Self {
            self.blocks.insert(pos, state);
            self
        }
    }

    impl SurvivalWorld for TestWorld {
        fn state_at(&self, pos: BlockPos) -> BlockStateModel {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(BlockStateModel::air)
        }

        fn raw_brightness(&self, _pos: BlockPos) -> i32 {
            15
        }
    }

    impl PlacementWorld for TestWorld {
        fn has_neighbor_signal(&self, _pos: BlockPos) -> bool {
            false
        }
    }

    const POS: BlockPos = BlockPos { x: 0, y: 64, z: 0 };

    fn block(id: &str) -> BlockStateModel {
        BlockStateModel::default_for(id).unwrap_or_else(|| panic!("unknown block {id}"))
    }

    fn tick(state: &BlockStateModel, world: &TestWorld) -> BlockTickOutcome {
        scheduled_tick(state, POS, world)
            .unwrap_or_else(|| panic!("unported tick for {}", state.registry_id))
    }

    #[test]
    fn unsupported_blocks_break_with_drops_on_tick() {
        let empty = TestWorld::default();
        assert_eq!(
            tick(&block("minecraft:cactus"), &empty),
            BlockTickOutcome::Destroy { drop: true }
        );
        let sandy = TestWorld::default().with((0, 63, 0), block("minecraft:sand"));
        assert_eq!(
            tick(&block("minecraft:cactus"), &sandy),
            BlockTickOutcome::None
        );
        assert_eq!(
            tick(&block("minecraft:kelp"), &empty),
            BlockTickOutcome::Destroy { drop: true }
        );
    }

    #[test]
    fn leaves_rewrite_their_distance_on_tick() {
        let logged = TestWorld::default().with((0, 64, 1), block("minecraft:oak_log"));
        let outcome = tick(&block("minecraft:oak_leaves"), &logged);
        let BlockTickOutcome::SetState { state, reschedule } = outcome else {
            panic!("expected SetState, got {outcome:?}");
        };
        assert_eq!(state.property("distance"), Some("1"));
        assert_eq!(reschedule, None);
        // Already-correct distance is a no-op.
        let settled = block("minecraft:oak_leaves").try_set_property("distance", "1");
        assert_eq!(tick(&settled, &logged), BlockTickOutcome::None);
    }

    #[test]
    fn corals_die_without_water_and_observers_pulse() {
        let dry = TestWorld::default();
        let coral = block("minecraft:tube_coral").try_set_property("waterlogged", "false");
        let BlockTickOutcome::SetState { state, .. } = tick(&coral, &dry) else {
            panic!("expected coral death");
        };
        assert_eq!(state.registry_id, "minecraft:dead_tube_coral");
        // Wall fans keep their facing.
        let fan = block("minecraft:fire_coral_wall_fan")
            .try_set_property("waterlogged", "false")
            .try_set_property("facing", "east");
        let BlockTickOutcome::SetState { state, .. } = tick(&fan, &dry) else {
            panic!("expected fan death");
        };
        assert_eq!(state.registry_id, "minecraft:dead_fire_coral_wall_fan");
        assert_eq!(state.property("facing"), Some("east"));
        assert_eq!(state.property("waterlogged"), Some("false"));
        // Waterlogged corals stay alive.
        assert_eq!(
            tick(&block("minecraft:tube_coral"), &dry),
            BlockTickOutcome::None
        );

        // Observer pulse: on (reschedule 2) then off.
        let observer = block("minecraft:observer");
        let BlockTickOutcome::SetState { state, reschedule } = tick(&observer, &dry) else {
            panic!("expected observer flip");
        };
        assert_eq!(state.property("powered"), Some("true"));
        assert_eq!(reschedule, Some(2));
        let BlockTickOutcome::SetState { state, reschedule } = tick(&state, &dry) else {
            panic!("expected observer unflip");
        };
        assert_eq!(state.property("powered"), Some("false"));
        assert_eq!(reschedule, None);

        // Farmland reverts to dirt when capped.
        let capped = TestWorld::default().with((0, 65, 0), block("minecraft:stone"));
        let BlockTickOutcome::SetState { state, .. } = tick(&block("minecraft:farmland"), &capped)
        else {
            panic!("expected dirt");
        };
        assert_eq!(state.registry_id, "minecraft:dirt");

        // Falling blocks are explicitly unported until FallingBlockEntity.
        assert!(scheduled_tick(&block("minecraft:sand"), POS, &dry).is_none());
    }
}
