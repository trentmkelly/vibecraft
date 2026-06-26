#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::chat_component::Component;

#[path = "dialog_inputs.rs"]
mod dialog_inputs;
#[path = "dialog_actions.rs"]
mod dialog_actions;

#[allow(unused_imports)]
pub use dialog_actions::*;
#[allow(unused_imports)]
pub use dialog_inputs::*;

pub const DIALOG_WIDTH_MIN: i32 = 1;
pub const DIALOG_WIDTH_MAX: i32 = 1024;
pub const PLAIN_MESSAGE_DEFAULT_WIDTH: i32 = 200;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogActionType {
    OpenUrl,
    RunCommand,
    SuggestCommand,
    ShowDialog,
    ChangePage,
    CopyToClipboard,
    Custom,
    DynamicRunCommand,
    DynamicCustom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAction {
    Close,
    None,
    WaitForResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputControlType {
    Boolean,
    NumberRange,
    SingleOption,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogDefinition {
    pub id: String,
    pub dialog_type: DialogType,
    pub title: String,
    pub body_types: Vec<DialogBodyType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainMessageBody {
    pub contents: Component,
    pub width: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogBody {
    PlainMessage(PlainMessageBody),
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
        ("item", DialogBodyType::Item),
        ("plain_message", DialogBodyType::PlainMessage),
    ]
}

pub fn dialog_action_types() -> &'static [(&'static str, DialogActionType)] {
    &[
        ("open_url", DialogActionType::OpenUrl),
        ("run_command", DialogActionType::RunCommand),
        ("suggest_command", DialogActionType::SuggestCommand),
        ("show_dialog", DialogActionType::ShowDialog),
        ("change_page", DialogActionType::ChangePage),
        ("copy_to_clipboard", DialogActionType::CopyToClipboard),
        ("custom", DialogActionType::Custom),
        ("dynamic/run_command", DialogActionType::DynamicRunCommand),
        ("dynamic/custom", DialogActionType::DynamicCustom),
    ]
}

pub fn dialog_actions() -> &'static [(u8, &'static str, DialogAction)] {
    &[
        (0, "close", DialogAction::Close),
        (1, "none", DialogAction::None),
        (2, "wait_for_response", DialogAction::WaitForResponse),
    ]
}

impl DialogAction {
    pub fn id(self) -> u8 {
        match self {
            Self::Close => 0,
            Self::None => 1,
            Self::WaitForResponse => 2,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::None => "none",
            Self::WaitForResponse => "wait_for_response",
        }
    }

    pub fn will_unpause(self) -> bool {
        matches!(self, Self::Close | Self::WaitForResponse)
    }
}

impl PlainMessageBody {
    pub fn new(contents: Component, width: i32) -> Result<Self, String> {
        if !(DIALOG_WIDTH_MIN..=DIALOG_WIDTH_MAX).contains(&width) {
            return Err(format!(
                "plain message width {width} is outside Java Dialog.WIDTH_CODEC range {DIALOG_WIDTH_MIN}..={DIALOG_WIDTH_MAX}"
            ));
        }

        Ok(Self { contents, width })
    }

    pub fn with_default_width(contents: Component) -> Self {
        Self {
            contents,
            width: PLAIN_MESSAGE_DEFAULT_WIDTH,
        }
    }

    pub fn body_type(&self) -> DialogBodyType {
        DialogBodyType::PlainMessage
    }
}

impl DialogBody {
    pub fn body_type(&self) -> DialogBodyType {
        match self {
            Self::PlainMessage(message) => message.body_type(),
        }
    }
}

