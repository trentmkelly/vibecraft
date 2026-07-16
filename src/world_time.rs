#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::registry::{Identifier, ResourceKey};
use crate::storage::nbt::Tag;

/// Java `WorldClock` is a zero-field record. Its direct codec encodes an empty
/// map, while registry holders carry the clock's resource key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldClock;

impl WorldClock {
    /// Java `WorldClock.DIRECT_CODEC`: a zero-field record is represented by
    /// an empty compound in the direct/NBT form.
    pub fn encode_direct(&self) -> Tag {
        Tag::Compound(Vec::new())
    }
}

/// Built-in world-clock keys and bootstrap order from Java's `WorldClocks`.
pub struct WorldClocks;

impl WorldClocks {
    pub fn overworld() -> Result<ResourceKey<WorldClock>, String> {
        Self::key("overworld")
    }

    pub fn the_end() -> Result<ResourceKey<WorldClock>, String> {
        Self::key("the_end")
    }

    /// Java `WorldClocks.bootstrap`: registers overworld before the End.
    pub fn bootstrap() -> Result<[ResourceKey<WorldClock>; 2], String> {
        Ok([Self::overworld()?, Self::the_end()?])
    }

    /// The registered values produced by Java's bootstrap method.
    pub fn bootstrap_values() -> Result<[(ResourceKey<WorldClock>, WorldClock); 2], String> {
        let keys = Self::bootstrap()?;
        Ok([(keys[0].clone(), WorldClock), (keys[1].clone(), WorldClock)])
    }

    fn key(id: &str) -> Result<ResourceKey<WorldClock>, String> {
        Ok(ResourceKey::new(
            Identifier::parse("minecraft:world_clock")?,
            Identifier::with_default_namespace(id)?,
        ))
    }
}

pub const DAY_LENGTH_TICKS: i64 = 24_000;
pub const MOON_CYCLE_TICKS: i64 = DAY_LENGTH_TICKS * 8;
pub const WAKE_UP_FROM_SLEEP_TIME: i64 = 0;
pub const PHANTOM_INSOMNIA_THRESHOLD_TICKS: i32 = 72_000;
pub const MOON_BRIGHTNESS_PER_PHASE: [f32; 8] = [1.0, 0.75, 0.5, 0.25, 0.0, 0.25, 0.5, 0.75];
pub const OVERWORLD_DAY_TIMELINE: &str = "minecraft:day";
pub const MOON_TIMELINE: &str = "minecraft:moon";
pub const VILLAGER_SCHEDULE_TIMELINE: &str = "minecraft:villager_schedule";
pub const EARLY_GAME_TIMELINE: &str = "minecraft:early_game";

/// VarInt registry IDs for the minecraft:world_clock registry, determined by registration order.
/// Java: net/minecraft/world/clock/WorldClocks.java:12–13
pub const OVERWORLD_CLOCK_ID: i32 = 0;
pub const THE_END_CLOCK_ID: i32 = 1;

/// Network state for a single world clock, transmitted in ClientboundSetTimePacket.
/// Rate is sent as 0.0 when the clock is paused or the ADVANCE_TIME gamerule is false,
/// which tells the client to freeze the clock in its Timeline system.
///
/// Java: net/minecraft/world/clock/ClockNetworkState.java
/// Wire: VAR_LONG totalTicks, FLOAT partialTick, FLOAT rate
#[derive(Debug, Clone, PartialEq)]
pub struct ClockNetworkState {
    pub total_ticks: i64,
    pub partial_tick: f32,
    pub rate: f32,
}

/// Server-side state for a single named world clock.
///
/// Java: net/minecraft/world/clock/ServerClockManager.java ClockInstance (inner class)
#[derive(Debug, Clone, PartialEq)]
pub struct ClockInstance {
    pub total_ticks: i64,
    pub partial_tick: f32,
    /// Fractional tick rate; 1.0 is normal speed. Java: ClockInstance.rate default 1.0
    pub rate: f32,
    pub paused: bool,
}

impl Default for ClockInstance {
    fn default() -> Self {
        Self {
            total_ticks: 0,
            partial_tick: 0.0,
            rate: 1.0,
            paused: false,
        }
    }
}

impl ClockInstance {
    /// Advances the clock by one game tick.
    /// Java: ServerClockManager.ClockInstance.tick()
    pub fn tick(&mut self) {
        if !self.paused {
            self.partial_tick += self.rate;
            let full = self.partial_tick.floor() as i64;
            self.partial_tick -= full as f32;
            self.total_ticks += full;
        }
    }

    /// Returns this clock's position within the current day cycle (0..24000).
    pub fn day_cycle_ticks(&self) -> i64 {
        self.total_ticks.rem_euclid(DAY_LENGTH_TICKS)
    }

