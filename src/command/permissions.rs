use super::*;

pub fn command_required_permission(command: &str) -> PermissionLevel {
    match command {
        // `biome` is a VibeCraft-only debug command, not a Java parity command.
        "" | "biome" | "chase" | "help" | "list" | "me" | "msg" | "random" | "teammsg" | "tell"
        | "tm" | "trigger" | "w" | "version" => PermissionLevel::All,
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
