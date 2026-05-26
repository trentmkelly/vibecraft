use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolarBearState {
    pub standing: bool,
    pub warning_sound_ticks: i32,
    pub client_stand_animation: f32,
    pub client_stand_animation_old: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarBearMeleeSignal {
    AttackAndStopStanding,
    StandAndWarn,
    StopStanding,
}

pub const POLAR_BEAR_MAX_HEALTH: f32 = 30.0;
pub const POLAR_BEAR_FOLLOW_RANGE: f64 = 20.0;
pub const POLAR_BEAR_PLAYER_ATTACK_FOLLOW_DISTANCE_FACTOR: f64 = 0.5;
pub const POLAR_BEAR_MOVEMENT_SPEED: f64 = 0.25;
pub const POLAR_BEAR_ATTACK_DAMAGE: f32 = 6.0;
pub const POLAR_BEAR_MELEE_SPEED: f32 = 1.25;
pub const POLAR_BEAR_PANIC_SPEED: f32 = 2.0;
pub const POLAR_BEAR_FOLLOW_PARENT_SPEED: f32 = 1.25;
pub const POLAR_BEAR_RANDOM_STROLL_SPEED: f32 = 1.0;
pub const POLAR_BEAR_LOOK_AT_PLAYER_DISTANCE: f32 = 6.0;
pub const POLAR_BEAR_STAND_ANIMATION_TICKS: f32 = 6.0;
pub const POLAR_BEAR_WARNING_SOUND_COOLDOWN_TICKS: i32 = 40;
pub const POLAR_BEAR_WARNING_ATTACK_TICKS: i32 = 10;
pub const POLAR_BEAR_CUB_ALERT_XZ_RANGE: f64 = 8.0;
pub const POLAR_BEAR_CUB_ALERT_Y_RANGE: f64 = 4.0;
pub const POLAR_BEAR_PERSISTENT_ANGER_MIN_TICKS: i32 = 20 * 20;
pub const POLAR_BEAR_PERSISTENT_ANGER_MAX_TICKS: i32 = 39 * 20;
pub const POLAR_BEAR_WATER_SLOWDOWN: f32 = 0.98;

impl PolarBearState {
    pub fn new() -> Self {
        Self {
            standing: false,
            warning_sound_ticks: 0,
            client_stand_animation: 0.0,
            client_stand_animation_old: 0.0,
        }
    }

    pub fn play_warning_sound(&mut self) -> bool {
        if self.warning_sound_ticks <= 0 {
            self.warning_sound_ticks = POLAR_BEAR_WARNING_SOUND_COOLDOWN_TICKS;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, client_side: bool) {
        if client_side {
            self.client_stand_animation_old = self.client_stand_animation;
            if self.standing {
                self.client_stand_animation = (self.client_stand_animation + 1.0)
                    .clamp(0.0, POLAR_BEAR_STAND_ANIMATION_TICKS);
            } else {
                self.client_stand_animation = (self.client_stand_animation - 1.0)
                    .clamp(0.0, POLAR_BEAR_STAND_ANIMATION_TICKS);
            }
        }
        if self.warning_sound_ticks > 0 {
            self.warning_sound_ticks -= 1;
        }
    }

    pub fn standing_animation_scale(self, partial_tick: f32) -> f32 {
        lerp(
            partial_tick,
            self.client_stand_animation_old,
            self.client_stand_animation,
        ) / POLAR_BEAR_STAND_ANIMATION_TICKS
    }

    pub fn dimensions_height_scale(self) -> f32 {
        if self.client_stand_animation > 0.0 {
            1.0 + self.client_stand_animation / POLAR_BEAR_STAND_ANIMATION_TICKS
        } else {
            1.0
        }
    }
}

pub fn polar_bear_should_attack_player(
    bear_is_baby: bool,
    nearest_target_goal_can_use: bool,
    baby_bear_nearby: bool,
) -> bool {
    !bear_is_baby && nearest_target_goal_can_use && baby_bear_nearby
}

pub fn polar_bear_should_attack_fox(bear_is_baby: bool) -> bool {
    !bear_is_baby
}

pub fn polar_bear_should_alert_other_on_hurt(
    other_is_polar_bear: bool,
    other_is_baby: bool,
) -> bool {
    other_is_polar_bear && !other_is_baby
}

pub fn polar_bear_melee_signal(
    can_perform_attack: bool,
    distance_to_target_squared: f64,
    target_width: f32,
    ticks_until_next_attack: i32,
    time_to_attack: bool,
) -> PolarBearMeleeSignal {
    if can_perform_attack {
        PolarBearMeleeSignal::AttackAndStopStanding
    } else {
        let warning_distance = (target_width as f64 + 3.0) * (target_width as f64 + 3.0);
        if distance_to_target_squared < warning_distance {
            if ticks_until_next_attack <= POLAR_BEAR_WARNING_ATTACK_TICKS {
                PolarBearMeleeSignal::StandAndWarn
            } else {
                let _ = time_to_attack;
                PolarBearMeleeSignal::StopStanding
            }
        } else {
            PolarBearMeleeSignal::StopStanding
        }
    }
}
