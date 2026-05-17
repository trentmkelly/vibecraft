#![allow(dead_code)]

use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::storage::nbt::{
    read_gzip_named_tag, read_named_tag, write_gzip_named_tag, write_named_tag, Tag,
};

use super::datafix::require_current_world_data_version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLayout {
    root: PathBuf,
}

impl WorldLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn level_dat(&self) -> PathBuf {
        self.root.join("level.dat")
    }

    pub fn level_dat_old(&self) -> PathBuf {
        self.root.join("level.dat_old")
    }

    pub fn session_lock(&self) -> PathBuf {
        self.root.join("session.lock")
    }

    pub fn region_dir(&self) -> PathBuf {
        self.root.join("region")
    }

    pub fn entities_dir(&self) -> PathBuf {
        self.root.join("entities")
    }

    pub fn poi_dir(&self) -> PathBuf {
        self.root.join("poi")
    }

    pub fn playerdata_dir(&self) -> PathBuf {
        self.root.join("playerdata")
    }

    pub fn advancements_dir(&self) -> PathBuf {
        self.root.join("advancements")
    }

    pub fn stats_dir(&self) -> PathBuf {
        self.root.join("stats")
    }

    pub fn datapacks_dir(&self) -> PathBuf {
        self.root.join("datapacks")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    pub fn player_data_file(&self, uuid: &str) -> PathBuf {
        self.playerdata_dir().join(format!("{uuid}.dat"))
    }

    pub fn player_data_old_file(&self, uuid: &str) -> PathBuf {
        self.playerdata_dir().join(format!("{uuid}.dat_old"))
    }

    pub fn advancements_file(&self, uuid: &str) -> PathBuf {
        self.advancements_dir().join(format!("{uuid}.json"))
    }

    pub fn stats_file(&self, uuid: &str) -> PathBuf {
        self.stats_dir().join(format!("{uuid}.json"))
    }

    pub fn saved_data_file(&self, name: &str) -> PathBuf {
        self.data_dir().join(format!("{name}.dat"))
    }

    pub fn map_data_file(&self, id: i32) -> PathBuf {
        self.saved_data_file(&format!("map_{id}"))
    }

    pub fn ensure_base_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)?;
        for dir in [
            self.region_dir(),
            self.entities_dir(),
            self.poi_dir(),
            self.playerdata_dir(),
            self.advancements_dir(),
            self.stats_dir(),
            self.datapacks_dir(),
            self.data_dir(),
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn validate_relative_path(&self, relative: &Path) -> std::io::Result<PathBuf> {
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "path escapes world root",
            ));
        }
        Ok(self.root.join(relative))
    }

    pub fn reject_symlink(&self, path: &Path) -> std::io::Result<()> {
        if fs::symlink_metadata(path)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "symlinks are not allowed",
            ))
        } else {
            Ok(())
        }
    }

    pub fn write_session_lock(&self, token: i64) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)?;
        fs::write(self.session_lock(), token.to_be_bytes())
    }

    pub fn read_session_lock(&self) -> std::io::Result<i64> {
        let bytes = fs::read(self.session_lock())?;
        if bytes.len() != 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "session.lock must contain one big-endian i64",
            ));
        }
        let mut token = [0u8; 8];
        token.copy_from_slice(&bytes);
        Ok(i64::from_be_bytes(token))
    }

    pub fn save_level_dat(&self, tag: &Tag) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)?;
        let mut bytes = Vec::new();
        write_named_tag(&mut bytes, "Data", tag)?;
        durable_write_with_backup(&self.level_dat(), Some(&self.level_dat_old()), &bytes)
    }

    pub fn load_level_dat(&self) -> std::io::Result<Tag> {
        self.load_level_dat_with_backup()
    }

    pub fn load_level_dat_with_backup(&self) -> std::io::Result<Tag> {
        match read_named_tag_file(&self.level_dat()) {
            Ok((_name, tag)) => Ok(tag),
            Err(primary_err) => match read_named_tag_file(&self.level_dat_old()) {
                Ok((_name, tag)) => Ok(tag),
                Err(_) => Err(primary_err),
            },
        }
    }

    pub fn load_level_dat_checked(&self) -> std::io::Result<Tag> {
        let tag = self.load_level_dat_with_backup()?;
        let data_version = data_version_from_level_dat(&tag).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "level.dat missing DataVersion",
            )
        })?;
        require_current_world_data_version(data_version)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        Ok(tag)
    }

    pub fn save_player_data(&self, uuid: &str, tag: &Tag) -> std::io::Result<()> {
        fs::create_dir_all(self.playerdata_dir())?;
        let mut bytes = Vec::new();
        write_gzip_named_tag(&mut bytes, "", tag)?;
        durable_write_with_backup(
            &self.player_data_file(uuid),
            Some(&self.player_data_old_file(uuid)),
            &bytes,
        )
    }

    pub fn load_player_data(&self, uuid: &str) -> std::io::Result<Tag> {
        match read_gzip_named_tag_file(&self.player_data_file(uuid)) {
            Ok((_name, tag)) => Ok(tag),
            Err(primary_err) => {
                self.backup_corrupt_player_data(uuid, ".dat")?;
                match read_gzip_named_tag_file(&self.player_data_old_file(uuid)) {
                    Ok((_name, tag)) => Ok(tag),
                    Err(_) => Err(primary_err),
                }
            }
        }
    }

    pub fn backup_corrupt_player_data(&self, uuid: &str, suffix: &str) -> std::io::Result<()> {
        let source = self.playerdata_dir().join(format!("{uuid}{suffix}"));
        if source.is_file() {
            let backup = self.playerdata_dir().join(format!(
                "{uuid}_corrupted_{}{}",
                corruption_backup_stamp(),
                suffix
            ));
            fs::copy(source, backup)?;
        }
        Ok(())
    }

    pub fn save_json_sidecar(&self, path: PathBuf, json: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        durable_write_with_backup(&path, None, json.as_bytes())
    }

    pub fn save_advancements(&self, uuid: &str, json: &str) -> std::io::Result<()> {
        self.save_json_sidecar(self.advancements_file(uuid), json)
    }

    pub fn load_advancements(&self, uuid: &str) -> std::io::Result<String> {
        fs::read_to_string(self.advancements_file(uuid))
    }

    pub fn save_stats(&self, uuid: &str, json: &str) -> std::io::Result<()> {
        self.save_json_sidecar(self.stats_file(uuid), json)
    }

    pub fn load_stats(&self, uuid: &str) -> std::io::Result<String> {
        fs::read_to_string(self.stats_file(uuid))
    }

    pub fn save_saved_data(&self, name: &str, tag: &Tag) -> std::io::Result<()> {
        fs::create_dir_all(self.data_dir())?;
        let mut bytes = Vec::new();
        write_named_tag(&mut bytes, "", tag)?;
        durable_write_with_backup(&self.saved_data_file(name), None, &bytes)
    }

    pub fn load_saved_data(&self, name: &str) -> std::io::Result<Tag> {
        let bytes = fs::read(self.saved_data_file(name))?;
        let (_name, tag) = read_named_tag(&mut bytes.as_slice())?;
        Ok(tag)
    }

    pub fn save_scoreboard(&self, tag: &Tag) -> std::io::Result<()> {
        self.save_saved_data("scoreboard", tag)
    }

    pub fn load_scoreboard(&self) -> std::io::Result<Tag> {
        self.load_saved_data("scoreboard")
    }

    pub fn save_raids(&self, dimension_suffix: &str, tag: &Tag) -> std::io::Result<()> {
        self.save_saved_data(&format!("raids{dimension_suffix}"), tag)
    }

    pub fn load_raids(&self, dimension_suffix: &str) -> std::io::Result<Tag> {
        self.load_saved_data(&format!("raids{dimension_suffix}"))
    }

    pub fn save_map_data(&self, id: i32, tag: &Tag) -> std::io::Result<()> {
        fs::create_dir_all(self.data_dir())?;
        let mut bytes = Vec::new();
        write_named_tag(&mut bytes, "", tag)?;
        durable_write_with_backup(&self.map_data_file(id), None, &bytes)
    }

    pub fn load_map_data(&self, id: i32) -> std::io::Result<Tag> {
        let bytes = fs::read(self.map_data_file(id))?;
        let (_name, tag) = read_named_tag(&mut bytes.as_slice())?;
        Ok(tag)
    }

    pub fn save_forced_chunks(&self, tag: &Tag) -> std::io::Result<()> {
        self.save_saved_data("forcedchunks", tag)
    }

    pub fn load_forced_chunks(&self) -> std::io::Result<Tag> {
        self.load_saved_data("forcedchunks")
    }

    pub fn save_command_storage(&self, namespace: &str, tag: &Tag) -> std::io::Result<()> {
        self.save_saved_data(&format!("command_storage_{namespace}"), tag)
    }

    pub fn load_command_storage(&self, namespace: &str) -> std::io::Result<Tag> {
        self.load_saved_data(&format!("command_storage_{namespace}"))
    }

    pub fn save_custom_bossbars(&self, tag: &Tag) -> std::io::Result<()> {
        self.save_saved_data("custom_boss_events", tag)
    }

    pub fn load_custom_bossbars(&self) -> std::io::Result<Tag> {
        self.load_saved_data("custom_boss_events")
    }

    pub fn save_random_sequences(&self, tag: &Tag) -> std::io::Result<()> {
        self.save_saved_data("random_sequences", tag)
    }

    pub fn load_random_sequences(&self) -> std::io::Result<Tag> {
        self.load_saved_data("random_sequences")
    }
}

