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

// Representative subset of server-facing localization keys used by protocol and gameplay paths.
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
        "chat.type.admin",
        MessageKeyFamily::Chat,
        &["sender", "message"],
    ),
    key(
        "chat.type.team.sent",
        MessageKeyFamily::Chat,
        &["recipient", "sender", "content"],
    ),
    key(
        "chat.type.team.text",
        MessageKeyFamily::Chat,
        &["sender", "recipient", "content"],
    ),
    key(
        "chat.type.text.narrate",
        MessageKeyFamily::Chat,
        &["speaker", "message"],
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
        "multiplayer.disconnect.authservers_down",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.banned.reason",
        MessageKeyFamily::Disconnect,
        &["reason"],
    ),
    key(
        "multiplayer.disconnect.banned_ip.reason",
        MessageKeyFamily::Disconnect,
        &["reason"],
    ),
    key(
        "multiplayer.disconnect.duplicate_login",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.expired_public_key",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.flying",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.ip_banned",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.incompatible",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.invalid_player_movement",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.invalid_public_key_signature",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.invalid_vehicle_movement",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.outdated_client",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.server_shutdown",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "multiplayer.disconnect.unverified_username",
        MessageKeyFamily::Disconnect,
        &[],
    ),
    key(
        "chat.disabled.invalid_signature",
        MessageKeyFamily::Chat,
        &[],
    ),
    key(
        "chat.disabled.missingProfileKey",
        MessageKeyFamily::Chat,
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
        "resourcepack.downloading",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "resourcePack.broken_assets",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "resourcePack.high_contrast.name",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "resourcePack.load_fail",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "resourcePack.programmer_art.name",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key("resourcepack.progress", MessageKeyFamily::ResourcePack, &[]),
    key(
        "resourcepack.requesting",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "resourcePack.runtime_failure",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key("resourcePack.title", MessageKeyFamily::ResourcePack, &[]),
    key(
        "resourcePack.vanilla.description",
        MessageKeyFamily::ResourcePack,
        &[],
    ),
    key(
        "resourcePack.vanilla.name",
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
    key(
        "commands.attribute.base_value.get.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.attribute.base_value.reset.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.attribute.base_value.set.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.attribute.modifier.add.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.attribute.modifier.remove.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.attribute.modifier.value.get.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.attribute.value.get.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.banip.info", MessageKeyFamily::Command, &[]),
    key("commands.banlist.list", MessageKeyFamily::Command, &[]),
    key(
        "commands.bossbar.create.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.bossbar.get.max", MessageKeyFamily::Command, &[]),
    key("commands.bossbar.get.value", MessageKeyFamily::Command, &[]),
    key(
        "commands.bossbar.get.visible.visible",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.bossbar.list.bars.some",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.bossbar.remove.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.chase.follow.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.chase.lead.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.chase.stop", MessageKeyFamily::Command, &[]),
    key(
        "commands.clear.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.clear.test.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.clone.success", MessageKeyFamily::Command, &[]),
    key("commands.damage.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.datapack.create.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.datapack.list.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.datapack.modify.disable",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.datapack.modify.enable",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.debug.function.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.debug.started", MessageKeyFamily::Command, &[]),
    key("commands.debug.stopped", MessageKeyFamily::Command, &[]),
    key(
        "commands.debugconfig.config",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.debugconfig.dialog",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.debugconfig.missing",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.debugmobspawning.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.debugpath.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.defaultgamemode.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.dialog.clear.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.dialog.show.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.difficulty.query", MessageKeyFamily::Command, &[]),
    key(
        "commands.difficulty.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.drop.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.drop.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.effect.give.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.enchant.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.enchant.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.fetchprofile.id.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.fetchprofile.name.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.fill.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.fillbiome.success.count",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.forceload.added.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.forceload.list.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.forceload.query.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.forceload.removed.all",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.forceload.removed.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.function.scheduled.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.function.scheduled.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.gamerule.query", MessageKeyFamily::Command, &[]),
    key("commands.gamerule.set", MessageKeyFamily::Command, &[]),
    key(
        "commands.give.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.give.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.help.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.item.block.set.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.jfr.started", MessageKeyFamily::Command, &[]),
    key("commands.jfr.stopped", MessageKeyFamily::Command, &[]),
    key(
        "commands.kill.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.kill.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.locate.biome.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.locate.poi.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.locate.structure.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.me.success", MessageKeyFamily::Command, &[]),
    key("commands.message.display", MessageKeyFamily::Command, &[]),
    key("commands.pardonip.success", MessageKeyFamily::Command, &[]),
    key("commands.particle.success", MessageKeyFamily::Command, &[]),
    key("commands.perf.started", MessageKeyFamily::Command, &[]),
    key("commands.perf.stopped", MessageKeyFamily::Command, &[]),
    key(
        "commands.place.feature.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.place.jigsaw.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.place.structure.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.place.template.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.playsound.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.playsound.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.publish.started", MessageKeyFamily::Command, &[]),
    key(
        "commands.raid.already_started",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.raid.check.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.raid.omen.changed", MessageKeyFamily::Command, &[]),
    key(
        "commands.raid.omen.too_high",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.raid.spawnleader.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.raid.start.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.random.reset.all.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.random.reset.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.random.roll", MessageKeyFamily::Command, &[]),
    key(
        "commands.random.sample.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.recipe.give.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.recipe.give.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.recipe.take.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.reload.success", MessageKeyFamily::Command, &[]),
    key("commands.return.fail", MessageKeyFamily::Command, &[]),
    key("commands.return.run", MessageKeyFamily::Command, &[]),
    key("commands.return.success", MessageKeyFamily::Command, &[]),
    key("commands.say.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.ride.dismount.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.ride.mount.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.rotate.success", MessageKeyFamily::Command, &[]),
    key("commands.save.disabled", MessageKeyFamily::Command, &[]),
    key("commands.save.enabled", MessageKeyFamily::Command, &[]),
    key(
        "commands.schedule.cleared.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.schedule.created.function",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.schedule.created.tag",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.scoreboard.objectives.remove.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.serverpack.pop", MessageKeyFamily::Command, &[]),
    key("commands.serverpack.push", MessageKeyFamily::Command, &[]),
    key("commands.setblock.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.setidletimeout.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.setworldspawn.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.spawn_armor_trims.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.spawnpoint.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.spectate.success.started",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.spectate.success.stopped",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.spreadplayers.success.teams",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.stopwatch.create.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.stopwatch.query", MessageKeyFamily::Command, &[]),
    key(
        "commands.stopwatch.remove.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.stopwatch.restart.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.summon.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.swing.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.swing.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tag.add.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tag.list.multiple.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tag.list.single.empty",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tag.remove.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.team.add.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.team.empty.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.team.join.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.team.leave.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.team.list.members.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.teammsg.success", MessageKeyFamily::Command, &[]),
    key("commands.tellraw.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.team.remove.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.tick.rate.success", MessageKeyFamily::Command, &[]),
    key(
        "commands.tick.sprint.stop.fail",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tick.sprint.stop.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tick.status.frozen",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tick.status.lagging",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tick.status.running",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.tick.status.sprinting",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.tick.step.fail", MessageKeyFamily::Command, &[]),
    key(
        "commands.tick.step.stop.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.tick.step.success", MessageKeyFamily::Command, &[]),
    key("commands.time.pause", MessageKeyFamily::Command, &[]),
    key(
        "commands.time.query.absolute",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.time.query.gametime",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.time.query.timeline",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.time.rate", MessageKeyFamily::Command, &[]),
    key("commands.time.resume", MessageKeyFamily::Command, &[]),
    key("commands.time.set.absolute", MessageKeyFamily::Command, &[]),
    key(
        "commands.time.set.time_marker",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.title.show.title.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.title.times.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.transfer.success.multiple",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.transfer.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.trigger.add.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.trigger.set.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.trigger.simple.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.version.header", MessageKeyFamily::Command, &[]),
    key(
        "commands.warden_spawn_tracker.clear.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.warden_spawn_tracker.set.success.single",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.waypoint.list.empty",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.waypoint.list.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.waypoint.modify.color",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.waypoint.modify.color.reset",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.waypoint.modify.style",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.weather.set.clear", MessageKeyFamily::Command, &[]),
    key("commands.weather.set.rain", MessageKeyFamily::Command, &[]),
    key(
        "commands.weather.set.thunder",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.whitelist.disabled",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.whitelist.enabled", MessageKeyFamily::Command, &[]),
    key("commands.whitelist.list", MessageKeyFamily::Command, &[]),
    key("commands.whitelist.none", MessageKeyFamily::Command, &[]),
    key(
        "commands.whitelist.reloaded",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.worldborder.center.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.worldborder.damage.amount.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.worldborder.damage.buffer.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key("commands.worldborder.get", MessageKeyFamily::Command, &[]),
    key(
        "commands.worldborder.set.grow",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.worldborder.set.immediate",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.worldborder.set.shrink",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.worldborder.warning.distance.success",
        MessageKeyFamily::Command,
        &[],
    ),
    key(
        "commands.worldborder.warning.time.success",
        MessageKeyFamily::Command,
        &[],
    ),
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
        for entry in LOCALIZATION_KEYS {
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
}
