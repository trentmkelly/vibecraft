//! Vanilla entity types suggested by the `summonable_entities` command provider.
//!
//! Java's `SuggestionProviders.SUMMONABLE_ENTITIES` walks the entity registry in
//! registry order, keeping types whose required features are enabled and whose
//! `EntityType.canSummon()` is true. In 26.1.2 all registrations use the vanilla
//! feature set; only `player` and `fishing_bobber` call `Builder.noSummon()`.
//! Registry ids come from `vanilla-data/reports/registries_26_1_2.json` in
//! protocol order.

use std::sync::OnceLock;

/// Namespaced entity ids that Java's vanilla-enabled summonable-entities provider
/// can suggest. The `String`s are initialized once and shared across calls.
pub(super) fn summonable_entities() -> &'static [String] {
    static ENTITIES: OnceLock<Vec<String>> = OnceLock::new();
    ENTITIES.get_or_init(|| {
        let mut entities = super::registry_entries("minecraft:entity_type");
        // EntityType.Builder.noSummon() is used only for PLAYER and
        // FISHING_BOBBER in the authoritative 26.1.2 EntityType.java source.
        entities.retain(|id| id != "minecraft:player" && id != "minecraft:fishing_bobber");
        entities
    })
}

#[cfg(test)]
mod tests {
    use super::summonable_entities;

    #[test]
    fn list_matches_vanilla_enabled_summonable_registry_entries() {
        let entities = summonable_entities();

        // The 26.1.2 entity registry has 157 entries. Java marks exactly the
        // player and fishing bobber as non-summonable; all others use
        // FeatureFlags.VANILLA_SET and are enabled by the vanilla feature set.
        let mut expected = super::super::registry_entries("minecraft:entity_type");
        expected.retain(|id| id != "minecraft:player" && id != "minecraft:fishing_bobber");
        assert_eq!(entities, &expected);
        assert_eq!(entities.len(), 155);
        assert!(!entities.iter().any(|id| id == "minecraft:player"));
        assert!(!entities.iter().any(|id| id == "minecraft:fishing_bobber"));
        assert!(entities.iter().all(|id| id.starts_with("minecraft:")));
        assert!(entities.iter().any(|id| id == "minecraft:happy_ghast"));
        assert!(entities.iter().any(|id| id == "minecraft:copper_golem"));
    }

    #[test]
    fn list_is_cached_and_preserves_registry_order() {
        let first = summonable_entities();
        let second = summonable_entities();

        assert!(std::ptr::eq(first, second));
        assert_eq!(
            first.first().map(String::as_str),
            Some("minecraft:acacia_boat")
        );
        assert_eq!(
            first.last().map(String::as_str),
            Some("minecraft:zombified_piglin")
        );
    }
}
