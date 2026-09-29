use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct RespawnData {
    pub dimension: String,
    pub position: BlockPos,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSpawn {
    pub player: NameAndId,
    pub respawn: RespawnData,
    pub forced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateEntry {
    pub dimension: String,
    pub position: BlockPos,
    pub block: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeEntry {
    pub dimension: String,
    pub position: BlockPos,
    pub biome: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetBlockEvent {
    pub dimension: String,
    pub position: BlockPos,
    pub block: String,
    pub mode: SetBlockMode,
    pub strict: bool,
    pub destroyed_block: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloneEvent {
    pub source_dimension: String,
    pub target_dimension: String,
    pub begin: BlockPos,
    pub end: BlockPos,
    pub destination: BlockPos,
    pub filter: CloneFilter,
    pub mode: CloneMode,
    pub strict: bool,
    pub default_update_flags: i32,
    pub move_barrier_update_flags: Option<i32>,
    pub move_air_update_flags: Option<i32>,
    pub neighbour_updates: bool,
    pub block_ticks_copied: bool,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneFilter {
    Replace,
    Masked,
    Filtered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneMode {
    Normal,
    Force,
    Move,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillEvent {
    pub dimension: String,
    pub begin: BlockPos,
    pub end: BlockPos,
    pub block: String,
    pub mode: FillMode,
    pub filter: Option<String>,
    pub strict: bool,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillMode {
    Replace,
    Outline,
    Hollow,
    Destroy,
    Keep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillBiomeEvent {
    pub dimension: String,
    pub begin: BlockPos,
    pub end: BlockPos,
    pub biome: String,
    pub filter: Option<String>,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForcedChunk {
    pub dimension: String,
    pub chunk: ChunkPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLocatableEntry {
    pub kind: LocateKind,
    pub id: String,
    pub tags: Vec<String>,
    pub position: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLocateResult {
    pub kind: LocateKind,
    pub query: String,
    pub found_id: String,
    pub position: BlockPos,
    pub distance: i32,
    pub include_y: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocateKind {
    Structure,
    Biome,
    Poi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetBlockMode {
    Replace,
    Destroy,
    Keep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMembership {
    pub player: NameAndId,
    pub team: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamState {
    pub name: String,
    pub display_name: String,
    pub color: String,
    pub friendly_fire: bool,
    pub see_friendly_invisibles: bool,
    pub nametag_visibility: String,
    pub death_message_visibility: String,
    pub collision_rule: String,
    pub prefix: String,
    pub suffix: String,
}

impl TeamState {
    pub fn new(name: String, display_name: String) -> Self {
        Self {
            name,
            display_name,
            color: "reset".to_string(),
            friendly_fire: true,
            see_friendly_invisibles: true,
            nametag_visibility: "always".to_string(),
            death_message_visibility: "always".to_string(),
            collision_rule: "always".to_string(),
            prefix: String::new(),
            suffix: String::new(),
        }
    }

    pub fn packed_options(&self) -> u8 {
        u8::from(self.friendly_fire) | (u8::from(self.see_friendly_invisibles) << 1)
    }

    pub fn apply_packed_options(&mut self, options: u8) {
        self.friendly_fire = options & 1 != 0;
        self.see_friendly_invisibles = options & 2 != 0;
    }

    pub fn formatted_member_name(&self, member_name: &str) -> String {
        let name = format!("{}{}{}", self.prefix, member_name, self.suffix);
        if self.color == "reset" {
            name
        } else {
            format!("{}:{name}", self.color)
        }
    }

    pub fn formatted_display_name(&self) -> String {
        if self.color == "reset" {
            format!("[{}]", self.display_name)
        } else {
            format!("{}:[{}]", self.color, self.display_name)
        }
    }
}

pub fn player_team<'a>(
    teams: &'a [TeamState],
    memberships: &[TeamMembership],
    player: &NameAndId,
) -> Option<&'a TeamState> {
    let team_name = memberships
        .iter()
        .find(|membership| membership.player.uuid == player.uuid)
        .map(|membership| membership.team.as_str())?;
    teams.iter().find(|team| team.name == team_name)
}

pub fn players_allied(
    memberships: &[TeamMembership],
    first: &NameAndId,
    second: &NameAndId,
) -> bool {
    let first_team = memberships
        .iter()
        .find(|membership| membership.player.uuid == first.uuid)
        .map(|membership| membership.team.as_str());
    first_team.is_some_and(|team| {
        memberships
            .iter()
            .any(|membership| membership.player.uuid == second.uuid && membership.team == team)
    })
}

pub fn team_allows_friendly_damage(
    teams: &[TeamState],
    memberships: &[TeamMembership],
    attacker: &NameAndId,
    victim: &NameAndId,
) -> bool {
    if !players_allied(memberships, attacker, victim) {
        return true;
    }
    player_team(teams, memberships, attacker).is_none_or(|team| team.friendly_fire)
}

pub fn team_allows_collision(
    teams: &[TeamState],
    memberships: &[TeamMembership],
    first: &NameAndId,
    second: &NameAndId,
) -> bool {
    let allied = players_allied(memberships, first, second);
    let Some(team) = player_team(teams, memberships, first) else {
        return true;
    };
    match team.collision_rule.as_str() {
        "never" => false,
        "pushOwnTeam" => allied,
        "pushOtherTeams" => !allied,
        _ => true,
    }
}

pub fn team_allows_visibility(
    teams: &[TeamState],
    memberships: &[TeamMembership],
    viewer: &NameAndId,
    subject: &NameAndId,
    death_message: bool,
) -> bool {
    let allied = players_allied(memberships, viewer, subject);
    let Some(team) = player_team(teams, memberships, subject) else {
        return true;
    };
    let rule = if death_message {
        &team.death_message_visibility
    } else {
        &team.nametag_visibility
    };
    match rule.as_str() {
        "never" => false,
        "hideForOtherTeams" => allied,
        "hideForOwnTeam" => !allied,
        _ => true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardObjective {
    pub name: String,
    pub criteria: String,
    pub display_name: String,
    pub render_type: String,
    pub display_auto_update: bool,
    pub number_format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardScore {
    pub owner: String,
    pub objective: String,
    pub value: i32,
    pub locked: bool,
    pub display_name: Option<String>,
    pub number_format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardDisplaySlot {
    pub slot: String,
    pub objective: String,
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
    Emote,
    Private,
    Team,
    TellRaw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleCommandEvent {
    pub targets: Vec<NameAndId>,
    pub action: TitleCommandAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TitleCommandAction {
    Clear {
        reset: bool,
    },
    Text {
        kind: TitleTextKind,
        component: String,
    },
    Times {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleTextKind {
    Title,
    Subtitle,
    ActionBar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SoundCommandEvent {
    Play(PlaySoundRequest),
    Stop(StopSoundRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WardenSpawnTrackerState {
    pub player: NameAndId,
    pub warning_level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointState {
    pub entity: EntityRef,
    pub dimension: String,
    pub color: Option<i32>,
    pub style: String,
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
    pub deliveries: Vec<PlaySoundDelivery>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaySoundDelivery {
    pub target: NameAndId,
    pub position: Vec3,
    pub volume: f32,
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
pub struct SummonedEntity {
    pub entity_type: String,
    pub entity: EntityRef,
    pub position: Vec3,
    pub nbt: Option<String>,
    pub finalized_spawn: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArmorTrimSpawn {
    pub pattern: String,
    pub material: String,
    pub item: String,
    pub position: Vec3,
    pub y_rot: f32,
    pub no_gravity: bool,
    pub named: bool,
    pub custom_name_visible: bool,
    pub invisible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwingCommandEvent {
    pub target: EntityRef,
    pub hand: InteractionHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    MainHand,
    OffHand,
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

#[derive(Debug, Clone, PartialEq)]
pub struct DamageCommandEvent {
    pub target: EntityRef,
    pub amount: f32,
    pub source: DamageCommandSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DamageCommandSource {
    Generic,
    Type {
        damage_type: String,
    },
    At {
        damage_type: String,
        location: Vec3,
    },
    By {
        damage_type: String,
        entity: EntityRef,
        cause: Option<EntityRef>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMount {
    pub target: EntityRef,
    pub vehicle: EntityRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityPosition {
    pub entity: EntityRef,
    pub dimension: String,
    pub position: Vec3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeleportSideEffect {
    pub target: EntityRef,
    pub clear_vertical_motion_and_set_on_ground: bool,
    pub stop_pathfinding_navigation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickCommandFeedbackEvent {
    pub feedback_key: &'static str,
    pub broadcast_to_admins: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityState {
    pub entity: EntityRef,
    pub kind: EntityKind,
    pub dimension: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTags {
    pub entity: EntityRef,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Generic,
    NonLiving,
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

impl Default for RespawnData {
    fn default() -> Self {
        Self {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 0, y: 0, z: 0 },
            yaw: 0.0,
            pitch: 0.0,
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

/// `Entity.getRotationVector()` of an entity: `x_rot` is the pitch and `y_rot` the yaw.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityRotation {
    pub entity: EntityRef,
    pub x_rot: f32,
    pub y_rot: f32,
}

/// The single-target entity links `/execute on` follows.
///
/// Mounts (`vehicle`, `passengers`) come from [`EntityMount`]; every other relation of
/// `ExecuteCommand.createRelationOperations` is stored explicitly because the command model
/// has no `OwnableEntity`/`Leashable`/`Targeting`/`Attackable`/`TraceableEntity` state.
// TODO(execute-on-live-entities): populate from live entity ownership, leash, AI-target,
// last-attacker and projectile-owner state once the entity simulation exposes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityRelationKind {
    /// `OwnableEntity.getOwner()`.
    Owner,
    /// `Leashable.getLeashHolder()`.
    Leasher,
    /// `Targeting.getTarget()`.
    Target,
    /// `Attackable.getLastAttacker()`.
    Attacker,
    /// `Entity.getControllingPassenger()`.
    Controller,
    /// `TraceableEntity.getOwner()`.
    Origin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRelation {
    pub entity: EntityRef,
    pub kind: EntityRelationKind,
    pub target: EntityRef,
}

/// A chunk known to the command model (`ServerChunkCache.getChunkNow`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedChunk {
    pub dimension: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
    /// `chunk.getFullStatus() == FullChunkStatus.ENTITY_TICKING` and its entities are loaded.
    pub entity_ticking: bool,
}

/// A datapack loot predicate (`LootItemCondition`) as its decoded SNBT/JSON definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandLootPredicate {
    pub id: String,
    pub definition: Tag,
}

/// Java `ExecutionContext` cost accounting shared by every command of one top-level chain.
///
/// `quota` is Java's `commandQuota` (starts at `max_command_sequence_length`, at least 1) and
/// `fork_limit` is `max_command_forks`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandExecutionBudget {
    pub active: bool,
    pub quota: i32,
    pub fork_limit: i32,
}
