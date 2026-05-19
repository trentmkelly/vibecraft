/// Server-side representation of a dropped item entity.
///
/// Java reference: `net/minecraft/world/entity/item/ItemEntity.java`
///
/// A `DroppedItem` is created whenever a block drop is spawned (via `Block.popResource`)
/// and is retained in `PlaySessionState::dropped_items` so that:
///   - The age tick can expire the entity after `ITEM_LIFETIME` ticks.
///   - Proximity checks can trigger pickup when a player walks over the item.

/// Ticks before an item entity despawns.  Java: `ItemEntity.LIFETIME = 6000`.
pub const ITEM_LIFETIME: i32 = 6000;

/// Sentinel pickup-delay value meaning the item can never be picked up.
/// Java: `ItemEntity.INFINITE_PICKUP_DELAY = Short.MAX_VALUE = 32767`.
pub const INFINITE_PICKUP_DELAY: i32 = 32767;

/// Sentinel age value meaning the item never ages out (never despawns naturally).
/// Java: `ItemEntity.INFINITE_LIFETIME = Short.MIN_VALUE = -32768`.
pub const INFINITE_LIFETIME_AGE: i32 = -32768;

/// Ticks added to a freshly-spawned item before it can be picked up.
/// Java: `ItemEntity.setDefaultPickUpDelay()` sets `pickupDelay = 10`, called by
/// `Block.popResource()` for all block drops.
pub const DEFAULT_PICKUP_DELAY: i32 = 10;

/// Server-side state for a single dropped item entity.
#[derive(Debug, Clone, PartialEq)]
pub struct DroppedItem {
    /// Network entity ID assigned at spawn.
    pub entity_id: i32,
    /// Canonical registry name (e.g. `"minecraft:coal"`).  Stored as `&'static str` so it
    /// can be passed directly to `ItemStack::new` without allocation.
    pub item: &'static str,
    pub count: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Ticks remaining before this entity can be picked up.
    /// `INFINITE_PICKUP_DELAY` means the entity is never pickable.
    pub pickup_delay: i32,
    /// Entity age in ticks.  Counts up each tick; the entity expires when it reaches
    /// `ITEM_LIFETIME`.  `INFINITE_LIFETIME_AGE` disables natural despawning.
    pub age: i32,
    /// When set, only the identified player may pick up this item.
    /// Java: `ItemEntity.target` UUID field.
    pub target_uuid: Option<String>,
}

impl DroppedItem {
    /// Returns true if `player_uuid` is allowed to collect this item right now.
    ///
    /// Java: `ItemEntity.playerTouch` — `pickupDelay == 0 && (target == null || target.equals(player.getUUID()))`
    pub fn can_be_picked_up_by(&self, player_uuid: &str) -> bool {
        self.pickup_delay == 0 && self.target_uuid.as_ref().map_or(true, |t| t == player_uuid)
    }
}

/// Returns true when the item entity position falls inside the player's inflated pickup AABB.
///
/// Java: `Player.aiStep` — `getBoundingBox().inflate(1.0, 0.5, 1.0)`.
/// Player bounding box: width 0.6, height 1.8, origin at foot level.
/// After inflation:
///   x ∈ [px − 1.3, px + 1.3]
///   y ∈ [py − 0.5, py + 2.3]
///   z ∈ [pz − 1.3, pz + 1.3]
pub fn in_pickup_range(px: f64, py: f64, pz: f64, ex: f64, ey: f64, ez: f64) -> bool {
    ex >= px - 1.3
        && ex <= px + 1.3
        && ey >= py - 0.5
        && ey <= py + 2.3
        && ez >= pz - 1.3
        && ez <= pz + 1.3
}

