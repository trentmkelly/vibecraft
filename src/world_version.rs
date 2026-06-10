#![allow(dead_code)]

use serde_json::Value;

use crate::resources::PackFormat;

pub const CURRENT_VERSION_ID: &str = "26.1.2";
pub const CURRENT_VERSION_NAME: &str = "26.1.2";
pub const CURRENT_DATA_VERSION: i32 = 4790;
pub const CURRENT_DATA_SERIES: &str = "main";
pub const CURRENT_PROTOCOL_VERSION: i32 = 775;
pub const CURRENT_BUILD_TIME: &str = "2026-04-09T10:11:03+00:00";
pub const CURRENT_BUILD_TIME_EPOCH_MILLIS: i64 = 1_775_729_463_000;
pub const CURRENT_STABLE: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataVersionModel {
    pub version: i32,
    pub series: String,
}

impl DataVersionModel {
    pub fn new(version: i32, series: impl Into<String>) -> Self {
        Self {
            version,
            series: series.into(),
        }
    }

    pub fn current_main() -> Self {
        Self::new(CURRENT_DATA_VERSION, CURRENT_DATA_SERIES)
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.series == other.series
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackTypeModel {
    ClientResources,
    ServerData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldVersionModel {
    pub id: String,
    pub name: String,
    pub data_version: DataVersionModel,
    pub protocol_version: i32,
    pub resource_pack_version: PackFormat,
    pub data_pack_version: PackFormat,
    pub build_time: JavaDateModel,
    pub stable: bool,
}

impl WorldVersionModel {
    pub fn pack_version(&self, pack_type: PackTypeModel) -> PackFormat {
        match pack_type {
            PackTypeModel::ClientResources => self.resource_pack_version,
            PackTypeModel::ServerData => self.data_pack_version,
        }
    }

    pub fn current_26_1_2() -> Self {
        Self {
            id: CURRENT_VERSION_ID.to_string(),
            name: CURRENT_VERSION_NAME.to_string(),
            data_version: DataVersionModel::current_main(),
            protocol_version: CURRENT_PROTOCOL_VERSION,
            resource_pack_version: PackFormat::current_client_resources(),
            data_pack_version: PackFormat::current_server_data(),
            build_time: JavaDateModel {
                source: CURRENT_BUILD_TIME.to_string(),
                epoch_millis: Some(CURRENT_BUILD_TIME_EPOCH_MILLIS),
            },
            stable: CURRENT_STABLE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaDateModel {
    pub source: String,
    pub epoch_millis: Option<i64>,
}

impl JavaDateModel {
    pub fn parse_zoned(source: &str) -> Result<Self, DetectedVersionError> {
        let epoch_millis = parse_java_zoned_datetime_epoch_millis(source)?;
        Ok(Self {
            source: source.to_string(),
            epoch_millis: Some(epoch_millis),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedVersionOutcome {
    Detected(WorldVersionModel),
    MissingVersionInformation {
        warning: &'static str,
        built_in: WorldVersionModel,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedVersionError {
    pub message: String,
}

impl DetectedVersionError {
    fn corrupt(cause: impl Into<String>) -> Self {
        Self {
            message: format!("Game version information is corrupt: {}", cause.into()),
        }
    }
}

pub fn create_built_in_world_version(id: &str, name: &str, stable: bool) -> WorldVersionModel {
    WorldVersionModel {
        id: id.to_string(),
        name: name.to_string(),
        data_version: DataVersionModel::current_main(),
        protocol_version: CURRENT_PROTOCOL_VERSION,
        resource_pack_version: PackFormat::current_client_resources(),
        data_pack_version: PackFormat::current_server_data(),
        build_time: JavaDateModel {
            source: "new Date()".to_string(),
            epoch_millis: None,
        },
        stable,
    }
}

pub fn create_world_version_from_json(
    json: &str,
) -> Result<WorldVersionModel, DetectedVersionError> {
    let root: Value = serde_json::from_str(json)
        .map_err(|error| DetectedVersionError::corrupt(error.to_string()))?;
    let pack_version = object_field(&root, "pack_version")?;

    Ok(WorldVersionModel {
        id: string_field(&root, "id")?.to_string(),
        name: string_field(&root, "name")?.to_string(),
        data_version: DataVersionModel::new(
            int_field(&root, "world_version")?,
            optional_string_field(&root, "series_id").unwrap_or(CURRENT_DATA_SERIES),
        ),
        protocol_version: int_field(&root, "protocol_version")?,
        resource_pack_version: PackFormat {
            major: u32_field(pack_version, "resource_major")?,
            minor: u32_field(pack_version, "resource_minor")?,
        },
        data_pack_version: PackFormat {
            major: u32_field(pack_version, "data_major")?,
            minor: u32_field(pack_version, "data_minor")?,
        },
        build_time: JavaDateModel::parse_zoned(string_field(&root, "build_time")?)?,
        stable: bool_field(&root, "stable")?,
    })
}

pub fn try_detect_version_from_resource(
    version_json: Option<&str>,
    built_in: WorldVersionModel,
) -> Result<DetectedVersionOutcome, DetectedVersionError> {
    match version_json {
        Some(json) => create_world_version_from_json(json).map(DetectedVersionOutcome::Detected),
        None => Ok(DetectedVersionOutcome::MissingVersionInformation {
            warning: "Missing version information!",
            built_in,
        }),
    }
}

fn object_field<'a>(root: &'a Value, field: &str) -> Result<&'a Value, DetectedVersionError> {
    root.get(field)
        .filter(|value| value.is_object())
        .ok_or_else(|| DetectedVersionError::corrupt(format!("missing object field {field}")))
}

fn string_field<'a>(root: &'a Value, field: &str) -> Result<&'a str, DetectedVersionError> {
    root.get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| DetectedVersionError::corrupt(format!("missing string field {field}")))
}

fn optional_string_field<'a>(root: &'a Value, field: &str) -> Option<&'a str> {
    root.get(field).and_then(Value::as_str)
}

fn int_field(root: &Value, field: &str) -> Result<i32, DetectedVersionError> {
    let value = root
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| DetectedVersionError::corrupt(format!("missing int field {field}")))?;
    i32::try_from(value)
        .map_err(|_| DetectedVersionError::corrupt(format!("int field {field} out of range")))
}

fn u32_field(root: &Value, field: &str) -> Result<u32, DetectedVersionError> {
    let value = int_field(root, field)?;
    u32::try_from(value)
        .map_err(|_| DetectedVersionError::corrupt(format!("int field {field} out of range")))
}

fn bool_field(root: &Value, field: &str) -> Result<bool, DetectedVersionError> {
    root.get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| DetectedVersionError::corrupt(format!("missing boolean field {field}")))
}

fn parse_java_zoned_datetime_epoch_millis(source: &str) -> Result<i64, DetectedVersionError> {
    let (date, time_and_offset) = source
        .split_once('T')
        .ok_or_else(|| DetectedVersionError::corrupt("build_time missing T separator"))?;
    let mut date_parts = date.split('-');
    let year = parse_i32_part(date_parts.next(), "year")?;
    let month = parse_i32_part(date_parts.next(), "month")?;
    let day = parse_i32_part(date_parts.next(), "day")?;
    if date_parts.next().is_some() {
        return Err(DetectedVersionError::corrupt(
            "build_time date has extra fields",
        ));
    }

    let (time, offset_seconds) = split_time_offset(time_and_offset)?;
    let mut time_parts = time.split(':');
    let hour = parse_i32_part(time_parts.next(), "hour")?;
    let minute = parse_i32_part(time_parts.next(), "minute")?;
    let second = parse_i32_part(time_parts.next(), "second")?;
    if time_parts.next().is_some() {
        return Err(DetectedVersionError::corrupt(
            "build_time time has extra fields",
        ));
    }

    validate_datetime_parts(year, month, day, hour, minute, second)?;

    let days = days_from_civil(year, month, day);
    Ok((days * 86_400 + i64::from(hour * 3_600 + minute * 60 + second) - offset_seconds) * 1_000)
}

fn split_time_offset(time_and_offset: &str) -> Result<(&str, i64), DetectedVersionError> {
    if let Some(time) = time_and_offset.strip_suffix('Z') {
        return Ok((time, 0));
    }

    let sign_index = time_and_offset
        .char_indices()
        .skip(1)
        .find_map(|(index, ch)| (ch == '+' || ch == '-').then_some(index))
        .ok_or_else(|| DetectedVersionError::corrupt("build_time missing timezone offset"))?;
    let (time, offset) = time_and_offset.split_at(sign_index);
    let sign = if offset.starts_with('+') { 1 } else { -1 };
    let offset = &offset[1..];
    let (hours, minutes) = offset
        .split_once(':')
        .ok_or_else(|| DetectedVersionError::corrupt("build_time offset missing colon"))?;
    let hours = hours
        .parse::<i64>()
        .map_err(|_| DetectedVersionError::corrupt("invalid build_time offset hour"))?;
    let minutes = minutes
        .parse::<i64>()
        .map_err(|_| DetectedVersionError::corrupt("invalid build_time offset minute"))?;
    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        return Err(DetectedVersionError::corrupt(
            "build_time offset out of range",
        ));
    }

    Ok((time, sign * (hours * 3_600 + minutes * 60)))
}

fn parse_i32_part(part: Option<&str>, name: &str) -> Result<i32, DetectedVersionError> {
    part.ok_or_else(|| DetectedVersionError::corrupt(format!("build_time missing {name}")))?
        .parse()
        .map_err(|_| DetectedVersionError::corrupt(format!("invalid build_time {name}")))
}

fn validate_datetime_parts(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
) -> Result<(), DetectedVersionError> {
    if !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return Err(DetectedVersionError::corrupt(
            "build_time value out of range",
        ));
    }
    Ok(())
}

fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: i32, day: i32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    i64::from(era * 146_097 + day_of_era - 719_468)
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const VANILLA_VERSION_JSON: &str = include_str!("../../decompiled-server-26.1.2/version.json");

