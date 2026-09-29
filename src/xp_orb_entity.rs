//! Live world experience orb entities.
//!
//! Java reference: `net.minecraft.world.entity.ExperienceOrb`. The orbs live in the shared
//! [`WorldItemEntities`] store (next to dropped items) so they survive across sessions and
//! share its entity-id counter.
//!
//! Simulation scope: the server does not simulate block collision, gravity or fluids for
//! any dropped entity yet (see `item_entity.rs`, where the client owns vertical physics).
//! Orbs therefore integrate only what the server needs to stay authoritative over:
//! horizontal drag, following the nearest player, merging, expiry and pickup. The vanilla
//! client runs the same `followNearbyPlayer`/`tick` code, so both sides agree on the pull.

use crate::experience_system::{experience_value, ExperienceOrb};
use crate::item_entity::WorldItemEntities;

/// `ExperienceOrb.MAX_FOLLOW_DIST` — radius searched for a player to follow.
pub const XP_ORB_MAX_FOLLOW_DIST: f64 = 8.0;
/// `ExperienceOrb.ORB_GROUPS_PER_AREA` — modulus used to keep orbs of one award apart.
pub const XP_ORB_GROUPS_PER_AREA: i32 = 40;
/// `ExperienceOrb.ORB_MERGE_DISTANCE` — AABB inflation used by `scanForMerges`.
pub const XP_ORB_MERGE_DISTANCE: f64 = 0.5;
/// `EntityType.EXPERIENCE_ORB` bounding box edge (`sized(0.5, 0.5)`).
pub const XP_ORB_SIZE: f64 = 0.5;

/// Small deterministic generator standing in for `Entity.random` / `Level.getRandom()`.
///
/// Java draws orb spawn velocity, yaw and the merge group id from `RandomSource`; the
/// values only affect cosmetic scatter and which nearby orbs may merge, so an
/// injectable splitmix64 keeps the entity code testable without a global RNG.
#[derive(Debug, Clone)]
pub struct XpOrbRandom(u64);

impl XpOrbRandom {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// `RandomSource.nextDouble()`.
    pub fn next_double(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// `RandomSource.nextInt(bound)`.
    pub fn next_int(&mut self, bound: i32) -> i32 {
        (self.next_double() * f64::from(bound)) as i32
    }
}

/// One live experience orb: the pure orb model plus its world state.
#[derive(Debug, Clone, PartialEq)]
pub struct XpOrbEntity {
    /// Value/count/age/health bookkeeping shared with the unit-tested orb model.
    pub orb: ExperienceOrb,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub vel_z: f64,
    /// `Entity.tickCount`.
    pub tick_count: i32,
    /// UUID of the player currently followed (`ExperienceOrb.followingPlayer`).
    pub following: Option<String>,
}

/// The view of a player the orb tick needs (`Level.getNearestPlayer` inputs).
#[derive(Debug, Clone, Copy)]
pub struct XpOrbPlayer<'a> {
    pub id: &'a str,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// `Entity.getEyeHeight()`.
    pub eye_height: f64,
    pub is_spectator: bool,
    /// `LivingEntity.isDeadOrDying()`.
    pub is_dead: bool,
}

/// Outcome of ticking every orb once.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct XpOrbTickResult {
    /// Entity ids discarded this tick (expired or merged away); the caller sends
    /// `ClientboundRemoveEntitiesPacket` for them.
    pub removed: Vec<i32>,
}

