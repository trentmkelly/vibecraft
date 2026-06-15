#![allow(dead_code)]

use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PardonIpError {
    Invalid,
    NotBanned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PardonIpOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub removed_ip: String,
    pub broadcast_to_admins: bool,
}

pub fn pardon_ip(target: &str, banned_ips: &[String]) -> Result<PardonIpOutput, PardonIpError> {
    if !is_inet_address(target) {
        return Err(PardonIpError::Invalid);
    }

    if !banned_ips.iter().any(|ip| ip == target) {
        return Err(PardonIpError::NotBanned);
    }

    Ok(PardonIpOutput {
        success_count: 1,
        feedback_key: "commands.pardonip.success",
        removed_ip: target.to_string(),
        broadcast_to_admins: true,
    })
}

fn is_inet_address(target: &str) -> bool {
    target.parse::<IpAddr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const PARDON_IP_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/PardonIpCommand.java");

    #[test]
    fn pardon_ip_validates_ip_before_ban_list_lookup_like_java() {
        let bans = vec!["203.0.113.7".to_string()];

        assert_eq!(pardon_ip("not-an-ip", &bans), Err(PardonIpError::Invalid));
        assert_eq!(
            pardon_ip("203.0.113.8", &bans),
            Err(PardonIpError::NotBanned)
        );
        assert_eq!(
            pardon_ip("203.0.113.7", &bans),
            Ok(PardonIpOutput {
                success_count: 1,
                feedback_key: "commands.pardonip.success",
                removed_ip: "203.0.113.7".to_string(),
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    fn pardon_ip_accepts_ipv4_and_ipv6_literals_like_inet_addresses() {
        let bans = vec!["2001:db8::1".to_string(), "192.0.2.44".to_string()];

        assert!(pardon_ip("192.0.2.44", &bans).is_ok());
        assert!(pardon_ip("2001:db8::1", &bans).is_ok());
        assert_eq!(pardon_ip("192.0.2.999", &bans), Err(PardonIpError::Invalid));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn pardon_ip_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"pardon-ip\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
            "Commands.argument(\"target\", StringArgumentType.word())",
            "SharedSuggestionProvider.suggest(((CommandSourceStack)c.getSource()).getServer().getPlayerList().getIpBans().getUserList(), p)",
            "private static int unban(final CommandSourceStack source, final String ip)",
            "if (!InetAddresses.isInetAddress(ip))",
            "throw ERROR_INVALID.create();",
            "if (!bans.isBanned(ip))",
            "throw ERROR_NOT_BANNED.create();",
            "bans.remove(ip);",
            "Component.translatable(\"commands.pardonip.success\", ip)",
            "return 1;",
        ] {
            assert!(
                PARDON_IP_COMMAND_JAVA.contains(sentinel),
                "PardonIpCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
