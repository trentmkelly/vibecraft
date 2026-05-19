# Item Pickup Implementation Plan

## Overview

This document is the implementation plan for server-side item pickup — the mechanic by which a player walking over dropped item entities has those items added to their inventory. All constants and behavioral logic are sourced from the decompiled server at `decompiled-server-26.1.2/`.

---

## 1. Java Reference: Constants and Key Logic

### Constants (`ItemEntity.java`)

| Constant | Value | Meaning |
|---|---|---|
| `LIFETIME` | `6000` | Ticks before item entity despawns |
| `INFINITE_PICKUP_DELAY` | `32767` (`Short.MAX_VALUE`) | Sentinel: never pickable |
| `INFINITE_LIFETIME` | `-32768` (`Short.MIN_VALUE`) | Sentinel: never despawns |
| `DEFAULT_HEALTH` | `5` | Item entity hit points |

The pickup delay is set to **10** by `ItemEntity.setDefaultPickUpDelay()` (line ~399), which is called by `Block.popResource()` for all block drops.

### Pickup Condition (`ItemEntity.playerTouch`, line ~339)

```java
if (this.pickupDelay == 0
    && (this.target == null || this.target.equals(player.getUUID()))
    && player.getInventory().add(itemStack)) {
    player.take(this, orgCount);          // sends TakeItemEntityPacket
    if (itemStack.isEmpty()) {
        this.discard();                   // sends RemoveEntitiesPacket
        itemStack.setCount(orgCount);
    }
}
```

### Pickup AABB (`Player.aiStep`, line ~442)

```java
AABB pickupArea = this.getBoundingBox().inflate(1.0, 0.5, 1.0);
List<Entity> nearby = level.getEntities(this, pickupArea);
for (Entity e : nearby) { e.playerTouch(this); }
```

The player bounding box (`EntityType.java`, `PLAYER` entry): width `0.6`, height `1.8`. The entity origin is at foot level. After inflation the pickup zone becomes:

```
x: [player.x - 1.3,  player.x + 1.3]
y: [player.y - 0.5,  player.y + 2.3]
z: [player.z - 1.3,  player.z + 1.3]
```

### Inventory Fill Order (`Inventory.addResource`, line ~195)

1. `getSlotWithRemainingSpace`: check `items[selected]`, then `items[40]` (offhand), then `items[0..36]` in order — fill into any slot that already holds this item and has room.
2. If no partial-fill slot found, `getFreeSlot`: first empty slot in `items[0..36]`.
3. Repeat until count reaches 0 or no progress is made.

`PlayerInventory::add` in `src/player_inventory.rs` already implements this order faithfully.

### Age and Delay Tick (`ItemEntity.tick`, line ~122)

```java
if (pickupDelay > 0 && pickupDelay != INFINITE_PICKUP_DELAY) pickupDelay--;
if (age != INFINITE_LIFETIME)                                  age++;
if (!level.isClientSide() && age >= LIFETIME)                  this.discard();
```

---

## 2. Server-Side Item Entity Tracking

### Problem

When a block is broken, `status.rs` spawns an item drop by writing `ClientboundAddEntityPacket` + `ClientboundSetEntityDataPacket` to the socket and immediately forgets the entity. There is no server-side registry, so the server cannot subsequently check proximity or perform pickup.

### New File: `src/item_entity.rs`

```rust
pub const ITEM_LIFETIME:           i32 = 6000;
pub const INFINITE_PICKUP_DELAY:   i32 = 32767;
pub const INFINITE_LIFETIME_AGE:   i32 = -32768;
pub const DEFAULT_PICKUP_DELAY:    i32 = 10;     // Block.popResource path

pub struct DroppedItem {
    pub entity_id:    i32,
    pub item:         &'static str,  // e.g. "minecraft:coal" — used with PlayerInventory::add
    pub count:        i32,
    pub x:            f64,
    pub y:            f64,
    pub z:            f64,
    pub pickup_delay: i32,
    pub age:          i32,
    pub target_uuid:  Option<String>,
}

impl DroppedItem {
    pub fn can_be_picked_up_by(&self, player_uuid: &str) -> bool {
        self.pickup_delay == 0
            && self.target_uuid.as_ref().map_or(true, |t| t == player_uuid)
    }
}

/// Returns true when the item entity position falls inside the player's inflated pickup AABB.
/// Player origin is at foot level; item position is its centre point.
/// Java: Player.aiStep → getBoundingBox().inflate(1.0, 0.5, 1.0)
pub fn in_pickup_range(px: f64, py: f64, pz: f64, ex: f64, ey: f64, ez: f64) -> bool {
    ex >= px - 1.3 && ex <= px + 1.3
        && ey >= py - 0.5 && ey <= py + 2.3
        && ez >= pz - 1.3 && ez <= pz + 1.3
}

/// Advance age and pickup_delay for all entities; drain any that have reached ITEM_LIFETIME.
/// Caller must send ClientboundRemoveEntitiesPacket for every returned entity_id.
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
```

