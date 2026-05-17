#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn length_sqr(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f64 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = self.length();
        if length < 1.0E-7 {
            Self::ZERO
        } else {
            scale(self, 1.0 / length)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn center(self) -> Vec3 {
        Vec3 {
            x: (self.min.x + self.max.x) * 0.5,
            y: (self.min.y + self.max.y) * 0.5,
            z: (self.min.z + self.max.z) * 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityContact {
    pub first_center: Vec3,
    pub second_center: Vec3,
    pub first_pushable: bool,
    pub second_pushable: bool,
    pub team_collision_allows: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PushImpulse {
    pub first_delta: Vec3,
    pub second_delta: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionCallback {
    StepOn,
    FallOn,
    EntityInside,
    OnLand,
    HorizontalCollision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PistonAxisState {
    pub axis: DirectionAxis,
    pub accumulated_delta: f64,
    pub game_time_matches: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionPlan {
    pub block_radius: i32,
    pub damage: f32,
    pub knockback: Vec3,
    pub fire_blocks: bool,
    pub destroy_blocks: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionBlockInteraction {
    Keep,
    Destroy,
    DestroyWithDecay,
    TriggerBlock,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    pub distance: f64,
    pub point: Vec3,
}

pub fn entity_push(contact: EntityContact) -> Option<PushImpulse> {
    if !contact.first_pushable || !contact.second_pushable || !contact.team_collision_allows {
        return None;
    }
    let dx = contact.second_center.x - contact.first_center.x;
    let dz = contact.second_center.z - contact.first_center.z;
    let distance_sqr = (dx * dx + dz * dz).max(0.01);
    let distance = distance_sqr.sqrt();
    let strength = (1.0 / distance).min(1.0) * 0.05;
    let push = Vec3 {
        x: dx / distance * strength,
        y: 0.0,
        z: dz / distance * strength,
    };
    Some(PushImpulse {
        first_delta: scale(push, -1.0),
        second_delta: push,
    })
}

pub fn cramming_damage(
    entity_count: usize,
    max_entity_cramming: usize,
    gamerule_enabled: bool,
) -> f32 {
    if gamerule_enabled && max_entity_cramming > 0 && entity_count > max_entity_cramming {
        6.0
    } else {
        0.0
    }
}

pub fn collision_callbacks(
    on_ground: bool,
    falling_distance: f32,
    horizontal_collision: bool,
    inside_block: bool,
) -> Vec<CollisionCallback> {
    let mut callbacks = Vec::new();
    if on_ground {
        callbacks.push(CollisionCallback::StepOn);
        if falling_distance > 0.0 {
            callbacks.push(CollisionCallback::FallOn);
            callbacks.push(CollisionCallback::OnLand);
        }
    }
    if horizontal_collision {
        callbacks.push(CollisionCallback::HorizontalCollision);
    }
    if inside_block {
        callbacks.push(CollisionCallback::EntityInside);
    }
    callbacks
}

pub fn fluid_push(flow: Vec3, affected_by_fluids: bool, can_stand_on_fluid: bool) -> Vec3 {
    if !affected_by_fluids || can_stand_on_fluid {
        Vec3::ZERO
    } else {
        scale(flow.normalize(), 0.014)
    }
}

pub fn piston_move_delta(state: PistonAxisState, requested: f64) -> (f64, f64) {
    let accumulated = if state.game_time_matches {
        state.accumulated_delta
    } else {
        0.0
    };
    let clamped = (requested + accumulated).clamp(-0.51, 0.51);
    (clamped - accumulated, clamped)
}

pub fn explosion_plan(
    center: Vec3,
    entity_eye: Vec3,
    power: f32,
    exposure: f32,
    interaction: ExplosionBlockInteraction,
    causes_fire: bool,
) -> ExplosionPlan {
    let radius = (power * 2.0).ceil() as i32;
    let distance_fraction = (sub(entity_eye, center).length() / f64::from(power * 2.0)).min(1.0);
    let impact = ((1.0 - distance_fraction) as f32 * exposure).max(0.0);
    let damage = ((impact * impact + impact) / 2.0 * 7.0 * power * 2.0 + 1.0).floor();
    ExplosionPlan {
        block_radius: radius,
        damage,
        knockback: scale(sub(entity_eye, center).normalize(), f64::from(impact)),
        fire_blocks: causes_fire,
        destroy_blocks: matches!(
            interaction,
            ExplosionBlockInteraction::Destroy
                | ExplosionBlockInteraction::DestroyWithDecay
                | ExplosionBlockInteraction::TriggerBlock
        ),
    }
}

pub fn ray_intersects_aabb(from: Vec3, to: Vec3, aabb: Aabb) -> Option<RayHit> {
    let direction = sub(to, from);
    let mut t_min: f64 = 0.0;
    let mut t_max: f64 = 1.0;
    for (origin, delta, min, max) in [
        (from.x, direction.x, aabb.min.x, aabb.max.x),
        (from.y, direction.y, aabb.min.y, aabb.max.y),
        (from.z, direction.z, aabb.min.z, aabb.max.z),
    ] {
        if delta.abs() < 1.0E-7 {
            if origin < min || origin > max {
                return None;
            }
        } else {
            let inv = 1.0 / delta;
            let mut near = (min - origin) * inv;
            let mut far = (max - origin) * inv;
            if near > far {
                std::mem::swap(&mut near, &mut far);
            }
            t_min = t_min.max(near);
            t_max = t_max.min(far);
            if t_min > t_max {
                return None;
            }
        }
    }
    Some(RayHit {
        distance: t_min,
        point: Vec3 {
            x: from.x + direction.x * t_min,
            y: from.y + direction.y * t_min,
            z: from.z + direction.z * t_min,
        },
    })
}

fn sub(first: Vec3, second: Vec3) -> Vec3 {
    Vec3 {
        x: first.x - second.x,
        y: first.y - second.y,
        z: first.z - second.z,
    }
}

fn scale(vec: Vec3, scalar: f64) -> Vec3 {
    Vec3 {
        x: vec.x * scalar,
        y: vec.y * scalar,
        z: vec.z * scalar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_push_and_cramming_follow_vanilla_thresholds() {
        let push = entity_push(EntityContact {
            first_center: Vec3::ZERO,
            second_center: Vec3 {
                x: 2.0,
                y: 0.0,
                z: 0.0,
            },
            first_pushable: true,
            second_pushable: true,
            team_collision_allows: true,
        })
        .unwrap();
        assert_eq!(push.first_delta.x, -0.025);
        assert_eq!(push.second_delta.x, 0.025);
        assert!(entity_push(EntityContact {
            first_pushable: false,
            second_pushable: true,
            team_collision_allows: true,
            first_center: Vec3::ZERO,
            second_center: Vec3::ZERO,
        })
        .is_none());
        assert_eq!(cramming_damage(25, 24, true), 6.0);
        assert_eq!(cramming_damage(24, 24, true), 0.0);
        assert_eq!(cramming_damage(100, 24, false), 0.0);
    }

    #[test]
    fn collision_callbacks_and_fluid_push_capture_entity_move_side_effects() {
        assert_eq!(
            collision_callbacks(true, 3.0, true, true),
            vec![
                CollisionCallback::StepOn,
                CollisionCallback::FallOn,
                CollisionCallback::OnLand,
                CollisionCallback::HorizontalCollision,
                CollisionCallback::EntityInside,
            ]
        );
        assert_eq!(
            fluid_push(
                Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 0.0,
                },
                true,
                false
            ),
            Vec3 {
                x: 0.014,
                y: 0.0,
                z: 0.0,
            }
        );
        assert_eq!(
            fluid_push(
                Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 0.0,
                },
                false,
                false
            ),
            Vec3::ZERO
        );
    }

    #[test]
    fn piston_delta_clamps_per_axis_and_resets_each_game_time() {
        assert_eq!(
            piston_move_delta(
                PistonAxisState {
                    axis: DirectionAxis::X,
                    accumulated_delta: 0.3,
                    game_time_matches: true,
                },
                0.4,
            ),
            (0.21000000000000002, 0.51)
        );
        assert_eq!(
            piston_move_delta(
                PistonAxisState {
                    axis: DirectionAxis::Y,
                    accumulated_delta: 0.3,
                    game_time_matches: false,
                },
                -0.7,
            ),
            (-0.51, -0.51)
        );
        assert_eq!(
            piston_move_delta(
                PistonAxisState {
                    axis: DirectionAxis::Z,
                    accumulated_delta: -0.2,
                    game_time_matches: true,
                },
                0.1,
            ),
            (0.1, -0.1)
        );
    }

    #[test]
    fn explosion_plan_matches_damage_knockback_and_block_interaction_surface() {
        let plan = explosion_plan(
            Vec3::ZERO,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 2.0,
            },
            4.0,
            1.0,
            ExplosionBlockInteraction::DestroyWithDecay,
            true,
        );
        assert_eq!(plan.block_radius, 8);
        assert_eq!(plan.damage, 37.0);
        assert_eq!(
            plan.knockback,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.75,
            }
        );
        assert!(plan.fire_blocks);
        assert!(plan.destroy_blocks);
        assert!(
            !explosion_plan(
                Vec3::ZERO,
                Vec3 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                },
                4.0,
                1.0,
                ExplosionBlockInteraction::Keep,
                false,
            )
            .destroy_blocks
        );
        assert!(
            explosion_plan(
                Vec3::ZERO,
                Vec3 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                },
                4.0,
                1.0,
                ExplosionBlockInteraction::Destroy,
                false,
            )
            .destroy_blocks
        );
        assert!(
            explosion_plan(
                Vec3::ZERO,
                Vec3 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                },
                4.0,
                1.0,
                ExplosionBlockInteraction::TriggerBlock,
                false,
            )
            .destroy_blocks
        );
    }

    #[test]
    fn ray_tracing_uses_slab_intersection_and_reports_first_hit() {
        let aabb = Aabb::new(
            Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Vec3 {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
        );
        assert_eq!(
            aabb.center(),
            Vec3 {
                x: 1.5,
                y: 1.5,
                z: 1.5,
            }
        );
        let hit = ray_intersects_aabb(
            Vec3 {
                x: 0.0,
                y: 1.5,
                z: 1.5,
            },
            Vec3 {
                x: 3.0,
                y: 1.5,
                z: 1.5,
            },
            aabb,
        )
        .unwrap();
        assert_eq!(hit.distance, 1.0 / 3.0);
        assert_eq!(
            hit.point,
            Vec3 {
                x: 1.0,
                y: 1.5,
                z: 1.5,
            }
        );
        assert!(ray_intersects_aabb(
            Vec3 {
                x: 0.0,
                y: 3.0,
                z: 1.5,
            },
            Vec3 {
                x: 3.0,
                y: 3.0,
                z: 1.5,
            },
            aabb,
        )
        .is_none());
    }
}
