//! Java LevelVersion.parse defaults, shared with primary metadata loading.
use super::*;

impl LevelVersion {
    pub fn parse_level_dat(tag: &Tag) -> Option<Self> {
        let data = level_dat_data_compound(tag)?;
        let field = |key: &str| {
            data.iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value)
        };
        let raw_version = field("Version");
        let version = compound_tag(data, "Version");
        let version_field = |key: &str| {
            version.and_then(|fields| {
                fields
                    .iter()
                    .find(|(name, _)| name == key)
                    .map(|(_, value)| value)
            })
        };
        // OptionalDynamic.result tests presence, not whether the value is a map.
        // Missing Version means legacy metadata; malformed/present Version uses
        // the running version as the fallback for unavailable child fields.
        let present = raw_version.is_some();
        Some(Self {
            level_data_version: field("version")
                .and_then(difficulty_settings::nbt_integer)
                .unwrap_or(0),
            data_version: field("DataVersion")
                .and_then(Tag::numeric_value)
                .map(|number| number.int_value()),
            last_played: field("LastPlayed")
                .and_then(Tag::numeric_value)
                .map(|number| number.boxed_long_value())
                .unwrap_or(0),
            minecraft_version_name: version
                .and_then(|fields| compound_string(fields, "Name"))
                .unwrap_or(if present { CURRENT_VERSION_NAME } else { "" })
                .to_owned(),
            minecraft_version: MinecraftDataVersion {
                id: version_field("Id")
                    .and_then(difficulty_settings::nbt_integer)
                    .unwrap_or(if present {
                        crate::storage::datafix::TARGET_DATA_VERSION
                    } else {
                        0
                    }),
                series: version
                    .and_then(|fields| compound_string(fields, "Series"))
                    .unwrap_or(CURRENT_VERSION_SERIES)
                    .to_owned(),
            },
            snapshot: version_field("Snapshot")
                .and_then(difficulty_settings::nbt_boolean)
                .unwrap_or(present && CURRENT_VERSION_SNAPSHOT),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(fields: Vec<(&str, Tag)>) -> Tag {
        Tag::Compound(vec![(
            "Data".to_owned(),
            Tag::Compound(
                fields
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value))
                    .collect(),
            ),
        )])
    }

    #[test]
    fn missing_version_differs_from_present_empty_or_malformed_version() {
        let missing = LevelVersion::parse_level_dat(&root(vec![])).unwrap();
        assert_eq!(missing.minecraft_version_name, "");
        assert_eq!(missing.minecraft_version.id, 0);
        assert_eq!(missing.minecraft_version.series, "main");
        assert_eq!(missing.level_data_version, 0);
        assert_eq!(missing.last_played, 0);
        assert_eq!(missing.data_version, None);
        for value in [
            Tag::Compound(vec![]),
            Tag::String("bad".to_owned()),
            Tag::Int(0),
        ] {
            let version = LevelVersion::parse_level_dat(&root(vec![("Version", value)])).unwrap();
            assert_eq!(version.minecraft_version_name, CURRENT_VERSION_NAME);
            assert_eq!(
                version.minecraft_version.id,
                crate::storage::datafix::TARGET_DATA_VERSION
            );
            assert_eq!(version.snapshot, CURRENT_VERSION_SNAPSHOT);
        }
    }

    #[test]
    fn version_fields_use_dynamic_numeric_conversions_and_per_field_defaults() {
        let tag = root(vec![
            ("version", Tag::Double(19133.9)),
            ("LastPlayed", Tag::Double(-1.9)),
            (
                "Version",
                Tag::Compound(vec![
                    ("Id".to_owned(), Tag::Long(4_294_967_297)),
                    ("Name".to_owned(), Tag::Int(12)),
                    ("Series".to_owned(), Tag::String("custom".to_owned())),
                    ("Snapshot".to_owned(), Tag::Int(257)),
                ]),
            ),
        ]);
        let version = LevelVersion::parse_level_dat(&tag).unwrap();
        assert_eq!(version.level_data_version, 19133);
        assert_eq!(version.last_played, -1);
        assert_eq!(version.minecraft_version.id, 1);
        assert_eq!(version.minecraft_version_name, CURRENT_VERSION_NAME);
        assert_eq!(version.minecraft_version.series, "custom");
        assert!(version.snapshot);
    }

    #[test]
    fn primary_metadata_uses_version_defaults_and_dynamic_time_and_booleans() {
        let empty = PrimaryLevelData::from_level_dat(&root(vec![])).unwrap();
        assert_eq!(empty.data_version, -1);
        assert_eq!(empty.level_data_version, 0);
        assert_eq!(empty.version.name, "");
        assert_eq!(empty.version.id, 0);
        assert_eq!(empty.time, 0);
        assert!(empty.initialized);
        assert!(!empty.was_modded);
        let data = PrimaryLevelData::from_level_dat(&root(vec![
            ("Version", Tag::Compound(vec![])),
            ("Time", Tag::Double(-2.9)),
            ("initialized", Tag::Int(256)),
            ("WasModded", Tag::Long(257)),
        ]))
        .unwrap();
        assert_eq!(data.version.name, CURRENT_VERSION_NAME);
        assert_eq!(data.time, -2);
        assert!(!data.initialized);
        assert!(data.was_modded);
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source_contract_distinguishes_presence_and_defaults() {
        const JAVA: &str =
            vibecraft_java_source!("/net/minecraft/world/level/storage/LevelVersion.java");
        for fragment in [
            "version.result().isPresent()",
            "new LevelVersion(levelDataVersion, lastPlayed, \"\", 0, \"main\", false)",
            "input.get(\"version\").asInt(0)",
            "input.get(\"LastPlayed\").asLong(0L)",
        ] {
            assert!(JAVA.contains(fragment), "missing Java contract: {fragment}");
        }
    }
    #[test]
    fn summaries_share_settings_and_derive_conversion_from_storage_version() {
        let directory = LevelDirectory::new("worlds/example");
        for (format_version, needs_conversion) in [(19132, true), (19133, false)] {
            let tag = root(vec![
                ("version", Tag::Int(format_version)),
                ("GameType", Tag::Byte(1)),
                ("LevelName", Tag::String(String::new())),
                (
                    "difficulty_settings",
                    DifficultySettings {
                        difficulty: LevelDifficulty::Hard,
                        hardcore: true,
                        locked: false,
                    }
                    .to_nbt(),
                ),
                (
                    "requiresManualConversion",
                    Tag::Byte(i8::from(!needs_conversion)),
                ),
            ]);
            let summary = LevelSummary::from_level_dat(&directory, &tag, true).unwrap();
            assert_eq!(summary.level_name, "example");
            assert_eq!(summary.game_type, LevelGameType::Creative);
            assert!(summary.hardcore);
            assert!(summary.cheats);
            assert!(summary.locked);
            assert_eq!(summary.requires_manual_conversion, needs_conversion);
        }
        for format_version in [0, 19131, 19134, -1] {
            let error = LevelSummary::from_level_dat(
                &directory,
                &root(vec![("version", Tag::Int(format_version))]),
                false,
            )
            .unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
            assert_eq!(
                error.to_string(),
                format!("Unknown data version: {:x}", format_version as u32)
            );
        }
    }
}
