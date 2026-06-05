#![allow(dead_code)]

//! Server-side evaluators for overworld environment attributes driven by the day/night cycle.
//!
//! The client computes all sky rendering via its own Timeline copy; this module serves
//! server-only queries: daylight sensors, mob sunburn, bee/eyeblossom/creaking behaviour,
//! slime spawn, and lighting calculations.
//!
//! All keyframe data is copied verbatim from Java's Timelines.java.
//! Java refs:
//!   net/minecraft/world/timeline/Timelines.java:36–175  — keyframe data
//!   net/minecraft/util/EasingType.java                  — CubicBezier Newton-Raphson solver
//!   net/minecraft/world/attribute/EnvironmentAttributes.java — attribute definitions

use crate::world_time::DAY_LENGTH_TICKS;

// ---------------------------------------------------------------------------
// Cubic bezier easing
// ---------------------------------------------------------------------------

/// Cubic bezier easing matching Java's `EasingType.CubicBezier`.
///
/// Uses Newton-Raphson with 4 iterations to invert the x-curve, then evaluates the y-curve.
/// Java: EasingType.java CubicBezier.apply(x) — NEWTON_RAPHSON_ITERATIONS = 4
pub struct CubicBezierEasing {
    /// xCurve coefficients: `((a * t + b) * t + c) * t`
    x_a: f32,
    x_b: f32,
    x_c: f32,
    /// yCurve coefficients (same polynomial form)
    y_a: f32,
    y_b: f32,
    y_c: f32,
}

impl CubicBezierEasing {
    /// Constructs from control points (x1, y1, x2, y2).
    /// Java: EasingType.curveFromControls(v1, v2) = CubicCurve(3v1-3v2+1, -6v1+3v2, 3v1)
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            x_a: 3.0 * x1 - 3.0 * x2 + 1.0,
            x_b: -6.0 * x1 + 3.0 * x2,
            x_c: 3.0 * x1,
            y_a: 3.0 * y1 - 3.0 * y2 + 1.0,
            y_b: -6.0 * y1 + 3.0 * y2,
            y_c: 3.0 * y1,
        }
    }

    /// Applies the easing to a normalised input `x` ∈ [0, 1].
    /// Java: CubicBezier.apply(float x)
    pub fn apply(&self, x: f32) -> f32 {
        let mut t = x;
        for _ in 0..4 {
            let gradient = (3.0 * self.x_a * t + 2.0 * self.x_b) * t + self.x_c;
            if gradient < 1.0e-5 {
                break;
            }
            let error = ((self.x_a * t + self.x_b) * t + self.x_c) * t - x;
            t -= error / gradient;
        }
        ((self.y_a * t + self.y_b) * t + self.y_c) * t
    }
}

/// `EasingType.symmetricCubicBezier(0.362F, 0.241F)` — used for celestial angles.
/// Java: Timelines.java:38 `EasingType skyAngleEase = EasingType.symmetricCubicBezier(0.362F, 0.241F)`
pub const SKY_ANGLE_EASING: CubicBezierEasing = CubicBezierEasing::new(0.362, 0.241, 0.638, 0.759);

// ---------------------------------------------------------------------------
// Piecewise linear helpers
// ---------------------------------------------------------------------------

