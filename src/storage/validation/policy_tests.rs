use super::*;
use crate::storage::world::{LevelStorageSource, WorldLayout};
use std::io::Cursor;

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "vibecraft-symlink-policy-{name}-{}",
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
fn plain_allow_list_ignores_blank_and_comment_lines_and_defaults_to_deny() {
    let allow_list = PathAllowList::read_plain(Cursor::new("\n   \n# comment\n")).unwrap();
    assert!(!allow_list.matches(Path::new("anything")));

    let allow_list = PathAllowList::read_plain(Cursor::new(
        "# comment\n\nraw/target\n[prefix]literal-prefix\n",
    ))
    .unwrap();
    assert!(allow_list.matches(Path::new("raw/target/file")));
    assert!(allow_list.matches(Path::new("literal-prefix-and-suffix")));
    assert!(!allow_list.matches(Path::new("other/raw/target/file")));
}

#[test]
fn plain_allow_list_supports_filesystem_glob_and_regex_matchers() {
    let allow_list = PathAllowList::read_plain(Cursor::new(
        "[glob]../allowed/**\n[regex]raw-[0-9]+/target\n",
    ))
    .unwrap();

    assert!(allow_list.matches(Path::new("../allowed/nested/dangling")));
    assert!(allow_list.matches(Path::new("raw-42/target")));
    assert!(!allow_list.matches(Path::new("../denied/nested/dangling")));
    assert!(!allow_list.matches(Path::new("raw-x/target")));
}

#[test]
fn malformed_definition_is_an_error_and_matcher_compile_failure_denies_every_entry() {
    assert!(PathAllowList::read_plain(Cursor::new("[unsupported]path\n")).is_err());

    // Java parses the type but leaves matcher syntax validation until first
    // use, so a malformed glob is a valid definition that fails closed later.
    let malformed_glob = PathAllowList::read_plain(Cursor::new("[glob][unterminated\n")).unwrap();
    assert!(!malformed_glob.matches(Path::new("any-target")));

    // The prefix would allow this target on its own. Java compiles the full
    // list as one unit, so a later invalid filesystem pattern makes the cached
    // matcher deny all paths.
    let allow_list =
        PathAllowList::read_plain(Cursor::new("allowed-prefix\n[glob][unterminated\n")).unwrap();
    assert!(!allow_list.matches(Path::new("allowed-prefix/target")));
}

#[cfg(unix)]
#[test]
fn default_source_denies_missing_or_malformed_config_before_creating_session_lock() {
    use std::os::unix::fs::symlink;

    for malformed in [false, true] {
        let fixture = Fixture::new(if malformed { "malformed" } else { "missing" });
        let base = fixture.0.join("worlds");
        let target = fixture.0.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        symlink("missing-target", target.join("dangling")).unwrap();
        if malformed {
            fs::write(base.join("allowed_symlinks.txt"), "[unsupported]pattern\n").unwrap();
        }

        let source = LevelStorageSource::create_default(&base).unwrap();
        symlink(&target, base.join("world")).unwrap();
        let error = source.validate_and_create_access("world").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!target.join("session.lock").exists());
    }
}

#[cfg(unix)]
#[test]
fn configured_allow_list_checks_raw_dangling_nested_targets_and_allows_access() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new("nested-raw-targets");
    let base = fixture.0.join("worlds");
    let target = fixture.0.join("target");
    fs::create_dir_all(&base).unwrap();
    fs::create_dir_all(&target).unwrap();
    // Prefix matching is performed on the exact read_link result. This target
    // is dangling and relative to the world, yet matches the configured prefix.
    fs::write(base.join("allowed_symlinks.txt"), "../permitted\n").unwrap();
    symlink("../permitted-nested-dangling", target.join("nested-link")).unwrap();

    let source = LevelStorageSource::create_default(&base).unwrap();
    symlink(&target, base.join("world")).unwrap();
    let access = source.validate_and_create_access("world").unwrap();
    assert!(target.join("session.lock").is_file());
    drop(access);
}

#[cfg(unix)]
#[test]
fn configured_source_summaries_allow_only_matching_level_dat_symlink_targets() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new("summary-level-dat");
    let base = fixture.0.join("worlds");
    fs::create_dir_all(&base).unwrap();
    let allowed_target = fixture.0.join("allowed-source/level.dat");
    let forbidden_target = fixture.0.join("forbidden-source/level.dat");
    let valid_level_data = crate::storage::nbt::Tag::Compound(vec![(
        "Data".to_owned(),
        crate::storage::nbt::Tag::Compound(vec![
            (
                "LevelName".to_owned(),
                crate::storage::nbt::Tag::String("Target".to_owned()),
            ),
            ("version".to_owned(), crate::storage::nbt::Tag::Int(19133)),
            (
                "DataVersion".to_owned(),
                crate::storage::nbt::Tag::Int(4790),
            ),
            (
                "LastPlayed".to_owned(),
                crate::storage::nbt::Tag::Long(9000),
            ),
        ]),
    )]);
    WorldLayout::new(fixture.0.join("allowed-source"))
        .save_level_dat(&valid_level_data)
        .unwrap();
    WorldLayout::new(fixture.0.join("forbidden-source"))
        .save_level_dat(&valid_level_data)
        .unwrap();

    fs::write(
        base.join("allowed_symlinks.txt"),
        format!("{}\n", allowed_target.display()),
    )
    .unwrap();
    let source = LevelStorageSource::create_default(&base).unwrap();
    fs::create_dir_all(base.join("allowed-link")).unwrap();
    fs::create_dir_all(base.join("forbidden-link")).unwrap();
    symlink(&allowed_target, base.join("allowed-link/level.dat")).unwrap();
    symlink(&forbidden_target, base.join("forbidden-link/level.dat")).unwrap();

    let summaries = source.load_level_summaries().unwrap();
    let by_name = summaries
        .into_iter()
        .map(|result| {
            let summary = result.unwrap();
            (summary.directory_name().to_owned(), summary)
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert!(matches!(
        by_name.get("allowed-link"),
        Some(crate::storage::world::summary::WorldSummary::Normal(_))
    ));
    assert!(matches!(
        by_name.get("forbidden-link"),
        Some(crate::storage::world::summary::WorldSummary::Symlink { .. })
    ));
}
