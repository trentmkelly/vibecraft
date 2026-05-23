use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AllayAttributes {
    pub max_health: f32,
    pub flying_speed: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
}

pub fn allay_attributes() -> AllayAttributes {
    AllayAttributes {
        max_health: ALLAY_MAX_HEALTH,
        flying_speed: ALLAY_FLYING_SPEED,
        movement_speed: ALLAY_MOVEMENT_SPEED,
        attack_damage: ALLAY_ATTACK_DAMAGE,
    }
}

pub fn allay_duplicate_item(item: &str) -> bool {
    item == "minecraft:amethyst_shard"
}

pub fn allay_can_pick_up_loot(pickup_cooldown: bool, has_item_in_hand: bool) -> bool {
    !pickup_cooldown && has_item_in_hand
}

pub fn allay_considers_item_equal(
    hand_item: &str,
    pickup_item: &str,
    hand_potion: Option<&str>,
    pickup_potion: Option<&str>,
) -> bool {
    hand_item == pickup_item && hand_potion == pickup_potion
}

pub fn allay_wants_to_pick_up(
    hand_item: &str,
    pickup_item: &str,
    hand_potion: Option<&str>,
    pickup_potion: Option<&str>,
    mob_griefing: bool,
    inventory_can_add: bool,
) -> bool {
    !hand_item.is_empty()
        && mob_griefing
        && inventory_can_add
        && allay_considers_item_equal(hand_item, pickup_item, hand_potion, pickup_potion)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllayInteractPlan {
    Duplicate {
        consumed: i32,
        parent_cooldown: i32,
        child_cooldown: i32,
        event: u8,
        hearts: i32,
        sound: &'static str,
    },
    GiveItem {
        consumed: i32,
        held_count: i32,
        remember_liked_player: bool,
        sound: &'static str,
    },
    TakeItem {
        clear_liked_player: bool,
        return_held_item: bool,
        throw_inventory: bool,
        sound: &'static str,
    },
    Delegate,
}

pub fn allay_interact_plan(
    interaction_item: &str,
    hand_is_main_hand: bool,
    allay_holding_item: bool,
    dancing: bool,
    can_duplicate: bool,
    player_empty_hand: bool,
) -> AllayInteractPlan {
    if dancing && allay_duplicate_item(interaction_item) && can_duplicate {
        return AllayInteractPlan::Duplicate {
            consumed: 1,
            parent_cooldown: ALLAY_DUPLICATION_COOLDOWN_TICKS,
            child_cooldown: ALLAY_DUPLICATION_COOLDOWN_TICKS,
            event: ALLAY_DUPLICATION_EVENT,
            hearts: ALLAY_NUM_DUPLICATION_HEARTS,
            sound: "minecraft:block.amethyst_block.chime",
        };
    }
    if !allay_holding_item && !interaction_item.is_empty() {
        return AllayInteractPlan::GiveItem {
            consumed: 1,
            held_count: 1,
            remember_liked_player: true,
            sound: "minecraft:entity.allay.item_given",
        };
    }
    if allay_holding_item && hand_is_main_hand && player_empty_hand {
        return AllayInteractPlan::TakeItem {
            clear_liked_player: true,
            return_held_item: true,
            throw_inventory: true,
            sound: "minecraft:entity.allay.item_taken",
        };
    }
    AllayInteractPlan::Delegate
}

pub fn allay_duplication_cooldown_tick(cooldown: i32, client_side: bool) -> (i32, bool) {
    let next = if !client_side && cooldown > 0 {
        cooldown - 1
    } else {
        cooldown
    };
    (next, next == 0)
}

pub fn allay_set_jukebox_playing(
    current_jukebox: Option<(i32, i32, i32)>,
    jukebox: (i32, i32, i32),
    is_playing: bool,
    dancing: bool,
) -> (Option<(i32, i32, i32)>, bool) {
    if is_playing {
        if !dancing {
            (Some(jukebox), true)
        } else {
            (current_jukebox, dancing)
        }
    } else if current_jukebox == Some(jukebox) || current_jukebox.is_none() {
        (None, false)
    } else {
        (current_jukebox, dancing)
    }
}

pub fn allay_should_stop_dancing(
    jukebox_pos: Option<(i32, i32, i32)>,
    distance_to_jukebox_center: f32,
    jukebox_block_still_present: bool,
    notification_radius: f32,
) -> bool {
    jukebox_pos.is_none()
        || distance_to_jukebox_center >= notification_radius
        || !jukebox_block_still_present
}

pub fn allay_set_dancing_allowed(
    client_side: bool,
    effective_ai: bool,
    new_dancing: bool,
    panicking: bool,
) -> bool {
    !client_side && effective_ai && (!new_dancing || !panicking)
}

pub fn allay_is_spinning(dancing_animation_ticks: f32) -> bool {
    dancing_animation_ticks % ALLAY_DANCING_LOOP_DURATION < ALLAY_SPINNING_ANIMATION_DURATION
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllayNoteBlockPlan {
    pub liked_noteblock: Option<(i32, i32, i32)>,
    pub cooldown_ticks: Option<i32>,
}

pub fn allay_hear_noteblock(
    current_liked: Option<(i32, i32, i32)>,
    heard_pos: (i32, i32, i32),
) -> AllayNoteBlockPlan {
    if current_liked.is_none() || current_liked == Some(heard_pos) {
        AllayNoteBlockPlan {
            liked_noteblock: Some(heard_pos),
            cooldown_ticks: Some(ALLAY_TIME_TO_FORGET_NOTEBLOCK),
        }
    } else {
        AllayNoteBlockPlan {
            liked_noteblock: current_liked,
            cooldown_ticks: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllayDepositTarget {
    NoteBlockAbove,
    LikedPlayer,
    None,
}

pub fn allay_deposit_target(
    liked_noteblock: Option<(i32, i32, i32)>,
    note_block_close_enough: bool,
    note_block_still_present: bool,
    noteblock_cooldown_present: bool,
    liked_player_available: bool,
) -> AllayDepositTarget {
    if liked_noteblock.is_some() {
        if note_block_close_enough && note_block_still_present && noteblock_cooldown_present {
            AllayDepositTarget::NoteBlockAbove
        } else if liked_player_available {
            AllayDepositTarget::LikedPlayer
        } else {
            AllayDepositTarget::None
        }
    } else if liked_player_available {
        AllayDepositTarget::LikedPlayer
    } else {
        AllayDepositTarget::None
    }
}

pub fn allay_liked_player_available(
    server_side: bool,
    player_survival_or_creative: bool,
    distance_to_allay: f32,
) -> bool {
    server_side && player_survival_or_creative && distance_to_allay < ALLAY_LIKED_PLAYER_DISTANCE
}

pub fn allay_can_receive_note_vibration(
    no_ai: bool,
    liked_noteblock: Option<(i32, i32, i32)>,
    event_pos: (i32, i32, i32),
    close_enough: bool,
) -> bool {
    if no_ai {
        false
    } else {
        liked_noteblock
            .map(|pos| close_enough && pos == event_pos)
            .unwrap_or(true)
    }
}

pub fn allay_hurt_allowed(source_is_liked_player: bool) -> bool {
    !source_is_liked_player
}

pub fn allay_ambient_sound(has_item: bool) -> &'static str {
    if has_item {
        "minecraft:entity.allay.ambient_with_item"
    } else {
        "minecraft:entity.allay.ambient_without_item"
    }
}

pub fn allay_remove_when_far_away() -> bool {
    false
}

pub fn allay_leash_offset(eye_height: f32, width: f32) -> (f32, f32, f32) {
    (0.0, eye_height * 0.6, width * 0.1)
}

pub fn allay_throw_sound_can_play(game_time: i64, random_under_point_nine: bool) -> bool {
    game_time % 7 == 0 && random_under_point_nine
}

