//! Exceptional world-list entries carry no fabricated game settings or versions.
use super::*;
use crate::chat_component::{Component, Style};

#[derive(Debug, Clone, PartialEq)]
pub enum WorldSummary {
    Normal(Box<LevelSummary>),
    Corrupted {
        directory_name: String,
        icon_file: PathBuf,
        last_played: i64,
    },
    Symlink {
        directory_name: String,
        icon_file: PathBuf,
    },
}

impl WorldSummary {
    pub fn normal(&self) -> Option<&LevelSummary> {
        match self {
            Self::Normal(summary) => Some(summary),
            _ => None,
        }
    }
    pub fn directory_name(&self) -> &str {
        match self {
            Self::Normal(summary) => &summary.directory_name,
            Self::Corrupted { directory_name, .. } | Self::Symlink { directory_name, .. } => {
                directory_name
            }
        }
    }
    pub fn level_name(&self) -> &str {
        self.normal()
            .map_or_else(|| self.directory_name(), |summary| &summary.level_name)
    }
    pub fn icon_file(&self) -> &Path {
        match self {
            Self::Normal(summary) => &summary.icon_file,
            Self::Corrupted { icon_file, .. } | Self::Symlink { icon_file, .. } => icon_file,
        }
    }
    pub fn last_played(&self) -> i64 {
        match self {
            Self::Normal(summary) => summary.version.last_played,
            Self::Corrupted { last_played, .. } => *last_played,
            Self::Symlink { .. } => -1,
        }
    }
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        compare_entries(
            self.last_played(),
            self.directory_name(),
            other.last_played(),
            other.directory_name(),
        )
    }
    pub fn is_disabled(&self) -> bool {
        self.normal().is_some_and(LevelSummary::is_disabled)
    }
    pub fn primary_action_active(&self) -> bool {
        !self.is_disabled()
    }
    pub fn can_upload(&self) -> bool {
        self.normal().is_some_and(LevelSummary::can_upload)
    }
    pub fn can_edit(&self) -> bool {
        self.normal().is_some_and(LevelSummary::can_edit)
    }
    pub fn can_recreate(&self) -> bool {
        self.normal().is_some_and(LevelSummary::can_recreate)
    }
    pub fn can_delete(&self) -> bool {
        true
    }
    pub fn primary_action_message(&self) -> Component {
        match self {
            Self::Normal(summary) => summary.primary_action_message(),
            Self::Corrupted { .. } => Component::translatable("recover_world.button", vec![]),
            Self::Symlink { .. } => Component::translatable("symlink_warning.more_info", vec![]),
        }
    }
    pub fn info(&self) -> Component {
        let key = match self {
            Self::Normal(summary) => return summary.info(),
            Self::Corrupted { .. } => "recover_world.warning",
            Self::Symlink { .. } => "symlink_warning.title",
        };
        Component::translatable(key, vec![]).styled(Style::empty().with_rgb_color(0xff0000))
    }
}

impl LevelDirectory {
    pub fn load_summary(&self) -> std::io::Result<WorldSummary> {
        let locked = self.layout().is_session_locked()?;
        let path = self.data_file();
        if path.exists() {
            // The current directory validator rejects every symlink. Allow-list
            // support is a separate storage task; never follow a forbidden link.
            if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.is_symlink()) {
                return Ok(WorldSummary::Symlink {
                    directory_name: self.directory_name(),
                    icon_file: self.icon_file(),
                });
            }
            if let Ok(root) = read_lightweight_data(&path) {
                // getCompoundOrEmpty("Data") does not accept legacy flat roots.
                if let Tag::Compound(fields) = &root {
                    if compound_tag(fields, "Data").is_some() {
                        if let Ok(summary) = LevelSummary::from_level_dat(self, &root, locked) {
                            return Ok(WorldSummary::Normal(Box::new(summary)));
                        }
                    }
                }
            }
        }
        // Recovery uses only file timestamps from the fallback, never its NBT.
        let last_played = modification_time(&path)
            .or_else(|| modification_time(&self.old_data_file()))
            .unwrap_or(-1);
        Ok(WorldSummary::Corrupted {
            directory_name: self.directory_name(),
            icon_file: self.icon_file(),
            last_played,
        })
    }
}

fn read_lightweight_data(path: &Path) -> std::io::Result<Tag> {
    use crate::storage::nbt::tag_access::{NbtFieldSelectorSpec, SkipFieldsVisitor};
    use crate::storage::nbt::tag_metadata::tag_type;
    let mut visitor = SkipFieldsVisitor::new(&[
        NbtFieldSelectorSpec::child("Data", tag_type(10), "Player"),
        NbtFieldSelectorSpec::child("Data", tag_type(10), "WorldGenSettings"),
    ]);
    crate::storage::nbt::nbt_io::parse_compressed_path(path, &mut visitor)?;
    visitor
        .get_result()
        .cloned()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing summary root"))
}

fn modification_time(path: &Path) -> Option<i64> {
    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()?;
    Some(chrono::DateTime::<chrono::Utc>::from(modified).timestamp_millis())
}

#[cfg(test)]
mod tests;