/// Samples a piecewise-linear keyframe track (wrapped, period = DAY_LENGTH_TICKS).
///
/// Keyframes are `(tick, value)` pairs in ascending order within [0, period).
/// Outside keyframe ranges, the track wraps from the last keyframe back to the first.
fn sample_piecewise_linear(day_cycle_ticks: i64, keyframes: &[(i64, f32)]) -> f32 {
    debug_assert!(!keyframes.is_empty(), "keyframes must not be empty");
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    let n = keyframes.len();

    // Find segment: find the keyframe just before or at `t`.
    let (prev_tick, prev_val, next_tick, next_val) = if t < keyframes[0].0 {
        // Before the first keyframe: wrap from last keyframe to first.
        let last = keyframes[n - 1];
        let first = keyframes[0];
        (last.0 - DAY_LENGTH_TICKS, last.1, first.0, first.1)
    } else if t >= keyframes[n - 1].0 {
        // After the last keyframe: wrap from last keyframe to first of next period.
        let last = keyframes[n - 1];
        let first = keyframes[0];
        (last.0, last.1, first.0 + DAY_LENGTH_TICKS, first.1)
    } else {
        // Find the surrounding keyframes.
        let idx = keyframes.partition_point(|&(k, _)| k <= t) - 1;
        let (pk, pv) = keyframes[idx];
        let (nk, nv) = keyframes[idx + 1];
        (pk, pv, nk, nv)
    };

    let span = (next_tick - prev_tick) as f32;
    if span <= 0.0 {
        return next_val;
    }
    let frac = (t - prev_tick) as f32 / span;
    prev_val + frac * (next_val - prev_val)
}

// ---------------------------------------------------------------------------
// Sun and celestial angles
// ---------------------------------------------------------------------------

/// Sun angle in degrees (0–360), evaluated using `symmetricCubicBezier(0.362, 0.241)` easing.
///
/// Track has two keyframes at tick 6000 (noon): `[(6000, 360.0), (6000, 0.0)]`.
/// This creates two segments per period:
///   - tick 0..6000: bezier(t/6000) × 360° (fast sunrise to zenith)
///   - tick 6000..24000: bezier((t-6000)/18000) × 360° (slow zenith to next sunrise)
///
/// Java: Timelines.java:54 SUN_ANGLE track, EasingType.symmetricCubicBezier(0.362F, 0.241F)
pub fn sun_angle(day_cycle_ticks: i64) -> f32 {
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    let frac = if t < 6_000 {
        t as f32 / 6_000.0
    } else {
        (t - 6_000) as f32 / 18_000.0
    };
    SKY_ANGLE_EASING.apply(frac) * 360.0
}

/// Moon angle in degrees (0–360). Identical easing to sun_angle; moon is 180° offset.
///
/// Track: `[(6000, 540.0), (6000, 180.0)]`; same two-segment structure.
/// Java: Timelines.java:55 MOON_ANGLE track
pub fn moon_angle(day_cycle_ticks: i64) -> f32 {
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    let frac = if t < 6_000 {
        t as f32 / 6_000.0
    } else {
        (t - 6_000) as f32 / 18_000.0
    };
    180.0 + SKY_ANGLE_EASING.apply(frac) * 360.0
}

// ---------------------------------------------------------------------------
// Sky light level
// ---------------------------------------------------------------------------

/// Piecewise-linear sky light multiplier (1.0 at noon → 0.267 at midnight).
///
/// Java: Timelines.java:81–83 FloatModifier.MULTIPLY track (LINEAR easing)
/// Keyframes: [(133, 1.0), (11867, 1.0), (13670, 0.267), (22330, 0.267)]
pub fn sky_light_level(day_cycle_ticks: i64) -> f32 {
    const KEYFRAMES: &[(i64, f32)] = &[
        (133, 1.0),
        (11867, 1.0),
        (13670, 0.266_666_68),
        (22330, 0.266_666_68),
    ];
    sample_piecewise_linear(day_cycle_ticks, KEYFRAMES)
}

// ---------------------------------------------------------------------------
// Star brightness
// ---------------------------------------------------------------------------

/// Star brightness multiplier (0.0 during day, peak 0.5 at midnight).
///
/// Java: Timelines.java:122–135 STAR_BRIGHTNESS FloatModifier.MAXIMUM track (LINEAR easing)
pub fn star_brightness(day_cycle_ticks: i64) -> f32 {
    const KEYFRAMES: &[(i64, f32)] = &[
        (92, 0.037),
        (627, 0.0),
        (11373, 0.0),
        (11732, 0.016),
        (11959, 0.044),
        (12399, 0.143),
        (12729, 0.258),
        (13228, 0.5),
        (22772, 0.5),
        (23032, 0.364),
        (23356, 0.225),
        (23758, 0.101),
    ];
    sample_piecewise_linear(day_cycle_ticks, KEYFRAMES)
}

