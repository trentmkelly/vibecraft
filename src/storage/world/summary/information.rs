//! Normal-world summary text uses the same component tree as LevelSummary.createInfo.
use super::*;
use crate::chat_component::{Component, ComponentArgument, Style};
use crate::chat_formatting::ChatFormatting;

pub(super) fn experimental_features(data: &[(String, Tag)]) -> std::io::Result<bool> {
    let Some((_, Tag::List(entries))) = data.iter().find(|(key, _)| key == "enabled_features")
    else {
        // Numeric arrays also form Dynamic streams, but contain no string IDs.
        return Ok(false);
    };
    let registry =
        crate::registry::FeatureFlagRegistry::main_26_1_2().map_err(std::io::Error::other)?;
    let vanilla = crate::registry::feature_flags::vanilla_set();
    // Summary parsing tolerates individual invalid and unknown IDs. Reusing the
    // strict feature-set codec would wrongly discard recognized experimental IDs.
    Ok(entries
        .iter()
        .filter_map(|entry| {
            let Tag::String(name) = entry else {
                return None;
            };
            let id = crate::registry::Identifier::parse(name).ok()?;
            registry.resolve_names(&[id]).ok()
        })
        .any(|features| !features.is_subset_of(vanilla)))
}

fn translated(key: &str) -> Component {
    Component::translatable(key, vec![])
}

impl LevelSummary {
    pub fn world_version_name(&self) -> Component {
        if self.version.minecraft_version_name.is_empty() {
            translated("selectWorld.versionUnknown")
        } else {
            Component::literal(&self.version.minecraft_version_name)
        }
    }

    pub fn primary_action_message(&self) -> Component {
        translated(self.primary_action_translation_key())
    }

    /// Build fresh text because the Rust summary fields remain publicly mutable.
    /// Caching Java's immutable-summary result here would return stale information.
    pub fn info(&self) -> Component {
        let red = Style::empty().apply_format(ChatFormatting::Red);
        if self.locked {
            return translated("selectWorld.locked").styled(red);
        }
        if self.requires_manual_conversion {
            return translated("selectWorld.conversion").styled(red);
        }
        if !self.is_compatible() {
            return Component::translatable(
                "selectWorld.incompatible.info",
                vec![ComponentArgument::Component(Box::new(
                    self.world_version_name(),
                ))],
            )
            .styled(red);
        }
        let mut result = if self.hardcore {
            Component::empty().append(
                translated("gameMode.hardcore").styled(Style::empty().with_rgb_color(0xff0000)),
            )
        } else {
            translated(match self.game_type {
                LevelGameType::Survival => "gameMode.survival",
                LevelGameType::Creative => "gameMode.creative",
                LevelGameType::Adventure => "gameMode.adventure",
                LevelGameType::Spectator => "gameMode.spectator",
            })
        };
        if self.cheats {
            result = result
                .append(Component::literal(", "))
                .append(translated("selectWorld.commands"));
        }
        if self.experimental {
            result = result.append(Component::literal(", ")).append(
                translated("selectWorld.experimental")
                    .styled(Style::empty().apply_format(ChatFormatting::Yellow)),
            );
        }
        let mut version_name = self.world_version_name();
        if self.should_backup() {
            version_name =
                version_name.styled(Style::empty().apply_format(if self.is_downgrade() {
                    ChatFormatting::Red
                } else {
                    ChatFormatting::Italic
                }));
        }
        result.append(
            Component::literal(", ")
                .append(translated("selectWorld.version"))
                .append(Component::literal(" "))
                .append(version_name),
        )
    }
}

#[cfg(test)]
mod tests;
