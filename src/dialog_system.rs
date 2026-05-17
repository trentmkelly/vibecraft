#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    Notice,
    ServerLinks,
    DialogList,
    MultiAction,
    Confirmation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogBodyType {
    PlainMessage,
    Item,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogDefinition {
    pub id: String,
    pub dialog_type: DialogType,
    pub title: String,
    pub body_types: Vec<DialogBodyType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogPacket {
    Show { target: String, dialog: String },
    Clear { target: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownServerLinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerLinkType {
    Known(KnownServerLinkType),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLinkEntry {
    pub link_type: ServerLinkType,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntrustedServerLinkEntry {
    pub link_type: ServerLinkType,
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationPacket {
    ServerLinks(Vec<UntrustedServerLinkEntry>),
    CodeOfConduct(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationEvent {
    PlayerJoined(String),
    PlayerLeft(String),
    ServerStarted,
    ServerShuttingDown,
    ServerSaveStarted,
    ServerSaveCompleted,
    ServerActivityOccurred,
    PlayerOpped(String),
    PlayerDeopped(String),
    PlayerAddedToAllowlist(String),
    PlayerRemovedFromAllowlist(String),
    IpBanned(String),
    IpUnbanned(String),
    PlayerBanned(String),
    PlayerUnbanned(String),
    GameRuleChanged { rule: String, value: String },
    StatusHeartbeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationServiceLog {
    pub service_id: String,
    pub events: Vec<NotificationEvent>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationManager {
    services: Vec<NotificationServiceLog>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerActivityMonitor {
    minimum_millis_between_notifications: u64,
    last_notification_time: u64,
    pending_activity: bool,
}

pub fn dialog_types() -> &'static [(&'static str, DialogType)] {
    &[
        ("notice", DialogType::Notice),
        ("server_links", DialogType::ServerLinks),
        ("dialog_list", DialogType::DialogList),
        ("multi_action", DialogType::MultiAction),
        ("confirmation", DialogType::Confirmation),
    ]
}

pub fn dialog_body_types() -> &'static [(&'static str, DialogBodyType)] {
    &[
        ("plain_message", DialogBodyType::PlainMessage),
        ("item", DialogBodyType::Item),
    ]
}

pub fn dialog_tags() -> &'static [&'static str] {
    &["pause_screen_additions", "quick_actions"]
}

pub fn show_dialog(targets: &[String], dialog: &str) -> Vec<DialogPacket> {
    targets
        .iter()
        .map(|target| DialogPacket::Show {
            target: target.clone(),
            dialog: normalize_identifier(dialog),
        })
        .collect()
}

pub fn clear_dialog(targets: &[String]) -> Vec<DialogPacket> {
    targets
        .iter()
        .map(|target| DialogPacket::Clear {
            target: target.clone(),
        })
        .collect()
}

pub fn untrust_server_links(entries: &[ServerLinkEntry]) -> Vec<UntrustedServerLinkEntry> {
    entries
        .iter()
        .map(|entry| UntrustedServerLinkEntry {
            link_type: entry.link_type.clone(),
            link: entry.uri.clone(),
        })
        .collect()
}

pub fn known_link_display_key(link_type: KnownServerLinkType) -> &'static str {
    match link_type {
        KnownServerLinkType::BugReport => "known_server_link.report_bug",
        KnownServerLinkType::CommunityGuidelines => "known_server_link.community_guidelines",
        KnownServerLinkType::Support => "known_server_link.support",
        KnownServerLinkType::Status => "known_server_link.status",
        KnownServerLinkType::Feedback => "known_server_link.feedback",
        KnownServerLinkType::Community => "known_server_link.community",
        KnownServerLinkType::Website => "known_server_link.website",
        KnownServerLinkType::Forums => "known_server_link.forums",
        KnownServerLinkType::News => "known_server_link.news",
        KnownServerLinkType::Announcements => "known_server_link.announcements",
    }
}

pub fn start_configuration_extras(
    server_links: &[ServerLinkEntry],
    code_of_conducts: &BTreeMap<String, String>,
    client_language: &str,
) -> Vec<ConfigurationPacket> {
    let mut packets = Vec::new();
    if !server_links.is_empty() {
        packets.push(ConfigurationPacket::ServerLinks(untrust_server_links(
            server_links,
        )));
    }
    if let Some(code) = select_code_of_conduct(code_of_conducts, client_language) {
        packets.push(ConfigurationPacket::CodeOfConduct(code));
    }
    packets
}

pub fn select_code_of_conduct(
    code_of_conducts: &BTreeMap<String, String>,
    client_language: &str,
) -> Option<String> {
    if code_of_conducts.is_empty() {
        return None;
    }
    let language = client_language.to_lowercase();
    code_of_conducts
        .get(&language)
        .or_else(|| code_of_conducts.get("en_us"))
        .or_else(|| code_of_conducts.values().next())
        .cloned()
}

impl NotificationManager {
    pub fn register_service(&mut self, service_id: impl Into<String>) {
        self.services.push(NotificationServiceLog {
            service_id: service_id.into(),
            events: Vec::new(),
        });
    }

    pub fn notify(&mut self, event: NotificationEvent) {
        for service in &mut self.services {
            service.events.push(event.clone());
        }
    }

    pub fn services(&self) -> &[NotificationServiceLog] {
        &self.services
    }
}

impl ServerActivityMonitor {
    pub fn new(seconds_between_notifications: u64) -> Self {
        Self {
            minimum_millis_between_notifications: seconds_between_notifications * 1_000,
            last_notification_time: 0,
            pending_activity: false,
        }
    }

    pub fn report_login_activity(
        &mut self,
        now_millis: u64,
        manager: &mut NotificationManager,
    ) {
        self.pending_activity = true;
        self.process_with_rate_limit(now_millis, manager);
    }

    pub fn tick(&mut self, now_millis: u64, manager: &mut NotificationManager) {
        self.process_with_rate_limit(now_millis, manager);
    }

    fn process_with_rate_limit(&mut self, now_millis: u64, manager: &mut NotificationManager) {
        if self.pending_activity
            && now_millis.saturating_sub(self.last_notification_time)
                >= self.minimum_millis_between_notifications
        {
            manager.notify(NotificationEvent::ServerActivityOccurred);
            self.last_notification_time = now_millis;
        }
        self.pending_activity = false;
    }
}

fn normalize_identifier(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("minecraft:{input}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_registry_surface_and_tags_match_vanilla_bootstrap() {
        assert_eq!(
            dialog_types().iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![
                "notice",
                "server_links",
                "dialog_list",
                "multi_action",
                "confirmation"
            ]
        );
        assert_eq!(
            dialog_body_types()
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            vec!["plain_message", "item"]
        );
        assert_eq!(dialog_tags(), &["pause_screen_additions", "quick_actions"]);
    }

    #[test]
    fn dialog_show_clear_packets_normalize_resource_ids_for_targets() {
        assert_eq!(
            show_dialog(&["Steve".to_string(), "Alex".to_string()], "welcome"),
            vec![
                DialogPacket::Show {
                    target: "Steve".to_string(),
                    dialog: "minecraft:welcome".to_string()
                },
                DialogPacket::Show {
                    target: "Alex".to_string(),
                    dialog: "minecraft:welcome".to_string()
                }
            ]
        );
        assert_eq!(
            clear_dialog(&["Steve".to_string()]),
            vec![DialogPacket::Clear {
                target: "Steve".to_string()
            }]
        );
    }

    #[test]
    fn server_links_untrust_known_and_custom_entries_for_configuration_packet() {
        let links = vec![
            ServerLinkEntry {
                link_type: ServerLinkType::Known(KnownServerLinkType::Support),
                uri: "https://example.com/support".to_string(),
            },
            ServerLinkEntry {
                link_type: ServerLinkType::Custom("Rules".to_string()),
                uri: "https://example.com/rules".to_string(),
            },
        ];

        assert_eq!(
            known_link_display_key(KnownServerLinkType::Support),
            "known_server_link.support"
        );
        assert_eq!(
            untrust_server_links(&links),
            vec![
                UntrustedServerLinkEntry {
                    link_type: ServerLinkType::Known(KnownServerLinkType::Support),
                    link: "https://example.com/support".to_string()
                },
                UntrustedServerLinkEntry {
                    link_type: ServerLinkType::Custom("Rules".to_string()),
                    link: "https://example.com/rules".to_string()
                }
            ]
        );
    }

    #[test]
    fn configuration_extras_send_links_then_localized_code_of_conduct() {
        let links = vec![ServerLinkEntry {
            link_type: ServerLinkType::Known(KnownServerLinkType::Website),
            uri: "https://example.com".to_string(),
        }];
        let codes = BTreeMap::from([
            ("en_us".to_string(), "English rules".to_string()),
            ("fr_fr".to_string(), "Regles francaises".to_string()),
        ]);

        assert_eq!(
            select_code_of_conduct(&codes, "FR_FR"),
            Some("Regles francaises".to_string())
        );
        assert_eq!(
            start_configuration_extras(&links, &codes, "es_es"),
            vec![
                ConfigurationPacket::ServerLinks(untrust_server_links(&links)),
                ConfigurationPacket::CodeOfConduct("English rules".to_string())
            ]
        );
    }

    #[test]
    fn notification_manager_fans_out_every_event_to_registered_services() {
        let mut manager = NotificationManager::default();
        manager.register_service("audit");
        manager.register_service("webhook");

        manager.notify(NotificationEvent::ServerStarted);
        manager.notify(NotificationEvent::PlayerJoined("Steve".to_string()));
        manager.notify(NotificationEvent::GameRuleChanged {
            rule: "doDaylightCycle".to_string(),
            value: "false".to_string(),
        });

        assert_eq!(manager.services().len(), 2);
        assert!(manager
            .services()
            .iter()
            .all(|service| service.events.len() == 3));
        assert_eq!(
            manager.services()[0].events[1],
            NotificationEvent::PlayerJoined("Steve".to_string())
        );
    }

    #[test]
    fn activity_monitor_rate_limits_login_activity_notifications() {
        let mut manager = NotificationManager::default();
        manager.register_service("audit");
        let mut monitor = ServerActivityMonitor::new(10);

        monitor.report_login_activity(1_000, &mut manager);
        assert!(manager.services()[0].events.is_empty());
        monitor.report_login_activity(10_000, &mut manager);
        assert_eq!(
            manager.services()[0].events,
            vec![NotificationEvent::ServerActivityOccurred]
        );
        monitor.report_login_activity(15_000, &mut manager);
        assert_eq!(manager.services()[0].events.len(), 1);
        monitor.report_login_activity(20_000, &mut manager);
        assert_eq!(manager.services()[0].events.len(), 2);
    }
}