    pub fn moon_phase(&self) -> MoonPhase {
        moon_phase(self.total_ticks)
    }

    pub fn moon_brightness(&self) -> f32 {
        MOON_BRIGHTNESS_PER_PHASE[self.moon_phase().index()]
    }

    /// Returns this clock's state packed for network transmission.
    /// Rate is sent as 0.0 when paused or advance_time is false (client freezes the clock).
    /// Java: ServerClockManager.ClockInstance.packNetworkState(MinecraftServer)
    pub fn pack_network_state(&self, advance_time: bool) -> ClockNetworkState {
        ClockNetworkState {
            total_ticks: self.total_ticks,
            partial_tick: self.partial_tick,
            rate: if self.paused || !advance_time {
                0.0
            } else {
                self.rate
            },
        }
    }
}

/// Manages all world clocks, tracks absolute server game time, and fires scheduled functions.
///
/// In vanilla Java, game_time is owned by the overworld ServerLevel (`getGameTime()`). In
/// VibeCraft we store it here because the full ServerLevel is not yet implemented.
///
/// Java: net/minecraft/world/clock/ServerClockManager.java
pub struct ServerClockManager {
    /// Absolute server tick counter. Increments every tick regardless of ADVANCE_TIME.
    /// Java: MinecraftServer overworld().getGameTime()
    pub game_time: i64,
    /// The overworld day/night cycle clock (minecraft:world_clock registry ID 0).
    pub overworld: ClockInstance,
    /// The End dimension clock (minecraft:world_clock registry ID 1).
    pub the_end: ClockInstance,
}

impl Default for ServerClockManager {
    fn default() -> Self {
        Self {
            game_time: 0,
            // Start at noon of day 1 (tick 6000) so first-join sky is not midnight black.
            overworld: ClockInstance {
                total_ticks: 6_000,
                ..ClockInstance::default()
            },
            the_end: ClockInstance::default(),
        }
    }
}

impl ServerClockManager {
    /// Advances all clocks by one game tick and returns any fired scheduled functions.
    ///
    /// `advance_time` mirrors the ADVANCE_TIME gamerule — when false, clock total_ticks do not
    /// advance but game_time still increments and scheduled functions still fire.
    ///
    /// Java: ServerClockManager.tick(), MinecraftServer.tickChildren()
    pub fn tick(
        &mut self,
        advance_time: bool,
        scheduled: &mut ScheduledTimeChanges,
    ) -> Vec<TimeEvent> {
        self.game_time += 1;
        if advance_time {
            self.overworld.tick();
            self.the_end.tick();
        }
        scheduled
            .pop_due(self.game_time)
            .into_iter()
            .map(TimeEvent::ScheduledFunction)
            .collect()
    }

    /// Returns the (game_time, clock_states) pair needed for a full sync packet.
    /// Called on player join so the client can initialize its Timeline system.
    ///
    /// Java: ServerClockManager.createFullSyncPacket()
    pub fn full_sync_data(&self, advance_time: bool) -> (i64, Vec<(i32, ClockNetworkState)>) {
        (
            self.game_time,
            vec![
                (
                    OVERWORLD_CLOCK_ID,
                    self.overworld.pack_network_state(advance_time),
                ),
                (
                    THE_END_CLOCK_ID,
                    self.the_end.pack_network_state(advance_time),
                ),
            ],
        )
    }

    /// Returns game_time for the periodic heartbeat packet (empty clock map).
    /// Java: MinecraftServer.forceGameTimeSynchronization() — broadcasts every 20 ticks
    pub fn heartbeat_game_time(&self) -> i64 {
        self.game_time
    }

