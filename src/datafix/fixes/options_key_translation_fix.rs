//! Port of `net.minecraft.util.datafix.fixes.OptionsKeyTranslationFix`.

use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new OptionsKeyTranslationFix(schema, changesType)`: `key.x` bindings become
/// `key.keyboard.x`.
pub fn fix() -> Fix {
    Fix::everywhere(
        "OptionsKeyTranslationFix",
        Target::Type(r::OPTIONS),
        |options| {
            let Tag::Compound(entries) = options else {
                return;
            };
            for (key, value) in entries {
                if !key.starts_with("key_") {
                    continue;
                }
                let old_value = match value {
                    Tag::String(text) => text.as_str(),
                    _ => "",
                };
                if old_value.starts_with("key.mouse") || old_value.starts_with("scancode.") {
                    continue;
                }
                // Java's `substring("key.".length())` throws for shorter values.
                if let Some(rest) = old_value.get("key.".len()..) {
                    *value = Tag::String(format!("key.keyboard.{rest}"));
                }
            }
        },
    )
}
