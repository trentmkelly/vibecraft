//! Vanilla sound-event identifiers used by the `minecraft:available_sounds`
//! command suggestion provider.
//!
//! `CommandSourceStack.getAvailableSounds` streams the entries in
//! `BuiltInRegistries.SOUND_EVENT`. The checked-in registry report records the
//! same entries and their protocol IDs. The shared provider registry loader
//! sorts those IDs to preserve registry iteration order independently of JSON
//! object key ordering.

use std::sync::OnceLock;

static AVAILABLE_SOUNDS: OnceLock<Vec<String>> = OnceLock::new();

/// Returns vanilla sound event identifiers in built-in registry order.
///
/// This list matches `CommandSourceStack.getAvailableSounds`: entries are
/// sound-event locations, including their `minecraft:` namespace. The
/// registry report is pinned to the server version this port targets.
pub(super) fn available_sounds() -> &'static [String] {
    AVAILABLE_SOUNDS.get_or_init(|| super::registry_entries("minecraft:sound_event"))
}

#[cfg(test)]
mod tests {
    use super::available_sounds;

    #[test]
    fn available_sounds_match_the_vendored_sound_event_registry() {
        let sounds = available_sounds();
        assert_eq!(sounds.len(), 1902);
        assert_eq!(
            sounds.first().map(String::as_str),
            Some("minecraft:entity.allay.ambient_with_item")
        );
        assert_eq!(
            sounds.last().map(String::as_str),
            Some("minecraft:item.nautilus_saddle_equip")
        );
        assert!(sounds.iter().all(|sound| sound.starts_with("minecraft:")));
    }

    #[test]
    fn available_sounds_are_unique_and_follow_registry_protocol_order() {
        let sounds = available_sounds();
        let unique = sounds.iter().collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), sounds.len());

        // These surrounding entries pin the ordering to SOUND_EVENT protocol
        // IDs rather than lexicographic JSON key order.
        let allay_ambient = sounds
            .iter()
            .position(|sound| sound == "minecraft:entity.allay.ambient_with_item");
        let allay_death = sounds
            .iter()
            .position(|sound| sound == "minecraft:entity.allay.death");
        assert!(allay_ambient < allay_death);
        assert_eq!(allay_ambient, Some(0));
        assert_eq!(allay_death, Some(2));
    }
}
