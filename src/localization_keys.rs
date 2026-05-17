#![allow(dead_code)]

use crate::presentation_data::{DamageTypeDef, FallVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKeyFamily {
    Chat,
    Command,
    Brigadier,
    Disconnect,
    Death,
    Sleep,
    ResourcePack,
    ServerStatus,
    RegistryDescription,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageKeyDef {
    pub key: &'static str,
    pub family: MessageKeyFamily,
    pub args: &'static [&'static str],
}

pub const LOCALIZATION_KEYS: &[MessageKeyDef] = &[
    key(
        "chat.type.text",
        MessageKeyFamily::Chat,
        &["sender", "content"],
    ),
    key(
        "chat.type.announcement",
        MessageKeyFamily::Chat,
        &["sender", "content"],
    ),
    key(
        "chat.type.emote",
        MessageKeyFamily::Chat,
        &["sender", "content"],
    ),
    key(
        "commands.message.display.incoming",
        MessageKeyFamily::Chat,
        &["sender", "content"],
    ),
    key(
        "commands.message.display.outgoing",
        MessageKeyFamily::Chat,
        &["target", "content"],
    ),
    key(
        "multiplayer.disconnect.kicked",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.banned",
        MessageKeyFamily::Disconnect,
        &["reason"],
    ),
    key(
        "multiplayer.disconnect.not_whitelisted",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.server_full",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.idling",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.chat_validation_failed",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "disconnect.exceeded_packet_rate",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "resourcePack.server.name",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "multiplayer.requiredTexturePrompt.disconnect",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "sleep.players_sleeping",
        MessageKeyFamily::Sleep,
        &["sleeping", "needed"],
    ),
    key("sleep.skipping_night", MessageKeyFamily::Sleep, &[]),
    key("death.attack.generic", MessageKeyFamily::Death, &["victim"]),
    key(
        "death.attack.generic.player",
        MessageKeyFamily::Death,
        &["victim", "attacker"],
    ),
    key(
        "death.attack.generic.item",
        MessageKeyFamily::Death,
        &["victim", "attacker", "item"],
    ),
    key(
        "death.attack.player",
        MessageKeyFamily::Death,
        &["victim", "attacker"],
    ),
    key(
        "death.attack.player.item",
        MessageKeyFamily::Death,
        &["victim", "attacker", "item"],
    ),
    key(
        "death.attack.fireball",
        MessageKeyFamily::Death,
        &["victim", "projectile"],
    ),
    key(
        "death.attack.fireball.player",
        MessageKeyFamily::Death,
        &["victim", "attacker"],
    ),
    key(
        "death.attack.fireball.item",
        MessageKeyFamily::Death,
        &["victim", "attacker", "item"],
    ),
    key(
        "death.attack.badRespawnPoint.message",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "death.fell.accident.ladder",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "death.fell.accident.vines",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "death.fell.accident.weeping_vines",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "death.fell.accident.twisting_vines",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "death.fell.accident.scaffolding",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "death.fell.accident.other_climbable",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "death.fell.accident.generic",
        MessageKeyFamily::Death,
        &["victim"],
    ),
    key(
        "argument.double.low",
        MessageKeyFamily::Brigadier,
        &["min", "found"],
    ),
    key(
        "argument.double.big",
        MessageKeyFamily::Brigadier,
        &["max", "found"],
    ),
    key(
        "argument.float.low",
        MessageKeyFamily::Brigadier,
        &["min", "found"],
    ),
    key(
        "argument.float.big",
        MessageKeyFamily::Brigadier,
        &["max", "found"],
    ),
    key(
        "argument.integer.low",
        MessageKeyFamily::Brigadier,
        &["min", "found"],
    ),
    key(
        "argument.integer.big",
        MessageKeyFamily::Brigadier,
        &["max", "found"],
    ),
    key(
        "argument.long.low",
        MessageKeyFamily::Brigadier,
        &["min", "found"],
    ),
    key(
        "argument.long.big",
        MessageKeyFamily::Brigadier,
        &["max", "found"],
    ),
    key(
        "argument.literal.incorrect",
        MessageKeyFamily::Brigadier,
        &["expected"],
    ),
    key(
        "parsing.quote.expected.start",
        MessageKeyFamily::Brigadier,
        &[],
    ),
    key(
        "parsing.quote.expected.end",
        MessageKeyFamily::Brigadier,
        &[],
    ),
    key(
        "parsing.quote.escape",
        MessageKeyFamily::Brigadier,
        &["character"],
    ),
    key(
        "parsing.bool.invalid",
        MessageKeyFamily::Brigadier,
        &["value"],
    ),
    key("parsing.bool.expected", MessageKeyFamily::Brigadier, &[]),
    key(
        "parsing.int.invalid",
        MessageKeyFamily::Brigadier,
        &["value"],
    ),
    key("parsing.int.expected", MessageKeyFamily::Brigadier, &[]),
    key(
        "parsing.long.invalid",
        MessageKeyFamily::Brigadier,
        &["value"],
    ),
    key("parsing.long.expected", MessageKeyFamily::Brigadier, &[]),
    key(
        "parsing.double.invalid",
        MessageKeyFamily::Brigadier,
        &["value"],
    ),
    key("parsing.double.expected", MessageKeyFamily::Brigadier, &[]),
    key(
        "parsing.float.invalid",
        MessageKeyFamily::Brigadier,
        &["value"],
    ),
    key("parsing.float.expected", MessageKeyFamily::Brigadier, &[]),
    key("parsing.expected", MessageKeyFamily::Brigadier, &["symbol"]),
    key("command.unknown.command", MessageKeyFamily::Brigadier, &[]),
    key("command.unknown.argument", MessageKeyFamily::Brigadier, &[]),
    key(
        "command.expected.separator",
        MessageKeyFamily::Brigadier,
        &[],
    ),
    key(
        "command.exception",
        MessageKeyFamily::Brigadier,
        &["message"],
    ),
    key("commands.help.failed", MessageKeyFamily::Command, &[]),
    key(
        "commands.list.players",
        MessageKeyFamily::Command,
        &["count", "max", "players"],
    ),
    key(
        "commands.kick.success",
        MessageKeyFamily::Command,
        &["player", "reason"],
    ),
    key(
        "commands.op.success",
        MessageKeyFamily::Command,
        &["player"],
    ),
    key(
        "commands.deop.success",
        MessageKeyFamily::Command,
        &["player"],
    ),
    key(
        "commands.ban.success",
        MessageKeyFamily::Command,
        &["player", "reason"],
    ),
    key(
        "commands.pardon.success",
        MessageKeyFamily::Command,
        &["player"],
    ),
    key(
        "commands.whitelist.add.success",
        MessageKeyFamily::Command,
        &["player"],
    ),
    key(
        "commands.whitelist.remove.success",
        MessageKeyFamily::Command,
        &["player"],
    ),
    key(
        "commands.gamemode.success.self",
        MessageKeyFamily::Command,
        &["mode"],
    ),
    key(
        "commands.gamemode.success.other",
        MessageKeyFamily::Command,
        &["player", "mode"],
    ),
    key(
        "commands.seed.success",
        MessageKeyFamily::Command,
        &["seed"],
    ),
    key("commands.save.saving", MessageKeyFamily::Command, &[]),
    key("commands.save.success", MessageKeyFamily::Command, &[]),
    key("commands.stop.stopping", MessageKeyFamily::Command, &[]),
    key("menu.online", MessageKeyFamily::ServerStatus, &[]),
    key(
        "trim_material.minecraft.resin",
        MessageKeyFamily::RegistryDescription,
        &[],
    ),
    key(
        "trim_pattern.minecraft.bolt",
        MessageKeyFamily::RegistryDescription,
        &[],
    ),
    key(
        "painting.minecraft.wither.title",
        MessageKeyFamily::RegistryDescription,
        &[],
    ),
    key(
        "block.minecraft.banner.guster",
        MessageKeyFamily::RegistryDescription,
        &[],
    ),
];

pub fn find_key(key_name: &str) -> Option<&'static MessageKeyDef> {
    LOCALIZATION_KEYS.iter().find(|entry| entry.key == key_name)
}

