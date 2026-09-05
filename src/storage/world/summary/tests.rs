use super::*;

fn root(version: i32, data_version: i32, played: i64) -> Tag {
    Tag::Compound(vec![(
        "Data".to_owned(),
        Tag::Compound(vec![
            ("version".to_owned(), Tag::Int(19133)),
            ("DataVersion".to_owned(), Tag::Int(data_version)),
            ("LastPlayed".to_owned(), Tag::Long(played)),
            (
                "Version".to_owned(),
                Tag::Compound(vec![("Id".to_owned(), Tag::Int(version))]),
            ),
        ]),
    )])
}

fn summary(version: i32, data_version: i32) -> LevelSummary {
    LevelSummary::from_level_dat(
        &LevelDirectory::new("world"),
        &root(version, data_version, 0),
        false,
    )
    .unwrap()
}

#[test]
fn backup_policy_obeys_file_fixing_priority_and_stable_snapshot_boundaries() {
    for (saved, current, snapshot, expected) in [
        (-1, 4790, false, BackupStatus::FileFixingRequired),
        (4772, 4790, true, BackupStatus::FileFixingRequired),
        (4773, 4790, false, BackupStatus::None),
        (4773, 4790, true, BackupStatus::UpgradeToSnapshot),
        (4790, 4790, true, BackupStatus::None),
        (4791, 4790, false, BackupStatus::Downgrade),
        (4791, 4790, true, BackupStatus::Downgrade),
    ] {
        assert_eq!(
            summary(saved, saved).backup_status_for(current, snapshot),
            expected
        );
    }
    for (status, backup, severe, key) in [
        (BackupStatus::None, false, false, ""),
        (BackupStatus::Downgrade, true, true, "downgrade"),
        (BackupStatus::UpgradeToSnapshot, true, false, "snapshot"),
        (
            BackupStatus::FileFixingRequired,
            true,
            false,
            "file_fixing_required",
        ),
    ] {
        assert_eq!(status.should_backup(), backup);
        assert_eq!(status.is_severe(), severe);
        assert_eq!(status.translation_key(), key);
    }
}

#[test]
fn summary_and_backup_file_fixing_use_distinct_java_version_inputs() {
    let first = summary(4790, 4772);
    assert!(first.requires_file_fixing);
    assert!(!first.should_backup());
    assert_eq!(
        first.primary_action_translation_key(),
        "selectWorld.upgrade_and_play"
    );
    let second = summary(4772, 4790);
    assert!(!second.requires_file_fixing);
    assert!(second.should_backup());
    assert_eq!(
        second.primary_action_translation_key(),
        "selectWorld.select"
    );
    assert!(summary(4791, 4790).is_downgrade());
}

#[test]
fn action_policies_keep_upload_and_conversion_distinct_from_compatibility() {
    for locked in [false, true] {
        for conversion in [false, true] {
            for compatible in [false, true] {
                for file_fix in [false, true] {
                    let mut value = summary(4790, if file_fix { 4772 } else { 4790 });
                    value.locked = locked;
                    value.requires_manual_conversion = conversion;
                    value.version.minecraft_version.series =
                        if compatible { "main" } else { "other" }.to_owned();
                    let disabled = locked || conversion || !compatible;
                    assert_eq!(value.is_disabled(), disabled);
                    assert_eq!(value.primary_action_active(), !disabled);
                    assert_eq!(value.can_upload(), !locked && !conversion);
                    assert_eq!(value.can_edit(), !disabled && !file_fix);
                    assert_eq!(value.can_recreate(), !disabled && !file_fix);
                    assert!(value.can_delete());
                }
            }
        }
    }
}

#[test]
fn summaries_sort_by_recent_time_then_java_utf16_world_id_and_keep_errors_visible() {
    let path = std::env::temp_dir().join(format!("vibecraft-summary-order-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    let source = LevelStorageSource::create_default(&path).unwrap();
    for (name, time) in [
        ("old", i64::MIN),
        ("recent", i64::MAX),
        ("\u{10000}", 0),
        ("\u{e000}", 0),
    ] {
        let layout = WorldLayout::new(path.join(name));
        layout.save_level_dat(&root(4790, 4790, time)).unwrap();
    }
    fs::create_dir(path.join("broken")).unwrap();
    fs::write(path.join("broken/level.dat"), b"broken").unwrap();
    let summaries = source.load_level_summaries().unwrap();
    let names: Vec<_> = summaries
        .iter()
        .filter_map(|value| value.as_ref().ok())
        .map(|value| value.directory_name.as_str())
        .collect();
    assert_eq!(names, ["recent", "\u{10000}", "\u{e000}", "old"]);
    assert!(summaries.last().unwrap().is_err());
    fs::remove_dir_all(path).unwrap();
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_summary_policy_and_latest_file_fixer_contracts() {
    const SUMMARY: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/LevelSummary.java");
    const FIXERS: &str = vibecraft_java_source!("/net/minecraft/util/datafix/DataFixers.java");
    const FILE_FIXER: &str =
        vibecraft_java_source!("/net/minecraft/util/filefix/FileFixerUpper.java");
    assert!(FIXERS.contains("fileFixerUpper.addSchema(fixerUpper, 4773, SAME_NAMESPACED)"));
    assert!(FILE_FIXER.contains("levelDataVersion < 4772 ? 0 : levelDataVersion"));
    for fragment in [
        "this.levelId.compareTo(rhs.levelId)",
        "DataFixers.getFileFixer().requiresFileFixing(levelVersionNumber)",
        "!currentVersion.stable() && levelVersionNumber < currentVersionNumber",
        "return !this.requiresManualConversion() && !this.isLocked()",
        "return !this.isDisabled() && !this.requiresFileFixing()",
    ] {
        assert!(
            SUMMARY.contains(fragment),
            "missing Java summary contract: {fragment}"
        );
    }
}