// ---------------------------------------------------------------------------
// Boolean environment attributes (simple range checks)
// ---------------------------------------------------------------------------

/// Whether monsters should burn in sunlight (does not account for rain — caller must check).
///
/// True from tick 0 to 12542 (day) and from tick 23460 to 24000 (early morning).
/// Java: Timelines.java:157 BooleanModifier.OR: addKeyframe(12542, false).addKeyframe(23460, true)
pub fn monsters_burn(day_cycle_ticks: i64) -> bool {
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    !(12_542..23_460).contains(&t)
}

/// Whether bees should remain in their hive (night hours).
///
/// True from tick 12542 to 23460.
/// Java: Timelines.java:156 BooleanModifier.OR: addKeyframe(12542, true).addKeyframe(23460, false)
pub fn bees_stay_in_hive(day_cycle_ticks: i64) -> bool {
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    (12_542..23_460).contains(&t)
}

/// Whether an eyeblossom flower is open (night hours, slightly longer range).
///
/// Java: Timelines.java:144 addKeyframe(12600, TriState.TRUE).addKeyframe(23401, TriState.FALSE)
pub fn eyeblossom_open(day_cycle_ticks: i64) -> bool {
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    (12_600..23_401).contains(&t)
}

/// Whether a creaking is active (night hours, same range as eyeblossom).
///
/// Java: Timelines.java:145 BooleanModifier.OR: addKeyframe(12600, true).addKeyframe(23401, false)
pub fn creaking_active(day_cycle_ticks: i64) -> bool {
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    (12_600..23_401).contains(&t)
}

/// Whether firefly-bush ambient sounds play (night hours, same range as creaking).
///
/// Java: Timelines.java:56 `FIREFLY_BUSH_SOUNDS` BooleanModifier.OR:
/// addKeyframe(12600, true).addKeyframe(23401, false)
pub fn firefly_bush_sounds_active(day_cycle_ticks: i64) -> bool {
    let t = day_cycle_ticks.rem_euclid(DAY_LENGTH_TICKS);
    (12_600..23_401).contains(&t)
}

// ---------------------------------------------------------------------------
// Weather effect on SKY_LIGHT_LEVEL
// ---------------------------------------------------------------------------

/// `Mth.lerp(alpha, p0, p1)`.
fn lerp(alpha: f32, p0: f32, p1: f32) -> f32 {
    p0 + alpha * (p1 - p0)
}

// WeatherAttributes ALPHA_BLEND arguments for SKY_LIGHT_LEVEL (target 4.0).
// Java: net/minecraft/world/attribute/WeatherAttributes.java
//   RAIN    -> FloatWithAlpha(4.0F, 0.3125F)
//   THUNDER -> FloatWithAlpha(4.0F, 0.52734375F)
const SKY_LIGHT_WEATHER_TARGET: f32 = 4.0;
const SKY_LIGHT_RAIN_ALPHA: f32 = 0.3125;
const SKY_LIGHT_THUNDER_ALPHA: f32 = 0.527_343_75;

/// Apply the `WeatherAttributes` modifiers to a base `SKY_LIGHT_LEVEL`, 1:1 with
/// the `EnvironmentAttributes` evaluation: the rain entry blends the value toward
/// 4.0 (`FloatModifier.ALPHA_BLEND`, alpha 0.3125) scaled by the rain-minus-
/// thunder level via the float state-change lerp, then the thunder entry blends
/// toward 4.0 (alpha 0.52734375) scaled by the thunder level. `ALPHA_BLEND(s, v,
/// a) = lerp(a, s, v)` and the state-change lerp is `Mth.lerp`.
///
/// `rain_level`/`thunder_level` are the `0.0..=1.0` weather levels (thunder is a
/// subset of rain, so a full thunderstorm has both at 1.0).
pub fn weather_sky_light_level(base: f32, rain_level: f32, thunder_level: f32) -> f32 {
    let mut result = base;
    let rain_only = rain_level - thunder_level;
    if rain_only > 0.0 {
        let rain_value = lerp(SKY_LIGHT_RAIN_ALPHA, result, SKY_LIGHT_WEATHER_TARGET);
        result = lerp(rain_only, result, rain_value);
    }
    if thunder_level > 0.0 {
        let thunder_value = lerp(SKY_LIGHT_THUNDER_ALPHA, result, SKY_LIGHT_WEATHER_TARGET);
        result = lerp(thunder_level, result, thunder_value);
    }
    result
}