pub fn keys_in_family(family: MessageKeyFamily) -> Vec<&'static str> {
    LOCALIZATION_KEYS
        .iter()
        .filter(|entry| entry.family == family)
        .map(|entry| entry.key)
        .collect()
}

pub fn emitted_disconnect_key(reason: DisconnectReason) -> &'static str {
    match reason {
        DisconnectReason::Kicked => "multiplayer.disconnect.kicked",
        DisconnectReason::Banned => "multiplayer.disconnect.banned",
        DisconnectReason::NotWhitelisted => "multiplayer.disconnect.not_whitelisted",
        DisconnectReason::ServerFull => "multiplayer.disconnect.server_full",
        DisconnectReason::IdleTimeout => "multiplayer.disconnect.idling",
        DisconnectReason::ChatValidationFailed => "multiplayer.disconnect.chat_validation_failed",
        DisconnectReason::ExceededPacketRate => "disconnect.exceeded_packet_rate",
        DisconnectReason::RequiredResourcePackDeclined => {
            "multiplayer.requiredTexturePrompt.disconnect"
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReason {
    Kicked,
    Banned,
    NotWhitelisted,
    ServerFull,
    IdleTimeout,
    ChatValidationFailed,
    ExceededPacketRate,
    RequiredResourcePackDeclined,
}

pub fn emitted_death_key(
    damage_type: DamageTypeDef,
    has_attacker: bool,
    has_item: bool,
    fall_variant: Option<FallVariant>,
) -> String {
    match fall_variant {
        Some(variant) => damage_type
            .death_message_key(has_attacker, has_item, Some(variant))
            .to_string(),
        None => damage_type.typed_death_message_key(has_attacker, has_item),
    }
}

pub fn assert_known_emitted_key(key_name: &str) -> Result<&'static MessageKeyDef, String> {
    find_key(key_name).ok_or_else(|| format!("unknown emitted localization key: {key_name}"))
}

const fn key(
    key: &'static str,
    family: MessageKeyFamily,
    args: &'static [&'static str],
) -> MessageKeyDef {
    MessageKeyDef { key, family, args }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation_data::{find_damage_type, FallVariant};

    #[test]
    fn catalog_covers_disconnect_keys_emitted_by_login_play_and_management_paths() {
        for reason in [
            DisconnectReason::Kicked,
            DisconnectReason::Banned,
            DisconnectReason::NotWhitelisted,
            DisconnectReason::ServerFull,
            DisconnectReason::IdleTimeout,
            DisconnectReason::ChatValidationFailed,
            DisconnectReason::ExceededPacketRate,
            DisconnectReason::RequiredResourcePackDeclined,
        ] {
            let key = emitted_disconnect_key(reason);
            assert_eq!(
                assert_known_emitted_key(key).unwrap().family,
                if reason == DisconnectReason::RequiredResourcePackDeclined {
                    MessageKeyFamily::ResourcePack
                } else {
                    MessageKeyFamily::Disconnect
                }
            );
        }
    }

    #[test]
    fn catalog_covers_brigadier_parsing_keys_from_vanilla_exception_provider() {
        for key in [
            "argument.double.low",
            "argument.integer.big",
            "argument.literal.incorrect",
            "parsing.quote.expected.start",
            "parsing.bool.invalid",
            "parsing.int.expected",
            "command.unknown.command",
            "command.exception",
        ] {
            assert_eq!(
                assert_known_emitted_key(key).unwrap().family,
                MessageKeyFamily::Brigadier
            );
        }
    }

    #[test]
    fn catalog_covers_chat_sleep_and_common_command_feedback_keys() {
        for key in [
            "chat.type.text",
            "commands.message.display.incoming",
            "sleep.players_sleeping",
            "sleep.skipping_night",
            "commands.list.players",
            "commands.kick.success",
            "commands.gamemode.success.other",
            "commands.stop.stopping",
        ] {
            assert!(assert_known_emitted_key(key).is_ok(), "{key}");
        }
        assert_eq!(
            find_key("sleep.players_sleeping").unwrap().args,
            ["sleeping", "needed"]
        );
    }

    #[test]
    fn damage_type_message_keys_are_cataloged_for_default_item_and_fall_variants() {
        let fireball = *find_damage_type("minecraft:fireball").unwrap();
        let key = emitted_death_key(fireball, true, true, None);
        assert_eq!(key, "death.attack.fireball.item");
        assert_eq!(
            assert_known_emitted_key(&key).unwrap().family,
            MessageKeyFamily::Death
        );

        let fall = *find_damage_type("minecraft:fall").unwrap();
        let fall_key = emitted_death_key(fall, false, false, Some(FallVariant::Scaffolding));
        assert_eq!(fall_key, "death.fell.accident.scaffolding");
        assert!(assert_known_emitted_key(&fall_key).is_ok());
    }

    #[test]
    fn registry_description_keys_for_newer_assets_are_explicit() {
        for key in [
            "trim_material.minecraft.resin",
            "trim_pattern.minecraft.bolt",
            "painting.minecraft.wither.title",
            "block.minecraft.banner.guster",
        ] {
            assert_eq!(
                assert_known_emitted_key(key).unwrap().family,
                MessageKeyFamily::RegistryDescription
            );
        }
    }

    #[test]
    fn families_can_be_queried_without_unknown_or_duplicate_keys() {
        let mut keys = LOCALIZATION_KEYS
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), LOCALIZATION_KEYS.len());
        assert!(keys_in_family(MessageKeyFamily::Command).contains(&"commands.seed.success"));
        assert!(keys_in_family(MessageKeyFamily::Death).contains(&"death.attack.generic"));
    }
}
