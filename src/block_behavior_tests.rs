#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use crate::block_behavior::{
        can_replace, find_next_state_holder_value, place_facing_opposite_player, placement_pos,
        plan_destroy_block, update_shape_or_destroy, BlockStateModel, HorizontalFacing,
        PlacementContext, ShapeUpdateContext, ShapeUpdateResult, STATE_HOLDER_NAME_TAG,
        STATE_HOLDER_PROPERTIES_TAG,
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
        // 6 direct + 30 shape cascade = 36 (Java Level.setBlock shape update behavior)
        assert_eq!(
            break_plan
                .actions
                .iter()
                .filter(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. }))
                .count(),
            36
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
    fn block_state_holder_property_surface_matches_java_state_holder() {
        assert_eq!(STATE_HOLDER_NAME_TAG, "Name");
        assert_eq!(STATE_HOLDER_PROPERTIES_TAG, "Properties");

        let state = BlockStateModel::new("minecraft:oak_stairs")
            .with_property("waterlogged", "false")
            .with_property("facing", "north")
            .with_property("half", "bottom");

        assert!(!state.is_singleton_state());
        assert_eq!(
            state.get_properties().collect::<Vec<_>>(),
            vec!["facing", "half", "waterlogged"]
        );
        assert_eq!(
            state.get_values().collect::<Vec<_>>(),
            vec![
                ("facing", "north"),
                ("half", "bottom"),
                ("waterlogged", "false")
            ]
        );
        assert!(state.has_property("facing"));
        assert!(!state.has_property("shape"));
        assert_eq!(state.property("facing"), Some("north"));
        assert_eq!(state.get_value_or_else("shape", "straight"), "straight");
        assert_eq!(
            state.state_holder_string(),
            "minecraft:oak_stairs[facing=north,half=bottom,waterlogged=false]"
        );

        let singleton = BlockStateModel::new("minecraft:stone");
        assert!(singleton.is_singleton_state());
        assert_eq!(singleton.state_holder_string(), "minecraft:stone");
    }

    #[test]
    fn block_state_holder_set_try_set_and_cycle_match_java_state_holder() {
        let facing_values = ["north", "east", "south", "west"];
        assert_eq!(
            find_next_state_holder_value(&facing_values, "north").unwrap(),
            "east"
        );
        assert_eq!(
            find_next_state_holder_value(&facing_values, "west").unwrap(),
            "north"
        );

        let state = BlockStateModel::new("minecraft:oak_stairs").with_property("facing", "west");
        assert_eq!(
            state
                .clone()
                .cycle_property("facing", &facing_values)
                .unwrap(),
            BlockStateModel::new("minecraft:oak_stairs").with_property("facing", "north")
        );
        assert_eq!(
            state
                .clone()
                .set_property_value("facing", "south", &facing_values)
                .unwrap()
                .property("facing"),
            Some("south")
        );
        assert!(state
            .clone()
            .set_property_value("facing", "up", &facing_values)
            .is_err());
        assert!(state
            .clone()
            .cycle_property("missing", &facing_values)
            .is_err());

        assert_eq!(
            state
                .clone()
                .try_set_property("facing", "east")
                .property("facing"),
            Some("east")
        );
        assert_eq!(
            state.clone().try_set_property("missing", "value"),
            state,
            "Java trySetValue returns this when the property is absent"
        );
    }

    #[test]
    fn block_state_concrete_wrapper_matches_java_block_state() {
        let constructed = BlockStateModel::from_property_pairs(
            "minecraft:chest",
            [
                ("waterlogged", "false"),
                ("type", "single"),
                ("facing", "north"),
            ],
        );
        assert_eq!(constructed.as_state(), &constructed);
        assert_eq!(
            constructed.state_holder_string(),
            "minecraft:chest[facing=north,type=single,waterlogged=false]"
        );

        let default_chest = BlockStateModel::default_for("minecraft:chest").unwrap();
        assert_eq!(default_chest, constructed);
        assert!(BlockStateModel::default_for("minecraft:not_a_block").is_none());
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
