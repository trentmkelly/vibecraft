//! Port of `net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix`.

use crate::datafix::dynamic::parse_java_int;
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::options_key_lwjgl3_fix_keys::KEY_NAMES;

/// `OptionsKeyLwjgl3Fix.KEY_UNKNOWN`.
pub const KEY_UNKNOWN: &str = "key.unknown";

/// `new OptionsKeyLwjgl3Fix(schema, changesType)`: LWJGL2 key codes become
/// LWJGL3 key names.
pub fn fix() -> Fix {
    Fix::everywhere("OptionsKeyLwjgl3Fix", Target::Type(r::OPTIONS), |options| {
        let Tag::Compound(entries) = options else {
            return;
        };
        for (key, value) in entries {
            if !key.starts_with("key_") {
                continue;
            }
            let text = match value {
                Tag::String(text) => text.as_str(),
                _ => "",
            };
            // `Integer.parseInt` throws for anything else; those values are kept.
            let Some(old_value) = parse_java_int(text) else {
                continue;
            };
            *value = Tag::String(key_name(old_value));
        }
    })
}

fn key_name(old_value: i32) -> String {
    if old_value < 0 {
        let button = old_value + 100;
        match button {
            0 => "key.mouse.left".to_string(),
            1 => "key.mouse.right".to_string(),
            2 => "key.mouse.middle".to_string(),
            _ => format!("key.mouse.{}", button + 1),
        }
    } else {
        match KEY_NAMES.binary_search_by_key(&old_value, |(code, _)| *code) {
            Ok(index) => KEY_NAMES[index].1.to_string(),
            Err(_) => KEY_UNKNOWN.to_string(),
        }
    }
}
