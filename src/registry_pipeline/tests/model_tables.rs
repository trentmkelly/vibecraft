//! The store is the source of truth for registry ids; the hand-maintained model
//! tables elsewhere in the crate must agree with it.

use std::collections::BTreeSet;

use super::registries;
use crate::registry::Identifier;
use crate::{biome, damage_type, equipment_trim, presentation_data};

fn ids(registry: &str) -> Vec<String> {
    registries()
        .lookup(&Identifier::parse(registry).expect("registry id"))
        .unwrap_or_else(|| panic!("{registry} missing"))
        .elements()
        .iter()
        .map(|element| element.key.to_string())
        .collect()
}

fn set(values: impl IntoIterator<Item = String>) -> BTreeSet<String> {
    values.into_iter().collect()
}

#[test]
fn trim_registries_match_presentation_and_models() {
    let materials = ids("minecraft:trim_material");
    let presentation: Vec<String> = presentation_data::TRIM_MATERIALS
        .iter()
        .map(|material| material.id.to_string())
        .collect();
    let model: Vec<String> = equipment_trim::TRIM_MATERIALS
        .iter()
        .map(|material| material.id.to_string())
        .collect();
    // The model tables list trims in bootstrap order; the synchronised registry is
    // sorted by resource file, so only membership is compared.
    assert_eq!(set(materials.clone()), set(presentation));
    assert_eq!(set(materials), set(model));

    let patterns = ids("minecraft:trim_pattern");
    let presentation: Vec<String> = presentation_data::TRIM_PATTERNS
        .iter()
        .map(|pattern| pattern.id.to_string())
        .collect();
    let model: Vec<String> = equipment_trim::TRIM_PATTERNS
        .iter()
        .map(|pattern| format!("minecraft:{}", pattern.id))
        .collect();
    assert_eq!(set(patterns.clone()), set(presentation));
    assert_eq!(set(patterns), set(model));
}

#[test]
fn instrument_damage_type_and_biome_registries_match_models() {
    assert_eq!(
        set(ids("minecraft:instrument")),
        set(presentation_data::INSTRUMENTS
            .iter()
            .map(|entry| entry.id.to_string()))
    );

    let damage_types = set(ids("minecraft:damage_type"));
    let presentation = set(presentation_data::DAMAGE_TYPES
        .iter()
        .map(|entry| entry.id.to_string()));
    let model = set(damage_type::BUILTIN_DAMAGE_TYPES
        .iter()
        .map(|entry| entry.id.to_string()));
    assert!(damage_types.is_superset(&presentation));
    assert!(damage_types.is_superset(&model));

    assert_eq!(
        set(ids("minecraft:worldgen/biome")),
        set(biome::BUILTIN_BIOMES
            .iter()
            .map(|biome| biome.id.to_string()))
    );
}

#[test]
fn painting_jukebox_and_banner_registries_match_presentation() {
    assert_eq!(
        ids("minecraft:painting_variant"),
        presentation_data::PAINTING_VARIANTS
            .iter()
            .map(|painting| painting.id.to_string())
            .collect::<Vec<_>>()
    );
    let jukebox = set(ids("minecraft:jukebox_song"));
    let presentation = set(presentation_data::JUKEBOX_SONGS
        .iter()
        .map(|song| song.id.to_string()));
    assert!(presentation.is_subset(&jukebox));
    assert_eq!(
        set(ids("minecraft:banner_pattern")),
        set(presentation_data::BANNER_PATTERNS
            .iter()
            .map(|pattern| pattern.id.to_string()))
    );
}