    #[test]
    fn world_version_simple_pack_version_switch_matches_java() {
        let version = WorldVersionModel::current_26_1_2();
        assert_eq!(version.id, "26.1.2");
        assert_eq!(version.name, "26.1.2");
        assert_eq!(version.data_version, DataVersionModel::new(4790, "main"));
        assert_eq!(version.protocol_version, 775);
        assert_eq!(
            version.pack_version(PackTypeModel::ClientResources),
            PackFormat {
                major: 84,
                minor: 0
            }
        );
        assert_eq!(
            version.pack_version(PackTypeModel::ServerData),
            PackFormat {
                major: 101,
                minor: 1
            }
        );
        assert_eq!(version.build_time.source, "2026-04-09T10:11:03+00:00");
        assert!(version.stable);
    }

    #[test]
    fn detected_version_create_built_in_matches_java_constants() {
        let version = create_built_in_world_version("dev-id", "Development Version", false);
        assert_eq!(version.id, "dev-id");
        assert_eq!(version.name, "Development Version");
        assert_eq!(version.data_version, DataVersionModel::new(4790, "main"));
        assert_eq!(version.protocol_version, 775);
        assert_eq!(
            version.resource_pack_version,
            PackFormat::current_client_resources()
        );
        assert_eq!(version.data_pack_version, PackFormat::current_server_data());
        assert!(!version.stable);
    }

