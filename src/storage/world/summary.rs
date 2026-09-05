//! World-summary compatibility, backup policy, and ordering from Java LevelSummary.
use super::*;

/// FileFixerUpper maps pre-4772 versions to zero; DataFixers registers its
/// latest file-fixer schema at 4773. This reports the need, not migration support.
pub(super) fn requires_file_fixing(version: i32) -> bool {
    version < 4773
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupStatus {
    None,
    Downgrade,
    UpgradeToSnapshot,
    FileFixingRequired,
}

impl BackupStatus {
    pub fn should_backup(self) -> bool {
        self != Self::None
    }
    pub fn is_severe(self) -> bool {
        self == Self::Downgrade
    }
    pub fn translation_key(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Downgrade => "downgrade",
            Self::UpgradeToSnapshot => "snapshot",
            Self::FileFixingRequired => "file_fixing_required",
        }
    }
}

impl LevelSummary {
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        other
            .version
            .last_played
            .cmp(&self.version.last_played)
            .then_with(|| {
                // String.compareTo uses UTF-16 code units, which differs from UTF-8
                // lexical order for supplementary characters versus high BMP values.
                self.directory_name
                    .encode_utf16()
                    .cmp(other.directory_name.encode_utf16())
            })
    }

    pub fn backup_status(&self) -> BackupStatus {
        self.backup_status_for(
            crate::storage::datafix::TARGET_DATA_VERSION,
            CURRENT_VERSION_SNAPSHOT,
        )
    }

    fn backup_status_for(&self, current_version: i32, snapshot: bool) -> BackupStatus {
        let saved_version = self.version.minecraft_version.id;
        if requires_file_fixing(saved_version) {
            BackupStatus::FileFixingRequired
        } else if snapshot && saved_version < current_version {
            BackupStatus::UpgradeToSnapshot
        } else if saved_version > current_version {
            BackupStatus::Downgrade
        } else {
            BackupStatus::None
        }
    }

    pub fn should_backup(&self) -> bool {
        self.backup_status().should_backup()
    }
    pub fn is_downgrade(&self) -> bool {
        self.backup_status() == BackupStatus::Downgrade
    }
    pub fn is_compatible(&self) -> bool {
        self.version.minecraft_version.series == CURRENT_VERSION_SERIES
    }
    pub fn is_disabled(&self) -> bool {
        self.locked || self.requires_manual_conversion || !self.is_compatible()
    }
    pub fn primary_action_active(&self) -> bool {
        !self.is_disabled()
    }
    pub fn primary_action_translation_key(&self) -> &'static str {
        if self.requires_file_fixing {
            "selectWorld.upgrade_and_play"
        } else {
            "selectWorld.select"
        }
    }
    pub fn can_upload(&self) -> bool {
        !self.requires_manual_conversion && !self.locked
    }
    pub fn can_edit(&self) -> bool {
        !self.is_disabled() && !self.requires_file_fixing
    }
    pub fn can_recreate(&self) -> bool {
        self.can_edit()
    }
    pub fn can_delete(&self) -> bool {
        true
    }

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
        if !matches!(version.level_data_version, 19132 | 19133) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Unknown data version: {:x}",
                    version.level_data_version as u32
                ),
            ));
        }
        let settings = level_settings::parse(data);
        let level_name = if settings.level_name.is_empty() {
            directory.directory_name()
        } else {
            settings.level_name
        };
        let requires_manual_conversion = version.level_data_version != 19133;
        let requires_file_fixing =
            summary::requires_file_fixing(version.data_version.unwrap_or(-1));

        Ok(Self {
            directory_name: directory.directory_name(),
            level_name,
            version,
            game_type: settings.game_type,
            hardcore: settings.difficulty.hardcore,
            cheats: settings.allow_commands,
            requires_manual_conversion,
            requires_file_fixing,
            icon_file: directory.icon_file(),
            locked,
        })
    }
}

#[cfg(test)]
mod tests;
