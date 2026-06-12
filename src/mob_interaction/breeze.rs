#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeShootWhenStuckStep {
    pub can_start: bool,
    pub can_still_use: bool,
    pub shoot_memory_expiry_ticks: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeShootStartCheck {
    pub can_start: bool,
    pub erase_shoot_memory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeShootStartStep {
    pub pose: Option<&'static str>,
    pub charging_memory_expiry_ticks: i64,
    pub sound: (&'static str, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeShootStopStep {
    pub pose: Option<&'static str>,
    pub cooldown_memory_expiry_ticks: i64,
    pub erase_shoot_memory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeShootTickInput {
    pub breeze_position: BreezeVec3,
    pub breeze_firing_y: f64,
    pub target_position: BreezeVec3,
    pub target_height: f64,
    pub target_passenger: bool,
    pub target_present: bool,
    pub charging_memory_present: bool,
    pub recovering_memory_present: bool,
    pub difficulty_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeShootProjectile {
    pub kind: &'static str,
    pub direction: BreezeVec3,
    pub movement_scale: f32,
    pub uncertainty: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeShootTickStep {
    pub look_at_target_eyes: bool,
    pub recovering_memory_expiry_ticks: Option<i64>,
    pub projectile: Option<BreezeShootProjectile>,
    pub sound: Option<(&'static str, f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeSlideStartInput {
    pub breeze_position: BreezeVec3,
    pub enemy_position: BreezeVec3,
    pub within_inner_ring: bool,
    pub away_candidate: Option<BreezeVec3>,
    pub away_candidate_has_line_of_sight: bool,
    pub random_next_boolean: bool,
    pub behind_target_head_y_rot_degrees: f64,
    pub behind_target_gaussian: f64,
    pub behind_target_random_float: f64,
    pub middle_circle_random_double: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeSlideWalkTarget {
    pub position: BreezeVec3,
    pub block_pos: (i32, i32, i32),
    pub speed_modifier: f32,
    pub close_enough_dist: i32,
}

pub const BREEZE_UTIL_MAX_LINE_OF_SIGHT_TEST_RANGE: f64 = 50.0;
pub const BREEZE_UTIL_BEHIND_TARGET_BASE_DEGREES: f64 = 180.0;
pub const BREEZE_UTIL_BEHIND_TARGET_SPREAD_DEGREES: f64 = 90.0;
pub const BREEZE_UTIL_BEHIND_TARGET_MIN_DISTANCE: f64 = 4.0;
pub const BREEZE_UTIL_BEHIND_TARGET_MAX_DISTANCE: f64 = 8.0;
pub const BREEZE_SHOOT_ATTACK_RANGE_MAX_SQR: f64 = 256.0;
pub const BREEZE_SHOOT_UNCERTAINTY_BASE: i32 = 5;
pub const BREEZE_SHOOT_UNCERTAINTY_MULTIPLIER: i32 = 4;
pub const BREEZE_SHOOT_PROJECTILE_MOVEMENT_SCALE: f32 = 0.7;
pub const BREEZE_SHOOT_INITIAL_DELAY_TICKS: i64 = 15;
pub const BREEZE_SHOOT_RECOVER_DELAY_TICKS: i64 = 4;
pub const BREEZE_SHOOT_COOLDOWN_TICKS: i64 = 10;
pub const BREEZE_SHOOT_BEHAVIOR_DURATION_TICKS: i64 =
    BREEZE_SHOOT_INITIAL_DELAY_TICKS + 1 + BREEZE_SHOOT_RECOVER_DELAY_TICKS;
pub const BREEZE_SHOOT_INHALE_SOUND: (&str, f32, f32) =
    ("minecraft:entity.breeze.inhale", 1.0, 1.0);
pub const BREEZE_SHOOT_SOUND: (&str, f32, f32) = ("minecraft:entity.breeze.shoot", 1.5, 1.0);
pub const BREEZE_SHOOT_PROJECTILE_KIND: &str = "minecraft:breeze_wind_charge";
pub const BREEZE_SHOOT_WHEN_STUCK_MEMORY_EXPIRY_TICKS: i64 = 60;
pub const BREEZE_SLIDE_AWAY_HORIZONTAL_RANGE: i32 = 5;
pub const BREEZE_SLIDE_AWAY_VERTICAL_RANGE: i32 = 5;
pub const BREEZE_SLIDE_MIDDLE_MIN_DISTANCE: f64 = 4.0;
pub const BREEZE_SLIDE_MIDDLE_MAX_DISTANCE: f64 = 8.0;
pub const BREEZE_SLIDE_WALK_TARGET_SPEED: f32 = 0.6;
pub const BREEZE_SLIDE_WALK_TARGET_CLOSE_ENOUGH_DIST: i32 = 1;

pub const BREEZE_SHOOT_WHEN_STUCK_MEMORY_REQUIREMENTS: [(&str, &str); 5] = [
    ("attack_target", "value_present"),
    ("breeze_jump_inhaling", "value_absent"),
    ("breeze_jump_target", "value_absent"),
    ("walk_target", "value_absent"),
    ("breeze_shoot", "value_absent"),
];

pub const BREEZE_SHOOT_MEMORY_REQUIREMENTS: [(&str, &str); 7] = [
    ("attack_target", "value_present"),
    ("breeze_shoot_cooldown", "value_absent"),
    ("breeze_shoot_charging", "value_absent"),
    ("breeze_shoot_recovering", "value_absent"),
    ("breeze_shoot", "value_present"),
    ("walk_target", "value_absent"),
    ("breeze_jump_target", "value_absent"),
];

pub const BREEZE_SLIDE_MEMORY_REQUIREMENTS: [(&str, &str); 4] = [
    ("attack_target", "value_present"),
    ("walk_target", "value_absent"),
    ("breeze_jump_cooldown", "value_absent"),
    ("breeze_shoot", "value_absent"),
];

impl BreezeVec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn distance_to(self, other: Self) -> f64 {
        self.distance_to_sqr(other).sqrt()
    }

    pub fn distance_to_sqr(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn subtract(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn length(self) -> f64 {
        self.distance_to_sqr(Self::new(0.0, 0.0, 0.0)).sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = self.length();
        if length < 1.0e-12 {
            Self::new(0.0, 0.0, 0.0)
        } else {
            self.scale(1.0 / length)
        }
    }
}

/// Java `Vec3.directionFromRotation(0.0F, yaw).scale(r)` as used by
/// `BreezeUtil.randomPointBehindTarget`.
pub fn breeze_random_point_behind_target(
    target_position: BreezeVec3,
    target_head_y_rot_degrees: f64,
    gaussian: f64,
    random_float: f64,
) -> BreezeVec3 {
    let view_angle = target_head_y_rot_degrees
        + BREEZE_UTIL_BEHIND_TARGET_BASE_DEGREES
        + gaussian * BREEZE_UTIL_BEHIND_TARGET_SPREAD_DEGREES / 2.0;
    let radius = lerp(
        random_float,
        BREEZE_UTIL_BEHIND_TARGET_MIN_DISTANCE,
        BREEZE_UTIL_BEHIND_TARGET_MAX_DISTANCE,
    );
    target_position.add(direction_from_rotation(0.0, view_angle).scale(radius))
}

pub fn breeze_has_line_of_sight(
    breeze_position: BreezeVec3,
    target_position: BreezeVec3,
    follow_range: f64,
    clip_misses: bool,
) -> bool {
    if target_position.distance_to(breeze_position)
        > BREEZE_UTIL_MAX_LINE_OF_SIGHT_TEST_RANGE.max(follow_range)
    {
        return false;
    }
    clip_misses
}

pub fn breeze_shoot_when_stuck_step(
    passenger: bool,
    in_water: bool,
    has_levitation: bool,
) -> BreezeShootWhenStuckStep {
    let can_start = passenger || in_water || has_levitation;
    BreezeShootWhenStuckStep {
        can_start,
        can_still_use: false,
        shoot_memory_expiry_ticks: can_start.then_some(BREEZE_SHOOT_WHEN_STUCK_MEMORY_EXPIRY_TICKS),
    }
}

pub fn breeze_shoot_check_start(
    pose: &str,
    target_present: bool,
    target_distance_sqr: f64,
) -> BreezeShootStartCheck {
    if pose != "standing" || !target_present {
        return BreezeShootStartCheck {
            can_start: false,
            erase_shoot_memory: false,
        };
    }
    let can_start = target_distance_sqr < BREEZE_SHOOT_ATTACK_RANGE_MAX_SQR;
    BreezeShootStartCheck {
        can_start,
        erase_shoot_memory: !can_start,
    }
}

pub fn breeze_shoot_can_still_use(attack_target_present: bool, shoot_memory_present: bool) -> bool {
    attack_target_present && shoot_memory_present
}

pub fn breeze_shoot_start(target_present: bool) -> BreezeShootStartStep {
    BreezeShootStartStep {
        pose: target_present.then_some("shooting"),
        charging_memory_expiry_ticks: BREEZE_SHOOT_INITIAL_DELAY_TICKS,
        sound: BREEZE_SHOOT_INHALE_SOUND,
    }
}

pub fn breeze_shoot_stop(current_pose: &str) -> BreezeShootStopStep {
    BreezeShootStopStep {
        pose: (current_pose == "shooting").then_some("standing"),
        cooldown_memory_expiry_ticks: BREEZE_SHOOT_COOLDOWN_TICKS,
        erase_shoot_memory: true,
    }
}

pub fn breeze_shoot_tick(input: BreezeShootTickInput) -> BreezeShootTickStep {
    if !input.target_present {
        return BreezeShootTickStep {
            look_at_target_eyes: false,
            recovering_memory_expiry_ticks: None,
            projectile: None,
            sound: None,
        };
    }
    if input.charging_memory_present || input.recovering_memory_present {
        return BreezeShootTickStep {
            look_at_target_eyes: true,
            recovering_memory_expiry_ticks: None,
            projectile: None,
            sound: None,
        };
    }

    let target_y_scale = if input.target_passenger { 0.8 } else { 0.3 };
    BreezeShootTickStep {
        look_at_target_eyes: true,
        recovering_memory_expiry_ticks: Some(BREEZE_SHOOT_RECOVER_DELAY_TICKS),
        projectile: Some(BreezeShootProjectile {
            kind: BREEZE_SHOOT_PROJECTILE_KIND,
            direction: BreezeVec3::new(
                input.target_position.x - input.breeze_position.x,
                input.target_position.y + input.target_height * target_y_scale
                    - input.breeze_firing_y,
                input.target_position.z - input.breeze_position.z,
            ),
            movement_scale: BREEZE_SHOOT_PROJECTILE_MOVEMENT_SCALE,
            uncertainty: BREEZE_SHOOT_UNCERTAINTY_BASE
                - input.difficulty_id * BREEZE_SHOOT_UNCERTAINTY_MULTIPLIER,
        }),
        sound: Some(BREEZE_SHOOT_SOUND),
    }
}

pub fn breeze_slide_can_start(on_ground: bool, in_water: bool, pose: &str) -> bool {
    on_ground && !in_water && pose == "standing"
}

pub fn breeze_slide_start(input: BreezeSlideStartInput) -> BreezeSlideWalkTarget {
    let position = if let Some(candidate) = input.away_candidate.filter(|candidate| {
        input.within_inner_ring
            && input.away_candidate_has_line_of_sight
            && input.enemy_position.distance_to_sqr(*candidate)
                > input.enemy_position.distance_to_sqr(input.breeze_position)
    }) {
        candidate
    } else if input.random_next_boolean {
        breeze_random_point_behind_target(
            input.enemy_position,
            input.behind_target_head_y_rot_degrees,
            input.behind_target_gaussian,
            input.behind_target_random_float,
        )
    } else {
        breeze_slide_random_point_in_middle_circle(
            input.breeze_position,
            input.enemy_position,
            input.middle_circle_random_double,
        )
    };
    BreezeSlideWalkTarget {
        position,
        block_pos: block_pos_containing(position),
        speed_modifier: BREEZE_SLIDE_WALK_TARGET_SPEED,
        close_enough_dist: BREEZE_SLIDE_WALK_TARGET_CLOSE_ENOUGH_DIST,
    }
}

pub fn breeze_slide_random_point_in_middle_circle(
    breeze_position: BreezeVec3,
    enemy_position: BreezeVec3,
    random_double: f64,
) -> BreezeVec3 {
    let direction = enemy_position.subtract(breeze_position);
    let distance = direction.length()
        - lerp(
            random_double,
            BREEZE_SLIDE_MIDDLE_MAX_DISTANCE,
            BREEZE_SLIDE_MIDDLE_MIN_DISTANCE,
        );
    breeze_position.add(direction.normalize().scale(distance))
}

fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

fn direction_from_rotation(x_rot_degrees: f64, y_rot_degrees: f64) -> BreezeVec3 {
    let yaw = -y_rot_degrees.to_radians() - std::f64::consts::PI;
    let pitch = -x_rot_degrees.to_radians();
    let cos_yaw = yaw.cos();
    let sin_yaw = yaw.sin();
    let cos_pitch = pitch.cos();
    let sin_pitch = pitch.sin();
    BreezeVec3 {
        x: sin_yaw * cos_pitch,
        y: sin_pitch,
        z: cos_yaw * cos_pitch,
    }
}

trait BreezeVec3Scale {
    fn scale(self, scale: f64) -> Self;
}

impl BreezeVec3Scale for BreezeVec3 {
    fn scale(self, scale: f64) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
            z: self.z * scale,
        }
    }
}

fn block_pos_containing(position: BreezeVec3) -> (i32, i32, i32) {
    (
        position.x.floor() as i32,
        position.y.floor() as i32,
        position.z.floor() as i32,
    )
}
