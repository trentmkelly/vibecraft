#![allow(dead_code)]

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WeatherData {
    pub clear_weather_time: i32,
    pub rain_time: i32,
    pub thunder_time: i32,
    pub raining: bool,
    pub thundering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherCycle {
    pub data: WeatherData,
    pub old_rain_level: f32,
    pub rain_level: f32,
    pub old_thunder_level: f32,
    pub thunder_level: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherRandomDurations {
    pub rain_delay: i32,
    pub rain_duration: i32,
    pub thunder_delay: i32,
    pub thunder_duration: i32,
}

impl WeatherRandomDurations {
    /// Sample fresh random durations matching Java `ServerLevel` ranges:
    /// - `RAIN_DELAY`: UniformInt.of(12000, 180000)
    /// - `RAIN_DURATION`: UniformInt.of(12000, 24000)
    /// - `THUNDER_DELAY`: UniformInt.of(12000, 180000)
    /// - `THUNDER_DURATION`: UniformInt.of(3600, 15600)
    pub fn sample_vanilla() -> Self {
        Self {
            rain_delay: uniform_int_sample(12_000, 180_000),
            rain_duration: uniform_int_sample(12_000, 24_000),
            thunder_delay: uniform_int_sample(12_000, 180_000),
            thunder_duration: uniform_int_sample(3_600, 15_600),
        }
    }
}

fn uniform_int_sample(min: i32, max: i32) -> i32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let state = RandomState::new();
    let mut h = state.build_hasher();
    h.write_u64(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0));
    let range = (max - min + 1) as u64;
    min + (h.finish() % range) as i32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherGameEvent {
    StartRaining,
    StopRaining,
    RainLevelChange(f32),
    ThunderLevelChange(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precipitation {
    None,
    Rain,
    Snow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherBlockAction {
    FreezeWater,
    PlaceSnow,
    AddSnowLayer(u8),
    HandlePrecipitation(Precipitation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrecipitationContext {
    pub top_y: i32,
    pub pos_y: i32,
    pub can_see_sky: bool,
    pub biome_precipitation: Precipitation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrecipitationTickContext {
    pub raining: bool,
    pub biome_should_freeze: bool,
    pub biome_should_snow: bool,
    pub current_snow_layers: Option<u8>,
    pub max_snow_accumulation_height: i32,
    pub below_precipitation: Precipitation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningTarget {
    LightningRod { x: i32, y: i32, z: i32 },
    LivingEntity { x: i32, y: i32, z: i32 },
    Heightmap { x: i32, y: i32, z: i32 },
}

impl WeatherCycle {
    pub fn new(data: WeatherData) -> Self {
        let mut cycle = Self {
            data,
            old_rain_level: 0.0,
            rain_level: 0.0,
            old_thunder_level: 0.0,
            thunder_level: 0.0,
        };
        cycle.prepare_weather();
        cycle
    }

    pub fn prepare_weather(&mut self) {
        if self.data.raining {
            self.set_rain_level(1.0);
            if self.data.thundering {
                self.set_thunder_level(1.0);
            }
        }
    }

    pub fn set_rain_level(&mut self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        self.old_rain_level = level;
        self.rain_level = level;
    }

    pub fn set_thunder_level(&mut self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        self.old_thunder_level = level;
        self.thunder_level = level;
    }

    pub fn rain_level(&self, partial_tick: f32) -> f32 {
        lerp(partial_tick, self.old_rain_level, self.rain_level)
    }

    pub fn thunder_level(&self, partial_tick: f32) -> f32 {
        lerp(partial_tick, self.old_thunder_level, self.thunder_level)
            * self.rain_level(partial_tick)
    }

    pub fn is_raining(&self, can_have_weather: bool) -> bool {
        can_have_weather && self.rain_level(1.0) > 0.2
    }

    pub fn is_thundering(&self, can_have_weather: bool) -> bool {
        can_have_weather && self.thunder_level(1.0) > 0.9
    }

    pub fn advance(
        &mut self,
        can_have_weather: bool,
        advance_weather_rule: bool,
        durations: WeatherRandomDurations,
    ) -> Vec<WeatherGameEvent> {
        let was_raining = self.is_raining(can_have_weather);
        if can_have_weather {
            if advance_weather_rule {
                self.tick_weather_data(durations);
            }
            self.old_thunder_level = self.thunder_level;
            self.thunder_level = if self.data.thundering {
                self.thunder_level + 0.01
            } else {
                self.thunder_level - 0.01
            }
            .clamp(0.0, 1.0);
            self.old_rain_level = self.rain_level;
            self.rain_level = if self.data.raining {
                self.rain_level + 0.01
            } else {
                self.rain_level - 0.01
            }
            .clamp(0.0, 1.0);
        }

        let mut events = Vec::new();
        if self.old_rain_level != self.rain_level {
            events.push(WeatherGameEvent::RainLevelChange(self.rain_level));
        }
        if self.old_thunder_level != self.thunder_level {
            events.push(WeatherGameEvent::ThunderLevelChange(self.thunder_level));
        }
        if was_raining != self.is_raining(can_have_weather) {
            events.push(if was_raining {
                WeatherGameEvent::StopRaining
            } else {
                WeatherGameEvent::StartRaining
            });
            events.push(WeatherGameEvent::RainLevelChange(self.rain_level));
            events.push(WeatherGameEvent::ThunderLevelChange(self.thunder_level));
        }
        events
    }

    fn tick_weather_data(&mut self, durations: WeatherRandomDurations) {
        if self.data.clear_weather_time > 0 {
            self.data.clear_weather_time -= 1;
            self.data.thunder_time = if self.data.thundering { 0 } else { 1 };
            self.data.rain_time = if self.data.raining { 0 } else { 1 };
            self.data.thundering = false;
            self.data.raining = false;
            return;
        }

        if self.data.thunder_time > 0 {
            self.data.thunder_time -= 1;
            if self.data.thunder_time == 0 {
                self.data.thundering = !self.data.thundering;
            }
        } else if self.data.thundering {
            self.data.thunder_time = durations.thunder_duration;
        } else {
            self.data.thunder_time = durations.thunder_delay;
        }

        if self.data.rain_time > 0 {
            self.data.rain_time -= 1;
            if self.data.rain_time == 0 {
                self.data.raining = !self.data.raining;
            }
        } else if self.data.raining {
            self.data.rain_time = durations.rain_duration;
        } else {
            self.data.rain_time = durations.rain_delay;
        }
    }
}

pub fn can_have_weather(has_skylight: bool, has_ceiling: bool, is_end: bool) -> bool {
    has_skylight && !has_ceiling && !is_end
}

pub fn precipitation_at(is_raining: bool, context: PrecipitationContext) -> Precipitation {
    if !is_raining
        || !context.can_see_sky
        || context.top_y > context.pos_y
        || context.biome_precipitation == Precipitation::None
    {
        Precipitation::None
    } else {
        context.biome_precipitation
    }
}

pub fn precipitation_tick_actions(context: PrecipitationTickContext) -> Vec<WeatherBlockAction> {
    let mut actions = Vec::new();
    if context.biome_should_freeze {
        actions.push(WeatherBlockAction::FreezeWater);
    }
    if context.raining {
        let max_height = context.max_snow_accumulation_height;
        if max_height > 0 && context.biome_should_snow {
            match context.current_snow_layers {
                Some(layers) if layers < max_height.min(8) as u8 => {
                    actions.push(WeatherBlockAction::AddSnowLayer(layers + 1));
                }
                Some(_) => {}
                None => actions.push(WeatherBlockAction::PlaceSnow),
            }
        }
        if context.below_precipitation != Precipitation::None {
            actions.push(WeatherBlockAction::HandlePrecipitation(
                context.below_precipitation,
            ));
        }
    }
    actions
}

pub fn lightning_tick_roll(raining: bool, thundering: bool, random_next_100000: i32) -> bool {
    raining && thundering && random_next_100000 == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningEntityEffect {
    None,
    ConvertToZombifiedPiglin,
    ConvertToWitch,
    ChargeCreeper,
}

pub fn lightning_entity_effect(entity_type: &str) -> LightningEntityEffect {
    match entity_type {
        "minecraft:pig" => LightningEntityEffect::ConvertToZombifiedPiglin,
        "minecraft:villager" => LightningEntityEffect::ConvertToWitch,
        "minecraft:creeper" => LightningEntityEffect::ChargeCreeper,
        _ => LightningEntityEffect::None,
    }
}

pub fn lightning_starts_fire(
    do_fire_tick: bool,
    is_peaceful: bool,
    target_block_is_air: bool,
    random_next_3: i32,
) -> bool {
    do_fire_tick && !is_peaceful && target_block_is_air && random_next_3 == 0
}

pub fn channeling_trident_summons_lightning(
    has_channeling: bool,
    thundering: bool,
    target_can_see_sky: bool,
    hit_living_entity: bool,
) -> bool {
    has_channeling && thundering && target_can_see_sky && hit_living_entity
}

pub fn skeleton_horse_trap_roll(
    spawn_mobs: bool,
    effective_difficulty: f64,
    random_next_double: f64,
    block_below_is_lightning_rod: bool,
) -> bool {
    spawn_mobs && random_next_double < effective_difficulty * 0.01 && !block_below_is_lightning_rod
}

pub fn choose_lightning_target(
    center: (i32, i32, i32),
    min_y: i32,
    max_y: i32,
    lightning_rod: Option<(i32, i32, i32)>,
    living_entities: &[(i32, i32, i32)],
    random_entity_index: usize,
) -> LightningTarget {
    if let Some((x, y, z)) = lightning_rod {
        return LightningTarget::LightningRod { x, y: y + 1, z };
    }
    if !living_entities.is_empty() {
        let (x, y, z) = living_entities[random_entity_index % living_entities.len()];
        return LightningTarget::LivingEntity { x, y, z };
    }
    let (x, y, z) = if center.1 == min_y - 1 {
        (center.0, center.1 + 2, center.2)
    } else {
        center
    };
    let y = y.min(max_y + 1);
    LightningTarget::Heightmap { x, y, z }
}

fn lerp(delta: f32, from: f32, to: f32) -> f32 {
    from + delta * (to - from)
}

/// Return the sky light reduction for the current weather state.
///
/// This is the pre-EnvironmentAttributes approximation (CLEAR=0, RAIN/THUNDER=5).
///
/// TODO(26.1.2 parity): vanilla no longer uses a flat −5. `Level.tickTime`
/// computes `skyDarken = (int)(15 - EnvironmentAttributes.SKY_LIGHT_LEVEL)`,
/// where `WeatherAttributes` modifies `SKY_LIGHT_LEVEL` by alpha-blending it
/// toward 4.0 (rain alpha 0.3125, thunder alpha 0.52734375) scaled by the
/// current rain/thunder levels. A faithful port needs the EnvironmentAttributes
/// / FloatModifier (ALPHA_BLEND) system; until then this returns the legacy
/// approximation and `effective_sky_light` is not 1:1 with 26.1.2.
pub fn sky_darken_amount(raining: bool, thundering: bool) -> i32 {
    if raining || thundering {
        5
    } else {
        0
    }
}

/// Full sky light level accounting for weather.
pub fn effective_sky_light(raining: bool, thundering: bool) -> i32 {
    15 - sky_darken_amount(raining, thundering)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SunburnableMobKind {
    Skeleton,
    Zombie,
    Phantom,
    Drowned,
    ZombieVillager,
    Husk,
    Stray,
    ZombifiedPiglin,
    Bogged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RainMobBehavior {
    None,
    DrownedRangedAttackEnabled,
    PillagerPatrolAllowed,
}

pub fn rain_mob_behavior(entity_type: &str, raining: bool, can_see_sky: bool) -> RainMobBehavior {
    if !raining || !can_see_sky {
        return RainMobBehavior::None;
    }
    match entity_type {
        "minecraft:drowned" => RainMobBehavior::DrownedRangedAttackEnabled,
        "minecraft:pillager" => RainMobBehavior::PillagerPatrolAllowed,
        _ => RainMobBehavior::None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobSunburnContext {
    pub kind: SunburnableMobKind,
    pub wearing_helmet: bool,
    pub in_water: bool,
    pub in_powder_snow: bool,
    /// The block sky light level at the mob's position (0-15)
    pub sky_light_at_pos: i32,
    pub raining: bool,
    pub day_cycle_time: i64, // 0-23999 ticks within the day
    pub can_see_sky: bool,
}

/// Returns true if the mob should catch fire this tick due to sunlight.
///
/// Uses `environment_attributes::monsters_burn` for exact vanilla day/night boundaries
/// (tick 12542 onset, tick 23460 end) instead of the former approximation.
/// Java: Monster.isSunBurnTick() + EnvironmentAttributes.MONSTERS_BURN Timeline track
pub fn mob_should_burn_in_sunlight(ctx: MobSunburnContext) -> bool {
    if !is_sun_sensitive(ctx.kind) {
        return false;
    }
    if ctx.raining {
        return false;
    }
    if !crate::environment_attributes::monsters_burn(ctx.day_cycle_time) {
        return false;
    }
    if ctx.wearing_helmet {
        return false;
    }
    if ctx.in_water || ctx.in_powder_snow {
        return false;
    }
    if !ctx.can_see_sky {
        return false;
    }
    // Sky light < 15 means in shade (under blocks, trees, etc.)
    if ctx.sky_light_at_pos < 15 {
        return false;
    }
    true
}

/// Returns true when the in-game time corresponds to daytime (monsters would burn).
///
/// Delegates to `environment_attributes::monsters_burn` for exact vanilla boundaries.
/// Java: Level.isDay() — used for phantom/bat spawning, etc.
pub fn is_daytime(day_cycle_time: i64) -> bool {
    crate::environment_attributes::monsters_burn(day_cycle_time)
}

/// Returns true when a mob of the given kind is sun-sensitive.
pub fn is_sun_sensitive(kind: SunburnableMobKind) -> bool {
    matches!(
        kind,
        SunburnableMobKind::Skeleton
            | SunburnableMobKind::Zombie
            | SunburnableMobKind::Phantom
            | SunburnableMobKind::Drowned
            | SunburnableMobKind::ZombieVillager
            | SunburnableMobKind::Stray
            | SunburnableMobKind::Bogged
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DURATIONS: WeatherRandomDurations = WeatherRandomDurations {
        rain_delay: 12_000,
        rain_duration: 6_000,
        thunder_delay: 18_000,
        thunder_duration: 3_000,
    };

    #[test]
    fn weather_data_clear_timer_forces_clear_and_starts_transition_packets() {
        let mut cycle = WeatherCycle::new(WeatherData {
            clear_weather_time: 2,
            rain_time: 99,
            thunder_time: 99,
            raining: true,
            thundering: true,
        });

        let events = cycle.advance(true, true, DURATIONS);

        assert_eq!(cycle.data.clear_weather_time, 1);
        assert_eq!(cycle.data.rain_time, 0);
        assert_eq!(cycle.data.thunder_time, 0);
        assert!(!cycle.data.raining);
        assert!(!cycle.data.thundering);
        assert_eq!(cycle.rain_level, 0.99);
        assert_eq!(cycle.thunder_level, 0.99);
        assert_eq!(
            events,
            vec![
                WeatherGameEvent::RainLevelChange(0.99),
                WeatherGameEvent::ThunderLevelChange(0.99)
            ]
        );
    }

    #[test]
    fn weather_cycle_toggles_rain_thunder_and_broadcasts_threshold_events() {
        let mut cycle = WeatherCycle::new(WeatherData {
            clear_weather_time: 0,
            rain_time: 1,
            thunder_time: 1,
            raining: false,
            thundering: false,
        });
        cycle.rain_level = 0.2;
        cycle.old_rain_level = 0.2;
        cycle.thunder_level = 0.9;
        cycle.old_thunder_level = 0.9;

        let events = cycle.advance(true, true, DURATIONS);

        assert!(cycle.data.raining);
        assert!(cycle.data.thundering);
        assert_eq!(cycle.data.rain_time, 0);
        assert_eq!(cycle.data.thunder_time, 0);
        assert_eq!(cycle.rain_level, 0.21000001);
        assert_eq!(cycle.thunder_level, 0.90999997);
        assert_eq!(
            events,
            vec![
                WeatherGameEvent::RainLevelChange(0.21000001),
                WeatherGameEvent::ThunderLevelChange(0.90999997),
                WeatherGameEvent::StartRaining,
                WeatherGameEvent::RainLevelChange(0.21000001),
                WeatherGameEvent::ThunderLevelChange(0.90999997),
            ]
        );
    }

    #[test]
    fn weather_rule_false_keeps_timers_but_levels_chase_saved_flags() {
        let mut cycle = WeatherCycle::new(WeatherData {
            clear_weather_time: 0,
            rain_time: 1,
            thunder_time: 1,
            raining: true,
            thundering: false,
        });
        cycle.set_rain_level(0.5);
        cycle.set_thunder_level(0.5);

        cycle.advance(true, false, DURATIONS);

        assert_eq!(cycle.data.rain_time, 1);
        assert_eq!(cycle.data.thunder_time, 1);
        assert_eq!(cycle.rain_level, 0.51);
        assert_eq!(cycle.thunder_level, 0.49);
        assert_eq!(cycle.thunder_level(1.0), 0.2499);
    }

    #[test]
    fn dimension_and_precipitation_gates_match_level_checks() {
        assert!(can_have_weather(true, false, false));
        assert!(!can_have_weather(false, false, false));
        assert!(!can_have_weather(true, true, false));
        assert!(!can_have_weather(true, false, true));

        assert_eq!(
            precipitation_at(
                true,
                PrecipitationContext {
                    top_y: 64,
                    pos_y: 64,
                    can_see_sky: true,
                    biome_precipitation: Precipitation::Snow,
                },
            ),
            Precipitation::Snow
        );
        assert_eq!(
            precipitation_at(
                true,
                PrecipitationContext {
                    top_y: 65,
                    pos_y: 64,
                    can_see_sky: true,
                    biome_precipitation: Precipitation::Rain,
                },
            ),
            Precipitation::None
        );
    }

    #[test]
    fn precipitation_tick_freezes_snow_accumulates_and_notifies_blocks() {
        assert_eq!(
            precipitation_tick_actions(PrecipitationTickContext {
                raining: true,
                biome_should_freeze: true,
                biome_should_snow: true,
                current_snow_layers: Some(2),
                max_snow_accumulation_height: 4,
                below_precipitation: Precipitation::Rain,
            }),
            vec![
                WeatherBlockAction::FreezeWater,
                WeatherBlockAction::AddSnowLayer(3),
                WeatherBlockAction::HandlePrecipitation(Precipitation::Rain),
            ]
        );
        assert_eq!(
            precipitation_tick_actions(PrecipitationTickContext {
                raining: true,
                biome_should_freeze: false,
                biome_should_snow: true,
                current_snow_layers: None,
                max_snow_accumulation_height: 8,
                below_precipitation: Precipitation::None,
            }),
            vec![WeatherBlockAction::PlaceSnow]
        );
    }

    #[test]
    fn lightning_roll_target_and_trap_rules_match_server_level_paths() {
        assert!(lightning_tick_roll(true, true, 0));
        assert!(!lightning_tick_roll(true, true, 1));
        assert!(skeleton_horse_trap_roll(true, 2.0, 0.01, false));
        assert!(!skeleton_horse_trap_roll(true, 2.0, 0.01, true));

        assert_eq!(
            choose_lightning_target((0, 70, 0), 0, 320, Some((4, 80, 4)), &[], 0),
            LightningTarget::LightningRod { x: 4, y: 81, z: 4 }
        );
        assert_eq!(
            choose_lightning_target((0, 70, 0), 0, 320, None, &[(1, 65, 1), (2, 66, 2)], 1),
            LightningTarget::LivingEntity { x: 2, y: 66, z: 2 }
        );
        assert_eq!(
            choose_lightning_target((0, -1, 0), 0, 320, None, &[], 0),
            LightningTarget::Heightmap { x: 0, y: 1, z: 0 }
        );
    }

    #[test]
    fn lightning_entity_fire_and_channeling_effects_match_vanilla_cases() {
        assert_eq!(
            lightning_entity_effect("minecraft:pig"),
            LightningEntityEffect::ConvertToZombifiedPiglin
        );
        assert_eq!(
            lightning_entity_effect("minecraft:villager"),
            LightningEntityEffect::ConvertToWitch
        );
        assert_eq!(
            lightning_entity_effect("minecraft:creeper"),
            LightningEntityEffect::ChargeCreeper
        );
        assert_eq!(
            lightning_entity_effect("minecraft:cow"),
            LightningEntityEffect::None
        );

        assert!(lightning_starts_fire(true, false, true, 0));
        assert!(!lightning_starts_fire(false, false, true, 0));
        assert!(!lightning_starts_fire(true, true, true, 0));
        assert!(!lightning_starts_fire(true, false, false, 0));
        assert!(!lightning_starts_fire(true, false, true, 1));

        assert!(channeling_trident_summons_lightning(true, true, true, true));
        assert!(!channeling_trident_summons_lightning(
            false, true, true, true
        ));
        assert!(!channeling_trident_summons_lightning(
            true, false, true, true
        ));
        assert!(!channeling_trident_summons_lightning(
            true, true, false, true
        ));
        assert!(!channeling_trident_summons_lightning(
            true, true, true, false
        ));
    }

    #[test]
    fn sky_darken_amount_matches_vanilla_clear_rain_thunder() {
        assert_eq!(sky_darken_amount(false, false), 0);
        assert_eq!(sky_darken_amount(true, false), 5);
        assert_eq!(sky_darken_amount(false, true), 5);
        assert_eq!(sky_darken_amount(true, true), 5);
        assert_eq!(effective_sky_light(false, false), 15);
        assert_eq!(effective_sky_light(true, false), 10);
        assert_eq!(effective_sky_light(false, true), 10);
    }

    #[test]
    fn mob_sunburn_conditions_match_vanilla_monster_tick_rules() {
        let clear_day_exposed = MobSunburnContext {
            kind: SunburnableMobKind::Skeleton,
            wearing_helmet: false,
            in_water: false,
            in_powder_snow: false,
            sky_light_at_pos: 15,
            raining: false,
            day_cycle_time: 6_000,
            can_see_sky: true,
        };

        // Skeleton in daylight, exposed → burns
        assert!(mob_should_burn_in_sunlight(clear_day_exposed));

        // Wearing helmet → no burn
        assert!(!mob_should_burn_in_sunlight(MobSunburnContext {
            wearing_helmet: true,
            ..clear_day_exposed
        }));

        // In water → no burn
        assert!(!mob_should_burn_in_sunlight(MobSunburnContext {
            in_water: true,
            ..clear_day_exposed
        }));

        // Raining → no burn
        assert!(!mob_should_burn_in_sunlight(MobSunburnContext {
            raining: true,
            ..clear_day_exposed
        }));

        // Nighttime (14000) → no burn
        assert!(!mob_should_burn_in_sunlight(MobSunburnContext {
            day_cycle_time: 14_000,
            ..clear_day_exposed
        }));

        // Under shade (sky light < 15) → no burn
        assert!(!mob_should_burn_in_sunlight(MobSunburnContext {
            sky_light_at_pos: 14,
            ..clear_day_exposed
        }));

        // Cannot see sky → no burn
        assert!(!mob_should_burn_in_sunlight(MobSunburnContext {
            can_see_sky: false,
            ..clear_day_exposed
        }));

        assert!(!mob_should_burn_in_sunlight(MobSunburnContext {
            kind: SunburnableMobKind::ZombifiedPiglin,
            ..clear_day_exposed
        }));

        // Daytime check boundaries — exact vanilla values from EnvironmentAttributes.MONSTERS_BURN.
        // Java: Timelines.java:157 BooleanModifier.OR: addKeyframe(12542, false).addKeyframe(23460, true)
        assert!(is_daytime(0));
        assert!(is_daytime(12_541)); // last daytime tick
        assert!(!is_daytime(12_542)); // first night tick
        assert!(!is_daytime(18_000)); // midnight
        assert!(!is_daytime(23_459)); // last night tick
        assert!(is_daytime(23_460)); // first dawn tick
    }

    #[test]
    fn sun_sensitive_mob_kinds_cover_vanilla_mob_list() {
        assert!(is_sun_sensitive(SunburnableMobKind::Skeleton));
        assert!(is_sun_sensitive(SunburnableMobKind::Zombie));
        assert!(is_sun_sensitive(SunburnableMobKind::Phantom));
        assert!(is_sun_sensitive(SunburnableMobKind::Stray));
        assert!(is_sun_sensitive(SunburnableMobKind::Drowned));
        assert!(!is_sun_sensitive(SunburnableMobKind::Husk));
        assert!(!is_sun_sensitive(SunburnableMobKind::ZombifiedPiglin));
    }

    #[test]
    fn rain_mob_behavior_covers_drowned_and_pillager_weather_cases() {
        assert_eq!(
            rain_mob_behavior("minecraft:drowned", true, true),
            RainMobBehavior::DrownedRangedAttackEnabled
        );
        assert_eq!(
            rain_mob_behavior("minecraft:pillager", true, true),
            RainMobBehavior::PillagerPatrolAllowed
        );
        assert_eq!(
            rain_mob_behavior("minecraft:drowned", false, true),
            RainMobBehavior::None
        );
        assert_eq!(
            rain_mob_behavior("minecraft:pillager", true, false),
            RainMobBehavior::None
        );
    }
}
