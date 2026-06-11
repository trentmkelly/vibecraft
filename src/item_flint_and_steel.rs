//! Java `FlintAndSteelItem.useOn`: lighting campfires/candles/candle cakes in
//! place, or placing a `BaseFireBlock` state against the clicked face.
//!
//! This module is the pure decision half (what Java computes before touching
//! the `Level`); the live world mutation, packets, and durability glue live in
//! `network::status::item_use_live`.

use crate::block_behavior::BlockStateModel;
use crate::block_placement::connecting::base_fire_state;
use crate::block_placement::PlacementWorld;
use crate::block_tags::block_tag_contains;
use crate::block_update::{BlockPos, Direction};

/// The effect of one `FlintAndSteelItem.useOn` interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlintAndSteelUse {
    /// Campfire/candle/candle-cake branch: `LIT` flips to true at the clicked
    /// position (Java `level.setBlock(pos, state.setValue(LIT, true), 11)`).
    Light { pos: BlockPos, state: BlockStateModel },
    /// Fire branch: `BaseFireBlock.getState` written at the position adjacent
    /// to the clicked face (Java `level.setBlock(relativePos, fireState, 11)`).
    PlaceFire { pos: BlockPos, state: BlockStateModel },
    /// `BaseFireBlock.canBePlacedAt` failed: `InteractionResult.FAIL`, no
    /// world change.
    Fail,
}

/// Java `FlintAndSteelItem.useOn(context)` up to (but not including) the
/// `Level` writes: decides which branch fires and the exact state to write.
pub fn use_on(
    world: &impl PlacementWorld,
    clicked_pos: BlockPos,
    clicked_face: Direction,
) -> FlintAndSteelUse {
    let clicked_state = world.state_at(clicked_pos);
    if can_light(&clicked_state) {
        return FlintAndSteelUse::Light {
            pos: clicked_pos,
            state: clicked_state.try_set_property("lit", "true"),
        };
    }
    let fire_pos = clicked_pos.relative(clicked_face);
    if fire_can_be_placed_at(world, fire_pos) {
        FlintAndSteelUse::PlaceFire {
            pos: fire_pos,
            state: base_fire_state(world, fire_pos),
        }
    } else {
        FlintAndSteelUse::Fail
    }
}

/// The `CampfireBlock.canLight || CandleBlock.canLight ||
/// CandleCakeBlock.canLight` gate at the top of `FlintAndSteelItem.useOn`.
pub fn can_light(state: &BlockStateModel) -> bool {
    campfire_can_light(state) || candle_can_light(state) || candle_cake_can_light(state)
}

/// Java `CampfireBlock.canLight`: in `#minecraft:campfires` with the LIT and
/// WATERLOGGED properties, and currently neither lit nor waterlogged. The
/// property comparisons against `Some("false")` encode both Java checks: a
/// state without the property compares as `None` and fails the gate.
fn campfire_can_light(state: &BlockStateModel) -> bool {
    block_tag_contains("campfires", &state.registry_id)
        && state.property("waterlogged") == Some("false")
        && state.property("lit") == Some("false")
}

/// Java `CandleBlock.canLight`: in `#minecraft:candles`, not lit, not
/// waterlogged.
fn candle_can_light(state: &BlockStateModel) -> bool {
    block_tag_contains("candles", &state.registry_id)
        && state.property("lit") == Some("false")
        && state.property("waterlogged") == Some("false")
}

/// Java `CandleCakeBlock.canLight`: in `#minecraft:candle_cakes`, not lit.
fn candle_cake_can_light(state: &BlockStateModel) -> bool {
    block_tag_contains("candle_cakes", &state.registry_id)
        && state.property("lit") == Some("false")
}

