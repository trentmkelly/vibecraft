//! Tests of the player-side `TntBlock` behaviour.

use super::chunk_d_2::evaluate_block_loot;
use super::tnt_interaction_live::tnt_will_destroy;
use super::tnt_live_tests::World;
use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_update::BlockPos;

fn broken(name: &str) -> BlockStateModel {
    crate::fluid::block_state_model_from_name(name)
}

#[test]
fn breaking_unstable_tnt_in_survival_lights_it() {
    let world = World::new("unstable");
    tnt_will_destroy(
        &world.items,
        &world.cache.level_random,
        GameMode::Survival,
        &broken("minecraft:tnt[unstable=true]"),
        BlockPos { x: 1, y: 70, z: 1 },
    );
    assert_eq!(world.tnt_count(), 1);
    assert_eq!(lock_status_mutex(&world.items).primed_tnts[0].owner, None);
}

#[test]
fn stable_tnt_and_creative_breakers_do_not_light_it() {
    let world = World::new("stable");
    let pos = BlockPos { x: 1, y: 70, z: 1 };
    tnt_will_destroy(&world.items, &world.cache.level_random, GameMode::Survival, &broken("minecraft:tnt[unstable=false]"), pos);
    tnt_will_destroy(&world.items, &world.cache.level_random, GameMode::Creative, &broken("minecraft:tnt[unstable=true]"), pos);
    tnt_will_destroy(&world.items, &world.cache.level_random, GameMode::Survival, &broken("minecraft:stone"), pos);
    assert_eq!(world.tnt_count(), 0);
}

#[test]
fn unstable_tnt_is_not_lit_when_tnt_explodes_is_off() {
    let world = World::new("unstable-off");
    lock_status_mutex(&world.items).explosion_rules.tnt_explodes = false;
    tnt_will_destroy(
        &world.items,
        &world.cache.level_random,
        GameMode::Survival,
        &broken("minecraft:tnt[unstable=true]"),
        BlockPos { x: 1, y: 70, z: 1 },
    );
    assert_eq!(world.tnt_count(), 0);
}

#[test]
fn tnt_loot_drops_the_block_unless_unstable() {
    // data/minecraft/loot_table/blocks/tnt.json.
    assert_eq!(
        evaluate_block_loot("minecraft:tnt[unstable=false]", 7),
        vec![("minecraft:tnt", 1)]
    );
    assert!(evaluate_block_loot("minecraft:tnt[unstable=true]", 7).is_empty());
}
