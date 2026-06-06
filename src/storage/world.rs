#![allow(dead_code)]

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use crate::storage::nbt::{read_named_tag, write_gzip_named_tag, write_named_tag, Tag};
use crate::storage::region::{ChunkPos, RegionFile};

use super::datafix::require_current_world_data_version;

const SESSION_LOCK_MARKER: &[u8] = "\u{2603}".as_bytes();
const CURRENT_VERSION_NAME: &str = "26.1.2";
const CURRENT_VERSION_SERIES: &str = "main";
const CURRENT_VERSION_SNAPSHOT: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftDataVersion {
    pub id: i32,
    pub series: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelVersion {
    pub level_data_version: i32,
    pub data_version: Option<i32>,
    pub last_played: i64,
    pub minecraft_version_name: String,
    pub minecraft_version: MinecraftDataVersion,
    pub snapshot: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLevelData {
    pub data_version: i32,
    pub level_data_version: i32,
    pub version: LevelVersionInfo,
    pub level_name: String,
    pub spawn: LevelSpawnData,
    pub game_type: LevelGameType,
    pub difficulty: LevelDifficulty,
    pub day_time: i64,
    pub time: i64,
    pub generator_name: String,
    pub generator_settings: Tag,
    pub allow_commands: bool,
    pub hardcore: bool,
    pub initialized: bool,
    pub was_modded: bool,
    pub data_packs: DataPackSelection,
    pub scheduled_events: Tag,
    pub server_brands: Vec<String>,
    pub custom_boss_events: Tag,
    pub dragon_fight: Tag,
    pub scoreboard: Tag,
    pub game_rules: Tag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelVersionInfo {
    pub id: i32,
    pub name: String,
    pub series: String,
    pub snapshot: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelSpawnData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub angle: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPackSelection {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

impl PrimaryLevelData {
    pub fn from_level_dat(tag: &Tag) -> Option<Self> {
        let data = level_dat_data_compound(tag)?;
        let version = compound_tag(data, "Version")?;
        let data_packs = compound_tag(data, "DataPacks").unwrap_or(&[]);

        Some(Self {
            data_version: compound_i32(data, "DataVersion")?,
            level_data_version: compound_i32(data, "version").unwrap_or(19133),
            version: LevelVersionInfo {
                id: compound_i32(version, "Id")?,
                name: compound_string(version, "Name")?.to_string(),
                series: compound_string(version, "Series")
                    .unwrap_or(CURRENT_VERSION_SERIES)
                    .to_string(),
                snapshot: compound_bool(version, "Snapshot").unwrap_or(false),
            },
            level_name: compound_string(data, "LevelName")?.to_string(),
            spawn: LevelSpawnData {
                x: compound_i32(data, "SpawnX").unwrap_or_default(),
                y: compound_i32(data, "SpawnY").unwrap_or_default(),
                z: compound_i32(data, "SpawnZ").unwrap_or_default(),
                angle: compound_f32(data, "SpawnAngle").unwrap_or_default(),
            },
            game_type: LevelGameType::from_id(compound_i32(data, "GameType")?)?,
            difficulty: LevelDifficulty::from_id(compound_i8(data, "Difficulty")?)?,
            day_time: compound_i64(data, "DayTime").unwrap_or_default(),
            time: compound_i64(data, "Time").unwrap_or_default(),
            generator_name: compound_string(data, "generatorName")
                .unwrap_or("default")
                .to_string(),
            generator_settings: compound_clone(data, "generatorSettings")
                .unwrap_or_else(empty_compound_tag),
            allow_commands: compound_bool(data, "allowCommands").unwrap_or(false),
            hardcore: compound_bool(data, "hardcore").unwrap_or(false),
            initialized: compound_bool(data, "initialized").unwrap_or(true),
            was_modded: compound_bool(data, "WasModded").unwrap_or(false),
            data_packs: DataPackSelection {
                enabled: compound_string_list(data_packs, "Enabled"),
                disabled: compound_string_list(data_packs, "Disabled"),
            },
            scheduled_events: compound_clone(data, "ScheduledEvents")
                .unwrap_or_else(empty_list_tag),
            server_brands: compound_string_list(data, "ServerBrands"),
            custom_boss_events: compound_clone(data, "CustomBossEvents")
                .unwrap_or_else(empty_compound_tag),
            dragon_fight: compound_clone(data, "DragonFight").unwrap_or_else(empty_compound_tag),
            scoreboard: compound_clone(data, "scoreboard").unwrap_or_else(empty_compound_tag),
            game_rules: compound_clone(data, "GameRules").unwrap_or_else(empty_compound_tag),
        })
    }

    pub fn to_level_dat(&self) -> Tag {
        Tag::Compound(vec![(
            "Data".to_string(),
            Tag::Compound(vec![
                ("DataVersion".to_string(), Tag::Int(self.data_version)),
                ("version".to_string(), Tag::Int(self.level_data_version)),
                (
                    "Version".to_string(),
                    Tag::Compound(vec![
                        ("Id".to_string(), Tag::Int(self.version.id)),
                        ("Name".to_string(), Tag::String(self.version.name.clone())),
                        (
                            "Series".to_string(),
                            Tag::String(self.version.series.clone()),
                        ),
                        (
                            "Snapshot".to_string(),
                            Tag::Byte(i8::from(self.version.snapshot)),
                        ),
                    ]),
                ),
                (
                    "LevelName".to_string(),
                    Tag::String(self.level_name.clone()),
                ),
                ("SpawnX".to_string(), Tag::Int(self.spawn.x)),
                ("SpawnY".to_string(), Tag::Int(self.spawn.y)),
                ("SpawnZ".to_string(), Tag::Int(self.spawn.z)),
                ("SpawnAngle".to_string(), Tag::Float(self.spawn.angle)),
                ("GameType".to_string(), Tag::Int(self.game_type.id())),
                ("Difficulty".to_string(), Tag::Byte(self.difficulty.id())),
                ("DayTime".to_string(), Tag::Long(self.day_time)),
                ("Time".to_string(), Tag::Long(self.time)),
                (
                    "generatorName".to_string(),
                    Tag::String(self.generator_name.clone()),
                ),
                (
                    "generatorSettings".to_string(),
                    self.generator_settings.clone(),
                ),
                (
                    "allowCommands".to_string(),
                    Tag::Byte(i8::from(self.allow_commands)),
                ),
                ("hardcore".to_string(), Tag::Byte(i8::from(self.hardcore))),
                (
                    "initialized".to_string(),
                    Tag::Byte(i8::from(self.initialized)),
                ),
                (
                    "WasModded".to_string(),
                    Tag::Byte(i8::from(self.was_modded)),
                ),
                (
                    "DataPacks".to_string(),
                    Tag::Compound(vec![
                        (
                            "Enabled".to_string(),
                            string_list_tag(self.data_packs.enabled.iter()),
                        ),
                        (
                            "Disabled".to_string(),
                            string_list_tag(self.data_packs.disabled.iter()),
                        ),
                    ]),
                ),
                ("ScheduledEvents".to_string(), self.scheduled_events.clone()),
                (
                    "ServerBrands".to_string(),
                    string_list_tag(self.server_brands.iter()),
                ),
                (
                    "CustomBossEvents".to_string(),
                    self.custom_boss_events.clone(),
                ),
                ("DragonFight".to_string(), self.dragon_fight.clone()),
                ("scoreboard".to_string(), self.scoreboard.clone()),
                ("GameRules".to_string(), self.game_rules.clone()),
            ]),
        )])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLayout {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDataStorage {
    layout: WorldLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelDirectory {
    path: PathBuf,
}

impl LevelDirectory {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn directory_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string()
    }

    pub fn layout(&self) -> WorldLayout {
        WorldLayout::new(&self.path)
    }

    pub fn data_file(&self) -> PathBuf {
        self.layout().level_dat()
    }

    pub fn old_data_file(&self) -> PathBuf {
        self.layout().level_dat_old()
    }

    pub fn icon_file(&self) -> PathBuf {
        self.path.join("icon.png")
    }

    pub fn lock_file(&self) -> PathBuf {
        self.layout().session_lock()
    }

    pub fn load_summary(&self) -> std::io::Result<LevelSummary> {
        let layout = self.layout();
        let tag = layout.load_level_dat_with_backup()?;
        LevelSummary::from_level_dat(self, &tag, layout.is_session_locked()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelCandidates {
    levels: Vec<LevelDirectory>,
}

impl LevelCandidates {
    pub fn new(levels: Vec<LevelDirectory>) -> Self {
        Self { levels }
    }

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    pub fn levels(&self) -> &[LevelDirectory] {
        &self.levels
    }

    pub fn summaries(&self) -> Vec<std::io::Result<LevelSummary>> {
        self.levels
            .iter()
            .map(LevelDirectory::load_summary)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelSummary {
    pub directory_name: String,
    pub level_name: String,
    pub version: LevelVersion,
    pub game_type: LevelGameType,
    pub hardcore: bool,
    pub cheats: bool,
    pub requires_manual_conversion: bool,
    pub icon_file: PathBuf,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelStorageSource {
    base_dir: PathBuf,
    backup_dir: PathBuf,
}

impl LevelStorageSource {
    pub fn create_default(base_dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let base_dir = base_dir.into();
        fs::create_dir_all(&base_dir)?;
        let backup_dir = base_dir
            .parent()
            .map(|parent| parent.join("backups"))
            .unwrap_or_else(|| PathBuf::from("backups"));
        Ok(Self {
            base_dir,
            backup_dir,
        })
    }

    pub fn new(
        base_dir: impl Into<PathBuf>,
        backup_dir: impl Into<PathBuf>,
    ) -> std::io::Result<Self> {
        let base_dir = base_dir.into();
        fs::create_dir_all(&base_dir)?;
        Ok(Self {
            base_dir,
            backup_dir: backup_dir.into(),
        })
    }

    pub fn name(&self) -> &'static str {
        "Anvil"
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    pub fn level_path(&self, level_id: &str) -> std::io::Result<PathBuf> {
        validate_level_id(level_id)?;
        Ok(self.base_dir.join(level_id))
    }

    pub fn level_exists(&self, level_id: &str) -> bool {
        self.level_path(level_id)
            .map(|path| path.is_dir())
            .unwrap_or(false)
    }

    pub fn is_new_level_id_acceptable(&self, level_id: &str) -> bool {
        let Ok(path) = self.level_path(level_id) else {
            return false;
        };
        match fs::create_dir(&path) {
            Ok(()) => fs::remove_dir(&path).is_ok(),
            Err(_) => false,
        }
    }

    pub fn find_level_candidates(&self) -> std::io::Result<LevelCandidates> {
        let mut levels = Vec::new();
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let level = LevelDirectory::new(path);
                if level.data_file().is_file() || level.old_data_file().is_file() {
                    levels.push(level);
                }
            }
        }
        levels.sort_by_key(LevelDirectory::directory_name);
        Ok(LevelCandidates::new(levels))
    }

    pub fn load_level_summaries(&self) -> std::io::Result<Vec<std::io::Result<LevelSummary>>> {
        Ok(self.find_level_candidates()?.summaries())
    }

    pub fn validate_and_create_access(
        &self,
        level_id: &str,
    ) -> std::io::Result<LevelStorageAccess> {
        let path = self.level_path(level_id)?;
        reject_symlinks_recursive(&path)?;
        self.create_access(level_id)
    }

    pub fn create_access(&self, level_id: &str) -> std::io::Result<LevelStorageAccess> {
        let path = self.level_path(level_id)?;
        LevelStorageAccess::open(level_id.to_string(), path, self.backup_dir.clone())
    }
}

#[derive(Debug)]
pub struct LevelStorageAccess {
    level_id: String,
    level_directory: LevelDirectory,
    backup_dir: PathBuf,
    lock: SessionLock,
}

impl LevelStorageAccess {
    fn open(level_id: String, path: PathBuf, backup_dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&path)?;
        let layout = WorldLayout::new(&path);
        let lock = layout.acquire_session_lock()?;
        Ok(Self {
            level_id,
            level_directory: LevelDirectory::new(path),
            backup_dir,
            lock,
        })
    }

    pub fn level_id(&self) -> &str {
        &self.level_id
    }

    pub fn level_directory(&self) -> &LevelDirectory {
        &self.level_directory
    }

    pub fn layout(&self) -> WorldLayout {
        self.level_directory.layout()
    }

    pub fn get_dimension_path(&self, dimension_id: &str) -> std::io::Result<PathBuf> {
        self.layout().dimension_path(dimension_id)
    }

    pub fn read_level_data(&self) -> std::io::Result<Tag> {
        self.layout().load_level_dat_with_backup()
    }

    pub fn save_level_data(&self, tag: &Tag) -> std::io::Result<()> {
        self.layout().save_level_dat(tag)
    }

    pub fn has_world_data(&self) -> bool {
        self.level_directory.data_file().is_file() || self.level_directory.old_data_file().is_file()
    }

    pub fn rename_level(&self, new_name: &str) -> std::io::Result<()> {
        let mut tag = self.read_level_data()?;
        put_level_name(&mut tag, new_name.trim());
        self.save_level_data(&tag)
    }

    // TODO(world-backup-zip-format): NOT 1:1 with Java
    // LevelStorageSource.makeWorldBackup (lines 673-686), which writes a single
    // ZIP archive `backups/<FileNameDateFormatter>_<levelId>.zip` (via
    // ZipOutputStream + findAvailableName for collisions, excluding session.lock).
    // This currently copies the world into a backups/ subdirectory instead of
    // zipping it — the world is preserved but the on-disk artifact differs.
    // Completing CHECKLIST_STORAGE #28 requires producing the vanilla .zip
    // (needs a zip writer) with FileNameDateFormatter naming.
    pub fn make_world_backup(&self) -> std::io::Result<PathBuf> {
        fs::create_dir_all(&self.backup_dir)?;
        let backup_path = self.backup_dir.join(format!(
            "{}_{}",
            corruption_backup_stamp(),
            sanitize_backup_name(&self.level_id)
        ));
        copy_dir_recursive(
            self.level_directory.path(),
            &backup_path,
            Some("session.lock"),
        )?;
        Ok(backup_path)
    }

    pub fn delete_level(self) -> std::io::Result<()> {
        let Self {
            level_directory,
            lock,
            ..
        } = self;
        let lock_path = level_directory.lock_file();
        drop(lock);
        if level_directory.path().exists() {
            remove_dir_recursive_except(level_directory.path(), &lock_path)?;
            let _ = fs::remove_file(lock_path);
            let _ = fs::remove_dir(level_directory.path());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelGameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl LevelGameType {
    pub fn id(self) -> i32 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Survival),
            1 => Some(Self::Creative),
            2 => Some(Self::Adventure),
            3 => Some(Self::Spectator),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl LevelDifficulty {
    pub fn id(self) -> i8 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }

    pub fn from_id(id: i8) -> Option<Self> {
        match id {
            0 => Some(Self::Peaceful),
            1 => Some(Self::Easy),
            2 => Some(Self::Normal),
            3 => Some(Self::Hard),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelRespawnData {
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub angle: f32,
}

impl Default for LevelRespawnData {
    fn default() -> Self {
        Self {
            dimension: "minecraft:overworld".to_string(),
            x: 0,
            y: 0,
            z: 0,
            angle: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldDataView {
    pub level_name: String,
    pub game_type: LevelGameType,
    pub hardcore: bool,
    pub allow_commands: bool,
    pub difficulty: LevelDifficulty,
    pub difficulty_locked: bool,
    pub seed: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerLevelDataView {
    pub respawn_data: LevelRespawnData,
    pub game_time: i64,
    pub initialized: bool,
    pub seed: i64,
}

pub type WorldData = WorldDataView;
pub type ServerLevelData = ServerLevelDataView;

#[derive(Debug, Clone, PartialEq)]
pub struct DerivedLevelData {
    world_data: WorldDataView,
    wrapped: ServerLevelDataView,
}

impl DerivedLevelData {
    pub fn new(world_data: WorldDataView, wrapped: ServerLevelDataView) -> Self {
        Self {
            world_data,
            wrapped,
        }
    }

    pub fn respawn_data(&self) -> &LevelRespawnData {
        &self.wrapped.respawn_data
    }

    pub fn set_spawn(&mut self, respawn_data: LevelRespawnData) {
        self.wrapped.respawn_data = respawn_data;
    }

    pub fn game_time(&self) -> i64 {
        self.wrapped.game_time
    }

    pub fn set_game_time(&mut self, _time: i64) {
        // Java DerivedLevelData.setGameTime is intentionally a no-op.
    }

    pub fn level_name(&self) -> &str {
        &self.world_data.level_name
    }

    pub fn game_type(&self) -> LevelGameType {
        self.world_data.game_type
    }

    pub fn set_game_type(&mut self, _game_type: LevelGameType) {
        // Java DerivedLevelData.setGameType is intentionally a no-op.
    }

    pub fn is_hardcore(&self) -> bool {
        self.world_data.hardcore
    }

    pub fn allow_commands(&self) -> bool {
        self.world_data.allow_commands
    }

    pub fn initialized(&self) -> bool {
        self.wrapped.initialized
    }

    pub fn set_initialized(&mut self, _initialized: bool) {
        // Java DerivedLevelData.setInitialized is intentionally a no-op.
    }

    pub fn difficulty(&self) -> LevelDifficulty {
        self.world_data.difficulty
    }

    pub fn difficulty_locked(&self) -> bool {
        self.world_data.difficulty_locked
    }

    pub fn world_seed(&self) -> i64 {
        self.world_data.seed
    }

    pub fn dimension_seed(&self) -> i64 {
        self.wrapped.seed
    }
}

impl LevelSummary {
    pub fn from_level_dat(
        directory: &LevelDirectory,
        tag: &Tag,
        locked: bool,
    ) -> std::io::Result<Self> {
        let data = level_dat_data_compound(tag).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "level.dat root is not compound",
            )
        })?;
        let version = LevelVersion::parse_level_dat(tag).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "level.dat missing version data",
            )
        })?;
        let level_name = compound_string(data, "LevelName")
            .map(str::to_string)
            .unwrap_or_else(|| directory.directory_name());
        let game_type = compound_i32(data, "GameType")
            .and_then(LevelGameType::from_id)
            .unwrap_or(LevelGameType::Survival);

        Ok(Self {
            directory_name: directory.directory_name(),
            level_name,
            version,
            game_type,
            hardcore: compound_bool(data, "hardcore").unwrap_or(false),
            cheats: compound_bool(data, "allowCommands").unwrap_or(false),
            requires_manual_conversion: compound_bool(data, "requiresManualConversion")
                .unwrap_or(false),
            icon_file: directory.icon_file(),
            locked,
        })
    }
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

    // TODO(dimension-folder-layout-26.1.2): PRE-26.1.2 layout. Java 26.1.2 stores
    // EVERY dimension (incl. overworld) under `dimensions/<ns>/<path>/` —
    // ChunkMap.java:174 uses `getDimensionPath(dim).resolve("region")`,
    // getDimensionPath = DimensionType.getStorageFolder = `<root>/dimensions/<ns>/
    // <path>` (no DIM-1/DIM1 special-casing). So overworld region belongs at
    // `dimensions/minecraft/overworld/{region,entities,poi}`. Deferred (foundational
    // + needs migration). Blocks STORAGE #110/#28/#29/#117. See memory note.
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

    pub fn dimensions_dir(&self) -> PathBuf {
        self.root.join("dimensions")
    }

    pub fn dimension_path(&self, dimension_id: &str) -> std::io::Result<PathBuf> {
        let (namespace, path) = dimension_id
            .split_once(':')
            .unwrap_or(("minecraft", dimension_id));
        validate_resource_location_namespace(namespace)?;
        validate_resource_location_path(path)?;
        Ok(self.dimensions_dir().join(namespace).join(path))
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
            self.dimensions_dir(),
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

    pub fn acquire_session_lock(&self) -> std::io::Result<SessionLock> {
        SessionLock::acquire(&self.root)
    }

    pub fn is_session_locked(&self) -> std::io::Result<bool> {
        SessionLock::is_locked(&self.root)
    }

    pub fn save_level_dat(&self, tag: &Tag) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)?;
        let mut bytes = Vec::new();
        // Java LevelStorageSource writes level.dat via NbtIo.writeCompressed: a
        // gzip-compressed root compound (empty name) whose "Data" child holds the
        // level data. `tag` is already `{ "Data": <leveldata> }` (see to_level_dat),
        // so we gzip it under the empty root name to match vanilla exactly.
        write_gzip_named_tag(&mut bytes, "", tag)?;
        durable_write_with_backup(&self.level_dat(), Some(&self.level_dat_old()), &bytes)
    }

    pub fn load_level_dat(&self) -> std::io::Result<Tag> {
        self.load_level_dat_with_backup()
    }

    pub fn load_level_dat_with_backup(&self) -> std::io::Result<Tag> {
        match read_level_dat_file(&self.level_dat()) {
            Ok((_name, tag)) => Ok(tag),
            Err(primary_err) => match read_level_dat_file(&self.level_dat_old()) {
                Ok((_name, tag)) => Ok(tag),
                Err(_) => Err(primary_err),
            },
        }
    }

    pub fn load_level_dat_checked(&self) -> std::io::Result<Tag> {
        let tag = self.load_level_dat_with_backup()?;
        let level_version = LevelVersion::parse_level_dat(&tag).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "level.dat missing DataVersion",
            )
        })?;
        let data_version = level_version.data_version.ok_or_else(|| {
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
        let tag = tag_with_data_version(tag);
        write_gzip_named_tag(&mut bytes, "", &tag)?;
        durable_write_with_backup(
            &self.player_data_file(uuid),
            Some(&self.player_data_old_file(uuid)),
            &bytes,
        )
    }

    pub fn load_player_data(&self, uuid: &str) -> std::io::Result<Tag> {
        match read_gzip_named_tag_file(&self.player_data_file(uuid)) {
            Ok((_name, tag)) => checked_saved_tag("playerdata", tag),
            Err(primary_err) => {
                // Java catches backup failures silently (logs warning, continues).
                let _ = self.backup_corrupt_player_data(uuid, ".dat");
                match read_gzip_named_tag_file(&self.player_data_old_file(uuid)) {
                    Ok((_name, tag)) => checked_saved_tag("playerdata backup", tag),
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
        self.save_json_sidecar(
            self.advancements_file(uuid),
            &json_with_data_version("advancements", json)?,
        )
    }

    pub fn load_advancements(&self, uuid: &str) -> std::io::Result<String> {
        let json = fs::read_to_string(self.advancements_file(uuid))?;
        checked_json_data_version("advancements", &json)?;
        Ok(json)
    }

    pub fn save_stats(&self, uuid: &str, json: &str) -> std::io::Result<()> {
        self.save_json_sidecar(
            self.stats_file(uuid),
            &json_with_data_version("stats", json)?,
        )
    }

    pub fn load_stats(&self, uuid: &str) -> std::io::Result<String> {
        let json = fs::read_to_string(self.stats_file(uuid))?;
        checked_json_data_version("stats", &json)?;
        Ok(json)
    }

    pub fn save_saved_data(&self, name: &str, tag: &Tag) -> std::io::Result<()> {
        fs::create_dir_all(self.data_dir())?;
        let mut bytes = Vec::new();
        let tag = tag_with_data_version(tag);
        write_named_tag(&mut bytes, "", &tag)?;
        durable_write_with_backup(&self.saved_data_file(name), None, &bytes)
    }

    pub fn load_saved_data(&self, name: &str) -> std::io::Result<Tag> {
        let bytes = fs::read(self.saved_data_file(name))?;
        let (_name, tag) = read_named_tag(&mut bytes.as_slice())?;
        checked_saved_tag(name, tag)
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
        let tag = tag_with_data_version(tag);
        write_named_tag(&mut bytes, "", &tag)?;
        durable_write_with_backup(&self.map_data_file(id), None, &bytes)
    }

    pub fn load_map_data(&self, id: i32) -> std::io::Result<Tag> {
        let bytes = fs::read(self.map_data_file(id))?;
        let (_name, tag) = read_named_tag(&mut bytes.as_slice())?;
        checked_saved_tag(&format!("map_{id}"), tag)
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

    pub fn save_entity_region_chunk(&self, pos: ChunkPos, tag: &Tag) -> std::io::Result<()> {
        let region = RegionFile::open(&self.entities_dir(), pos.region())?;
        region.write_chunk_nbt(pos, "", tag)
    }

    pub fn load_entity_region_chunk(&self, pos: ChunkPos) -> std::io::Result<Option<Tag>> {
        let region = RegionFile::open(&self.entities_dir(), pos.region())?;
        region
            .read_chunk_nbt(pos)
            .map(|chunk| chunk.map(|(_name, tag)| tag))
    }

    pub fn save_poi_region_chunk(&self, pos: ChunkPos, tag: &Tag) -> std::io::Result<()> {
        let region = RegionFile::open(&self.poi_dir(), pos.region())?;
        region.write_chunk_nbt(pos, "", tag)
    }

    pub fn load_poi_region_chunk(&self, pos: ChunkPos) -> std::io::Result<Option<Tag>> {
        let region = RegionFile::open(&self.poi_dir(), pos.region())?;
        region
            .read_chunk_nbt(pos)
            .map(|chunk| chunk.map(|(_name, tag)| tag))
    }
}

impl PlayerDataStorage {
    pub fn new(layout: WorldLayout) -> Self {
        Self { layout }
    }

    pub fn player_data_file(&self, uuid: &str) -> PathBuf {
        self.layout.player_data_file(uuid)
    }

    pub fn player_data_old_file(&self, uuid: &str) -> PathBuf {
        self.layout.player_data_old_file(uuid)
    }

    pub fn save(&self, uuid: &str, tag: &Tag) -> std::io::Result<()> {
        self.layout.save_player_data(uuid, tag)
    }

    pub fn load(&self, uuid: &str) -> std::io::Result<Tag> {
        self.layout.load_player_data(uuid)
    }

    pub fn backup_corrupt_player_data(&self, uuid: &str, suffix: &str) -> std::io::Result<()> {
        self.layout.backup_corrupt_player_data(uuid, suffix)
    }
}

impl LevelVersion {
    pub fn parse_level_dat(tag: &Tag) -> Option<Self> {
        let data = level_dat_data_compound(tag)?;
        let data_version = compound_i32(data, "DataVersion");
        let version = compound_tag(data, "Version");
        Some(Self {
            level_data_version: compound_i32(data, "version").unwrap_or(0),
            data_version,
            last_played: compound_i64(data, "LastPlayed").unwrap_or(0),
            minecraft_version_name: version
                .and_then(|version| compound_string(version, "Name"))
                .unwrap_or(CURRENT_VERSION_NAME)
                .to_string(),
            minecraft_version: MinecraftDataVersion {
                id: version
                    .and_then(|version| compound_i32(version, "Id"))
                    .unwrap_or(crate::storage::datafix::TARGET_DATA_VERSION),
                series: version
                    .and_then(|version| compound_string(version, "Series"))
                    .unwrap_or(CURRENT_VERSION_SERIES)
                    .to_string(),
            },
            snapshot: version
                .and_then(|version| compound_bool(version, "Snapshot"))
                .unwrap_or(CURRENT_VERSION_SNAPSHOT),
        })
    }
}

#[derive(Debug)]
pub struct SessionLock {
    file: File,
}

impl SessionLock {
    pub fn acquire(dir: &Path) -> std::io::Result<Self> {
        fs::create_dir_all(dir)?;
        let lock_path = dir.join("session.lock");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&lock_path)?;
        file.write_all(SESSION_LOCK_MARKER)?;
        file.sync_all()?;

        lock_file_exclusive_nonblocking(&file, &lock_path)?;
        Ok(Self { file })
    }

    pub fn is_locked(dir: &Path) -> std::io::Result<bool> {
        let lock_path = dir.join("session.lock");
        let file = match OpenOptions::new().write(true).open(&lock_path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return Ok(true),
            Err(err) => return Err(err),
        };

        match try_lock_file_exclusive_nonblocking(&file) {
            Ok(()) => {
                unlock_file(&file)?;
                Ok(false)
            }
            Err(err) if is_would_block_lock_error(&err) => Ok(true),
            Err(err) => Err(err),
        }
    }
}

impl Drop for SessionLock {
    fn drop(&mut self) {
        let _ = unlock_file(&self.file);
    }
}

mod helpers;
use helpers::*;

#[cfg(test)]
mod tests;
