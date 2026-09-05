use super::*;

struct Fixture {
    access: LevelStorageAccess,
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("vibecraft-delete-{name}-{}", std::process::id()));
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

#[test]
fn lock_stays_held_through_children_and_closes_before_root_removal() {
    let mut fixture = Fixture::new("order");
    let world = fixture.access.level_directory.path().to_owned();
    fs::create_dir_all(world.join("nested")).unwrap();
    fs::write(world.join("nested/session.lock"), b"ordinary nested file").unwrap();
    fs::write(world.join("nested/data"), b"payload").unwrap();
    let mut visited = Vec::new();
    fixture
        .access
        .delete_with(
            |path, directory| {
                let final_root_step = path == world || path == world.join("session.lock");
                assert_eq!(SessionLock::is_locked(&world).unwrap(), !final_root_step);
                visited.push(path.to_owned());
                remove_if_exists(path, directory)
            },
            |_| panic!("successful deletion must not retry"),
        )
        .unwrap();
    assert!(!fixture.access.lock.is_valid());
    assert!(!world.exists());
    assert!(visited.contains(&world.join("nested/session.lock")));
    assert_eq!(
        &visited[visited.len() - 2..],
        &[world.join("session.lock"), world.clone()]
    );
}

#[test]
fn transient_failures_retry_four_times_then_succeed_with_lock_held() {
    let mut fixture = Fixture::new("transient");
    let world = fixture.access.level_directory.path().to_owned();
    let blocked = world.join("busy.dat");
    fs::write(&blocked, b"payload").unwrap();
    let mut attempts = 0;
    let mut waits = Vec::new();
    fixture
        .access
        .delete_with(
            |path, directory| {
                if path == blocked {
                    attempts += 1;
                    assert!(SessionLock::is_locked(&world).unwrap());
                    if attempts < 5 {
                        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "busy file"));
                    }
                }
                remove_if_exists(path, directory)
            },
            |delay| waits.push(delay),
        )
        .unwrap();
    assert_eq!(attempts, 5);
    assert_eq!(waits, vec![Duration::from_millis(500); 4]);
    assert!(!world.exists());
}

#[test]
fn permanent_child_failure_returns_error_and_retains_session_ownership() {
    let mut fixture = Fixture::new("permanent");
    let world = fixture.access.level_directory.path().to_owned();
    fs::write(world.join("busy.dat"), b"payload").unwrap();
    let mut attempts = 0;
    let error = fixture
        .access
        .delete_with(
            |_, _| {
                attempts += 1;
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "busy file"))
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(attempts, 5);
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(fixture.access.lock.is_valid());
    assert!(SessionLock::is_locked(&world).unwrap());
    assert_eq!(fs::read(world.join("busy.dat")).unwrap(), b"payload");
    fixture.access.delete_level().unwrap();
}

#[test]
fn root_removal_failure_is_reported_after_lock_release_and_closed_access_is_rejected() {
    let mut fixture = Fixture::new("root");
    let world = fixture.access.level_directory.path().to_owned();
    let mut attempts = 0;
    let error = fixture
        .access
        .delete_with(
            |path, directory| {
                if path == world {
                    attempts += 1;
                    return Err(io::Error::other("root removal failed"));
                }
                remove_if_exists(path, directory)
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(error.to_string(), "root removal failed");
    assert_eq!(attempts, 5);
    assert!(!fixture.access.lock.is_valid());
    assert!(world.exists());
    assert!(!world.join("session.lock").exists());
    assert_eq!(
        fixture.access.delete_level().unwrap_err().to_string(),
        "Lock is no longer valid"
    );
    assert_eq!(
        fixture.access.make_world_backup().unwrap_err().to_string(),
        "Lock is no longer valid"
    );
    assert!(!fixture.root.join("backups").exists());
}

#[cfg(unix)]
#[test]
fn deletion_unlinks_directory_and_dangling_links_without_touching_targets() {
    let mut fixture = Fixture::new("symlinks");
    let outside = fixture.root.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("keep"), b"outside world").unwrap();
    let world = fixture.access.level_directory.path();
    std::os::unix::fs::symlink(&outside, world.join("directory-link")).unwrap();
    std::os::unix::fs::symlink("missing", world.join("dangling-link")).unwrap();
    fixture.access.delete_level().unwrap();
    assert_eq!(fs::read(outside.join("keep")).unwrap(), b"outside world");
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_deletion_contract_keeps_lock_until_root_postvisit_and_retries_five_times() {
    const JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/LevelStorageSource.java");
    for fragment in [
        "for (int attempt = 1; attempt <= 5; attempt++)",
        "if (!file.equals(lockPath))",
        "if (dir.equals(LevelStorageAccess.this.levelDirectory.path()))",
        "LevelStorageAccess.this.lock.close()",
        "Files.deleteIfExists(lockPath)",
        "Thread.sleep(500L)",
    ] {
        assert!(
            JAVA.contains(fragment),
            "missing Java deletion contract: {fragment}"
        );
    }
}