impl XpOrbEntity {
    /// `new ExperienceOrb(level, pos, roughly, value)` for a server level (without the
    /// `unstuckIfPossible` collision fix-up, which needs block collisions).
    pub fn spawn(
        entity_id: i32,
        pos: (f64, f64, f64),
        rough_direction: (f64, f64, f64),
        value: i32,
        random: &mut XpOrbRandom,
    ) -> Self {
        // Entity.setYRot(random.nextFloat() * 360) is drawn first; it is not tracked.
        let _yaw = random.next_double() * 360.0;
        let mut movement = (
            (random.next_double() * 0.2 - 0.1) * 2.0,
            random.next_double() * 0.2 * 2.0,
            (random.next_double() * 0.2 - 0.1) * 2.0,
        );
        let rough_len_sqr = dot(rough_direction, rough_direction);
        if rough_len_sqr > 0.0 && dot(rough_direction, movement) < 0.0 {
            movement = (-movement.0, -movement.1, -movement.2);
        }
        // pos.add(roughly.normalize().scale(size * 0.5)); a zero vector normalizes to zero.
        let offset = if rough_len_sqr > 0.0 {
            let len = rough_len_sqr.sqrt();
            let scale = XP_ORB_SIZE * 0.5 / len;
            (
                rough_direction.0 * scale,
                rough_direction.1 * scale,
                rough_direction.2 * scale,
            )
        } else {
            (0.0, 0.0, 0.0)
        };
        Self {
            orb: ExperienceOrb::new(entity_id, value),
            x: pos.0 + offset.0,
            y: pos.1 + offset.1,
            z: pos.2 + offset.2,
            vel_x: movement.0,
            vel_y: movement.1,
            vel_z: movement.2,
            tick_count: 0,
            following: None,
        }
    }

    pub fn entity_id(&self) -> i32 {
        self.orb.id
    }

    /// `ExperienceOrb.getValue()`.
    pub fn value(&self) -> i32 {
        self.orb.value
    }

    /// Whether this orb's bounding box (`0.5` cube standing on `y`) intersects `other`'s
    /// bounding box inflated by [`XP_ORB_MERGE_DISTANCE`] (`getBoundingBox().inflate(0.5)`).
    fn near_for_merge(&self, other: &XpOrbEntity) -> bool {
        let half = XP_ORB_SIZE / 2.0 + XP_ORB_MERGE_DISTANCE;
        (self.x - other.x).abs() < XP_ORB_SIZE / 2.0 + half
            && (self.z - other.z).abs() < XP_ORB_SIZE / 2.0 + half
            && self.y < other.y + XP_ORB_SIZE + XP_ORB_MERGE_DISTANCE
            && self.y + XP_ORB_SIZE > other.y - XP_ORB_MERGE_DISTANCE
    }

