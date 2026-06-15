#![allow(dead_code)]

use crate::runtime::TARGET_TPS;

const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_MILLISECOND: f64 = 1_000_000.0;
const MILLIS_PER_SECOND: f64 = 1000.0;

#[derive(Debug, Clone, PartialEq)]
pub struct ServerTickRateManager {
    tick_rate: f32,
    nanoseconds_per_tick: u64,
    frozen_ticks_to_run: i32,
    run_game_elements: bool,
    frozen: bool,
    remaining_sprint_ticks: i64,
    sprint_tick_start_time: u64,
    sprint_time_spend: u64,
    scheduled_current_sprint_ticks: i64,
    previous_is_frozen: bool,
    events: Vec<ServerTickRateEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerTickRateEvent {
    BroadcastState { tick_rate: f32, frozen: bool },
    BroadcastStep { tick_steps: i32 },
    JoiningPlayerState { player_id: String, tick_rate: f32, frozen: bool },
    JoiningPlayerStep { player_id: String, tick_steps: i32 },
    SprintReport {
        ticks_per_second: i32,
        milliseconds_per_tick: String,
    },
    TickRateChanged,
}

impl Default for ServerTickRateManager {
    fn default() -> Self {
        Self {
            tick_rate: TARGET_TPS as f32,
            nanoseconds_per_tick: NANOS_PER_SECOND / u64::from(TARGET_TPS),
            frozen_ticks_to_run: 0,
            run_game_elements: true,
            frozen: false,
            remaining_sprint_ticks: 0,
            sprint_tick_start_time: 0,
            sprint_time_spend: 0,
            scheduled_current_sprint_ticks: 0,
            previous_is_frozen: false,
            events: Vec::new(),
        }
    }
}

impl ServerTickRateManager {
    pub fn tick_rate(&self) -> f32 {
        self.tick_rate
    }

    pub fn milliseconds_per_tick(&self) -> f32 {
        self.nanoseconds_per_tick as f32 / NANOS_PER_MILLISECOND as f32
    }

