#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreakingAiActivityStep {
    pub activity: &'static str,
    pub priority: i32,
    pub behavior: &'static str,
}

pub const CREAKING_AI_CORE_SWIM_SPEED: f32 = 0.8;
pub const CREAKING_AI_LOOK_MIN_Y_ROT: i32 = 45;
pub const CREAKING_AI_LOOK_MAX_X_ROT: i32 = 90;
pub const CREAKING_AI_IDLE_PRIORITY: i32 = 10;
pub const CREAKING_AI_FIGHT_PRIORITY: i32 = 10;
pub const CREAKING_AI_LOOK_TARGET_RANGE: f32 = 8.0;
pub const CREAKING_AI_LOOK_INTERVAL_MIN: i32 = 30;
pub const CREAKING_AI_LOOK_INTERVAL_MAX: i32 = 60;
pub const CREAKING_AI_RANDOM_STROLL_SPEED: f32 = 0.3;
pub const CREAKING_AI_SET_WALK_FROM_LOOK_SPEED: f32 = 0.3;
pub const CREAKING_AI_SET_WALK_FROM_LOOK_CLOSE_ENOUGH: i32 = 3;
pub const CREAKING_AI_DO_NOTHING_MIN_TICKS: i32 = 30;
pub const CREAKING_AI_DO_NOTHING_MAX_TICKS: i32 = 60;
pub const CREAKING_AI_RANDOM_STROLL_WEIGHT: i32 = 2;
pub const CREAKING_AI_WALK_FROM_LOOK_WEIGHT: i32 = 2;
pub const CREAKING_AI_DO_NOTHING_WEIGHT: i32 = 1;
pub const CREAKING_AI_ATTACK_WALK_SPEED: f32 = 1.0;
pub const CREAKING_AI_MELEE_COOLDOWN_TICKS: i32 = 40;

pub const CREAKING_AI_ACTIVITY_ORDER: [&str; 3] = ["core", "idle", "fight"];

pub const CREAKING_AI_CORE_ACTIVITY: [CreakingAiActivityStep; 3] = [
    CreakingAiActivityStep {
        activity: "core",
        priority: 0,
        behavior: "swim_if_can_move",
    },
    CreakingAiActivityStep {
        activity: "core",
        priority: 0,
        behavior: "look_at_target_sink",
    },
    CreakingAiActivityStep {
        activity: "core",
        priority: 0,
        behavior: "move_to_target_sink",
    },
];

pub const CREAKING_AI_IDLE_ACTIVITY: [CreakingAiActivityStep; 4] = [
    CreakingAiActivityStep {
        activity: "idle",
        priority: 0,
        behavior: "start_attacking_nearest_visible_attackable_player_when_active",
    },
    CreakingAiActivityStep {
        activity: "idle",
        priority: 1,
        behavior: "set_entity_look_target_sometimes",
    },
    CreakingAiActivityStep {
        activity: "idle",
        priority: 2,
        behavior: "random_stroll",
    },
    CreakingAiActivityStep {
        activity: "idle",
        priority: 2,
        behavior: "set_walk_target_from_look_target",
    },
];

pub const CREAKING_AI_FIGHT_ACTIVITY: [CreakingAiActivityStep; 3] = [
    CreakingAiActivityStep {
        activity: "fight",
        priority: 0,
        behavior: "set_walk_target_from_attack_target_if_out_of_reach",
    },
    CreakingAiActivityStep {
        activity: "fight",
        priority: 0,
        behavior: "melee_attack_if_can_move",
    },
    CreakingAiActivityStep {
        activity: "fight",
        priority: 0,
        behavior: "stop_attacking_if_target_invalid",
    },
];

pub const CREAKING_AI_FIGHT_REQUIREMENTS: [(&str, &str); 1] =
    [("attack_target", "value_present")];

pub fn creaking_ai_swim_can_start(can_move: bool, swim_conditions_pass: bool) -> bool {
    can_move && swim_conditions_pass
}

pub fn creaking_ai_idle_start_target(is_active: bool, nearest_visible_attackable_player: bool) -> bool {
    is_active && nearest_visible_attackable_player
}

pub fn creaking_ai_melee_can_start(can_move: bool) -> bool {
    can_move
}

pub fn creaking_ai_attack_target_still_reachable(
    target_is_player: bool,
    visible_attackable_players_present: bool,
    visible_attackable_players_contains_target: bool,
) -> bool {
    visible_attackable_players_present && target_is_player && visible_attackable_players_contains_target
}

pub fn creaking_ai_update_activity(can_move: bool) -> CreakingAiActivityUpdate {
    if can_move {
        CreakingAiActivityUpdate {
            use_default_activity: false,
            first_valid_order: Some(["fight", "idle"]),
        }
    } else {
        CreakingAiActivityUpdate {
            use_default_activity: true,
            first_valid_order: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreakingAiActivityUpdate {
    pub use_default_activity: bool,
    pub first_valid_order: Option<[&'static str; 2]>,
}