    /// `ExperienceOrb.followNearbyPlayer()`.
    fn follow_nearby_player(&mut self, players: &[XpOrbPlayer<'_>]) {
        let still_valid = self.following.as_deref().is_some_and(|id| {
            players.iter().any(|player| {
                player.id == id
                    && !player.is_spectator
                    && self.distance_sqr_to(player.x, player.y, player.z)
                        <= XP_ORB_MAX_FOLLOW_DIST * XP_ORB_MAX_FOLLOW_DIST
            })
        });
        if !still_valid {
            // Level.getNearestPlayer(this, 8.0): closest player strictly inside the radius.
            let nearest = players
                .iter()
                .filter(|player| !player.is_spectator)
                .map(|player| (self.distance_sqr_to(player.x, player.y, player.z), player))
                .filter(|(dist, _)| *dist < XP_ORB_MAX_FOLLOW_DIST * XP_ORB_MAX_FOLLOW_DIST)
                .min_by(|a, b| a.0.total_cmp(&b.0));
            self.following = nearest
                .filter(|(_, player)| !player.is_dead)
                .map(|(_, player)| player.id.to_string());
        }
        let Some(player) = self
            .following
            .as_deref()
            .and_then(|id| players.iter().find(|player| player.id == id))
        else {
            return;
        };
        let delta = (
            player.x - self.x,
            player.y + player.eye_height / 2.0 - self.y,
            player.z - self.z,
        );
        let length_sqr = dot(delta, delta);
        let power = 1.0 - length_sqr.sqrt() / XP_ORB_MAX_FOLLOW_DIST;
        if length_sqr > 0.0 {
            let scale = power * power * 0.1 / length_sqr.sqrt();
            self.vel_x += delta.0 * scale;
            self.vel_y += delta.1 * scale;
            self.vel_z += delta.2 * scale;
        }
    }

    fn distance_sqr_to(&self, x: f64, y: f64, z: f64) -> f64 {
        let (dx, dy, dz) = (self.x - x, self.y - y, self.z - z);
        dx * dx + dy * dy + dz * dz
    }

    /// The movement/aging half of `ExperienceOrb.tick()` (everything except the merge
    /// scan, which needs the other orbs). Block collision, gravity and fluid handling are
    /// client-side for now; see the module docs.
    fn tick_motion(&mut self, players: &[XpOrbPlayer<'_>]) {
        self.tick_count += 1;
        self.follow_nearby_player(players);
        self.x += self.vel_x;
        self.y += self.vel_y;
        self.z += self.vel_z;
        // Airborne friction; the on-ground block friction needs block lookups.
        self.vel_x *= 0.98;
        self.vel_y *= 0.98;
        self.vel_z *= 0.98;
        self.orb.tick_age();
    }
}

/// `ExperienceOrb.tick()` for every orb, including `scanForMerges` on `tickCount % 20 == 1`.
pub fn tick_orbs(orbs: &mut Vec<XpOrbEntity>, players: &[XpOrbPlayer<'_>]) -> XpOrbTickResult {
    let mut result = XpOrbTickResult::default();
    for index in 0..orbs.len() {
        if orbs[index].orb.removed {
            continue;
        }
        orbs[index].tick_motion(players);
        if orbs[index].tick_count % 20 == 1 {
            merge_neighbours(orbs, index);
        }
    }
    orbs.retain(|orb| {
        if orb.orb.removed {
            result.removed.push(orb.entity_id());
            false
        } else {
            true
        }
    });
    result
}

/// `ExperienceOrb.scanForMerges()` for the orb at `index`.
fn merge_neighbours(orbs: &mut [XpOrbEntity], index: usize) {
    for other in 0..orbs.len() {
        if other == index || orbs[other].orb.removed || !orbs[index].near_for_merge(&orbs[other]) {
            continue;
        }
        let (id, value) = (orbs[index].orb.id, orbs[index].orb.value);
        if !orbs[other].orb.can_merge(id, value) {
            continue;
        }
        let mut absorbed = orbs[other].orb.clone();
        if orbs[index].orb.merge(&mut absorbed) {
            orbs[other].orb.removed = true;
        }
    }
}

impl WorldItemEntities {
    /// `ExperienceOrb.award(level, pos, amount)` (`awardWithDirection` with a zero
    /// direction): splits `amount` into vanilla orb values, folding each into an existing
    /// nearby orb when possible. Returns the freshly spawned orbs so the caller can
    /// announce them to clients.
    pub fn award_experience(
        &mut self,
        pos: (f64, f64, f64),
        amount: i32,
        random: &mut XpOrbRandom,
    ) -> Vec<XpOrbEntity> {
        let mut remaining = amount;
        let mut spawned = Vec::new();
        while remaining > 0 {
            let value = experience_value(remaining);
            remaining -= value;
            if self.try_merge_award_into_existing(pos, value, random) {
                continue;
            }
            let entity_id = self.alloc_entity_id();
            let orb = XpOrbEntity::spawn(entity_id, pos, (0.0, 0.0, 0.0), value, random);
            self.xp_orbs.push(orb.clone());
            spawned.push(orb);
        }
        spawned
    }

    /// `ExperienceOrb.tryMergeToExisting`: an orb inside a 1-block cube around `pos` whose
    /// id matches a random merge group absorbs the new value as one more `count`.
    fn try_merge_award_into_existing(
        &mut self,
        pos: (f64, f64, f64),
        value: i32,
        random: &mut XpOrbRandom,
    ) -> bool {
        let id = random.next_int(XP_ORB_GROUPS_PER_AREA);
        let target = self.xp_orbs.iter_mut().find(|orb| {
            (orb.x - pos.0).abs() < XP_ORB_SIZE / 2.0 + 0.5
                && (orb.z - pos.2).abs() < XP_ORB_SIZE / 2.0 + 0.5
                && orb.y < pos.1 + 0.5
                && orb.y + XP_ORB_SIZE > pos.1 - 0.5
                && orb.orb.can_merge(id, value)
        });
        match target {
            Some(orb) => {
                orb.orb.count += 1;
                orb.orb.age = 0;
                true
            }
            None => false,
        }
    }
}

fn dot(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

#[cfg(test)]
mod tests;
