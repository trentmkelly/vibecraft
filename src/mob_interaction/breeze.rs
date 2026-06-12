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
pub struct BreezeLongJumpCanRunInput {
    pub on_ground: bool,
    pub in_water: bool,
    pub should_swim: bool,
    pub jump_target_present: bool,
    pub attack_target_present: bool,
    pub out_of_aggro_range: bool,
    pub too_close_for_jump: bool,
    pub can_jump_from_current_position: bool,
    pub snapped_target: Option<(i32, i32, i32)>,
    pub target_below_dangerous: bool,
    pub line_of_sight_to_target_center: bool,
    pub line_of_sight_to_target_above_four: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeLongJumpCanRunStep {
    pub can_run: bool,
    pub erase_attack_target: bool,
    pub set_jump_target: Option<(i32, i32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeLongJumpStartStep {
    pub inhaling_memory_expiry_ticks: Option<i64>,
    pub pose: &'static str,
    pub sound: (&'static str, &'static str, f32, f32),
    pub look_at_jump_target: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeLongJumpTickInput {
    pub pose: &'static str,
    pub in_water: bool,
    pub on_ground: bool,
    pub leaving_water_memory_present: bool,
    pub inhaling_memory_present: bool,
    pub optimal_jump_vector: Option<BreezeVec3>,
    pub hurt_by_memory_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeLongJumpTickStep {
    pub erase_leaving_water_memory: bool,
    pub set_leaving_water_memory: bool,
    pub pose: Option<&'static str>,
    pub sound: Option<(&'static str, f32, f32)>,
    pub discard_friction: Option<bool>,
    pub delta_movement: Option<BreezeVec3>,
    pub y_rot_from_body: bool,
    pub jump_cooldown_expiry_ticks: Option<i64>,
    pub shoot_memory_expiry_ticks: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeLongJumpStopStep {
    pub pose: Option<&'static str>,
    pub erase_jump_target: bool,
    pub erase_inhaling: bool,
    pub erase_leaving_water: bool,
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
pub const BREEZE_LONG_JUMP_REQUIRED_AIR_BLOCKS_ABOVE: i32 = 4;
pub const BREEZE_LONG_JUMP_COOLDOWN_TICKS: i64 = 10;
pub const BREEZE_LONG_JUMP_COOLDOWN_WHEN_HURT_TICKS: i64 = 2;
pub const BREEZE_LONG_JUMP_INHALING_DURATION_TICKS: i64 = 10;
pub const BREEZE_LONG_JUMP_DEFAULT_FOLLOW_RANGE: f32 = 24.0;
pub const BREEZE_LONG_JUMP_DEFAULT_MAX_JUMP_VELOCITY: f32 = 1.4;
pub const BREEZE_LONG_JUMP_MAX_VELOCITY_MULTIPLIER: f32 = 0.058333334;
pub const BREEZE_LONG_JUMP_BEHAVIOR_DURATION_TICKS: i64 = 200;
pub const BREEZE_LONG_JUMP_SNAP_TRACE_DISTANCE: f64 = 10.0;
pub const BREEZE_LONG_JUMP_MIN_ATTACK_TARGET_DISTANCE: f64 = 4.0;
pub const BREEZE_LONG_JUMP_SHOOT_MEMORY_EXPIRY_TICKS: i64 = 100;
pub const BREEZE_LONG_JUMP_CHARGE_SOUND: (&str, &str, f32, f32) =
    ("minecraft:entity.breeze.charge", "hostile", 1.0, 1.0);
pub const BREEZE_LONG_JUMP_JUMP_SOUND: (&str, f32, f32) =
    ("minecraft:entity.breeze.jump", 1.0, 1.0);
pub const BREEZE_LONG_JUMP_LAND_SOUND: (&str, f32, f32) =
    ("minecraft:entity.breeze.land", 1.0, 1.0);
pub const BREEZE_LONG_JUMP_ALLOWED_ANGLES: [i32; 5] = [40, 55, 60, 75, 80];
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

pub const BREEZE_LONG_JUMP_MEMORY_REQUIREMENTS: [(&str, &str); 7] = [
    ("attack_target", "value_present"),
    ("breeze_jump_cooldown", "value_absent"),
    ("breeze_jump_inhaling", "registered"),
    ("breeze_jump_target", "registered"),
    ("breeze_shoot", "value_absent"),
    ("walk_target", "value_absent"),
    ("breeze_leaving_water", "registered"),
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

pub fn breeze_long_jump_can_run(input: BreezeLongJumpCanRunInput) -> BreezeLongJumpCanRunStep {
    if !input.on_ground && !input.in_water {
        return BreezeLongJumpCanRunStep::blocked();
    }
    if input.should_swim {
        return BreezeLongJumpCanRunStep::blocked();
    }
    if input.jump_target_present {
        return BreezeLongJumpCanRunStep {
            can_run: true,
            erase_attack_target: false,
            set_jump_target: None,
        };
    }
    if !input.attack_target_present {
        return BreezeLongJumpCanRunStep::blocked();
    }
    if input.out_of_aggro_range {
        return BreezeLongJumpCanRunStep {
            can_run: false,
            erase_attack_target: true,
            set_jump_target: None,
        };
    }
    if input.too_close_for_jump
        || !input.can_jump_from_current_position
        || input.target_below_dangerous
    {
        return BreezeLongJumpCanRunStep::blocked();
    }
    let Some(target) = input.snapped_target else {
        return BreezeLongJumpCanRunStep::blocked();
    };
    if !input.line_of_sight_to_target_center && !input.line_of_sight_to_target_above_four {
        return BreezeLongJumpCanRunStep::blocked();
    }
    BreezeLongJumpCanRunStep {
        can_run: true,
        erase_attack_target: false,
        set_jump_target: Some(target),
    }
}

pub fn breeze_long_jump_out_of_aggro_range(distance: f64, follow_range: f64) -> bool {
    distance >= follow_range
}

pub fn breeze_long_jump_too_close_for_jump(distance: f64) -> bool {
    distance - BREEZE_LONG_JUMP_MIN_ATTACK_TARGET_DISTANCE <= 0.0
}

pub fn breeze_long_jump_can_jump_from_current_position(
    standing_on_honey: bool,
    four_blocks_above_air_or_water: [bool; 4],
) -> bool {
    !standing_on_honey && four_blocks_above_air_or_water.into_iter().all(|open| open)
}

pub fn breeze_long_jump_max_jump_velocity(follow_range: f32) -> f32 {
    BREEZE_LONG_JUMP_MAX_VELOCITY_MULTIPLIER * follow_range
}

pub fn breeze_long_jump_select_vector(
    shuffled_angle_candidates: &[(i32, Option<BreezeVec3>)],
    jump_boost_power: Option<f64>,
) -> Option<BreezeVec3> {
    shuffled_angle_candidates
        .iter()
        .find_map(|(_, vector)| *vector)
        .map(|vector| {
            if let Some(jump_boost_power) = jump_boost_power {
                vector.add(BreezeVec3::new(
                    0.0,
                    vector.normalize().y * jump_boost_power,
                    0.0,
                ))
            } else {
                vector
            }
        })
}

pub fn breeze_long_jump_start(
    inhaling_memory_absent: bool,
    jump_target_present: bool,
) -> BreezeLongJumpStartStep {
    BreezeLongJumpStartStep {
        inhaling_memory_expiry_ticks: inhaling_memory_absent
            .then_some(BREEZE_LONG_JUMP_INHALING_DURATION_TICKS),
        pose: "inhaling",
        sound: BREEZE_LONG_JUMP_CHARGE_SOUND,
        look_at_jump_target: jump_target_present,
    }
}

pub fn breeze_long_jump_can_still_use(pose: &str, jump_cooldown_present: bool) -> bool {
    pose != "standing" && !jump_cooldown_present
}

pub fn breeze_long_jump_tick(input: BreezeLongJumpTickInput) -> BreezeLongJumpTickStep {
    let erase_leaving_water_memory = !input.in_water && input.leaving_water_memory_present;
    if input.pose == "inhaling" && !input.inhaling_memory_present {
        let Some(velocity) = input.optimal_jump_vector else {
            return BreezeLongJumpTickStep {
                erase_leaving_water_memory,
                pose: Some("standing"),
                ..BreezeLongJumpTickStep::empty()
            };
        };
        return BreezeLongJumpTickStep {
            erase_leaving_water_memory,
            set_leaving_water_memory: input.in_water,
            pose: Some("long_jumping"),
            sound: Some(BREEZE_LONG_JUMP_JUMP_SOUND),
            discard_friction: Some(true),
            delta_movement: Some(velocity),
            y_rot_from_body: true,
            jump_cooldown_expiry_ticks: None,
            shoot_memory_expiry_ticks: None,
        };
    }

    let finished_jumping = input.pose == "long_jumping"
        && (input.on_ground || (input.in_water && !input.leaving_water_memory_present));
    if finished_jumping {
        return BreezeLongJumpTickStep {
            erase_leaving_water_memory,
            pose: Some("standing"),
            sound: Some(BREEZE_LONG_JUMP_LAND_SOUND),
            discard_friction: Some(false),
            delta_movement: None,
            y_rot_from_body: false,
            set_leaving_water_memory: false,
            jump_cooldown_expiry_ticks: Some(if input.hurt_by_memory_present {
                BREEZE_LONG_JUMP_COOLDOWN_WHEN_HURT_TICKS
            } else {
                BREEZE_LONG_JUMP_COOLDOWN_TICKS
            }),
            shoot_memory_expiry_ticks: Some(BREEZE_LONG_JUMP_SHOOT_MEMORY_EXPIRY_TICKS),
        };
    }

    BreezeLongJumpTickStep {
        erase_leaving_water_memory,
        ..BreezeLongJumpTickStep::empty()
    }
}

pub fn breeze_long_jump_stop(pose: &str) -> BreezeLongJumpStopStep {
    BreezeLongJumpStopStep {
        pose: matches!(pose, "long_jumping" | "inhaling").then_some("standing"),
        erase_jump_target: true,
        erase_inhaling: true,
        erase_leaving_water: true,
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

impl BreezeLongJumpCanRunStep {
    fn blocked() -> Self {
        Self {
            can_run: false,
            erase_attack_target: false,
            set_jump_target: None,
        }
    }
}

impl BreezeLongJumpTickStep {
    fn empty() -> Self {
        Self {
            erase_leaving_water_memory: false,
            set_leaving_water_memory: false,
            pose: None,
            sound: None,
            discard_friction: None,
            delta_movement: None,
            y_rot_from_body: false,
            jump_cooldown_expiry_ticks: None,
            shoot_memory_expiry_ticks: None,
        }
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