/// `skyDarken = (int)(15 - SKY_LIGHT_LEVEL)` from `Level.tickTime`, evaluated at
/// full daylight (base `SKY_LIGHT_LEVEL` 15) for the given weather levels. The
/// `(int)` cast truncates toward zero (these values are non-negative). At full
/// daylight: clear → 0 (light 15), rain → 3 (light 12), thunder → 5 (light 10).
pub fn weather_sky_darken(rain_level: f32, thunder_level: f32) -> i32 {
    (15.0 - weather_sky_light_level(15.0, rain_level, thunder_level)) as i32
}

// ---------------------------------------------------------------------------
// Convenience helpers
// ---------------------------------------------------------------------------

/// Whether it is currently daytime (sky light level above the 50% threshold).
///
/// Used to classify day/night for mob spawning, phantom insomnia, etc.
/// Threshold matches the `monsters_burn` boundary at tick 12542.
pub fn is_daytime(day_cycle_ticks: i64) -> bool {
    monsters_burn(day_cycle_ticks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monsters_burn_boundaries_match_java_timelines() {
        // At tick 12541 (last daytime tick before night), monsters should burn.
        assert!(monsters_burn(12_541));
        // At tick 12542 (first night tick), monsters should NOT burn.
        assert!(!monsters_burn(12_542));
        // At tick 23459 (last night tick), monsters should NOT burn.
        assert!(!monsters_burn(23_459));
        // At tick 23460 (first dawn tick), monsters should burn.
        assert!(monsters_burn(23_460));
        // Noon: monsters burn.
        assert!(monsters_burn(6_000));
        // Midnight: monsters don't burn.
        assert!(!monsters_burn(18_000));
    }

    #[test]
    fn monsters_burn_wraps_correctly_across_period() {
        // Tick 0 (start of day): daytime, burns.
        assert!(monsters_burn(0));
        // Tick 24000 == tick 0: same.
        assert!(monsters_burn(24_000));
        // Tick 24542 == tick 542: daytime.
        assert!(monsters_burn(24_542));
        // Tick 24000 + 18000 == tick 18000: midnight.
        assert!(!monsters_burn(24_000 + 18_000));
    }

    #[test]
    fn bees_stay_in_hive_is_complement_of_monsters_burn() {
        for tick in [0, 6_000, 12_541, 12_542, 18_000, 23_459, 23_460] {
            assert_eq!(
                bees_stay_in_hive(tick),
                !monsters_burn(tick),
                "bees/monsters burn should be complementary at tick {tick}"
            );
        }
    }

    #[test]
    fn night_boolean_tracks_share_the_12600_to_23401_window() {
        // eyeblossom, creaking, and firefly-bush sounds all use the same
        // Timelines keyframe window (12600 true → 23401 false).
        for tick in [0, 12_599, 12_600, 18_000, 23_400, 23_401] {
            let expected = (12_600..23_401).contains(&tick);
            assert_eq!(eyeblossom_open(tick), expected, "eyeblossom {tick}");
            assert_eq!(creaking_active(tick), expected, "creaking {tick}");
            assert_eq!(
                firefly_bush_sounds_active(tick),
                expected,
                "firefly {tick}"
            );
        }
    }

    #[test]
    fn sky_light_level_is_max_at_noon_and_min_at_midnight() {
        // Tick 6000 (noon) is well within the [133, 11867] max-light region.
        assert!((sky_light_level(6_000) - 1.0).abs() < 1e-4);
        // Tick 18000 (midnight) is well within the [13670, 22330] min-light region.
        assert!((sky_light_level(18_000) - 0.266_666_68).abs() < 1e-4);
    }

    #[test]
    fn weather_sky_light_level_blends_toward_four_per_weather_attributes() {
        // Clear: unchanged base.
        assert!((weather_sky_light_level(15.0, 0.0, 0.0) - 15.0).abs() < 1e-4);
        // Full rain: lerp(1.0, 15, lerp(0.3125, 15, 4.0)) = 11.5625.
        assert!((weather_sky_light_level(15.0, 1.0, 0.0) - 11.5625).abs() < 1e-4);
        // Full thunder (rain and thunder both 1.0; rain_only = 0):
        // lerp(1.0, 15, lerp(0.52734375, 15, 4.0)) = 9.19921875.
        assert!((weather_sky_light_level(15.0, 1.0, 1.0) - 9.199_219).abs() < 1e-4);

        // skyDarken = (int)(15 - SKY_LIGHT_LEVEL): clear 0, rain 3, thunder 5.
        assert_eq!(weather_sky_darken(0.0, 0.0), 0);
        assert_eq!(weather_sky_darken(1.0, 0.0), 3);
        assert_eq!(weather_sky_darken(1.0, 1.0), 5);

        // Partial rain (level 0.5) blends proportionally and still floors.
        let partial = weather_sky_light_level(15.0, 0.5, 0.0);
        assert!(partial > 11.5625 && partial < 15.0, "{partial}");
    }

    #[test]
    fn sky_light_level_transitions_between_day_and_night() {
        // At the exact transition keyframe ticks, values should match.
        assert!((sky_light_level(133) - 1.0).abs() < 1e-4);
        assert!((sky_light_level(11_867) - 1.0).abs() < 1e-4);
        assert!((sky_light_level(13_670) - 0.266_666_68).abs() < 1e-4);
        assert!((sky_light_level(22_330) - 0.266_666_68).abs() < 1e-4);
        // Dusk: between 11867 and 13670, should be between 0.267 and 1.0.
        let dusk = sky_light_level(12_768);
        assert!(dusk > 0.266 && dusk < 1.0);
    }

    #[test]
    fn star_brightness_is_zero_during_day_and_peaks_at_midnight() {
        // Day hours: very low star brightness.
        assert!((star_brightness(6_000)).abs() < 1e-4); // noon
        assert!((star_brightness(627)).abs() < 1e-4); // after dawn
        assert!((star_brightness(11_373)).abs() < 1e-4); // before dusk keyframe
                                                         // Midnight area: peak 0.5
        assert!((star_brightness(13_228) - 0.5).abs() < 1e-4);
        assert!((star_brightness(22_772) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn sun_angle_at_noon_approaches_360_from_segment_1() {
        // At tick 5999 (just before noon), should be near 360°.
        let angle = sun_angle(5_999);
        assert!(
            angle > 350.0,
            "sun angle before noon should be near 360, got {angle}"
        );
    }

    #[test]
    fn sun_angle_after_noon_starts_near_zero() {
        // At tick 6001 (just after noon), should be near 0°.
        let angle = sun_angle(6_001);
        assert!(
            angle < 10.0,
            "sun angle just after noon should be near 0, got {angle}"
        );
    }

    #[test]
    fn cubic_bezier_easing_identity_endpoints() {
        // Bezier must map 0→0 and 1→1.
        let easing = CubicBezierEasing::new(0.362, 0.241, 0.638, 0.759);
        assert!(easing.apply(0.0).abs() < 1e-5);
        assert!((easing.apply(1.0) - 1.0).abs() < 1e-4);
        // Midpoint should be 0.5 due to symmetry.
        assert!((easing.apply(0.5) - 0.5).abs() < 1e-4);
    }
}
