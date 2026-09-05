use super::*;

struct Fixture {
    access: LevelStorageAccess,
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("vibecraft-rename-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let source = LevelStorageSource::new(&root, root.join("backups")).unwrap();
        Self {
            access: source.create_access("world").unwrap(),
            root,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.access.lock.close().unwrap();
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn metadata() -> Tag {
    Tag::Compound(vec![
        ("RootExtension".to_owned(), Tag::Int(7)),
        (
            "Data".to_owned(),
            Tag::Compound(vec![
                ("LevelName".to_owned(), Tag::String("Original".to_owned())),
                ("DataVersion".to_owned(), Tag::Int(1)),
                (
                    "singleplayer_uuid".to_owned(),
                    Tag::IntArray(vec![1, 2, 3, 4]),
                ),
                (
                    "Player".to_owned(),
                    Tag::Compound(vec![("Health".to_owned(), Tag::Float(17.0))]),
                ),
                (
                    "UnknownExtension".to_owned(),
                    Tag::LongArray(vec![i64::MIN, i64::MAX]),
                ),
            ]),
        ),
    ])
}

#[test]
fn rename_preserves_unrelated_data_and_old_version_then_drops_only_identity() {
    let fixture = Fixture::new("preserve");
    let access = &fixture.access;
    let original = metadata();
    access.save_level_data(&original).unwrap();
    let before = fs::read(access.layout().level_dat()).unwrap();
    access.rename_level("\0\t New Name \r\u{1f}").unwrap();
    let mut expected = original;
    if let Tag::Compound(fields) = &mut expected {
        if let Tag::Compound(data) = &mut fields[1].1 {
            data[0].1 = Tag::String("New Name".to_owned());
        }
    }
    assert_eq!(access.read_level_data().unwrap(), expected);
    assert_eq!(fs::read(access.layout().level_dat_old()).unwrap(), before);
    let renamed_bytes = fs::read(access.layout().level_dat()).unwrap();
    access
        .rename_and_drop_player("  \u{a0} Name \u{2003}  ")
        .unwrap();
    if let Tag::Compound(fields) = &mut expected {
        if let Tag::Compound(data) = &mut fields[1].1 {
            data[0].1 = Tag::String("\u{a0} Name \u{2003}".to_owned());
            data.retain(|(key, _)| key != "singleplayer_uuid");
        }
    }
    assert_eq!(access.read_level_data().unwrap(), expected);
    assert_eq!(
        fs::read(access.layout().level_dat_old()).unwrap(),
        renamed_bytes
    );
}

#[test]
fn malformed_or_missing_data_is_saved_without_creating_a_replacement_compound() {
    let fixture = Fixture::new("shape");
    for root in [
        Tag::Compound(vec![]),
        Tag::Compound(vec![("Data".to_owned(), Tag::Int(4))]),
        Tag::Compound(vec![(
            "LevelName".to_owned(),
            Tag::String("flat".to_owned()),
        )]),
    ] {
        fixture.access.save_level_data(&root).unwrap();
        fixture.access.rename_and_drop_player("Renamed").unwrap();
        assert_eq!(fixture.access.read_level_data().unwrap(), root);
    }
}

#[test]
fn invalid_primary_never_uses_or_overwrites_the_backup() {
    let fixture = Fixture::new("invalid");
    let layout = fixture.access.layout();
    let mut good = Vec::new();
    write_gzip_named_tag(&mut good, "", &metadata()).unwrap();
    fs::write(layout.level_dat_old(), &good).unwrap();
    let mut non_compound = Vec::new();
    write_gzip_named_tag(&mut non_compound, "", &Tag::Int(7)).unwrap();
    let mut uncompressed = Vec::new();
    write_named_tag(&mut uncompressed, "", &metadata()).unwrap();
    for bytes in [b"truncated".to_vec(), non_compound, uncompressed] {
        fs::write(layout.level_dat(), &bytes).unwrap();
        assert!(fixture.access.rename_level("New").is_err());
        assert!(fixture.access.rename_and_drop_player("New").is_err());
        assert_eq!(fs::read(layout.level_dat()).unwrap(), bytes);
        assert_eq!(fs::read(layout.level_dat_old()).unwrap(), good);
    }
    fs::remove_file(layout.level_dat()).unwrap();
    assert_eq!(
        fixture.access.rename_level("New").unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    assert!(!layout.level_dat().exists());
    assert_eq!(fs::read(layout.level_dat_old()).unwrap(), good);
}

#[test]
fn closed_session_rejects_rename_before_reading_or_writing() {
    let mut fixture = Fixture::new("closed");
    fixture.access.save_level_data(&metadata()).unwrap();
    let before = fs::read(fixture.access.layout().level_dat()).unwrap();
    fixture.access.lock.close().unwrap();
    assert_eq!(
        fixture.access.rename_level("New").unwrap_err().to_string(),
        "Lock is no longer valid"
    );
    assert_eq!(
        fs::read(fixture.access.layout().level_dat()).unwrap(),
        before
    );
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_metadata_edit_contract_uses_primary_without_datafix() {
    const JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/LevelStorageSource.java");
    for fragment in [
        "renameAndDropPlayer(final String newName)",
        "tag.putString(\"LevelName\", newName.trim())",
        "tag.remove(\"singleplayer_uuid\")",
        "readLevelDataTagRaw(this.levelDirectory.dataFile())",
        "updater.accept(root.getCompoundOrEmpty(\"Data\"))",
    ] {
        assert!(
            JAVA.contains(fragment),
            "missing Java metadata contract: {fragment}"
        );
    }
}
