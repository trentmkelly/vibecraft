/// Server-side representation of a dropped item entity.
///
/// Java reference: `net/minecraft/world/entity/item/ItemEntity.java`
///
/// A `DroppedItem` is created whenever a block drop is spawned (via `Block.popResource`)
/// or a player Q-drops an item.  All live item entities are stored in the world-level
/// `WorldItemEntities` store so they persist across player sessions.

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
    /// Horizontal velocity in blocks/tick, updated each tick with 0.98 drag.
    /// Java: `Entity.setDeltaMovement` — initial vel set by Block.popResource or LivingEntity drop.
    pub vel_x: f64,
    /// Vertical velocity (stored for completeness; Y position is not simulated server-side).
    pub vel_y: f64,
    /// Horizontal velocity in blocks/tick, updated each tick with 0.98 drag.
    pub vel_z: f64,
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

/// World-level store for all on-ground item entities.
///
/// Shared across every player session via `Arc<Mutex<WorldItemEntities>>` so that item
/// entities persist when a player disconnects and reconnects.
///
/// Java reference: `ServerLevel.entityStorage` — entity lists belong to the world, not
/// to any individual player connection.
pub struct WorldItemEntities {
    pub entities: Vec<DroppedItem>,
    /// Monotonically-increasing entity ID counter.  Entity ID 1 is always reserved for
    /// the player; item entities start at 2.  Never resets between sessions, preventing
    /// ID collisions when a player reconnects while items are on the ground.
    next_entity_id: i32,
}

impl WorldItemEntities {
    /// Creates an empty store.  The first `alloc_entity_id` call returns 2 (1 is the player).
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_entity_id: 1,
        }
    }

    /// Restores a previously serialized store.
    /// Java: EntityStorage.loadEntities() — entities and their IDs are preserved from disk.
    pub fn restore(entities: Vec<DroppedItem>, next_entity_id: i32) -> Self {
        Self {
            entities,
            next_entity_id,
        }
    }

    /// Returns the current value of the entity ID counter (before the next allocation).
    /// Used during serialization to persist the counter across server restarts.
    pub fn next_entity_id(&self) -> i32 {
        self.next_entity_id
    }

    /// Allocates and returns the next unique entity ID.
    pub fn alloc_entity_id(&mut self) -> i32 {
        self.next_entity_id = self.next_entity_id.wrapping_add(1);
        self.next_entity_id
    }
}

