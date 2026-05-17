#![allow(dead_code)]

use crate::entity_physics::Vec3;
use crate::game_event::{game_event_by_id, GameEventContext, GameEventDefinition};

pub const NO_VIBRATION_FREQUENCY: u8 = 0;
pub const WARDEN_VIBRATION_COOLDOWN_TICKS: i32 = 40;
pub const WARDEN_RECENT_PROJECTILE_TICKS: i32 = 100;
pub const WARDEN_PROJECTILE_ANGER_BONUS: i32 = 10;
pub const WARDEN_PROJECTILE_OWNER_RANGE: f32 = 30.0;

#[derive(Debug, Clone, PartialEq)]
pub struct VibrationInfo {
    pub event: GameEventDefinition,
    pub distance: f32,
    pub pos: Vec3,
    pub source_entity: Option<String>,
    pub projectile_owner: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VibrationSelector {
    current: Option<(VibrationInfo, i64)>,
}

impl VibrationSelector {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn add_candidate(&mut self, vibration: VibrationInfo, tick_time: i64) {
        if self.should_replace(&vibration, tick_time) {
            self.current = Some((vibration, tick_time));
        }
    }

    pub fn chosen_candidate(&self, time: i64) -> Option<&VibrationInfo> {
        self.current
            .as_ref()
            .and_then(|(vibration, tick)| (*tick < time).then_some(vibration))
    }

    pub fn start_over(&mut self) {
        self.current = None;
    }

    fn should_replace(&self, new_vibration: &VibrationInfo, tick_time: i64) -> bool {
        let Some((previous, previous_tick)) = &self.current else {
            return true;
        };
        if tick_time != *previous_tick {
            return false;
        }
        if new_vibration.distance < previous.distance {
            true
        } else if new_vibration.distance > previous.distance {
            false
        } else {
            vibration_frequency(new_vibration.event.id) > vibration_frequency(previous.event.id)
        }
    }
}

impl Default for VibrationSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VibrationData {
    pub current_vibration: Option<VibrationInfo>,
    pub travel_time_in_ticks: i32,
    pub selector: VibrationSelector,
    pub reload_vibration_particle: bool,
}

impl VibrationData {
    pub fn new() -> Self {
        Self {
            current_vibration: None,
            travel_time_in_ticks: 0,
            selector: VibrationSelector::new(),
            reload_vibration_particle: false,
        }
    }

