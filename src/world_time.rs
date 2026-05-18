#![allow(dead_code)]

use std::collections::BTreeMap;

pub const DAY_LENGTH_TICKS: i64 = 24_000;
pub const MOON_CYCLE_TICKS: i64 = DAY_LENGTH_TICKS * 8;
pub const WAKE_UP_FROM_SLEEP_TIME: i64 = 0;
pub const PHANTOM_INSOMNIA_THRESHOLD_TICKS: i32 = 72_000;
pub const MOON_BRIGHTNESS_PER_PHASE: [f32; 8] = [1.0, 0.75, 0.5, 0.25, 0.0, 0.25, 0.5, 0.75];
pub const OVERWORLD_DAY_TIMELINE: &str = "minecraft:day";
pub const MOON_TIMELINE: &str = "minecraft:moon";
pub const VILLAGER_SCHEDULE_TIMELINE: &str = "minecraft:villager_schedule";
pub const EARLY_GAME_TIMELINE: &str = "minecraft:early_game";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldClock {
    pub game_time: i64,
    pub day_time: i64,
    pub tick_time: bool,
    pub has_default_clock: bool,
}

impl Default for WorldClock {
    fn default() -> Self {
        Self {
            game_time: 0,
            day_time: 0,
            tick_time: true,
            has_default_clock: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    FullMoon,
    WaningGibbous,
    ThirdQuarter,
    WaningCrescent,
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSleepState {
    pub spectator: bool,
    pub sleeping: bool,
    pub sleeping_long_enough: bool,
    pub time_since_rest: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SleepStatus {
    pub active_players: i32,
    pub sleeping_players: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeEvent {
    ScheduledFunction(String),
    SleepOverlay(&'static str),
    WakePlayers,
    ResetWeatherCycle,
    DayTimeChanged(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineDefinition {
    pub id: &'static str,
    pub clock: &'static str,
    pub period_ticks: i64,
    pub markers: &'static [(&'static str, i64)],
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScheduledTimeChanges {
    events: BTreeMap<i64, Vec<String>>,
}

impl WorldClock {
    pub fn tick_time(&mut self, scheduled: &mut ScheduledTimeChanges) -> Vec<TimeEvent> {
        if !self.tick_time {
            return Vec::new();
        }
        self.game_time += 1;
        scheduled
            .pop_due(self.game_time)
            .into_iter()
            .map(TimeEvent::ScheduledFunction)
            .collect()
    }

    pub fn advance_day_time(&mut self, advance_time_rule: bool) {
        if advance_time_rule && self.has_default_clock {
            self.day_time += 1;
        }
    }

    pub fn set_day_time(&mut self, day_time: i64) -> TimeEvent {
        self.day_time = day_time;
        TimeEvent::DayTimeChanged(day_time)
    }

    pub fn move_to_wake_up_marker(&mut self, advance_time_rule: bool) -> Option<TimeEvent> {
        if advance_time_rule && self.has_default_clock {
            Some(self.set_day_time(next_wake_up_time(self.day_time)))
        } else {
            None
        }
    }

    pub fn day_cycle_time(self) -> i64 {
        self.day_time.rem_euclid(DAY_LENGTH_TICKS)
    }

    pub fn moon_phase(self) -> MoonPhase {
        moon_phase(self.day_time)
    }

    pub fn moon_brightness(self) -> f32 {
        MOON_BRIGHTNESS_PER_PHASE[self.moon_phase().index()]
    }
}

impl MoonPhase {
    pub fn index(self) -> usize {
        match self {
            MoonPhase::FullMoon => 0,
            MoonPhase::WaningGibbous => 1,
            MoonPhase::ThirdQuarter => 2,
            MoonPhase::WaningCrescent => 3,
            MoonPhase::NewMoon => 4,
            MoonPhase::WaxingCrescent => 5,
            MoonPhase::FirstQuarter => 6,
            MoonPhase::WaxingGibbous => 7,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            MoonPhase::FullMoon => "full_moon",
            MoonPhase::WaningGibbous => "waning_gibbous",
            MoonPhase::ThirdQuarter => "third_quarter",
            MoonPhase::WaningCrescent => "waning_crescent",
            MoonPhase::NewMoon => "new_moon",
            MoonPhase::WaxingCrescent => "waxing_crescent",
            MoonPhase::FirstQuarter => "first_quarter",
            MoonPhase::WaxingGibbous => "waxing_gibbous",
        }
    }

    pub fn start_tick(self) -> i64 {
        self.index() as i64 * DAY_LENGTH_TICKS
    }
}

impl SleepStatus {
    pub fn sleepers_needed(self, sleep_percentage_needed: i32) -> i32 {
        ((self.active_players as f32 * sleep_percentage_needed as f32 / 100.0).ceil() as i32).max(1)
    }

    pub fn are_enough_sleeping(self, sleep_percentage_needed: i32) -> bool {
        self.sleeping_players >= self.sleepers_needed(sleep_percentage_needed)
    }

    pub fn are_enough_deep_sleeping(
        self,
        sleep_percentage_needed: i32,
        players: &[PlayerSleepState],
    ) -> bool {
        let deep_sleepers = players
            .iter()
            .filter(|player| !player.spectator && player.sleeping_long_enough)
            .count() as i32;
        deep_sleepers >= self.sleepers_needed(sleep_percentage_needed)
    }

    pub fn remove_all_sleepers(&mut self) {
        self.sleeping_players = 0;
    }

    pub fn update(&mut self, players: &[PlayerSleepState]) -> bool {
        let old_active_players = self.active_players;
        let old_sleeping_players = self.sleeping_players;
        self.active_players = 0;
        self.sleeping_players = 0;
        for player in players {
            if !player.spectator {
                self.active_players += 1;
                if player.sleeping {
                    self.sleeping_players += 1;
                }
            }
        }
        (old_sleeping_players > 0 || self.sleeping_players > 0)
            && (old_active_players != self.active_players
                || old_sleeping_players != self.sleeping_players)
    }
}

impl ScheduledTimeChanges {
    pub fn schedule(&mut self, trigger_tick: i64, function: impl Into<String>, replace: bool) {
        let function = function.into();
        let events = self.events.entry(trigger_tick).or_default();
        if replace {
            events.retain(|existing| existing != &function);
        }
        events.push(function);
    }

    pub fn pop_due(&mut self, game_time: i64) -> Vec<String> {
        let due_ticks = self
            .events
            .range(..=game_time)
            .map(|(tick, _)| *tick)
            .collect::<Vec<_>>();
        let mut due = Vec::new();
        for tick in due_ticks {
            if let Some(mut events) = self.events.remove(&tick) {
                due.append(&mut events);
            }
        }
        due
    }
}

pub fn moon_phase(day_time: i64) -> MoonPhase {
    match (day_time.rem_euclid(MOON_CYCLE_TICKS) / DAY_LENGTH_TICKS) as usize {
        0 => MoonPhase::FullMoon,
        1 => MoonPhase::WaningGibbous,
        2 => MoonPhase::ThirdQuarter,
        3 => MoonPhase::WaningCrescent,
        4 => MoonPhase::NewMoon,
        5 => MoonPhase::WaxingCrescent,
        6 => MoonPhase::FirstQuarter,
        _ => MoonPhase::WaxingGibbous,
    }
}

pub fn next_wake_up_time(day_time: i64) -> i64 {
    let days_elapsed = day_time.div_euclid(DAY_LENGTH_TICKS);
    (days_elapsed + 1) * DAY_LENGTH_TICKS + WAKE_UP_FROM_SLEEP_TIME
}

pub fn should_skip_night(
    status: SleepStatus,
    players: &[PlayerSleepState],
    sleep_percentage_needed: i32,
) -> bool {
    status.are_enough_sleeping(sleep_percentage_needed)
        && status.are_enough_deep_sleeping(sleep_percentage_needed, players)
}

pub fn sleep_status_overlay_key(status: SleepStatus, sleep_percentage_needed: i32) -> &'static str {
    if status.are_enough_sleeping(sleep_percentage_needed) {
        "sleep.skipping_night"
    } else {
        "sleep.players_sleeping"
    }
}

pub fn apply_sleep_skip(
    clock: &mut WorldClock,
    status: &mut SleepStatus,
    players: &mut [PlayerSleepState],
    sleep_percentage_needed: i32,
    advance_time_rule: bool,
    advance_weather_rule: bool,
    raining: bool,
) -> Vec<TimeEvent> {
    if !should_skip_night(*status, players, sleep_percentage_needed) {
        return Vec::new();
    }
    let mut events = Vec::new();
    if let Some(event) = clock.move_to_wake_up_marker(advance_time_rule) {
        events.push(event);
    }
    for player in players {
        if player.sleeping {
            player.sleeping = false;
            player.sleeping_long_enough = false;
            player.time_since_rest = 0;
        }
    }
    status.remove_all_sleepers();
    events.push(TimeEvent::WakePlayers);
    if advance_weather_rule && raining {
        events.push(TimeEvent::ResetWeatherCycle);
    }
    events
}

pub fn tick_insomnia(players: &mut [PlayerSleepState], spawn_phantoms_rule: bool) {
    if spawn_phantoms_rule {
        for player in players {
            if !player.spectator && !player.sleeping {
                player.time_since_rest = player.time_since_rest.saturating_add(1);
            }
        }
    }
}

pub fn is_insomniac(player: PlayerSleepState, spawn_phantoms_rule: bool) -> bool {
    spawn_phantoms_rule
        && !player.spectator
        && !player.sleeping
        && player.time_since_rest >= PHANTOM_INSOMNIA_THRESHOLD_TICKS
}

pub fn builtin_timelines() -> &'static [TimelineDefinition] {
    const DAY_MARKERS: &[(&str, i64)] = &[
        ("minecraft:day", 1_000),
        ("minecraft:noon", 6_000),
        ("minecraft:night", 13_000),
        ("minecraft:midnight", 18_000),
        ("minecraft:wake_up_from_sleep", 0),
        ("minecraft:roll_village_siege", 18_000),
    ];
    const VILLAGER_MARKERS: &[(&str, i64)] = &[
        ("minecraft:work", 2_000),
        ("minecraft:home", 9_000),
        ("minecraft:rest", 12_000),
    ];
    const TIMELINES: &[TimelineDefinition] = &[
        TimelineDefinition {
            id: OVERWORLD_DAY_TIMELINE,
            clock: "minecraft:overworld",
            period_ticks: DAY_LENGTH_TICKS,
            markers: DAY_MARKERS,
        },
        TimelineDefinition {
            id: MOON_TIMELINE,
            clock: "minecraft:overworld",
            period_ticks: MOON_CYCLE_TICKS,
            markers: &[],
        },
        TimelineDefinition {
            id: VILLAGER_SCHEDULE_TIMELINE,
            clock: "minecraft:overworld",
            period_ticks: DAY_LENGTH_TICKS,
            markers: VILLAGER_MARKERS,
        },
        TimelineDefinition {
            id: EARLY_GAME_TIMELINE,
            clock: "minecraft:overworld",
            period_ticks: DAY_LENGTH_TICKS,
            markers: &[],
        },
    ];
    TIMELINES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_clock_ticks_game_time_and_due_scheduled_functions() {
        let mut clock = WorldClock::default();
        let mut scheduled = ScheduledTimeChanges::default();
        scheduled.schedule(1, "minecraft:tick_one", false);
        scheduled.schedule(3, "minecraft:tick_three", false);

        assert_eq!(
            clock.tick_time(&mut scheduled),
            vec![TimeEvent::ScheduledFunction(
                "minecraft:tick_one".to_string()
            )]
        );
        assert_eq!(clock.game_time, 1);
        assert!(clock.tick_time(&mut scheduled).is_empty());
        assert_eq!(
            clock.tick_time(&mut scheduled),
            vec![TimeEvent::ScheduledFunction(
                "minecraft:tick_three".to_string()
            )]
        );

        clock.tick_time = false;
        assert!(clock.tick_time(&mut scheduled).is_empty());
        assert_eq!(clock.game_time, 3);
    }

    #[test]
    fn day_time_moon_phase_and_wake_marker_follow_vanilla_clock_cycle() {
        let mut clock = WorldClock {
            day_time: 23_000,
            ..WorldClock::default()
        };
        clock.advance_day_time(true);
        assert_eq!(clock.day_time, 23_001);
        assert_eq!(clock.day_cycle_time(), 23_001);
        assert_eq!(clock.moon_phase(), MoonPhase::FullMoon);
        assert_eq!(clock.moon_brightness(), 1.0);
        assert_eq!(moon_phase(4 * DAY_LENGTH_TICKS), MoonPhase::NewMoon);
        assert_eq!(MoonPhase::WaxingGibbous.start_tick(), 168_000);
        assert_eq!(MoonPhase::ThirdQuarter.serialized_name(), "third_quarter");

        assert_eq!(
            clock.move_to_wake_up_marker(true),
            Some(TimeEvent::DayTimeChanged(24_000))
        );
        assert_eq!(clock.day_time, 24_000);

        clock.has_default_clock = false;
        assert_eq!(clock.move_to_wake_up_marker(true), None);
    }

    #[test]
    fn sleep_status_counts_active_sleepers_and_announces_changes() {
        let players = vec![
            PlayerSleepState {
                spectator: false,
                sleeping: true,
                sleeping_long_enough: true,
                time_since_rest: 10,
            },
            PlayerSleepState {
                spectator: false,
                sleeping: false,
                sleeping_long_enough: false,
                time_since_rest: 10,
            },
            PlayerSleepState {
                spectator: true,
                sleeping: true,
                sleeping_long_enough: true,
                time_since_rest: 10,
            },
        ];
        let mut status = SleepStatus::default();

        assert!(status.update(&players));
        assert_eq!(status.active_players, 2);
        assert_eq!(status.sleeping_players, 1);
        assert_eq!(status.sleepers_needed(50), 1);
        assert!(status.are_enough_sleeping(50));
        assert!(status.are_enough_deep_sleeping(50, &players));
        assert_eq!(sleep_status_overlay_key(status, 50), "sleep.skipping_night");
        assert_eq!(
            sleep_status_overlay_key(status, 100),
            "sleep.players_sleeping"
        );
    }

    #[test]
    fn sleep_skip_wakes_players_resets_insomnia_and_optionally_clears_weather() {
        let mut clock = WorldClock {
            day_time: 13000,
            ..WorldClock::default()
        };
        let mut players = vec![
            PlayerSleepState {
                spectator: false,
                sleeping: true,
                sleeping_long_enough: true,
                time_since_rest: 80_000,
            },
            PlayerSleepState {
                spectator: false,
                sleeping: true,
                sleeping_long_enough: true,
                time_since_rest: 74_000,
            },
        ];
        let mut status = SleepStatus::default();
        status.update(&players);

        let events = apply_sleep_skip(&mut clock, &mut status, &mut players, 100, true, true, true);

        assert_eq!(
            events,
            vec![
                TimeEvent::DayTimeChanged(24_000),
                TimeEvent::WakePlayers,
                TimeEvent::ResetWeatherCycle,
            ]
        );
        assert_eq!(status.sleeping_players, 0);
        assert!(players.iter().all(|player| !player.sleeping));
        assert!(players.iter().all(|player| player.time_since_rest == 0));
    }

    #[test]
    fn insomnia_ticks_only_eligible_awake_players_and_uses_phantom_threshold() {
        let mut players = vec![
            PlayerSleepState {
                spectator: false,
                sleeping: false,
                sleeping_long_enough: false,
                time_since_rest: 71_999,
            },
            PlayerSleepState {
                spectator: true,
                sleeping: false,
                sleeping_long_enough: false,
                time_since_rest: 71_999,
            },
            PlayerSleepState {
                spectator: false,
                sleeping: true,
                sleeping_long_enough: false,
                time_since_rest: 71_999,
            },
        ];

        tick_insomnia(&mut players, true);

        assert_eq!(players[0].time_since_rest, PHANTOM_INSOMNIA_THRESHOLD_TICKS);
        assert_eq!(players[1].time_since_rest, 71_999);
        assert_eq!(players[2].time_since_rest, 71_999);
        assert!(is_insomniac(players[0], true));
        assert!(!is_insomniac(players[0], false));
    }

    #[test]
    fn builtin_timeline_surface_matches_new_registry_entries() {
        let timelines = builtin_timelines();
        assert_eq!(
            timelines
                .iter()
                .map(|timeline| timeline.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:day",
                "minecraft:moon",
                "minecraft:villager_schedule",
                "minecraft:early_game"
            ]
        );
        assert_eq!(timelines[0].period_ticks, DAY_LENGTH_TICKS);
        assert!(timelines[0]
            .markers
            .contains(&("minecraft:wake_up_from_sleep", 0)));
        assert_eq!(timelines[1].period_ticks, MOON_CYCLE_TICKS);
        assert!(timelines
            .iter()
            .all(|timeline| timeline.clock == "minecraft:overworld"));
    }

    #[test]
    fn sleep_skip_jumps_to_next_day_start_not_just_dawn_zero() {
        // Vanilla behavior: sleep skip advances to next day boundary (next multiple of 24000),
        // NOT to time 0 or simple dawn. The wake marker is (days_elapsed+1) * 24000 + 0.

        // Day 0, time 13000 (night) → skip to 24000 (start of day 1)
        let mut clock = WorldClock {
            day_time: 13_000,
            ..WorldClock::default()
        };
        let event = clock.move_to_wake_up_marker(true).unwrap();
        assert_eq!(event, TimeEvent::DayTimeChanged(24_000));
        assert_eq!(clock.day_time, 24_000);

        // Day 1, time 37000 → skip to 48000 (start of day 2)
        let mut clock2 = WorldClock {
            day_time: 37_000,
            ..WorldClock::default()
        };
        let event2 = clock2.move_to_wake_up_marker(true).unwrap();
        assert_eq!(event2, TimeEvent::DayTimeChanged(48_000));
        assert_eq!(clock2.day_time, 48_000);

        // Time exactly 0 (already at day start) → skip to 24000
        let mut clock3 = WorldClock {
            day_time: 0,
            ..WorldClock::default()
        };
        let event3 = clock3.move_to_wake_up_marker(true).unwrap();
        assert_eq!(event3, TimeEvent::DayTimeChanged(24_000));

        // Verify day_cycle_time after skip: 24000 mod 24000 = 0 (start of day)
        assert_eq!(clock.day_cycle_time(), 0);
        assert_eq!(clock2.day_cycle_time(), 0);
    }
}
