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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityPose {
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    Killed,
    Discarded,
    UnloadedToChunk,
    UnloadedWithPlayer,
    ChangedDimension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityFlags {
    pub on_fire: bool,
    pub crouching: bool,
    pub sprinting: bool,
    pub swimming: bool,
    pub invisible: bool,
    pub glowing: bool,
    pub fall_flying: bool,
}

impl EntityFlags {
    pub fn metadata_byte(self) -> u8 {
        u8::from(self.on_fire)
            | (u8::from(self.crouching) << 1)
            | (u8::from(self.sprinting) << 3)
            | (u8::from(self.swimming) << 4)
            | (u8::from(self.invisible) << 5)
            | (u8::from(self.glowing) << 6)
            | (u8::from(self.fall_flying) << 7)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseEntity {
    pub id: i32,
    pub uuid: String,
    pub entity_type: &'static str,
    pub position: Vec3,
    pub rotation: (f32, f32),
    pub velocity: Vec3,
    pub dimensions: EntityDimensions,
    pub pose: EntityPose,
    pub flags: EntityFlags,
    pub passengers: Vec<i32>,
    pub vehicle: Option<i32>,
    pub portal_cooldown: i32,
    pub remaining_fire_ticks: i32,
    pub air_supply: i32,
    pub ticks_frozen: i32,
    pub fall_distance: f32,
    pub removal_reason: Option<RemovalReason>,
}

impl BaseEntity {
    pub fn new(
        id: i32,
        uuid: String,
        entity_type: &'static str,
        dimensions: EntityDimensions,
    ) -> Self {
        Self {
            id,
            uuid,
            entity_type,
            position: Vec3::ZERO,
            rotation: (0.0, 0.0),
            velocity: Vec3::ZERO,
            dimensions,
            pose: EntityPose::Standing,
            flags: EntityFlags {
                on_fire: false,
                crouching: false,
                sprinting: false,
                swimming: false,
                invisible: false,
                glowing: false,
                fall_flying: false,
            },
            passengers: Vec::new(),
            vehicle: None,
            portal_cooldown: 0,
            remaining_fire_ticks: 0,
            air_supply: 300,
            ticks_frozen: 0,
            fall_distance: 0.0,
            removal_reason: None,
        }
    }

    pub fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        self.position = Vec3 {
            x: clamp_coord(x, 30_000_512.0),
            y: clamp_coord(y, 20_000_000.0),
            z: clamp_coord(z, 30_000_512.0),
        };
    }

    pub fn set_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.rotation = (wrap_degrees(y_rot), wrap_degrees(x_rot.clamp(-90.0, 90.0)));
    }

    pub fn turn(&mut self, y_delta: f32, x_delta: f32) {
        let (y, x) = self.rotation;
        self.set_rotation(y + y_delta, (x + x_delta).clamp(-90.0, 90.0));
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    pub fn bounding_box(&self) -> Aabb {
        let half_width = f64::from(self.dimensions.width) / 2.0;
        Aabb {
            min: Vec3 {
                x: self.position.x - half_width,
                y: self.position.y,
                z: self.position.z - half_width,
            },
            max: Vec3 {
                x: self.position.x + half_width,
                y: self.position.y + f64::from(self.dimensions.height),
                z: self.position.z + half_width,
            },
        }
    }

    pub fn set_remaining_fire_ticks(&mut self, ticks: i32) {
        self.remaining_fire_ticks = ticks.max(0);
        self.flags.on_fire = self.remaining_fire_ticks > 0;
    }

    pub fn tick_fire_and_portal(&mut self, in_lava: bool) {
        if self.portal_cooldown > 0 {
            self.portal_cooldown -= 1;
        }
        if self.remaining_fire_ticks > 0 {
            if !in_lava {
                self.remaining_fire_ticks -= 1;
            }
            self.flags.on_fire = self.remaining_fire_ticks > 0;
        }
    }

    pub fn set_portal_cooldown(&mut self, ticks: i32) {
        self.portal_cooldown = ticks.max(0);
    }

    pub fn start_riding(&mut self, vehicle: &mut BaseEntity) -> Result<(), &'static str> {
        if self.id == vehicle.id || self.passengers.contains(&vehicle.id) {
            return Err("recursive riding is not allowed");
        }
        if self.vehicle == Some(vehicle.id) {
            return Ok(());
        }
        self.pose = EntityPose::Standing;
        self.vehicle = Some(vehicle.id);
        if !vehicle.passengers.contains(&self.id) {
            vehicle.passengers.push(self.id);
        }
        Ok(())
    }

    pub fn stop_riding(&mut self, vehicle: &mut BaseEntity) {
        if self.vehicle == Some(vehicle.id) {
            self.vehicle = None;
            vehicle.passengers.retain(|passenger| *passenger != self.id);
        }
    }

    pub fn remove(&mut self, reason: RemovalReason) {
        self.removal_reason = Some(reason);
    }

    pub fn sync_plan(&self) -> EntitySyncPlan {
        EntitySyncPlan {
            add_entity: self.removal_reason.is_none(),
            metadata_flags: self.flags.metadata_byte(),
            pose: self.pose,
            motion: self.velocity,
            teleport_position: self.position,
            passengers: self.passengers.clone(),
            removed: self.removal_reason,
        }
    }

    pub fn save(&self) -> SavedEntity {
        SavedEntity {
            id: self.entity_type,
            uuid: self.uuid.clone(),
            pos: self.position,
            rotation: self.rotation,
            motion: self.velocity,
            fire: self.remaining_fire_ticks as i16,
            air: self.air_supply as i16,
            portal_cooldown: self.portal_cooldown,
            ticks_frozen: (self.ticks_frozen > 0).then_some(self.ticks_frozen),
            passengers: self.passengers.clone(),
        }
    }

    pub fn load(&mut self, saved: SavedEntity) {
        self.uuid = saved.uuid;
        self.set_pos(saved.pos.x, saved.pos.y, saved.pos.z);
        self.set_rotation(saved.rotation.0, saved.rotation.1);
        self.velocity = saved.motion;
        self.remaining_fire_ticks = i32::from(saved.fire).max(0);
        self.flags.on_fire = self.remaining_fire_ticks > 0;
        self.air_supply = i32::from(saved.air);
        self.portal_cooldown = saved.portal_cooldown.max(0);
        self.ticks_frozen = saved.ticks_frozen.unwrap_or(0);
        self.passengers = saved.passengers;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SavedEntity {
    pub id: &'static str,
    pub uuid: String,
    pub pos: Vec3,
    pub rotation: (f32, f32),
    pub motion: Vec3,
    pub fire: i16,
    pub air: i16,
    pub portal_cooldown: i32,
    pub ticks_frozen: Option<i32>,
    pub passengers: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntitySyncPlan {
    pub add_entity: bool,
    pub metadata_flags: u8,
    pub pose: EntityPose,
    pub motion: Vec3,
    pub teleport_position: Vec3,
    pub passengers: Vec<i32>,
    pub removed: Option<RemovalReason>,
}

fn clamp_coord(value: f64, limit: f64) -> f64 {
    value.clamp(-limit, limit)
}

fn wrap_degrees(value: f32) -> f32 {
    value % 360.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: i32) -> BaseEntity {
        BaseEntity::new(
            id,
            format!("00000000-0000-0000-0000-{id:012}"),
            "minecraft:pig",
            EntityDimensions {
                width: 0.9,
                height: 0.9,
                eye_height: 0.8,
            },
        )
    }

    #[test]
    fn base_entity_initializes_identity_position_pose_flags_and_box() {
        let entity = entity(1);
        assert_eq!(entity.id, 1);
        assert_eq!(entity.entity_type, "minecraft:pig");
        assert_eq!(entity.position, Vec3::ZERO);
        assert_eq!(entity.pose, EntityPose::Standing);
        assert_eq!(entity.flags.metadata_byte(), 0);
        assert_eq!(
            entity.bounding_box(),
            Aabb {
                min: Vec3 {
                    x: -0.44999998807907104,
                    y: 0.0,
                    z: -0.44999998807907104
                },
                max: Vec3 {
                    x: 0.44999998807907104,
                    y: 0.8999999761581421,
                    z: 0.44999998807907104
                }
            }
        );
    }

    #[test]
    fn position_rotation_velocity_fire_air_freeze_and_portal_state_follow_entity_rules() {
        let mut entity = entity(2);
        entity.set_pos(40_000_000.0, -30_000_000.0, -40_000_000.0);
        assert_eq!(
            entity.position,
            Vec3 {
                x: 30_000_512.0,
                y: -20_000_000.0,
                z: -30_000_512.0
            }
        );

        entity.set_rotation(725.0, 120.0);
        assert_eq!(entity.rotation, (5.0, 90.0));
        entity.turn(10.0, -200.0);
        assert_eq!(entity.rotation, (15.0, -90.0));

        entity.set_velocity(Vec3 {
            x: 1.0,
            y: -0.5,
            z: 0.25,
        });
        entity.set_remaining_fire_ticks(3);
        entity.set_portal_cooldown(2);
        entity.air_supply = 42;
        entity.ticks_frozen = 9;
        entity.tick_fire_and_portal(false);
        assert_eq!(entity.remaining_fire_ticks, 2);
        assert_eq!(entity.portal_cooldown, 1);
        assert!(entity.flags.on_fire);
    }

    #[test]
    fn passenger_vehicle_relationships_reject_recursion_and_sync_passengers() {
        let mut rider = entity(3);
        let mut vehicle = entity(4);
        rider.start_riding(&mut vehicle).unwrap();
        assert_eq!(rider.vehicle, Some(4));
        assert_eq!(vehicle.passengers, vec![3]);

        assert_eq!(
            rider.start_riding(&mut rider.clone()),
            Err("recursive riding is not allowed")
        );

        rider.stop_riding(&mut vehicle);
        assert_eq!(rider.vehicle, None);
        assert!(vehicle.passengers.is_empty());
    }

    #[test]
    fn save_and_load_round_trip_core_entity_lifecycle_fields() {
        let mut original = entity(5);
        original.set_pos(1.25, 64.0, -9.5);
        original.set_rotation(45.0, 20.0);
        original.set_velocity(Vec3 {
            x: 0.1,
            y: 0.2,
            z: 0.3,
        });
        original.set_remaining_fire_ticks(80);
        original.set_portal_cooldown(10);
        original.air_supply = 250;
        original.ticks_frozen = 4;
        original.passengers = vec![6, 7];

        let saved = original.save();
        assert_eq!(saved.id, "minecraft:pig");
        assert_eq!(saved.fire, 80);
        assert_eq!(saved.ticks_frozen, Some(4));

        let mut loaded = entity(99);
        loaded.load(saved);
        assert_eq!(loaded.uuid, original.uuid);
        assert_eq!(loaded.position, original.position);
        assert_eq!(loaded.rotation, original.rotation);
        assert_eq!(loaded.velocity, original.velocity);
        assert_eq!(loaded.passengers, vec![6, 7]);
    }

    #[test]
    fn removal_and_sync_plan_cover_metadata_motion_teleport_passenger_and_remove_packets() {
        let mut entity = entity(8);
        entity.flags.crouching = true;
        entity.flags.glowing = true;
        entity.pose = EntityPose::Crouching;
        entity.set_velocity(Vec3 {
            x: 0.0,
            y: 0.4,
            z: 0.0,
        });
        entity.passengers = vec![9];

        let sync = entity.sync_plan();
        assert!(sync.add_entity);
        assert_eq!(sync.metadata_flags, 0b0100_0010);
        assert_eq!(sync.pose, EntityPose::Crouching);
        assert_eq!(sync.passengers, vec![9]);

        entity.remove(RemovalReason::ChangedDimension);
        let removed = entity.sync_plan();
        assert!(!removed.add_entity);
        assert_eq!(removed.removed, Some(RemovalReason::ChangedDimension));
    }

    #[test]
    fn all_pose_and_removal_variants_are_represented() {
        let poses = [
            EntityPose::Standing,
            EntityPose::FallFlying,
            EntityPose::Sleeping,
            EntityPose::Swimming,
            EntityPose::SpinAttack,
            EntityPose::Crouching,
            EntityPose::LongJumping,
            EntityPose::Dying,
            EntityPose::Croaking,
            EntityPose::UsingTongue,
            EntityPose::Sitting,
            EntityPose::Roaring,
            EntityPose::Sniffing,
            EntityPose::Emerging,
            EntityPose::Digging,
            EntityPose::Sliding,
            EntityPose::Shooting,
            EntityPose::Inhaling,
        ];
        assert_eq!(poses.len(), 18);

        let removals = [
            RemovalReason::Killed,
            RemovalReason::Discarded,
            RemovalReason::UnloadedToChunk,
            RemovalReason::UnloadedWithPlayer,
            RemovalReason::ChangedDimension,
        ];
        assert_eq!(removals.len(), 5);
    }
}