Add `pub mod item_entity;` to `src/main.rs`.

---

## 3. Changes to `PlaySessionState` (`src/network/status.rs`)

Add three new fields:

```rust
pub struct PlaySessionState {
    // ... existing fields ...
    pub dropped_items:        Vec<DroppedItem>,     // server-tracked item entities
    pub inventory:            PlayerInventory,      // player item inventory
    pub inventory_state_id:   i32,                 // incremented on every slot mutation
}
```

`Default for PlaySessionState` initialises `dropped_items` as `Vec::new()`, `inventory` as `PlayerInventory::new()`, and `inventory_state_id` as `0`.

The existing `write_minimal_play_join` function sends 46 empty slots at join time. Once `inventory` is live in state, this call should draw from `play_state.inventory` so that a reconnecting player sees their items (persistence is out of scope for now; an empty inventory at join is acceptable).

---

## 4. Refactor: Loot Evaluation Return Type

`evaluate_block_loot` in `status.rs` currently returns `Vec<(i32, i32)>` — `(item_protocol_id, count)`. The item spawn path needs the `&'static str` registry name to construct an `ItemStack` for `PlayerInventory::add`. Change the return type to `Vec<(&'static str, i32)>` and derive the protocol ID from the name at spawn time:

```rust
// Before spawning the entity:
let Some(item_protocol_id) = item_catalog::item_protocol_id(item_name) else { continue; };
```

This requires changing `item_protocol_id` in `item_catalog.rs` to accept `&str` (it already does) and updating the entity-data packet write site in `status.rs`.

---

## 5. Item Entity Registration at Break Time (`src/network/status.rs`)

After writing `ClientboundSetEntityDataPacket` in the block-break handler (~line 1813), push to `play_state.dropped_items`:

```rust
play_state.dropped_items.push(DroppedItem {
    entity_id:   eid,
    item:        item_name,   // &'static str from loot evaluation
    count,
    x:           drop_x,
    y:           drop_y,
    z:           drop_z,
    pickup_delay: DEFAULT_PICKUP_DELAY,   // 10 ticks — Block.popResource path
    age:          0,
    target_uuid:  None,
});
```

`drop_x/y/z` are the same coordinates already computed for the entity spawn packet.

---

## 6. New Packets (`src/network/play.rs`)

### `ClientboundTakeItemEntityPacket` (ID 124 = `0x7C`)

Java: `net/minecraft/network/protocol/game/ClientboundTakeItemEntityPacket.java`

Triggers the item-collection animation and sound on all clients tracking the item.

```
VarInt  item_entity_id      — entity being collected
VarInt  collector_entity_id — the player (always 1 in the current single-player setup)
VarInt  amount              — number of items absorbed in this pickup event
```

### `ClientboundSetPlayerInventoryPacket` (ID 108 = `0x6C`)

Java: `net/minecraft/network/protocol/game/ClientboundSetPlayerInventoryPacket.java`

Synchronises a single inventory slot to the client.

```
VarInt    slot      — 0–35 main inventory, 36–39 armour, 40 offhand
ItemStack contents  — encoded as the existing optional ItemStack wire format
```

Add constants and `write` implementations for both packets, following the existing pattern in `play.rs`.

---

## 7. Tick Loop Integration (`src/network/status.rs`)

The play loop is currently packet-driven. Two hooks are needed.

### Hook A: Age tick (wall-clock, ~20 Hz)

Add before the play loop:

```rust
let mut last_item_tick = std::time::Instant::now();
const ITEM_TICK_MS: u64 = 50;
```

At the top of each loop iteration (after the existing keep-alive / time / weather heartbeats):

```rust
if last_item_tick.elapsed().as_millis() >= ITEM_TICK_MS as u128 {
    last_item_tick = std::time::Instant::now();
    let expired = item_entity::tick(&mut play_state.dropped_items);
    if !expired.is_empty() {
        // Send ClientboundRemoveEntitiesPacket for each expired entity
        write_framed_packet(..., CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, |p| {
            ClientboundRemoveEntitiesPacket { entity_ids: expired }.write(p)
        })?;
    }
}
```

### Hook B: Pickup check (after movement packet)

After `update_play_session_state` returns `true` and before `continue`:

```rust
if play_state.game_mode != GameMode::Spectator {
    process_item_pickups(stream, compression, &mut play_state, &player_uuid)?;
}
continue;
```

