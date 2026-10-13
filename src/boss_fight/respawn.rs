//! Deterministic action planning for `DragonRespawnStage`.
//!
//! Minecraft's fight controller owns the world and entity mutations. This module keeps
//! the stage machine itself explicit: a caller supplies the ordered respawn crystals
//! and the End spikes, then applies the returned actions in order. The branch order,
//! tick boundaries, and transition timer resets follow `DragonRespawnStage.tick` and
//! `EnderDragonFight.setRespawnStage` in the decompiled server source.

use super::DragonRespawnStage;

/// Block coordinates used by the respawn action plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl RespawnBlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// The geometry of one `EndSpikeFeature.EndSpike`, in the order returned by Java.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnSpike {
    pub center_x: i32,
    pub center_z: i32,
    pub radius: i32,
    pub height: i32,
    pub guarded: bool,
}

/// Position of one crystal in `EnderDragonFight.respawnCrystals` iteration order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RespawnCrystal {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Ordered world or entity mutations emitted for one Java stage tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragonRespawnAction {
    /// Set or clear the beam on the indexed crystal in the supplied crystal slice.
    SetCrystalBeamTarget {
        crystal_index: usize,
        target: Option<RespawnBlockPos>,
    },
    /// Emit the vanilla level event (`3001`, data `0`).
    LevelEvent3001 {
        position: RespawnBlockPos,
    },
    /// Java removes this closed cube with `removeBlock(pos, false)` before rebuilding a spike.
    ClearSpikeVolume {
        min: RespawnBlockPos,
        max: RespawnBlockPos,
        /// Exact second argument to `Level.removeBlock`, which denotes piston movement.
        moved_by_piston: bool,
    },
    /// Java's spike explosion uses `ExplosionInteraction.BLOCK`.
    ExplodeSpike {
        x: f64,
        y: f64,
        z: f64,
        power: f32,
    },
    /// Place the one selected spike using the exact respawn feature configuration.
    /// The executor should pass a fresh Java-equivalent `RandomSource.create()` to feature
    /// placement; that random source controls generated crystal rotation, not this plan.
    PlaceSpike {
        spike: RespawnSpike,
        crystal_invulnerable: bool,
        crystal_beam_target: RespawnBlockPos,
        origin: RespawnBlockPos,
    },
    /// `setRespawnStage(END)` clears the active stage, marks the dragon alive, creates it,
    /// and triggers SUMMONED_ENTITY for tracking players if creation succeeds.
    CompleteFightAndCreateDragon,
    /// `abortRespawnSequence` clears the active stage and resets its timer.
    AbortRespawnSequence,
    /// Calls `EnderDragonFight.resetSpikeCrystals()` across generated End spikes,
    /// making each spike crystal vulnerable and clearing its beam target.
    ResetSpikeCrystals,
    /// Rebuild the exit podium in its activated state, as `spawnExitPortal(true)` does.
    SpawnExitPortal {
        activated: bool,
    },
    /// Persist the controller mutation via `EnderDragonFight.setDirty()`.
    MarkFightDirty,
    /// Use this crystal as the explosion source with `ExplosionInteraction.NONE`.
    /// Java clears its beam immediately before this action.
    ExplodeRespawnCrystal {
        crystal_index: usize,
        x: f64,
        y: f64,
        z: f64,
        power: f32,
    },
    DiscardRespawnCrystal {
        crystal_index: usize,
    },
}

/// Persistent fight data needed to resume the sequence exactly at its next tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DragonRespawnState {
    /// `None` means no respawn animation is active.
    pub stage: Option<DragonRespawnStage>,
    pub time: i32,
}

/// The deterministic result of one `DragonRespawnStage.tick` invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct DragonRespawnTickPlan {
    pub next_state: DragonRespawnState,
    /// Apply in order. Some Java branches create the dragon before clearing crystal beams.
    pub actions: Vec<DragonRespawnAction>,
}

/// Invalid spike indexing encountered while replaying a corrupt negative timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonRespawnPlanError {
    pub time: i32,
    pub spike_index: i32,
}

const CENTER: RespawnBlockPos = RespawnBlockPos::new(0, 128, 0);

