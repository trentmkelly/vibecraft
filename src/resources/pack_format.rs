//! `PackFormat.IntermediaryFormat`: decoding and validation of the format fields of
//! `pack.mcmeta` (`pack` and `overlays` sections) with Java's exact error messages.

use serde_json::{Map, Value};

use super::{PackFormat, PackFormatRange};

/// `Integer.MAX_VALUE`, the "any minor" sentinel of `PackFormat.TOP_CODEC`.
pub const TOP_MINOR: u32 = i32::MAX as u32;

/// Reads a `NON_NEGATIVE_INT`.
fn non_negative_int(value: &Value) -> Result<u32, String> {
    let number = value
        .as_i64()
        .ok_or_else(|| format!("Not a number: {value}"))?;
    if !(0..=i64::from(i32::MAX)).contains(&number) {
        return Err(format!("Value must be non-negative: {number}"));
    }
    Ok(number as u32)
}

/// Reads a plain `Codec.INT` (any 32-bit integer).
fn int(value: &Value) -> Result<i32, String> {
    value
        .as_i64()
        .and_then(|number| i32::try_from(number).ok())
        .ok_or_else(|| format!("Not a number: {value}"))
}

/// `PackFormat.fullCodec(defaultMinor)`: an int or a list of 1..=256 ints.
fn full_format(value: &Value, default_minor: u32) -> Result<PackFormat, String> {
    match value {
        Value::Array(list) => {
            if list.is_empty() || list.len() > 256 {
                return Err(format!(
                    "List length ({}) must be between 1 and 256",
                    list.len()
                ));
            }
            let major = non_negative_int(&list[0])?;
            let minor = match list.get(1) {
                Some(minor) => non_negative_int(minor)?,
                None => default_minor,
            };
            list.iter()
                .try_for_each(|entry| non_negative_int(entry).map(drop))?;
            Ok(PackFormat { major, minor })
        }
        other => Ok(PackFormat {
            major: non_negative_int(other)?,
            minor: default_minor,
        }),
    }
}

/// `InclusiveRange.codec(Codec.INT)`: an int, `[min, max]` or
/// `{min_inclusive, max_inclusive}`.
fn int_range(value: &Value) -> Result<(u32, u32), String> {
    let (min, max) = match value {
        Value::Array(list) => match list.as_slice() {
            [min, max] => (int(min)?, int(max)?),
            _ => return Err(format!("Input is not a list of 2 elements: {value}")),
        },
        Value::Object(map) => {
            let field = |name: &str| {
                map.get(name)
                    .ok_or_else(|| format!("No key {name} in MapLike[{value}]"))
                    .and_then(int)
            };
            (field("min_inclusive")?, field("max_inclusive")?)
        }
        other => {
            let single = int(other)?;
            (single, single)
        }
    };
    if min > max {
        return Err("min_inclusive must be less than or equal to max_inclusive".to_string());
    }
    // Pack majors are compared as unsigned values; negatives never satisfy the
    // format checks below and are clamped to zero.
    Ok((min.max(0) as u32, max.max(0) as u32))
}

/// `PackFormat.IntermediaryFormat`.
#[derive(Debug, Clone, Default)]
pub struct IntermediaryFormat {
    min: Option<PackFormat>,
    max: Option<PackFormat>,
    format: Option<u32>,
    supported: Option<(u32, u32)>,
}

impl IntermediaryFormat {
    fn optional<T>(
        map: &Map<String, Value>,
        name: &str,
        parse: impl FnOnce(&Value) -> Result<T, String>,
    ) -> Result<Option<T>, String> {
        map.get(name).map(parse).transpose()
    }

    /// `IntermediaryFormat.PACK_CODEC`.
    pub fn from_pack(map: &Map<String, Value>) -> Result<Self, String> {
        Ok(Self {
            min: Self::optional(map, "min_format", |v| full_format(v, 0))?,
            max: Self::optional(map, "max_format", |v| full_format(v, TOP_MINOR))?,
            format: Self::optional(map, "pack_format", |v| int(v).map(|f| f.max(0) as u32))?,
            supported: Self::optional(map, "supported_formats", int_range)?,
        })
    }

    /// `IntermediaryFormat.OVERLAY_CODEC`.
    pub fn from_overlay(map: &Map<String, Value>) -> Result<Self, String> {
        let min = Self::optional(map, "min_format", |v| full_format(v, 0))?;
        Ok(Self {
            min,
            max: Self::optional(map, "max_format", |v| full_format(v, TOP_MINOR))?,
            format: min.map(|format| format.major),
            supported: Self::optional(map, "formats", int_range)?,
        })
    }

    /// True for an overlay entry that declares no format information at all.
    pub fn is_empty(&self) -> bool {
        self.min.is_none() && self.max.is_none() && self.supported.is_none()
    }

    /// `effectiveMinMajorVersion`.
    pub fn effective_min_major_version(&self) -> u32 {
        match (self.min, self.supported) {
            (Some(min), Some((supported_min, _))) => min.major.min(supported_min),
            (Some(min), None) => min.major,
            (None, Some((supported_min, _))) => supported_min,
            (None, None) => i32::MAX as u32,
        }
    }

