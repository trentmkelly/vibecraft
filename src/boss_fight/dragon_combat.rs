//! Deterministic plans for Ender Dragon part damage and death ticks.
//!
//! These plans model the ordering in `EnderDragon.hurt` and `tickDeath` from
//! the decompiled server. `dragon_damage_plan_with_hurt` lets the caller model
//! the `LivingEntity.hurtServer` call that occurs after the dragon-specific
//! phase and part reductions.

/// Combat phase properties that affect the dragon's damage handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonCombatPhase {
    /// Any non-sitting phase other than the terminal dying phase.
    Flying,
    /// A sitting phase, including scanning, attacking, or flaming.
    Sitting,
    /// The dying phase rejects all damage.
    Dying,
    /// Takeoff after the sitting damage threshold has been exceeded.
    Takeoff,
}

impl DragonCombatPhase {
    fn is_sitting(self) -> bool {
        self == Self::Sitting
    }
}

/// Dragon state used as input to a single damage transition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonCombatState {
    pub health: f32,
    pub max_health: f32,
    pub phase: DragonCombatPhase,
    /// Damage actually removed from health while in the current sitting bout.
    pub sitting_damage_received: f32,
}

/// Which body part was struck. All parts route damage to the parent dragon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonPartHit {
    Head,
    Other,
}

/// Direct projectile types with special behavior during sitting phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonDirectProjectile {
    None,
    Arrow,
    WindCharge,
}

/// Raw dragon damage and the relevant source properties before base hurt processing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonDamageInput {
    pub part: DragonPartHit,
    pub damage: f32,
    /// `DamageSource.getEntity() instanceof Player`.
    pub attacker_is_player: bool,
    /// Damage source has `DamageTypeTags.ALWAYS_HURTS_ENDER_DRAGONS`.
    pub always_hurts_ender_dragons: bool,
    /// Direct entity type used by sitting-phase `onHurt` handling.
    pub direct_projectile: DragonDirectProjectile,
}

/// Result of applying the dragon's ordered part and phase damage rules.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonDamagePlan {
    /// The boolean returned by `EnderDragon.hurt` / `hurtServer`.
    pub accepted: bool,
    /// Damage after phase handling and non-head part reduction.
    pub damage_after_reductions: f32,
    /// Health damage actually applied after the source gate.
    pub health_damage: f32,
    pub health_after: f32,
    pub phase_after: DragonCombatPhase,
    pub sitting_damage_received_after: f32,
    /// Sitting arrow and wind-charge hits ignite their direct entity for 1.0 seconds before
    /// the zero-damage threshold rejects them.
    pub ignite_direct_projectile: bool,
}

/// Health and phase reported by the underlying living-entity damage step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonHurtResult {
    pub health_after: f32,
    pub phase_after: DragonCombatPhase,
}

/// Apply `EnderDragon.hurt` semantics in Java order, delegating the base
/// `LivingEntity.hurtServer` operation to `hurt`.
///
/// The sitting-phase arrow/wind-charge filter runs first, followed by the
/// quarter-damage-plus-one rule for non-head parts and the 0.01 damage floor.
/// Only then does the player-or-tag gate decide whether health changes. A
/// positive, sufficiently large hit rejected by that gate still returns true
/// in vanilla. The callback receives the reduced amount passed to
/// `reallyHurt`; its result then drives the lethal and sitting-damage checks,
/// matching Java's order even if the base hurt operation changes phase.
pub fn dragon_damage_plan_with_hurt(
    state: DragonCombatState,
    input: DragonDamageInput,
    hurt: impl FnOnce(f32) -> DragonHurtResult,
) -> DragonDamagePlan {
    if state.phase == DragonCombatPhase::Dying {
        return DragonDamagePlan {
            accepted: false,
            damage_after_reductions: input.damage,
            health_damage: 0.0,
            health_after: state.health,
            phase_after: state.phase,
            sitting_damage_received_after: state.sitting_damage_received,
            ignite_direct_projectile: false,
        };
    }

    let ignite_direct_projectile = state.phase.is_sitting()
        && matches!(
            input.direct_projectile,
            DragonDirectProjectile::Arrow | DragonDirectProjectile::WindCharge
        );
    let mut damage = if ignite_direct_projectile {
        0.0
    } else {
        input.damage
    };
    if input.part != DragonPartHit::Head {
        // Java Math.min propagates NaN; Rust's f32::min does not.
        let minimum = if damage.is_nan() {
            f32::NAN
        } else {
            damage.min(1.0)
        };
        damage = damage / 4.0 + minimum;
    }
    if damage < 0.01 {
        return DragonDamagePlan {
            accepted: false,
            damage_after_reductions: damage,
            health_damage: 0.0,
            health_after: state.health,
            phase_after: state.phase,
            sitting_damage_received_after: state.sitting_damage_received,
            ignite_direct_projectile,
        };
    }

    if !(input.attacker_is_player || input.always_hurts_ender_dragons) {
        return DragonDamagePlan {
            accepted: true,
            damage_after_reductions: damage,
            health_damage: 0.0,
            health_after: state.health,
            phase_after: state.phase,
            sitting_damage_received_after: state.sitting_damage_received,
            ignite_direct_projectile,
        };
    }

    let hurt_result = hurt(damage);
    let health_after = hurt_result.health_after;
    let health_damage = state.health - health_after;
    let mut phase_after = hurt_result.phase_after;
    let mut sitting_damage_received_after = state.sitting_damage_received;

    if health_after <= 0.0 && !phase_after.is_sitting() {
        // Vanilla resets health to one before entering DYING.
        phase_after = DragonCombatPhase::Dying;
        return DragonDamagePlan {
            accepted: true,
            damage_after_reductions: damage,
            health_damage,
            health_after: 1.0,
            phase_after,
            sitting_damage_received_after,
            ignite_direct_projectile,
        };
    }

    if phase_after.is_sitting() {
        sitting_damage_received_after = sitting_damage_received_after + state.health - health_after;
        if sitting_damage_received_after > 0.25 * state.max_health {
            sitting_damage_received_after = 0.0;
            phase_after = DragonCombatPhase::Takeoff;
        }
    }

    DragonDamagePlan {
        accepted: true,
        damage_after_reductions: damage,
        health_damage,
        health_after,
        phase_after,
        sitting_damage_received_after,
        ignite_direct_projectile,
    }
}

