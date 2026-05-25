use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriedGhastTickPlan {
    Dehydrate {
        hydration_level: i32,
        game_event: &'static str,
    },
    Hydrate {
        hydration_level: i32,
        sound: &'static str,
        game_event: &'static str,
    },
    SpawnGhastling {
        remove_block: bool,
        baby: bool,
        sound: &'static str,
    },
}

pub fn dried_ghast_tick_plan(waterlogged: bool, hydration_level: i32) -> DriedGhastTickPlan {
    if waterlogged {
        if hydration_level == DRIED_GHAST_READY_HYDRATION_LEVEL {
            DriedGhastTickPlan::SpawnGhastling {
                remove_block: true,
                baby: true,
                sound: "minecraft:entity.ghastling.spawn",
            }
        } else {
            DriedGhastTickPlan::Hydrate {
                hydration_level: hydration_level + 1,
                sound: "minecraft:block.dried_ghast.transition",
                game_event: "minecraft:block_change",
            }
        }
    } else if hydration_level > 0 {
        DriedGhastTickPlan::Dehydrate {
            hydration_level: hydration_level - 1,
            game_event: "minecraft:block_change",
        }
    } else {
        DriedGhastTickPlan::Dehydrate {
            hydration_level: 0,
            game_event: "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnifferStateModel {
    Idling,
    FeelingHappy,
    Scenting,
    Sniffing,
    Searching,
    Digging,
    Rising,
}

impl SnifferStateModel {
    pub fn id(self) -> i32 {
        match self {
            Self::Idling => 0,
            Self::FeelingHappy => 1,
            Self::Scenting => 2,
            Self::Sniffing => 3,
            Self::Searching => 4,
            Self::Digging => 5,
            Self::Rising => 6,
        }
    }
}

pub fn sniffer_state_by_id(id: i32) -> SnifferStateModel {
    match id {
        1 => SnifferStateModel::FeelingHappy,
        2 => SnifferStateModel::Scenting,
        3 => SnifferStateModel::Sniffing,
        4 => SnifferStateModel::Searching,
        5 => SnifferStateModel::Digging,
        6 => SnifferStateModel::Rising,
        _ => SnifferStateModel::Idling,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnifferAttributes {
    pub movement_speed: f32,
    pub max_health: f32,
}

pub fn sniffer_attributes() -> SnifferAttributes {
    SnifferAttributes {
        movement_speed: SNIFFER_MOVEMENT_SPEED,
        max_health: SNIFFER_MAX_HEALTH,
    }
}

pub fn sniffer_food_item(item: &str) -> bool {
    item == "minecraft:torchflower_seeds"
}

pub fn sniffer_diggable_block(block: &str) -> bool {
    matches!(
        block,
        "minecraft:grass_block"
            | "minecraft:podzol"
            | "minecraft:dirt"
            | "minecraft:coarse_dirt"
            | "minecraft:rooted_dirt"
            | "minecraft:mud"
            | "minecraft:muddy_mangrove_roots"
            | "minecraft:moss_block"
    )
}

pub fn sniffer_egg_hatch_boost_block(block: &str) -> bool {
    block == "minecraft:moss_block"
}

pub fn sniffer_can_sniff(
    tempted: bool,
    panicking: bool,
    in_water: bool,
    in_love: bool,
    on_ground: bool,
    passenger: bool,
    leashed: bool,
) -> bool {
    !tempted && !panicking && !in_water && !in_love && on_ground && !passenger && !leashed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnifferDigBodyStateInput {
    pub panicking: bool,
    pub tempted: bool,
    pub baby: bool,
    pub in_water: bool,
    pub on_ground: bool,
    pub passenger: bool,
    pub head_block_below_diggable: bool,
    pub explored_position: bool,
    pub path_can_reach: bool,
}

pub fn sniffer_can_dig_body_state(input: SnifferDigBodyStateInput) -> bool {
    !input.panicking
        && !input.tempted
        && !input.baby
        && !input.in_water
        && input.on_ground
        && !input.passenger
        && input.head_block_below_diggable
        && !input.explored_position
        && input.path_can_reach
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnifferTransitionPlan {
    pub state: SnifferStateModel,
    pub sound: Option<&'static str>,
    pub drop_seed_at_tick: Option<i32>,
    pub event: Option<u8>,
}

pub fn sniffer_transition_plan(
    state: SnifferStateModel,
    tick_count: i32,
    baby: bool,
) -> SnifferTransitionPlan {
    match state {
        SnifferStateModel::Idling => SnifferTransitionPlan {
            state,
            sound: None,
            drop_seed_at_tick: None,
            event: None,
        },
        SnifferStateModel::FeelingHappy => SnifferTransitionPlan {
            state,
            sound: Some("minecraft:entity.sniffer.happy"),
            drop_seed_at_tick: None,
            event: None,
        },
        SnifferStateModel::Scenting => SnifferTransitionPlan {
            state,
            sound: Some(if baby {
                "minecraft:entity.sniffer.scenting@1.3"
            } else {
                "minecraft:entity.sniffer.scenting@1.0"
            }),
            drop_seed_at_tick: None,
            event: None,
        },
        SnifferStateModel::Sniffing => SnifferTransitionPlan {
            state,
            sound: Some("minecraft:entity.sniffer.sniffing"),
            drop_seed_at_tick: None,
            event: None,
        },
        SnifferStateModel::Searching => SnifferTransitionPlan {
            state,
            sound: None,
            drop_seed_at_tick: None,
            event: None,
        },
        SnifferStateModel::Digging => SnifferTransitionPlan {
            state,
            sound: None,
            drop_seed_at_tick: Some(tick_count + SNIFFER_DIGGING_DROP_SEED_OFFSET_TICKS),
            event: Some(63),
        },
        SnifferStateModel::Rising => SnifferTransitionPlan {
            state,
            sound: Some("minecraft:entity.sniffer.digging_stop"),
            drop_seed_at_tick: None,
            event: None,
        },
    }
}

pub fn sniffer_can_play_digging_sound(state: SnifferStateModel) -> bool {
    matches!(
        state,
        SnifferStateModel::Digging | SnifferStateModel::Searching
    )
}

pub fn sniffer_ambient_sound(state: SnifferStateModel) -> Option<&'static str> {
    if sniffer_can_play_digging_sound(state) {
        None
    } else {
        Some("minecraft:entity.sniffer.idle")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnifferDiggingTickPlan {
    pub drop_seed: bool,
    pub seed_loot_table: Option<&'static str>,
    pub seed_sound: Option<&'static str>,
    pub particles: i32,
    pub block_hit_sound: bool,
    pub game_event: bool,
}

pub fn sniffer_digging_tick_plan(
    state: SnifferStateModel,
    tick_count: i32,
    drop_seed_at_tick: i32,
    digging_animation_time_ms: i32,
    state_below_visible: bool,
) -> SnifferDiggingTickPlan {
    let digging = state == SnifferStateModel::Digging;
    let emit_particles = digging
        && digging_animation_time_ms > SNIFFER_DIGGING_PARTICLES_DELAY_TICKS
        && digging_animation_time_ms < SNIFFER_DIGGING_PARTICLES_DURATION_TICKS
        && state_below_visible;
    let drop_seed = digging && tick_count == drop_seed_at_tick;
    SnifferDiggingTickPlan {
        drop_seed,
        seed_loot_table: drop_seed.then_some("minecraft:gameplay/sniffer_digging"),
        seed_sound: drop_seed.then_some("minecraft:entity.sniffer.drop_seed"),
        particles: if emit_particles {
            SNIFFER_DIGGING_PARTICLES_AMOUNT
        } else {
            0
        },
        block_hit_sound: emit_particles && tick_count % 10 == 0,
        game_event: digging && tick_count % 10 == 0,
    }
}

pub fn sniffer_store_explored_position<T: Copy>(existing: &[T], new_pos: T) -> Vec<T> {
    let mut updated: Vec<T> = existing
        .iter()
        .copied()
        .take(SNIFFER_EXPLORED_POSITION_LIMIT)
        .collect();
    updated.insert(0, new_pos);
    updated
}

pub fn sniffer_can_mate_state(first: SnifferStateModel, second: SnifferStateModel) -> bool {
    let allowed = |state| {
        matches!(
            state,
            SnifferStateModel::Idling
                | SnifferStateModel::Scenting
                | SnifferStateModel::FeelingHappy
        )
    };
    allowed(first) && allowed(second)
}

pub fn sniffer_breeding_drop_plan() -> (&'static str, &'static str) {
    ("minecraft:sniffer_egg", "minecraft:block.sniffer_egg.plop")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnifferEggTickPlan {
    Crack {
        hatch_level: i32,
        sound: &'static str,
    },
    Hatch {
        destroy_block: bool,
        spawn_baby: bool,
        sound: &'static str,
    },
}

pub fn sniffer_egg_next_tick_delay(boosted: bool, random_offset: i32) -> i32 {
    let hatch_time = if boosted {
        SNIFFER_EGG_BOOSTED_HATCH_TIME_TICKS
    } else {
        SNIFFER_EGG_REGULAR_HATCH_TIME_TICKS
    };
    hatch_time / 3 + random_offset
}

pub fn sniffer_egg_tick_plan(hatch_level: i32) -> SnifferEggTickPlan {
    if hatch_level < SNIFFER_EGG_MAX_HATCH_LEVEL {
        SnifferEggTickPlan::Crack {
            hatch_level: hatch_level + 1,
            sound: "minecraft:block.sniffer_egg.crack",
        }
    } else {
        SnifferEggTickPlan::Hatch {
            destroy_block: true,
            spawn_baby: true,
            sound: "minecraft:block.sniffer_egg.hatch",
        }
    }
}
