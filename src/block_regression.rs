#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use crate::block_behavior::{
        can_replace, place_facing_opposite_player, placement_pos, plan_destroy_block, rotate_state,
        update_shape_or_destroy, BlockStateModel, HorizontalFacing, PlacementContext, Rotation,
        ShapeUpdateContext, ShapeUpdateResult,
    };
    use crate::block_update::{BlockPos, BlockUpdateAction, Direction, UpdateFlags};
    use crate::dispenser_cauldron::{
        cauldron_interaction, dispenser_neighbor_change, execute_dispense, CauldronAction,
        CauldronContent, CauldronItem, DispenseAction, DispenseBehaviorKind, DispensedItemKind,
    };
    use crate::fluid::{place_liquid, FluidKind, LiquidPlaceResult};
    use crate::redstone::{
        comparator_output, dust_propagated_power, input_reset_delay, input_signal,
        observer_on_neighbor_changed, piston_decision, repeater_delay_ticks, repeater_output,
        ComparatorMode, InputKind, ObserverState, PistonDecision, PistonKind, PistonState,
        RepeaterState, MAX_SIGNAL, OBSERVER_PULSE_TICKS,
    };
    use crate::respawn::{use_bed, BedPart, BedState, RespawnAction};
    use crate::special_block::{campfire_use, chiseled_bookshelf_use, SpecialBlockAction};
    use crate::{block_behavior::Mirror, portal::Dimension};

    #[derive(Debug)]
    struct VanillaTrace<T> {
        name: &'static str,
        expected: T,
        observed: T,
    }

    fn assert_trace<T: PartialEq + std::fmt::Debug>(trace: VanillaTrace<T>) {
        assert_eq!(
            trace.observed, trace.expected,
            "vanilla block regression trace failed: {}",
            trace.name
        );
    }

    fn origin() -> BlockPos {
        BlockPos {
            x: 10,
            y: 64,
            z: -4,
        }
    }

    #[test]
    fn vanilla_trace_place_replace_rotate_and_waterlog() {
        let context = PlacementContext {
            clicked_pos: origin(),
            clicked_face: Direction::Up,
            player_horizontal_facing: HorizontalFacing::South,
            replacing_clicked_block: false,
        };
        assert_trace(VanillaTrace {
            name: "BlockItem#getPlacementState offsets to clicked face",
            expected: BlockPos {
                x: 10,
                y: 65,
                z: -4,
            },
            observed: placement_pos(context),
        });

        let tall_grass =
            BlockStateModel::new("minecraft:tall_grass").with_property("replaceable", "true");
        assert_trace(VanillaTrace {
            name: "replaceable block is consumed by placement",
            expected: true,
            observed: can_replace(&tall_grass, "minecraft:oak_planks"),
        });

        let stairs = BlockStateModel::default_for("minecraft:oak_stairs").unwrap();
        let placed = place_facing_opposite_player(&stairs, context);
        assert_trace(VanillaTrace {
            name: "horizontal block faces opposite player on placement",
            expected: Some("north"),
            observed: placed.property("facing"),
        });
        assert_trace(VanillaTrace {
            name: "clockwise rotation advances horizontal facing",
            expected: Some("east"),
            observed: rotate_state(&placed, Rotation::Clockwise90).property("facing"),
        });
        assert_trace(VanillaTrace {
            name: "front-back mirror flips east/west facing only",
            expected: Some("west"),
            observed: crate::block_behavior::mirror_state(
                &placed.with_property("facing", "east"),
                Mirror::FrontBack,
            )
            .property("facing"),
        });

        assert_trace(VanillaTrace {
            name: "water bucket waterlogs an unfilled waterloggable state",
            expected: LiquidPlaceResult::Waterlogged(
                BlockStateModel::default_for("minecraft:oak_stairs")
                    .unwrap()
                    .with_property("waterlogged", "true"),
            ),
            observed: place_liquid(&stairs, FluidKind::Water),
        });
    }

    #[test]
    fn vanilla_trace_break_and_shape_updates_remove_block_entities_and_notify_neighbors() {
        let chest = BlockStateModel::new("minecraft:chest");
        let plan = plan_destroy_block(origin(), chest, UpdateFlags::NOTIFY_NEIGHBORS, true);
        assert_trace(VanillaTrace {
            name: "destroy block replaces with air",
            expected: BlockStateModel::air(),
            observed: plan.replacement.clone(),
        });
        assert!(plan
            .actions
            .contains(&BlockUpdateAction::RemoveBlockEntity(origin())));
        assert!(plan
            .actions
            .iter()
            .any(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. })));

        let unsupported = update_shape_or_destroy(
            &BlockStateModel::new("minecraft:torch"),
            ShapeUpdateContext {
                pos: origin(),
                neighbor_pos: origin().relative(Direction::Down),
                direction_to_neighbor: Direction::Down,
                neighbor_signal: false,
                can_survive: false,
            },
        );
        assert_trace(VanillaTrace {
            name: "unsupported shape update destroys fragile block",
            expected: ShapeUpdateResult::Destroy(BlockStateModel::air()),
            observed: unsupported,
        });
    }

    #[test]
    fn vanilla_trace_use_interactions_cover_beds_campfires_bookshelves_and_cauldrons() {
        let bed = BedState {
            part: BedPart::Foot,
            facing: Direction::North,
            occupied: false,
        };
        assert_trace(VanillaTrace {
            name: "bed use at night sleeps and stores head position",
            expected: RespawnAction::Sleep {
                bed_head: origin().relative(Direction::North),
            },
            observed: use_bed(origin(), bed, Dimension::Overworld, true, true),
        });
        assert_trace(VanillaTrace {
            name: "campfire food use inserts into cooking slot instead of toggling",
            expected: SpecialBlockAction::InsertItem { slot: 0 },
            observed: campfire_use(true, true, false),
        });
        assert_trace(VanillaTrace {
            name: "chiseled bookshelf occupied slot removes item",
            expected: SpecialBlockAction::RemoveItem { slot: 4 },
            observed: chiseled_bookshelf_use(4, true),
        });
        assert_trace(VanillaTrace {
            name: "water cauldron glass bottle lowers level and returns water potion",
            expected: CauldronAction::Success {
                new_content: CauldronContent::Water { level: 1 },
                returned_item: "minecraft:potion{water}",
                stat: "use_cauldron",
                sound: "bottle_fill",
                game_event: "fluid_pickup",
            },
            observed: cauldron_interaction(
                CauldronContent::Water { level: 2 },
                CauldronItem::GlassBottle,
                false,
            ),
        });
    }

    #[test]
    fn vanilla_trace_dispenser_behaviors_cover_trigger_default_and_bucket_paths() {
        assert_trace(VanillaTrace {
            name: "untriggered powered dispenser schedules four tick dispense",
            expected: DispenseAction::ScheduleTick {
                delay: 4,
                triggered: true,
            },
            observed: dispenser_neighbor_change(true, false, false),
        });

        let dropped = execute_dispense(
            origin(),
            Direction::South,
            DispensedItemKind::DefaultItem,
            "minecraft:stone",
            true,
            true,
        );
        assert!(matches!(
            dropped.as_slice(),
            [
                DispenseAction::SpawnItem {
                    item: "minecraft:stone",
                    ..
                },
                DispenseAction::LevelEvent(1000),
                DispenseAction::LevelEvent(2000)
            ]
        ));

        assert_trace(VanillaTrace {
            name: "water bucket dispenser uses target block interaction",
            expected: vec![DispenseAction::UseOnBlock {
                target: origin().relative(Direction::South),
                behavior: DispenseBehaviorKind::EmptyContainer,
            }],
            observed: execute_dispense(
                origin(),
                Direction::South,
                DispensedItemKind::ContainerBucket,
                "minecraft:water_bucket",
                true,
                true,
            ),
        });
    }

    #[test]
    fn vanilla_trace_redstone_regressions_cover_dust_repeaters_comparators_and_observers() {
        assert_trace(VanillaTrace {
            name: "redstone dust propagation loses one signal strength",
            expected: 14,
            observed: dust_propagated_power(15),
        });
        assert_trace(VanillaTrace {
            name: "wooden button reset delay is 30 ticks",
            expected: Some(30),
            observed: input_reset_delay(InputKind::WoodenButton),
        });
        assert_trace(VanillaTrace {
            name: "target block clamps impact strength",
            expected: MAX_SIGNAL,
            observed: input_signal(InputKind::TargetBlock, true, 99),
        });
        assert_trace(VanillaTrace {
            name: "repeater delay index two is six game ticks",
            expected: 6,
            observed: repeater_delay_ticks(2),
        });
        assert_trace(VanillaTrace {
            name: "locked powered repeater keeps full output",
            expected: MAX_SIGNAL,
            observed: repeater_output(
                RepeaterState {
                    facing: Direction::North,
                    delay_index: 1,
                    powered: true,
                    locked: true,
                },
                0,
                [15, 0],
            ),
        });
        assert_trace(VanillaTrace {
            name: "subtract comparator subtracts strongest side input",
            expected: 5,
            observed: comparator_output(ComparatorMode::Subtract, 12, 7),
        });
        let observer_tick = observer_on_neighbor_changed(
            origin(),
            ObserverState {
                facing: Direction::East,
                powered: false,
            },
            origin().relative(Direction::East),
            100,
        )
        .unwrap();
        assert_trace(VanillaTrace {
            name: "observer schedules two tick pulse when watched block changes",
            expected: 100 + i64::from(OBSERVER_PULSE_TICKS),
            observed: observer_tick.trigger_tick,
        });
    }

    #[test]
    fn vanilla_trace_piston_regressions_cover_push_limit_sticky_pull_and_quasi_power() {
        let sticky = PistonState {
            kind: PistonKind::Sticky,
            facing: Direction::East,
            extended: false,
        };
        assert_trace(VanillaTrace {
            name: "sticky piston extends from quasi connectivity",
            expected: PistonDecision::Extend { sticky: true },
            observed: piston_decision(sticky, false, true, 12),
        });
        assert_trace(VanillaTrace {
            name: "piston refuses to push more than twelve blocks",
            expected: PistonDecision::Stay,
            observed: piston_decision(sticky, true, false, 13),
        });
        assert_trace(VanillaTrace {
            name: "extended sticky piston retracts and pulls head",
            expected: PistonDecision::Retract {
                sticky: true,
                pull_head: true,
            },
            observed: piston_decision(
                PistonState {
                    extended: true,
                    ..sticky
                },
                false,
                false,
                0,
            ),
        });
    }
}
