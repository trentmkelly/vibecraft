#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrolBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatrolVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrollingMonsterState {
    pub patrol_target: Option<PatrolBlockPos>,
    pub patrol_leader: bool,
    pub patrolling: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrollingMonsterFinalizeInput {
    pub spawn_reason: PatrolSpawnReason,
    pub random_float: u32,
    pub can_be_leader: bool,
    pub already_patrol_leader: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatrolSpawnReason {
    Patrol,
    Event,
    Structure,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrollingMonsterFinalize {
    pub patrol_leader: bool,
    pub patrolling: bool,
    pub equip_ominous_banner: bool,
    pub head_drop_chance_percent: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrollingMonsterSpawnRuleInput {
    pub block_brightness: i32,
    pub peaceful_difficulty: bool,
    pub mob_spawn_rules_pass: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LongDistancePatrolCanUseInput {
    pub patrolling: bool,
    pub target_present: bool,
    pub controlling_passenger: bool,
    pub patrol_target_present: bool,
    pub game_time: i64,
    pub cooldown_until: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LongDistancePatrolTickInput {
    pub navigation_done: bool,
    pub patrolling: bool,
    pub companion_count: usize,
    pub patrol_leader: bool,
    pub close_to_patrol_target: bool,
    pub navigation_move_succeeds: bool,
    pub game_time: i64,
    pub self_position: PatrolVec3,
    pub patrol_target: PatrolBlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LongDistancePatrolTick {
    Noop,
    StopPatrolling,
    FindNewPatrolTarget,
    MoveTo {
        path_target: PatrolBlockPos,
        speed_modifier: f64,
        share_target_with_companions: bool,
    },
    MoveRandomly {
        cooldown_until: i64,
    },
}

pub const PATROLLING_MONSTER_DEFAULT_PATROL_LEADER: bool = false;
pub const PATROLLING_MONSTER_DEFAULT_PATROLLING: bool = false;
pub const PATROLLING_MONSTER_LEADER_CHANCE_PERCENT: u32 = 6;
pub const PATROLLING_MONSTER_HEAD_DROP_CHANCE_PERCENT: i32 = 200;
pub const PATROLLING_MONSTER_PATROL_GOAL_PRIORITY: i32 = 4;
pub const PATROLLING_MONSTER_PATROL_SPEED: f64 = 0.7;
pub const PATROLLING_MONSTER_LEADER_PATROL_SPEED: f64 = 0.595;
pub const PATROLLING_MONSTER_REMOVE_WHEN_FAR_DIST_SQR: f64 = 16_384.0;
pub const PATROLLING_MONSTER_COMPANION_RANGE: f64 = 16.0;
pub const PATROLLING_MONSTER_TARGET_REACHED_DISTANCE: f64 = 10.0;
pub const PATROLLING_MONSTER_NAVIGATION_FAILED_COOLDOWN: i64 = 200;
pub const PATROLLING_MONSTER_RANDOM_TARGET_OFFSET_BASE: i32 = -500;
pub const PATROLLING_MONSTER_RANDOM_TARGET_OFFSET_BOUND: i32 = 1000;
pub const PATROLLING_MONSTER_RANDOM_MOVE_OFFSET_BASE: i32 = -8;
pub const PATROLLING_MONSTER_RANDOM_MOVE_OFFSET_BOUND: i32 = 16;

pub fn patrolling_monster_default_state() -> PatrollingMonsterState {
    PatrollingMonsterState {
        patrol_target: None,
        patrol_leader: PATROLLING_MONSTER_DEFAULT_PATROL_LEADER,
        patrolling: PATROLLING_MONSTER_DEFAULT_PATROLLING,
    }
}

pub fn patrolling_monster_finalize_spawn(
    input: PatrollingMonsterFinalizeInput,
) -> PatrollingMonsterFinalize {
    let mut patrol_leader = input.already_patrol_leader;
    let mut patrolling = false;
    if !matches!(
        input.spawn_reason,
        PatrolSpawnReason::Patrol | PatrolSpawnReason::Event | PatrolSpawnReason::Structure
    ) && input.random_float < PATROLLING_MONSTER_LEADER_CHANCE_PERCENT
        && input.can_be_leader
    {
        patrol_leader = true;
    }
    let equip_ominous_banner = patrol_leader;
    if matches!(input.spawn_reason, PatrolSpawnReason::Patrol) {
        patrolling = true;
    }

    PatrollingMonsterFinalize {
        patrol_leader,
        patrolling,
        equip_ominous_banner,
        head_drop_chance_percent: if equip_ominous_banner {
            PATROLLING_MONSTER_HEAD_DROP_CHANCE_PERCENT
        } else {
            0
        },
    }
}

pub fn patrolling_monster_spawn_rules(input: PatrollingMonsterSpawnRuleInput) -> bool {
    input.block_brightness <= 8 && !input.peaceful_difficulty && input.mob_spawn_rules_pass
}

pub fn patrolling_monster_remove_when_far_away(patrolling: bool, dist_sqr: f64) -> bool {
    !patrolling || dist_sqr > PATROLLING_MONSTER_REMOVE_WHEN_FAR_DIST_SQR
}

pub fn patrolling_monster_set_patrol_target(
    mut state: PatrollingMonsterState,
    target: PatrolBlockPos,
) -> PatrollingMonsterState {
    state.patrol_target = Some(target);
    state.patrolling = true;
    state
}

pub fn patrolling_monster_set_patrol_leader(
    mut state: PatrollingMonsterState,
    is_leader: bool,
) -> PatrollingMonsterState {
    state.patrol_leader = is_leader;
    state.patrolling = true;
    state
}

pub fn patrolling_monster_find_patrol_target(
    block_position: PatrolBlockPos,
    random_x: i32,
    random_z: i32,
) -> PatrollingMonsterState {
    PatrollingMonsterState {
        patrol_target: Some(PatrolBlockPos {
            x: block_position.x + PATROLLING_MONSTER_RANDOM_TARGET_OFFSET_BASE + random_x,
            y: block_position.y,
            z: block_position.z + PATROLLING_MONSTER_RANDOM_TARGET_OFFSET_BASE + random_z,
        }),
        patrol_leader: false,
        patrolling: true,
    }
}

pub fn long_distance_patrol_can_use(input: LongDistancePatrolCanUseInput) -> bool {
    let on_cooldown = input.game_time < input.cooldown_until;
    input.patrolling
        && !input.target_present
        && !input.controlling_passenger
        && input.patrol_target_present
        && !on_cooldown
}

pub fn long_distance_patrol_tick(input: LongDistancePatrolTickInput) -> LongDistancePatrolTick {
    if !input.navigation_done {
        return LongDistancePatrolTick::Noop;
    }

    if input.patrolling && input.companion_count == 0 {
        return LongDistancePatrolTick::StopPatrolling;
    }

    if input.patrol_leader && input.close_to_patrol_target {
        return LongDistancePatrolTick::FindNewPatrolTarget;
    }

    let path_target = patrolling_monster_long_distance_path_target(
        input.self_position,
        input.patrol_target,
        input.patrol_target.y,
    );
    if !input.navigation_move_succeeds {
        return LongDistancePatrolTick::MoveRandomly {
            cooldown_until: input.game_time + PATROLLING_MONSTER_NAVIGATION_FAILED_COOLDOWN,
        };
    }

    LongDistancePatrolTick::MoveTo {
        path_target,
        speed_modifier: if input.patrol_leader {
            PATROLLING_MONSTER_LEADER_PATROL_SPEED
        } else {
            PATROLLING_MONSTER_PATROL_SPEED
        },
        share_target_with_companions: input.patrol_leader,
    }
}

pub fn patrolling_monster_long_distance_path_target(
    self_position: PatrolVec3,
    patrol_target: PatrolBlockPos,
    heightmap_y: i32,
) -> PatrolBlockPos {
    let target = PatrolVec3 {
        x: patrol_target.x as f64 + 0.5,
        y: patrol_target.y as f64,
        z: patrol_target.z as f64 + 0.5,
    };
    let distance = PatrolVec3 {
        x: self_position.x - target.x,
        y: self_position.y - target.y,
        z: self_position.z - target.z,
    };
    let rotated_scaled = PatrolVec3 {
        x: distance.z * 0.4,
        y: distance.y * 0.4,
        z: -distance.x * 0.4,
    };
    let long_distance_target = PatrolVec3 {
        x: rotated_scaled.x + target.x,
        y: rotated_scaled.y + target.y,
        z: rotated_scaled.z + target.z,
    };
    let toward = PatrolVec3 {
        x: long_distance_target.x - self_position.x,
        y: long_distance_target.y - self_position.y,
        z: long_distance_target.z - self_position.z,
    };
    let length = (toward.x * toward.x + toward.y * toward.y + toward.z * toward.z).sqrt();
    let move_target = PatrolVec3 {
        x: self_position.x + toward.x / length * 10.0,
        y: self_position.y + toward.y / length * 10.0,
        z: self_position.z + toward.z / length * 10.0,
    };

    PatrolBlockPos {
        x: move_target.x.floor() as i32,
        y: heightmap_y,
        z: move_target.z.floor() as i32,
    }
}
