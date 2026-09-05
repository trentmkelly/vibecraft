use super::*;
use std::time::{Duration, UNIX_EPOCH};

struct Fixture {
    root: PathBuf,
    directory: LevelDirectory,
}
impl Fixture {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("vibecraft-recovery-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("world")).unwrap();
        Self {
            directory: LevelDirectory::new(root.join("world")),
            root,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn set_time(path: &Path, millis: u64) {
    File::open(path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_millis(millis)))
        .unwrap();
}
fn normal_root() -> Tag {
    Tag::Compound(vec![(
        "Data".to_owned(),
        Tag::Compound(vec![
            ("version".to_owned(), Tag::Int(19133)),
            ("DataVersion".to_owned(), Tag::Int(4790)),
            ("LastPlayed".to_owned(), Tag::Long(9000)),
        ]),
    )])
}

#[test]
fn corrupt_primary_never_uses_backup_metadata_and_uses_primary_modification_time() {
    let fixture = Fixture::new("backup");
    let layout = fixture.directory.layout();
    layout.save_level_dat(&normal_root()).unwrap();
    fs::rename(layout.level_dat(), layout.level_dat_old()).unwrap();
    set_time(&layout.level_dat_old(), 7000);
    fs::write(layout.level_dat(), b"corrupt").unwrap();
    set_time(&layout.level_dat(), 3000);
    let summary = fixture.directory.load_summary().unwrap();
    assert!(matches!(summary, WorldSummary::Corrupted { .. }));
    assert_eq!(summary.last_played(), 3000);
    assert_eq!(summary.level_name(), "world");
    assert_eq!(fs::read(layout.level_dat()).unwrap(), b"corrupt");
    fs::remove_file(layout.level_dat()).unwrap();
    assert_eq!(
        fixture.directory.load_summary().unwrap().last_played(),
        7000
    );
    fs::remove_file(layout.level_dat_old()).unwrap();
    assert_eq!(fixture.directory.load_summary().unwrap().last_played(), -1);
}

#[test]
fn malformed_roots_and_uncompressed_files_produce_recovery_entries() {
    let fixture = Fixture::new("roots");
    for tag in [
        Tag::Int(3),
        Tag::Compound(vec![]),
        Tag::Compound(vec![("version".to_owned(), Tag::Int(19133))]),
    ] {
        let mut bytes = Vec::new();
        write_gzip_named_tag(&mut bytes, "", &tag).unwrap();
        fs::write(fixture.directory.data_file(), &bytes).unwrap();
        assert!(matches!(
            fixture.directory.load_summary().unwrap(),
            WorldSummary::Corrupted { .. }
        ));
    }
    let mut bytes = Vec::new();
    write_named_tag(&mut bytes, "", &normal_root()).unwrap();
    fs::write(fixture.directory.data_file(), bytes).unwrap();
    assert!(matches!(
        fixture.directory.load_summary().unwrap(),
        WorldSummary::Corrupted { .. }
    ));
    fixture
        .directory
        .layout()
        .save_level_dat(&normal_root())
        .unwrap();
    assert_eq!(
        fixture.directory.load_summary().unwrap().last_played(),
        9000
    );
}

#[test]
fn exceptional_actions_and_warning_components_do_not_expose_fake_normal_metadata() {
    let entries = [
        (
            WorldSummary::Corrupted {
                directory_name: "broken".to_owned(),
                icon_file: PathBuf::from("broken/icon.png"),
                last_played: 10,
            },
            "recover_world.button",
            "recover_world.warning",
        ),
        (
            WorldSummary::Symlink {
                directory_name: "linked".to_owned(),
                icon_file: PathBuf::from("linked/icon.png"),
            },
            "symlink_warning.more_info",
            "symlink_warning.title",
        ),
    ];
    for (summary, action, warning) in entries {
        assert!(summary.normal().is_none());
        assert_eq!(summary.level_name(), summary.directory_name());
        assert_eq!(
            summary.icon_file(),
            Path::new(summary.directory_name()).join("icon.png")
        );
        assert!(!summary.is_disabled());
        assert!(summary.primary_action_active());
        assert!(summary.can_delete());
        assert!(!summary.can_upload());
        assert!(!summary.can_edit());
        assert!(!summary.can_recreate());
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&summary.primary_action_message().to_json())
                .unwrap(),
            serde_json::json!({"translate":action})
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&summary.info().to_json()).unwrap(),
            serde_json::json!({"translate":warning,"color":"#FF0000"})
        );
    }
}

#[cfg(unix)]
#[test]
fn forbidden_primary_symlink_gets_warning_and_dangling_link_gets_recovery() {
    let fixture = Fixture::new("links");
    let target = fixture.root.join("target.dat");
    fs::write(&target, b"must not parse").unwrap();
    std::os::unix::fs::symlink(&target, fixture.directory.data_file()).unwrap();
    let summary = fixture.directory.load_summary().unwrap();
    assert!(matches!(summary, WorldSummary::Symlink { .. }));
    assert_eq!(summary.last_played(), -1);
    assert_eq!(fs::read(&target).unwrap(), b"must not parse");
    fs::remove_file(target).unwrap();
    assert!(matches!(
        fixture.directory.load_summary().unwrap(),
        WorldSummary::Corrupted { .. }
    ));
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_recovery_source_uses_file_times_and_dedicated_actions() {
    const STORAGE: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/LevelStorageSource.java");
    const SUMMARY: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/LevelSummary.java");
    for text in [
        "getFileModificationTime(level.dataFile())",
        "getFileModificationTime(level.oldDataFile())",
        "timeStamp == null ? -1L : timeStamp.toEpochMilli()",
    ] {
        assert!(STORAGE.contains(text));
    }
    for text in [
        "class CorruptedLevelSummary",
        "class SymlinkLevelSummary",
        "recover_world.button",
        "symlink_warning.more_info",
    ] {
        assert!(SUMMARY.contains(text));
    }
}
