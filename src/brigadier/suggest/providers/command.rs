//! Static suggestion lambdas attached to individual vanilla commands.
//!
//! These entries mirror the per-argument providers in the decompiled
//! `net.minecraft.server.commands` sources. Keep them keyed by the parsed
//! command path: the same Brigadier argument type can use different
//! suggestions in different commands.

use crate::brigadier::suggest::SuggestionsBuilder;

/// Runs the static per-command provider for `path` when one is registered.
///
/// The path contains literal names and argument node names from the parsed
/// command tree. `true` means this provider owns the argument, including when
/// its values do not match the current prefix.
pub(super) fn list_suggestions(path: &[&str], builder: &mut SuggestionsBuilder<'_>) -> bool {
    let values: &[&str] = match path {
        // TickCommand.register: FloatArgumentType.rate
        ["tick", "rate", "rate"] => &["20"],
        // TickCommand.register: TimeArgument.time under `step`
        ["tick", "step", "time"] => &["1t", "1s"],
        // TickCommand.register: TimeArgument.time under `sprint`
        ["tick", "sprint", "time"] => &["60s", "1d", "3d"],
        _ => return false,
    };

    super::suggest_matching(values.iter().copied(), builder);
    true
}

#[cfg(test)]
mod tests {
    use super::list_suggestions;
    use crate::brigadier::suggest::SuggestionsBuilder;

    fn suggestions(path: &[&str], remaining: &str) -> Option<Vec<String>> {
        let input: Vec<u16> = remaining.encode_utf16().collect();
        let mut builder = SuggestionsBuilder::new(&input, 0);
        if !list_suggestions(path, &mut builder) {
            return None;
        }
        Some(
            builder
                .result
                .into_iter()
                .map(|suggestion| suggestion.text)
                .collect(),
        )
    }

    #[test]
    fn tick_rate_uses_the_java_default_rate_suggestion() {
        assert_eq!(
            suggestions(&["tick", "rate", "rate"], ""),
            Some(vec!["20".into()])
        );
        assert_eq!(
            suggestions(&["tick", "rate", "rate"], "2"),
            Some(vec!["20".into()])
        );
        assert_eq!(suggestions(&["tick", "rate", "rate"], "3"), Some(vec![]));
    }

    #[test]
    fn tick_step_and_sprint_suggest_the_java_time_values() {
        assert_eq!(
            suggestions(&["tick", "step", "time"], "1"),
            Some(vec!["1t".into(), "1s".into()])
        );
        assert_eq!(
            suggestions(&["tick", "sprint", "time"], ""),
            Some(vec!["60s".into(), "1d".into(), "3d".into()])
        );
        assert_eq!(suggestions(&["tick", "sprint", "time"], "1d"), Some(vec![]));
    }

    #[test]
    fn unrelated_argument_paths_are_not_claimed() {
        assert_eq!(suggestions(&["time", "set", "time"], ""), None);
        assert_eq!(suggestions(&["tick", "step", "stop"], ""), None);
    }
}