impl Default for WorldItemEntities {
    fn default() -> Self {
        Self::new()
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

/// Result produced by a single call to `tick`.
///
/// The caller is responsible for sending the appropriate network packets:
///   - `removed`: send `ClientboundRemoveEntitiesPacket` for each ID.
///   - `count_updates`: send `ClientboundSetEntityDataPacket` for each entry so clients
///     display the correct merged stack size.
pub struct ItemTickResult {
    /// Entity IDs removed this tick (expired by age or absorbed into a merge target).
    /// Caller sends `RemoveEntities`.
    pub removed: Vec<i32>,
    /// `(entity_id, item_name, new_count)` for entities whose count grew via merge.
    /// Caller sends `SetEntityData` for each entry.
    pub count_updates: Vec<(i32, &'static str, i32)>,
}

/// Advances physics, age, and pickup delay for every entity, then merges nearby
/// same-type stacks.  Returns IDs that were removed and stacks whose count changed.
///
/// The caller must:
///   1. Send `ClientboundRemoveEntitiesPacket` for every ID in `result.removed`.
///   2. Send `ClientboundSetEntityDataPacket` for every entry in `result.count_updates`.
///
/// # Phase 1 — physics + age (Java: `ItemEntity.tick`)
///
/// For each entity:
///   - Apply horizontal drag: `x += vel_x; z += vel_z; vel_x *= 0.98; vel_z *= 0.98`.
///     Y position is not simulated server-side (gravity is a client-side visual).
///   - Decrement `pickup_delay` (skip if `INFINITE_PICKUP_DELAY`).
///   - Increment `age` (skip if `INFINITE_LIFETIME_AGE`); expire when `age >= ITEM_LIFETIME`.
///
/// # Phase 2 — merge (Java: `ItemEntity.mergeWithNeighbours`)
///
/// After physics, all pairs are compared.  Two entities merge when:
///   - Both have `pickup_delay == 0`.
///   - Same `item` name.
///   - `combined_count <= 64` (max stack).
///   - Centre-to-centre distance `|ax − bx| <= 0.75 && |az − bz| <= 0.75`
///     (Java: `inflate(0.5, 0.0, 0.5)` on a 0.25-wide item AABB).
///
/// The entity with the lower index absorbs the higher-index one: the survivor's count
/// grows, its age is set to `min(own_age, absorbed_age)` (Java keeps the younger age),
/// and the absorbed entity is removed.
pub fn tick(entities: &mut Vec<DroppedItem>) -> ItemTickResult {
    let mut removed: Vec<i32> = Vec::new();

    // ── Phase 1: physics + age ────────────────────────────────────────────────
    entities.retain_mut(|e| {
        // Horizontal motion with drag.  Y is a client-side visual; we skip it server-side.
        // Java: Entity.move() applies drag after each tick.
        e.x += e.vel_x;
        e.z += e.vel_z;
        e.vel_x *= 0.98;
        e.vel_z *= 0.98;

        // Pickup delay counts down once per tick.
        // Java: `if (pickupDelay > 0 && pickupDelay != INFINITE_PICKUP_DELAY) pickupDelay--;`
        if e.pickup_delay > 0 && e.pickup_delay != INFINITE_PICKUP_DELAY {
            e.pickup_delay -= 1;
        }

        // Age increments unconditionally except for the sentinel.
        // Java: `if (age != INFINITE_LIFETIME) age++;`
        if e.age != INFINITE_LIFETIME_AGE {
            e.age += 1;
        }

        // Expire once age reaches the lifetime ceiling.
        // Java: `if (!level.isClientSide() && age >= LIFETIME) this.discard();`
        if e.age >= ITEM_LIFETIME {
            removed.push(e.entity_id);
            false
        } else {
            true
        }
    });

    // ── Phase 2: merge (Java: ItemEntity.mergeWithNeighbours) ─────────────────
    //
    // We need index-based iteration to satisfy the borrow checker: two simultaneous
    // mutable references into the same Vec are forbidden, so we copy the fields we
    // need from entity j before mutating entity i.
    let len = entities.len();
    // absorbed[j] = true ⟹ entity j was absorbed into some earlier entity.
    let mut absorbed = vec![false; len];
    // did_absorb[i] = true ⟹ entity i gained count from at least one merge this tick.
    let mut did_absorb = vec![false; len];

    for i in 0..len {
        if absorbed[i] {
            continue;
        }
        for j in (i + 1)..len {
            if absorbed[j] {
                continue;
            }

            // Both must be ready for pickup (delay == 0).
            // Java: `mergeWithNeighbours` checks `!hasPickUpDelay()` on both sides.
            if entities[i].pickup_delay != 0 || entities[j].pickup_delay != 0 {
                continue;
            }

            // Must be the same item type.
            if entities[i].item != entities[j].item {
                continue;
            }

            // Combined count must not exceed the vanilla max stack size.
            let combined = entities[i].count + entities[j].count;
            if combined > 64 {
                continue;
            }

            // Must be within merge distance.
            // Java: item AABB is 0.25 wide; inflate(0.5, 0.0, 0.5) → 0.75 center-to-center.
            let dx = (entities[i].x - entities[j].x).abs();
            let dz = (entities[i].z - entities[j].z).abs();
            if dx > 0.75 || dz > 0.75 {
                continue;
            }

            // Merge j into i.
            // Copy fields from j before mutating i to satisfy the borrow checker.
            let j_age = entities[j].age;
            let j_eid = entities[j].entity_id;

            entities[i].count = combined;
            // Java: ItemEntity keeps the younger (lower) age of the two.
            entities[i].age = entities[i].age.min(j_age);

            absorbed[j] = true;
            did_absorb[i] = true;
            removed.push(j_eid);
        }
    }

    // Build count_updates before draining absorbed entities.
    let mut count_updates: Vec<(i32, &'static str, i32)> = Vec::new();
    for i in 0..len {
        if did_absorb[i] {
            count_updates.push((entities[i].entity_id, entities[i].item, entities[i].count));
        }
    }

    // Remove absorbed entities (walk in reverse so indices stay valid).
    for j in (0..len).rev() {
        if absorbed[j] {
            entities.remove(j);
        }
    }

    ItemTickResult {
        removed,
        count_updates,
    }
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
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
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
            assert!(
                tick(&mut items).removed.is_empty(),
                "should not expire on tick {i}"
            );
            assert!(
                !items[0].can_be_picked_up_by("uuid"),
                "should not be pickable with delay={}",
                items[0].pickup_delay
            );
        }
        // Final tick: delay goes 1 → 0; item becomes pickable.
        assert!(tick(&mut items).removed.is_empty());
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
        let result = tick(&mut items);
        assert_eq!(result.removed, vec![42]);
        assert!(items.is_empty());
    }

    #[test]
    fn item_survives_one_tick_before_lifetime() {
        let mut items = vec![make_item(7, 0, ITEM_LIFETIME - 2)];
        assert!(tick(&mut items).removed.is_empty());
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn infinite_lifetime_age_never_increments_or_expires() {
        // Java: INFINITE_LIFETIME (Short.MIN_VALUE) is exempt from the age increment branch.
        let mut items = vec![make_item(1, 0, INFINITE_LIFETIME_AGE)];
        for _ in 0..ITEM_LIFETIME + 10 {
            let result = tick(&mut items);
            assert!(
                result.removed.is_empty(),
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
        let result = tick(&mut items);
        assert_eq!(result.removed.len(), 2);
        assert!(result.removed.contains(&1));
        assert!(result.removed.contains(&3));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].entity_id, 2);
    }

    // ─── physics ─────────────────────────────────────────────────────────────

    #[test]
    fn horizontal_physics_moves_items_and_applies_drag() {
        // Java: Entity.move() applies velocity then multiplies by drag factor 0.98 each tick.
        let mut items = vec![DroppedItem {
            vel_x: 1.0,
            ..make_item(1, 0, 0)
        }];
        tick(&mut items);
        // Position advanced by original velocity before drag is applied.
        assert!(
            (items[0].x - 1.0).abs() < 1e-10,
            "x should be 1.0 after one tick, got {}",
            items[0].x
        );
        assert!(
            (items[0].vel_x - 0.98).abs() < 1e-10,
            "vel_x should be 0.98 after drag, got {}",
            items[0].vel_x
        );
    }

    // ─── item merge ──────────────────────────────────────────────────────────

    #[test]
    fn nearby_same_type_items_merge_when_delay_zero() {
        // Two dirt items within merge range — should collapse into one with count=2.
        let mut items = vec![
            DroppedItem {
                entity_id: 1,
                item: "minecraft:dirt",
                count: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 0,
                age: 0,
                target_uuid: None,
            },
            DroppedItem {
                entity_id: 2,
                item: "minecraft:dirt",
                count: 1,
                x: 0.3,
                y: 0.0,
                z: 0.3,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 0,
                age: 0,
                target_uuid: None,
            },
        ];
        let result = tick(&mut items);
        assert_eq!(items.len(), 1, "one stack should survive after merge");
        assert_eq!(items[0].count, 2, "survivor should have count=2");
        assert!(
            result.removed.contains(&2),
            "absorbed entity (id=2) should be in removed"
        );
        assert_eq!(
            result.count_updates,
            vec![(1, "minecraft:dirt", 2)],
            "count_updates should record (survivor_id, item, new_count)"
        );
    }

    #[test]
    fn items_do_not_merge_while_pickup_delay_nonzero() {
        // Java: ItemEntity.tick() decrements pickupDelay first, then calls mergeWithNeighbours
        // only if !hasPickUpDelay() (i.e. pickupDelay == 0 after the decrement).
        // Items starting with delay=2 still have delay=1 after one tick, so they do NOT merge.
        let mut items = vec![
            DroppedItem {
                entity_id: 1,
                item: "minecraft:dirt",
                count: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 2,
                age: 0,
                target_uuid: None,
            },
            DroppedItem {
                entity_id: 2,
                item: "minecraft:dirt",
                count: 1,
                x: 0.3,
                y: 0.0,
                z: 0.3,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 2,
                age: 0,
                target_uuid: None,
            },
        ];
        let result = tick(&mut items);
        // After one tick: delay=2 → delay=1, still nonzero, no merge.
        assert_eq!(
            items.len(),
            2,
            "no merge should occur while delay is nonzero after tick"
        );
        assert!(result.removed.is_empty());
        assert!(result.count_updates.is_empty());
        assert_eq!(items[0].pickup_delay, 1);
        assert_eq!(items[1].pickup_delay, 1);
    }

    #[test]
    fn items_do_not_merge_if_too_far() {
        // Items 1.0 blocks apart in X exceed the 0.75 merge threshold.
        let mut items = vec![
            DroppedItem {
                entity_id: 1,
                item: "minecraft:dirt",
                count: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 0,
                age: 0,
                target_uuid: None,
            },
            DroppedItem {
                entity_id: 2,
                item: "minecraft:dirt",
                count: 1,
                x: 1.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 0,
                age: 0,
                target_uuid: None,
            },
        ];
        let result = tick(&mut items);
        assert_eq!(items.len(), 2, "distant items should not merge");
        assert!(result.removed.is_empty());
        assert!(result.count_updates.is_empty());
    }

    #[test]
    fn items_do_not_merge_different_types() {
        // Different item types must never merge.
        let mut items = vec![
            DroppedItem {
                entity_id: 1,
                item: "minecraft:dirt",
                count: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 0,
                age: 0,
                target_uuid: None,
            },
            DroppedItem {
                entity_id: 2,
                item: "minecraft:stone",
                count: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                pickup_delay: 0,
                age: 0,
                target_uuid: None,
            },
        ];
        let result = tick(&mut items);
        assert_eq!(items.len(), 2, "different item types must not merge");
        assert!(result.removed.is_empty());
        assert!(result.count_updates.is_empty());
    }

    // ─── WorldItemEntities ────────────────────────────────────────────────────

    #[test]
    fn world_item_entities_first_alloc_returns_two() {
        // Entity ID 1 is reserved for the player; items start at 2.
        let mut store = WorldItemEntities::new();
        assert_eq!(store.alloc_entity_id(), 2);
    }

    #[test]
    fn world_item_entities_alloc_increments_monotonically() {
        let mut store = WorldItemEntities::new();
        let a = store.alloc_entity_id();
        let b = store.alloc_entity_id();
        let c = store.alloc_entity_id();
        assert_eq!(a, 2);
        assert_eq!(b, 3);
        assert_eq!(c, 4);
    }

    #[test]
    fn world_item_entities_starts_empty() {
        let store = WorldItemEntities::new();
        assert!(store.entities.is_empty());
    }

    #[test]
    fn world_item_entities_default_equals_new() {
        // Default and new() must produce stores with identical starting state.
        let mut a = WorldItemEntities::new();
        let mut b = WorldItemEntities::default();
        assert_eq!(a.entities.len(), b.entities.len());
        assert_eq!(
            a.alloc_entity_id(),
            b.alloc_entity_id(),
            "default and new should start with the same entity ID counter"
        );
    }

    #[test]
    fn world_item_entities_restore_preserves_counter_and_entities() {
        // Java: EntityStorage.loadEntities() restores entities and their IDs from disk;
        // the server must not reset the counter or re-use existing IDs after a reload.
        let items = vec![make_item(5, 0, 0), make_item(6, 0, 0)];
        let store = WorldItemEntities::restore(items, 6);
        assert_eq!(store.entities.len(), 2);
        // next_entity_id() reports the counter as stored (6).
        assert_eq!(store.next_entity_id(), 6);
    }

    #[test]
    fn world_item_entities_restore_alloc_continues_after_loaded_ids() {
        // After restoration the counter must not collide with any restored entity IDs.
        let items = vec![make_item(5, 0, 0), make_item(6, 0, 0)];
        let mut store = WorldItemEntities::restore(items, 6);
        // First allocation must return 7, not 2 (which would collide with restored IDs).
        assert_eq!(store.alloc_entity_id(), 7);
    }
}
