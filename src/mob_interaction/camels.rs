use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamelState {
    pub dashing: bool,
    pub dash_cooldown: i32,
    pub last_pose_change_tick: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamelTickOutcome {
    pub dash_ready_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamelPassengerAttachment {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub const CAMEL_BABY_SCALE: f32 = 0.6;
pub const CAMEL_DASH_COOLDOWN_TICKS: i32 = 55;
pub const CAMEL_MAX_HEAD_Y_ROT: i32 = 30;
pub const CAMEL_RUNNING_SPEED_BONUS: f32 = 0.1;
pub const CAMEL_DASH_VERTICAL_MOMENTUM: f32 = 1.4285;
pub const CAMEL_DASH_HORIZONTAL_MOMENTUM: f32 = 22.2222;
pub const CAMEL_DASH_MINIMUM_DURATION_TICKS: i32 = 5;
pub const CAMEL_SITDOWN_DURATION_TICKS: i64 = 40;
pub const CAMEL_STANDUP_DURATION_TICKS: i64 = 52;
pub const CAMEL_IDLE_MINIMAL_DURATION_TICKS: i32 = 80;
pub const CAMEL_SITTING_HEIGHT_DIFFERENCE: f32 = 1.43;
pub const CAMEL_SITTING_EYE_HEIGHT: f32 = 0.845;
pub const CAMEL_DEFAULT_LAST_POSE_CHANGE_TICK: i64 = 0;

impl CamelState {
    pub fn new_standing(current_game_time: i64) -> Self {
        let mut state = Self {
            dashing: false,
            dash_cooldown: 0,
            last_pose_change_tick: CAMEL_DEFAULT_LAST_POSE_CHANGE_TICK,
        };
        state.reset_last_pose_change_tick_to_full_stand(current_game_time);
        state
    }

    pub fn from_saved_pose_tick(saved_pose_tick: i64) -> Self {
        Self {
            dashing: false,
            dash_cooldown: 0,
            last_pose_change_tick: saved_pose_tick,
        }
    }

    pub fn is_sitting(self) -> bool {
        self.last_pose_change_tick < 0
    }

    pub fn pose_time(self, current_game_time: i64) -> i64 {
        current_game_time - self.last_pose_change_tick.abs()
    }

    pub fn is_in_pose_transition(self, current_game_time: i64) -> bool {
        let pose_time = self.pose_time(current_game_time);
        pose_time
            < if self.is_sitting() {
                CAMEL_SITDOWN_DURATION_TICKS
            } else {
                CAMEL_STANDUP_DURATION_TICKS
            }
    }

    pub fn refuse_to_move(self, current_game_time: i64) -> bool {
        self.is_sitting() || self.is_in_pose_transition(current_game_time)
    }

    pub fn sit_down(&mut self, current_game_time: i64) -> bool {
        if self.is_sitting() {
            return false;
        }
        self.last_pose_change_tick = -current_game_time;
        true
    }

    pub fn stand_up(&mut self, current_game_time: i64) -> bool {
        if !self.is_sitting() {
            return false;
        }
        self.last_pose_change_tick = current_game_time;
        true
    }

    pub fn stand_up_instantly(&mut self, current_game_time: i64) {
        self.reset_last_pose_change_tick_to_full_stand(current_game_time);
    }

    pub fn reset_last_pose_change_tick_to_full_stand(&mut self, current_game_time: i64) {
        self.last_pose_change_tick = (current_game_time - CAMEL_STANDUP_DURATION_TICKS - 1).max(0);
    }

    pub fn can_start_dash(self, saddled: bool, on_ground: bool) -> bool {
        saddled && self.dash_cooldown <= 0 && on_ground
    }

    pub fn start_dash(&mut self) {
        self.dashing = true;
        self.dash_cooldown = CAMEL_DASH_COOLDOWN_TICKS;
    }

    pub fn tick(
        &mut self,
        on_ground: bool,
        in_liquid: bool,
        is_passenger: bool,
    ) -> CamelTickOutcome {
        if self.dashing
            && self.dash_cooldown < CAMEL_DASH_COOLDOWN_TICKS - CAMEL_DASH_MINIMUM_DURATION_TICKS
            && (on_ground || in_liquid || is_passenger)
        {
            self.dashing = false;
        }

        let mut dash_ready_sound = false;
        if self.dash_cooldown > 0 {
            self.dash_cooldown -= 1;
            dash_ready_sound = self.dash_cooldown == 0;
        }

        CamelTickOutcome { dash_ready_sound }
    }

    pub fn ridden_speed(self, base_movement_speed: f32, controller_sprinting: bool) -> f32 {
        base_movement_speed
            + if controller_sprinting && self.dash_cooldown == 0 {
                CAMEL_RUNNING_SPEED_BONUS
            } else {
                0.0
            }
    }

    pub fn can_add_passenger(passenger_count: usize) -> bool {
        passenger_count <= 2
    }
}

pub fn camel_dash_impulse(
    amount: f32,
    movement_speed: f64,
    block_speed_factor: f64,
    jump_power: f64,
) -> (f64, f64) {
    (
        CAMEL_DASH_HORIZONTAL_MOMENTUM as f64 * amount as f64 * movement_speed * block_speed_factor,
        CAMEL_DASH_VERTICAL_MOMENTUM as f64 * amount as f64 * jump_power,
    )
}

pub fn camel_passenger_attachment_point(
    passenger_index: usize,
    passenger_count: usize,
    passenger_is_animal: bool,
    camel_sitting: bool,
    removed: bool,
    dimensions_width: f32,
    dimensions_height: f32,
    scale: f32,
) -> CamelPassengerAttachment {
    let driver = passenger_index == 0;
    let mut offset = 0.5;
    let height = if removed {
        0.01
    } else {
        camel_body_anchor_y(camel_sitting, false, driver, 0.0, dimensions_height, scale)
    };
    if passenger_count > 1 {
        if !driver {
            offset = -0.7;
        }
        if passenger_is_animal {
            offset += 0.2;
        }
    }
    let _ = dimensions_width;
    CamelPassengerAttachment {
        x: 0.0,
        y: height,
        z: offset as f64 * scale as f64,
    }
}

fn camel_body_anchor_y(
    sitting: bool,
    in_pose_transition: bool,
    front: bool,
    pose_time: f32,
    dimensions_height: f32,
    scale: f32,
) -> f64 {
    let mut base_sit_offset = dimensions_height - 0.375 * scale;
    let sitting_height_difference = scale * CAMEL_SITTING_HEIGHT_DIFFERENCE;
    let vertical_drop = sitting_height_difference - scale * 0.2;
    let bottom_point = sitting_height_difference - vertical_drop;
    if in_pose_transition {
        let animation_duration = if sitting {
            CAMEL_SITDOWN_DURATION_TICKS as f32
        } else {
            CAMEL_STANDUP_DURATION_TICKS as f32
        };
        let (half_point, flex_point_offset) = if sitting {
            (28.0, if front { 0.5 } else { 0.1 })
        } else {
            (
                if front { 24.0 } else { 32.0 },
                if front { 0.6 } else { 0.35 },
            )
        };
        let pose_time = pose_time.clamp(0.0, animation_duration);
        let first_part = pose_time < half_point;
        let part = if first_part {
            pose_time / half_point
        } else {
            (pose_time - half_point) / (animation_duration - half_point)
        };
        let flex_point = sitting_height_difference - flex_point_offset * vertical_drop;
        base_sit_offset += if sitting {
            lerp(
                part,
                if first_part {
                    sitting_height_difference
                } else {
                    flex_point
                },
                if first_part { flex_point } else { bottom_point },
            )
        } else {
            lerp(
                part,
                if first_part {
                    bottom_point - sitting_height_difference
                } else {
                    bottom_point - flex_point
                },
                if first_part {
                    bottom_point - flex_point
                } else {
                    0.0
                },
            )
        };
    }
    if sitting && !in_pose_transition {
        base_sit_offset += bottom_point;
    }
    base_sit_offset as f64
}