    pub fn decrement_travel_time(&mut self) {
        self.travel_time_in_ticks = self.travel_time_in_ticks.saturating_sub(1);
    }
}

impl Default for VibrationData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VibrationSourceState {
    pub spectator: bool,
    pub stepping_carefully: bool,
    pub dampens_vibrations: bool,
    pub affected_state_dampens_vibrations: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VibrationValidation {
    Accepted,
    RejectedSpectator,
    RejectedSneaking { trigger_avoid_advancement: bool },
    RejectedDampeningEntity,
    RejectedDampeningBlock,
    RejectedNotListenable,
    RejectedOccluded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VibrationTickAction {
    None,
    Selected {
        travel_time_in_ticks: i32,
        send_particle: bool,
    },
    ReloadParticle {
        travel_time_in_ticks: i32,
    },
    Received {
        event_id: &'static str,
        frequency: u8,
    },
}

pub fn vibration_frequency(event_id: &str) -> u8 {
    match event_id {
        "minecraft:step" | "minecraft:swim" | "minecraft:flap" => 1,
        "minecraft:projectile_land" | "minecraft:hit_ground" | "minecraft:splash" => 2,
        "minecraft:item_interact_finish"
        | "minecraft:projectile_shoot"
        | "minecraft:instrument_play" => 3,
        "minecraft:entity_action" | "minecraft:elytra_glide" | "minecraft:unequip" => 4,
        "minecraft:entity_dismount" | "minecraft:equip" => 5,
        "minecraft:entity_interact" | "minecraft:shear" | "minecraft:entity_mount" => 6,
        "minecraft:entity_damage" => 7,
        "minecraft:drink" | "minecraft:eat" => 8,
        "minecraft:container_close"
        | "minecraft:block_close"
        | "minecraft:block_deactivate"
        | "minecraft:block_detach" => 9,
        "minecraft:container_open"
        | "minecraft:block_open"
        | "minecraft:block_activate"
        | "minecraft:block_attach"
        | "minecraft:prime_fuse"
        | "minecraft:note_block_play" => 10,
        "minecraft:block_change" => 11,
        "minecraft:block_destroy" | "minecraft:fluid_pickup" => 12,
        "minecraft:block_place" | "minecraft:fluid_place" => 13,
        "minecraft:entity_place" | "minecraft:lightning_strike" | "minecraft:teleport" => 14,
        "minecraft:entity_die" | "minecraft:explode" => 15,
        id if id.starts_with("minecraft:resonate_") => id
            .trim_start_matches("minecraft:resonate_")
            .parse::<u8>()
            .ok()
            .filter(|frequency| (1..=15).contains(frequency))
            .unwrap_or(NO_VIBRATION_FREQUENCY),
        _ => NO_VIBRATION_FREQUENCY,
    }
}

pub fn resonance_event_by_frequency(frequency: u8) -> Option<GameEventDefinition> {
    if (1..=15).contains(&frequency) {
        let id = format!("minecraft:resonate_{frequency}");
        game_event_by_id(&id)
    } else {
        None
    }
}

pub fn redstone_strength_for_distance(distance: f32, listener_radius: i32) -> u8 {
    let power_scale = 15.0 / listener_radius as f32;
    (15 - (power_scale * distance).floor() as i32).max(1) as u8
}

pub fn validate_vibration(
    event: GameEventDefinition,
    context: &GameEventContext,
    state: VibrationSourceState,
    can_trigger_avoid_vibration: bool,
    event_is_listenable: bool,
    event_ignores_sneaking: bool,
    occluded: bool,
) -> VibrationValidation {
    if !event_is_listenable || vibration_frequency(event.id) == NO_VIBRATION_FREQUENCY {
        return VibrationValidation::RejectedNotListenable;
    }
    if context.source_entity.is_some() {
        if state.spectator {
            return VibrationValidation::RejectedSpectator;
        }
        if state.stepping_carefully && event_ignores_sneaking {
            return VibrationValidation::RejectedSneaking {
                trigger_avoid_advancement: can_trigger_avoid_vibration,
            };
        }
        if state.dampens_vibrations {
            return VibrationValidation::RejectedDampeningEntity;
        }
    }
    if state.affected_state_dampens_vibrations {
        return VibrationValidation::RejectedDampeningBlock;
    }
    if occluded {
        return VibrationValidation::RejectedOccluded;
    }
    VibrationValidation::Accepted
}

pub fn tick_vibration(data: &mut VibrationData, game_time: i64) -> VibrationTickAction {
    if data.current_vibration.is_none() {
        if let Some(candidate) = data.selector.chosen_candidate(game_time).cloned() {
            let travel_time_in_ticks = candidate.distance.floor() as i32;
            data.current_vibration = Some(candidate);
            data.travel_time_in_ticks = travel_time_in_ticks;
            data.selector.start_over();
            return VibrationTickAction::Selected {
                travel_time_in_ticks,
                send_particle: true,
            };
        }
    }

    let Some(current) = data.current_vibration.clone() else {
        return VibrationTickAction::None;
    };
    if data.reload_vibration_particle {
        data.reload_vibration_particle = false;
        return VibrationTickAction::ReloadParticle {
            travel_time_in_ticks: data.travel_time_in_ticks,
        };
    }
    data.decrement_travel_time();
    if data.travel_time_in_ticks <= 0 {
        data.current_vibration = None;
        VibrationTickAction::Received {
            event_id: current.event.id,
            frequency: vibration_frequency(current.event.id),
        }
    } else {
        VibrationTickAction::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SculkSensorAction {
    Ignore,
    Activate { frequency: u8, redstone: u8 },
    Cooldown,
    Deactivate,
}

pub fn sculk_sensor_receive(
    event_id: &str,
    distance: f32,
    listener_radius: i32,
) -> SculkSensorAction {
    let frequency = vibration_frequency(event_id);
    if frequency == NO_VIBRATION_FREQUENCY {
        SculkSensorAction::Ignore
    } else {
        SculkSensorAction::Activate {
            frequency,
            redstone: redstone_strength_for_distance(distance, listener_radius),
        }
    }
}

pub fn calibrated_sculk_sensor_receive(
    side_power: u8,
    event_id: &str,
    distance: f32,
    listener_radius: i32,
) -> SculkSensorAction {
    let action = sculk_sensor_receive(event_id, distance, listener_radius);
    match action {
        SculkSensorAction::Activate { frequency, .. }
            if side_power == 0 || side_power == frequency =>
        {
            action
        }
        SculkSensorAction::Activate { .. } => SculkSensorAction::Ignore,
        _ => action,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllayVibrationAction {
    ListenForJukebox { radius: i32 },
    Dance,
    StopDancing,
    Ignore,
}

pub fn allay_receive_event(event_id: &str, within_jukebox_radius: bool) -> AllayVibrationAction {
    match (event_id, within_jukebox_radius) {
        ("minecraft:jukebox_play", true) => AllayVibrationAction::Dance,
        ("minecraft:jukebox_stop_play", _) => AllayVibrationAction::StopDancing,
        ("minecraft:note_block_play", _) => AllayVibrationAction::ListenForJukebox { radius: 10 },
        _ => AllayVibrationAction::Ignore,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WardenVibrationAction {
    Ignore,
    TendrilClick {
        cooldown_ticks: i32,
    },
    IncreaseAnger {
        target: String,
        amount: i32,
        recent_projectile_ticks: Option<i32>,
    },
    Investigate {
        disturbance: (i32, i32, i32),
    },
}

pub fn warden_receive_vibration(
    source_entity: Option<&str>,
    projectile_owner: Option<&str>,
    source_pos: (i32, i32, i32),
    projectile_owner_within_range: bool,
    has_recent_projectile: bool,
    angry: bool,
) -> Vec<WardenVibrationAction> {
    let mut actions = vec![WardenVibrationAction::TendrilClick {
        cooldown_ticks: WARDEN_VIBRATION_COOLDOWN_TICKS,
    }];

    if let Some(owner) = projectile_owner {
        if projectile_owner_within_range {
            actions.push(WardenVibrationAction::IncreaseAnger {
                target: owner.to_string(),
                amount: if has_recent_projectile {
                    WARDEN_PROJECTILE_ANGER_BONUS
                } else {
                    1
                },
                recent_projectile_ticks: Some(WARDEN_RECENT_PROJECTILE_TICKS),
            });
        }
    } else if let Some(source) = source_entity {
        actions.push(WardenVibrationAction::IncreaseAnger {
            target: source.to_string(),
            amount: 1,
            recent_projectile_ticks: None,
        });
    }

    if !angry {
        actions.push(WardenVibrationAction::Investigate {
            disturbance: source_pos,
        });
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::{
        allay_receive_event, calibrated_sculk_sensor_receive, redstone_strength_for_distance,
        resonance_event_by_frequency, sculk_sensor_receive, tick_vibration, validate_vibration,
        vibration_frequency, warden_receive_vibration, AllayVibrationAction, SculkSensorAction,
        VibrationData, VibrationInfo, VibrationSelector, VibrationSourceState, VibrationTickAction,
        VibrationValidation, WardenVibrationAction, WARDEN_PROJECTILE_ANGER_BONUS,
        WARDEN_PROJECTILE_OWNER_RANGE, WARDEN_RECENT_PROJECTILE_TICKS,
        WARDEN_VIBRATION_COOLDOWN_TICKS,
    };
    use crate::entity_physics::Vec3;
    use crate::game_event::{game_event_by_id, GameEventContext};

    fn info(event_id: &'static str, distance: f32) -> VibrationInfo {
        VibrationInfo {
            event: game_event_by_id(event_id).unwrap(),
            distance,
            pos: Vec3 {
                x: distance as f64,
                y: 64.0,
                z: 0.0,
            },
            source_entity: Some("source".to_string()),
            projectile_owner: None,
        }
    }

    #[test]
    fn vibration_frequencies_and_resonance_events_match_vanilla_table() {
        assert_eq!(vibration_frequency("minecraft:step"), 1);
        assert_eq!(vibration_frequency("minecraft:projectile_land"), 2);
        assert_eq!(vibration_frequency("minecraft:block_change"), 11);
        assert_eq!(vibration_frequency("minecraft:explode"), 15);
        assert_eq!(vibration_frequency("minecraft:resonate_9"), 9);
        assert_eq!(vibration_frequency("minecraft:jukebox_play"), 0);
        assert_eq!(
            resonance_event_by_frequency(15).unwrap().id,
            "minecraft:resonate_15"
        );
        assert!(resonance_event_by_frequency(0).is_none());
    }

    #[test]
    fn selector_replaces_only_same_tick_by_distance_then_frequency() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info("minecraft:block_change", 8.0), 10);
        selector.add_candidate(info("minecraft:step", 4.0), 10);
        assert_eq!(
            selector.chosen_candidate(11).unwrap().event.id,
            "minecraft:step"
        );

        selector.start_over();
        selector.add_candidate(info("minecraft:step", 4.0), 12);
        selector.add_candidate(info("minecraft:explode", 4.0), 12);
        assert_eq!(
            selector.chosen_candidate(13).unwrap().event.id,
            "minecraft:explode"
        );

        selector.add_candidate(info("minecraft:block_destroy", 1.0), 14);
        assert_eq!(
            selector.chosen_candidate(15).unwrap().event.id,
            "minecraft:explode"
        );
    }

    #[test]
    fn validation_rejects_spectators_sneaking_dampening_unlisted_and_occluded_events() {
        let event = game_event_by_id("minecraft:step").unwrap();
        let context = GameEventContext::source("player");
        assert_eq!(
            validate_vibration(
                event,
                &context,
                VibrationSourceState {
                    spectator: true,
                    stepping_carefully: false,
                    dampens_vibrations: false,
                    affected_state_dampens_vibrations: false,
                },
                false,
                true,
                true,
                false,
            ),
            VibrationValidation::RejectedSpectator
        );
        assert_eq!(
            validate_vibration(
                event,
                &context,
                VibrationSourceState {
                    spectator: false,
                    stepping_carefully: true,
                    dampens_vibrations: false,
                    affected_state_dampens_vibrations: false,
                },
                true,
                true,
                true,
                false,
            ),
            VibrationValidation::RejectedSneaking {
                trigger_avoid_advancement: true
            }
        );
        assert_eq!(
            validate_vibration(
                event,
                &context,
                VibrationSourceState {
                    spectator: false,
                    stepping_carefully: false,
                    dampens_vibrations: true,
                    affected_state_dampens_vibrations: false,
                },
                false,
                true,
                false,
                false,
            ),
            VibrationValidation::RejectedDampeningEntity
        );
        assert_eq!(
            validate_vibration(
                event,
                &context,
                VibrationSourceState {
                    spectator: false,
                    stepping_carefully: false,
                    dampens_vibrations: false,
                    affected_state_dampens_vibrations: false,
                },
                false,
                true,
                false,
                true,
            ),
            VibrationValidation::RejectedOccluded
        );
    }

    #[test]
    fn vibration_tick_selects_after_tick_delay_sends_particle_and_receives_after_travel() {
        let mut data = VibrationData::new();
        data.selector
            .add_candidate(info("minecraft:block_destroy", 3.8), 5);
        assert_eq!(tick_vibration(&mut data, 5), VibrationTickAction::None);
        assert_eq!(
            tick_vibration(&mut data, 6),
            VibrationTickAction::Selected {
                travel_time_in_ticks: 3,
                send_particle: true
            }
        );
        assert_eq!(tick_vibration(&mut data, 7), VibrationTickAction::None);
        assert_eq!(tick_vibration(&mut data, 8), VibrationTickAction::None);
        assert_eq!(
            tick_vibration(&mut data, 9),
            VibrationTickAction::Received {
                event_id: "minecraft:block_destroy",
                frequency: 12
            }
        );
    }

    #[test]
    fn sculk_sensor_and_calibrated_sensor_convert_frequency_to_redstone() {
        assert_eq!(redstone_strength_for_distance(0.0, 16), 15);
        assert_eq!(redstone_strength_for_distance(15.9, 16), 1);
        assert_eq!(
            sculk_sensor_receive("minecraft:explode", 4.0, 16),
            SculkSensorAction::Activate {
                frequency: 15,
                redstone: 12
            }
        );
        assert_eq!(
            calibrated_sculk_sensor_receive(15, "minecraft:explode", 4.0, 16),
            SculkSensorAction::Activate {
                frequency: 15,
                redstone: 12
            }
        );
        assert_eq!(
            calibrated_sculk_sensor_receive(5, "minecraft:explode", 4.0, 16),
            SculkSensorAction::Ignore
        );
    }

    #[test]
    fn allay_and_warden_receive_events_match_listener_side_effects() {
        assert_eq!(
            allay_receive_event("minecraft:note_block_play", false),
            AllayVibrationAction::ListenForJukebox { radius: 10 }
        );
        assert_eq!(
            allay_receive_event("minecraft:jukebox_play", true),
            AllayVibrationAction::Dance
        );
        assert_eq!(
            warden_receive_vibration(Some("arrow"), Some("player"), (1, 64, 1), true, true, false,),
            vec![
                WardenVibrationAction::TendrilClick {
                    cooldown_ticks: WARDEN_VIBRATION_COOLDOWN_TICKS
                },
                WardenVibrationAction::IncreaseAnger {
                    target: "player".to_string(),
                    amount: WARDEN_PROJECTILE_ANGER_BONUS,
                    recent_projectile_ticks: Some(WARDEN_RECENT_PROJECTILE_TICKS)
                },
                WardenVibrationAction::Investigate {
                    disturbance: (1, 64, 1)
                }
            ]
        );
        assert_eq!(WARDEN_PROJECTILE_OWNER_RANGE, 30.0);
    }
}
