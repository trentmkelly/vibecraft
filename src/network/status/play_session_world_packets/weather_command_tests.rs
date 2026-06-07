use super::*;
use crate::command::{WeatherMode, WeatherState};

fn data(clear: i32, rain: i32, thunder: i32, raining: bool, thundering: bool) -> WeatherData {
    WeatherData {
        clear_weather_time: clear,
        rain_time: rain,
        thunder_time: thunder,
        raining,
        thundering,
    }
}

#[test]
fn weather_state_maps_cycle_to_command_model() {
    assert_eq!(
        weather_state_from_cycle(&WeatherCycle::new(data(5000, 0, 0, false, false))),
        WeatherState {
            mode: WeatherMode::Clear,
            duration_ticks: Some(5000)
        }
    );
    assert_eq!(
        weather_state_from_cycle(&WeatherCycle::new(data(0, 6000, 6000, true, false))),
        WeatherState {
            mode: WeatherMode::Rain,
            duration_ticks: Some(6000)
        }
    );
    assert_eq!(
        weather_state_from_cycle(&WeatherCycle::new(data(0, 3000, 3000, true, true))),
        WeatherState {
            mode: WeatherMode::Thunder,
            duration_ticks: Some(3000)
        }
    );
}

#[test]
fn apply_weather_command_mutates_live_cycle_like_java() {
    let mut cycle = WeatherCycle::new(data(0, 0, 0, false, false));
    // /weather rain 6000
    apply_weather_state_to_cycle(
        &mut cycle,
        WeatherState {
            mode: WeatherMode::Rain,
            duration_ticks: Some(6000),
        },
    );
    assert!(cycle.data.raining);
    assert!(!cycle.data.thundering);
    assert_eq!(cycle.data.rain_time, 6000);
    assert_eq!(cycle.rain_level, 1.0);

    // /weather clear with no duration samples the vanilla RAIN_DELAY range.
    apply_weather_state_to_cycle(
        &mut cycle,
        WeatherState {
            mode: WeatherMode::Clear,
            duration_ticks: None,
        },
    );
    assert!(!cycle.data.raining);
    assert_eq!(cycle.rain_level, 0.0);
    assert!((12_000..=180_000).contains(&cycle.data.clear_weather_time));
}
