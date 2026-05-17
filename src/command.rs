#![allow(dead_code)]

use crate::player_access::NameAndId;
use crate::runtime::{TickRateController, MAX_TICK_RATE, MIN_TICK_RATE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    All = 0,
    Moderators = 1,
    Gamemasters = 2,
    Admins = 3,
    Owners = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    CommandLevel(PermissionLevel),
    CommandsEntitySelectors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelBasedPermissionSet {
    level: PermissionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAvailability {
    Available,
    Hidden,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerCommandState {
    pub player_idle_timeout_minutes: u32,
    pub autosave_enabled: bool,
    pub save_all_requests: Vec<SaveAllRequest>,
    pub published_server: Option<PublishRequest>,
    pub next_available_publish_port: u16,
    pub random_sequences: Vec<RandomSequenceState>,
    pub random_seed_defaults: RandomSeedDefaults,
    pub random_broadcasts: Vec<RandomSample>,
    pub available_data_packs: Vec<String>,
    pub selected_data_packs: Vec<String>,
    pub disabled_data_packs: Vec<String>,
    pub reload_requests: Vec<ReloadRequest>,
    pub transfer_requests: Vec<TransferRequest>,
    pub perf_recording: bool,
    pub perf_reports: Vec<PerfReport>,
    pub jfr_recording: bool,
    pub jfr_recordings: Vec<String>,
    pub next_jfr_recording_path: String,
    pub known_recipes: Vec<String>,
    pub player_recipes: Vec<PlayerRecipeBook>,
    pub command_source_player: Option<NameAndId>,
    pub command_source_entity: Option<EntityRef>,
    pub online_players: Vec<NameAndId>,
    pub max_players: u32,
    pub singleplayer_owner: Option<NameAndId>,
    pub disconnected_players: Vec<PlayerDisconnect>,
    pub killed_entities: Vec<EntityRef>,
    pub player_teams: Vec<TeamMembership>,
    pub chat_events: Vec<ChatCommandEvent>,
    pub sound_events: Vec<SoundCommandEvent>,
    pub particle_events: Vec<ParticleCommandEvent>,
    pub server_pack_events: Vec<ServerPackCommandEvent>,
    pub rotation_requests: Vec<RotationRequest>,
    pub return_events: Vec<ReturnCommandEvent>,
    pub ride_events: Vec<RideCommandEvent>,
    pub entity_mounts: Vec<EntityMount>,
    pub entity_states: Vec<EntityState>,
    pub weather: WeatherState,
    pub whitelist_enabled: bool,
    pub whitelisted_players: Vec<NameAndId>,
    pub whitelist_reload_requests: u32,
    pub kick_unlisted_requests: u32,
    pub tick_rate: TickRateController,
    pub average_tick_time_nanos: u64,
    pub tick_time_samples_nanos: Vec<u64>,
    pub halt_requested: bool,
    pub world_seed: i64,
    pub version: VersionInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveAllRequest {
    pub flush: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest {
    pub host: String,
    pub port: u16,
    pub targets: Vec<NameAndId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishRequest {
    pub port: u16,
    pub allow_commands: bool,
    pub gamemode: Option<GameMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSequenceState {
    pub id: String,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomSeedDefaults {
    pub salt: i32,
    pub include_world_seed: bool,
    pub include_sequence_id: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSample {
    pub value: i32,
    pub min: i32,
    pub max: i32,
    pub sequence: Option<String>,
    pub announced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRef {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDisconnect {
    pub player: NameAndId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadRequest {
    pub selected_packs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerfReport {
    pub ticks: u32,
    pub duration_nanos: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRecipeBook {
    pub player: NameAndId,
    pub recipes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMembership {
    pub player: NameAndId,
    pub team: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatCommandEvent {
    pub kind: ChatCommandKind,
    pub sender: Option<NameAndId>,
    pub targets: Vec<NameAndId>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatCommandKind {
    Say,
    Private,
    Team,
    TellRaw,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SoundCommandEvent {
    Play(PlaySoundRequest),
    Stop(StopSoundRequest),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaySoundRequest {
    pub sound: String,
    pub source: SoundSource,
    pub targets: Vec<NameAndId>,
    pub position: Vec3,
    pub volume: f32,
    pub pitch: f32,
    pub min_volume: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopSoundRequest {
    pub targets: Vec<NameAndId>,
    pub source: Option<SoundSource>,
    pub sound: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Master,
    Music,
    Record,
    Weather,
    Block,
    Hostile,
    Neutral,
    Player,
    Ambient,
    Voice,
    Ui,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleCommandEvent {
    pub name: String,
    pub viewers: Vec<NameAndId>,
    pub position: Vec3,
    pub delta: Vec3,
    pub speed: f32,
    pub count: u32,
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerPackCommandEvent {
    Push(ServerPackPushRequest),
    Pop { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPackPushRequest {
    pub id: String,
    pub url: String,
    pub hash: String,
    pub required: bool,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotationRequest {
    pub target: EntityRef,
    pub mode: RotationMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RotationMode {
    Angles {
        yaw: f32,
        pitch: f32,
        yaw_relative: bool,
        pitch_relative: bool,
    },
    FacingEntity {
        entity: EntityRef,
        anchor: EntityAnchor,
    },
    FacingPosition(Vec3),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchor {
    Feet,
    Eyes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnCommandEvent {
    Success {
        value: i32,
        discard_frame: bool,
    },
    Failure {
        discard_frame: bool,
    },
    Run {
        command: String,
        discard_frame: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RideCommandEvent {
    Mount {
        target: EntityRef,
        vehicle: EntityRef,
    },
    Dismount {
        target: EntityRef,
        vehicle: EntityRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMount {
    pub target: EntityRef,
    pub vehicle: EntityRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityState {
    pub entity: EntityRef,
    pub kind: EntityKind,
    pub dimension: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Generic,
    Player,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherState {
    pub mode: WeatherMode,
    pub duration_ticks: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherMode {
    Clear,
    Rain,
    Thunder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub broadcast_to_admins: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    PermissionDenied,
    InvalidSyntax,
    SaveAlreadyOff,
    SaveAlreadyOn,
    SaveFailed,
    WhitelistAlreadyOn,
    WhitelistAlreadyOff,
    AlreadyWhitelisted,
    NotWhitelisted,
    NoPlayers,
    PublishAlreadyPublished,
    PublishFailed,
    RandomRangeTooSmall,
    RandomRangeTooLarge,
    KickSingleplayer,
    KickOwner,
    HelpFailed,
    TeamMsgNoTeam,
    PlaySoundTooFar,
    ParticleFailed,
    RideNotRiding,
    RideAlreadyRiding,
    RideMountingPlayer,
    RideMountingLoop,
    RideWrongDimension,
    RideMountFailed,
    PerfAlreadyRunning,
    PerfNotRunning,
    JfrStartFailed,
    JfrDumpFailed,
    RecipeGiveFailed,
    RecipeTakeFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub data_version: i32,
    pub series: &'static str,
    pub protocol_version: i32,
    pub build_time: &'static str,
    pub resource_pack_version: PackVersion,
    pub data_pack_version: PackVersion,
    pub stable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackVersion {
    pub major: u32,
    pub minor: u32,
}

impl std::fmt::Display for PackVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.minor == 0 {
            write!(formatter, "{}", self.major)
        } else {
            write!(formatter, "{}.{}", self.major, self.minor)
        }
    }
}

impl PermissionLevel {
    pub fn by_id(id: i32) -> Self {
        match id {
            i32::MIN..=-1 => Self::All,
            0 => Self::All,
            1 => Self::Moderators,
            2 => Self::Gamemasters,
            3 => Self::Admins,
            4..=i32::MAX => Self::Owners,
        }
    }

    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Moderators => "moderators",
            Self::Gamemasters => "gamemasters",
            Self::Admins => "admins",
            Self::Owners => "owners",
        }
    }

    pub fn is_equal_or_higher_than(self, other: Self) -> bool {
        self.id() >= other.id()
    }
}

impl Default for ServerCommandState {
    fn default() -> Self {
        Self {
            player_idle_timeout_minutes: 0,
            autosave_enabled: true,
            save_all_requests: Vec::new(),
            published_server: None,
            next_available_publish_port: 25565,
            random_sequences: Vec::new(),
            random_seed_defaults: RandomSeedDefaults::default(),
            random_broadcasts: Vec::new(),
            available_data_packs: vec!["vanilla".to_string()],
            selected_data_packs: vec!["vanilla".to_string()],
            disabled_data_packs: Vec::new(),
            reload_requests: Vec::new(),
            transfer_requests: Vec::new(),
            perf_recording: false,
            perf_reports: Vec::new(),
            jfr_recording: false,
            jfr_recordings: Vec::new(),
            next_jfr_recording_path: "debug/rustcraft.jfr".to_string(),
            known_recipes: Vec::new(),
            player_recipes: Vec::new(),
            command_source_player: None,
            command_source_entity: None,
            online_players: Vec::new(),
            max_players: 20,
            singleplayer_owner: None,
            disconnected_players: Vec::new(),
            killed_entities: Vec::new(),
            player_teams: Vec::new(),
            chat_events: Vec::new(),
            sound_events: Vec::new(),
            particle_events: Vec::new(),
            server_pack_events: Vec::new(),
            rotation_requests: Vec::new(),
            return_events: Vec::new(),
            ride_events: Vec::new(),
            entity_mounts: Vec::new(),
            entity_states: Vec::new(),
            weather: WeatherState::default(),
            whitelist_enabled: false,
            whitelisted_players: Vec::new(),
            whitelist_reload_requests: 0,
            kick_unlisted_requests: 0,
            tick_rate: TickRateController::default(),
            average_tick_time_nanos: 50_000_000,
            tick_time_samples_nanos: vec![50_000_000],
            halt_requested: false,
            world_seed: 0,
            version: VersionInfo::CURRENT_26_1_2,
        }
    }
}

impl VersionInfo {
    pub const CURRENT_26_1_2: Self = Self {
        id: "26.1.2",
        name: "26.1.2",
        data_version: 4790,
        series: "main",
        protocol_version: 775,
        build_time: "2026-04-09T10:11:03+00:00",
        resource_pack_version: PackVersion {
            major: 84,
            minor: 0,
        },
        data_pack_version: PackVersion {
            major: 101,
            minor: 1,
        },
        stable: true,
    };

    pub fn command_lines(&self) -> Vec<String> {
        vec![
            "commands.version.header".to_string(),
            format!("commands.version.id {}", self.id),
            format!("commands.version.name {}", self.name),
            format!("commands.version.data {}", self.data_version),
            format!("commands.version.series {}", self.series),
            format!(
                "commands.version.protocol {} 0x{:x}",
                self.protocol_version, self.protocol_version
            ),
            format!("commands.version.build_time {}", self.build_time),
            format!(
                "commands.version.pack.resource {}",
                self.resource_pack_version
            ),
            format!("commands.version.pack.data {}", self.data_pack_version),
            if self.stable {
                "commands.version.stable.yes".to_string()
            } else {
                "commands.version.stable.no".to_string()
            },
        ]
    }
}

impl LevelBasedPermissionSet {
    pub const ALL: Self = Self::new(PermissionLevel::All);
    pub const MODERATOR: Self = Self::new(PermissionLevel::Moderators);
    pub const GAMEMASTER: Self = Self::new(PermissionLevel::Gamemasters);
    pub const ADMIN: Self = Self::new(PermissionLevel::Admins);
    pub const OWNER: Self = Self::new(PermissionLevel::Owners);

    pub const fn new(level: PermissionLevel) -> Self {
        Self { level }
    }

    pub fn level(self) -> PermissionLevel {
        self.level
    }

    pub fn has_permission(self, permission: Permission) -> bool {
        match permission {
            Permission::CommandLevel(level) => self.level.is_equal_or_higher_than(level),
            Permission::CommandsEntitySelectors => self
                .level
                .is_equal_or_higher_than(PermissionLevel::Gamemasters),
        }
    }

    pub fn can_run(self, command: &str) -> CommandAvailability {
        let command = command
            .trim_start()
            .strip_prefix('/')
            .unwrap_or(command.trim_start())
            .split_whitespace()
            .next()
            .unwrap_or_default();
        let required = command_required_permission(command);
        if self.has_permission(Permission::CommandLevel(required)) {
            CommandAvailability::Available
        } else {
            CommandAvailability::Hidden
        }
    }
}

impl ServerCommandState {
    pub fn whitelist_names(&self) -> Vec<&str> {
        self.whitelisted_players
            .iter()
            .map(|profile| profile.name.as_str())
            .collect()
    }

    fn is_whitelisted(&self, profile: &NameAndId) -> bool {
        self.whitelisted_players
            .iter()
            .any(|entry| entry.uuid == profile.uuid)
    }

    fn add_whitelisted(&mut self, profile: NameAndId) -> bool {
        if self.is_whitelisted(&profile) {
            false
        } else {
            self.whitelisted_players.push(profile);
            true
        }
    }

    fn remove_whitelisted(&mut self, profile: &NameAndId) -> bool {
        let old_len = self.whitelisted_players.len();
        self.whitelisted_players
            .retain(|entry| entry.uuid != profile.uuid);
        self.whitelisted_players.len() != old_len
    }

    fn team_for_player(&self, player: &NameAndId) -> Option<&str> {
        self.player_teams
            .iter()
            .find(|membership| membership.player.uuid == player.uuid)
            .map(|membership| membership.team.as_str())
    }

    fn players_on_team(&self, team: &str) -> Vec<NameAndId> {
        self.player_teams
            .iter()
            .filter(|membership| membership.team == team)
            .map(|membership| membership.player.clone())
            .collect()
    }
}

pub fn execute_builtin_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    input: &str,
) -> Result<CommandResult, CommandError> {
    let input = input
        .trim_start()
        .strip_prefix('/')
        .unwrap_or(input.trim_start());
    let parts = input.split_whitespace().collect::<Vec<_>>();
    let Some(command) = parts.first().copied() else {
        return Err(CommandError::InvalidSyntax);
    };
    if permissions.can_run(command) == CommandAvailability::Hidden {
        return Err(CommandError::PermissionDenied);
    }

    match command {
        "setidletimeout" => {
            if parts.len() != 2 {
                return Err(CommandError::InvalidSyntax);
            }
            let minutes = parts[1]
                .parse::<u32>()
                .map_err(|_| CommandError::InvalidSyntax)?;
            state.player_idle_timeout_minutes = minutes;
            Ok(CommandResult {
                success_count: minutes as i32,
                feedback_key: if minutes > 0 {
                    "commands.setidletimeout.success"
                } else {
                    "commands.setidletimeout.success.disabled"
                },
                broadcast_to_admins: true,
            })
        }
        "save-all" => {
            let flush = match parts.as_slice() {
                ["save-all"] => false,
                ["save-all", "flush"] => true,
                _ => return Err(CommandError::InvalidSyntax),
            };
            state.save_all_requests.push(SaveAllRequest { flush });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.save.success",
                broadcast_to_admins: true,
            })
        }
        "save-off" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            if !state.autosave_enabled {
                return Err(CommandError::SaveAlreadyOff);
            }
            state.autosave_enabled = false;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.save.disabled",
                broadcast_to_admins: true,
            })
        }
        "save-on" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            if state.autosave_enabled {
                return Err(CommandError::SaveAlreadyOn);
            }
            state.autosave_enabled = true;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.save.enabled",
                broadcast_to_admins: true,
            })
        }
        "stop" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            state.halt_requested = true;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.stop.stopping",
                broadcast_to_admins: true,
            })
        }
        "help" => match parts.as_slice() {
            ["help"] => Ok(CommandResult {
                success_count: visible_command_usages(permissions).len() as i32,
                feedback_key: "commands.help.success",
                broadcast_to_admins: false,
            }),
            ["help", command @ ..] if !command.is_empty() => {
                let command = command.join(" ");
                let root = command.split_whitespace().next().unwrap_or_default();
                if command_usage(root, permissions).is_some() {
                    Ok(CommandResult {
                        success_count: 1,
                        feedback_key: "commands.help.success",
                        broadcast_to_admins: false,
                    })
                } else {
                    Err(CommandError::HelpFailed)
                }
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "jfr" => jfr_command(state, &parts),
        "say" => {
            if parts.len() < 2 {
                return Err(CommandError::InvalidSyntax);
            }
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::Say,
                sender: state.command_source_player.clone(),
                targets: state.online_players.clone(),
                message: parts[1..].join(" "),
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.say.success",
                broadcast_to_admins: false,
            })
        }
        "msg" | "tell" | "w" => {
            if parts.len() < 3 {
                return Err(CommandError::InvalidSyntax);
            }
            let split = parts[1..]
                .iter()
                .position(|part| *part == "--")
                .map(|index| index + 1)
                .unwrap_or(2);
            let target_names = &parts[1..split];
            let message_parts = if split < parts.len() && parts[split] == "--" {
                &parts[split + 1..]
            } else {
                &parts[split..]
            };
            if target_names.is_empty() || message_parts.is_empty() {
                return Err(CommandError::InvalidSyntax);
            }
            let targets = target_names
                .iter()
                .map(|target| NameAndId::create_offline(target))
                .collect::<Vec<_>>();
            let count = targets.len() as i32;
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::Private,
                sender: state.command_source_player.clone(),
                targets,
                message: message_parts.join(" "),
            });
            Ok(CommandResult {
                success_count: count,
                feedback_key: "commands.message.display",
                broadcast_to_admins: false,
            })
        }
        "teammsg" | "tm" => {
            if parts.len() < 2 {
                return Err(CommandError::InvalidSyntax);
            }
            let sender = state
                .command_source_player
                .clone()
                .ok_or(CommandError::InvalidSyntax)?;
            let team = state
                .team_for_player(&sender)
                .ok_or(CommandError::TeamMsgNoTeam)?
                .to_string();
            let targets = state.players_on_team(&team);
            let count = targets.len() as i32;
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::Team,
                sender: Some(sender),
                targets,
                message: parts[1..].join(" "),
            });
            Ok(CommandResult {
                success_count: count,
                feedback_key: "commands.teammsg.success",
                broadcast_to_admins: false,
            })
        }
        "tellraw" => {
            if parts.len() < 3 {
                return Err(CommandError::InvalidSyntax);
            }
            let split = parts[1..]
                .iter()
                .position(|part| *part == "--")
                .map(|index| index + 1)
                .unwrap_or(2);
            let target_names = &parts[1..split];
            let message_parts = if split < parts.len() && parts[split] == "--" {
                &parts[split + 1..]
            } else {
                &parts[split..]
            };
            if target_names.is_empty() || message_parts.is_empty() {
                return Err(CommandError::InvalidSyntax);
            }
            let targets = target_names
                .iter()
                .map(|target| NameAndId::create_offline(target))
                .collect::<Vec<_>>();
            let count = targets.len() as i32;
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::TellRaw,
                sender: state.command_source_player.clone(),
                targets,
                message: message_parts.join(" "),
            });
            Ok(CommandResult {
                success_count: count,
                feedback_key: "commands.tellraw.success",
                broadcast_to_admins: false,
            })
        }
        "playsound" => play_sound_command(state, &parts, permissions),
        "stopsound" => stop_sound_command(state, &parts),
        "particle" => particle_command(state, &parts),
        "perf" => perf_command(state, &parts),
        "rotate" => rotate_command(state, &parts),
        "return" => return_command(state, &parts),
        "ride" => ride_command(state, &parts),
        "seed" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            Ok(CommandResult {
                success_count: state.world_seed as i32,
                feedback_key: "commands.seed.success",
                broadcast_to_admins: false,
            })
        }
        "serverpack" => server_pack_command(state, &parts),
        "version" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            Ok(CommandResult {
                success_count: state.version.command_lines().len() as i32,
                feedback_key: "commands.version.header",
                broadcast_to_admins: false,
            })
        }
        "list" => match parts.as_slice() {
            ["list"] => Ok(CommandResult {
                success_count: state.online_players.len() as i32,
                feedback_key: "commands.list.players",
                broadcast_to_admins: false,
            }),
            ["list", "uuids"] => Ok(CommandResult {
                success_count: state.online_players.len() as i32,
                feedback_key: "commands.list.players",
                broadcast_to_admins: false,
            }),
            _ => Err(CommandError::InvalidSyntax),
        },
        "kick" => match parts.as_slice() {
            ["kick", targets @ ..] if !targets.is_empty() => {
                let split = targets
                    .iter()
                    .position(|part| *part == "--")
                    .unwrap_or(targets.len());
                let (targets, reason_parts) = targets.split_at(split);
                let reason = if reason_parts.is_empty() {
                    "multiplayer.disconnect.kicked".to_string()
                } else {
                    reason_parts[1..].join(" ")
                };
                kick_players(state, targets, reason)
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "kill" => match parts.as_slice() {
            ["kill"] => {
                let source = state
                    .command_source_entity
                    .clone()
                    .ok_or(CommandError::InvalidSyntax)?;
                kill_entities(state, vec![source])
            }
            ["kill", targets @ ..] if !targets.is_empty() => {
                let targets = targets
                    .iter()
                    .map(|target| EntityRef {
                        id: (*target).to_string(),
                        display_name: (*target).to_string(),
                    })
                    .collect::<Vec<_>>();
                kill_entities(state, targets)
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "whitelist" => match parts.as_slice() {
            ["whitelist", "on"] => {
                if state.whitelist_enabled {
                    return Err(CommandError::WhitelistAlreadyOn);
                }
                state.whitelist_enabled = true;
                state.kick_unlisted_requests += 1;
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.whitelist.enabled",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "off"] => {
                if !state.whitelist_enabled {
                    return Err(CommandError::WhitelistAlreadyOff);
                }
                state.whitelist_enabled = false;
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.whitelist.disabled",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "list"] => Ok(CommandResult {
                success_count: state.whitelisted_players.len() as i32,
                feedback_key: if state.whitelisted_players.is_empty() {
                    "commands.whitelist.none"
                } else {
                    "commands.whitelist.list"
                },
                broadcast_to_admins: false,
            }),
            ["whitelist", "reload"] => {
                state.whitelist_reload_requests += 1;
                state.kick_unlisted_requests += 1;
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.whitelist.reloaded",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "add", targets @ ..] if !targets.is_empty() => {
                let mut success = 0;
                for target in targets {
                    if state.add_whitelisted(NameAndId::create_offline(target)) {
                        success += 1;
                    }
                }
                if success == 0 {
                    return Err(CommandError::AlreadyWhitelisted);
                }
                Ok(CommandResult {
                    success_count: success,
                    feedback_key: "commands.whitelist.add.success",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "remove", targets @ ..] if !targets.is_empty() => {
                let mut success = 0;
                for target in targets {
                    let profile = NameAndId::create_offline(target);
                    if state.remove_whitelisted(&profile) {
                        success += 1;
                    }
                }
                if success == 0 {
                    return Err(CommandError::NotWhitelisted);
                }
                state.kick_unlisted_requests += 1;
                Ok(CommandResult {
                    success_count: success,
                    feedback_key: "commands.whitelist.remove.success",
                    broadcast_to_admins: true,
                })
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "tick" => match parts.as_slice() {
            ["tick", "query"] => Ok(CommandResult {
                success_count: state.tick_rate.tick_rate() as i32,
                feedback_key: tick_query_status_key(state),
                broadcast_to_admins: false,
            }),
            ["tick", "rate", rate] => {
                let rate = rate
                    .parse::<f32>()
                    .map_err(|_| CommandError::InvalidSyntax)?;
                if !(MIN_TICK_RATE..=MAX_TICK_RATE).contains(&rate) {
                    return Err(CommandError::InvalidSyntax);
                }
                state.tick_rate.set_tick_rate(rate);
                Ok(CommandResult {
                    success_count: rate as i32,
                    feedback_key: "commands.tick.rate.success",
                    broadcast_to_admins: true,
                })
            }
            ["tick", "freeze"] => {
                state.tick_rate.set_frozen(true);
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.tick.status.frozen",
                    broadcast_to_admins: true,
                })
            }
            ["tick", "unfreeze"] => {
                state.tick_rate.set_frozen(false);
                Ok(CommandResult {
                    success_count: 0,
                    feedback_key: "commands.tick.status.running",
                    broadcast_to_admins: true,
                })
            }
            ["tick", "step"] => tick_step(state, 1),
            ["tick", "step", "stop"] => {
                let stopped = state.tick_rate.stop_stepping();
                Ok(CommandResult {
                    success_count: i32::from(stopped),
                    feedback_key: if stopped {
                        "commands.tick.step.stop.success"
                    } else {
                        "commands.tick.step.stop.fail"
                    },
                    broadcast_to_admins: stopped,
                })
            }
            ["tick", "step", time] => tick_step(state, parse_time_ticks(time)?),
            ["tick", "sprint", "stop"] => {
                let stopped = state.tick_rate.stop_sprinting();
                Ok(CommandResult {
                    success_count: i32::from(stopped),
                    feedback_key: if stopped {
                        "commands.tick.sprint.stop.success"
                    } else {
                        "commands.tick.sprint.stop.fail"
                    },
                    broadcast_to_admins: stopped,
                })
            }
            ["tick", "sprint", time] => {
                let ticks = parse_time_ticks(time)? as u64;
                let _interrupted = state.tick_rate.request_game_to_sprint(ticks);
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.tick.status.sprinting",
                    broadcast_to_admins: true,
                })
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "transfer" => match parts.as_slice() {
            ["transfer", host] => {
                let source = state
                    .command_source_player
                    .clone()
                    .ok_or(CommandError::NoPlayers)?;
                queue_transfer(state, host, 25565, vec![source])
            }
            ["transfer", host, port] => {
                let source = state
                    .command_source_player
                    .clone()
                    .ok_or(CommandError::NoPlayers)?;
                queue_transfer(state, host, parse_port(port)?, vec![source])
            }
            ["transfer", host, port, players @ ..] if !players.is_empty() => {
                let targets = players
                    .iter()
                    .map(|player| NameAndId::create_offline(player))
                    .collect::<Vec<_>>();
                queue_transfer(state, host, parse_port(port)?, targets)
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "publish" => match parts.as_slice() {
            ["publish"] => publish_server(state, state.next_available_publish_port, false, None),
            ["publish", allow_commands] => publish_server(
                state,
                state.next_available_publish_port,
                parse_bool(allow_commands)?,
                None,
            ),
            ["publish", allow_commands, gamemode] => publish_server(
                state,
                state.next_available_publish_port,
                parse_bool(allow_commands)?,
                Some(parse_gamemode(gamemode)?),
            ),
            ["publish", allow_commands, gamemode, port] => publish_server(
                state,
                parse_publish_port(port)?,
                parse_bool(allow_commands)?,
                Some(parse_gamemode(gamemode)?),
            ),
            _ => Err(CommandError::InvalidSyntax),
        },
        "random" => match parts.as_slice() {
            ["random", "value", range] => random_sample(state, range, None, false),
            ["random", "roll", range] => random_sample(state, range, None, true),
            ["random", "value", range, sequence] => {
                require_gamemaster(permissions)?;
                random_sample(state, range, Some(*sequence), false)
            }
            ["random", "roll", range, sequence] => {
                require_gamemaster(permissions)?;
                random_sample(state, range, Some(*sequence), true)
            }
            ["random", "reset", "*"] => {
                require_gamemaster(permissions)?;
                let count = state.random_sequences.len() as i32;
                state.random_sequences.clear();
                Ok(CommandResult {
                    success_count: count,
                    feedback_key: "commands.random.reset.all.success",
                    broadcast_to_admins: false,
                })
            }
            ["random", "reset", "*", seed] => {
                require_gamemaster(permissions)?;
                random_reset_all(state, seed, true, true)
            }
            ["random", "reset", "*", seed, include_world_seed] => {
                require_gamemaster(permissions)?;
                random_reset_all(state, seed, parse_bool(include_world_seed)?, true)
            }
            ["random", "reset", "*", seed, include_world_seed, include_sequence_id] => {
                require_gamemaster(permissions)?;
                random_reset_all(
                    state,
                    seed,
                    parse_bool(include_world_seed)?,
                    parse_bool(include_sequence_id)?,
                )
            }
            ["random", "reset", sequence] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(state, sequence, None, true, true)
            }
            ["random", "reset", sequence, seed] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(state, sequence, Some(parse_i32(seed)?), true, true)
            }
            ["random", "reset", sequence, seed, include_world_seed] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(
                    state,
                    sequence,
                    Some(parse_i32(seed)?),
                    parse_bool(include_world_seed)?,
                    true,
                )
            }
            ["random", "reset", sequence, seed, include_world_seed, include_sequence_id] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(
                    state,
                    sequence,
                    Some(parse_i32(seed)?),
                    parse_bool(include_world_seed)?,
                    parse_bool(include_sequence_id)?,
                )
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "recipe" => recipe_command(state, &parts),
        "reload" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            let selected = discover_reload_packs(state);
            state.selected_data_packs = selected.clone();
            state.reload_requests.push(ReloadRequest {
                selected_packs: selected,
            });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.reload.success",
                broadcast_to_admins: true,
            })
        }
        "weather" => match parts.as_slice() {
            ["weather", "clear"] => set_weather(state, WeatherMode::Clear, None),
            ["weather", "clear", duration] => {
                set_weather(state, WeatherMode::Clear, Some(parse_time_ticks(duration)?))
            }
            ["weather", "rain"] => set_weather(state, WeatherMode::Rain, None),
            ["weather", "rain", duration] => {
                set_weather(state, WeatherMode::Rain, Some(parse_time_ticks(duration)?))
            }
            ["weather", "thunder"] => set_weather(state, WeatherMode::Thunder, None),
            ["weather", "thunder", duration] => set_weather(
                state,
                WeatherMode::Thunder,
                Some(parse_time_ticks(duration)?),
            ),
            _ => Err(CommandError::InvalidSyntax),
        },
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn publish_server(
    state: &mut ServerCommandState,
    port: u16,
    allow_commands: bool,
    gamemode: Option<GameMode>,
) -> Result<CommandResult, CommandError> {
    if state.published_server.is_some() {
        return Err(CommandError::PublishAlreadyPublished);
    }
    state.published_server = Some(PublishRequest {
        port,
        allow_commands,
        gamemode,
    });
    Ok(CommandResult {
        success_count: i32::from(port),
        feedback_key: "commands.publish.started",
        broadcast_to_admins: true,
    })
}

fn kick_players(
    state: &mut ServerCommandState,
    targets: &[&str],
    reason: String,
) -> Result<CommandResult, CommandError> {
    if state.published_server.is_none() {
        return Err(CommandError::KickSingleplayer);
    }

    let mut count = 0;
    for target in targets {
        let player = NameAndId::create_offline(target);
        if state
            .singleplayer_owner
            .as_ref()
            .is_some_and(|owner| owner.uuid == player.uuid)
        {
            continue;
        }
        state.disconnected_players.push(PlayerDisconnect {
            player,
            reason: reason.clone(),
        });
        count += 1;
    }

    if count == 0 {
        Err(CommandError::KickOwner)
    } else {
        Ok(CommandResult {
            success_count: count,
            feedback_key: "commands.kick.success",
            broadcast_to_admins: true,
        })
    }
}

fn kill_entities(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
) -> Result<CommandResult, CommandError> {
    let count = targets.len() as i32;
    state.killed_entities.extend(targets);
    Ok(CommandResult {
        success_count: count,
        feedback_key: if count == 1 {
            "commands.kill.success.single"
        } else {
            "commands.kill.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

fn discover_reload_packs(state: &ServerCommandState) -> Vec<String> {
    let mut selected = state.selected_data_packs.clone();
    for pack in &state.available_data_packs {
        if !state.disabled_data_packs.contains(pack) && !selected.contains(pack) {
            selected.push(pack.clone());
        }
    }
    selected
}

fn play_sound_command(
    state: &mut ServerCommandState,
    parts: &[&str],
    _permissions: LevelBasedPermissionSet,
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 {
        return Err(CommandError::InvalidSyntax);
    }
    let sound = parts[1].to_string();
    let source = parts
        .get(2)
        .map(|source| parse_sound_source(source))
        .transpose()?
        .unwrap_or(SoundSource::Master);
    let targets = if let Some(targets) = parts.get(3) {
        parse_name_list(targets)
    } else {
        state.command_source_player.clone().into_iter().collect()
    };
    if targets.is_empty() {
        return Err(CommandError::PlaySoundTooFar);
    }
    let position = if parts.len() >= 7 {
        Vec3 {
            x: parse_f64(parts[4])?,
            y: parse_f64(parts[5])?,
            z: parse_f64(parts[6])?,
        }
    } else {
        Vec3::default()
    };
    let volume = parts
        .get(7)
        .map(|value| parse_non_negative_f32(value))
        .transpose()?
        .unwrap_or(1.0);
    let pitch = parts
        .get(8)
        .map(|value| parse_bounded_f32(value, 0.0, 2.0))
        .transpose()?
        .unwrap_or(1.0);
    let min_volume = parts
        .get(9)
        .map(|value| parse_bounded_f32(value, 0.0, 1.0))
        .transpose()?
        .unwrap_or(0.0);
    if parts.len() > 10 {
        return Err(CommandError::InvalidSyntax);
    }

    let count = targets.len() as i32;
    state
        .sound_events
        .push(SoundCommandEvent::Play(PlaySoundRequest {
            sound,
            source,
            targets,
            position,
            volume,
            pitch,
            min_volume,
        }));
    Ok(CommandResult {
        success_count: count,
        feedback_key: if count == 1 {
            "commands.playsound.success.single"
        } else {
            "commands.playsound.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

fn stop_sound_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if !(2..=4).contains(&parts.len()) {
        return Err(CommandError::InvalidSyntax);
    }
    let targets = parse_name_list(parts[1]);
    let (source, sound) = match parts {
        ["stopsound", _targets] => (None, None),
        ["stopsound", _targets, "*"] => (None, None),
        ["stopsound", _targets, "*", sound] => (None, Some((*sound).to_string())),
        ["stopsound", _targets, source] => (Some(parse_sound_source(source)?), None),
        ["stopsound", _targets, source, sound] => (
            Some(parse_sound_source(source)?),
            Some((*sound).to_string()),
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };
    let count = targets.len() as i32;
    let feedback_key = match (source, sound.as_deref()) {
        (Some(_), Some(_)) => "commands.stopsound.success.source.sound",
        (Some(_), None) => "commands.stopsound.success.source.any",
        (None, Some(_)) => "commands.stopsound.success.sourceless.sound",
        (None, None) => "commands.stopsound.success.sourceless.any",
    };
    state
        .sound_events
        .push(SoundCommandEvent::Stop(StopSoundRequest {
            targets,
            source,
            sound,
        }));
    Ok(CommandResult {
        success_count: count,
        feedback_key,
        broadcast_to_admins: true,
    })
}

fn particle_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 {
        return Err(CommandError::InvalidSyntax);
    }
    let name = parts[1].to_string();
    let mut index = 2;
    let position = if parts.len() >= index + 3 && is_number(parts[index]) {
        let pos = Vec3 {
            x: parse_f64(parts[index])?,
            y: parse_f64(parts[index + 1])?,
            z: parse_f64(parts[index + 2])?,
        };
        index += 3;
        pos
    } else {
        Vec3::default()
    };

    let mut delta = Vec3::default();
    let mut speed = 0.0;
    let mut count = 0;
    if parts.len() > index {
        if parts.len() < index + 5 {
            return Err(CommandError::InvalidSyntax);
        }
        delta = Vec3 {
            x: parse_f64(parts[index])?,
            y: parse_f64(parts[index + 1])?,
            z: parse_f64(parts[index + 2])?,
        };
        speed = parse_non_negative_f32(parts[index + 3])?;
        count = parts[index + 4]
            .parse::<u32>()
            .map_err(|_| CommandError::InvalidSyntax)?;
        index += 5;
    }

    let mut force = false;
    if parts
        .get(index)
        .is_some_and(|mode| *mode == "force" || *mode == "normal")
    {
        force = parts[index] == "force";
        index += 1;
    }
    let viewers = if let Some(viewers) = parts.get(index) {
        index += 1;
        parse_name_list(viewers)
    } else {
        state.online_players.clone()
    };
    if index != parts.len() {
        return Err(CommandError::InvalidSyntax);
    }
    if viewers.is_empty() {
        return Err(CommandError::ParticleFailed);
    }

    let success_count = viewers.len() as i32;
    state.particle_events.push(ParticleCommandEvent {
        name,
        viewers,
        position,
        delta,
        speed,
        count,
        force,
    });
    Ok(CommandResult {
        success_count,
        feedback_key: "commands.particle.success",
        broadcast_to_admins: true,
    })
}

fn server_pack_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["serverpack", "push", url] => push_server_pack(state, url, None, None),
        ["serverpack", "push", url, id] => push_server_pack(state, url, Some(*id), None),
        ["serverpack", "push", url, id, hash] => {
            push_server_pack(state, url, Some(*id), Some(*hash))
        }
        ["serverpack", "pop", id] => {
            let id = parse_uuid_string(id)?;
            state
                .server_pack_events
                .push(ServerPackCommandEvent::Pop { id });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.serverpack.pop",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn push_server_pack(
    state: &mut ServerCommandState,
    url: &str,
    id: Option<&str>,
    hash: Option<&str>,
) -> Result<CommandResult, CommandError> {
    let id = match id {
        Some(id) => parse_uuid_string(id)?,
        None => name_uuid_from_bytes(url.as_bytes()),
    };
    state
        .server_pack_events
        .push(ServerPackCommandEvent::Push(ServerPackPushRequest {
            id,
            url: url.to_string(),
            hash: hash.unwrap_or_default().to_string(),
            required: false,
            prompt: None,
        }));
    Ok(CommandResult {
        success_count: 0,
        feedback_key: "commands.serverpack.push",
        broadcast_to_admins: false,
    })
}

fn perf_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["perf", "start"] => {
            if state.perf_recording {
                return Err(CommandError::PerfAlreadyRunning);
            }
            state.perf_recording = true;
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.started",
                broadcast_to_admins: false,
            })
        }
        ["perf", "stop"] => {
            if !state.perf_recording {
                return Err(CommandError::PerfNotRunning);
            }
            state.perf_recording = false;
            state.perf_reports.push(PerfReport {
                ticks: state.tick_time_samples_nanos.len() as u32,
                duration_nanos: state.tick_time_samples_nanos.iter().sum(),
            });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.stopped",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn jfr_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["jfr", "start"] => {
            if state.jfr_recording {
                return Err(CommandError::JfrStartFailed);
            }
            state.jfr_recording = true;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.jfr.started",
                broadcast_to_admins: false,
            })
        }
        ["jfr", "stop"] => {
            if !state.jfr_recording {
                return Err(CommandError::JfrDumpFailed);
            }
            state.jfr_recording = false;
            state
                .jfr_recordings
                .push(state.next_jfr_recording_path.clone());
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.jfr.stopped",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn recipe_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (mode, targets, recipes) = match parts {
        ["recipe", mode @ ("give" | "take"), targets, "*"] => {
            (*mode, parse_name_list(targets), state.known_recipes.clone())
        }
        ["recipe", mode @ ("give" | "take"), targets, recipe] => {
            (*mode, parse_name_list(targets), vec![(*recipe).to_string()])
        }
        _ => return Err(CommandError::InvalidSyntax),
    };
    if targets.is_empty() || recipes.is_empty() {
        return Err(match mode {
            "give" => CommandError::RecipeGiveFailed,
            "take" => CommandError::RecipeTakeFailed,
            _ => CommandError::InvalidSyntax,
        });
    }

    let mut success = 0;
    for target in &targets {
        let index = recipe_book_index(state, target);
        match mode {
            "give" => {
                for recipe in &recipes {
                    if !state.player_recipes[index].recipes.contains(recipe) {
                        state.player_recipes[index].recipes.push(recipe.clone());
                        success += 1;
                    }
                }
            }
            "take" => {
                for recipe in &recipes {
                    let old_len = state.player_recipes[index].recipes.len();
                    state.player_recipes[index]
                        .recipes
                        .retain(|known| known != recipe);
                    if state.player_recipes[index].recipes.len() != old_len {
                        success += 1;
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    if success == 0 {
        return Err(match mode {
            "give" => CommandError::RecipeGiveFailed,
            "take" => CommandError::RecipeTakeFailed,
            _ => CommandError::InvalidSyntax,
        });
    }

    Ok(CommandResult {
        success_count: success,
        feedback_key: match (mode, targets.len()) {
            ("give", 1) => "commands.recipe.give.success.single",
            ("give", _) => "commands.recipe.give.success.multiple",
            ("take", 1) => "commands.recipe.take.success.single",
            ("take", _) => "commands.recipe.take.success.multiple",
            _ => unreachable!(),
        },
        broadcast_to_admins: true,
    })
}

fn recipe_book_index(state: &mut ServerCommandState, player: &NameAndId) -> usize {
    if let Some(index) = state
        .player_recipes
        .iter()
        .position(|book| book.player.uuid == player.uuid)
    {
        index
    } else {
        state.player_recipes.push(PlayerRecipeBook {
            player: player.clone(),
            recipes: Vec::new(),
        });
        state.player_recipes.len() - 1
    }
}

fn rotate_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 4 {
        return Err(CommandError::InvalidSyntax);
    }
    let target = entity_ref(parts[1]);
    let mode = match parts {
        ["rotate", _target, "facing", "entity", facing_entity] => RotationMode::FacingEntity {
            entity: entity_ref(facing_entity),
            anchor: EntityAnchor::Feet,
        },
        ["rotate", _target, "facing", "entity", facing_entity, anchor] => {
            RotationMode::FacingEntity {
                entity: entity_ref(facing_entity),
                anchor: parse_entity_anchor(anchor)?,
            }
        }
        ["rotate", _target, "facing", x, y, z] => RotationMode::FacingPosition(Vec3 {
            x: parse_f64(x)?,
            y: parse_f64(y)?,
            z: parse_f64(z)?,
        }),
        ["rotate", _target, yaw, pitch] => {
            let yaw = parse_rotation_component(yaw)?;
            let pitch = parse_rotation_component(pitch)?;
            RotationMode::Angles {
                yaw: yaw.value,
                pitch: pitch.value,
                yaw_relative: yaw.relative,
                pitch_relative: pitch.relative,
            }
        }
        _ => return Err(CommandError::InvalidSyntax),
    };
    state
        .rotation_requests
        .push(RotationRequest { target, mode });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.rotate.success",
        broadcast_to_admins: true,
    })
}

fn return_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["return", "fail"] => {
            state.return_events.push(ReturnCommandEvent::Failure {
                discard_frame: true,
            });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.return.fail",
                broadcast_to_admins: false,
            })
        }
        ["return", "run", command @ ..] if !command.is_empty() => {
            let command = command.join(" ");
            state.return_events.push(ReturnCommandEvent::Run {
                command,
                discard_frame: true,
            });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.return.run",
                broadcast_to_admins: false,
            })
        }
        ["return", value] => {
            let value = parse_i32(value)?;
            state.return_events.push(ReturnCommandEvent::Success {
                value,
                discard_frame: true,
            });
            Ok(CommandResult {
                success_count: value,
                feedback_key: "commands.return.success",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn ride_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["ride", target, "mount", vehicle] => {
            mount_entity(state, entity_ref(target), entity_ref(vehicle))
        }
        ["ride", target, "dismount"] => dismount_entity(state, entity_ref(target)),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn mount_entity(
    state: &mut ServerCommandState,
    target: EntityRef,
    vehicle: EntityRef,
) -> Result<CommandResult, CommandError> {
    if mounted_vehicle(state, &target).is_some() {
        return Err(CommandError::RideAlreadyRiding);
    }
    if entity_kind(state, &vehicle) == EntityKind::Player {
        return Err(CommandError::RideMountingPlayer);
    }
    if is_self_or_passenger_of(state, &target, &vehicle) {
        return Err(CommandError::RideMountingLoop);
    }
    if entity_dimension(state, &target) != entity_dimension(state, &vehicle) {
        return Err(CommandError::RideWrongDimension);
    }

    let mount = EntityMount {
        target: target.clone(),
        vehicle: vehicle.clone(),
    };
    state.entity_mounts.push(mount);
    state
        .ride_events
        .push(RideCommandEvent::Mount { target, vehicle });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.ride.mount.success",
        broadcast_to_admins: true,
    })
}

fn dismount_entity(
    state: &mut ServerCommandState,
    target: EntityRef,
) -> Result<CommandResult, CommandError> {
    let Some(index) = state
        .entity_mounts
        .iter()
        .position(|mount| mount.target.id == target.id)
    else {
        return Err(CommandError::RideNotRiding);
    };
    let mount = state.entity_mounts.remove(index);
    state.ride_events.push(RideCommandEvent::Dismount {
        target: mount.target,
        vehicle: mount.vehicle,
    });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.ride.dismount.success",
        broadcast_to_admins: true,
    })
}

fn mounted_vehicle<'a>(state: &'a ServerCommandState, target: &EntityRef) -> Option<&'a EntityRef> {
    state
        .entity_mounts
        .iter()
        .find(|mount| mount.target.id == target.id)
        .map(|mount| &mount.vehicle)
}

fn is_self_or_passenger_of(
    state: &ServerCommandState,
    target: &EntityRef,
    candidate: &EntityRef,
) -> bool {
    if target.id == candidate.id {
        return true;
    }
    let mut current = candidate;
    let mut depth = 0;
    while let Some(vehicle) = mounted_vehicle(state, current) {
        if vehicle.id == target.id {
            return true;
        }
        current = vehicle;
        depth += 1;
        if depth > state.entity_mounts.len() {
            break;
        }
    }
    false
}

fn entity_kind(state: &ServerCommandState, entity: &EntityRef) -> EntityKind {
    if state
        .entity_states
        .iter()
        .find(|known| known.entity.id == entity.id)
        .map(|known| known.kind)
        .is_some_and(|kind| kind == EntityKind::Player)
        || state
            .online_players
            .iter()
            .any(|player| player.name == entity.id)
    {
        EntityKind::Player
    } else {
        EntityKind::Generic
    }
}

fn entity_dimension(state: &ServerCommandState, entity: &EntityRef) -> String {
    state
        .entity_states
        .iter()
        .find(|known| known.entity.id == entity.id)
        .map(|known| known.dimension.clone())
        .unwrap_or_else(|| "minecraft:overworld".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RotationComponent {
    value: f32,
    relative: bool,
}

fn parse_rotation_component(input: &str) -> Result<RotationComponent, CommandError> {
    if let Some(rest) = input.strip_prefix('~') {
        Ok(RotationComponent {
            value: if rest.is_empty() {
                0.0
            } else {
                rest.parse::<f32>()
                    .map_err(|_| CommandError::InvalidSyntax)?
            },
            relative: true,
        })
    } else {
        Ok(RotationComponent {
            value: input
                .parse::<f32>()
                .map_err(|_| CommandError::InvalidSyntax)?,
            relative: false,
        })
    }
}

fn parse_entity_anchor(input: &str) -> Result<EntityAnchor, CommandError> {
    match input {
        "feet" => Ok(EntityAnchor::Feet),
        "eyes" => Ok(EntityAnchor::Eyes),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn entity_ref(id: &str) -> EntityRef {
    EntityRef {
        id: id.to_string(),
        display_name: id.to_string(),
    }
}

fn is_number(input: &str) -> bool {
    input.parse::<f64>().is_ok()
}

fn parse_name_list(input: &str) -> Vec<NameAndId> {
    input
        .split(',')
        .filter(|name| !name.is_empty())
        .map(NameAndId::create_offline)
        .collect()
}

fn parse_uuid_string(input: &str) -> Result<String, CommandError> {
    let bytes = input.as_bytes();
    if bytes.len() != 36 || [8, 13, 18, 23].iter().any(|index| bytes[*index] != b'-') {
        return Err(CommandError::InvalidSyntax);
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| ![8, 13, 18, 23].contains(&index) && !byte.is_ascii_hexdigit())
    {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(input.to_ascii_lowercase())
}

fn name_uuid_from_bytes(input: &[u8]) -> String {
    let mut bytes = md5_digest(input);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format_uuid_bytes(bytes)
}

fn format_uuid_bytes(bytes: [u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

fn md5_digest(input: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let mut message = input.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_le_bytes());

    let mut a0 = 0x67452301u32;
    let mut b0 = 0xefcdab89u32;
    let mut c0 = 0x98badcfeu32;
    let mut d0 = 0x10325476u32;

    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 16];
        for (index, word) in words.iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_le_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let next = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(words[g])
                    .rotate_left(S[i]),
            );
            a = d;
            d = c;
            c = b;
            b = next;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut digest = [0u8; 16];
    digest[0..4].copy_from_slice(&a0.to_le_bytes());
    digest[4..8].copy_from_slice(&b0.to_le_bytes());
    digest[8..12].copy_from_slice(&c0.to_le_bytes());
    digest[12..16].copy_from_slice(&d0.to_le_bytes());
    digest
}

fn parse_sound_source(input: &str) -> Result<SoundSource, CommandError> {
    match input {
        "master" => Ok(SoundSource::Master),
        "music" => Ok(SoundSource::Music),
        "record" | "records" => Ok(SoundSource::Record),
        "weather" => Ok(SoundSource::Weather),
        "block" | "blocks" => Ok(SoundSource::Block),
        "hostile" => Ok(SoundSource::Hostile),
        "neutral" => Ok(SoundSource::Neutral),
        "player" | "players" => Ok(SoundSource::Player),
        "ambient" => Ok(SoundSource::Ambient),
        "voice" => Ok(SoundSource::Voice),
        "ui" => Ok(SoundSource::Ui),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn parse_f64(input: &str) -> Result<f64, CommandError> {
    input
        .parse::<f64>()
        .map_err(|_| CommandError::InvalidSyntax)
}

fn parse_non_negative_f32(input: &str) -> Result<f32, CommandError> {
    let value = input
        .parse::<f32>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if value < 0.0 {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(value)
    }
}

fn parse_bounded_f32(input: &str, min: f32, max: f32) -> Result<f32, CommandError> {
    let value = parse_non_negative_f32(input)?;
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub fn visible_command_usages(permissions: LevelBasedPermissionSet) -> Vec<&'static str> {
    known_command_usages()
        .iter()
        .copied()
        .filter(|(command, _usage)| permissions.can_run(command) == CommandAvailability::Available)
        .map(|(_command, usage)| usage)
        .collect()
}

pub fn command_usage(command: &str, permissions: LevelBasedPermissionSet) -> Option<&'static str> {
    known_command_usages()
        .iter()
        .find(|(root, _usage)| *root == command)
        .and_then(|(root, usage)| {
            (permissions.can_run(root) == CommandAvailability::Available).then_some(*usage)
        })
}

fn known_command_usages() -> &'static [(&'static str, &'static str)] {
    &[
        ("help", "/help [command]"),
        ("jfr", "/jfr <start|stop>"),
        ("kick", "/kick <targets> [reason]"),
        ("kill", "/kill [targets]"),
        ("list", "/list [uuids]"),
        ("msg", "/msg <targets> <message>"),
        (
            "playsound",
            "/playsound <sound> [source] [targets] [pos] [volume] [pitch] [minVolume]",
        ),
        (
            "particle",
            "/particle <name> [pos] [delta] [speed] [count] [force|normal] [viewers]",
        ),
        ("perf", "/perf <start|stop>"),
        ("publish", "/publish [allowCommands] [gamemode] [port]"),
        ("random", "/random value|roll|reset ..."),
        ("recipe", "/recipe <give|take> <targets> <recipe|*>"),
        ("reload", "/reload"),
        ("return", "/return <value>|fail|run <command>"),
        (
            "rotate",
            "/rotate <target> <rotation>|facing <entity|location>",
        ),
        ("ride", "/ride <target> mount <vehicle>|dismount"),
        ("save-all", "/save-all [flush]"),
        ("save-off", "/save-off"),
        ("save-on", "/save-on"),
        ("say", "/say <message>"),
        ("seed", "/seed"),
        (
            "serverpack",
            "/serverpack push <url> [uuid] [hash]|pop <uuid>",
        ),
        ("setidletimeout", "/setidletimeout <minutes>"),
        ("stop", "/stop"),
        ("stopsound", "/stopsound <targets> [source|*] [sound]"),
        ("teammsg", "/teammsg <message>"),
        ("tell", "/tell <targets> <message>"),
        ("tellraw", "/tellraw <targets> <message>"),
        ("tick", "/tick query|rate|step|sprint|freeze|unfreeze"),
        ("tm", "/tm <message>"),
        ("transfer", "/transfer <hostname> [port] [players]"),
        ("version", "/version"),
        ("weather", "/weather <clear|rain|thunder> [duration]"),
        ("whitelist", "/whitelist <on|off|list|add|remove|reload>"),
    ]
}

fn parse_bool(input: &str) -> Result<bool, CommandError> {
    match input {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn parse_gamemode(input: &str) -> Result<GameMode, CommandError> {
    match input {
        "survival" => Ok(GameMode::Survival),
        "creative" => Ok(GameMode::Creative),
        "adventure" => Ok(GameMode::Adventure),
        "spectator" => Ok(GameMode::Spectator),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn parse_publish_port(input: &str) -> Result<u16, CommandError> {
    input
        .parse::<u16>()
        .map_err(|_| CommandError::InvalidSyntax)
}

fn require_gamemaster(permissions: LevelBasedPermissionSet) -> Result<(), CommandError> {
    if permissions.has_permission(Permission::CommandLevel(PermissionLevel::Gamemasters)) {
        Ok(())
    } else {
        Err(CommandError::PermissionDenied)
    }
}

fn random_sample(
    state: &mut ServerCommandState,
    range: &str,
    sequence: Option<&str>,
    announced: bool,
) -> Result<CommandResult, CommandError> {
    let (min, max) = parse_int_range(range)?;
    let span = i64::from(max) - i64::from(min);
    if span == 0 {
        return Err(CommandError::RandomRangeTooSmall);
    }
    if span >= i64::from(i32::MAX) {
        return Err(CommandError::RandomRangeTooLarge);
    }

    let seed = match sequence {
        Some(id) => random_sequence_seed(state, id),
        None => state.world_seed as u64,
    };
    let next = lcg_next(seed);
    if let Some(id) = sequence {
        set_random_sequence_seed(state, id, next);
    } else {
        state.world_seed = next as i64;
    }

    let value = min + (next % (span as u64 + 1)) as i32;
    state.random_broadcasts.push(RandomSample {
        value,
        min,
        max,
        sequence: sequence.map(str::to_string),
        announced,
    });
    Ok(CommandResult {
        success_count: value,
        feedback_key: if announced {
            "commands.random.roll"
        } else {
            "commands.random.sample.success"
        },
        broadcast_to_admins: false,
    })
}

fn random_reset_all(
    state: &mut ServerCommandState,
    seed: &str,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Result<CommandResult, CommandError> {
    state.random_seed_defaults = RandomSeedDefaults {
        salt: parse_i32(seed)?,
        include_world_seed,
        include_sequence_id,
    };
    let count = state.random_sequences.len() as i32;
    state.random_sequences.clear();
    Ok(CommandResult {
        success_count: count,
        feedback_key: "commands.random.reset.all.success",
        broadcast_to_admins: false,
    })
}

fn reset_random_sequence(
    state: &mut ServerCommandState,
    sequence: &str,
    salt: Option<i32>,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Result<CommandResult, CommandError> {
    if sequence.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let salt = salt.unwrap_or(0);
    let mut seed = salt as u64;
    if include_world_seed {
        seed ^= state.world_seed as u64;
    }
    if include_sequence_id {
        seed ^= stable_hash(sequence);
    }
    set_random_sequence_seed(state, sequence, seed);
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.random.reset.success",
        broadcast_to_admins: false,
    })
}

fn random_sequence_seed(state: &mut ServerCommandState, sequence: &str) -> u64 {
    if let Some(existing) = state
        .random_sequences
        .iter()
        .find(|existing| existing.id == sequence)
    {
        existing.seed
    } else {
        let mut seed = state.random_seed_defaults.salt as u64;
        if state.random_seed_defaults.include_world_seed {
            seed ^= state.world_seed as u64;
        }
        if state.random_seed_defaults.include_sequence_id {
            seed ^= stable_hash(sequence);
        }
        state.random_sequences.push(RandomSequenceState {
            id: sequence.to_string(),
            seed,
        });
        seed
    }
}

fn set_random_sequence_seed(state: &mut ServerCommandState, sequence: &str, seed: u64) {
    if let Some(existing) = state
        .random_sequences
        .iter_mut()
        .find(|existing| existing.id == sequence)
    {
        existing.seed = seed;
    } else {
        state.random_sequences.push(RandomSequenceState {
            id: sequence.to_string(),
            seed,
        });
    }
}

fn lcg_next(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

fn stable_hash(input: &str) -> u64 {
    input.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn parse_int_range(input: &str) -> Result<(i32, i32), CommandError> {
    if let Some((min, max)) = input.split_once("..") {
        let min = if min.is_empty() {
            i32::MIN
        } else {
            parse_i32(min)?
        };
        let max = if max.is_empty() {
            i32::MAX
        } else {
            parse_i32(max)?
        };
        if min > max {
            return Err(CommandError::InvalidSyntax);
        }
        Ok((min, max))
    } else {
        let value = parse_i32(input)?;
        Ok((value, value))
    }
}

fn parse_i32(input: &str) -> Result<i32, CommandError> {
    input
        .parse::<i32>()
        .map_err(|_| CommandError::InvalidSyntax)
}

fn queue_transfer(
    state: &mut ServerCommandState,
    host: &str,
    port: u16,
    targets: Vec<NameAndId>,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() {
        return Err(CommandError::NoPlayers);
    }
    let success_count = targets.len() as i32;
    state.transfer_requests.push(TransferRequest {
        host: host.to_string(),
        port,
        targets,
    });
    Ok(CommandResult {
        success_count,
        feedback_key: if success_count == 1 {
            "commands.transfer.success.single"
        } else {
            "commands.transfer.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

fn parse_port(input: &str) -> Result<u16, CommandError> {
    let port = input
        .parse::<u16>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if port == 0 {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(port)
    }
}

fn set_weather(
    state: &mut ServerCommandState,
    mode: WeatherMode,
    duration_ticks: Option<u32>,
) -> Result<CommandResult, CommandError> {
    state.weather = WeatherState {
        mode,
        duration_ticks,
    };
    Ok(CommandResult {
        success_count: duration_ticks.map(|ticks| ticks as i32).unwrap_or(-1),
        feedback_key: match mode {
            WeatherMode::Clear => "commands.weather.set.clear",
            WeatherMode::Rain => "commands.weather.set.rain",
            WeatherMode::Thunder => "commands.weather.set.thunder",
        },
        broadcast_to_admins: true,
    })
}

fn tick_query_status_key(state: &ServerCommandState) -> &'static str {
    if state.tick_rate.is_sprinting() {
        "commands.tick.status.sprinting"
    } else if state.tick_rate.is_frozen() {
        "commands.tick.status.frozen"
    } else if state.tick_rate.tick_duration().as_nanos() < state.average_tick_time_nanos as u128 {
        "commands.tick.status.lagging"
    } else {
        "commands.tick.status.running"
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            mode: WeatherMode::Clear,
            duration_ticks: None,
        }
    }
}

impl Default for RandomSeedDefaults {
    fn default() -> Self {
        Self {
            salt: 0,
            include_world_seed: true,
            include_sequence_id: true,
        }
    }
}

fn tick_step(state: &mut ServerCommandState, ticks: u32) -> Result<CommandResult, CommandError> {
    let success = state.tick_rate.step_game_if_paused(ticks);
    Ok(CommandResult {
        success_count: 1,
        feedback_key: if success {
            "commands.tick.step.success"
        } else {
            "commands.tick.step.fail"
        },
        broadcast_to_admins: success,
    })
}

fn parse_time_ticks(input: &str) -> Result<u32, CommandError> {
    let (number, multiplier) = if let Some(number) = input.strip_suffix('t') {
        (number, 1)
    } else if let Some(number) = input.strip_suffix('s') {
        (number, 20)
    } else if let Some(number) = input.strip_suffix('d') {
        (number, 24_000)
    } else {
        (input, 1)
    };
    let ticks = number
        .parse::<u32>()
        .ok()
        .and_then(|number| number.checked_mul(multiplier))
        .filter(|ticks| *ticks >= 1)
        .ok_or(CommandError::InvalidSyntax)?;
    Ok(ticks)
}

pub fn command_required_permission(command: &str) -> PermissionLevel {
    match command {
        "" | "help" | "list" | "me" | "msg" | "random" | "teammsg" | "tell" | "tm" | "trigger"
        | "w" | "version" => PermissionLevel::All,
        "ban" | "ban-ip" | "banlist" | "deop" | "debug" | "debugconfig" | "kick" | "op"
        | "pardon" | "pardon-ip" | "setidletimeout" | "tick" | "transfer" | "whitelist" => {
            PermissionLevel::Admins
        }
        "jfr" | "perf" | "publish" | "save-all" | "save-off" | "save-on" | "stop" => {
            PermissionLevel::Owners
        }
        _ => PermissionLevel::Gamemasters,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        command_required_permission, command_usage, execute_builtin_command,
        visible_command_usages, ChatCommandKind, CommandAvailability, CommandError, EntityAnchor,
        EntityKind, EntityMount, EntityRef, EntityState, GameMode, LevelBasedPermissionSet,
        ParticleCommandEvent, PerfReport, Permission, PermissionLevel, PlaySoundRequest,
        PlayerRecipeBook, PublishRequest, ReloadRequest, ReturnCommandEvent, RideCommandEvent,
        RotationMode, RotationRequest, SaveAllRequest, ServerCommandState, ServerPackCommandEvent,
        ServerPackPushRequest, SoundCommandEvent, SoundSource, StopSoundRequest, TeamMembership,
        Vec3, VersionInfo, WeatherMode,
    };
    use crate::player_access::NameAndId;

    #[test]
    fn permission_levels_clamp_and_compare_like_vanilla() {
        assert_eq!(PermissionLevel::by_id(-5), PermissionLevel::All);
        assert_eq!(PermissionLevel::by_id(0), PermissionLevel::All);
        assert_eq!(PermissionLevel::by_id(2), PermissionLevel::Gamemasters);
        assert_eq!(PermissionLevel::by_id(99), PermissionLevel::Owners);
        assert!(PermissionLevel::Admins.is_equal_or_higher_than(PermissionLevel::Gamemasters));
        assert!(!PermissionLevel::Moderators.is_equal_or_higher_than(PermissionLevel::Admins));
        assert_eq!(PermissionLevel::Owners.serialized_name(), "owners");
    }

    #[test]
    fn level_based_permission_set_grants_command_levels_and_entity_selectors() {
        assert!(LevelBasedPermissionSet::GAMEMASTER
            .has_permission(Permission::CommandLevel(PermissionLevel::Gamemasters)));
        assert!(!LevelBasedPermissionSet::GAMEMASTER
            .has_permission(Permission::CommandLevel(PermissionLevel::Admins)));
        assert!(
            LevelBasedPermissionSet::GAMEMASTER.has_permission(Permission::CommandsEntitySelectors)
        );
        assert!(
            !LevelBasedPermissionSet::MODERATOR.has_permission(Permission::CommandsEntitySelectors)
        );
    }

    #[test]
    fn command_availability_uses_root_literal_and_permission_level() {
        assert_eq!(command_required_permission("stop"), PermissionLevel::Owners);
        assert_eq!(command_required_permission("op"), PermissionLevel::Admins);
        assert_eq!(
            command_required_permission("execute"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(command_required_permission("list"), PermissionLevel::All);

        assert_eq!(
            LevelBasedPermissionSet::ALL.can_run("/list"),
            CommandAvailability::Available
        );
        assert_eq!(
            LevelBasedPermissionSet::GAMEMASTER.can_run("/stop"),
            CommandAvailability::Hidden
        );
        assert_eq!(
            LevelBasedPermissionSet::ADMIN.can_run("/op Steve"),
            CommandAvailability::Available
        );
        assert_eq!(
            LevelBasedPermissionSet::OWNER.can_run("save-all flush"),
            CommandAvailability::Available
        );
    }

    #[test]
    fn set_idle_timeout_command_updates_minutes_and_feedback() {
        let mut state = ServerCommandState::default();
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "/setidletimeout 5",
        )
        .unwrap();
        assert_eq!(state.player_idle_timeout_minutes, 5);
        assert_eq!(result.success_count, 5);
        assert_eq!(result.feedback_key, "commands.setidletimeout.success");

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "setidletimeout 0",
        )
        .unwrap();
        assert_eq!(state.player_idle_timeout_minutes, 0);
        assert_eq!(
            result.feedback_key,
            "commands.setidletimeout.success.disabled"
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "setidletimeout 1",
            ),
            Err(CommandError::PermissionDenied)
        );
    }

    #[test]
    fn save_commands_track_flush_and_autosave_state() {
        let mut state = ServerCommandState::default();
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-all").unwrap();
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-all flush")
            .unwrap();
        assert_eq!(
            state.save_all_requests,
            vec![
                SaveAllRequest { flush: false },
                SaveAllRequest { flush: true }
            ]
        );

        let off = execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-off")
            .unwrap();
        assert!(!state.autosave_enabled);
        assert_eq!(off.feedback_key, "commands.save.disabled");
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-off"),
            Err(CommandError::SaveAlreadyOff)
        );

        let on =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-on").unwrap();
        assert!(state.autosave_enabled);
        assert_eq!(on.feedback_key, "commands.save.enabled");
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-on"),
            Err(CommandError::SaveAlreadyOn)
        );
    }

    #[test]
    fn stop_command_requests_halt_and_requires_owner() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "stop"),
            Err(CommandError::PermissionDenied)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "/stop").unwrap();
        assert!(state.halt_requested);
        assert_eq!(result.success_count, 1);
        assert_eq!(result.feedback_key, "commands.stop.stopping");
    }

    #[test]
    fn seed_command_reports_level_seed_with_gamemaster_permission() {
        let mut state = ServerCommandState {
            world_seed: 8_675_309,
            ..ServerCommandState::default()
        };
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "seed"),
            Err(CommandError::PermissionDenied)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "/seed")
                .unwrap();
        assert_eq!(result.success_count, 8_675_309);
        assert_eq!(result.feedback_key, "commands.seed.success");
        assert!(!result.broadcast_to_admins);
    }

    #[test]
    fn serverpack_push_generates_java_name_uuid_or_uses_explicit_uuid() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("serverpack"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "serverpack push https://example.invalid/pack.zip"
            ),
            Err(CommandError::PermissionDenied)
        );

        let generated = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "serverpack push https://example.invalid/pack.zip",
        )
        .unwrap();
        assert_eq!(generated.success_count, 0);
        assert_eq!(generated.feedback_key, "commands.serverpack.push");
        assert_eq!(
            state.server_pack_events[0],
            ServerPackCommandEvent::Push(ServerPackPushRequest {
                id: "f2dfd86d-3bee-3650-9f80-4317b330dccc".to_string(),
                url: "https://example.invalid/pack.zip".to_string(),
                hash: String::new(),
                required: false,
                prompt: None,
            })
        );

        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "serverpack push https://example.invalid/pack2.zip 00000000-0000-3000-8000-000000000001 abc123",
        )
        .unwrap();
        assert_eq!(
            state.server_pack_events[1],
            ServerPackCommandEvent::Push(ServerPackPushRequest {
                id: "00000000-0000-3000-8000-000000000001".to_string(),
                url: "https://example.invalid/pack2.zip".to_string(),
                hash: "abc123".to_string(),
                required: false,
                prompt: None,
            })
        );
    }

    #[test]
    fn serverpack_pop_records_uuid_and_rejects_bad_syntax() {
        let mut state = ServerCommandState::default();
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "serverpack pop AAAAAAAA-BBBB-3CCC-8DDD-EEEEEEEEEEEE",
        )
        .unwrap();
        assert_eq!(result.success_count, 0);
        assert_eq!(result.feedback_key, "commands.serverpack.pop");
        assert_eq!(
            state.server_pack_events,
            vec![ServerPackCommandEvent::Pop {
                id: "aaaaaaaa-bbbb-3ccc-8ddd-eeeeeeeeeeee".to_string(),
            }]
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "serverpack pop not-a-uuid"
            ),
            Err(CommandError::InvalidSyntax)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "serverpack"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn version_command_reports_26_1_2_metadata() {
        let mut state = ServerCommandState::default();
        let lines = VersionInfo::CURRENT_26_1_2.command_lines();
        assert!(lines.contains(&"commands.version.id 26.1.2".to_string()));
        assert!(lines.contains(&"commands.version.data 4790".to_string()));
        assert!(lines.contains(&"commands.version.protocol 775 0x307".to_string()));
        assert!(lines.contains(&"commands.version.pack.resource 84".to_string()));
        assert!(lines.contains(&"commands.version.pack.data 101.1".to_string()));
        assert!(lines.contains(&"commands.version.stable.yes".to_string()));

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "/version").unwrap();
        assert_eq!(result.success_count, lines.len() as i32);
        assert_eq!(result.feedback_key, "commands.version.header");
        assert!(!result.broadcast_to_admins);
    }

    #[test]
    fn whitelist_command_requires_admin_permission() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "whitelist on"
            ),
            Err(CommandError::PermissionDenied)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist on")
                .unwrap();
        assert!(state.whitelist_enabled);
        assert_eq!(state.kick_unlisted_requests, 1);
        assert_eq!(result.feedback_key, "commands.whitelist.enabled");
    }

    #[test]
    fn whitelist_on_off_match_vanilla_errors_and_feedback() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist off"),
            Err(CommandError::WhitelistAlreadyOff)
        );

        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist on")
            .unwrap();
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist on"),
            Err(CommandError::WhitelistAlreadyOn)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist off")
                .unwrap();
        assert!(!state.whitelist_enabled);
        assert_eq!(state.kick_unlisted_requests, 1);
        assert_eq!(result.success_count, 1);
        assert_eq!(result.feedback_key, "commands.whitelist.disabled");
    }

    #[test]
    fn whitelist_add_remove_and_list_track_profiles() {
        let mut state = ServerCommandState::default();
        let empty =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist list")
                .unwrap();
        assert_eq!(empty.success_count, 0);
        assert_eq!(empty.feedback_key, "commands.whitelist.none");

        let added = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "whitelist add Steve Alex",
        )
        .unwrap();
        assert_eq!(added.success_count, 2);
        assert_eq!(added.feedback_key, "commands.whitelist.add.success");
        assert_eq!(state.whitelist_names(), vec!["Steve", "Alex"]);
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::ADMIN,
                "whitelist add Steve"
            ),
            Err(CommandError::AlreadyWhitelisted)
        );

        let list =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist list")
                .unwrap();
        assert_eq!(list.success_count, 2);
        assert_eq!(list.feedback_key, "commands.whitelist.list");
        assert!(!list.broadcast_to_admins);

        let removed = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "whitelist remove Alex",
        )
        .unwrap();
        assert_eq!(removed.success_count, 1);
        assert_eq!(removed.feedback_key, "commands.whitelist.remove.success");
        assert_eq!(state.whitelist_names(), vec!["Steve"]);
        assert_eq!(state.kick_unlisted_requests, 1);
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::ADMIN,
                "whitelist remove Alex"
            ),
            Err(CommandError::NotWhitelisted)
        );
    }

    #[test]
    fn whitelist_reload_requests_storage_reload_and_kick() {
        let mut state = ServerCommandState::default();
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "whitelist reload",
        )
        .unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(result.feedback_key, "commands.whitelist.reloaded");
        assert_eq!(state.whitelist_reload_requests, 1);
        assert_eq!(state.kick_unlisted_requests, 1);
    }

    #[test]
    fn tick_rate_command_matches_admin_range_and_feedback() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "tick rate 40"
            ),
            Err(CommandError::PermissionDenied)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick rate 0.5"),
            Err(CommandError::InvalidSyntax)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick rate 40.5")
                .unwrap();
        assert_eq!(result.success_count, 40);
        assert_eq!(result.feedback_key, "commands.tick.rate.success");
        assert_eq!(state.tick_rate.tick_rate(), 40.5);
    }

    #[test]
    fn tick_freeze_unfreeze_step_and_stop_use_tick_controller() {
        let mut state = ServerCommandState::default();
        let failed_step =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick step")
                .unwrap();
        assert_eq!(failed_step.success_count, 1);
        assert_eq!(failed_step.feedback_key, "commands.tick.step.fail");
        assert!(!failed_step.broadcast_to_admins);

        let frozen =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick freeze")
                .unwrap();
        assert!(state.tick_rate.is_frozen());
        assert_eq!(frozen.success_count, 1);
        assert_eq!(frozen.feedback_key, "commands.tick.status.frozen");

        let stepped =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick step 1s")
                .unwrap();
        assert_eq!(stepped.feedback_key, "commands.tick.step.success");
        assert_eq!(state.tick_rate.frozen_ticks_to_run(), 20);

        let stopped =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick step stop")
                .unwrap();
        assert_eq!(stopped.success_count, 1);
        assert_eq!(stopped.feedback_key, "commands.tick.step.stop.success");
        assert_eq!(state.tick_rate.frozen_ticks_to_run(), 0);

        let unfrozen =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick unfreeze")
                .unwrap();
        assert!(!state.tick_rate.is_frozen());
        assert_eq!(unfrozen.success_count, 0);
        assert_eq!(unfrozen.feedback_key, "commands.tick.status.running");
    }

    #[test]
    fn tick_sprint_start_and_stop_use_time_arguments() {
        let mut state = ServerCommandState::default();
        let started =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick sprint 3d")
                .unwrap();
        assert!(state.tick_rate.is_sprinting());
        assert_eq!(started.success_count, 1);
        assert_eq!(started.feedback_key, "commands.tick.status.sprinting");

        let stopped = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "tick sprint stop",
        )
        .unwrap();
        assert!(!state.tick_rate.is_sprinting());
        assert_eq!(stopped.success_count, 1);
        assert_eq!(stopped.feedback_key, "commands.tick.sprint.stop.success");

        let stopped_again = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "tick sprint stop",
        )
        .unwrap();
        assert_eq!(stopped_again.success_count, 0);
        assert_eq!(stopped_again.feedback_key, "commands.tick.sprint.stop.fail");
    }

    #[test]
    fn tick_query_reports_sprinting_frozen_lagging_or_running_status() {
        let mut state = ServerCommandState::default();
        let running =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query")
                .unwrap();
        assert_eq!(running.success_count, 20);
        assert_eq!(running.feedback_key, "commands.tick.status.running");
        assert!(!running.broadcast_to_admins);

        state.average_tick_time_nanos = 60_000_000;
        let lagging =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query")
                .unwrap();
        assert_eq!(lagging.feedback_key, "commands.tick.status.lagging");

        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick freeze").unwrap();
        let frozen =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query")
                .unwrap();
        assert_eq!(frozen.feedback_key, "commands.tick.status.frozen");

        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick sprint 1t")
            .unwrap();
        let sprinting =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query")
                .unwrap();
        assert_eq!(sprinting.feedback_key, "commands.tick.status.sprinting");
    }

    #[test]
    fn transfer_command_requires_admin_and_valid_port() {
        let mut state = ServerCommandState::default();
        state.command_source_player = Some(NameAndId::create_offline("Steve"));
        assert_eq!(
            command_required_permission("transfer"),
            PermissionLevel::Admins
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "transfer example.org"
            ),
            Err(CommandError::PermissionDenied)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::ADMIN,
                "transfer example.org 0"
            ),
            Err(CommandError::InvalidSyntax)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::ADMIN,
                "transfer example.org 65536"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn transfer_command_defaults_to_source_player_and_port() {
        let mut state = ServerCommandState {
            command_source_player: Some(NameAndId::create_offline("Steve")),
            ..ServerCommandState::default()
        };
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "transfer mc.test",
        )
        .unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(result.feedback_key, "commands.transfer.success.single");
        assert_eq!(state.transfer_requests.len(), 1);
        assert_eq!(state.transfer_requests[0].host, "mc.test");
        assert_eq!(state.transfer_requests[0].port, 25565);
        assert_eq!(state.transfer_requests[0].targets[0].name, "Steve");
    }

    #[test]
    fn transfer_command_targets_explicit_players() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::ADMIN,
                "transfer mc.test"
            ),
            Err(CommandError::NoPlayers)
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "transfer mc.test 25566 Steve Alex",
        )
        .unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(result.feedback_key, "commands.transfer.success.multiple");
        assert_eq!(state.transfer_requests.len(), 1);
        assert_eq!(state.transfer_requests[0].host, "mc.test");
        assert_eq!(state.transfer_requests[0].port, 25566);
        assert_eq!(
            state.transfer_requests[0]
                .targets
                .iter()
                .map(|target| target.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Steve", "Alex"]
        );
    }

    #[test]
    fn weather_command_uses_gamemaster_permission_and_vanilla_feedback() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("weather"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "weather rain"
            ),
            Err(CommandError::PermissionDenied)
        );

        let rain = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "weather rain",
        )
        .unwrap();
        assert_eq!(state.weather.mode, WeatherMode::Rain);
        assert_eq!(state.weather.duration_ticks, None);
        assert_eq!(rain.success_count, -1);
        assert_eq!(rain.feedback_key, "commands.weather.set.rain");
        assert!(rain.broadcast_to_admins);
    }

    #[test]
    fn weather_command_accepts_clear_rain_thunder_with_time_arguments() {
        let mut state = ServerCommandState::default();
        let clear = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "weather clear 1s",
        )
        .unwrap();
        assert_eq!(state.weather.mode, WeatherMode::Clear);
        assert_eq!(state.weather.duration_ticks, Some(20));
        assert_eq!(clear.success_count, 20);
        assert_eq!(clear.feedback_key, "commands.weather.set.clear");

        let thunder = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "weather thunder 2d",
        )
        .unwrap();
        assert_eq!(state.weather.mode, WeatherMode::Thunder);
        assert_eq!(state.weather.duration_ticks, Some(48_000));
        assert_eq!(thunder.success_count, 48_000);
        assert_eq!(thunder.feedback_key, "commands.weather.set.thunder");

        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "weather rain 0"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn publish_command_requires_owner_and_records_default_publish_request() {
        let mut state = ServerCommandState {
            next_available_publish_port: 24454,
            ..ServerCommandState::default()
        };
        assert_eq!(
            command_required_permission("publish"),
            PermissionLevel::Owners
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "publish"),
            Err(CommandError::PermissionDenied)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "publish").unwrap();
        assert_eq!(result.success_count, 24454);
        assert_eq!(result.feedback_key, "commands.publish.started");
        assert!(result.broadcast_to_admins);
        assert_eq!(
            state.published_server,
            Some(super::PublishRequest {
                port: 24454,
                allow_commands: false,
                gamemode: None,
            })
        );
    }

    #[test]
    fn publish_command_parses_allow_commands_gamemode_and_port() {
        let mut state = ServerCommandState::default();
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::OWNER,
            "publish true creative 0",
        )
        .unwrap();
        assert_eq!(result.success_count, 0);
        assert_eq!(
            state.published_server,
            Some(super::PublishRequest {
                port: 0,
                allow_commands: true,
                gamemode: Some(GameMode::Creative),
            })
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::OWNER,
                "publish false survival"
            ),
            Err(CommandError::PublishAlreadyPublished)
        );
    }

    #[test]
    fn publish_command_rejects_invalid_bool_gamemode_or_port() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "publish yes"),
            Err(CommandError::InvalidSyntax)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::OWNER,
                "publish true builder"
            ),
            Err(CommandError::InvalidSyntax)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::OWNER,
                "publish true creative 65536"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn random_value_and_roll_sample_ranges_without_permission() {
        let mut state = ServerCommandState {
            world_seed: 123,
            ..ServerCommandState::default()
        };
        assert_eq!(command_required_permission("random"), PermissionLevel::All);

        let value = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "random value 1..6",
        )
        .unwrap();
        assert!((1..=6).contains(&value.success_count));
        assert_eq!(value.feedback_key, "commands.random.sample.success");
        assert!(!value.broadcast_to_admins);
        assert_eq!(state.random_broadcasts[0].announced, false);

        let roll = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "random roll -2..2",
        )
        .unwrap();
        assert!((-2..=2).contains(&roll.success_count));
        assert_eq!(roll.feedback_key, "commands.random.roll");
        assert_eq!(state.random_broadcasts[1].announced, true);
    }

    #[test]
    fn random_command_rejects_vanilla_range_errors() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "random value 5"),
            Err(CommandError::RandomRangeTooSmall)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "random value .."),
            Err(CommandError::RandomRangeTooLarge)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::ALL,
                "random value 6..1"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn random_named_sequences_and_resets_require_gamemaster() {
        let mut state = ServerCommandState {
            world_seed: 99,
            ..ServerCommandState::default()
        };
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::ALL,
                "random value 1..10 minecraft:test"
            ),
            Err(CommandError::PermissionDenied)
        );

        let sample = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "random value 1..10 minecraft:test",
        )
        .unwrap();
        assert!((1..=10).contains(&sample.success_count));
        assert_eq!(state.random_sequences.len(), 1);
        assert_eq!(state.random_sequences[0].id, "minecraft:test");
        assert_eq!(
            state.random_broadcasts[0].sequence.as_deref(),
            Some("minecraft:test")
        );

        let reset = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "random reset minecraft:test 42 false true",
        )
        .unwrap();
        assert_eq!(reset.success_count, 1);
        assert_eq!(reset.feedback_key, "commands.random.reset.success");
        assert_eq!(state.random_sequences.len(), 1);

        let reset_all = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "random reset * 7 true false",
        )
        .unwrap();
        assert_eq!(reset_all.success_count, 1);
        assert!(state.random_sequences.is_empty());
        assert_eq!(
            state.random_seed_defaults,
            super::RandomSeedDefaults {
                salt: 7,
                include_world_seed: true,
                include_sequence_id: false,
            }
        );
    }

    #[test]
    fn recipe_command_gives_and_takes_single_or_all_recipes() {
        let mut state = ServerCommandState {
            known_recipes: vec![
                "minecraft:planks".to_string(),
                "minecraft:stick".to_string(),
            ],
            ..ServerCommandState::default()
        };
        assert_eq!(
            command_required_permission("recipe"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "recipe give Steve minecraft:planks"
            ),
            Err(CommandError::PermissionDenied)
        );

        let given = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "recipe give Steve minecraft:planks",
        )
        .unwrap();
        assert_eq!(given.success_count, 1);
        assert_eq!(given.feedback_key, "commands.recipe.give.success.single");
        assert_eq!(
            state.player_recipes,
            vec![PlayerRecipeBook {
                player: NameAndId::create_offline("Steve"),
                recipes: vec!["minecraft:planks".to_string()],
            }]
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "recipe give Steve minecraft:planks"
            ),
            Err(CommandError::RecipeGiveFailed)
        );

        let all = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "recipe give Steve,Alex *",
        )
        .unwrap();
        assert_eq!(all.success_count, 3);
        assert_eq!(all.feedback_key, "commands.recipe.give.success.multiple");

        let taken = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "recipe take Steve minecraft:stick",
        )
        .unwrap();
        assert_eq!(taken.success_count, 1);
        assert_eq!(taken.feedback_key, "commands.recipe.take.success.single");

        let taken_all = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "recipe take Steve,Alex *",
        )
        .unwrap();
        assert_eq!(taken_all.success_count, 3);
        assert_eq!(
            taken_all.feedback_key,
            "commands.recipe.take.success.multiple"
        );
    }

    #[test]
    fn recipe_command_fails_when_no_recipes_change() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "recipe give Steve *"
            ),
            Err(CommandError::RecipeGiveFailed)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "recipe take Steve minecraft:stick"
            ),
            Err(CommandError::RecipeTakeFailed)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "recipe"),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn list_command_reports_online_player_count_for_all_sources() {
        let mut state = ServerCommandState {
            online_players: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex"),
            ],
            max_players: 40,
            ..ServerCommandState::default()
        };
        assert_eq!(command_required_permission("list"), PermissionLevel::All);

        let list =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list").unwrap();
        assert_eq!(list.success_count, 2);
        assert_eq!(list.feedback_key, "commands.list.players");
        assert!(!list.broadcast_to_admins);

        let list_uuids =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list uuids")
                .unwrap();
        assert_eq!(list_uuids.success_count, 2);
        assert_eq!(list_uuids.feedback_key, "commands.list.players");
    }

    #[test]
    fn kick_command_requires_admin_published_server_and_non_owner_target() {
        let mut state = ServerCommandState::default();
        assert_eq!(command_required_permission("kick"), PermissionLevel::Admins);
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "kick Steve"
            ),
            Err(CommandError::PermissionDenied)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "kick Steve"),
            Err(CommandError::KickSingleplayer)
        );

        let owner = NameAndId::create_offline("Steve");
        state.singleplayer_owner = Some(owner);
        state.published_server = Some(PublishRequest {
            port: 25565,
            allow_commands: false,
            gamemode: None,
        });
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "kick Steve"),
            Err(CommandError::KickOwner)
        );
    }

    #[test]
    fn kick_command_disconnects_targets_with_default_or_custom_reason() {
        let mut state = ServerCommandState {
            published_server: Some(PublishRequest {
                port: 25565,
                allow_commands: false,
                gamemode: None,
            }),
            ..ServerCommandState::default()
        };
        let kicked =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "kick Steve")
                .unwrap();
        assert_eq!(kicked.success_count, 1);
        assert_eq!(kicked.feedback_key, "commands.kick.success");
        assert_eq!(
            state.disconnected_players[0].reason,
            "multiplayer.disconnect.kicked"
        );

        let kicked = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "kick Alex -- maintenance window",
        )
        .unwrap();
        assert_eq!(kicked.success_count, 1);
        assert_eq!(state.disconnected_players[1].player.name, "Alex");
        assert_eq!(state.disconnected_players[1].reason, "maintenance window");
    }

    #[test]
    fn kill_command_requires_gamemaster_and_defaults_to_source_entity() {
        let mut state = ServerCommandState {
            command_source_entity: Some(EntityRef {
                id: "source".to_string(),
                display_name: "Source".to_string(),
            }),
            ..ServerCommandState::default()
        };
        assert_eq!(
            command_required_permission("kill"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "kill"),
            Err(CommandError::PermissionDenied)
        );

        let killed =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "kill")
                .unwrap();
        assert_eq!(killed.success_count, 1);
        assert_eq!(killed.feedback_key, "commands.kill.success.single");
        assert_eq!(state.killed_entities[0].id, "source");
    }

    #[test]
    fn kill_command_accepts_multiple_targets() {
        let mut state = ServerCommandState::default();
        let killed = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "kill zombie creeper",
        )
        .unwrap();
        assert_eq!(killed.success_count, 2);
        assert_eq!(killed.feedback_key, "commands.kill.success.multiple");
        assert_eq!(
            state
                .killed_entities
                .iter()
                .map(|entity| entity.id.as_str())
                .collect::<Vec<_>>(),
            vec!["zombie", "creeper"]
        );
    }

    #[test]
    fn help_command_lists_visible_root_usages() {
        let mut state = ServerCommandState::default();
        assert_eq!(command_required_permission("help"), PermissionLevel::All);

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "help").unwrap();
        let all_usages = visible_command_usages(LevelBasedPermissionSet::ALL);
        assert_eq!(result.success_count, all_usages.len() as i32);
        assert!(all_usages.contains(&"/help [command]"));
        assert!(all_usages.contains(&"/list [uuids]"));
        assert!(!all_usages.contains(&"/stop"));
        assert_eq!(result.feedback_key, "commands.help.success");
        assert!(!result.broadcast_to_admins);
    }

    #[test]
    fn help_command_reports_specific_visible_command_or_failure() {
        let mut state = ServerCommandState::default();
        let help =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "help list").unwrap();
        assert_eq!(help.success_count, 1);
        assert_eq!(
            command_usage("list", LevelBasedPermissionSet::ALL),
            Some("/list [uuids]")
        );

        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "help stop"),
            Err(CommandError::HelpFailed)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "help stop"),
            Ok(super::CommandResult {
                success_count: 1,
                feedback_key: "commands.help.success",
                broadcast_to_admins: false,
            })
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "help missing"),
            Err(CommandError::HelpFailed)
        );
    }

    #[test]
    fn reload_command_requires_gamemaster_and_returns_zero() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("reload"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "reload"),
            Err(CommandError::PermissionDenied)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "reload")
                .unwrap();
        assert_eq!(result.success_count, 0);
        assert_eq!(result.feedback_key, "commands.reload.success");
        assert!(result.broadcast_to_admins);
        assert_eq!(state.reload_requests.len(), 1);
    }

    #[test]
    fn reload_command_discovers_new_packs_without_enabling_disabled_packs() {
        let mut state = ServerCommandState {
            available_data_packs: vec![
                "vanilla".to_string(),
                "kept".to_string(),
                "new_pack".to_string(),
                "disabled_pack".to_string(),
            ],
            selected_data_packs: vec!["vanilla".to_string(), "kept".to_string()],
            disabled_data_packs: vec!["disabled_pack".to_string()],
            ..ServerCommandState::default()
        };
        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "reload")
                .unwrap();
        assert_eq!(result.success_count, 0);
        assert_eq!(
            state.selected_data_packs,
            vec!["vanilla", "kept", "new_pack"]
        );
        assert_eq!(
            state.reload_requests,
            vec![ReloadRequest {
                selected_packs: vec![
                    "vanilla".to_string(),
                    "kept".to_string(),
                    "new_pack".to_string(),
                ],
            }]
        );
    }

    #[test]
    fn say_command_requires_gamemaster_and_broadcasts_to_online_players() {
        let mut state = ServerCommandState {
            command_source_player: Some(NameAndId::create_offline("Console")),
            online_players: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex"),
            ],
            ..ServerCommandState::default()
        };
        assert_eq!(
            command_required_permission("say"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "say hello"),
            Err(CommandError::PermissionDenied)
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "say hello all",
        )
        .unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(state.chat_events.len(), 1);
        assert_eq!(state.chat_events[0].kind, ChatCommandKind::Say);
        assert_eq!(state.chat_events[0].targets.len(), 2);
        assert_eq!(state.chat_events[0].message, "hello all");
    }

    #[test]
    fn msg_tell_and_w_send_private_messages_without_permission_gate() {
        let mut state = ServerCommandState {
            command_source_player: Some(NameAndId::create_offline("Steve")),
            ..ServerCommandState::default()
        };
        assert_eq!(command_required_permission("msg"), PermissionLevel::All);

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "msg Alex -- secret plan",
        )
        .unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(state.chat_events[0].kind, ChatCommandKind::Private);
        assert_eq!(state.chat_events[0].targets[0].name, "Alex");
        assert_eq!(state.chat_events[0].message, "secret plan");

        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "tell Steve hello")
            .unwrap();
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "w Alex hi").unwrap();
        assert_eq!(state.chat_events.len(), 3);
    }

    #[test]
    fn teammsg_requires_source_team_and_targets_team_members() {
        let steve = NameAndId::create_offline("Steve");
        let alex = NameAndId::create_offline("Alex");
        let mut state = ServerCommandState {
            command_source_player: Some(steve.clone()),
            player_teams: vec![
                TeamMembership {
                    player: steve,
                    team: "red".to_string(),
                },
                TeamMembership {
                    player: alex,
                    team: "red".to_string(),
                },
                TeamMembership {
                    player: NameAndId::create_offline("Bob"),
                    team: "blue".to_string(),
                },
            ],
            ..ServerCommandState::default()
        };
        assert_eq!(command_required_permission("teammsg"), PermissionLevel::All);

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "teammsg push left",
        )
        .unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(state.chat_events[0].kind, ChatCommandKind::Team);
        assert_eq!(state.chat_events[0].message, "push left");
        assert_eq!(
            state.chat_events[0]
                .targets
                .iter()
                .map(|target| target.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Steve", "Alex"]
        );

        state.command_source_player = Some(NameAndId::create_offline("NoTeam"));
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "tm hello"),
            Err(CommandError::TeamMsgNoTeam)
        );
    }

    #[test]
    fn tellraw_requires_gamemaster_and_sends_system_message_to_targets() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("tellraw"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "tellraw Steve {\"text\":\"hi\"}"
            ),
            Err(CommandError::PermissionDenied)
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "tellraw Steve Alex -- {\"text\":\"hi\"}",
        )
        .unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(state.chat_events[0].kind, ChatCommandKind::TellRaw);
        assert_eq!(state.chat_events[0].message, "{\"text\":\"hi\"}");
        assert_eq!(state.chat_events[0].targets.len(), 2);
    }

    #[test]
    fn playsound_requires_gamemaster_and_defaults_to_source_player() {
        let mut state = ServerCommandState {
            command_source_player: Some(NameAndId::create_offline("Steve")),
            ..ServerCommandState::default()
        };
        assert_eq!(
            command_required_permission("playsound"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "playsound minecraft:block.note_block.harp"
            ),
            Err(CommandError::PermissionDenied)
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:block.note_block.harp",
        )
        .unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(result.feedback_key, "commands.playsound.success.single");
        assert_eq!(
            state.sound_events[0],
            SoundCommandEvent::Play(PlaySoundRequest {
                sound: "minecraft:block.note_block.harp".to_string(),
                source: SoundSource::Master,
                targets: vec![NameAndId::create_offline("Steve")],
                position: Vec3::default(),
                volume: 1.0,
                pitch: 1.0,
                min_volume: 0.0,
            })
        );
    }

    #[test]
    fn playsound_accepts_source_targets_position_volume_pitch_and_min_volume() {
        let mut state = ServerCommandState::default();
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:entity.arrow.hit player Steve,Alex 1.5 2.0 3.5 4.0 0.75 0.25",
        )
        .unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(result.feedback_key, "commands.playsound.success.multiple");
        assert_eq!(
            state.sound_events[0],
            SoundCommandEvent::Play(PlaySoundRequest {
                sound: "minecraft:entity.arrow.hit".to_string(),
                source: SoundSource::Player,
                targets: vec![
                    NameAndId::create_offline("Steve"),
                    NameAndId::create_offline("Alex")
                ],
                position: Vec3 {
                    x: 1.5,
                    y: 2.0,
                    z: 3.5,
                },
                volume: 4.0,
                pitch: 0.75,
                min_volume: 0.25,
            })
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "playsound minecraft:bad player Steve 0 0 0 1 2.5"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn playsound_fails_when_no_target_receives_sound() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "playsound minecraft:empty"
            ),
            Err(CommandError::PlaySoundTooFar)
        );
    }

    #[test]
    fn stopsound_queues_source_and_sound_filters() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("stopsound"),
            PermissionLevel::Gamemasters
        );
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopsound Steve,Alex player minecraft:entity.arrow.hit",
        )
        .unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(
            result.feedback_key,
            "commands.stopsound.success.source.sound"
        );
        assert_eq!(
            state.sound_events[0],
            SoundCommandEvent::Stop(StopSoundRequest {
                targets: vec![
                    NameAndId::create_offline("Steve"),
                    NameAndId::create_offline("Alex")
                ],
                source: Some(SoundSource::Player),
                sound: Some("minecraft:entity.arrow.hit".to_string()),
            })
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopsound Steve * minecraft:music.menu",
        )
        .unwrap();
        assert_eq!(
            result.feedback_key,
            "commands.stopsound.success.sourceless.sound"
        );
    }

    #[test]
    fn particle_command_requires_gamemaster_and_defaults_to_all_online_players() {
        let mut state = ServerCommandState {
            online_players: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex"),
            ],
            ..ServerCommandState::default()
        };
        assert_eq!(
            command_required_permission("particle"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "particle flame"
            ),
            Err(CommandError::PermissionDenied)
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle flame",
        )
        .unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(result.feedback_key, "commands.particle.success");
        assert_eq!(
            state.particle_events[0],
            ParticleCommandEvent {
                name: "flame".to_string(),
                viewers: vec![
                    NameAndId::create_offline("Steve"),
                    NameAndId::create_offline("Alex")
                ],
                position: Vec3::default(),
                delta: Vec3::default(),
                speed: 0.0,
                count: 0,
                force: false,
            }
        );
    }

    #[test]
    fn particle_command_parses_position_delta_speed_count_mode_and_viewers() {
        let mut state = ServerCommandState::default();
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle minecraft:dust 1 2 3 0.1 0.2 0.3 0.4 12 force Steve,Alex",
        )
        .unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(
            state.particle_events[0],
            ParticleCommandEvent {
                name: "minecraft:dust".to_string(),
                viewers: vec![
                    NameAndId::create_offline("Steve"),
                    NameAndId::create_offline("Alex")
                ],
                position: Vec3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                delta: Vec3 {
                    x: 0.1,
                    y: 0.2,
                    z: 0.3,
                },
                speed: 0.4,
                count: 12,
                force: true,
            }
        );
    }

    #[test]
    fn particle_command_fails_when_no_players_receive_particles() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "particle flame"
            ),
            Err(CommandError::ParticleFailed)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "particle flame 0 0 0 0 0 0 -1 1"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn jfr_command_requires_owner_and_records_start_stop_path() {
        let mut state = ServerCommandState {
            next_jfr_recording_path: "debug/test-recording.jfr".to_string(),
            ..ServerCommandState::default()
        };
        assert_eq!(command_required_permission("jfr"), PermissionLevel::Owners);
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "jfr start"),
            Err(CommandError::PermissionDenied)
        );

        let started =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr start")
                .unwrap();
        assert!(state.jfr_recording);
        assert_eq!(started.success_count, 1);
        assert_eq!(started.feedback_key, "commands.jfr.started");
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr start"),
            Err(CommandError::JfrStartFailed)
        );

        let stopped =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr stop")
                .unwrap();
        assert!(!state.jfr_recording);
        assert_eq!(stopped.success_count, 1);
        assert_eq!(stopped.feedback_key, "commands.jfr.stopped");
        assert_eq!(state.jfr_recordings, vec!["debug/test-recording.jfr"]);
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr stop"),
            Err(CommandError::JfrDumpFailed)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr"),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn perf_command_requires_owner_and_toggles_metrics_recording() {
        let mut state = ServerCommandState {
            tick_time_samples_nanos: vec![50_000_000, 60_000_000],
            ..ServerCommandState::default()
        };
        assert_eq!(command_required_permission("perf"), PermissionLevel::Owners);
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "perf start"),
            Err(CommandError::PermissionDenied)
        );

        let started =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf start")
                .unwrap();
        assert!(state.perf_recording);
        assert_eq!(started.success_count, 0);
        assert_eq!(started.feedback_key, "commands.perf.started");
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf start"),
            Err(CommandError::PerfAlreadyRunning)
        );

        let stopped =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf stop")
                .unwrap();
        assert!(!state.perf_recording);
        assert_eq!(stopped.success_count, 0);
        assert_eq!(stopped.feedback_key, "commands.perf.stopped");
        assert_eq!(
            state.perf_reports,
            vec![PerfReport {
                ticks: 2,
                duration_nanos: 110_000_000,
            }]
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf stop"),
            Err(CommandError::PerfNotRunning)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf"),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn rotate_command_requires_gamemaster_and_records_absolute_or_relative_angles() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("rotate"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "rotate pig 90 15"
            ),
            Err(CommandError::PermissionDenied)
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "rotate pig 90 ~-15",
        )
        .unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(result.feedback_key, "commands.rotate.success");
        assert_eq!(
            state.rotation_requests[0],
            RotationRequest {
                target: EntityRef {
                    id: "pig".to_string(),
                    display_name: "pig".to_string(),
                },
                mode: RotationMode::Angles {
                    yaw: 90.0,
                    pitch: -15.0,
                    yaw_relative: false,
                    pitch_relative: true,
                },
            }
        );
    }

    #[test]
    fn rotate_command_records_facing_entity_or_position() {
        let mut state = ServerCommandState::default();
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "rotate pig facing entity cow eyes",
        )
        .unwrap();
        assert_eq!(
            state.rotation_requests[0],
            RotationRequest {
                target: EntityRef {
                    id: "pig".to_string(),
                    display_name: "pig".to_string(),
                },
                mode: RotationMode::FacingEntity {
                    entity: EntityRef {
                        id: "cow".to_string(),
                        display_name: "cow".to_string(),
                    },
                    anchor: EntityAnchor::Eyes,
                },
            }
        );

        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "rotate pig facing 1.5 64 -2",
        )
        .unwrap();
        assert_eq!(
            state.rotation_requests[1].mode,
            RotationMode::FacingPosition(Vec3 {
                x: 1.5,
                y: 64.0,
                z: -2.0,
            })
        );

        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "rotate pig facing entity cow head"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn return_command_requires_gamemaster_and_records_success_or_failure() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("return"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "return 7"),
            Err(CommandError::PermissionDenied)
        );

        let result =
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "return 7")
                .unwrap();
        assert_eq!(result.success_count, 7);
        assert_eq!(result.feedback_key, "commands.return.success");
        assert_eq!(
            state.return_events[0],
            ReturnCommandEvent::Success {
                value: 7,
                discard_frame: true,
            }
        );

        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "return fail",
        )
        .unwrap();
        assert_eq!(result.success_count, 0);
        assert_eq!(result.feedback_key, "commands.return.fail");
        assert_eq!(
            state.return_events[1],
            ReturnCommandEvent::Failure {
                discard_frame: true,
            }
        );
    }

    #[test]
    fn return_command_records_forwarded_command_and_rejects_invalid_syntax() {
        let mut state = ServerCommandState::default();
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "return run say hello from function",
        )
        .unwrap();
        assert_eq!(result.success_count, 0);
        assert_eq!(result.feedback_key, "commands.return.run");
        assert_eq!(
            state.return_events,
            vec![ReturnCommandEvent::Run {
                command: "say hello from function".to_string(),
                discard_frame: true,
            }]
        );

        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "return"),
            Err(CommandError::InvalidSyntax)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "return run"
            ),
            Err(CommandError::InvalidSyntax)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "return nope"
            ),
            Err(CommandError::InvalidSyntax)
        );
    }

    #[test]
    fn ride_command_requires_gamemaster_and_mounts_then_dismounts() {
        let mut state = ServerCommandState::default();
        assert_eq!(
            command_required_permission("ride"),
            PermissionLevel::Gamemasters
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::MODERATOR,
                "ride pig mount boat"
            ),
            Err(CommandError::PermissionDenied)
        );

        let mounted = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig mount boat",
        )
        .unwrap();
        assert_eq!(mounted.success_count, 1);
        assert_eq!(mounted.feedback_key, "commands.ride.mount.success");
        assert_eq!(
            state.entity_mounts,
            vec![EntityMount {
                target: EntityRef {
                    id: "pig".to_string(),
                    display_name: "pig".to_string(),
                },
                vehicle: EntityRef {
                    id: "boat".to_string(),
                    display_name: "boat".to_string(),
                },
            }]
        );
        assert_eq!(
            state.ride_events[0],
            RideCommandEvent::Mount {
                target: EntityRef {
                    id: "pig".to_string(),
                    display_name: "pig".to_string(),
                },
                vehicle: EntityRef {
                    id: "boat".to_string(),
                    display_name: "boat".to_string(),
                },
            }
        );

        let dismounted = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig dismount",
        )
        .unwrap();
        assert_eq!(dismounted.success_count, 1);
        assert_eq!(dismounted.feedback_key, "commands.ride.dismount.success");
        assert!(state.entity_mounts.is_empty());
        assert!(matches!(
            state.ride_events[1],
            RideCommandEvent::Dismount { .. }
        ));
    }

    #[test]
    fn ride_command_rejects_vanilla_mount_failures() {
        let mut state = ServerCommandState {
            online_players: vec![NameAndId::create_offline("Steve")],
            entity_states: vec![
                EntityState {
                    entity: EntityRef {
                        id: "pig".to_string(),
                        display_name: "pig".to_string(),
                    },
                    kind: EntityKind::Generic,
                    dimension: "minecraft:overworld".to_string(),
                },
                EntityState {
                    entity: EntityRef {
                        id: "strider".to_string(),
                        display_name: "strider".to_string(),
                    },
                    kind: EntityKind::Generic,
                    dimension: "minecraft:the_nether".to_string(),
                },
            ],
            ..ServerCommandState::default()
        };

        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "ride pig dismount"
            ),
            Err(CommandError::RideNotRiding)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "ride pig mount Steve"
            ),
            Err(CommandError::RideMountingPlayer)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "ride pig mount pig"
            ),
            Err(CommandError::RideMountingLoop)
        );
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "ride pig mount strider"
            ),
            Err(CommandError::RideWrongDimension)
        );

        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig mount boat",
        )
        .unwrap();
        assert_eq!(
            execute_builtin_command(
                &mut state,
                LevelBasedPermissionSet::GAMEMASTER,
                "ride pig mount minecart"
            ),
            Err(CommandError::RideAlreadyRiding)
        );
        assert_eq!(
            execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "ride pig"),
            Err(CommandError::InvalidSyntax)
        );
    }
}