fn data_version_from_level_dat(tag: &Tag) -> Option<i32> {
    let Tag::Compound(values) = tag else {
        return None;
    };
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            ("DataVersion", Tag::Int(version)) => Some(*version),
            ("Data", Tag::Compound(data_values)) => {
                data_values
                    .iter()
                    .find_map(|(name, value)| match (name.as_str(), value) {
                        ("DataVersion", Tag::Int(version)) => Some(*version),
                        _ => None,
                    })
            }
            _ => None,
        })
}

fn read_named_tag_file(path: &Path) -> std::io::Result<(String, Tag)> {
    let bytes = fs::read(path)?;
    read_named_tag(&mut bytes.as_slice())
}

fn read_gzip_named_tag_file(path: &Path) -> std::io::Result<(String, Tag)> {
    let bytes = fs::read(path)?;
    read_gzip_named_tag(bytes.as_slice())
}

fn corruption_backup_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}

fn durable_write_with_backup(
    target: &Path,
    backup: Option<&Path>,
    bytes: &[u8],
) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Some(backup) = backup {
        if target.exists() {
            fs::copy(target, backup)?;
        }
    }

    let tmp = target.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, target)
}

#[cfg(test)]
mod tests {
    use super::WorldLayout;
    use std::fs;

    #[test]
    fn exposes_vanilla_world_paths() {
        let layout = WorldLayout::new("world");
        assert_eq!(
            layout.level_dat(),
            std::path::PathBuf::from("world/level.dat")
        );
        assert_eq!(
            layout.level_dat_old(),
            std::path::PathBuf::from("world/level.dat_old")
        );
        assert_eq!(
            layout.session_lock(),
            std::path::PathBuf::from("world/session.lock")
        );
        assert_eq!(
            layout.region_dir(),
            std::path::PathBuf::from("world/region")
        );
        assert_eq!(
            layout.entities_dir(),
            std::path::PathBuf::from("world/entities")
        );
        assert_eq!(layout.poi_dir(), std::path::PathBuf::from("world/poi"));
        assert_eq!(
            layout.playerdata_dir(),
            std::path::PathBuf::from("world/playerdata")
        );
        assert_eq!(
            layout.advancements_dir(),
            std::path::PathBuf::from("world/advancements")
        );
        assert_eq!(layout.stats_dir(), std::path::PathBuf::from("world/stats"));
        assert_eq!(
            layout.datapacks_dir(),
            std::path::PathBuf::from("world/datapacks")
        );
        assert_eq!(layout.data_dir(), std::path::PathBuf::from("world/data"));
        assert_eq!(
            layout.player_data_file("uuid"),
            std::path::PathBuf::from("world/playerdata/uuid.dat")
        );
        assert_eq!(
            layout.advancements_file("uuid"),
            std::path::PathBuf::from("world/advancements/uuid.json")
        );
        assert_eq!(
            layout.stats_file("uuid"),
            std::path::PathBuf::from("world/stats/uuid.json")
        );
        assert_eq!(
            layout.saved_data_file("scoreboard"),
            std::path::PathBuf::from("world/data/scoreboard.dat")
        );
    }

