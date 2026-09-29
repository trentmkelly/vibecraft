//! Server-side `net.minecraft.locale.Language` plus the shared `TranslatableContents`
//! placeholder formatting.
//!
//! The `en_us` language file is vendored under `vanilla-data/assets/minecraft/lang/` (taken from
//! the server jar) and embedded at compile time, so translated command feedback never depends on
//! the optional Java decompilation. Every consumer that formats `%s` / `%n$s` / `%%` templates
//! (`TranslatableContents.decomposeTemplate`) goes through [`decompose_template`].

use std::collections::HashMap;
use std::sync::OnceLock;

/// The vendored `en_us.json` language file.
const EN_US_JSON: &str = include_str!("../vanilla-data/assets/minecraft/lang/en_us.json");

/// The vendored `deprecated.json` (`DeprecatedTranslationsInfo`) applied on top of `en_us`.
#[cfg(all(test, vibecraft_has_decompiled_sources))]
pub const DEPRECATED_JSON: &str = include_str!("../vanilla-data/assets/minecraft/lang/deprecated.json");

/// The raw vendored `en_us.json` text.
#[cfg(all(test, vibecraft_has_decompiled_sources))]
pub const EN_US_RAW: &str = EN_US_JSON;

/// The `en_us` translations (Java `Language.getInstance()` on a dedicated server).
pub fn en_us() -> &'static HashMap<String, String> {
    static LANGUAGE: OnceLock<HashMap<String, String>> = OnceLock::new();
    LANGUAGE.get_or_init(|| {
        // The vendored file is validated by `embedded_en_us_resolves_without_the_decompilation`.
        serde_json::from_str(EN_US_JSON).unwrap_or_default()
    })
}

/// `Language.getOrDefault(key)`: the translation, or the key itself when missing.
pub fn get_or_default(key: &str) -> &str {
    en_us().get(key).map_or(key, String::as_str)
}

/// Java `TranslatableContents.decomposeTemplate`: splits `template` into literal and argument
/// parts, resolving each `%s` / `%n$s` through `argument` (0-based index) and `%%` to `%`.
///
/// Returns `Err(())` for malformed templates or missing arguments (Java throws
/// `TranslatableFormatException`, after which callers render the raw template).
pub fn decompose_template(
    template: &str,
    mut argument: impl FnMut(usize) -> Option<String>,
) -> Result<Vec<String>, ()> {
    let bytes = template.as_bytes();
    let mut parts = Vec::new();
    let mut current = 0;
    let mut next_index = 0usize;
    while let Some(relative) = template[current..].find('%') {
        let start = current + relative;
        if start > current {
            parts.push(template[current..start].to_string());
        }
        let after_percent = start + 1;
        if after_percent >= template.len() {
            return Err(());
        }
        if bytes[after_percent] == b'%' {
            parts.push("%".to_string());
            current = after_percent + 1;
            continue;
        }
        let mut scan = after_percent;
        while scan < template.len() && bytes[scan].is_ascii_digit() {
            scan += 1;
        }
        let index = if scan > after_percent {
            if scan >= template.len() || bytes[scan] != b'$' {
                return Err(());
            }
            let parsed = template[after_percent..scan].parse::<usize>().map_err(|_| ())?;
            scan += 1;
            parsed.checked_sub(1).ok_or(())?
        } else {
            next_index += 1;
            next_index - 1
        };
        if scan >= template.len() || bytes[scan] != b's' {
            return Err(());
        }
        parts.push(argument(index).ok_or(())?);
        current = scan + 1;
    }
    if current < template.len() {
        parts.push(template[current..].to_string());
    }
    Ok(parts)
}

/// Formats `template` with plain-string arguments; malformed templates render unchanged, like
/// `TranslatableContents.visit`.
pub fn format_template(template: &str, args: &[String]) -> String {
    match decompose_template(template, |index| args.get(index).cloned()) {
        Ok(parts) => parts.concat(),
        Err(()) => template.to_string(),
    }
}

/// `Component.translatable(key, args).getString()` against the `en_us` language.
pub fn translate(key: &str, args: &[String]) -> String {
    format_template(get_or_default(key), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> [String; 2] {
        ["a".to_string(), "b".to_string()]
    }

    #[test]
    fn placeholders_follow_translatable_contents() {
        assert_eq!(format_template("[%s: %s]", &args()), "[a: b]");
        assert_eq!(format_template("%2$s then %1$s", &args()), "b then a");
        assert_eq!(format_template("100%% sure", &args()), "100% sure");
    }

    #[test]
    fn malformed_templates_and_missing_arguments_render_raw() {
        assert_eq!(format_template("50%", &args()), "50%");
        assert_eq!(format_template("%d", &args()), "%d");
        assert_eq!(format_template("%3$s", &args()), "%3$s");
    }

    #[test]
    fn embedded_en_us_resolves_without_the_decompilation() {
        assert_eq!(get_or_default("commands.seed.success"), "Seed: %s");
        assert_eq!(get_or_default("no.such.key"), "no.such.key");
        assert!(en_us().contains_key("command.unknown.command"));
        assert_eq!(
            translate("commands.list.players", &["1".into(), "20".into(), "Steve".into()]),
            "There are 1 of a max of 20 players online: Steve"
        );
        assert_eq!(translate("no.such.key", &[]), "no.such.key");
    }
}
