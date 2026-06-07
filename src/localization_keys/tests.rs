use super::*;
use crate::presentation_data::{find_damage_type, FallVariant};

const INTENTIONALLY_INTERNAL_LOCALIZATION_KEYS: &[&str] = &[
    "block.minecraft.banner.guster",
    "death.attack.generic.item",
    "death.attack.fireball.player",
    "commands.chase.follow.success",
    "commands.chase.lead.success",
    "commands.chase.stop",
    "commands.datapack.list.success",
    "commands.debugconfig.config",
    "commands.debugconfig.dialog",
    "commands.debugconfig.missing",
    "commands.debugmobspawning.success",
    "commands.debugpath.success",
    "commands.help.success",
    "commands.me.success",
    "commands.message.display",
    "commands.raid.already_started",
    "commands.raid.check.success",
    "commands.raid.omen.changed",
    "commands.raid.omen.too_high",
    "commands.raid.spawnleader.success",
    "commands.raid.start.success",
    "commands.return.fail",
    "commands.return.run",
    "commands.return.success",
    "commands.rustcraft.debug.biome",
    "commands.say.success",
    "commands.serverpack.pop",
    "commands.serverpack.push",
    "commands.spawn_armor_trims.success",
    "commands.teammsg.success",
    "commands.tellraw.success",
    "commands.warden_spawn_tracker.clear.success.single",
    "commands.warden_spawn_tracker.set.success.single",
];

fn en_us_lang() -> &'static str {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../decompiled-server-26.1.2/assets/minecraft/lang/en_us.json"
    ))
}

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
fn catalog_covers_extended_disconnect_and_resource_pack_keys() {
    for key in [
        "multiplayer.disconnect.authservers_down",
        "multiplayer.disconnect.banned.reason",
        "multiplayer.disconnect.banned_ip.reason",
        "multiplayer.disconnect.duplicate_login",
        "multiplayer.disconnect.expired_public_key",
        "multiplayer.disconnect.flying",
        "multiplayer.disconnect.ip_banned",
        "multiplayer.disconnect.incompatible",
        "multiplayer.disconnect.invalid_player_movement",
        "multiplayer.disconnect.invalid_public_key_signature",
        "multiplayer.disconnect.invalid_vehicle_movement",
        "multiplayer.disconnect.outdated_client",
        "multiplayer.disconnect.server_shutdown",
        "multiplayer.disconnect.unverified_username",
        "resourcePack.server.name",
        "resourcePack.vanilla.name",
    ] {
        assert!(assert_known_emitted_key(key).is_ok(), "{key}");
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
    for key in [
        "chat.type.admin",
        "chat.type.team.sent",
        "chat.type.team.text",
        "chat.type.text.narrate",
        "chat.disabled.invalid_signature",
        "chat.disabled.missingProfileKey",
        "commands.save.disabled",
        "commands.save.enabled",
        "commands.version.header",
        "resourcePack.vanilla.description",
        "resourcepack.downloading",
    ] {
        assert!(assert_known_emitted_key(key).is_ok(), "{key}");
    }
    assert_eq!(
        find_key("sleep.players_sleeping").unwrap().args,
        ["sleeping", "needed"]
    );
}

#[test]
fn cataloged_keys_exist_in_en_us_or_are_intentionally_internal() {
    let en_us = en_us_lang();
    for entry in LOCALIZATION_KEYS.iter() {
        let in_en_us = en_us.contains(&format!("\"{}\":", entry.key));
        if in_en_us || INTENTIONALLY_INTERNAL_LOCALIZATION_KEYS.contains(&entry.key) {
            continue;
        }
        panic!(
            "localization key `{}` not found in en_us.json and not intentionally internal",
            entry.key
        );
    }
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