    #[test]
    fn creates_base_dirs_and_round_trips_session_lock() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-world-layout-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        layout.ensure_base_dirs().unwrap();
        layout.write_session_lock(123456).unwrap();

        assert!(layout.region_dir().is_dir());
        assert!(layout.entities_dir().is_dir());
        assert_eq!(layout.read_session_lock().unwrap(), 123456);

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn saves_level_dat_and_rotates_old_copy() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-level-dat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let first = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(4790),
        )]);
        let second = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(4791),
        )]);

        layout.save_level_dat(&first).unwrap();
        assert_eq!(layout.load_level_dat().unwrap(), first);
        layout.save_level_dat(&second).unwrap();
        assert_eq!(layout.load_level_dat().unwrap(), second);
        assert!(layout.level_dat_old().is_file());

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn saves_player_data_and_json_sidecars() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-player-storage-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let uuid = "00000000-0000-0000-0000-000000000001";
        let player = crate::storage::nbt::Tag::Compound(vec![(
            "Health".to_string(),
            crate::storage::nbt::Tag::Float(20.0),
        )]);
        layout.save_player_data(uuid, &player).unwrap();
        layout.save_advancements(uuid, "{\"done\":true}").unwrap();
        layout
            .save_stats(uuid, "{\"minecraft:custom\":{}}")
            .unwrap();

        assert_eq!(layout.load_player_data(uuid).unwrap(), player);
        assert_eq!(layout.load_advancements(uuid).unwrap(), "{\"done\":true}");
        assert_eq!(
            layout.load_stats(uuid).unwrap(),
            "{\"minecraft:custom\":{}}"
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn player_data_uses_dat_old_and_corrupt_backup_like_vanilla_storage() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-player-corrupt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let uuid = "00000000-0000-0000-0000-000000000002";
        let first = crate::storage::nbt::Tag::Compound(vec![(
            "Pos".to_string(),
            crate::storage::nbt::Tag::List(vec![
                crate::storage::nbt::Tag::Double(1.0),
                crate::storage::nbt::Tag::Double(80.0),
                crate::storage::nbt::Tag::Double(1.0),
            ]),
        )]);
        let second = crate::storage::nbt::Tag::Compound(vec![(
            "Pos".to_string(),
            crate::storage::nbt::Tag::List(vec![
                crate::storage::nbt::Tag::Double(2.0),
                crate::storage::nbt::Tag::Double(81.0),
                crate::storage::nbt::Tag::Double(2.0),
            ]),
        )]);

        layout.save_player_data(uuid, &first).unwrap();
        layout.save_player_data(uuid, &second).unwrap();
        assert_eq!(layout.load_player_data(uuid).unwrap(), second);
        assert!(layout.player_data_old_file(uuid).is_file());

        fs::write(layout.player_data_file(uuid), b"corrupt playerdata").unwrap();
        assert_eq!(layout.load_player_data(uuid).unwrap(), first);
        let corrupt_backups = fs::read_dir(layout.playerdata_dir())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{uuid}_corrupted_"))
            })
            .count();
        assert_eq!(corrupt_backups, 1);

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn saves_vanilla_named_data_files() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-saved-data-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let tag = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(4790),
        )]);

        layout.save_scoreboard(&tag).unwrap();
        layout.save_raids("", &tag).unwrap();
        layout.save_map_data(0, &tag).unwrap();
        layout.save_forced_chunks(&tag).unwrap();
        layout.save_command_storage("minecraft", &tag).unwrap();
        layout.save_custom_bossbars(&tag).unwrap();
        layout.save_random_sequences(&tag).unwrap();

        assert_eq!(layout.load_scoreboard().unwrap(), tag);
        assert_eq!(layout.load_raids("").unwrap(), tag);
        assert_eq!(layout.load_map_data(0).unwrap(), tag);
        assert_eq!(layout.load_forced_chunks().unwrap(), tag);
        assert_eq!(layout.load_command_storage("minecraft").unwrap(), tag);
        assert_eq!(layout.load_custom_bossbars().unwrap(), tag);
        assert_eq!(layout.load_random_sequences().unwrap(), tag);
        assert!(layout.saved_data_file("scoreboard").is_file());
        assert!(layout.map_data_file(0).is_file());

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn validates_world_relative_paths_and_rejects_symlinks() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-path-safety-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let layout = WorldLayout::new(&path);
        assert_eq!(
            layout
                .validate_relative_path(std::path::Path::new("data/scoreboard.dat"))
                .unwrap(),
            path.join("data/scoreboard.dat")
        );
        assert!(layout
            .validate_relative_path(std::path::Path::new("../outside.dat"))
            .is_err());
        assert!(layout
            .validate_relative_path(std::path::Path::new("/outside.dat"))
            .is_err());

        let normal = path.join("normal.dat");
        fs::write(&normal, b"ok").unwrap();
        assert!(layout.reject_symlink(&normal).is_ok());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let target = path.join("target.dat");
            let link = path.join("link.dat");
            fs::write(&target, b"target").unwrap();
            symlink(&target, &link).unwrap();
            assert!(layout.reject_symlink(&link).is_err());
        }

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn falls_back_to_level_dat_old_when_primary_is_corrupt() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-level-corrupt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let first = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(4790),
        )]);
        let second = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(4791),
        )]);

        layout.save_level_dat(&first).unwrap();
        layout.save_level_dat(&second).unwrap();
        fs::write(layout.level_dat(), b"corrupt").unwrap();

        assert_eq!(layout.load_level_dat_with_backup().unwrap(), first);

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn refuses_world_metadata_when_primary_and_backup_are_corrupt() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "rustcraft-level-both-corrupt-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let layout = WorldLayout::new(&path);
        fs::write(layout.level_dat(), b"corrupt primary").unwrap();
        fs::write(layout.level_dat_old(), b"corrupt backup").unwrap();

        let err = layout.load_level_dat_with_backup().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn checked_level_dat_refuses_incompatible_or_missing_data_version() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-level-version-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let valid = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        )]);
        layout.save_level_dat(&valid).unwrap();
        assert_eq!(layout.load_level_dat_checked().unwrap(), valid);

        let invalid = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION - 1),
        )]);
        layout.save_level_dat(&invalid).unwrap();
        let err = layout.load_level_dat_checked().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("Unsupported world DataVersion"));

        let missing = crate::storage::nbt::Tag::Compound(vec![]);
        layout.save_level_dat(&missing).unwrap();
        let err = layout.load_level_dat_checked().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("missing DataVersion"));

        let _ = fs::remove_dir_all(&path);
    }
}