/// Plan one stage tick using caller-supplied Java-order crystal and spike data.
///
/// `time` is the value passed to `DragonRespawnStage.tick`; Java increments the saved
/// `respawnTime` with a post-increment before dispatching the stage. Stage changes reset
/// that saved timer to zero, so the resulting `next_state` includes that reset.
/// `spikes` must preserve `EndSpikeFeature.getSpikesForLevel` order. This plans the raw
/// stage body after `EnderDragonFight.tick` has checked that respawn crystals remain. Use
/// [`plan_dragon_respawn_controller_tick`] to include the controller's empty-crystal abort.
pub fn plan_dragon_respawn_tick(
    state: DragonRespawnState,
    crystals: &[RespawnCrystal],
    spikes: &[RespawnSpike],
) -> Result<DragonRespawnTickPlan, DragonRespawnPlanError> {
    let Some(stage) = state.stage else {
        return Ok(DragonRespawnTickPlan {
            next_state: state,
            actions: Vec::new(),
        });
    };

    let mut next_state = DragonRespawnState {
        stage: Some(stage),
        time: state.time.wrapping_add(1),
    };
    let mut actions = Vec::new();
    let mut transition = None;

    match stage {
        DragonRespawnStage::Start => {
            set_all_beams(&mut actions, crystals, Some(CENTER));
            transition = Some(DragonRespawnStage::PreparingToSummonPillars);
        }
        DragonRespawnStage::PreparingToSummonPillars => {
            if state.time < 100 {
                if matches!(state.time, 0 | 50 | 51 | 52) || state.time >= 95 {
                    actions.push(DragonRespawnAction::LevelEvent3001 { position: CENTER });
                }
            } else {
                transition = Some(DragonRespawnStage::SummoningPillars);
            }
        }
        DragonRespawnStage::SummoningPillars => {
            let starts_beam = state.time % 40 == 0;
            let ends_beam = state.time % 40 == 39;
            if starts_beam || ends_beam {
                // Division truncates toward zero like Java. A negative index passes Java's
                // `index < size` check and then fails at `spikes.get(index)`.
                let spike_index = state.time / 40;
                if spike_index < spikes.len() as i32 {
                    if spike_index < 0 {
                        return Err(DragonRespawnPlanError {
                            time: state.time,
                            spike_index,
                        });
                    }
                    let spike = spikes[spike_index as usize];
                    if starts_beam {
                        let target = RespawnBlockPos::new(
                            spike.center_x,
                            spike.height.wrapping_add(1),
                            spike.center_z,
                        );
                        set_all_beams(&mut actions, crystals, Some(target));
                    } else {
                        rebuild_spike(&mut actions, spike);
                    }
                } else if starts_beam {
                    transition = Some(DragonRespawnStage::SummoningDragon);
                }
            }
        }
        DragonRespawnStage::SummoningDragon => {
            if state.time >= 100 {
                complete_respawn(&mut actions, crystals);
                next_state.stage = None;
                next_state.time = 0;
            } else if state.time >= 80 {
                actions.push(DragonRespawnAction::LevelEvent3001 { position: CENTER });
            } else if state.time == 0 {
                set_all_beams(&mut actions, crystals, Some(CENTER));
            } else if state.time < 5 {
                actions.push(DragonRespawnAction::LevelEvent3001 { position: CENTER });
            }
        }
        DragonRespawnStage::End => {}
    }

    if let Some(stage) = transition {
        next_state.stage = Some(stage);
        next_state.time = 0;
        actions.push(DragonRespawnAction::MarkFightDirty);
    }

    Ok(DragonRespawnTickPlan {
        next_state,
        actions,
    })
}

/// Rebuild one pillar in Java order: clear its volume, explode, then place the feature.
fn rebuild_spike(actions: &mut Vec<DragonRespawnAction>, spike: RespawnSpike) {
    actions.push(DragonRespawnAction::ClearSpikeVolume {
        min: RespawnBlockPos::new(
            spike.center_x.wrapping_sub(10),
            spike.height.wrapping_sub(10),
            spike.center_z.wrapping_sub(10),
        ),
        max: RespawnBlockPos::new(
            spike.center_x.wrapping_add(10),
            spike.height.wrapping_add(10),
            spike.center_z.wrapping_add(10),
        ),
        moved_by_piston: false,
    });
    actions.push(DragonRespawnAction::ExplodeSpike {
        x: (spike.center_x as f32 + 0.5) as f64,
        y: spike.height as f64,
        z: (spike.center_z as f32 + 0.5) as f64,
        power: 5.0,
    });
    actions.push(DragonRespawnAction::PlaceSpike {
        spike,
        crystal_invulnerable: true,
        crystal_beam_target: CENTER,
        origin: RespawnBlockPos::new(spike.center_x, 45, spike.center_z),
    });
}