    #[test]
    fn detected_version_parses_vanilla_version_json_like_java() {
        let version = create_world_version_from_json(VANILLA_VERSION_JSON).unwrap();
        assert_eq!(version, WorldVersionModel::current_26_1_2());
        assert_eq!(version.build_time.epoch_millis, Some(1_775_729_463_000));
    }

    #[test]
    fn detected_version_defaults_missing_series_to_main() {
        let json = r#"{
            "id": "custom",
            "name": "Custom",
            "world_version": 4790,
            "protocol_version": 775,
            "pack_version": {
                "resource_major": 84,
                "resource_minor": 0,
                "data_major": 101,
                "data_minor": 1
            },
            "build_time": "2026-04-09T12:11:03+02:00",
            "stable": false
        }"#;

        let version = create_world_version_from_json(json).unwrap();
        assert_eq!(version.data_version, DataVersionModel::new(4790, "main"));
        assert_eq!(version.build_time.epoch_millis, Some(1_775_729_463_000));
        assert!(!version.stable);
    }

    #[test]
    fn try_detect_version_missing_resource_returns_built_in_with_warning() {
        let built_in = create_built_in_world_version("generated", "Development Version", true);
        assert_eq!(
            try_detect_version_from_resource(None, built_in.clone()).unwrap(),
            DetectedVersionOutcome::MissingVersionInformation {
                warning: "Missing version information!",
                built_in,
            }
        );
    }

    #[test]
    fn detected_version_corrupt_json_uses_java_illegal_state_message() {
        let error = create_world_version_from_json(r#"{"id":"broken"}"#).unwrap_err();
        assert!(error
            .message
            .starts_with("Game version information is corrupt:"));
    }

    #[test]
    fn data_version_compatibility_matches_java_series_equality() {
        assert!(DataVersionModel::new(4790, "main")
            .is_compatible_with(&DataVersionModel::new(5000, "main")));
        assert!(!DataVersionModel::new(4790, "main")
            .is_compatible_with(&DataVersionModel::new(4790, "experimental")));
    }
}
