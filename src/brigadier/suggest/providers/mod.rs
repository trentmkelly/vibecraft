//! Server implementations of the providers retained in the synchronized graph.
//!
//! Named providers survive command-tree serialization. Anonymous lambdas become
//! `ask_server` on the wire, so the server must recover their implementation from
//! the parsed command path instead of treating every such node as ASK_SERVER.

mod command;
mod entities;
mod sounds;

use super::argument::{matches_sub_str, suggest_matching};
use super::SuggestionsBuilder;

/// Reads the pinned built-in registry in Java registry iteration order.
fn registry_entries(registry: &str) -> Vec<String> {
    let entries = (|| {
        let key = crate::registry::Identifier::parse(registry).map_err(|e| e.to_string())?;
        let builtins = crate::registry_pipeline::builtin::BuiltinRegistries::vanilla()?;
        let values = builtins
            .get(&key)
            .ok_or_else(|| format!("missing built-in registry {registry}"))?;
        Ok::<_, String>(values.elements().iter().map(ToString::to_string).collect())
    })();
    // Follow CommandGraph::vanilla's embedded-data failure policy: a corrupt
    // build yields no completions instead of crashing a live packet handler.
    entries.unwrap_or_else(|error| {
        eprintln!("command suggestion registry unavailable: {error}");
        Vec::new()
    })
}

/// `SharedSuggestionProvider.filterResources`: namespace-free input matches
/// either the namespace or the path; qualified input matches the full id.
fn resource_matches(remaining: &str, identifier: &str) -> bool {
    if remaining.contains(':') {
        matches_sub_str(remaining, identifier)
    } else {
        let (namespace, path) = identifier
            .split_once(':')
            .unwrap_or(("minecraft", identifier));
        matches_sub_str(remaining, namespace) || matches_sub_str(remaining, path)
    }
}

pub(super) fn list_suggestions(
    provider: &str,
    path: &[&str],
    builder: &mut SuggestionsBuilder<'_>,
) {
    let remaining = builder.remaining_lower_case();
    match provider {
        "minecraft:available_sounds" => {
            for identifier in sounds::available_sounds() {
                if resource_matches(&remaining, identifier) {
                    builder.suggest(identifier);
                }
            }
        }
        "minecraft:summonable_entities" => {
            for identifier in entities::summonable_entities() {
                if resource_matches(&remaining, identifier) {
                    // EntityType.getDescription -> Util.makeDescriptionId.
                    let description =
                        format!("entity.{}", identifier.replace([':', '/'], "."));
                    builder.suggest_with_tooltip(identifier, Some(description));
                }
            }
        }
        "minecraft:ask_server" => {
            command::list_suggestions(path, builder);
        }
        // SuggestionProviders.getProvider falls back to ASK_SERVER, whose
        // CommandSourceStack implementation returns Suggestions.empty().
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::resource_matches;

    #[test]
    fn resource_matching_uses_java_namespace_and_segment_rules() {
        assert!(resource_matches("pig", "minecraft:pig"));
        assert!(resource_matches("mine", "minecraft:pig"));
        assert!(resource_matches("ambient", "minecraft:entity.pig.ambient"));
        assert!(resource_matches(
            "minecraft:entity.pig",
            "minecraft:entity.pig.ambient"
        ));
        assert!(!resource_matches("craft:pig", "minecraft:pig"));
        assert!(!resource_matches("ig", "minecraft:pig"));
        assert!(!resource_matches("minecraft:p", "other:pig"));
    }
}
