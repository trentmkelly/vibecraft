#![allow(dead_code)]

use crate::command::{
    BossBarCommandColor, BossBarCommandOverlay, ChatCommandEvent, ChatCommandKind, CustomBossBar,
    TeamMembership, TeamState, TitleCommandAction, TitleCommandEvent, TitleTextKind,
};
use crate::player_access::NameAndId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationPacketKind {
    PlayerChat,
    DisguisedChat,
    SystemChat,
    SetActionBarText,
    SetTitleText,
    SetSubtitleText,
    SetTitlesAnimation,
    ClearTitles,
    BossEventAdd,
    BossEventUpdate,
    BossEventRemove,
    TabListHeaderFooter,
    PlayerCombatKill,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresentationPacket {
    pub target: NameAndId,
    pub kind: PresentationPacketKind,
    pub payload: PresentationPayload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PresentationPayload {
    Chat {
        sender: Option<NameAndId>,
        message: String,
        chat_type: ChatRoute,
        filtered: bool,
    },
    Component(String),
    TitleTimes {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
    ClearTitles {
        reset: bool,
    },
    BossBar(BossBarView),
    TabList {
        header: String,
        footer: String,
    },
    Death {
        victim_id: String,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRoute {
    Chat,
    Emote,
    Private,
    Team,
    TellRaw,
    System,
    ActionBar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BossBarView {
    pub id: String,
    pub name: String,
    pub progress: f32,
    pub color: BossBarCommandColor,
    pub overlay: BossBarCommandOverlay,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPresentationState {
    pub player: NameAndId,
    pub team: Option<String>,
    pub filters_chat: bool,
    pub receives_system_messages: bool,
}

#[derive(Debug, Default, Clone)]
pub struct PresentationDispatcher {
    players: Vec<PlayerPresentationState>,
}

impl PresentationDispatcher {
    pub fn new(players: Vec<PlayerPresentationState>) -> Self {
        Self { players }
    }

    pub fn broadcast_chat(&self, event: &ChatCommandEvent) -> Vec<PresentationPacket> {
        let route = match event.kind {
            ChatCommandKind::Say => ChatRoute::Chat,
            ChatCommandKind::Emote => ChatRoute::Emote,
            ChatCommandKind::Private => ChatRoute::Private,
            ChatCommandKind::Team => ChatRoute::Team,
            ChatCommandKind::TellRaw => ChatRoute::TellRaw,
        };
        let kind = if event.kind == ChatCommandKind::TellRaw {
            PresentationPacketKind::DisguisedChat
        } else {
            PresentationPacketKind::PlayerChat
        };
        self.targets_for(event)
            .into_iter()
            .map(|target| PresentationPacket {
                target: target.player.clone(),
                kind,
                payload: PresentationPayload::Chat {
                    sender: event.sender.clone(),
                    message: event.message.clone(),
                    chat_type: route,
                    filtered: target.filters_chat,
                },
            })
            .collect()
    }

    pub fn system_message(
        &self,
        message: impl Into<String>,
        overlay: bool,
    ) -> Vec<PresentationPacket> {
        let message = message.into();
        self.players
            .iter()
            .filter(|player| player.receives_system_messages)
            .map(|target| PresentationPacket {
                target: target.player.clone(),
                kind: if overlay {
                    PresentationPacketKind::SetActionBarText
                } else {
                    PresentationPacketKind::SystemChat
                },
                payload: PresentationPayload::Component(message.clone()),
            })
            .collect()
    }

    pub fn title_packets(&self, event: &TitleCommandEvent) -> Vec<PresentationPacket> {
        event
            .targets
            .iter()
            .flat_map(|target| {
                let kind = match &event.action {
                    TitleCommandAction::Clear { .. } => PresentationPacketKind::ClearTitles,
                    TitleCommandAction::Text { kind, .. } => match kind {
                        TitleTextKind::Title => PresentationPacketKind::SetTitleText,
                        TitleTextKind::Subtitle => PresentationPacketKind::SetSubtitleText,
                        TitleTextKind::ActionBar => PresentationPacketKind::SetActionBarText,
                    },
                    TitleCommandAction::Times { .. } => PresentationPacketKind::SetTitlesAnimation,
                };
                self.player(target).map(|player| PresentationPacket {
                    target: player.player.clone(),
                    kind,
                    payload: match &event.action {
                        TitleCommandAction::Clear { reset } => {
                            PresentationPayload::ClearTitles { reset: *reset }
                        }
                        TitleCommandAction::Text { component, .. } => {
                            PresentationPayload::Component(component.clone())
                        }
                        TitleCommandAction::Times {
                            fade_in,
                            stay,
                            fade_out,
                        } => PresentationPayload::TitleTimes {
                            fade_in: *fade_in,
                            stay: *stay,
                            fade_out: *fade_out,
                        },
                    },
                })
            })
            .collect()
    }

    pub fn bossbar_packets(
        &self,
        before: Option<&CustomBossBar>,
        after: Option<&CustomBossBar>,
    ) -> Vec<PresentationPacket> {
        match (before, after) {
            (None, Some(after)) => self.bossbar_to_players(after, PresentationPacketKind::BossEventAdd),
            (Some(before), Some(after)) if before != after => {
                self.bossbar_to_players(after, PresentationPacketKind::BossEventUpdate)
            }
            (Some(before), None) => self.bossbar_to_players(before, PresentationPacketKind::BossEventRemove),
            _ => Vec::new(),
        }
    }

    pub fn tab_list_header_footer(
        &self,
        header: impl Into<String>,
        footer: impl Into<String>,
    ) -> Vec<PresentationPacket> {
        let header = header.into();
        let footer = footer.into();
        self.players
            .iter()
            .map(|target| PresentationPacket {
                target: target.player.clone(),
                kind: PresentationPacketKind::TabListHeaderFooter,
                payload: PresentationPayload::TabList {
                    header: header.clone(),
                    footer: footer.clone(),
                },
            })
            .collect()
    }

    pub fn death_message(
        &self,
        victim: &NameAndId,
        source_key: &str,
        teams: &[TeamState],
        memberships: &[TeamMembership],
        show_death_messages: bool,
    ) -> Vec<PresentationPacket> {
        if !show_death_messages {
            return Vec::new();
        }
        let visibility = membership_team(victim, teams, memberships)
            .map(|team| team.death_message_visibility.as_str())
            .unwrap_or("always");
        self.players
            .iter()
            .filter(|target| {
                death_message_visible(victim, &target.player, visibility, memberships)
            })
            .map(|target| PresentationPacket {
                target: target.player.clone(),
                kind: PresentationPacketKind::PlayerCombatKill,
                payload: PresentationPayload::Death {
                    victim_id: victim.uuid.clone(),
                    message: format!("{{\"translate\":\"{source_key}\",\"with\":[\"{}\"]}}", victim.name),
                },
            })
            .collect()
    }

    fn targets_for(&self, event: &ChatCommandEvent) -> Vec<&PlayerPresentationState> {
        if event.targets.is_empty() {
            self.players.iter().collect()
        } else {
            self.players
                .iter()
                .filter(|player| event.targets.iter().any(|target| target.uuid == player.player.uuid))
                .collect()
        }
    }

    fn player(&self, target: &NameAndId) -> Option<&PlayerPresentationState> {
        self.players.iter().find(|player| player.player.uuid == target.uuid)
    }

    fn bossbar_to_players(
        &self,
        bar: &CustomBossBar,
        kind: PresentationPacketKind,
    ) -> Vec<PresentationPacket> {
        let targets: Vec<NameAndId> = if bar.players.is_empty() {
            self.players.iter().map(|player| player.player.clone()).collect()
        } else {
            bar.players.clone()
        };
        targets
            .iter()
            .filter_map(|target| self.player(target))
            .map(|target| PresentationPacket {
                target: target.player.clone(),
                kind,
                payload: PresentationPayload::BossBar(BossBarView {
                    id: bar.id.clone(),
                    name: bar.name.clone(),
                    progress: if bar.max <= 0 {
                        0.0
                    } else {
                        (bar.value as f32 / bar.max as f32).clamp(0.0, 1.0)
                    },
                    color: bar.color,
                    overlay: bar.overlay,
                    visible: bar.visible,
                }),
            })
            .collect()
    }
}

fn membership_team<'a>(
    player: &NameAndId,
    teams: &'a [TeamState],
    memberships: &[TeamMembership],
) -> Option<&'a TeamState> {
    let team_name = memberships
        .iter()
        .find(|membership| membership.player.uuid == player.uuid)?
        .team
        .as_str();
    teams.iter().find(|team| team.name == team_name)
}

fn death_message_visible(
    victim: &NameAndId,
    viewer: &NameAndId,
    visibility: &str,
    memberships: &[TeamMembership],
) -> bool {
    match visibility {
        "never" => false,
        "hideForOtherTeams" => same_team(victim, viewer, memberships),
        "hideForOwnTeam" => !same_team(victim, viewer, memberships),
        _ => true,
    }
}

fn same_team(a: &NameAndId, b: &NameAndId, memberships: &[TeamMembership]) -> bool {
    let a_team = memberships
        .iter()
        .find(|membership| membership.player.uuid == a.uuid)
        .map(|membership| membership.team.as_str());
    let b_team = memberships
        .iter()
        .find(|membership| membership.player.uuid == b.uuid)
        .map(|membership| membership.team.as_str());
    a_team.is_some() && a_team == b_team
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{BossBarCommandColor, BossBarCommandOverlay, TitleTextKind};

    fn player(name: &str) -> NameAndId {
        NameAndId::create_offline(name)
    }

    fn dispatcher() -> PresentationDispatcher {
        PresentationDispatcher::new(vec![
            PlayerPresentationState {
                player: player("Steve"),
                team: Some("red".to_string()),
                filters_chat: false,
                receives_system_messages: true,
            },
            PlayerPresentationState {
                player: player("Alex"),
                team: Some("blue".to_string()),
                filters_chat: true,
                receives_system_messages: true,
            },
        ])
    }

    #[test]
    fn chat_broadcast_routes_player_filtered_private_team_and_tellraw_messages() {
        let event = ChatCommandEvent {
            kind: ChatCommandKind::Say,
            sender: Some(player("Steve")),
            targets: vec![],
            message: "hello".to_string(),
        };
        let packets = dispatcher().broadcast_chat(&event);
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].kind, PresentationPacketKind::PlayerChat);
        assert!(matches!(
            &packets[1].payload,
            PresentationPayload::Chat { filtered: true, chat_type: ChatRoute::Chat, .. }
        ));

        let tellraw = ChatCommandEvent {
            kind: ChatCommandKind::TellRaw,
            sender: None,
            targets: vec![player("Alex")],
            message: "{\"text\":\"hi\"}".to_string(),
        };
        let packets = dispatcher().broadcast_chat(&tellraw);
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].kind, PresentationPacketKind::DisguisedChat);
        assert_eq!(packets[0].target.name, "Alex");
    }

    #[test]
    fn system_actionbar_title_and_tab_packets_use_vanilla_packet_families() {
        let dispatcher = dispatcher();
        assert_eq!(
            dispatcher.system_message("server restarting", false)[0].kind,
            PresentationPacketKind::SystemChat
        );
        assert_eq!(
            dispatcher.system_message("overlay", true)[0].kind,
            PresentationPacketKind::SetActionBarText
        );

        let title = TitleCommandEvent {
            targets: vec![player("Steve")],
            action: TitleCommandAction::Text {
                kind: TitleTextKind::Title,
                component: "{\"text\":\"Go\"}".to_string(),
            },
        };
        assert_eq!(
            dispatcher.title_packets(&title)[0].kind,
            PresentationPacketKind::SetTitleText
        );

        let times = TitleCommandEvent {
            targets: vec![player("Steve")],
            action: TitleCommandAction::Times {
                fade_in: 10,
                stay: 70,
                fade_out: 20,
            },
        };
        assert_eq!(
            dispatcher.title_packets(&times)[0].payload,
            PresentationPayload::TitleTimes {
                fade_in: 10,
                stay: 70,
                fade_out: 20
            }
        );

        let tab = dispatcher.tab_list_header_footer("header", "footer");
        assert_eq!(tab.len(), 2);
        assert!(matches!(
            &tab[0].payload,
            PresentationPayload::TabList { header, footer } if header == "header" && footer == "footer"
        ));
    }

    #[test]
    fn bossbar_add_update_remove_targets_players_and_clamps_progress() {
        let dispatcher = dispatcher();
        let before = CustomBossBar {
            id: "minecraft:raid".to_string(),
            name: "Raid".to_string(),
            color: BossBarCommandColor::Red,
            overlay: BossBarCommandOverlay::Progress,
            value: 20,
            max: 100,
            visible: true,
            players: vec![player("Steve")],
        };
        let mut after = before.clone();
        after.value = 150;

        assert_eq!(
            dispatcher.bossbar_packets(None, Some(&before))[0].kind,
            PresentationPacketKind::BossEventAdd
        );
        let update = dispatcher.bossbar_packets(Some(&before), Some(&after));
        assert_eq!(update[0].kind, PresentationPacketKind::BossEventUpdate);
        assert!(matches!(
            &update[0].payload,
            PresentationPayload::BossBar(view) if (view.progress - 1.0).abs() < f32::EPSILON
        ));
        assert_eq!(
            dispatcher.bossbar_packets(Some(&after), None)[0].kind,
            PresentationPacketKind::BossEventRemove
        );
    }

    #[test]
    fn death_messages_respect_gamerule_and_team_visibility() {
        let dispatcher = dispatcher();
        let teams = vec![TeamState {
            name: "red".to_string(),
            display_name: "Red".to_string(),
            color: "red".to_string(),
            friendly_fire: true,
            see_friendly_invisibles: true,
            nametag_visibility: "always".to_string(),
            death_message_visibility: "hideForOtherTeams".to_string(),
            collision_rule: "always".to_string(),
            prefix: String::new(),
            suffix: String::new(),
        }];
        let memberships = vec![
            TeamMembership {
                player: player("Steve"),
                team: "red".to_string(),
            },
            TeamMembership {
                player: player("Alex"),
                team: "blue".to_string(),
            },
        ];

        let packets = dispatcher.death_message(
            &player("Steve"),
            "death.attack.player",
            &teams,
            &memberships,
            true,
        );
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].target.name, "Steve");
        assert_eq!(
            dispatcher.death_message(&player("Steve"), "death.attack.generic", &teams, &memberships, false),
            Vec::new()
        );
    }
}