`process_item_pickups` is a free function (or closure) in `status.rs`:

```rust
fn process_item_pickups(
    stream:      &mut TcpStream,
    compression: Option<i32>,
    state:       &mut PlaySessionState,
    player_uuid: &str,
) -> io::Result<()> {
    let mut to_remove: Vec<i32> = Vec::new();

    for entity in &mut state.dropped_items {
        if !entity.can_be_picked_up_by(player_uuid) { continue; }
        if !item_entity::in_pickup_range(state.x, state.y, state.z,
                                         entity.x, entity.y, entity.z) { continue; }

        let original_count = entity.count;
        let stack = ItemStack::new(entity.item, entity.count);
        let (picked_up, new_count) = match state.inventory.add(stack) {
            InventoryAddResult::FullyAdded                       => (original_count, 0),
            InventoryAddResult::PartiallyAdded { remaining }     => (original_count - remaining, remaining),
            InventoryAddResult::NotAdded                         => continue,
        };

        entity.count = new_count;

        // 1. TakeItemEntity — client pickup animation / sound
        write_framed_packet(stream, compression,
            CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
            |p| ClientboundTakeItemEntityPacket {
                item_entity_id:      entity.entity_id,
                collector_entity_id: 1,
                amount:              picked_up,
            }.write(p))?;

        // 2. RemoveEntities — only when the full stack was consumed
        if new_count <= 0 {
            to_remove.push(entity.entity_id);
            write_framed_packet(stream, compression,
                CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
                |p| ClientboundRemoveEntitiesPacket {
                    entity_ids: vec![entity.entity_id],
                }.write(p))?;
        }

        // 3. SetPlayerInventory — sync every changed slot to the client
        state.inventory_state_id += 1;
        for slot in 0_usize..46 {
            let stack = state.inventory.get_slot(slot);
            write_framed_packet(stream, compression,
                CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
                |p| ClientboundSetPlayerInventoryPacket {
                    slot:     slot as i32,
                    contents: RawItemStack::from_item_stack(stack),
                }.write(p))?;
        }
    }

    state.dropped_items.retain(|e| e.count > 0);
    Ok(())
}
```

Sending all 46 slots on every pickup is safe and keeps the client in sync. A subsequent optimisation can diff the inventory snapshot before/after `add` and send only dirty slots.

---

## 8. Edge Cases

| Case | Handling |
|---|---|
| Inventory full | `add` returns `NotAdded`; skip. No packets sent. Item entity unchanged. |
| Partial pickup (one slot has room for 3 of a stack of 10) | `PartiallyAdded { remaining: 7 }`; `TakeItemEntity` sent with `amount=3`; entity stays with `count=7`. |
| Spectator mode | Guard in Hook B: skip `process_item_pickups` entirely. |
| Player disconnect | `dropped_items` lives in `PlaySessionState` and is dropped with the session. Items vanish. Persistence is a later task. |
| Items spawned by means other than block break | Not yet supported (e.g. mob drops, thrown items). `DEFAULT_PICKUP_DELAY = 10` applies to all block-break drops via `Block.popResource`. |
| Aged-out items | Hook A sends `RemoveEntitiesPacket` for expired entities. |

---

## 9. Summary of File Changes

| File | Change |
|---|---|
| `src/item_entity.rs` | **New.** `DroppedItem`, constants, `tick`, `in_pickup_range`, `can_be_picked_up_by`. |
| `src/main.rs` | Add `pub mod item_entity;` |
| `src/network/play.rs` | Add `ClientboundTakeItemEntityPacket` (ID 124) and `ClientboundSetPlayerInventoryPacket` (ID 108) with `write` impls and ID constants. |
| `src/network/status.rs` | Add `dropped_items`, `inventory`, `inventory_state_id` to `PlaySessionState`; refactor `evaluate_block_loot` return type; register item entities at break time; add item age tick (Hook A) and pickup check (Hook B) to play loop. |

---

## 10. Out of Scope (Follow-Up)

- **Item persistence** across sessions (serialise `dropped_items` to NBT / region files).
- **Fortune and silk touch** — affects loot table outputs, not pickup mechanics.
- **Multi-player broadcast** — `TakeItemEntity` and `RemoveEntities` should be sent to all players tracking the entity, not just the picker. Requires a shared entity registry.
- **Optimised slot diffs** — only send `SetPlayerInventoryPacket` for slots that actually changed.
- **Stationary player pickup** — currently pickup only fires on movement packets. A true 20 Hz wall-clock tick loop would fix this.
- **Item merging** — nearby item entities of the same type merging together (Java: `ItemEntity.mergeWithNeighbours`).
