use crate::registry::{FeatureFlagRegistry, FeatureFlagSet, Identifier};

use super::{
    DataPackMetadata, PackCompatibility, PackFormat, PackFormatRange,
    LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT,
};

pub fn parse_pack_metadata(contents: &str) -> Result<DataPackMetadata, String> {
    let pack_object = object_slice(contents, "pack").ok_or("missing pack metadata")?;
    let description = string_field(pack_object, "description").unwrap_or_default();
    let supported_formats = parse_supported_formats(pack_object)?;
    let requested_features = if let Some(features_object) = object_slice(contents, "features") {
        parse_feature_flags(features_object)?
    } else {
        FeatureFlagSet::empty()
    };

    Ok(DataPackMetadata {
        description,
        supported_formats,
        compatibility: PackCompatibility::for_version(
            supported_formats,
            PackFormat::current_server_data(),
        ),
        requested_features,
    })
}

fn parse_supported_formats(pack_object: &str) -> Result<PackFormatRange, String> {
    if let (Some(min), Some(max)) = (
        pack_format_field(pack_object, "min_format"),
        pack_format_field(pack_object, "max_format"),
    ) {
        if min > max {
            return Err("min_format is greater than max_format".to_string());
        }
        if min.major <= LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT
            && !has_field(pack_object, "supported_formats")
        {
            return Err("supported_formats required for pre-minor pack formats".to_string());
        }
        return Ok(PackFormatRange { min, max });
    }

    if let Some(range) = int_range_field(pack_object, "supported_formats") {
        if range.max.major > LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT {
            return Err("old supported_formats cannot exceed last pre-minor format".to_string());
        }
        return Ok(range);
    }

    if let Some(pack_format) = int_field(pack_object, "pack_format") {
        if pack_format > LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT {
            return Err("new pack formats require min_format and max_format".to_string());
        }
        return Ok(PackFormatRange {
            min: PackFormat {
                major: pack_format,
                minor: 0,
            },
            max: PackFormat {
                major: pack_format,
                minor: 0,
            },
        });
    }

    Err("missing format version information".to_string())
}

fn parse_feature_flags(features_object: &str) -> Result<FeatureFlagSet, String> {
    let names = string_array_field(features_object, "enabled")
        .unwrap_or_default()
        .into_iter()
        .map(|name| Identifier::parse(&name))
        .collect::<Result<Vec<_>, _>>()?;
    FeatureFlagRegistry::main_26_1_2()
        .from_names(&names)
        .map_err(|unknown| format!("unknown feature flags: {unknown:?}"))
}

fn object_slice<'a>(contents: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{field}\"");
    let key_index = contents.find(&key)?;
    let start = contents[key_index + key.len()..].find('{')? + key_index + key.len();
    let end = matching_delimiter(contents, start, '{', '}')?;
    Some(&contents[start + 1..end])
}

fn has_field(contents: &str, field: &str) -> bool {
    contents.contains(&format!("\"{field}\""))
}

fn string_field(contents: &str, field: &str) -> Option<String> {
    let raw = field_value(contents, field)?;
    if raw.trim_start().starts_with('"') {
        parse_json_string(raw.trim_start()).map(|(value, _)| value)
    } else {
        Some(raw.trim().to_string())
    }
}

fn int_field(contents: &str, field: &str) -> Option<u32> {
    let raw = field_value(contents, field)?;
    parse_u32_prefix(raw.trim_start())
}

fn int_range_field(contents: &str, field: &str) -> Option<PackFormatRange> {
    let raw = field_value(contents, field)?.trim_start();
    if raw.starts_with('[') {
        let end = matching_delimiter(raw, 0, '[', ']')?;
        let values = raw[1..end]
            .split(',')
            .filter_map(|part| parse_u32_prefix(part.trim()))
            .collect::<Vec<_>>();
        match values.as_slice() {
            [one] => Some(PackFormatRange {
                min: PackFormat {
                    major: *one,
                    minor: 0,
                },
                max: PackFormat {
                    major: *one,
                    minor: 0,
                },
            }),
            [min, max] => Some(PackFormatRange {
                min: PackFormat {
                    major: *min,
                    minor: 0,
                },
                max: PackFormat {
                    major: *max,
                    minor: 0,
                },
            }),
            _ => None,
        }
    } else {
        int_field(contents, field).map(|value| PackFormatRange {
            min: PackFormat {
                major: value,
                minor: 0,
            },
            max: PackFormat {
                major: value,
                minor: 0,
            },
        })
    }
}

fn pack_format_field(contents: &str, field: &str) -> Option<PackFormat> {
    let raw = field_value(contents, field)?.trim_start();
    if raw.starts_with('[') {
        let end = matching_delimiter(raw, 0, '[', ']')?;
        let values = raw[1..end]
            .split(',')
            .filter_map(|part| parse_u32_prefix(part.trim()))
            .collect::<Vec<_>>();
        Some(PackFormat {
            major: *values.first()?,
            minor: *values.get(1).unwrap_or(&0),
        })
    } else {
        int_field(contents, field).map(|major| PackFormat { major, minor: 0 })
    }
}

fn string_array_field(contents: &str, field: &str) -> Option<Vec<String>> {
    let raw = field_value(contents, field)?.trim_start();
    if !raw.starts_with('[') {
        return None;
    }
    let end = matching_delimiter(raw, 0, '[', ']')?;
    let mut values = Vec::new();
    let mut rest = raw[1..end].trim_start();
    while !rest.is_empty() {
        if let Some((value, remaining)) = parse_json_string(rest) {
            values.push(value);
            rest = remaining.trim_start();
            if rest.starts_with(',') {
                rest = rest[1..].trim_start();
            } else {
                break;
            }
        } else {
            return None;
        }
    }
    Some(values)
}

fn field_value<'a>(contents: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{field}\"");
    let key_index = contents.find(&key)?;
    let after_key = &contents[key_index + key.len()..];
    let colon = after_key.find(':')?;
    Some(&after_key[colon + 1..])
}

fn matching_delimiter(contents: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in contents[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

fn parse_json_string(contents: &str) -> Option<(String, &str)> {
    let mut chars = contents.char_indices();
    if chars.next()?.1 != '"' {
        return None;
    }
    let mut value = String::new();
    let mut escaped = false;
    for (index, ch) in chars {
        if escaped {
            value.push(match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some((value, &contents[index + 1..]));
        } else {
            value.push(ch);
        }
    }
    None
}

fn parse_u32_prefix(contents: &str) -> Option<u32> {
    let digits = contents
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

