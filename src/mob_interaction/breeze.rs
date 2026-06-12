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
