//! Field-level expectations for individual vanilla registry elements.

use super::{compound, entry_map, field, string};
use crate::storage::nbt::Tag;

fn assert_sound_fields(set: &Tag, names: &[&str]) {
    for name in names {
        assert!(
            matches!(field(set, name), Some(Tag::String(sound)) if sound.starts_with("minecraft:")),
            "{name} must be a sound event id"
        );
    }
}

#[test]
fn sound_variants_match_the_nested_26_1_2_codecs() {
    // CowSoundVariant is flat; the others nest adult/baby sound sets.
    assert_sound_fields(
        &entry_map("minecraft:cow_sound_variant")["minecraft:classic"],
        &["ambient_sound", "hurt_sound", "death_sound", "step_sound"],
    );
    let nested = [
        (
            "minecraft:chicken_sound_variant",
            &["ambient_sound", "hurt_sound", "death_sound", "step_sound"][..],
        ),
        (
            "minecraft:pig_sound_variant",
            &[
                "ambient_sound",
                "hurt_sound",
                "death_sound",
                "step_sound",
                "eat_sound",
            ][..],
        ),
        (
            "minecraft:cat_sound_variant",
            &[
                "ambient_sound",
                "stray_ambient_sound",
                "hiss_sound",
                "hurt_sound",
                "death_sound",
                "eat_sound",
                "beg_for_food_sound",
                "purr_sound",
                "purreow_sound",
            ][..],
        ),
        (
            "minecraft:wolf_sound_variant",
            &[
                "ambient_sound",
                "death_sound",
                "growl_sound",
                "hurt_sound",
                "pant_sound",
                "whine_sound",
                "step_sound",
            ][..],
        ),
    ];
    for (registry, names) in nested {
        let classic = &entry_map(registry)["minecraft:classic"];
        assert_sound_fields(compound(classic, "adult_sounds"), names);
        assert_sound_fields(compound(classic, "baby_sounds"), names);
    }
}

#[test]
fn trim_materials_match_vanilla_colors_and_overrides() {
    // (id, description color as serialized by TextColor `#%06X`, override_armor_assets)
    type TrimExpectation = (
        &'static str,
        &'static str,
        &'static [(&'static str, &'static str)],
    );
    let expected: &[TrimExpectation] = &[
        ("quartz", "#E3D4C4", &[]),
        ("iron", "#ECECEC", &[("minecraft:iron", "iron_darker")]),
        (
            "netherite",
            "#625859",
            &[("minecraft:netherite", "netherite_darker")],
        ),
        ("redstone", "#971607", &[]),
        (
            "copper",
            "#B4684D",
            &[("minecraft:copper", "copper_darker")],
        ),
        ("gold", "#DEB12D", &[("minecraft:gold", "gold_darker")]),
        ("emerald", "#11A036", &[]),
        (
            "diamond",
            "#6EECD2",
            &[("minecraft:diamond", "diamond_darker")],
        ),
        ("lapis", "#416E97", &[]),
        ("amethyst", "#9A5CC6", &[]),
        ("resin", "#FC7812", &[]),
    ];
    let materials = entry_map("minecraft:trim_material");
    assert_eq!(materials.len(), expected.len());
    for (id, color, overrides) in expected {
        let material = &materials[&format!("minecraft:{id}")];
        assert_eq!(field(material, "asset_name"), Some(&string(id)), "{id}");
        let description = compound(material, "description");
        assert_eq!(
            field(description, "translate"),
            Some(&string(&format!("trim_material.minecraft.{id}")))
        );
        assert_eq!(field(description, "color"), Some(&string(color)), "{id}");
        if overrides.is_empty() {
            assert!(field(material, "override_armor_assets").is_none(), "{id}");
        } else {
            let group = compound(material, "override_armor_assets");
            for (asset, suffix) in *overrides {
                assert_eq!(field(group, asset), Some(&string(suffix)), "{id} {asset}");
            }
        }
    }
}

#[test]
fn trim_pattern_banner_pattern_and_instrument_fields() {
    let sentry = &entry_map("minecraft:trim_pattern")["minecraft:sentry"];
    assert_eq!(field(sentry, "asset_id"), Some(&string("minecraft:sentry")));
    assert_eq!(field(sentry, "decal"), Some(&Tag::Byte(0)));
    assert_eq!(
        field(compound(sentry, "description"), "translate"),
        Some(&string("trim_pattern.minecraft.sentry"))
    );

    let flower = &entry_map("minecraft:banner_pattern")["minecraft:flower"];
    assert_eq!(field(flower, "asset_id"), Some(&string("minecraft:flower")));
    assert_eq!(
        field(flower, "translation_key"),
        Some(&string("block.minecraft.banner.flower"))
    );

    let horn = &entry_map("minecraft:instrument")["minecraft:ponder_goat_horn"];
    assert_eq!(
        field(horn, "sound_event"),
        Some(&string("minecraft:item.goat_horn.sound.0"))
    );
    assert_eq!(field(horn, "use_duration"), Some(&Tag::Float(7.0)));
    assert_eq!(field(horn, "range"), Some(&Tag::Float(256.0)));
}

#[test]
fn variants_reference_the_client_assets_vanilla_expects() {
    let cats = entry_map("minecraft:cat_variant");
    assert_eq!(
        field(&cats["minecraft:tabby"], "asset_id"),
        Some(&string("minecraft:entity/cat/cat_tabby"))
    );
    assert_eq!(
        field(&cats["minecraft:tabby"], "baby_asset_id"),
        Some(&string("minecraft:entity/cat/cat_tabby_baby"))
    );
    let frogs = entry_map("minecraft:frog_variant");
    assert_eq!(
        field(&frogs["minecraft:cold"], "asset_id"),
        Some(&string("minecraft:entity/frog/frog_cold"))
    );
    let wolves = entry_map("minecraft:wolf_variant");
    let ashen = &wolves["minecraft:ashen"];
    assert_eq!(
        field(compound(ashen, "assets"), "angry"),
        Some(&string("minecraft:entity/wolf/wolf_ashen_angry"))
    );
    assert_eq!(
        field(compound(ashen, "baby_assets"), "wild"),
        Some(&string("minecraft:entity/wolf/wolf_ashen_baby"))
    );
    let nautilus = entry_map("minecraft:zombie_nautilus_variant");
    assert_eq!(
        field(&nautilus["minecraft:warm"], "model"),
        Some(&string("warm"))
    );
}
