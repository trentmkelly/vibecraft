//! Operator metadata edits use the primary file without datafixing or fallback.
use super::*;

impl LevelStorageAccess {
    pub fn rename_level(&self, new_name: &str) -> std::io::Result<()> {
        self.rename_metadata(new_name, false)
    }

    /// Rename a copied world and detach its singleplayer identity, as Java's
    /// LevelStorageAccess.renameAndDropPlayer does. Other player data is retained.
    pub fn rename_and_drop_player(&self, new_name: &str) -> std::io::Result<()> {
        self.rename_metadata(new_name, true)
    }

    fn rename_metadata(&self, new_name: &str, drop_player: bool) -> std::io::Result<()> {
        if !self.lock.is_valid() {
            return Err(std::io::Error::other("Lock is no longer valid"));
        }
        // Unlike normal world loading, a failed primary read must not promote
        // level.dat_old and replace the damaged primary as a side effect of rename.
        let (_, mut root) = read_gzip_named_tag_file(&self.level_directory.data_file())?;
        if let Tag::Compound(fields) = &mut root {
            if let Some((_, Tag::Compound(data))) = fields.iter_mut().find(|(key, _)| key == "Data")
            {
                // String.trim strips codepoints <= U+0020, not Unicode whitespace.
                put_compound_string(data, "LevelName", new_name.trim_matches(|c| c <= '\u{20}'));
                if drop_player {
                    data.retain(|(key, _)| key != "singleplayer_uuid");
                }
            }
        }
        // Java's getCompoundOrEmpty returns a detached empty compound for missing
        // or malformed Data. The original root is still saved without adding Data.
        self.save_level_data(&root)
    }
}

#[cfg(test)]
mod tests;
