use super::*;
use crate::storage::world::{LevelGameType, PrimaryLevelData};

fn metadata(extra: Vec<(&str, Tag)>) -> Tag {
    let mut fields = vec![
        (
            "DataVersion".to_owned(),
            Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        (
            "Version".to_owned(),
            Tag::Compound(vec![
                (
                    "Id".to_owned(),
                    Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
                ),
                ("Name".to_owned(), Tag::String("26.1.2".to_owned())),
            ]),
        ),
    ];
    fields.extend(
        extra
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value)),
    );
    Tag::Compound(vec![("Data".to_owned(), Tag::Compound(fields))])
}

#[test]
fn difficulty_record_preserves_all_flags_and_named_difficulties() {
    for difficulty in [
        LevelDifficulty::Peaceful,
        LevelDifficulty::Easy,
        LevelDifficulty::Normal,
        LevelDifficulty::Hard,
    ] {
        for hardcore in [false, true] {
            for locked in [false, true] {
                let value = DifficultySettings {
                    difficulty,
                    hardcore,
                    locked,
                };
                assert_eq!(
                    DifficultySettings::from_nbt(&value.to_nbt()).unwrap(),
                    value
                );
                assert_eq!(
                    LevelDifficulty::from_name(difficulty.serialized_name()),
                    Some(difficulty)
                );
            }
        }
    }
    assert_eq!(
        DifficultySettings::default(),
        DifficultySettings {
            difficulty: LevelDifficulty::Normal,
            hardcore: false,
            locked: false
        }
    );
}

#[test]
fn invalid_record_defaults_as_a_whole_in_level_settings() {
    let Tag::Compound(fields) = (DifficultySettings {
        difficulty: LevelDifficulty::Hard,
        hardcore: true,
        locked: true,
    })
    .to_nbt() else {
        panic!("compound")
    };
    for index in 0..fields.len() {
        let mut missing = fields.clone();
        missing.remove(index);
        let tag = Tag::Compound(missing);
        assert!(DifficultySettings::from_nbt(&tag).is_err());
        let loaded =
            PrimaryLevelData::from_level_dat(&metadata(vec![("difficulty_settings", tag)]))
                .unwrap();
        assert_eq!(loaded.difficulty_settings, DifficultySettings::default());
    }
    for (index, invalid) in [
        (0, Tag::String("HARD".to_owned())),
        (0, Tag::Byte(3)),
        (1, Tag::String("true".to_owned())),
        (2, Tag::List(vec![])),
    ] {
        let mut bad = fields.clone();
        bad[index].1 = invalid;
        assert!(DifficultySettings::from_nbt(&Tag::Compound(bad)).is_err());
    }
}

#[test]
fn numeric_booleans_use_java_byte_narrowing() {
    for (value, expected) in [
        (Tag::Int(256), false),
        (Tag::Long(257), true),
        (Tag::Short(-256), false),
        (Tag::Double(0.9), false),
        (Tag::Float(-1.9), true),
        (Tag::Double(f64::NAN), false),
    ] {
        let tag = Tag::Compound(vec![
            ("difficulty".to_owned(), Tag::String("easy".to_owned())),
            ("hardcore".to_owned(), value.clone()),
            ("locked".to_owned(), value),
        ]);
        let parsed = DifficultySettings::from_nbt(&tag).unwrap();
        assert_eq!((parsed.hardcore, parsed.locked), (expected, expected));
    }
}

#[test]
fn level_settings_use_java_defaults_and_ignore_legacy_difficulty_fields() {
    let parsed = PrimaryLevelData::from_level_dat(&metadata(vec![
        ("Difficulty", Tag::Byte(3)),
        ("hardcore", Tag::Byte(1)),
        ("DifficultyLocked", Tag::Byte(1)),
    ]))
    .unwrap();
    assert_eq!(parsed.level_name, "");
    assert_eq!(parsed.game_type, LevelGameType::Survival);
    assert!(!parsed.allow_commands);
    assert_eq!(parsed.difficulty_settings, DifficultySettings::default());
    for (mode, expected) in [
        (Tag::Byte(1), LevelGameType::Creative),
        (Tag::Double(2.9), LevelGameType::Adventure),
        (Tag::Long(4_294_967_297), LevelGameType::Creative),
        (Tag::Int(99), LevelGameType::Survival),
        (Tag::Int(-1), LevelGameType::Survival),
        (Tag::String("creative".to_owned()), LevelGameType::Survival),
    ] {
        let loaded = PrimaryLevelData::from_level_dat(&metadata(vec![("GameType", mode)])).unwrap();
        assert_eq!(loaded.game_type, expected);
        assert_eq!(loaded.allow_commands, expected == LevelGameType::Creative);
    }
    let creative = PrimaryLevelData::from_level_dat(&metadata(vec![
        ("GameType", Tag::Int(1)),
        ("allowCommands", Tag::Int(256)),
    ]))
    .unwrap();
    assert!(!creative.allow_commands);
}

#[test]
fn primary_metadata_saves_difficulty_record_without_legacy_fields() {
    let expected = DifficultySettings {
        difficulty: LevelDifficulty::Hard,
        hardcore: true,
        locked: true,
    };
    let loaded = PrimaryLevelData::from_level_dat(&metadata(vec![(
        "difficulty_settings",
        expected.to_nbt(),
    )]))
    .unwrap();
    let saved = loaded.to_level_dat().unwrap();
    assert_eq!(
        PrimaryLevelData::from_level_dat(&saved)
            .unwrap()
            .difficulty_settings,
        expected
    );
    let Tag::Compound(root) = saved else {
        panic!("root")
    };
    let Tag::Compound(data) = &root[0].1 else {
        panic!("data")
    };
    assert!(data
        .iter()
        .any(|(key, value)| key == "difficulty_settings" && value == &expected.to_nbt()));
    assert!(!data
        .iter()
        .any(|(key, _)| ["Difficulty", "DifficultyLocked", "hardcore"].contains(&key.as_str())));
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_source_defines_required_record_fields_and_settings_defaults() {
    const JAVA: &str = vibecraft_java_source!("/net/minecraft/world/level/LevelSettings.java");
    for fragment in [
        "Difficulty.CODEC.fieldOf(\"difficulty\")",
        "Codec.BOOL.fieldOf(\"hardcore\")",
        "Codec.BOOL.fieldOf(\"locked\")",
        "Difficulty.NORMAL, false, false",
        "input.get(\"LevelName\").asString(\"\")",
        "input.get(\"GameType\").asInt(0)",
        "asBoolean(gameType == GameType.CREATIVE)",
    ] {
        assert!(JAVA.contains(fragment), "missing Java contract: {fragment}");
    }
    const PRIMARY: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/PrimaryLevelData.java");
    assert!(PRIMARY.contains("tag.store(\"difficulty_settings\", LevelSettings.DifficultySettings.CODEC, this.settings.difficultySettings())"));
}