    /// Moves the overworld clock to the next wake_up_from_sleep time marker (next day boundary).
    /// Returns a DayTimeChanged event with the new total_ticks, or None if advance_time is false.
    ///
    /// Java: ServerClockManager.moveToTimeMarker(OVERWORLD, WAKE_UP_FROM_SLEEP)
    pub fn move_overworld_to_wake_up_marker(&mut self, advance_time: bool) -> Option<TimeEvent> {
        if advance_time {
            let new_time = next_wake_up_time(self.overworld.total_ticks);
            self.overworld.total_ticks = new_time;
            self.overworld.partial_tick = 0.0;
            Some(TimeEvent::DayTimeChanged(new_time))
        } else {
            None
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
            .filter(|player| player.sleeping_long_enough)
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
    manager: &mut ServerClockManager,
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
    if let Some(event) = manager.move_overworld_to_wake_up_marker(advance_time_rule) {
        events.push(event);
    }
    for player in players.iter_mut() {
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

    #[cfg(vibecraft_has_decompiled_sources)]
    const WORLD_CLOCK_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/clock/WorldClock.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const WORLD_CLOCKS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/clock/WorldClocks.java");

    #[test]
    fn world_clock_keys_match_java_bootstrap_order() {
        let keys = match WorldClocks::bootstrap() {
            Ok(keys) => keys,
            Err(error) => panic!("failed to construct built-in world clock keys: {error}"),
        };
        assert_eq!(keys[0].location().to_string(), "minecraft:overworld");
        assert_eq!(keys[1].location().to_string(), "minecraft:the_end");
        assert_eq!(keys[0].registry().to_string(), "minecraft:world_clock");
        assert_eq!(keys[1].registry(), keys[0].registry());
        assert_eq!(OVERWORLD_CLOCK_ID, 0);
        assert_eq!(THE_END_CLOCK_ID, 1);
        let registrations = match WorldClocks::bootstrap_values() {
            Ok(registrations) => registrations,
            Err(error) => panic!("failed to construct world clock registrations: {error}"),
        };
        assert_eq!(registrations[0].0, keys[0]);
        assert_eq!(registrations[1].0, keys[1]);
        assert_eq!(registrations[0].1.encode_direct(), Tag::Compound(Vec::new()));
        assert_eq!(registrations[1].1.encode_direct(), Tag::Compound(Vec::new()));
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn world_clock_sources_match_zero_field_record_and_bootstrap() {
        assert_eq!(WORLD_CLOCK_JAVA.lines().count(), 16);
        for fragment in [
            "public record WorldClock()",
            "RegistryFixedCodec.create(Registries.WORLD_CLOCK)",
            "MapCodec.unitCodec(WorldClock::new)",
        ] {
            assert!(WORLD_CLOCK_JAVA.contains(fragment), "missing WorldClock source fragment: {fragment}");
        }
        assert_eq!(WORLD_CLOCKS_JAVA.lines().count(), 20);
        for fragment in [
            "public interface WorldClocks",
            "ResourceKey<WorldClock> OVERWORLD = key(\"overworld\");",
            "ResourceKey<WorldClock> THE_END = key(\"the_end\");",
            "context.register(OVERWORLD, new WorldClock());",
            "context.register(THE_END, new WorldClock());",
            "ResourceKey.create(Registries.WORLD_CLOCK, Identifier.withDefaultNamespace(id))",
        ] {
            assert!(WORLD_CLOCKS_JAVA.contains(fragment), "missing WorldClocks source fragment: {fragment}");
        }
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    const SLEEP_STATUS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/players/SleepStatus.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn sleep_status_source_matches_java_counting_and_threshold_surface() {
        for fragment in [
            "private int activePlayers",
            "private int sleepingPlayers",
            "sleepingPlayers >= this.sleepersNeeded(sleepPercentageNeeded)",
            "Mth.ceil(this.activePlayers * sleepPercentageNeeded / 100.0F)",
            "Player::isSleepingLongEnough",
            "player.isSpectator()",
            "this.sleepingPlayers = 0",
        ] {
            assert!(SLEEP_STATUS_JAVA.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn server_clock_manager_ticks_game_time_and_fires_scheduled_functions() {
        let mut manager = ServerClockManager::default();
        let mut scheduled = ScheduledTimeChanges::default();
        scheduled.schedule(1, "minecraft:tick_one", false);
        scheduled.schedule(3, "minecraft:tick_three", false);

        assert_eq!(
            manager.tick(true, &mut scheduled),
            vec![TimeEvent::ScheduledFunction(
                "minecraft:tick_one".to_string()
            )]
        );
        assert_eq!(manager.game_time, 1);
        assert!(manager.tick(true, &mut scheduled).is_empty());
        assert_eq!(
            manager.tick(true, &mut scheduled),
            vec![TimeEvent::ScheduledFunction(
                "minecraft:tick_three".to_string()
            )]
        );
        assert_eq!(manager.game_time, 3);

        // advance_time=false: game_time still increments but clock total_ticks do not advance.
        let ticks_before = manager.overworld.total_ticks;
        manager.tick(false, &mut ScheduledTimeChanges::default());
        assert_eq!(manager.game_time, 4);
        assert_eq!(manager.overworld.total_ticks, ticks_before);
    }

    #[test]
    fn clock_instance_moon_phase_and_wake_marker_follow_vanilla_clock_cycle() {
        let mut manager = ServerClockManager {
            game_time: 0,
            overworld: ClockInstance {
                total_ticks: 23_000,
                ..ClockInstance::default()
            },
            ..ServerClockManager::default()
        };
        manager.tick(true, &mut ScheduledTimeChanges::default());
        assert_eq!(manager.overworld.total_ticks, 23_001);
        assert_eq!(manager.overworld.day_cycle_ticks(), 23_001);
        assert_eq!(manager.overworld.moon_phase(), MoonPhase::FullMoon);
        assert_eq!(manager.overworld.moon_brightness(), 1.0);
        assert_eq!(moon_phase(4 * DAY_LENGTH_TICKS), MoonPhase::NewMoon);
        assert_eq!(MoonPhase::WaxingGibbous.start_tick(), 168_000);
        assert_eq!(MoonPhase::ThirdQuarter.serialized_name(), "third_quarter");

        assert_eq!(
            manager.move_overworld_to_wake_up_marker(true),
            Some(TimeEvent::DayTimeChanged(24_000))
        );
        assert_eq!(manager.overworld.total_ticks, 24_000);
        assert_eq!(manager.overworld.partial_tick, 0.0);

        // advance_time=false: marker not moved.
        assert_eq!(manager.move_overworld_to_wake_up_marker(false), None);
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
        let mut manager = ServerClockManager {
            game_time: 0,
            overworld: ClockInstance {
                total_ticks: 13_000,
                ..ClockInstance::default()
            },
            ..ServerClockManager::default()
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

        let events = apply_sleep_skip(
            &mut manager,
            &mut status,
            &mut players,
            100,
            true,
            true,
            true,
        );

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
        let mut manager = ServerClockManager {
            game_time: 0,
            overworld: ClockInstance {
                total_ticks: 13_000,
                ..ClockInstance::default()
            },
            ..ServerClockManager::default()
        };
        let event = manager.move_overworld_to_wake_up_marker(true).unwrap();
        assert_eq!(event, TimeEvent::DayTimeChanged(24_000));
        assert_eq!(manager.overworld.total_ticks, 24_000);

        // Day 1, time 37000 → skip to 48000 (start of day 2)
        let mut manager2 = ServerClockManager {
            game_time: 0,
            overworld: ClockInstance {
                total_ticks: 37_000,
                ..ClockInstance::default()
            },
            ..ServerClockManager::default()
        };
        let event2 = manager2.move_overworld_to_wake_up_marker(true).unwrap();
        assert_eq!(event2, TimeEvent::DayTimeChanged(48_000));
        assert_eq!(manager2.overworld.total_ticks, 48_000);

        // Time exactly 0 (already at day start) → skip to 24000
        let mut manager3 = ServerClockManager {
            game_time: 0,
            overworld: ClockInstance {
                total_ticks: 0,
                ..ClockInstance::default()
            },
            ..ServerClockManager::default()
        };
        let event3 = manager3.move_overworld_to_wake_up_marker(true).unwrap();
        assert_eq!(event3, TimeEvent::DayTimeChanged(24_000));

        // Verify day_cycle_ticks after skip: 24000 mod 24000 = 0 (start of day)
        assert_eq!(manager.overworld.day_cycle_ticks(), 0);
        assert_eq!(manager2.overworld.day_cycle_ticks(), 0);
    }

    #[test]
    fn clock_instance_tick_advances_partial_and_floors_correctly() {
        // Java: ClockInstance.tick() — partial_tick += rate, floor() gives full ticks to add
        let mut clock = ClockInstance::default();
        // tick once at rate=1.0: partial_tick = 1.0 → floor=1 → total_ticks=1, partial_tick=0.0
        clock.tick();
        assert_eq!(clock.total_ticks, 1);
        assert_eq!(clock.partial_tick, 0.0);

        // rate=0.5: two ticks needed for one total_tick increment
        let mut slow_clock = ClockInstance {
            rate: 0.5,
            ..ClockInstance::default()
        };
        slow_clock.tick();
        assert_eq!(slow_clock.total_ticks, 0);
        assert!((slow_clock.partial_tick - 0.5).abs() < f32::EPSILON);
        slow_clock.tick();
        assert_eq!(slow_clock.total_ticks, 1);
        assert_eq!(slow_clock.partial_tick, 0.0);

        // paused: no change
        let mut paused = ClockInstance {
            paused: true,
            ..ClockInstance::default()
        };
        paused.tick();
        assert_eq!(paused.total_ticks, 0);
    }

    #[test]
    fn clock_instance_pack_network_state_zeroes_rate_when_paused_or_frozen() {
        // Java: ClockInstance.packNetworkState — rate=0 when paused or !advanceTime
        let clock = ClockInstance {
            total_ticks: 100,
            partial_tick: 0.25,
            rate: 2.0,
            paused: false,
        };
        let state = clock.pack_network_state(true);
        assert_eq!(state.rate, 2.0);

        let state_frozen = clock.pack_network_state(false);
        assert_eq!(state_frozen.rate, 0.0);

        let paused_clock = ClockInstance {
            paused: true,
            ..clock.clone()
        };
        let state_paused = paused_clock.pack_network_state(true);
        assert_eq!(state_paused.rate, 0.0);
    }
}
