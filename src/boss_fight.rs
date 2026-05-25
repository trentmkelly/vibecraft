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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WitherAltHeadTickContext {
    pub head: i32,
    pub tick_count: i32,
    pub next_head_update: i32,
    pub difficulty_allows_idle_attack: bool,
    pub idle_head_updates: i32,
    pub random_0_to_9: i32,
    pub alternative_target_entity_id: Option<i32>,
    pub current_target_valid: bool,
    pub current_target_distance_sqr: f32,
    pub current_target_line_of_sight: bool,
    pub nearby_targets_available: bool,
}

pub fn wither_alt_head_tick_action(context: WitherAltHeadTickContext) -> WitherHeadTickAction {
    if context.tick_count < context.next_head_update {
        return WitherHeadTickAction::Wait;
    }
    let next_update_delay = WITHER_ALT_HEAD_UPDATE_DELAY_MIN
        + context
            .random_0_to_9
            .rem_euclid(WITHER_ALT_HEAD_UPDATE_DELAY_RANDOM_BOUND);
    if context.difficulty_allows_idle_attack
        && context.idle_head_updates > WITHER_IDLE_HEAD_ATTACK_THRESHOLD
    {
        return WitherHeadTickAction::FireIdleBlueSkull {
            head: context.head,
            next_update_delay,
            reset_idle_updates: true,
        };
    }
    if context.alternative_target_entity_id.is_some() {
        if context.current_target_valid
            && context.current_target_distance_sqr <= WITHER_HEAD_TARGET_RANGE_SQUARED
            && context.current_target_line_of_sight
        {
            return WitherHeadTickAction::FireAtCurrentTarget {
                head: context.head,
                dangerous: false,
                next_update_delay: WITHER_ALT_HEAD_TARGET_ATTACK_DELAY_MIN
                    + context
                        .random_0_to_9
                        .rem_euclid(WITHER_ALT_HEAD_TARGET_ATTACK_DELAY_RANDOM_BOUND),
                reset_idle_updates: true,
            };
        }
        return WitherHeadTickAction::ClearInvalidTarget { head: context.head };
    }
    if context.nearby_targets_available {
        WitherHeadTickAction::AcquireNearbyTarget { head: context.head }
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
mod tests;