/// Preserve the completion ordering in `DragonRespawnStage.SUMMONING_DRAGON`:
/// create the dragon and persist its fight state before resetting or discarding crystals.
fn complete_respawn(actions: &mut Vec<DragonRespawnAction>, crystals: &[RespawnCrystal]) {
    // setRespawnStage(END) creates the dragon synchronously before these actions.
    actions.push(DragonRespawnAction::CompleteFightAndCreateDragon);
    actions.push(DragonRespawnAction::MarkFightDirty);
    actions.push(DragonRespawnAction::ResetSpikeCrystals);
    for (crystal_index, crystal) in crystals.iter().enumerate() {
        actions.push(DragonRespawnAction::SetCrystalBeamTarget {
            crystal_index,
            target: None,
        });
        actions.push(DragonRespawnAction::ExplodeRespawnCrystal {
            crystal_index,
            x: crystal.x,
            y: crystal.y,
            z: crystal.z,
            power: 6.0,
        });
        actions.push(DragonRespawnAction::DiscardRespawnCrystal { crystal_index });
    }
}

/// Plan the respawn portion of `EnderDragonFight.tick`, including its empty-crystal abort.
///
/// Java aborts before incrementing the stage timer when the saved respawn crystal
/// references resolve to an empty list. The stage-only planner above represents the body
/// of `DragonRespawnStage.tick`; this controller-level entry point preserves that guard.
pub fn plan_dragon_respawn_controller_tick(
    state: DragonRespawnState,
    crystals: &[RespawnCrystal],
    spikes: &[RespawnSpike],
) -> Result<DragonRespawnTickPlan, DragonRespawnPlanError> {
    if state.stage.is_some() && crystals.is_empty() {
        return Ok(DragonRespawnTickPlan {
            next_state: DragonRespawnState::default(),
            actions: vec![
                DragonRespawnAction::AbortRespawnSequence,
                DragonRespawnAction::ResetSpikeCrystals,
                DragonRespawnAction::SpawnExitPortal { activated: true },
                DragonRespawnAction::MarkFightDirty,
            ],
        });
    }

    let mut plan = plan_dragon_respawn_tick(state, crystals, spikes)?;
    if state.stage.is_some() {
        // EnderDragonFight.tick persists the counter after every successful stage tick.
        plan.actions.push(DragonRespawnAction::MarkFightDirty);
    }
    Ok(plan)
}