/// Convenience planner for callers whose base hurt step is direct health
/// subtraction with no mitigation, rejection, or phase transition.
pub fn dragon_damage_plan(state: DragonCombatState, input: DragonDamageInput) -> DragonDamagePlan {
    dragon_damage_plan_with_hurt(state, input, |damage| DragonHurtResult {
        health_after: if damage.is_nan() {
            f32::NAN
        } else {
            (state.health - damage).max(0.0)
        },
        phase_after: state.phase,
    })
}

/// Inputs needed to plan one `EnderDragon.tickDeath` invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonDeathTickInput {
    /// Counter value before this invocation; Java increments it first.
    pub dragon_death_time: i32,
    pub has_fight: bool,
    pub previously_killed: bool,
    pub mob_drops: bool,
    pub silent: bool,
    pub server_side: bool,
}

/// Ordered observable effects produced by one death tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonDeathTickPlan {
    /// Update the fight's dragon health before incrementing the death counter.
    pub update_dragon_fight: bool,
    pub dragon_death_time: i32,
    pub emit_explosion_emitter: bool,
    pub play_first_tick_sound: bool,
    /// Periodic award occurs before the upward death movement.
    pub periodic_experience: Option<u32>,
    /// Java constructs `Vec3(0.0, 0.1F, 0.0)`, so the f32 literal is promoted
    /// to f64 for movement.
    pub death_move_y: f64,
    /// Final award occurs after the upward death movement.
    pub final_experience: Option<u32>,
    pub set_dragon_killed: bool,
    pub remove_as_killed: bool,
    pub emit_entity_die_game_event: bool,
}

