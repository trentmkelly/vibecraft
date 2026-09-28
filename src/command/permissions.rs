use super::*;

pub fn command_required_permission(command: &str) -> PermissionLevel {
    match command {
        // `biome` is a VibeCraft-only debug command, not a Java parity command.
        // `version` is Java `checkPermissions ? LEVEL_GAMEMASTERS : LEVEL_ALL`;
        // a dedicated server (`CommandSelection != INTEGRATED`) checks permissions,
        // so it falls through to the gamemaster default below.
        "" | "biome" | "chase" | "help" | "list" | "me" | "msg" | "random" | "teammsg" | "tell"
        | "tm" | "trigger" | "w" => PermissionLevel::All,
        "ban" | "ban-ip" | "banlist" | "deop" | "debug" | "debugconfig" | "kick" | "op"
        | "pardon" | "pardon-ip" | "setidletimeout" | "tick" | "transfer" | "whitelist" => {
            PermissionLevel::Admins
        }
        "raid" => PermissionLevel::Admins,
        "jfr" | "perf" | "publish" | "save-all" | "save-off" | "save-on" | "stop" => {
            PermissionLevel::Owners
        }
        _ => PermissionLevel::Gamemasters,
    }
}
