use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmadilloStateModel {
    Idle,
    Rolling,
    Scared,
    Unrolling,
}

impl ArmadilloStateModel {
    pub fn id(self) -> i32 {
        match self {
            Self::Idle => 0,
            Self::Rolling => 1,
            Self::Scared => 2,
            Self::Unrolling => 3,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Rolling => "rolling",
            Self::Scared => "scared",
            Self::Unrolling => "unrolling",
        }
    }

    pub fn animation_duration(self) -> i32 {
        match self {
            Self::Idle => 0,
            Self::Rolling => 10,
            Self::Scared => 50,
            Self::Unrolling => 30,
        }
    }

    pub fn is_threatened(self) -> bool {
        self != Self::Idle
    }

    pub fn should_hide_in_shell(self, ticks_in_state: i64) -> bool {
        match self {
            Self::Idle => false,
            Self::Rolling => ticks_in_state > 5,
            Self::Scared => true,
            Self::Unrolling => ticks_in_state < 26,
        }
    }
}

pub fn armadillo_state_by_id(id: i32) -> ArmadilloStateModel {
    match id {
        1 => ArmadilloStateModel::Rolling,
        2 => ArmadilloStateModel::Scared,
        3 => ArmadilloStateModel::Unrolling,
        _ => ArmadilloStateModel::Idle,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmadilloAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
}

pub fn armadillo_attributes() -> ArmadilloAttributes {
    ArmadilloAttributes {
        max_health: ARMADILLO_MAX_HEALTH,
        movement_speed: ARMADILLO_MOVEMENT_SPEED,
    }
}

pub fn armadillo_food_item(item: &str) -> bool {
    item == "minecraft:spider_eye"
}

pub fn armadillo_spawnable_on(block: &str) -> bool {
    matches!(
        block,
        "minecraft:grass_block"
            | "minecraft:red_sand"
            | "minecraft:coarse_dirt"
            | "minecraft:terracotta"
            | "minecraft:white_terracotta"
            | "minecraft:orange_terracotta"
            | "minecraft:magenta_terracotta"
            | "minecraft:light_blue_terracotta"
            | "minecraft:yellow_terracotta"
            | "minecraft:lime_terracotta"
            | "minecraft:pink_terracotta"
            | "minecraft:gray_terracotta"
            | "minecraft:light_gray_terracotta"
            | "minecraft:cyan_terracotta"
            | "minecraft:purple_terracotta"
            | "minecraft:blue_terracotta"
            | "minecraft:brown_terracotta"
            | "minecraft:green_terracotta"
            | "minecraft:red_terracotta"
            | "minecraft:black_terracotta"
    )
}

pub fn armadillo_spawn_allowed(block_below: &str, bright_enough: bool) -> bool {
    armadillo_spawnable_on(block_below) && bright_enough
}

pub fn armadillo_pick_next_scute_drop_time(random_offset: i32) -> i32 {
    ARMADILLO_SCUTE_DROP_MIN_TICKS + random_offset.clamp(0, ARMADILLO_SCUTE_DROP_RANDOM_BOUND - 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmadilloScuteDropPlan {
    pub next_scute_time: i32,
    pub loot_table: Option<&'static str>,
    pub sound: Option<&'static str>,
    pub game_event: Option<&'static str>,
}

pub fn armadillo_scute_drop_tick(
    alive: bool,
    scute_time: i32,
    should_drop_loot: bool,
    loot_dropped: bool,
    next_random_offset: i32,
) -> ArmadilloScuteDropPlan {
    if alive && scute_time - 1 <= 0 && should_drop_loot {
        ArmadilloScuteDropPlan {
            next_scute_time: armadillo_pick_next_scute_drop_time(next_random_offset),
            loot_table: Some("minecraft:gameplay/armadillo_shed"),
            sound: loot_dropped.then_some("minecraft:entity.armadillo.scute_drop"),
            game_event: loot_dropped.then_some("minecraft:entity_place"),
        }
    } else {
        ArmadilloScuteDropPlan {
            next_scute_time: scute_time - 1,
            loot_table: None,
            sound: None,
            game_event: None,
        }
    }
}

pub fn armadillo_is_scared_by(
    intersects_in_scare_box: bool,
    target_undead: bool,
    target_last_hurt_by: bool,
    target_is_player: bool,
    player_spectator: bool,
    player_sprinting: bool,
    player_passenger: bool,
) -> bool {
    if !intersects_in_scare_box {
        false
    } else if target_undead || target_last_hurt_by {
        true
    } else if target_is_player {
        !player_spectator && (player_sprinting || player_passenger)
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmadilloRollPlan {
    pub state: ArmadilloStateModel,
    pub sound: &'static str,
    pub game_event: &'static str,
    pub stop_in_place: bool,
    pub reset_love: bool,
}

pub fn armadillo_roll_up_plan(state: ArmadilloStateModel) -> Option<ArmadilloRollPlan> {
    if state.is_threatened() {
        None
    } else {
        Some(ArmadilloRollPlan {
            state: ArmadilloStateModel::Rolling,
            sound: "minecraft:entity.armadillo.roll",
            game_event: "minecraft:entity_action",
            stop_in_place: true,
            reset_love: true,
        })
    }
}

pub fn armadillo_roll_out_plan(state: ArmadilloStateModel) -> Option<ArmadilloRollPlan> {
    if state.is_threatened() {
        Some(ArmadilloRollPlan {
            state: ArmadilloStateModel::Idle,
            sound: "minecraft:entity.armadillo.unroll_finish",
            game_event: "minecraft:entity_action",
            stop_in_place: false,
            reset_love: false,
        })
    } else {
        None
    }
}

pub fn armadillo_can_stay_rolled_up(
    panicking: bool,
    in_liquid: bool,
    leashed: bool,
    passenger: bool,
    vehicle: bool,
) -> bool {
    !panicking && !in_liquid && !leashed && !passenger && !vehicle
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmadilloHurtReaction {
    DangerMemory { ticks: i32, roll_up: bool },
    RollOutEnvironmental,
    None,
}

pub fn armadillo_damage_after_shell(state: ArmadilloStateModel, damage: f32) -> f32 {
    if state.is_threatened() {
        (damage - 1.0) / 2.0
    } else {
        damage
    }
}

pub fn armadillo_hurt_reaction(
    no_ai: bool,
    dead_or_dying: bool,
    source_entity_living: bool,
    panic_environmental_source: bool,
    can_stay_rolled_up: bool,
) -> ArmadilloHurtReaction {
    if no_ai || dead_or_dying {
        ArmadilloHurtReaction::None
    } else if source_entity_living {
        ArmadilloHurtReaction::DangerMemory {
            ticks: ARMADILLO_SCARE_CHECK_INTERVAL,
            roll_up: can_stay_rolled_up,
        }
    } else if panic_environmental_source {
        ArmadilloHurtReaction::RollOutEnvironmental
    } else {
        ArmadilloHurtReaction::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmadilloInteractPlan {
    Brush {
        loot_table: &'static str,
        sound: &'static str,
        game_event: &'static str,
        tool_damage: i32,
    },
    FailScared,
    Delegate,
}

pub fn armadillo_interact_plan(item: &str, baby: bool, scared: bool) -> ArmadilloInteractPlan {
    if item == "minecraft:brush" && !baby {
        ArmadilloInteractPlan::Brush {
            loot_table: "minecraft:gameplay/armadillo_brush",
            sound: "minecraft:entity.armadillo.brush",
            game_event: "minecraft:entity_interact",
            tool_damage: ARMADILLO_BRUSH_DAMAGE,
        }
    } else if scared {
        ArmadilloInteractPlan::FailScared
    } else {
        ArmadilloInteractPlan::Delegate
    }
}

pub fn armadillo_can_fall_in_love(super_can_fall_in_love: bool, scared: bool) -> bool {
    super_can_fall_in_love && !scared
}

pub fn armadillo_ambient_sound(scared: bool) -> Option<&'static str> {
    (!scared).then_some("minecraft:entity.armadillo.ambient")
}

pub fn armadillo_hurt_sound(scared: bool) -> &'static str {
    if scared {
        "minecraft:entity.armadillo.hurt_reduced"
    } else {
        "minecraft:entity.armadillo.hurt"
    }
}

pub fn armadillo_max_head_y_rot(scared: bool) -> i32 {
    if scared {
        0
    } else {
        32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmadilloBallUpTickPlan {
    SwitchToScared {
        sound: Option<&'static str>,
    },
    Peek {
        event: u8,
        next_peek_timer_min: i32,
        next_peek_timer_max: i32,
    },
    StartUnrolling {
        sound: &'static str,
    },
    ReturnToScared,
    None,
}

pub fn armadillo_ball_up_tick_plan(
    state: ArmadilloStateModel,
    in_state_ticks: i64,
    on_ground: bool,
    danger_ticks_remaining: i64,
    next_peek_timer: i32,
) -> ArmadilloBallUpTickPlan {
    if state == ArmadilloStateModel::Rolling
        && in_state_ticks > ArmadilloStateModel::Rolling.animation_duration() as i64
    {
        return ArmadilloBallUpTickPlan::SwitchToScared {
            sound: on_ground.then_some("minecraft:entity.armadillo.land"),
        };
    }

    let danger_is_around = danger_ticks_remaining > ARMADILLO_DANGER_THRESHOLD_TICKS as i64;
    if state == ArmadilloStateModel::Scared {
        if next_peek_timer == 0 && on_ground && danger_is_around {
            return ArmadilloBallUpTickPlan::Peek {
                event: ARMADILLO_PEEK_EVENT,
                next_peek_timer_min: ArmadilloStateModel::Scared.animation_duration() + 100,
                next_peek_timer_max: ArmadilloStateModel::Scared.animation_duration() + 400,
            };
        }
        if danger_ticks_remaining < ArmadilloStateModel::Unrolling.animation_duration() as i64 {
            return ArmadilloBallUpTickPlan::StartUnrolling {
                sound: "minecraft:entity.armadillo.unroll_start",
            };
        }
    } else if state == ArmadilloStateModel::Unrolling
        && danger_ticks_remaining > ArmadilloStateModel::Unrolling.animation_duration() as i64
    {
        return ArmadilloBallUpTickPlan::ReturnToScared;
    }

    ArmadilloBallUpTickPlan::None
}