pub fn input_control_types() -> &'static [(&'static str, InputControlType)] {
    &[
        ("boolean", InputControlType::Boolean),
        ("number_range", InputControlType::NumberRange),
        ("single_option", InputControlType::SingleOption),
        ("text", InputControlType::Text),
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

    pub fn report_login_activity(&mut self, now_millis: u64, manager: &mut NotificationManager) {
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
    fn dialog_actions_match_java_ids_names_and_unpause_behavior() {
        assert_eq!(
            dialog_actions(),
            &[
                (0, "close", DialogAction::Close),
                (1, "none", DialogAction::None),
                (2, "wait_for_response", DialogAction::WaitForResponse)
            ]
        );

        for (id, name, action) in dialog_actions() {
            assert_eq!(*id, action.id());
            assert_eq!(*name, action.serialized_name());
        }

        assert!(DialogAction::Close.will_unpause());
        assert!(!DialogAction::None.will_unpause());
        assert!(DialogAction::WaitForResponse.will_unpause());
    }

    #[test]
    fn plain_message_body_uses_java_default_width_and_width_range() {
        let message = PlainMessageBody::with_default_width(Component::literal("Rules"));
        assert_eq!(message.contents, Component::literal("Rules"));
        assert_eq!(message.width, PLAIN_MESSAGE_DEFAULT_WIDTH);
        assert_eq!(message.body_type(), DialogBodyType::PlainMessage);
        assert_eq!(
            DialogBody::PlainMessage(message.clone()).body_type(),
            DialogBodyType::PlainMessage
        );

        assert_eq!(
            PlainMessageBody::new(Component::literal("Narrow"), DIALOG_WIDTH_MIN),
            Ok(PlainMessageBody {
                contents: Component::literal("Narrow"),
                width: 1
            })
        );
        assert_eq!(
            PlainMessageBody::new(Component::literal("Wide"), DIALOG_WIDTH_MAX),
            Ok(PlainMessageBody {
                contents: Component::literal("Wide"),
                width: 1024
            })
        );
        assert!(PlainMessageBody::new(Component::empty(), 0).is_err());
        assert!(PlainMessageBody::new(Component::empty(), 1025).is_err());
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn plain_message_body_source_matches_java_26_1_2() {
        const DIALOG: &str = vibecraft_java_source!("/net/minecraft/server/dialog/Dialog.java");
        const PLAIN_MESSAGE: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/body/PlainMessage.java");

        for sentinel in [
            "Codec<Integer> WIDTH_CODEC = ExtraCodecs.intRange(1, 1024);",
            "Codec<Dialog> DIRECT_CODEC = BuiltInRegistries.DIALOG_TYPE.byNameCodec().dispatch(Dialog::codec, c -> c);",
        ] {
            assert!(
                DIALOG.contains(sentinel),
                "Dialog.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record PlainMessage(Component contents, int width) implements DialogBody",
            "public static final int DEFAULT_WIDTH = 200;",
            "ComponentSerialization.CODEC.fieldOf(\"contents\").forGetter(PlainMessage::contents)",
            "Dialog.WIDTH_CODEC.optionalFieldOf(\"width\", 200).forGetter(PlainMessage::width)",
            "Codec.withAlternative(",
            "ComponentSerialization.CODEC, contents -> new PlainMessage(contents, 200)",
            "return MAP_CODEC;",
        ] {
            assert!(
                PLAIN_MESSAGE.contains(sentinel),
                "PlainMessage.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn dialog_action_source_matches_java_26_1_2() {
        const DIALOG_ACTION: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/DialogAction.java");

        for sentinel in [
            "CLOSE(0, \"close\"),",
            "NONE(1, \"none\"),",
            "WAIT_FOR_RESPONSE(2, \"wait_for_response\");",
            "ByteBufCodecs.idMapper(BY_ID, s -> s.id)",
            "return this.name;",
            "return this == CLOSE || this == WAIT_FOR_RESPONSE;",
        ] {
            assert!(
                DIALOG_ACTION.contains(sentinel),
                "DialogAction.java is missing sentinel: {sentinel}"
            );
        }
    }

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
            vec!["item", "plain_message"]
        );
        assert_eq!(
            dialog_action_types()
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            vec![
                "open_url",
                "run_command",
                "suggest_command",
                "show_dialog",
                "change_page",
                "copy_to_clipboard",
                "custom",
                "dynamic/run_command",
                "dynamic/custom"
            ]
        );
        assert_eq!(
            input_control_types()
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            vec!["boolean", "number_range", "single_option", "text"]
        );
        assert_eq!(dialog_tags(), &["pause_screen_additions", "quick_actions"]);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn dialog_registry_bootstrap_sources_match_java_26_1_2() {
        const DIALOG_TYPES: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/DialogTypes.java");
        const ACTION_TYPES: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/action/ActionTypes.java");
        const BODY_TYPES: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/body/DialogBodyTypes.java");
        const INPUT_TYPES: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/input/InputControlTypes.java");

        for sentinel in [
            "Registry.register(registry, \"notice\", NoticeDialog.MAP_CODEC);",
            "Registry.register(registry, \"server_links\", ServerLinksDialog.MAP_CODEC);",
            "Registry.register(registry, \"dialog_list\", DialogListDialog.MAP_CODEC);",
            "Registry.register(registry, \"multi_action\", MultiActionDialog.MAP_CODEC);",
            "return Registry.register(registry, \"confirmation\", ConfirmationDialog.MAP_CODEC);",
        ] {
            assert!(
                DIALOG_TYPES.contains(sentinel),
                "DialogTypes.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "StaticAction.WRAPPED_CODECS.forEach((action, codec) -> Registry.register(registry, Identifier.withDefaultNamespace(action.getSerializedName()), codec));",
            "Registry.register(registry, Identifier.withDefaultNamespace(\"dynamic/run_command\"), CommandTemplate.MAP_CODEC);",
            "return Registry.register(registry, Identifier.withDefaultNamespace(\"dynamic/custom\"), CustomAll.MAP_CODEC);",
        ] {
            assert!(
                ACTION_TYPES.contains(sentinel),
                "ActionTypes.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "Registry.register(registry, Identifier.withDefaultNamespace(\"item\"), ItemBody.MAP_CODEC);",
            "return Registry.register(registry, Identifier.withDefaultNamespace(\"plain_message\"), PlainMessage.MAP_CODEC);",
        ] {
            assert!(
                BODY_TYPES.contains(sentinel),
                "DialogBodyTypes.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "Registry.register(registry, Identifier.withDefaultNamespace(\"boolean\"), BooleanInput.MAP_CODEC);",
            "Registry.register(registry, Identifier.withDefaultNamespace(\"number_range\"), NumberRangeInput.MAP_CODEC);",
            "Registry.register(registry, Identifier.withDefaultNamespace(\"single_option\"), SingleOptionInput.MAP_CODEC);",
            "return Registry.register(registry, Identifier.withDefaultNamespace(\"text\"), TextInput.MAP_CODEC);",
        ] {
            assert!(
                INPUT_TYPES.contains(sentinel),
                "InputControlTypes.java is missing sentinel: {sentinel}"
            );
        }
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
