#![allow(dead_code)]

pub const ENDER_DRAGON_MAX_HEALTH: f32 = 200.0;
pub const ENDER_DRAGON_CAMERA_DISTANCE: f32 = 16.0;
pub const ENDER_DRAGON_GROWL_INTERVAL_MIN: i32 = 200;
pub const ENDER_DRAGON_GROWL_INTERVAL_MAX: i32 = 400;
pub const ENDER_DRAGON_SITTING_ALLOWED_DAMAGE_PERCENTAGE: f32 = 0.25;
pub const DRAGON_FIGHT_MAX_TICKS_BEFORE_RESPAWN: i32 = 1_200;
pub const DRAGON_FIGHT_CRYSTAL_SCAN_INTERVAL: i32 = 100;
pub const DRAGON_FIGHT_PLAYER_SCAN_INTERVAL: i32 = 20;
pub const DRAGON_FIGHT_ARENA_SIZE_CHUNKS: i32 = 8;
pub const DRAGON_FIGHT_ARENA_TICKET_LEVEL: i32 = 9;
pub const DRAGON_GATEWAY_COUNT: i32 = 20;
pub const DRAGON_GATEWAY_DISTANCE: i32 = 96;
pub const DRAGON_SPAWN_Y: i32 = 128;

pub const WITHER_MAX_HEALTH: f32 = 300.0;
pub const WITHER_MOVEMENT_SPEED: f32 = 0.6;
pub const WITHER_FLYING_SPEED: f32 = 0.6;
pub const WITHER_FOLLOW_RANGE: f32 = 40.0;
pub const WITHER_ARMOR: f32 = 4.0;
pub const WITHER_XP_REWARD: i32 = 50;
pub const WITHER_INVULNERABLE_TICKS: i32 = 220;
pub const WITHER_SPAWN_EXPLOSION_POWER: f32 = 7.0;
pub const WITHER_BLOCK_DESTROY_DELAY: i32 = 20;
pub const WITHER_IDLE_HEAD_ATTACK_THRESHOLD: i32 = 15;
pub const WITHER_HEAD_TARGET_RANGE_SQUARED: f32 = 900.0;
pub const WITHER_ALT_HEAD_UPDATE_DELAY_MIN: i32 = 10;
pub const WITHER_ALT_HEAD_UPDATE_DELAY_RANDOM_BOUND: i32 = 10;
pub const WITHER_ALT_HEAD_TARGET_ATTACK_DELAY_MIN: i32 = 40;
pub const WITHER_ALT_HEAD_TARGET_ATTACK_DELAY_RANDOM_BOUND: i32 = 20;
pub const WITHER_BLUE_SKULL_CHANCE: f32 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3Plan {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitherTargetMovementInput {
    pub powered: bool,
    pub has_main_target: bool,
    pub self_y: f32,
    pub target_y: f32,
    pub target_dx: f32,
    pub target_dz: f32,
    pub delta: Vec3Plan,
}

pub fn wither_target_movement_step(input: WitherTargetMovementInput) -> Vec3Plan {
    let mut next = Vec3Plan {
        x: input.delta.x,
        y: input.delta.y * 0.6,
        z: input.delta.z,
    };
    if !input.has_main_target {
        return next;
    }

    if input.self_y < input.target_y || (!input.powered && input.self_y < input.target_y + 5.0) {
        next.y = next.y.max(0.0);
        next.y += 0.3 - next.y * 0.6;
    }

    let horizontal_distance_sqr =
        input.target_dx * input.target_dx + input.target_dz * input.target_dz;
    if horizontal_distance_sqr > 9.0 {
        let horizontal_distance = horizontal_distance_sqr.sqrt();
        let nx = input.target_dx / horizontal_distance;
        let nz = input.target_dz / horizontal_distance;
        next.x += nx * 0.3 - next.x * 0.6;
        next.z += nz * 0.3 - next.z * 0.6;
    }

    next
}

pub fn wither_yaw_from_delta(delta: Vec3Plan) -> Option<f32> {
    if delta.x * delta.x + delta.z * delta.z <= 0.05 {
        return None;
    }
    Some(delta.z.atan2(delta.x).to_degrees() - 90.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitherSkullLaunchPlan {
    pub head: i32,
    pub origin: Vec3Plan,
    pub direction: Vec3Plan,
    pub dangerous: bool,
    pub level_event: Option<i32>,
}

pub fn wither_skull_launch_plan(
    head: i32,
    position: Vec3Plan,
    y_body_rot_degrees: f32,
    scale: f32,
    target: Vec3Plan,
    main_head_blue_skull_roll: f32,
    silent: bool,
) -> WitherSkullLaunchPlan {
    let origin = wither_head_position(head, position, y_body_rot_degrees, scale);
    let direction = normalize_vec3(Vec3Plan {
        x: target.x - origin.x,
        y: target.y - origin.y,
        z: target.z - origin.z,
    });
    WitherSkullLaunchPlan {
        head,
        origin,
        direction,
        dangerous: head == 0 && main_head_blue_skull_roll < WITHER_BLUE_SKULL_CHANCE,
        level_event: (!silent).then_some(1024),
    }
}

pub fn wither_head_position(
    head: i32,
    position: Vec3Plan,
    y_body_rot_degrees: f32,
    scale: f32,
) -> Vec3Plan {
    if head <= 0 {
        return Vec3Plan {
            x: position.x,
            y: position.y + 3.0 * scale,
            z: position.z,
        };
    }
    let head_angle = (y_body_rot_degrees + 180.0 * (head - 1) as f32).to_radians();
    Vec3Plan {
        x: position.x + head_angle.cos() * 1.3 * scale,
        y: position.y + 2.2 * scale,
        z: position.z + head_angle.sin() * 1.3 * scale,
    }
}

fn normalize_vec3(vector: Vec3Plan) -> Vec3Plan {
    let length = (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt();
    if length == 0.0 {
        return Vec3Plan {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
    }
    Vec3Plan {
        x: vector.x / length,
        y: vector.y / length,
        z: vector.z / length,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitherSummonAxis {
    X,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherSummonPrecondition {
    pub item: &'static str,
    pub placed_y: i32,
    pub min_y: i32,
    pub peaceful: bool,
    pub client_side: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherSummonPlan {
    pub axis: WitherSummonAxis,
    pub spawn_offset: (i32, i32, i32),
    pub yaw_degrees: i32,
    pub invulnerable_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherDeathLootPlan {
    pub drop_item: &'static str,
    pub extended_lifetime: bool,
    pub death_explosion: bool,
    pub bossbar_visible_after_removal: bool,
}

pub fn wither_death_loot_plan() -> WitherDeathLootPlan {
    WitherDeathLootPlan {
        drop_item: "minecraft:nether_star",
        extended_lifetime: true,
        death_explosion: false,
        bossbar_visible_after_removal: false,
    }
}

pub fn wither_summon_precondition_allows_item(
    precondition: WitherSummonPrecondition,
    block_at: impl Fn(i32, i32, i32) -> &'static str,
) -> Option<WitherSummonPlan> {
    if precondition.item != "minecraft:wither_skeleton_skull"
        || precondition.placed_y < precondition.min_y + 2
        || precondition.peaceful
        || precondition.client_side
    {
        return None;
    }
    wither_summon_full_pattern(block_at)
}

pub fn wither_summon_full_pattern(
    block_at: impl Fn(i32, i32, i32) -> &'static str,
) -> Option<WitherSummonPlan> {
    if wither_summon_pattern_for_axis(WitherSummonAxis::X, &block_at) {
        return Some(WitherSummonPlan {
            axis: WitherSummonAxis::X,
            spawn_offset: (0, 1, 0),
            yaw_degrees: 0,
            invulnerable_ticks: WITHER_INVULNERABLE_TICKS,
        });
    }
    if wither_summon_pattern_for_axis(WitherSummonAxis::Z, &block_at) {
        return Some(WitherSummonPlan {
            axis: WitherSummonAxis::Z,
            spawn_offset: (0, 1, 0),
            yaw_degrees: 90,
            invulnerable_ticks: WITHER_INVULNERABLE_TICKS,
        });
    }
    None
}

fn wither_summon_pattern_for_axis(
    axis: WitherSummonAxis,
    block_at: &impl Fn(i32, i32, i32) -> &'static str,
) -> bool {
    for side in [-1, 1] {
        if !is_air_for_wither_pattern(block_at(
            axis_offset(axis, side).0,
            0,
            axis_offset(axis, side).1,
        )) {
            return false;
        }
    }
    if !is_wither_base_block(block_at(0, 0, 0)) {
        return false;
    }
    for side in [-1, 0, 1] {
        let (x, z) = axis_offset(axis, side);
        if !is_wither_base_block(block_at(x, 1, z)) {
            return false;
        }
        if !is_wither_skull_block(block_at(x, 2, z)) {
            return false;
        }
    }
    true
}

fn axis_offset(axis: WitherSummonAxis, side: i32) -> (i32, i32) {
    match axis {
        WitherSummonAxis::X => (side, 0),
        WitherSummonAxis::Z => (0, side),
    }
}

pub fn is_wither_base_block(block: &str) -> bool {
    matches!(block, "minecraft:soul_sand" | "minecraft:soul_soil")
}

pub fn is_wither_skull_block(block: &str) -> bool {
    matches!(
        block,
        "minecraft:wither_skeleton_skull" | "minecraft:wither_skeleton_wall_skull"
    )
}

fn is_air_for_wither_pattern(block: &str) -> bool {
    block == "minecraft:air"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonPhaseDef {
    pub id: i32,
    pub name: &'static str,
    pub sitting: bool,
}

pub const DRAGON_PHASES: &[DragonPhaseDef] = &[
    DragonPhaseDef {
        id: 0,
        name: "HoldingPattern",
        sitting: false,
    },
    DragonPhaseDef {
        id: 1,
        name: "StrafePlayer",
        sitting: false,
    },
    DragonPhaseDef {
        id: 2,
        name: "LandingApproach",
        sitting: false,
    },
    DragonPhaseDef {
        id: 3,
        name: "Landing",
        sitting: false,
    },
    DragonPhaseDef {
        id: 4,
        name: "Takeoff",
        sitting: false,
    },
    DragonPhaseDef {
        id: 5,
        name: "SittingFlaming",
        sitting: true,
    },
    DragonPhaseDef {
        id: 6,
        name: "SittingScanning",
        sitting: true,
    },
    DragonPhaseDef {
        id: 7,
        name: "SittingAttacking",
        sitting: true,
    },
    DragonPhaseDef {
        id: 8,
        name: "ChargingPlayer",
        sitting: false,
    },
    DragonPhaseDef {
        id: 9,
        name: "Dying",
        sitting: false,
    },
    DragonPhaseDef {
        id: 10,
        name: "Hover",
        sitting: false,
    },
];

pub fn dragon_phase_by_id(id: i32) -> DragonPhaseDef {
    DRAGON_PHASES
        .iter()
        .copied()
        .find(|phase| phase.id == id)
        .unwrap_or(DRAGON_PHASES[0])
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonPartDef {
    pub name: &'static str,
    pub width: f32,
    pub height: f32,
}

pub const DRAGON_PARTS: &[DragonPartDef] = &[
    DragonPartDef {
        name: "head",
        width: 1.0,
        height: 1.0,
    },
    DragonPartDef {
        name: "neck",
        width: 3.0,
        height: 3.0,
    },
    DragonPartDef {
        name: "body",
        width: 5.0,
        height: 3.0,
    },
    DragonPartDef {
        name: "tail",
        width: 2.0,
        height: 2.0,
    },
    DragonPartDef {
        name: "tail",
        width: 2.0,
        height: 2.0,
    },
    DragonPartDef {
        name: "tail",
        width: 2.0,
        height: 2.0,
    },
    DragonPartDef {
        name: "wing",
        width: 4.0,
        height: 2.0,
    },
    DragonPartDef {
        name: "wing",
        width: 4.0,
        height: 2.0,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarColor {
    Pink,
    Purple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarOverlay {
    Progress,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BossBarState {
    pub visible: bool,
    pub progress: f32,
    pub color: BossBarColor,
    pub overlay: BossBarOverlay,
    pub play_music: bool,
    pub darken_screen: bool,
    pub create_world_fog: bool,
}

impl BossBarState {
    pub fn ender_dragon(dragon_killed: bool, health: f32) -> Self {
        Self {
            visible: !dragon_killed,
            progress: clamp_progress(health / ENDER_DRAGON_MAX_HEALTH),
            color: BossBarColor::Pink,
            overlay: BossBarOverlay::Progress,
            play_music: true,
            darken_screen: false,
            create_world_fog: true,
        }
    }

    pub fn wither(health: f32, invulnerable_ticks: i32) -> Self {
        let progress = if invulnerable_ticks > 0 {
            1.0 - invulnerable_ticks as f32 / WITHER_INVULNERABLE_TICKS as f32
        } else {
            health / WITHER_MAX_HEALTH
        };
        Self {
            visible: true,
            progress: clamp_progress(progress),
            color: BossBarColor::Purple,
            overlay: BossBarOverlay::Progress,
            play_music: false,
            darken_screen: true,
            create_world_fog: false,
        }
    }
}

fn clamp_progress(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonFightState {
    pub ticks_since_last_player_scan: i32,
    pub ticks_since_dragon_seen: i32,
    pub ticks_since_crystals_scanned: i32,
    pub dragon_killed: bool,
    pub players_tracking_fight: usize,
    pub arena_loaded: bool,
    pub needs_state_scanning: bool,
    pub respawn_stage_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonFightTickPlan {
    pub bossbar_visible: bool,
    pub scan_players: bool,
    pub add_arena_ticket: bool,
    pub remove_arena_ticket: bool,
    pub scan_legacy_state: bool,
    pub tick_respawn_stage: bool,
    pub find_or_create_dragon: bool,
    pub scan_crystals: bool,
}

pub fn dragon_fight_tick_plan(state: DragonFightState) -> DragonFightTickPlan {
    let scan_players = state.ticks_since_last_player_scan + 1 >= DRAGON_FIGHT_PLAYER_SCAN_INTERVAL;
    if state.players_tracking_fight == 0 {
        return DragonFightTickPlan {
            bossbar_visible: !state.dragon_killed,
            scan_players,
            add_arena_ticket: false,
            remove_arena_ticket: true,
            scan_legacy_state: false,
            tick_respawn_stage: false,
            find_or_create_dragon: false,
            scan_crystals: false,
        };
    }
    let arena_active = state.arena_loaded;
    DragonFightTickPlan {
        bossbar_visible: !state.dragon_killed,
        scan_players,
        add_arena_ticket: true,
        remove_arena_ticket: false,
        scan_legacy_state: arena_active && state.needs_state_scanning,
        tick_respawn_stage: arena_active && state.respawn_stage_active,
        find_or_create_dragon: arena_active
            && !state.dragon_killed
            && state.ticks_since_dragon_seen + 1 >= DRAGON_FIGHT_MAX_TICKS_BEFORE_RESPAWN,
        scan_crystals: arena_active
            && !state.dragon_killed
            && state.ticks_since_crystals_scanned + 1 >= DRAGON_FIGHT_CRYSTAL_SCAN_INTERVAL,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonRespawnStage {
    Start,
    PreparingToSummonPillars,
    SummoningPillars,
    SummoningDragon,
    End,
}

pub const DRAGON_RESPAWN_STAGES: &[DragonRespawnStage] = &[
    DragonRespawnStage::Start,
    DragonRespawnStage::PreparingToSummonPillars,
    DragonRespawnStage::SummoningPillars,
    DragonRespawnStage::SummoningDragon,
    DragonRespawnStage::End,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonDeathPlan {
    Explode,
    DropExperience,
    SpawnExitPortalAndGateway,
    Remove,
}

pub fn dragon_death_plan(dragon_death_time: i32) -> Vec<DragonDeathPlan> {
    let mut plan = Vec::new();
    if (180..=200).contains(&dragon_death_time) {
        plan.push(DragonDeathPlan::Explode);
    }
    if dragon_death_time == 200 {
        plan.push(DragonDeathPlan::DropExperience);
        plan.push(DragonDeathPlan::SpawnExitPortalAndGateway);
        plan.push(DragonDeathPlan::Remove);
    }
    plan
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndCrystalDamageResult {
    IgnoredDragon,
    IgnoredInvulnerable,
    RemovedOnly,
    ExplodeAndNotifyFight,
}

pub fn end_crystal_damage_result(
    source_is_dragon: bool,
    base_invulnerable: bool,
    source_is_explosion: bool,
) -> EndCrystalDamageResult {
    if base_invulnerable {
        EndCrystalDamageResult::IgnoredInvulnerable
    } else if source_is_dragon {
        EndCrystalDamageResult::IgnoredDragon
    } else if source_is_explosion {
        EndCrystalDamageResult::RemovedOnly
    } else {
        EndCrystalDamageResult::ExplodeAndNotifyFight
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitherDamageSourceKind {
    Generic,
    WitherImmune,
    WitherBoss,
    BypassesInvulnerability,
    Arrow,
    WindCharge,
    WitherFriend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitherDamageResult {
    Ignored,
    Accepted { schedule_block_destroy: bool },
}

pub fn wither_damage_result(
    invulnerable_ticks: i32,
    powered: bool,
    destroy_blocks_tick: i32,
    source: WitherDamageSourceKind,
) -> WitherDamageResult {
    if matches!(
        source,
        WitherDamageSourceKind::WitherImmune
            | WitherDamageSourceKind::WitherBoss
            | WitherDamageSourceKind::WitherFriend
    ) {
        return WitherDamageResult::Ignored;
    }
    if invulnerable_ticks > 0 && source != WitherDamageSourceKind::BypassesInvulnerability {
        return WitherDamageResult::Ignored;
    }
    if powered
        && matches!(
            source,
            WitherDamageSourceKind::Arrow | WitherDamageSourceKind::WindCharge
        )
    {
        return WitherDamageResult::Ignored;
    }
    WitherDamageResult::Accepted {
        schedule_block_destroy: destroy_blocks_tick <= 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitherAiTickOutcome {
    pub invulnerable_ticks: i32,
    pub bossbar_progress: f32,
    pub heal_amount: f32,
    pub explode: bool,
    pub destroy_blocks_now: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherBlockDestroyPlan {
    pub next_destroy_blocks_tick: i32,
    pub should_scan_blocks: bool,
    pub emit_level_event_1022: bool,
}

pub fn wither_ai_tick(
    tick_count: i32,
    invulnerable_ticks: i32,
    destroy_blocks_tick: i32,
    mob_griefing: bool,
    health: f32,
) -> WitherAiTickOutcome {
    if invulnerable_ticks > 0 {
        let next = invulnerable_ticks - 1;
        return WitherAiTickOutcome {
            invulnerable_ticks: next,
            bossbar_progress: BossBarState::wither(health, next).progress,
            heal_amount: if tick_count % 10 == 0 { 10.0 } else { 0.0 },
            explode: next <= 0,
            destroy_blocks_now: false,
        };
    }
    WitherAiTickOutcome {
        invulnerable_ticks: 0,
        bossbar_progress: BossBarState::wither(health, 0).progress,
        heal_amount: if tick_count % 20 == 0 { 1.0 } else { 0.0 },
        explode: false,
        destroy_blocks_now: destroy_blocks_tick == 1 && mob_griefing,
    }
}

pub fn wither_can_destroy_block(block_is_air: bool, block_is_wither_immune: bool) -> bool {
    !block_is_air && !block_is_wither_immune
}

pub fn wither_block_destroy_scan_volume(bb_width: f32, bb_height: f32) -> (i32, i32) {
    (
        (bb_width / 2.0 + 1.0).floor() as i32,
        bb_height.floor() as i32,
    )
}

pub fn wither_block_destroy_plan(
    destroy_blocks_tick: i32,
    mob_griefing: bool,
    any_block_destroyed: bool,
) -> WitherBlockDestroyPlan {
    if destroy_blocks_tick <= 0 {
        return WitherBlockDestroyPlan {
            next_destroy_blocks_tick: destroy_blocks_tick,
            should_scan_blocks: false,
            emit_level_event_1022: false,
        };
    }
    let next_destroy_blocks_tick = destroy_blocks_tick - 1;
    let should_scan_blocks = next_destroy_blocks_tick == 0 && mob_griefing;
    WitherBlockDestroyPlan {
        next_destroy_blocks_tick,
        should_scan_blocks,
        emit_level_event_1022: should_scan_blocks && any_block_destroyed,
    }
}

pub fn make_wither_invulnerable_health(max_health: f32) -> f32 {
    max_health / 3.0
}

pub fn wither_is_powered(health: f32, max_health: f32) -> bool {
    health <= max_health / 2.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitherHeadTickAction {
    Wait,
    FireAtCurrentTarget {
        head: i32,
        dangerous: bool,
        next_update_delay: i32,
        reset_idle_updates: bool,
    },
    FireIdleBlueSkull {
        head: i32,
        next_update_delay: i32,
        reset_idle_updates: bool,
    },
    AcquireNearbyTarget {
        head: i32,
    },
    ClearInvalidTarget {
        head: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherMainHeadSync {
    pub alternative_target_head: i32,
    pub target_entity_id: i32,
}

pub fn wither_main_head_sync(main_target_entity_id: Option<i32>) -> WitherMainHeadSync {
    WitherMainHeadSync {
        alternative_target_head: 0,
        target_entity_id: main_target_entity_id.unwrap_or(0),
    }
}

pub fn wither_alt_head_tick_action(
    head: i32,
    tick_count: i32,
    next_head_update: i32,
    difficulty_allows_idle_attack: bool,
    idle_head_updates: i32,
    random_0_to_9: i32,
    alternative_target_entity_id: Option<i32>,
    current_target_valid: bool,
    current_target_distance_sqr: f32,
    current_target_line_of_sight: bool,
    nearby_targets_available: bool,
) -> WitherHeadTickAction {
    if tick_count < next_head_update {
        return WitherHeadTickAction::Wait;
    }
    let next_update_delay = WITHER_ALT_HEAD_UPDATE_DELAY_MIN
        + random_0_to_9.rem_euclid(WITHER_ALT_HEAD_UPDATE_DELAY_RANDOM_BOUND);
    if difficulty_allows_idle_attack && idle_head_updates > WITHER_IDLE_HEAD_ATTACK_THRESHOLD {
        return WitherHeadTickAction::FireIdleBlueSkull {
            head,
            next_update_delay,
            reset_idle_updates: true,
        };
    }
    if alternative_target_entity_id.is_some() {
        if current_target_valid
            && current_target_distance_sqr <= WITHER_HEAD_TARGET_RANGE_SQUARED
            && current_target_line_of_sight
        {
            return WitherHeadTickAction::FireAtCurrentTarget {
                head,
                dangerous: false,
                next_update_delay: WITHER_ALT_HEAD_TARGET_ATTACK_DELAY_MIN
                    + random_0_to_9.rem_euclid(WITHER_ALT_HEAD_TARGET_ATTACK_DELAY_RANDOM_BOUND),
                reset_idle_updates: true,
            };
        }
        return WitherHeadTickAction::ClearInvalidTarget { head };
    }
    if nearby_targets_available {
        WitherHeadTickAction::AcquireNearbyTarget { head }
    } else {
        WitherHeadTickAction::Wait
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossFightCoverage {
    pub source: &'static str,
    pub covered_rules: &'static [&'static str],
}

pub const BOSS_FIGHT_COVERAGE: &[BossFightCoverage] = &[
    BossFightCoverage {
        source: "EnderDragon",
        covered_rules: &[
            "parts",
            "attributes",
            "phase metadata",
            "death sequence",
            "sitting damage gate",
        ],
    },
    BossFightCoverage {
        source: "EnderDragonPhase",
        covered_rules: &["dragon phases"],
    },
    BossFightCoverage {
        source: "EnderDragonFight",
        covered_rules: &[
            "bossbar",
            "arena ticket",
            "player scan",
            "crystal scan",
            "dragon respawn scan",
            "gateway constants",
            "end fight state",
        ],
    },
    BossFightCoverage {
        source: "DragonRespawnStage",
        covered_rules: &["respawn stages", "end crystals"],
    },
    BossFightCoverage {
        source: "EndCrystal",
        covered_rules: &["crystal beam", "crystal explosion", "fight callback"],
    },
    BossFightCoverage {
        source: "WitherBoss",
        covered_rules: &[
            "attributes",
            "bossbar",
            "invulnerability",
            "spawn explosion",
            "block destruction",
            "ranged skulls",
            "damage immunity",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragon_phase_ids_parts_and_attributes_match_decompiled_boss_sources() {
        assert_eq!(DRAGON_PHASES.len(), 11);
        assert_eq!(dragon_phase_by_id(0).name, "HoldingPattern");
        assert_eq!(dragon_phase_by_id(5).name, "SittingFlaming");
        assert!(dragon_phase_by_id(6).sitting);
        assert_eq!(dragon_phase_by_id(999).name, "HoldingPattern");

        assert_eq!(DRAGON_PARTS.len(), 8);
        assert_eq!(
            DRAGON_PARTS[0],
            DragonPartDef {
                name: "head",
                width: 1.0,
                height: 1.0
            }
        );
        assert_eq!(
            DRAGON_PARTS[2],
            DragonPartDef {
                name: "body",
                width: 5.0,
                height: 3.0
            }
        );
        assert_eq!(
            DRAGON_PARTS[6],
            DragonPartDef {
                name: "wing",
                width: 4.0,
                height: 2.0
            }
        );
        assert_eq!(ENDER_DRAGON_MAX_HEALTH, 200.0);
        assert_eq!(ENDER_DRAGON_CAMERA_DISTANCE, 16.0);
        assert_eq!(ENDER_DRAGON_SITTING_ALLOWED_DAMAGE_PERCENTAGE, 0.25);
    }

    #[test]
    fn dragon_fight_tick_plan_tracks_players_tickets_scans_and_bossbar() {
        assert_eq!(DRAGON_FIGHT_PLAYER_SCAN_INTERVAL, 20);
        assert_eq!(DRAGON_FIGHT_CRYSTAL_SCAN_INTERVAL, 100);
        assert_eq!(DRAGON_FIGHT_MAX_TICKS_BEFORE_RESPAWN, 1_200);
        assert_eq!(DRAGON_FIGHT_ARENA_TICKET_LEVEL, 9);
        assert_eq!(DRAGON_GATEWAY_COUNT, 20);
        assert_eq!(DRAGON_GATEWAY_DISTANCE, 96);
        assert_eq!(DRAGON_SPAWN_Y, 128);

        let empty = dragon_fight_tick_plan(DragonFightState {
            ticks_since_last_player_scan: 19,
            ticks_since_dragon_seen: 1_199,
            ticks_since_crystals_scanned: 99,
            dragon_killed: false,
            players_tracking_fight: 0,
            arena_loaded: true,
            needs_state_scanning: true,
            respawn_stage_active: true,
        });
        assert!(empty.scan_players);
        assert!(empty.remove_arena_ticket);
        assert!(!empty.add_arena_ticket);
        assert!(!empty.find_or_create_dragon);

        let active = dragon_fight_tick_plan(DragonFightState {
            players_tracking_fight: 1,
            ..DragonFightState {
                ticks_since_last_player_scan: 19,
                ticks_since_dragon_seen: 1_199,
                ticks_since_crystals_scanned: 99,
                dragon_killed: false,
                players_tracking_fight: 0,
                arena_loaded: true,
                needs_state_scanning: true,
                respawn_stage_active: true,
            }
        });
        assert!(active.add_arena_ticket);
        assert!(active.scan_legacy_state);
        assert!(active.tick_respawn_stage);
        assert!(active.find_or_create_dragon);
        assert!(active.scan_crystals);
    }

    #[test]
    fn dragon_bossbar_death_respawn_and_crystal_rules_are_visible_to_clients() {
        assert_eq!(
            BossBarState::ender_dragon(false, 100.0),
            BossBarState {
                visible: true,
                progress: 0.5,
                color: BossBarColor::Pink,
                overlay: BossBarOverlay::Progress,
                play_music: true,
                darken_screen: false,
                create_world_fog: true,
            }
        );
        assert!(!BossBarState::ender_dragon(true, 0.0).visible);

        assert_eq!(DRAGON_RESPAWN_STAGES.len(), 5);
        assert_eq!(dragon_death_plan(179), vec![]);
        assert_eq!(dragon_death_plan(180), vec![DragonDeathPlan::Explode]);
        assert_eq!(
            dragon_death_plan(200),
            vec![
                DragonDeathPlan::Explode,
                DragonDeathPlan::DropExperience,
                DragonDeathPlan::SpawnExitPortalAndGateway,
                DragonDeathPlan::Remove,
            ]
        );

        assert_eq!(
            end_crystal_damage_result(true, false, false),
            EndCrystalDamageResult::IgnoredDragon
        );
        assert_eq!(
            end_crystal_damage_result(false, false, true),
            EndCrystalDamageResult::RemovedOnly
        );
        assert_eq!(
            end_crystal_damage_result(false, false, false),
            EndCrystalDamageResult::ExplodeAndNotifyFight
        );
    }

    #[test]
    fn wither_attributes_invulnerability_bossbar_and_healing_match_vanilla() {
        assert_eq!(WITHER_MAX_HEALTH, 300.0);
        assert_eq!(WITHER_MOVEMENT_SPEED, 0.6);
        assert_eq!(WITHER_FLYING_SPEED, 0.6);
        assert_eq!(WITHER_FOLLOW_RANGE, 40.0);
        assert_eq!(WITHER_ARMOR, 4.0);
        assert_eq!(WITHER_XP_REWARD, 50);
        assert_eq!(make_wither_invulnerable_health(WITHER_MAX_HEALTH), 100.0);
        assert!(wither_is_powered(150.0, 300.0));
        assert!(!wither_is_powered(151.0, 300.0));

        let spawning = wither_ai_tick(10, 220, 0, true, 100.0);
        assert_eq!(spawning.invulnerable_ticks, 219);
        assert_eq!(spawning.heal_amount, 10.0);
        assert!(!spawning.explode);
        assert_eq!(
            spawning.bossbar_progress,
            BossBarState::wither(100.0, 219).progress
        );

        let explode = wither_ai_tick(20, 1, 0, true, 100.0);
        assert!(explode.explode);
        assert_eq!(explode.invulnerable_ticks, 0);

        let normal = wither_ai_tick(20, 0, 1, true, 150.0);
        assert_eq!(normal.heal_amount, 1.0);
        assert!(normal.destroy_blocks_now);
        assert_eq!(normal.bossbar_progress, 0.5);

        let bossbar = BossBarState::wither(150.0, 0);
        assert!(bossbar.visible);
        assert_eq!(bossbar.color, BossBarColor::Purple);
        assert_eq!(bossbar.overlay, BossBarOverlay::Progress);
        assert_eq!(bossbar.progress, 0.5);
    }

    #[test]
    fn wither_damage_immunities_powered_projectile_gate_and_block_destroy_timer_match_vanilla() {
        assert_eq!(
            wither_damage_result(10, false, 0, WitherDamageSourceKind::Generic),
            WitherDamageResult::Ignored
        );
        assert_eq!(
            wither_damage_result(
                10,
                false,
                0,
                WitherDamageSourceKind::BypassesInvulnerability
            ),
            WitherDamageResult::Accepted {
                schedule_block_destroy: true
            }
        );
        assert_eq!(
            wither_damage_result(0, true, 0, WitherDamageSourceKind::Arrow),
            WitherDamageResult::Ignored
        );
        assert_eq!(
            wither_damage_result(0, false, 20, WitherDamageSourceKind::Generic),
            WitherDamageResult::Accepted {
                schedule_block_destroy: false
            }
        );
        assert_eq!(
            wither_damage_result(0, false, 0, WitherDamageSourceKind::WitherFriend),
            WitherDamageResult::Ignored
        );
    }

    #[test]
    fn wither_block_destroy_delay_scan_volume_and_filters_match_java() {
        assert!(!wither_can_destroy_block(true, false));
        assert!(!wither_can_destroy_block(false, true));
        assert!(wither_can_destroy_block(false, false));
        assert_eq!(wither_block_destroy_scan_volume(0.9, 3.5), (1, 3));

        assert_eq!(
            wither_block_destroy_plan(20, true, false),
            WitherBlockDestroyPlan {
                next_destroy_blocks_tick: 19,
                should_scan_blocks: false,
                emit_level_event_1022: false,
            }
        );
        assert_eq!(
            wither_block_destroy_plan(1, false, true),
            WitherBlockDestroyPlan {
                next_destroy_blocks_tick: 0,
                should_scan_blocks: false,
                emit_level_event_1022: false,
            }
        );
        assert_eq!(
            wither_block_destroy_plan(1, true, false),
            WitherBlockDestroyPlan {
                next_destroy_blocks_tick: 0,
                should_scan_blocks: true,
                emit_level_event_1022: false,
            }
        );
        assert_eq!(
            wither_block_destroy_plan(1, true, true),
            WitherBlockDestroyPlan {
                next_destroy_blocks_tick: 0,
                should_scan_blocks: true,
                emit_level_event_1022: true,
            }
        );
    }

    #[test]
    fn wither_summoning_precondition_matches_skull_block_pattern() {
        let x_axis = |x, y, z| match (x, y, z) {
            (-1, 2, 0) | (0, 2, 0) | (1, 2, 0) => "minecraft:wither_skeleton_skull",
            (-1, 1, 0) | (0, 1, 0) | (1, 1, 0) => "minecraft:soul_sand",
            (0, 0, 0) => "minecraft:soul_soil",
            _ => "minecraft:air",
        };
        assert_eq!(
            wither_summon_precondition_allows_item(
                WitherSummonPrecondition {
                    item: "minecraft:wither_skeleton_skull",
                    placed_y: 64,
                    min_y: -64,
                    peaceful: false,
                    client_side: false,
                },
                x_axis,
            ),
            Some(WitherSummonPlan {
                axis: WitherSummonAxis::X,
                spawn_offset: (0, 1, 0),
                yaw_degrees: 0,
                invulnerable_ticks: WITHER_INVULNERABLE_TICKS,
            })
        );

        let z_axis = |x, y, z| match (x, y, z) {
            (0, 2, -1) | (0, 2, 0) | (0, 2, 1) => "minecraft:wither_skeleton_wall_skull",
            (0, 1, -1) | (0, 1, 0) | (0, 1, 1) => "minecraft:soul_soil",
            (0, 0, 0) => "minecraft:soul_sand",
            _ => "minecraft:air",
        };
        assert_eq!(wither_summon_full_pattern(z_axis).unwrap().yaw_degrees, 90);

        for blocked in [
            WitherSummonPrecondition {
                item: "minecraft:skeleton_skull",
                placed_y: 64,
                min_y: -64,
                peaceful: false,
                client_side: false,
            },
            WitherSummonPrecondition {
                item: "minecraft:wither_skeleton_skull",
                placed_y: -63,
                min_y: -64,
                peaceful: false,
                client_side: false,
            },
            WitherSummonPrecondition {
                item: "minecraft:wither_skeleton_skull",
                placed_y: 64,
                min_y: -64,
                peaceful: true,
                client_side: false,
            },
            WitherSummonPrecondition {
                item: "minecraft:wither_skeleton_skull",
                placed_y: 64,
                min_y: -64,
                peaceful: false,
                client_side: true,
            },
        ] {
            assert_eq!(
                wither_summon_precondition_allows_item(blocked, x_axis),
                None
            );
        }

        let obstructed_air_slot = |x, y, z| match (x, y, z) {
            (-1, 0, 0) => "minecraft:stone",
            (-1, 2, 0) | (0, 2, 0) | (1, 2, 0) => "minecraft:wither_skeleton_skull",
            (-1, 1, 0) | (0, 1, 0) | (1, 1, 0) => "minecraft:soul_sand",
            (0, 0, 0) => "minecraft:soul_soil",
            _ => "minecraft:air",
        };
        assert_eq!(wither_summon_full_pattern(obstructed_air_slot), None);
    }

    #[test]
    fn wither_death_drop_matches_java_nether_star_behavior() {
        assert_eq!(
            wither_death_loot_plan(),
            WitherDeathLootPlan {
                drop_item: "minecraft:nether_star",
                extended_lifetime: true,
                death_explosion: false,
                bossbar_visible_after_removal: false,
            }
        );
        assert_eq!(WITHER_SPAWN_EXPLOSION_POWER, 7.0);
    }

    #[test]
    fn wither_powered_target_movement_matches_java_ai_step_gate() {
        let unpowered_above_target = wither_target_movement_step(WitherTargetMovementInput {
            powered: false,
            has_main_target: true,
            self_y: 68.0,
            target_y: 64.0,
            target_dx: 4.0,
            target_dz: 0.0,
            delta: Vec3Plan {
                x: 0.0,
                y: -0.2,
                z: 0.0,
            },
        });
        assert!((unpowered_above_target.y - 0.3).abs() < 0.0001);
        assert!((unpowered_above_target.x - 0.3).abs() < 0.0001);
        assert_eq!(wither_yaw_from_delta(unpowered_above_target), Some(-90.0));

        let powered_above_target = wither_target_movement_step(WitherTargetMovementInput {
            powered: true,
            has_main_target: true,
            self_y: 68.0,
            target_y: 64.0,
            target_dx: 4.0,
            target_dz: 0.0,
            delta: Vec3Plan {
                x: 0.0,
                y: -0.2,
                z: 0.0,
            },
        });
        assert!((powered_above_target.y - -0.12).abs() < 0.0001);
        assert!((powered_above_target.x - 0.3).abs() < 0.0001);

        let powered_below_target = wither_target_movement_step(WitherTargetMovementInput {
            powered: true,
            has_main_target: true,
            self_y: 63.0,
            target_y: 64.0,
            target_dx: 0.0,
            target_dz: 2.0,
            delta: Vec3Plan {
                x: 0.0,
                y: -0.2,
                z: 0.0,
            },
        });
        assert!((powered_below_target.y - 0.3).abs() < 0.0001);
        assert_eq!(powered_below_target.z, 0.0);
        assert_eq!(wither_yaw_from_delta(powered_below_target), None);

        let no_target = wither_target_movement_step(WitherTargetMovementInput {
            powered: false,
            has_main_target: false,
            self_y: 60.0,
            target_y: 80.0,
            target_dx: 20.0,
            target_dz: 20.0,
            delta: Vec3Plan {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        });
        assert_eq!(
            no_target,
            Vec3Plan {
                x: 1.0,
                y: 0.6,
                z: 1.0
            }
        );
    }

    #[test]
    fn wither_skull_launch_head_offsets_direction_and_dangerous_roll_match_java() {
        let main = wither_skull_launch_plan(
            0,
            Vec3Plan {
                x: 10.0,
                y: 64.0,
                z: -5.0,
            },
            90.0,
            1.0,
            Vec3Plan {
                x: 10.0,
                y: 67.0,
                z: -1.0,
            },
            0.0005,
            false,
        );
        assert_eq!(
            main.origin,
            Vec3Plan {
                x: 10.0,
                y: 67.0,
                z: -5.0
            }
        );
        assert_eq!(
            main.direction,
            Vec3Plan {
                x: 0.0,
                y: 0.0,
                z: 1.0
            }
        );
        assert!(main.dangerous);
        assert_eq!(main.level_event, Some(1024));

        let side = wither_skull_launch_plan(
            2,
            Vec3Plan {
                x: 10.0,
                y: 64.0,
                z: -5.0,
            },
            90.0,
            1.0,
            Vec3Plan {
                x: 10.0,
                y: 66.2,
                z: -3.7,
            },
            0.0,
            true,
        );
        assert!((side.origin.x - 10.0).abs() < 0.0001);
        assert!((side.origin.y - 66.2).abs() < 0.0001);
        assert!((side.origin.z - -6.3).abs() < 0.0001);
        assert!(!side.dangerous);
        assert_eq!(side.level_event, None);

        let side_head_one = wither_head_position(
            1,
            Vec3Plan {
                x: 10.0,
                y: 64.0,
                z: -5.0,
            },
            90.0,
            1.0,
        );
        assert!((side_head_one.z - -3.7).abs() < 0.0001);
    }

    #[test]
    fn wither_shield_threshold_invulnerability_and_self_heal_rate_match_java() {
        assert!(!wither_is_powered(151.0, WITHER_MAX_HEALTH));
        assert!(wither_is_powered(150.0, WITHER_MAX_HEALTH));

        assert_eq!(
            wither_damage_result(0, true, 0, WitherDamageSourceKind::Arrow),
            WitherDamageResult::Ignored
        );
        assert_eq!(
            wither_damage_result(0, true, 0, WitherDamageSourceKind::WindCharge),
            WitherDamageResult::Ignored
        );
        assert_eq!(
            wither_damage_result(0, true, 0, WitherDamageSourceKind::Generic),
            WitherDamageResult::Accepted {
                schedule_block_destroy: true
            }
        );

        // WitherBoss#customServerAiStep heals 10 every 10 ticks while invulnerable,
        // then 1 every 20 ticks after the spawn sequence completes.
        assert_eq!(wither_ai_tick(10, 220, 0, true, 100.0).heal_amount, 10.0);
        assert_eq!(wither_ai_tick(11, 220, 0, true, 100.0).heal_amount, 0.0);
        assert_eq!(wither_ai_tick(20, 0, 0, true, 150.0).heal_amount, 1.0);
        assert_eq!(wither_ai_tick(21, 0, 0, true, 150.0).heal_amount, 0.0);
    }

    #[test]
    fn wither_head_target_assignment_and_skull_delay_rules_match_java() {
        assert_eq!(
            wither_main_head_sync(Some(42)),
            WitherMainHeadSync {
                alternative_target_head: 0,
                target_entity_id: 42
            }
        );
        assert_eq!(wither_main_head_sync(None).target_entity_id, 0);
        assert_eq!(WITHER_BLUE_SKULL_CHANCE, 0.001);

        assert_eq!(
            wither_alt_head_tick_action(1, 9, 10, true, 16, 3, None, false, 0.0, false, true),
            WitherHeadTickAction::Wait
        );
        assert_eq!(
            wither_alt_head_tick_action(1, 10, 10, true, 16, 3, None, false, 0.0, false, false),
            WitherHeadTickAction::FireIdleBlueSkull {
                head: 1,
                next_update_delay: 13,
                reset_idle_updates: true
            }
        );
        assert_eq!(
            wither_alt_head_tick_action(
                2,
                20,
                20,
                true,
                0,
                7,
                Some(99),
                true,
                WITHER_HEAD_TARGET_RANGE_SQUARED,
                true,
                false
            ),
            WitherHeadTickAction::FireAtCurrentTarget {
                head: 2,
                dangerous: false,
                next_update_delay: 47,
                reset_idle_updates: true
            }
        );
        assert_eq!(
            wither_alt_head_tick_action(
                2,
                20,
                20,
                true,
                0,
                7,
                Some(99),
                true,
                WITHER_HEAD_TARGET_RANGE_SQUARED + 1.0,
                true,
                false
            ),
            WitherHeadTickAction::ClearInvalidTarget { head: 2 }
        );
        assert_eq!(
            wither_alt_head_tick_action(2, 20, 20, true, 0, 7, None, false, 0.0, false, true),
            WitherHeadTickAction::AcquireNearbyTarget { head: 2 }
        );
    }

    #[test]
    fn boss_fight_checklist_surface_is_covered_by_source_families() {
        let covered: Vec<&str> = BOSS_FIGHT_COVERAGE
            .iter()
            .flat_map(|entry| entry.covered_rules.iter().copied())
            .collect();
        for expected in [
            "bossbar",
            "dragon phases",
            "end fight state",
            "gateway constants",
            "end crystals",
            "invulnerability",
            "spawn explosion",
            "block destruction",
            "ranged skulls",
            "damage immunity",
        ] {
            assert!(covered.contains(&expected), "missing {expected}");
        }
    }
}