    pub fn frozen_ticks_to_run(&self) -> i32 {
        self.frozen_ticks_to_run
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn runs_normally(&self) -> bool {
        self.run_game_elements
    }

    pub fn is_sprinting(&self) -> bool {
        self.scheduled_current_sprint_ticks > 0
    }

    pub fn events(&self) -> &[ServerTickRateEvent] {
        &self.events
    }

    pub fn take_events(&mut self) -> Vec<ServerTickRateEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn tick(&mut self) {
        self.run_game_elements = !self.frozen || self.frozen_ticks_to_run > 0;
        if self.frozen_ticks_to_run > 0 {
            self.frozen_ticks_to_run -= 1;
        }
    }

    pub fn set_tick_rate(&mut self, rate: f32) {
        self.tick_rate = rate.max(1.0);
        self.nanoseconds_per_tick = (f64::from(NANOS_PER_SECOND as u32) / f64::from(self.tick_rate)) as u64;
        self.events.push(ServerTickRateEvent::TickRateChanged);
        self.update_state_to_clients();
    }

    pub fn set_frozen(&mut self, frozen: bool) {
        self.frozen = frozen;
        self.update_state_to_clients();
    }

    fn update_state_to_clients(&mut self) {
        self.events.push(ServerTickRateEvent::BroadcastState {
            tick_rate: self.tick_rate,
            frozen: self.frozen,
        });
    }

    fn update_step_ticks(&mut self) {
        self.events.push(ServerTickRateEvent::BroadcastStep {
            tick_steps: self.frozen_ticks_to_run,
        });
    }

    pub fn step_game_if_paused(&mut self, ticks: i32) -> bool {
        if !self.frozen {
            return false;
        }

        self.frozen_ticks_to_run = ticks;
        self.update_step_ticks();
        true
    }

    pub fn stop_stepping(&mut self) -> bool {
        if self.frozen_ticks_to_run > 0 {
            self.frozen_ticks_to_run = 0;
            self.update_step_ticks();
            true
        } else {
            false
        }
    }

    pub fn request_game_to_sprint(&mut self, time: i32) -> bool {
        let interrupted = self.remaining_sprint_ticks > 0;
        self.sprint_time_spend = 0;
        self.scheduled_current_sprint_ticks = i64::from(time);
        self.remaining_sprint_ticks = i64::from(time);
        self.previous_is_frozen = self.frozen;
        self.set_frozen(false);
        interrupted
    }

    pub fn stop_sprinting(&mut self) -> bool {
        if self.remaining_sprint_ticks > 0 {
            self.finish_tick_sprint();
            true
        } else {
            false
        }
    }

    pub fn check_should_sprint_this_tick_at(&mut self, now_nanos: u64) -> bool {
        if !self.run_game_elements {
            return false;
        }
        if self.remaining_sprint_ticks > 0 {
            self.sprint_tick_start_time = now_nanos;
            self.remaining_sprint_ticks -= 1;
            true
        } else {
            self.finish_tick_sprint();
            false
        }
    }

    pub fn end_tick_work_at(&mut self, now_nanos: u64) {
        self.sprint_time_spend = self
            .sprint_time_spend
            .saturating_add(now_nanos.saturating_sub(self.sprint_tick_start_time));
    }

    pub fn update_joining_player(&mut self, player_id: impl Into<String>) {
        let player_id = player_id.into();
        self.events.push(ServerTickRateEvent::JoiningPlayerState {
            player_id: player_id.clone(),
            tick_rate: self.tick_rate,
            frozen: self.frozen,
        });
        self.events.push(ServerTickRateEvent::JoiningPlayerStep {
            player_id,
            tick_steps: self.frozen_ticks_to_run,
        });
    }

    fn finish_tick_sprint(&mut self) {
        let completed_ticks = self.scheduled_current_sprint_ticks - self.remaining_sprint_ticks;
        let milliseconds_to_complete =
            self.sprint_time_spend.max(1) as f64 / NANOS_PER_MILLISECOND;
        let ticks_per_second =
            (MILLIS_PER_SECOND * completed_ticks as f64 / milliseconds_to_complete) as i32;
        let milliseconds_per_tick = if completed_ticks == 0 {
            self.milliseconds_per_tick()
        } else {
            (milliseconds_to_complete / completed_ticks as f64) as f32
        };

        self.scheduled_current_sprint_ticks = 0;
        self.sprint_time_spend = 0;
        self.events.push(ServerTickRateEvent::SprintReport {
            ticks_per_second,
            milliseconds_per_tick: format!("{milliseconds_per_tick:.2}"),
        });
        self.remaining_sprint_ticks = 0;
        self.set_frozen(self.previous_is_frozen);
        self.events.push(ServerTickRateEvent::TickRateChanged);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/ServerTickRateManager.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_tick_rate_manager_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public class ServerTickRateManager extends TickRateManager"));
        assert!(JAVA_SOURCE.contains("private long remainingSprintTicks = 0L;"));
        assert!(JAVA_SOURCE.contains("private boolean previousIsFrozen = false;"));
        assert!(JAVA_SOURCE.contains("this.server.getPlayerList().broadcastAll(ClientboundTickingStatePacket.from(this));"));
        assert!(JAVA_SOURCE.contains("this.server.getPlayerList().broadcastAll(ClientboundTickingStepPacket.from(this));"));
        assert!(JAVA_SOURCE.contains("public boolean stepGameIfPaused(final int ticks)"));
        assert!(JAVA_SOURCE.contains("public boolean stopStepping()"));
        assert!(JAVA_SOURCE.contains("public boolean stopSprinting()"));
        assert!(JAVA_SOURCE.contains("public boolean requestGameToSprint(final int time)"));
        assert!(JAVA_SOURCE.contains("public boolean checkShouldSprintThisTick()"));
        assert!(JAVA_SOURCE.contains("public void endTickWork()"));
        assert!(JAVA_SOURCE.contains("public void updateJoiningPlayer(final ServerPlayer player)"));
    }

    #[test]
    fn server_tick_rate_manager_broadcasts_state_and_step_updates_like_java() {
        let mut manager = ServerTickRateManager::default();

        manager.set_frozen(true);
        assert!(manager.step_game_if_paused(3));
        assert!(manager.stop_stepping());
        assert!(!manager.stop_stepping());

        assert_eq!(
            manager.events(),
            &[
                ServerTickRateEvent::BroadcastState {
                    tick_rate: 20.0,
                    frozen: true,
                },
                ServerTickRateEvent::BroadcastStep { tick_steps: 3 },
                ServerTickRateEvent::BroadcastStep { tick_steps: 0 },
            ]
        );
    }

    #[test]
    fn server_tick_rate_manager_sprint_restores_previous_freeze_and_reports() {
        let mut manager = ServerTickRateManager::default();
        manager.set_frozen(true);
        manager.take_events();

        assert!(!manager.request_game_to_sprint(2));
        assert!(!manager.is_frozen());
        assert!(manager.check_should_sprint_this_tick_at(10));
        manager.end_tick_work_at(2_000_010);
        assert!(manager.check_should_sprint_this_tick_at(2_000_020));
        manager.end_tick_work_at(4_000_020);
        assert!(!manager.check_should_sprint_this_tick_at(4_000_030));

        assert!(manager.is_frozen());
        assert!(!manager.is_sprinting());
        assert!(manager.events().contains(&ServerTickRateEvent::SprintReport {
            ticks_per_second: 500,
            milliseconds_per_tick: "2.00".to_string(),
        }));
        assert!(manager.events().contains(&ServerTickRateEvent::TickRateChanged));
    }

    #[test]
    fn server_tick_rate_manager_syncs_joining_player_with_state_then_step() {
        let mut manager = ServerTickRateManager::default();
        manager.set_frozen(true);
        assert!(manager.step_game_if_paused(7));
        manager.take_events();

        manager.update_joining_player("Alex");

        assert_eq!(
            manager.events(),
            &[
                ServerTickRateEvent::JoiningPlayerState {
                    player_id: "Alex".to_string(),
                    tick_rate: 20.0,
                    frozen: true,
                },
                ServerTickRateEvent::JoiningPlayerStep {
                    player_id: "Alex".to_string(),
                    tick_steps: 7,
                },
            ]
        );
    }
}
