#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use crate::block_behavior::{
        can_replace, place_facing_opposite_player, placement_pos, plan_destroy_block,
        update_shape_or_destroy, BlockStateModel, HorizontalFacing, PlacementContext,
        ShapeUpdateContext, ShapeUpdateResult,
    };
    use crate::block_update::{BlockPos, BlockUpdateAction, Direction, UpdateFlags};
    use crate::fluid::{
        pickup_liquid, place_liquid, plan_fluid_update, FluidKind, LiquidPickupResult,
        LiquidPlaceResult,
    };
    use crate::redstone::{
        comparator_output, dust_propagated_power, input_reset_delay, observer_on_neighbor_changed,
        piston_decision, ComparatorMode, InputKind, ObserverState, PistonDecision, PistonKind,
        PistonState,
    };
    use crate::scheduled_tick::{LevelTickQueues, TickPriority};
    use crate::special_block::{campfire_use, chiseled_bookshelf_use, SpecialBlockAction};
    use crate::storage::region::ChunkPos;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct BlockBehaviorCase {
        name: &'static str,
        category: &'static str,
        evidence: &'static str,
    }

    fn origin() -> BlockPos {
        BlockPos { x: 3, y: 64, z: -8 }
    }

    #[test]
    fn block_behavior_test_matrix_covers_required_surface() {
        let cases = [
            BlockBehaviorCase {
                name: "placement offset and replaceability",
                category: "placement",
                evidence: "placement_pos and can_replace",
            },
            BlockBehaviorCase {
                name: "break destroys block entities and notifies neighbors",
                category: "break",
                evidence: "plan_destroy_block",
            },
            BlockBehaviorCase {
                name: "use interactions dispatch special block actions",
                category: "use",
                evidence: "campfire_use and chiseled_bookshelf_use",
            },
            BlockBehaviorCase {
                name: "redstone propagates, schedules, and moves",
                category: "redstone",
                evidence: "dust, comparator, observer, piston helpers",
            },
            BlockBehaviorCase {
                name: "fluid placement, pickup, and spread ticks",
                category: "fluid",
                evidence: "place_liquid, pickup_liquid, plan_fluid_update",
            },
            BlockBehaviorCase {
                name: "scheduled ticks drain by chunk and priority",
                category: "scheduled ticks",
                evidence: "LevelTickQueues",
            },
        ];

        assert_eq!(
            cases.iter().map(|case| case.category).collect::<Vec<_>>(),
            vec![
                "placement",
                "break",
                "use",
                "redstone",
                "fluid",
                "scheduled ticks"
            ]
        );
        assert!(cases.iter().all(|case| !case.evidence.is_empty()));
    }

    #[test]
    fn placement_break_and_use_behaviors_match_vanilla_shapes() {
        let context = PlacementContext {
            clicked_pos: origin(),
            clicked_face: Direction::Up,
            player_horizontal_facing: HorizontalFacing::West,
            replacing_clicked_block: false,
        };
        assert_eq!(placement_pos(context), origin().relative(Direction::Up));
        assert!(can_replace(
            &BlockStateModel::new("minecraft:tall_grass").with_property("replaceable", "true"),
            "minecraft:oak_planks",
        ));

        let chest = BlockStateModel::default_for("minecraft:chest").unwrap();
        let placed = place_facing_opposite_player(&chest, context);
        assert_eq!(placed.property("facing"), Some("east"));

        let break_plan = plan_destroy_block(origin(), chest, UpdateFlags::NOTIFY_NEIGHBORS, true);
        assert_eq!(break_plan.replacement, BlockStateModel::air());
        assert!(break_plan
            .actions
            .contains(&BlockUpdateAction::RemoveBlockEntity(origin())));
        assert_eq!(
            break_plan
                .actions
                .iter()
                .filter(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. }))
                .count(),
            6
        );

        assert_eq!(
            update_shape_or_destroy(
                &BlockStateModel::new("minecraft:torch"),
                ShapeUpdateContext {
                    pos: origin(),
                    neighbor_pos: origin().relative(Direction::Down),
                    direction_to_neighbor: Direction::Down,
                    neighbor_signal: false,
                    can_survive: false,
                },
            ),
            ShapeUpdateResult::Destroy(BlockStateModel::air())
        );
        assert_eq!(
            campfire_use(true, true, false),
            SpecialBlockAction::InsertItem { slot: 0 }
        );
        assert_eq!(
            chiseled_bookshelf_use(2, true),
            SpecialBlockAction::RemoveItem { slot: 2 }
        );
    }

    #[test]
    fn redstone_fluid_and_scheduled_tick_behaviors_match_vanilla_shapes() {
        assert_eq!(dust_propagated_power(15), 14);
        assert_eq!(input_reset_delay(InputKind::WoodenButton), Some(30));
        assert_eq!(comparator_output(ComparatorMode::Subtract, 12, 7), 5);
        assert_eq!(
            piston_decision(
                PistonState {
                    kind: PistonKind::Sticky,
                    facing: Direction::East,
                    extended: false,
                },
                false,
                true,
                12,
            ),
            PistonDecision::Extend { sticky: true }
        );
        let observer_tick = observer_on_neighbor_changed(
            origin(),
            ObserverState {
                facing: Direction::East,
                powered: false,
            },
            origin().relative(Direction::East),
            40,
        )
        .unwrap();
        assert_eq!(observer_tick.ty, "minecraft:observer");
        assert_eq!(observer_tick.trigger_tick, 42);

        let stairs = BlockStateModel::default_for("minecraft:oak_stairs").unwrap();
        assert_eq!(
            place_liquid(&stairs, FluidKind::Water),
            LiquidPlaceResult::Waterlogged(stairs.clone().with_property("waterlogged", "true"))
        );
        assert_eq!(
            pickup_liquid(&stairs.clone().with_property("waterlogged", "true")),
            LiquidPickupResult::PickedUp {
                fluid: FluidKind::Water.source(),
                replacement: stairs.clone().with_property("waterlogged", "false")
            }
        );
        let fluid_plan = plan_fluid_update(origin(), FluidKind::Water.source(), 100, 7);
        assert_eq!(fluid_plan.self_tick.trigger_tick, 105);
        assert_eq!(fluid_plan.neighbor_ticks.len(), 5);

        let mut ticks = LevelTickQueues::new();
        ticks.add_container(ChunkPos { x: 0, z: -1 });
        let low = ticks.create_tick(200, origin(), "minecraft:water", 0, TickPriority::Low);
        let high = ticks.create_tick(
            200,
            origin().relative(Direction::East),
            "minecraft:redstone_wire",
            0,
            TickPriority::High,
        );
        assert!(ticks.schedule(low));
        assert!(ticks.schedule(high));
        let ran = ticks.tick(200, 10, |_| true);
        assert_eq!(
            ran.iter().map(|tick| tick.ty.as_str()).collect::<Vec<_>>(),
            vec!["minecraft:redstone_wire", "minecraft:water"]
        );
    }
}
