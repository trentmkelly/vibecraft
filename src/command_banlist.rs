#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanListEntryModel {
    pub display_name: String,
    pub source: String,
    pub reason_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanListOutput {
    pub success_count: usize,
    pub messages: Vec<BanListMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BanListMessage {
    None,
    List { count: usize },
    Entry {
        display_name: String,
        source: String,
        reason_message: String,
    },
}

pub fn show_ban_list(entries: &[BanListEntryModel]) -> BanListOutput {
    let mut messages = Vec::new();
    if entries.is_empty() {
        messages.push(BanListMessage::None);
    } else {
        messages.push(BanListMessage::List {
            count: entries.len(),
        });
        for entry in entries {
            messages.push(BanListMessage::Entry {
                display_name: entry.display_name.clone(),
                source: entry.source.clone(),
                reason_message: entry.reason_message.clone(),
            });
        }
    }
    BanListOutput {
        success_count: entries.len(),
        messages,
    }
}

pub fn select_banlist_entries<'a>(
    mode: Option<&str>,
    player_bans: &'a [BanListEntryModel],
    ip_bans: &'a [BanListEntryModel],
) -> Option<Vec<&'a BanListEntryModel>> {
    match mode {
        None => Some(player_bans.iter().chain(ip_bans.iter()).collect()),
        Some("players") => Some(player_bans.iter().collect()),
        Some("ips") => Some(ip_bans.iter().collect()),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const BAN_LIST_COMMANDS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/BanListCommands.java");

    fn entry(name: &str, source: &str, reason: &str) -> BanListEntryModel {
        BanListEntryModel {
            display_name: name.to_string(),
            source: source.to_string(),
            reason_message: reason.to_string(),
        }
    }

    #[test]
    fn banlist_selector_matches_java_default_players_and_ips_literals() {
        let players = vec![entry("Steve", "Console", "griefing")];
        let ips = vec![entry("203.0.113.4", "Console", "spam")];

        assert_eq!(
            select_banlist_entries(None, &players, &ips)
                .unwrap_or_else(|| panic!("default selector should exist"))
                .into_iter()
                .map(|entry| entry.display_name.as_str())
                .collect::<Vec<_>>(),
            vec!["Steve", "203.0.113.4"]
        );
        assert_eq!(
            select_banlist_entries(Some("players"), &players, &ips)
                .unwrap_or_else(|| panic!("players selector should exist"))
                .into_iter()
                .map(|entry| entry.display_name.as_str())
                .collect::<Vec<_>>(),
            vec!["Steve"]
        );
        assert_eq!(
            select_banlist_entries(Some("ips"), &players, &ips)
                .unwrap_or_else(|| panic!("ips selector should exist"))
                .into_iter()
                .map(|entry| entry.display_name.as_str())
                .collect::<Vec<_>>(),
            vec!["203.0.113.4"]
        );
        assert_eq!(select_banlist_entries(Some("bad"), &players, &ips), None);
    }

    #[test]
    fn show_list_empty_and_non_empty_messages_match_java_order() {
        assert_eq!(
            show_ban_list(&[]),
            BanListOutput {
                success_count: 0,
                messages: vec![BanListMessage::None],
            }
        );

        let entries = vec![
            entry("Steve", "Console", "griefing"),
            entry("203.0.113.4", "Admin", "spam"),
        ];
        assert_eq!(
            show_ban_list(&entries),
            BanListOutput {
                success_count: 2,
                messages: vec![
                    BanListMessage::List { count: 2 },
                    BanListMessage::Entry {
                        display_name: "Steve".to_string(),
                        source: "Console".to_string(),
                        reason_message: "griefing".to_string(),
                    },
                    BanListMessage::Entry {
                        display_name: "203.0.113.4".to_string(),
                        source: "Admin".to_string(),
                        reason_message: "spam".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn banlist_commands_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"banlist\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
            "Lists.newArrayList(Iterables.concat(players.getBans().getEntries(), players.getIpBans().getEntries()))",
            "Commands.literal(\"ips\")",
            "getServer().getPlayerList().getIpBans().getEntries()",
            "Commands.literal(\"players\")",
            "getServer().getPlayerList().getBans().getEntries()",
            "private static int showList(final CommandSourceStack source, final Collection<? extends BanListEntry<?>> list)",
            "Component.translatable(\"commands.banlist.none\")",
            "Component.translatable(\"commands.banlist.list\", list.size())",
            "Component.translatable(\"commands.banlist.entry\", entry.getDisplayName(), entry.getSource(), entry.getReasonMessage())",
            "return list.size();",
        ] {
            assert!(
                BAN_LIST_COMMANDS_JAVA.contains(sentinel),
                "BanListCommands.java is missing sentinel: {sentinel}"
            );
        }
    }
}