/// Plan one death tick, including Java's increment-before-comparison behavior.
///
/// Periodic experience uses `floor(xpCount * 0.08F)` and is awarded at times
/// greater than 150 divisible by five. At time 200 that award is followed by
/// the separate final `floor(xpCount * 0.2F)` award after movement.
pub fn dragon_death_tick_plan(input: DragonDeathTickInput) -> DragonDeathTickPlan {
    let dragon_death_time = input.dragon_death_time.wrapping_add(1);
    let xp_count = if input.has_fight && !input.previously_killed {
        12_000
    } else {
        500
    };
    let xp_as_f32 = xp_count as f32;
    let server_side = input.server_side;
    let final_tick = dragon_death_time >= 200;

    DragonDeathTickPlan {
        update_dragon_fight: input.has_fight,
        dragon_death_time,
        emit_explosion_emitter: (180..=200).contains(&dragon_death_time),
        play_first_tick_sound: server_side && dragon_death_time == 1 && !input.silent,
        periodic_experience: (server_side
            && input.mob_drops
            && dragon_death_time > 150
            && dragon_death_time % 5 == 0)
            .then(|| (xp_as_f32 * 0.08_f32).floor() as u32),
        death_move_y: 0.1_f32 as f64,
        final_experience: (server_side && input.mob_drops && final_tick)
            .then(|| (xp_as_f32 * 0.2_f32).floor() as u32),
        set_dragon_killed: server_side && final_tick && input.has_fight,
        remove_as_killed: server_side && final_tick,
        emit_entity_die_game_event: server_side && final_tick,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(
        phase: DragonCombatPhase,
        health: f32,
        sitting_damage_received: f32,
    ) -> DragonCombatState {
        DragonCombatState {
            health,
            max_health: 200.0,
            phase,
            sitting_damage_received,
        }
    }

    fn hit(part: DragonPartHit, damage: f32, player: bool) -> DragonDamageInput {
        DragonDamageInput {
            part,
            damage,
            attacker_is_player: player,
            always_hurts_ender_dragons: false,
            direct_projectile: DragonDirectProjectile::None,
        }
    }

    #[test]
    fn part_reduction_precedes_the_source_gate_and_only_players_or_tagged_damage_hurt() {
        let rejected_source = dragon_damage_plan(
            state(DragonCombatPhase::Flying, 200.0, 0.0),
            hit(DragonPartHit::Other, 12.0, false),
        );
        assert!(rejected_source.accepted);
        assert_eq!(rejected_source.damage_after_reductions, 4.0);
        assert_eq!(rejected_source.health_damage, 0.0);
        assert_eq!(rejected_source.health_after, 200.0);

        let tagged = dragon_damage_plan(
            state(DragonCombatPhase::Flying, 200.0, 0.0),
            DragonDamageInput {
                always_hurts_ender_dragons: true,
                ..hit(DragonPartHit::Other, 12.0, false)
            },
        );
        assert!(tagged.accepted);
        assert_eq!(tagged.health_after, 196.0);
    }

    #[test]
    fn lethal_flying_hit_sets_one_health_and_enters_dying() {
        let plan = dragon_damage_plan(
            state(DragonCombatPhase::Flying, 10.0, 0.0),
            hit(DragonPartHit::Head, 20.0, true),
        );
        assert_eq!(plan.health_damage, 10.0);
        assert_eq!(plan.health_after, 1.0);
        assert_eq!(plan.phase_after, DragonCombatPhase::Dying);
    }

    #[test]
    fn sitting_direct_arrows_are_ignited_then_rejected_and_sitting_damage_accumulates() {
        let arrow = dragon_damage_plan(
            state(DragonCombatPhase::Sitting, 200.0, 0.0),
            DragonDamageInput {
                direct_projectile: DragonDirectProjectile::Arrow,
                ..hit(DragonPartHit::Head, 10.0, true)
            },
        );
        assert!(!arrow.accepted);
        assert!(arrow.ignite_direct_projectile);
        assert_eq!(arrow.damage_after_reductions, 0.0);

        let below_threshold = dragon_damage_plan(
            state(DragonCombatPhase::Sitting, 200.0, 0.0),
            hit(DragonPartHit::Head, 50.0, true),
        );
        assert_eq!(below_threshold.health_after, 150.0);
        assert_eq!(below_threshold.phase_after, DragonCombatPhase::Sitting);
        assert_eq!(below_threshold.sitting_damage_received_after, 50.0);

        let over_threshold = dragon_damage_plan(
            state(DragonCombatPhase::Sitting, 150.0, 50.0),
            hit(DragonPartHit::Head, 1.0, true),
        );
        assert_eq!(over_threshold.phase_after, DragonCombatPhase::Takeoff);
        assert_eq!(over_threshold.sitting_damage_received_after, 0.0);

        let wind_charge = dragon_damage_plan(
            state(DragonCombatPhase::Sitting, 200.0, 0.0),
            DragonDamageInput {
                direct_projectile: DragonDirectProjectile::WindCharge,
                ..hit(DragonPartHit::Head, 10.0, true)
            },
        );
        assert!(!wind_charge.accepted);
        assert!(wind_charge.ignite_direct_projectile);
    }

    #[test]
    fn sitting_lethal_damage_takes_off_instead_of_entering_dying() {
        let plan = dragon_damage_plan(
            state(DragonCombatPhase::Sitting, 10.0, 49.0),
            hit(DragonPartHit::Head, 20.0, true),
        );
        assert_eq!(plan.health_after, 0.0);
        assert_eq!(plan.phase_after, DragonCombatPhase::Takeoff);
    }

    #[test]
    fn dying_phase_rejects_damage_and_exact_damage_floor_is_inclusive() {
        let dying = dragon_damage_plan(
            state(DragonCombatPhase::Dying, 1.0, 0.0),
            hit(DragonPartHit::Head, 100.0, true),
        );
        assert!(!dying.accepted);
        assert_eq!(dying.health_after, 1.0);

        let exact_floor = dragon_damage_plan(
            state(DragonCombatPhase::Flying, 10.0, 0.0),
            hit(DragonPartHit::Head, 0.01, true),
        );
        assert!(exact_floor.accepted);
        assert!((exact_floor.health_damage - 0.01).abs() < 0.000_01);
    }

    #[test]
    fn underlying_hurt_rejection_and_phase_change_feed_the_following_java_checks() {
        let base_rejected = dragon_damage_plan_with_hurt(
            state(DragonCombatPhase::Flying, 20.0, 0.0),
            hit(DragonPartHit::Head, 5.0, true),
            |_| DragonHurtResult {
                health_after: 20.0,
                phase_after: DragonCombatPhase::Flying,
            },
        );
        assert!(base_rejected.accepted);
        assert_eq!(base_rejected.health_damage, 0.0);
        assert_eq!(base_rejected.health_after, 20.0);

        let phase_changed_by_hurt = dragon_damage_plan_with_hurt(
            state(DragonCombatPhase::Sitting, 10.0, 0.0),
            hit(DragonPartHit::Head, 20.0, true),
            |_| DragonHurtResult {
                health_after: 0.0,
                phase_after: DragonCombatPhase::Flying,
            },
        );
        assert_eq!(phase_changed_by_hurt.health_after, 1.0);
        assert_eq!(phase_changed_by_hurt.phase_after, DragonCombatPhase::Dying);
        assert_eq!(phase_changed_by_hurt.sitting_damage_received_after, 0.0);
    }

    #[test]
    fn java_min_propagates_nan_in_the_non_head_damage_formula() {
        let plan = dragon_damage_plan(
            state(DragonCombatPhase::Flying, 20.0, 0.0),
            hit(DragonPartHit::Other, f32::NAN, false),
        );
        assert!(plan.damage_after_reductions.is_nan());
        // Java's `< 0.01` comparison is false for NaN, so the source gate
        // still runs and rejects this non-player, untagged source.
        assert!(plan.accepted);
        assert_eq!(plan.health_after, 20.0);
    }

    #[test]
    fn death_tick_increment_experience_boundaries_and_final_tick_match_java() {
        let periodic = dragon_death_tick_plan(DragonDeathTickInput {
            dragon_death_time: 154,
            has_fight: true,
            previously_killed: false,
            mob_drops: true,
            silent: false,
            server_side: true,
        });
        assert_eq!(periodic.dragon_death_time, 155);
        assert_eq!(periodic.periodic_experience, Some(960));
        assert!(!periodic.remove_as_killed);

        let before_emitter = dragon_death_tick_plan(DragonDeathTickInput {
            dragon_death_time: 178,
            has_fight: false,
            previously_killed: false,
            mob_drops: true,
            silent: false,
            server_side: true,
        });
        assert!(!before_emitter.emit_explosion_emitter);
        assert_eq!(before_emitter.periodic_experience, None);

        let final_tick = dragon_death_tick_plan(DragonDeathTickInput {
            dragon_death_time: 199,
            has_fight: true,
            previously_killed: false,
            mob_drops: true,
            silent: false,
            server_side: true,
        });
        assert!(final_tick.emit_explosion_emitter);
        assert_eq!(final_tick.periodic_experience, Some(960));
        assert_eq!(final_tick.final_experience, Some(2400));
        assert!(final_tick.set_dragon_killed);
        assert!(final_tick.remove_as_killed);
        assert!(final_tick.emit_entity_die_game_event);

        let no_drops_or_client = dragon_death_tick_plan(DragonDeathTickInput {
            dragon_death_time: 199,
            has_fight: true,
            previously_killed: false,
            mob_drops: false,
            silent: false,
            server_side: false,
        });
        assert_eq!(no_drops_or_client.final_experience, None);
        assert_eq!(no_drops_or_client.periodic_experience, None);
        assert!(!no_drops_or_client.set_dragon_killed);
        assert!(!no_drops_or_client.remove_as_killed);

        let after_terminal = dragon_death_tick_plan(DragonDeathTickInput {
            dragon_death_time: 200,
            has_fight: false,
            previously_killed: false,
            mob_drops: true,
            silent: false,
            server_side: true,
        });
        assert_eq!(after_terminal.dragon_death_time, 201);
        assert!(!after_terminal.emit_explosion_emitter);
        assert_eq!(after_terminal.final_experience, Some(100));
        assert!(after_terminal.remove_as_killed);

        let wrapped = dragon_death_tick_plan(DragonDeathTickInput {
            dragon_death_time: i32::MAX,
            has_fight: false,
            previously_killed: false,
            mob_drops: true,
            silent: false,
            server_side: true,
        });
        assert_eq!(wrapped.dragon_death_time, i32::MIN);
        assert!(!wrapped.remove_as_killed);
    }
}