    /// `IntermediaryFormat.validate`.
    pub fn validate(
        &self,
        last_pre_minor: u32,
        has_pack_format_field: bool,
        require_old_field: bool,
        context: &str,
        old_field_name: &str,
    ) -> Result<PackFormatRange, String> {
        if self.min.is_some() != self.max.is_some() {
            return Err(format!(
                "{context} missing field, must declare both min_format and max_format"
            ));
        }
        if require_old_field && self.supported.is_none() {
            return Err(format!(
                "{context} missing required field {old_field_name}, must be present in all overlays for any overlays to work across game versions"
            ));
        }
        if self.min.is_some() {
            return self.validate_new_format(
                last_pre_minor,
                has_pack_format_field,
                require_old_field,
                context,
                old_field_name,
            );
        }
        if self.supported.is_some() {
            return self.validate_old_format(
                last_pre_minor,
                has_pack_format_field,
                context,
                old_field_name,
            );
        }
        if let (true, Some(main)) = (has_pack_format_field, self.format) {
            if main > last_pre_minor {
                return Err(format!(
                    "{context} declares support for version newer than {last_pre_minor}, but is missing mandatory fields min_format and max_format"
                ));
            }
            let format = PackFormat {
                major: main,
                minor: 0,
            };
            return Ok(PackFormatRange {
                min: format,
                max: format,
            });
        }
        Err(format!(
            "{context} could not be parsed, missing format version information"
        ))
    }

    fn validate_new_format(
        &self,
        last_pre_minor: u32,
        has_pack_format_field: bool,
        require_old_field: bool,
        context: &str,
        old_field_name: &str,
    ) -> Result<PackFormatRange, String> {
        let (Some(min), Some(max)) = (self.min, self.max) else {
            return Err(format!("{context} could not be parsed"));
        };
        if min > max {
            return Err(format!(
                "{context} min_format ({min}) is greater than max_format ({max})"
            ));
        }
        if min.major > last_pre_minor && !require_old_field {
            if self.supported.is_some() {
                return Err(format!(
                    "{context} key {old_field_name} is deprecated starting from pack format {}. Remove {old_field_name} from your pack.mcmeta.",
                    last_pre_minor + 1
                ));
            }
            if let (true, Some(main)) = (has_pack_format_field, self.format) {
                validate_pack_format_for_range(main, min.major, max.major)?;
            }
        } else {
            let Some((old_min, old_max)) = self.supported else {
                return Err(format!(
                    "{context} declares support for format {major}, but game versions supporting formats 17 to {last_pre_minor} require a {old_field_name} field. Add \"{old_field_name}\": [{major}, {last_pre_minor}] or require a version greater or equal to {next}.0.",
                    major = min.major,
                    next = last_pre_minor + 1
                ));
            };
            if old_min != min.major {
                return Err(format!(
                    "{context} version declaration mismatch between {old_field_name} (from {old_min}) and min_format ({min})"
                ));
            }
            if old_max != max.major && old_max != last_pre_minor {
                return Err(format!(
                    "{context} version declaration mismatch between {old_field_name} (up to {old_max}) and max_format ({max})"
                ));
            }
            if has_pack_format_field {
                let Some(main) = self.format else {
                    return Err(format!(
                        "{context} declares support for formats up to {last_pre_minor}, but game versions supporting formats 17 to {last_pre_minor} require a pack_format field. Add \"pack_format\": {} or require a version greater or equal to {}.0.",
                        min.major,
                        last_pre_minor + 1
                    ));
                };
                validate_pack_format_for_range(main, min.major, max.major)?;
            }
        }
        Ok(PackFormatRange { min, max })
    }

    fn validate_old_format(
        &self,
        last_pre_minor: u32,
        has_pack_format_field: bool,
        context: &str,
        _old_field_name: &str,
    ) -> Result<PackFormatRange, String> {
        let Some((min, max)) = self.supported else {
            return Err(format!("{context} could not be parsed"));
        };
        if max > last_pre_minor {
            return Err(format!(
                "{context} declares support for version newer than {last_pre_minor}, but is missing mandatory fields min_format and max_format"
            ));
        }
        if has_pack_format_field {
            let Some(main) = self.format else {
                return Err(format!(
                    "{context} declares support for formats up to {last_pre_minor}, but game versions supporting formats 17 to {last_pre_minor} require a pack_format field. Add \"pack_format\": {min} or require a version greater or equal to {}.0.",
                    last_pre_minor + 1
                ));
            };
            validate_pack_format_for_range(main, min, max)?;
        }
        Ok(PackFormatRange {
            min: PackFormat {
                major: min,
                minor: 0,
            },
            max: PackFormat {
                major: max,
                minor: 0,
            },
        })
    }
}

/// `validatePackFormatForRange`.
fn validate_pack_format_for_range(main: u32, min: u32, max: u32) -> Result<(), String> {
    if main < min || main > max {
        Err(format!(
            "Pack declared support for versions {min} to {max} but declared main format is {main}"
        ))
    } else if main < 15 {
        Err("Multi-version packs cannot support minimum version of less than 15, since this will leave versions in range unable to load pack.".to_string())
    } else {
        Ok(())
    }
}
