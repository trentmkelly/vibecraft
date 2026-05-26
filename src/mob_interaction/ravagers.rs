use super::*;

pub fn ravager_entity_type_surface() -> RavagerEntityTypeSurface {
    RavagerEntityTypeSurface {
        width: RAVAGER_WIDTH,
        height: RAVAGER_HEIGHT,
        passenger_attachment_y: RAVAGER_PASSENGER_ATTACHMENT_Y,
        passenger_attachment_z: RAVAGER_PASSENGER_ATTACHMENT_Z,
        client_tracking_range: RAVAGER_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn ravager_target_selector_matches(entity_type: &'static str, is_baby: bool) -> bool {
    matches!(entity_type, "minecraft:player" | "minecraft:iron_golem")
        || (entity_type == "minecraft:villager" && !is_baby)
}

pub fn ravager_control_flags_enabled(
    controlling_passenger_is_mob: bool,
    controlling_passenger_is_raider: bool,
    vehicle_is_boat: bool,
) -> (bool, bool, bool, bool) {
    let no_controller = !controlling_passenger_is_mob || controlling_passenger_is_raider;
    (
        no_controller,
        no_controller && !vehicle_is_boat,
        no_controller,
        no_controller,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerAiStepInput {
    pub base_movement_speed: f32,
    pub has_target: bool,
    pub immobile: bool,
    pub attack_tick: i32,
    pub stunned_tick: i32,
    pub roar_tick: i32,
    pub horizontal_collision: bool,
    pub mob_griefing: bool,
    pub destroyed_leaves: bool,
    pub on_ground: bool,
}

pub fn ravager_ai_step(input: RavagerAiStepInput) -> RavagerAiStep {
    let movement_speed = if input.immobile {
        0.0
    } else {
        let target_speed = if input.has_target {
            RAVAGER_ATTACK_MOVEMENT_SPEED
        } else {
            RAVAGER_BASE_MOVEMENT_SPEED
        };
        input.base_movement_speed + (target_speed - input.base_movement_speed) * 0.1
    };

    let mut next_roar = input.roar_tick;
    let mut roar_now = false;
    if next_roar > 0 {
        next_roar -= 1;
        roar_now = next_roar == RAVAGER_ROAR_DAMAGE_TICK;
    }

    let next_attack = (input.attack_tick - 1).max(0);
    let mut next_stun = input.stunned_tick;
    let mut start_roar_sound = false;
    if next_stun > 0 {
        next_stun -= 1;
        if next_stun == 0 {
            start_roar_sound = true;
            next_roar = RAVAGER_ROAR_WINDUP_TICKS;
        }
    }

    RavagerAiStep {
        movement_speed,
        attack_tick: next_attack,
        stunned_tick: next_stun,
        roar_tick: next_roar,
        roar_now,
        start_roar_sound,
        should_jump_after_leaf_collision: input.horizontal_collision
            && input.mob_griefing
            && !input.destroyed_leaves
            && input.on_ground,
    }
}

pub fn ravager_is_immobile(
    super_immobile: bool,
    attack_tick: i32,
    stunned_tick: i32,
    roar_tick: i32,
) -> bool {
    super_immobile || attack_tick > 0 || stunned_tick > 0 || roar_tick > 0
}

pub fn ravager_has_line_of_sight_allowed(
    stunned_tick: i32,
    roar_tick: i32,
    super_has_line_of_sight: bool,
) -> bool {
    stunned_tick <= 0 && roar_tick <= 0 && super_has_line_of_sight
}

pub fn ravager_blocked_by_item(roar_tick: i32, stun_roll_under_half: bool) -> RavagerBlockedByItem {
    if roar_tick != 0 {
        return RavagerBlockedByItem {
            stunned_tick: 0,
            roar_tick,
            stun_event: None,
            strong_knockback: false,
            defender_hurt_marked: false,
        };
    }
    RavagerBlockedByItem {
        stunned_tick: if stun_roll_under_half {
            RAVAGER_STUN_DURATION
        } else {
            0
        },
        roar_tick: 0,
        stun_event: stun_roll_under_half.then_some(RAVAGER_STUN_EVENT_ID),
        strong_knockback: !stun_roll_under_half,
        defender_hurt_marked: true,
    }
}

pub fn ravager_roar_effect(
    target_entity_type: &'static str,
    target_alive: bool,
    mob_griefing: bool,
) -> Option<RavagerRoarEffect> {
    if !target_alive || target_entity_type == "minecraft:ravager" {
        return None;
    }
    if !mob_griefing && target_entity_type == "minecraft:armor_stand" {
        return None;
    }
    let target_is_illager = matches!(
        target_entity_type,
        "minecraft:evoker" | "minecraft:illusioner" | "minecraft:pillager" | "minecraft:vindicator"
    );
    Some(RavagerRoarEffect {
        damage: (!target_is_illager).then_some(RAVAGER_ROAR_DAMAGE),
        strong_knockback: target_entity_type != "minecraft:player",
        include_armor_stand: target_entity_type == "minecraft:armor_stand" && mob_griefing,
        event: Some(RAVAGER_ROAR_EVENT_ID),
    })
}

pub fn ravager_do_hurt_target_event() -> (i32, u8) {
    (RAVAGER_ATTACK_DURATION, RAVAGER_ATTACK_EVENT_ID)
}

pub fn ravager_can_spawn_without_obstruction(contains_liquid_in_bounding_box: bool) -> bool {
    !contains_liquid_in_bounding_box
}

pub fn ravager_can_be_raid_leader() -> bool {
    false
}