fn set_all_beams(
    actions: &mut Vec<DragonRespawnAction>,
    crystals: &[RespawnCrystal],
    target: Option<RespawnBlockPos>,
) {
    actions.extend(crystals.iter().enumerate().map(|(crystal_index, _)| {
        DragonRespawnAction::SetCrystalBeamTarget {
            crystal_index,
            target,
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crystal() -> RespawnCrystal {
        RespawnCrystal {
            x: 1.5,
            y: 65.0,
            z: -2.5,
        }
    }

    fn spike() -> RespawnSpike {
        RespawnSpike {
            center_x: 42,
            center_z: 0,
            radius: 3,
            height: 82,
            guarded: false,
        }
    }

    fn plan(stage: DragonRespawnStage, time: i32) -> DragonRespawnTickPlan {
        plan_dragon_respawn_tick(
            DragonRespawnState {
                stage: Some(stage),
                time,
            },
            &[crystal()],
            &[spike()],
        )
        .unwrap()
    }

    #[test]
    fn start_and_preparation_ticks_match_java_boundaries() {
        let start = plan(DragonRespawnStage::Start, 23);
        assert_eq!(
            start.next_state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::PreparingToSummonPillars),
                time: 0,
            }
        );
        assert_eq!(
            start.actions,
            vec![
                DragonRespawnAction::SetCrystalBeamTarget {
                    crystal_index: 0,
                    target: Some(CENTER),
                },
                DragonRespawnAction::MarkFightDirty,
            ]
        );

        for time in [0, 50, 51, 52, 95, 99] {
            assert_eq!(
                plan(DragonRespawnStage::PreparingToSummonPillars, time).actions,
                vec![DragonRespawnAction::LevelEvent3001 { position: CENTER }],
                "preparation event at time {time}"
            );
        }
        for time in [1, 49, 53, 94] {
            assert!(
                plan(DragonRespawnStage::PreparingToSummonPillars, time)
                    .actions
                    .is_empty(),
                "no preparation event at time {time}"
            );
        }
        let boundary = plan(DragonRespawnStage::PreparingToSummonPillars, 100);
        assert_eq!(boundary.actions, vec![DragonRespawnAction::MarkFightDirty]);
        assert_eq!(
            boundary.next_state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningPillars),
                time: 0,
            }
        );
    }

    #[test]
    fn pillar_beams_rebuild_order_and_completion_boundary_match_java() {
        assert_eq!(
            plan(DragonRespawnStage::SummoningPillars, 0).actions,
            vec![DragonRespawnAction::SetCrystalBeamTarget {
                crystal_index: 0,
                target: Some(RespawnBlockPos::new(42, 83, 0)),
            }]
        );
        assert_eq!(
            plan(DragonRespawnStage::SummoningPillars, 39).actions,
            vec![
                DragonRespawnAction::ClearSpikeVolume {
                    min: RespawnBlockPos::new(32, 72, -10),
                    max: RespawnBlockPos::new(52, 92, 10),
                    moved_by_piston: false,
                },
                DragonRespawnAction::ExplodeSpike {
                    x: 42.5,
                    y: 82.0,
                    z: 0.5,
                    power: 5.0,
                },
                DragonRespawnAction::PlaceSpike {
                    spike: spike(),
                    crystal_invulnerable: true,
                    crystal_beam_target: CENTER,
                    origin: RespawnBlockPos::new(42, 45, 0),
                },
            ]
        );
        let finished = plan_dragon_respawn_tick(
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningPillars),
                time: 40,
            },
            &[crystal()],
            &[spike()],
        )
        .unwrap();
        assert_eq!(
            finished.next_state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningDragon),
                time: 0,
            }
        );
        assert_eq!(finished.actions, vec![DragonRespawnAction::MarkFightDirty]);

        let ten_spikes = [spike(); 10];
        let final_beam = plan_dragon_respawn_tick(
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningPillars),
                time: 399,
            },
            &[crystal()],
            &ten_spikes,
        )
        .unwrap();
        assert!(matches!(
            final_beam.actions.as_slice(),
            [DragonRespawnAction::ClearSpikeVolume { .. }, ..]
        ));
        assert_eq!(
            final_beam.next_state.stage,
            Some(DragonRespawnStage::SummoningPillars)
        );
        let transition = plan_dragon_respawn_tick(
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningPillars),
                time: 400,
            },
            &[crystal()],
            &ten_spikes,
        )
        .unwrap();
        assert_eq!(
            transition.actions,
            vec![DragonRespawnAction::MarkFightDirty]
        );
        assert_eq!(
            transition.next_state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningDragon),
                time: 0,
            }
        );
    }

    #[test]
    fn dragon_summoning_order_and_timer_edges_match_java() {
        assert_eq!(
            plan(DragonRespawnStage::SummoningDragon, 0).actions,
            vec![DragonRespawnAction::SetCrystalBeamTarget {
                crystal_index: 0,
                target: Some(CENTER),
            }]
        );
        for time in [1, 4, 80, 99] {
            assert_eq!(
                plan(DragonRespawnStage::SummoningDragon, time).actions,
                vec![DragonRespawnAction::LevelEvent3001 { position: CENTER }],
                "summoning event at time {time}"
            );
        }
        for time in [5, 79] {
            assert!(
                plan(DragonRespawnStage::SummoningDragon, time)
                    .actions
                    .is_empty(),
                "no summoning event at time {time}"
            );
        }
        let complete = plan(DragonRespawnStage::SummoningDragon, 100);
        assert_eq!(complete.next_state, DragonRespawnState::default());
        assert_eq!(
            complete.actions,
            vec![
                DragonRespawnAction::CompleteFightAndCreateDragon,
                DragonRespawnAction::MarkFightDirty,
                DragonRespawnAction::ResetSpikeCrystals,
                DragonRespawnAction::SetCrystalBeamTarget {
                    crystal_index: 0,
                    target: None,
                },
                DragonRespawnAction::ExplodeRespawnCrystal {
                    crystal_index: 0,
                    x: 1.5,
                    y: 65.0,
                    z: -2.5,
                    power: 6.0,
                },
                DragonRespawnAction::DiscardRespawnCrystal { crystal_index: 0 },
            ]
        );
    }

    #[test]
    fn controller_aborts_empty_crystals_before_ticking_and_restores_the_podium() {
        let active = DragonRespawnState {
            stage: Some(DragonRespawnStage::SummoningPillars),
            time: 39,
        };
        let aborted = plan_dragon_respawn_controller_tick(active, &[], &[spike()]).unwrap();
        assert_eq!(aborted.next_state, DragonRespawnState::default());
        assert_eq!(
            aborted.actions,
            vec![
                DragonRespawnAction::AbortRespawnSequence,
                DragonRespawnAction::ResetSpikeCrystals,
                DragonRespawnAction::SpawnExitPortal { activated: true },
                DragonRespawnAction::MarkFightDirty,
            ]
        );

        let ticking = plan_dragon_respawn_controller_tick(
            DragonRespawnState {
                stage: Some(DragonRespawnStage::PreparingToSummonPillars),
                time: 1,
            },
            &[crystal()],
            &[spike()],
        )
        .unwrap();
        assert_eq!(ticking.actions, vec![DragonRespawnAction::MarkFightDirty]);
        let transitioning = plan_dragon_respawn_controller_tick(
            DragonRespawnState {
                stage: Some(DragonRespawnStage::PreparingToSummonPillars),
                time: 100,
            },
            &[crystal()],
            &[spike()],
        )
        .unwrap();
        assert_eq!(
            transitioning.actions,
            vec![
                DragonRespawnAction::MarkFightDirty,
                DragonRespawnAction::MarkFightDirty
            ]
        );
    }

    #[test]
    fn complete_sequence_advances_through_every_java_stage_boundary() {
        let crystals = [crystal()];
        let spikes = [spike(); 10];
        let mut state = DragonRespawnState {
            stage: Some(DragonRespawnStage::Start),
            time: 0,
        };

        state = plan_dragon_respawn_controller_tick(state, &crystals, &spikes)
            .unwrap()
            .next_state;
        assert_eq!(
            state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::PreparingToSummonPillars),
                time: 0,
            }
        );
        for _ in 0..=100 {
            state = plan_dragon_respawn_controller_tick(state, &crystals, &spikes)
                .unwrap()
                .next_state;
        }
        assert_eq!(
            state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningPillars),
                time: 0,
            }
        );
        for _ in 0..=400 {
            state = plan_dragon_respawn_controller_tick(state, &crystals, &spikes)
                .unwrap()
                .next_state;
        }
        assert_eq!(
            state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningDragon),
                time: 0,
            }
        );
        for _ in 0..=100 {
            state = plan_dragon_respawn_controller_tick(state, &crystals, &spikes)
                .unwrap()
                .next_state;
        }
        assert_eq!(state, DragonRespawnState::default());
    }

    #[test]
    fn java_overflow_and_negative_spike_index_behavior_are_explicit() {
        let extreme_spike = RespawnSpike {
            center_x: i32::MAX,
            center_z: i32::MIN,
            radius: 2,
            height: i32::MAX,
            guarded: false,
        };
        let plan = plan_dragon_respawn_tick(
            DragonRespawnState {
                stage: Some(DragonRespawnStage::SummoningPillars),
                time: 39,
            },
            &[crystal()],
            &[extreme_spike],
        )
        .unwrap();
        assert!(matches!(
            plan.actions.as_slice(),
            [
                DragonRespawnAction::ClearSpikeVolume {
                    min: RespawnBlockPos {
                        x: 2_147_483_637,
                        y: 2_147_483_637,
                        z: 2_147_483_638,
                    },
                    max: RespawnBlockPos {
                        x: -2_147_483_639,
                        y: -2_147_483_639,
                        z: -2_147_483_638,
                    },
                    ..
                },
                DragonRespawnAction::ExplodeSpike {
                    y: 2_147_483_647.0,
                    ..
                },
                ..
            ]
        ));

        assert_eq!(
            plan_dragon_respawn_tick(
                DragonRespawnState {
                    stage: Some(DragonRespawnStage::SummoningPillars),
                    time: -40,
                },
                &[crystal()],
                &[spike()],
            ),
            Err(DragonRespawnPlanError {
                time: -40,
                spike_index: -1,
            })
        );
    }

    #[test]
    fn inactive_or_end_stage_is_a_timer_only_noop() {
        let idle = DragonRespawnState {
            stage: None,
            time: 17,
        };
        assert_eq!(
            plan_dragon_respawn_tick(idle, &[], &[]).unwrap().next_state,
            idle
        );

        let end = plan(DragonRespawnStage::End, 17);
        assert_eq!(
            end.next_state,
            DragonRespawnState {
                stage: Some(DragonRespawnStage::End),
                time: 18,
            }
        );
        assert!(end.actions.is_empty());
    }
}