/// Java `BaseFireBlock.canBePlacedAt`: the target must be air and the fire
/// state that would be placed there must survive.
///
/// TODO(live-nether-portal): Java also accepts a non-survivable position when
/// it completes an empty portal frame (`BaseFireBlock.isPortal`, keyed off the
/// player's horizontal direction); no portal system exists yet, so lighting a
/// portal frame currently fails like ordinary unsupported fire.
pub fn fire_can_be_placed_at(world: &impl PlacementWorld, pos: BlockPos) -> bool {
    let existing = world.state_at(pos);
    if !existing.is_air() {
        return false;
    }
    let fire = base_fire_state(world, pos);
    crate::block_survival::can_survive(&fire, pos, world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_survival::SurvivalWorld;
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

    const CLICKED: BlockPos = BlockPos { x: 0, y: 64, z: 0 };
    const ABOVE: BlockPos = BlockPos { x: 0, y: 65, z: 0 };

    fn unlit_campfire() -> BlockStateModel {
        BlockStateModel::new("minecraft:campfire")
            .with_property("lit", "false")
            .with_property("waterlogged", "false")
            .with_property("facing", "north")
            .with_property("signal_fire", "false")
    }

    #[test]
    fn lighting_an_unlit_campfire_flips_lit_in_place() {
        let world = TestWorld::default().with((0, 64, 0), unlit_campfire());
        let result = use_on(&world, CLICKED, Direction::Up);
        assert_eq!(
            result,
            FlintAndSteelUse::Light {
                pos: CLICKED,
                state: unlit_campfire().try_set_property("lit", "true"),
            }
        );
    }

    #[test]
    fn lit_or_waterlogged_campfires_and_candles_cannot_be_lit() {
        assert!(!can_light(
            &unlit_campfire().try_set_property("lit", "true")
        ));
        assert!(!can_light(
            &unlit_campfire().try_set_property("waterlogged", "true")
        ));
        assert!(!can_light(
            &BlockStateModel::new("minecraft:red_candle")
                .with_property("lit", "false")
                .with_property("waterlogged", "true")
                .with_property("candles", "1")
        ));
        // Blocks outside the three tags never light, even with a lit property.
        assert!(!can_light(
            &BlockStateModel::new("minecraft:furnace")
                .with_property("lit", "false")
                .with_property("facing", "north")
        ));
    }

    #[test]
    fn soul_campfires_candles_and_candle_cakes_light() {
        assert!(can_light(
            &BlockStateModel::new("minecraft:soul_campfire")
                .with_property("lit", "false")
                .with_property("waterlogged", "false")
        ));
        assert!(can_light(
            &BlockStateModel::new("minecraft:candle")
                .with_property("lit", "false")
                .with_property("waterlogged", "false")
                .with_property("candles", "2")
        ));
        assert!(can_light(
            &BlockStateModel::new("minecraft:cyan_candle_cake").with_property("lit", "false")
        ));
    }

    #[test]
    fn clicking_sturdy_ground_places_default_fire_on_the_face() {
        let world =
            TestWorld::default().with((0, 64, 0), BlockStateModel::new("minecraft:stone"));
        let result = use_on(&world, CLICKED, Direction::Up);
        assert_eq!(
            result,
            FlintAndSteelUse::PlaceFire {
                pos: ABOVE,
                state: crate::block_placement::default_state("minecraft:fire"),
            }
        );
    }

    #[test]
    fn clicking_a_soul_fire_base_places_soul_fire() {
        let world = TestWorld::default()
            .with((0, 64, 0), BlockStateModel::new("minecraft:soul_sand"));
        let result = use_on(&world, CLICKED, Direction::Up);
        assert_eq!(
            result,
            FlintAndSteelUse::PlaceFire {
                pos: ABOVE,
                state: crate::block_placement::default_state("minecraft:soul_fire"),
            }
        );
    }

    #[test]
    fn fire_against_a_flammable_side_carries_directional_faces() {
        // Clicking the side of an oak log: the fire cell floats (air below)
        // but survives via the flammable west neighbour, so FireBlock's
        // placement faces are computed per direction.
        let world =
            TestWorld::default().with((0, 64, 0), BlockStateModel::new("minecraft:oak_log"));
        let fire_pos = BlockPos { x: 1, y: 64, z: 0 };
        let result = use_on(&world, CLICKED, Direction::East);
        match result {
            FlintAndSteelUse::PlaceFire { pos, state } => {
                assert_eq!(pos, fire_pos);
                assert_eq!(state.registry_id, "minecraft:fire");
                assert_eq!(state.property("west"), Some("true"));
                assert_eq!(state.property("east"), Some("false"));
                assert_eq!(state.property("up"), Some("false"));
            }
            other => panic!("expected PlaceFire, got {other:?}"),
        }
    }

    #[test]
    fn fire_fails_on_occupied_or_unsupported_positions() {
        // Occupied: the face-adjacent cell already holds a block.
        let occupied = TestWorld::default()
            .with((0, 64, 0), BlockStateModel::new("minecraft:stone"))
            .with((0, 65, 0), BlockStateModel::new("minecraft:stone"));
        assert_eq!(use_on(&occupied, CLICKED, Direction::Up), FlintAndSteelUse::Fail);

        // Unsupported: clicking the side of stone leaves the fire cell
        // floating with no sturdy ground below and no flammable neighbour.
        let unsupported =
            TestWorld::default().with((0, 64, 0), BlockStateModel::new("minecraft:stone"));
        assert_eq!(
            use_on(&unsupported, CLICKED, Direction::East),
            FlintAndSteelUse::Fail
        );
    }
}
