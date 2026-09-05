use super::*;

struct Fixture(PathBuf);
impl Fixture {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "vibecraft-validation-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn absent_root_is_allowed_but_regular_file_is_not_a_directory() {
    let fixture = Fixture::new("roots");
    let validator = DirectoryValidator::deny_all();
    let missing = fixture.0.join("missing");
    assert!(validator
        .validate_directory(&missing, false)
        .unwrap()
        .is_empty());
    assert!(!missing.exists());
    let file = fixture.0.join("file");
    fs::write(&file, b"contents").unwrap();
    assert_eq!(
        validator
            .validate_directory(&file, true)
            .unwrap_err()
            .to_string(),
        format!("Path {} is not a directory", file.display())
    );
    assert_eq!(
        validator
            .validate_known_directory(&missing, &mut vec![])
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn validation_diagnostics_preserve_link_target_pairs_and_java_message_format() {
    let error = ContentValidationError {
        directory: PathBuf::from("world"),
        entries: vec![
            ForbiddenSymlinkInfo {
                link: PathBuf::from("world/a"),
                target: PathBuf::from("../a"),
            },
            ForbiddenSymlinkInfo {
                link: PathBuf::from("world/b"),
                target: PathBuf::from("/b"),
            },
        ],
    };
    assert_eq!(
        error.to_string(),
        "Failed to validate 'world'. Found forbidden symlinks: world/a->../a, world/b->/b"
    );
}

#[cfg(unix)]
#[test]
fn raw_targets_and_dangling_links_are_checked_without_following_allowed_directories() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("raw");
    let world = fixture.0.join("world");
    let outside = fixture.0.join("outside");
    fs::create_dir(&world).unwrap();
    fs::create_dir(&outside).unwrap();
    symlink("not-allowed", outside.join("ignored-child")).unwrap();
    symlink(&outside, world.join("allowed-directory")).unwrap();
    symlink("../raw-target", world.join("allowed-relative")).unwrap();
    symlink("../denied", world.join("denied")).unwrap();
    let validator = DirectoryValidator::new(|target: &Path| {
        target == outside || target == Path::new("../raw-target")
    });
    let issues = validator.validate_directory(&world, false).unwrap();
    assert_eq!(
        issues,
        vec![ForbiddenSymlinkInfo {
            link: world.join("denied"),
            target: PathBuf::from("../denied")
        }]
    );
    assert!(validator
        .validate_symlink(&world.join("allowed-relative"))
        .unwrap()
        .is_empty());
    let deny = DirectoryValidator::deny_all();
    assert_eq!(deny.validate_directory(&world, false).unwrap().len(), 3);
    assert_eq!(
        deny.validate_directory(&world.join("denied"), false)
            .unwrap()
            .len(),
        1
    );
}

#[cfg(unix)]
#[test]
fn top_symlink_flag_selects_link_validation_or_raw_target_traversal() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("top");
    let target = fixture.0.join("target");
    fs::create_dir(&target).unwrap();
    let link = fixture.0.join("link");
    symlink(&target, &link).unwrap();
    let validator = DirectoryValidator::deny_all();
    assert_eq!(
        validator.validate_directory(&link, false).unwrap(),
        vec![ForbiddenSymlinkInfo {
            link: link.clone(),
            target: target.clone()
        }]
    );
    assert!(validator
        .validate_directory(&link, true)
        .unwrap()
        .is_empty());
    symlink("missing", target.join("child")).unwrap();
    assert_eq!(
        validator.validate_directory(&link, true).unwrap(),
        vec![ForbiddenSymlinkInfo {
            link: target.join("child"),
            target: PathBuf::from("missing")
        }]
    );
}

#[cfg(unix)]
#[test]
fn world_access_allows_top_symlink_and_rejects_dangling_children_before_locking() {
    use crate::storage::world::LevelStorageSource;
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("access");
    let target = fixture.0.join("target");
    fs::create_dir(&target).unwrap();
    let source =
        LevelStorageSource::new(fixture.0.join("worlds"), fixture.0.join("backups")).unwrap();
    symlink(&target, fixture.0.join("worlds/world")).unwrap();
    let access = source.validate_and_create_access("world").unwrap();
    assert!(target.join("session.lock").is_file());
    drop(access);
    fs::remove_file(target.join("session.lock")).unwrap();
    symlink("missing", target.join("bad-link")).unwrap();
    let error = source.validate_and_create_access("world").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    let details = error
        .get_ref()
        .unwrap()
        .downcast_ref::<ContentValidationError>()
        .unwrap();
    assert_eq!(details.entries[0].target, PathBuf::from("missing"));
    assert!(!target.join("session.lock").exists());
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn source_contract_covers_raw_targets_and_no_follow_root_attributes() {
    const JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/level/validation/DirectoryValidator.java");
    for fragment in [
        "Files.readSymbolicLink(path)",
        "this.symlinkTargetAllowList.matches(target)",
        "BasicFileAttributes.class, LinkOption.NOFOLLOW_LINKS",
        "if (!allowTopSymlink)",
        "directory = Files.readSymbolicLink(directory)",
        "Files.walkFileTree(directory",
    ] {
        assert!(
            JAVA.contains(fragment),
            "missing Java validation contract: {fragment}"
        );
    }
}
