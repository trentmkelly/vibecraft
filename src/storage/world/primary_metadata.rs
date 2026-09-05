//! Primary world save metadata. Loaded version information is retained for
//! inspection, but Java writes the running version when creating a new save.
use super::*;

impl PrimaryLevelData {
    /// Java stamps the current server version and wall-clock time on every save.
    pub fn to_level_dat(&self) -> Result<Tag, String> {
        self.to_level_dat_with_player_uuid(None)
    }

    /// Java createTag uses a supplied UUID for this save, falling back to the
    /// stored value for null without changing the loaded single-player identity.
    pub fn to_level_dat_with_player_uuid(
        &self,
        player_uuid: Option<crate::network::codec::Uuid>,
    ) -> Result<Tag, String> {
        self.encode_level_dat(chrono::Utc::now().timestamp_millis(), player_uuid)
    }

    fn to_level_dat_at(&self, epoch_millis: i64) -> Result<Tag, String> {
        self.encode_level_dat(epoch_millis, None)
    }

    fn encode_level_dat(
        &self,
        epoch_millis: i64,
        player_uuid: Option<crate::network::codec::Uuid>,
    ) -> Result<Tag, String> {
        let mut data = vec![
            (
                "DataVersion".to_string(),
                Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
            ),
            ("version".to_string(), Tag::Int(19133)),
            (
                "Version".to_string(),
                Tag::Compound(vec![
                    (
                        "Id".to_string(),
                        Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
                    ),
                    (
                        "Name".to_string(),
                        Tag::String(CURRENT_VERSION_NAME.to_owned()),
                    ),
                    (
                        "Series".to_string(),
                        Tag::String(CURRENT_VERSION_SERIES.to_owned()),
                    ),
                    (
                        "Snapshot".to_string(),
                        Tag::Byte(i8::from(CURRENT_VERSION_SNAPSHOT)),
                    ),
                ]),
            ),
            (
                "LevelName".to_string(),
                Tag::String(self.level_name.clone()),
            ),
            ("GameType".to_string(), Tag::Int(self.game_type.id())),
            (
                "difficulty_settings".to_string(),
                self.difficulty_settings.to_nbt(),
            ),
            ("DayTime".to_string(), Tag::Long(self.day_time)),
            ("Time".to_string(), Tag::Long(self.time)),
            ("LastPlayed".to_string(), Tag::Long(epoch_millis)),
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
            (
                "initialized".to_string(),
                Tag::Byte(i8::from(self.initialized)),
            ),
            (
                "WasModded".to_string(),
                Tag::Byte(i8::from(self.was_modded)),
            ),
            ("ScheduledEvents".to_string(), self.scheduled_events.clone()),
            (
                "ServerBrands".to_string(),
                string_list_tag(unique_server_brands(&self.server_brands).iter()),
            ),
            (
                "CustomBossEvents".to_string(),
                self.custom_boss_events.clone(),
            ),
            ("DragonFight".to_string(), self.dragon_fight.clone()),
            ("scoreboard".to_string(), self.scoreboard.clone()),
            ("GameRules".to_string(), self.game_rules.clone()),
        ];
        if let Some(uuid) = player_uuid.or(self.singleplayer_uuid) {
            data.push((
                "singleplayer_uuid".to_owned(),
                crate::storage::nbt::uuid_codec::uuid_to_nbt(uuid),
            ));
        }
        if !self.removed_features.is_empty() {
            data.push((
                "removed_features".to_owned(),
                string_list_tag(self.removed_features.iter()),
            ));
        }
        let registry = crate::registry::FeatureFlagRegistry::main_26_1_2()?;
        if let Tag::Compound(configuration) = self.data_configuration.to_nbt(&registry) {
            data.extend(configuration);
        }
        // Java CompoundTag.store calls getOrThrow: invalid values fail the save.
        data.push(("spawn".to_owned(), self.spawn.to_nbt()?));
        Ok(Tag::Compound(vec![(
            "Data".to_owned(),
            Tag::Compound(data),
        )]))
    }

    /// Mirrors setModdedInfo: brands are insertion-ordered and the flag is sticky.
    pub fn set_modded_info(&mut self, server_brand: impl Into<String>, is_modded: bool) {
        let brand = server_brand.into();
        if !self.server_brands.contains(&brand) {
            self.server_brands.push(brand);
        }
        self.was_modded |= is_modded;
    }

    pub fn known_server_brands(&self) -> Vec<String> {
        unique_server_brands(&self.server_brands)
    }
}

/// Java loads ServerBrands into a LinkedHashSet. Preserve first occurrence order
/// even for externally constructed Rust values containing duplicate brands.
pub(super) fn unique_server_brands(values: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    values
        .iter()
        .filter(|value| seen.insert(value.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests;
