use super::*;
use std::io::Read;

#[test]
fn archives_preserve_world_root_bytes_and_collision_backups() {
    let root = std::env::temp_dir().join(format!("vibecraft-zip-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let world = root.join("world");
    let backups = root.join("backups");
    fs::create_dir_all(world.join("dimensions/minecraft/the_nether/region")).unwrap();
    fs::create_dir_all(world.join("empty")).unwrap();
    let region = "dimensions/minecraft/the_nether/region/r.0.0.mca";
    let bytes: Vec<_> = (0..200_000).map(|i| (i % 251) as u8).collect();
    fs::write(world.join(region), &bytes).unwrap();
    fs::write(world.join("level.dat"), b"original").unwrap();
    fs::write(world.join("session.lock"), b"locked").unwrap();
    fs::write(world.join("empty/session.lock"), b"also locked").unwrap();
    let stamp = "2026-09-05_12-34-56";
    let first = write_world_backup(&world, &backups, "My World.v2", stamp).unwrap();
    fs::write(world.join("level.dat"), b"changed").unwrap();
    let second = write_world_backup(&world, &backups, "My World.v2", stamp).unwrap();
    assert_eq!(
        first.file_name().unwrap(),
        "2026-09-05_12-34-56_My World_v2.zip"
    );
    assert_eq!(
        second.file_name().unwrap(),
        "2026-09-05_12-34-56_My World_v2 (1).zip"
    );
    for (path, expected) in [
        (first, b"original".as_slice()),
        (second, b"changed".as_slice()),
    ] {
        let mut zip = zip::ZipArchive::new(File::open(path).unwrap()).unwrap();
        assert_eq!(zip.len(), 2);
        let mut contents = Vec::new();
        {
            let mut entry = zip.by_name("My World.v2/level.dat").unwrap();
            assert_eq!(entry.compression(), CompressionMethod::Deflated);
            entry.read_to_end(&mut contents).unwrap();
        }
        assert_eq!(contents, expected);
        contents.clear();
        zip.by_name(&format!("My World.v2/{region}"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, bytes);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn archive_names_preserve_unicode_and_skip_existing_files_and_directories() {
    let root = std::env::temp_dir().join(format!("vibecraft-zip-names-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let prefix = "2026-09-05_12-34-56_世界: one.two";
    let base = "2026-09-05_12-34-56_世界_ one_two";
    fs::write(root.join(format!("{base}.zip")), b"keep").unwrap();
    fs::create_dir(root.join(format!("{base} (1).zip"))).unwrap();
    let (path, file) = create_archive(&root, prefix).unwrap();
    drop(file);
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        format!("{base} (2).zip")
    );
    assert_eq!(fs::read(root.join(format!("{base}.zip"))).unwrap(), b"keep");
    assert_eq!(java_prefix("a😀z", 3), "a😀");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_world_backup_contract_uses_zip_file_visits_and_world_id_root() {
    const JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/LevelStorageSource.java");
    for fragment in [
        "FileUtil.findAvailableName(root, zipFilePrefix, \".zip\")",
        "new ZipOutputStream(new BufferedOutputStream(Files.newOutputStream(zipFilePath)))",
        "Paths.get(this.levelId)",
        "path.endsWith(\"session.lock\")",
        "stream.putNextEntry(entry)",
        "asByteSource(path.toFile()).copyTo(stream)",
    ] {
        assert!(
            JAVA.contains(fragment),
            "missing Java backup contract: {fragment}"
        );
    }
}

#[cfg(unix)]
#[test]
fn unreadable_link_target_fails_backup_instead_of_silently_omitting_data() {
    let root = std::env::temp_dir().join(format!("vibecraft-zip-error-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let world = root.join("world");
    fs::create_dir_all(&world).unwrap();
    std::os::unix::fs::symlink("missing", world.join("linked.dat")).unwrap();
    let error = write_world_backup(
        &world,
        &root.join("backups"),
        "world",
        "2026-09-05_12-34-56",
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    fs::remove_dir_all(root).unwrap();
}