/// Advances age and pickup delay for every entity, removing any that have expired.
///
/// Returns the entity IDs of entities removed due to reaching `ITEM_LIFETIME`.
/// The caller must send a `ClientboundRemoveEntitiesPacket` for each returned ID.
///
/// Java: `ItemEntity.tick` —
///   `if (pickupDelay > 0 && pickupDelay != INFINITE_PICKUP_DELAY) pickupDelay--;`
///   `if (age != INFINITE_LIFETIME) age++;`
///   `if (!level.isClientSide() && age >= LIFETIME) this.discard();`
pub fn tick(entities: &mut Vec<DroppedItem>) -> Vec<i32> {
    let mut expired = Vec::new();
    entities.retain_mut(|e| {
        if e.pickup_delay > 0 && e.pickup_delay != INFINITE_PICKUP_DELAY {
            e.pickup_delay -= 1;
        }
        if e.age != INFINITE_LIFETIME_AGE {
            e.age += 1;
        }
        if e.age >= ITEM_LIFETIME {
            expired.push(e.entity_id);
            false
        } else {
            true
        }
    });
    expired
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(entity_id: i32, pickup_delay: i32, age: i32) -> DroppedItem {
        DroppedItem {
            entity_id,
            item: "minecraft:coal",
            count: 1,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            pickup_delay,
            age,
            target_uuid: None,
        }
    }

    // ─── pickup delay ───────────────────────────────────────────────────────────

    #[test]
    fn default_pickup_delay_counts_down_to_pickable() {
        // Java: ItemEntity.setDefaultPickUpDelay() sets delay to 10; each tick decrements by 1.
        // After DEFAULT_PICKUP_DELAY ticks the delay reaches 0 and the item becomes pickable.
        let mut items = vec![make_item(1, DEFAULT_PICKUP_DELAY, 0)];
        // Ticks 1..(DEFAULT_PICKUP_DELAY-1): delay > 0, item is not yet pickable.
        for i in 0..DEFAULT_PICKUP_DELAY - 1 {
            assert!(tick(&mut items).is_empty(), "should not expire on tick {i}");
            assert!(
                !items[0].can_be_picked_up_by("uuid"),
                "should not be pickable with delay={}",
                items[0].pickup_delay
            );
        }
        // Final tick: delay goes 1 → 0; item becomes pickable.
        assert!(tick(&mut items).is_empty());
        assert_eq!(items[0].pickup_delay, 0);
        assert!(items[0].can_be_picked_up_by("uuid"));
    }

    #[test]
    fn infinite_pickup_delay_sentinel_never_decrements() {
        // Java: INFINITE_PICKUP_DELAY (Short.MAX_VALUE) is exempt from the decrement branch.
        let mut items = vec![make_item(1, INFINITE_PICKUP_DELAY, 0)];
        for _ in 0..100 {
            tick(&mut items);
        }
        assert_eq!(items[0].pickup_delay, INFINITE_PICKUP_DELAY);
        assert!(!items[0].can_be_picked_up_by("uuid"));
    }

    // ─── age and lifetime ────────────────────────────────────────────────────────

    #[test]
    fn item_expires_when_age_reaches_lifetime() {
        // Java: age >= LIFETIME → discard().
        let mut items = vec![make_item(42, 0, ITEM_LIFETIME - 1)];
        let expired = tick(&mut items);
        assert_eq!(expired, vec![42]);
        assert!(items.is_empty());
    }

    #[test]
    fn item_survives_one_tick_before_lifetime() {
        let mut items = vec![make_item(7, 0, ITEM_LIFETIME - 2)];
        assert!(tick(&mut items).is_empty());
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn infinite_lifetime_age_never_increments_or_expires() {
        // Java: INFINITE_LIFETIME (Short.MIN_VALUE) is exempt from the age increment branch.
        let mut items = vec![make_item(1, 0, INFINITE_LIFETIME_AGE)];
        for _ in 0..ITEM_LIFETIME + 10 {
            let expired = tick(&mut items);
            assert!(
                expired.is_empty(),
                "infinite-lifetime item must never expire"
            );
        }
        assert_eq!(items[0].age, INFINITE_LIFETIME_AGE);
    }

    // ─── target UUID restriction ──────────────────────────────────────────────

    #[test]
    fn target_uuid_restricts_pickup_to_named_player() {
        let item = DroppedItem {
            target_uuid: Some("player-a".to_string()),
            ..make_item(1, 0, 0)
        };
        assert!(item.can_be_picked_up_by("player-a"));
        assert!(!item.can_be_picked_up_by("player-b"));
    }

    #[test]
    fn no_target_uuid_allows_any_player() {
        let item = make_item(1, 0, 0);
        assert!(item.can_be_picked_up_by("anyone"));
        assert!(item.can_be_picked_up_by("someone-else"));
    }

    // ─── pickup AABB ──────────────────────────────────────────────────────────

    #[test]
    fn in_pickup_range_matches_java_aabb_inflation() {
        // Player at origin. Pickup zone: x ∈ [-1.3, 1.3], y ∈ [-0.5, 2.3], z ∈ [-1.3, 1.3].
        assert!(in_pickup_range(0.0, 0.0, 0.0, 0.0, 0.0, 0.0), "centre");
        assert!(
            in_pickup_range(0.0, 0.0, 0.0, 1.3, 2.3, 1.3),
            "positive corner"
        );
        assert!(
            in_pickup_range(0.0, 0.0, 0.0, -1.3, -0.5, -1.3),
            "negative corner"
        );
        // Just outside each boundary
        assert!(!in_pickup_range(0.0, 0.0, 0.0, 1.31, 0.0, 0.0), "past +x");
        assert!(!in_pickup_range(0.0, 0.0, 0.0, -1.31, 0.0, 0.0), "past -x");
        assert!(!in_pickup_range(0.0, 0.0, 0.0, 0.0, 2.31, 0.0), "past +y");
        assert!(!in_pickup_range(0.0, 0.0, 0.0, 0.0, -0.51, 0.0), "past -y");
        assert!(!in_pickup_range(0.0, 0.0, 0.0, 0.0, 0.0, 1.31), "past +z");
    }

    #[test]
    fn in_pickup_range_accounts_for_non_zero_player_position() {
        // Player at (10, 64, -5)
        let (px, py, pz) = (10.0_f64, 64.0_f64, -5.0_f64);
        assert!(in_pickup_range(px, py, pz, px + 1.0, py + 1.0, pz + 1.0));
        assert!(!in_pickup_range(px, py, pz, px + 1.5, py, pz));
    }

    // ─── multiple entity batch tick ───────────────────────────────────────────

    #[test]
    fn tick_removes_only_expired_entities_from_batch() {
        let mut items = vec![
            make_item(1, 0, ITEM_LIFETIME - 1), // will expire
            make_item(2, 0, 0),                 // young, survives
            make_item(3, 0, ITEM_LIFETIME - 1), // will expire
        ];
        let expired = tick(&mut items);
        assert_eq!(expired.len(), 2);
        assert!(expired.contains(&1));
        assert!(expired.contains(&3));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].entity_id, 2);
    }
}
