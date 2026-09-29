//! Tests for the `ServerExplosion` / `ExplosionDamageCalculator` port.

use super::*;
use crate::entity_collision::test_world::TestWorld;

/// A scripted `RandomSource`: `float` for `nextFloat`, ascending ints.
struct Fixed {
    float: f32,
    next: i32,
}

impl Fixed {
    fn new(float: f32) -> Self {
        Self { float, next: 0 }
    }
}

impl ExplosionRandom for Fixed {
    fn next_float(&mut self) -> f32 {
        self.float
    }

    fn next_double(&mut self) -> f64 {
        0.5
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        let value = self.next % bound;
        self.next += 1;
        value
    }
}

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}

fn state(name: &str) -> BlockStateModel {
    crate::fluid::block_state_model_from_name(name)
}

#[test]
fn level_explosion_interaction_picks_the_block_interaction_from_the_rules() {
    let rules = ExplosionRules::default();
    assert_eq!(
        LevelExplosionInteraction::None.block_interaction(&rules),
        ExplosionBlockInteraction::Keep
    );
    assert_eq!(
        LevelExplosionInteraction::Block.block_interaction(&rules),
        ExplosionBlockInteraction::DestroyWithDecay
    );
    assert_eq!(
        LevelExplosionInteraction::Mob.block_interaction(&rules),
        ExplosionBlockInteraction::DestroyWithDecay
    );
    // `tnt_explosion_drop_decay` defaults to false: TNT destroys without decay.
    assert_eq!(
        LevelExplosionInteraction::Tnt.block_interaction(&rules),
        ExplosionBlockInteraction::Destroy
    );
    assert_eq!(
        LevelExplosionInteraction::Trigger.block_interaction(&rules),
        ExplosionBlockInteraction::TriggerBlock
    );
}

#[test]
fn mob_explosions_keep_blocks_when_mob_griefing_is_off() {
    let rules = ExplosionRules {
        mob_griefing: false,
        ..ExplosionRules::default()
    };
    assert_eq!(
        LevelExplosionInteraction::Mob.block_interaction(&rules),
        ExplosionBlockInteraction::Keep
    );
    // TNT ignores mob griefing.
    assert_eq!(
        LevelExplosionInteraction::Tnt.block_interaction(&rules),
        ExplosionBlockInteraction::Destroy
    );
}

#[test]
fn drop_decay_rules_select_destroy_with_decay() {
    let rules = ExplosionRules {
        tnt_explosion_drop_decay: true,
        block_explosion_drop_decay: false,
        ..ExplosionRules::default()
    };
    assert_eq!(
        LevelExplosionInteraction::Tnt.block_interaction(&rules),
        ExplosionBlockInteraction::DestroyWithDecay
    );
    assert_eq!(
        LevelExplosionInteraction::Block.block_interaction(&rules),
        ExplosionBlockInteraction::Destroy
    );
}

#[test]
fn rules_are_read_from_the_live_game_rule_store() {
    let mut live = crate::game_rules::LiveGameRules::new(crate::game_rules::GameRules::new(false));
    live.set("tnt_explodes", "false", true).unwrap();
    live.set("tnt_explosion_drop_decay", "true", true).unwrap();
    let rules = ExplosionRules::read(&live);
    assert!(!rules.tnt_explodes);
    assert!(rules.tnt_explosion_drop_decay);
    assert!(rules.block_drops);
    assert!(rules.mob_griefing);
}

#[test]
fn blocklike_entities_are_affected_only_by_destroying_interactions() {
    assert!(interaction_affects_blocklike_entities(ExplosionBlockInteraction::Destroy));
    assert!(interaction_affects_blocklike_entities(
        ExplosionBlockInteraction::DestroyWithDecay
    ));
    assert!(!interaction_affects_blocklike_entities(ExplosionBlockInteraction::Keep));
    assert!(!interaction_affects_blocklike_entities(
        ExplosionBlockInteraction::TriggerBlock
    ));
}

#[test]
fn block_resistance_uses_the_block_and_fluid_maximum() {
    let calc = BlockCalculator::Default;
    assert_eq!(calc.block_resistance(&BlockStateModel::air()), None);
    assert_eq!(calc.block_resistance(&state("minecraft:stone")), Some(6.0));
    assert_eq!(calc.block_resistance(&state("minecraft:obsidian")), Some(1200.0));
    assert_eq!(calc.block_resistance(&state("minecraft:dirt")), Some(0.5));
    // Fluid resistance is 100 even where the block resistance is lower.
    assert_eq!(calc.block_resistance(&state("minecraft:water[level=0]")), Some(100.0));
    assert_eq!(
        calc.block_resistance(&state("minecraft:oak_slab[type=bottom,waterlogged=true]")),
        Some(100.0)
    );
    assert_eq!(calc.block_resistance(&state("minecraft:lava[level=3]")), Some(100.0));
    // Resistant blocks still win over the fluid.
    assert_eq!(
        calc.block_resistance(&state("minecraft:obsidian")).unwrap(),
        1200.0
    );
}

#[test]
fn the_used_portal_calculator_skips_nether_portals_only() {
    let portal = state("minecraft:nether_portal[axis=x]");
    assert!(BlockCalculator::Default.block_resistance(&portal).is_some());
    assert!(BlockCalculator::Default.should_block_explode(&portal));
    assert_eq!(BlockCalculator::UsedPortal.block_resistance(&portal), None);
    assert!(!BlockCalculator::UsedPortal.should_block_explode(&portal));
    assert_eq!(
        BlockCalculator::UsedPortal.block_resistance(&state("minecraft:stone")),
        Some(6.0)
    );
}

#[test]
fn world_bounds_are_the_overworld_build_height_and_horizontal_limit() {
    assert!(is_in_world_bounds(BlockPos { x: 0, y: -64, z: 0 }));
    assert!(is_in_world_bounds(BlockPos { x: 0, y: 319, z: 0 }));
    assert!(!is_in_world_bounds(BlockPos { x: 0, y: -65, z: 0 }));
    assert!(!is_in_world_bounds(BlockPos { x: 0, y: 320, z: 0 }));
    assert!(!is_in_world_bounds(BlockPos { x: 30_000_000, y: 0, z: 0 }));
    assert!(is_in_world_bounds(BlockPos { x: -30_000_000, y: 0, z: 0 }));
}

#[test]
fn rays_destroy_low_resistance_blocks_and_stop_at_obsidian() {
    // Centre one block above a 1-thick floor; every ray has 4.0 power.
    let mut stone = TestWorld::floor(63);
    let mut obsidian = TestWorld::default();
    for x in -8..8 {
        for z in -8..8 {
            obsidian.set(x, 63, z, "minecraft:obsidian");
        }
    }
    let centre = v(0.5, 65.0, 0.5);
    let broken = calculate_exploded(&stone, centre, 4.0, BlockCalculator::Default, &mut Fixed::new(0.5));
    assert!(broken.contains(&BlockPos { x: 0, y: 63, z: 0 }), "stone below the blast is destroyed");
    let survived =
        calculate_exploded(&obsidian, centre, 4.0, BlockCalculator::Default, &mut Fixed::new(0.5));
    // Air positions are exploded too (`toBlow.size()` counts them); solid
    // obsidian never is.
    assert!(
        survived.iter().all(|pos| obsidian.state_at(*pos).is_air()),
        "obsidian absorbs every ray"
    );
    assert!(!survived.is_empty(), "the blast still reaches the surrounding air");
    stone.blocks.clear();
    assert!(calculate_exploded(&stone, centre, 4.0, BlockCalculator::Default, &mut Fixed::new(0.5))
        .iter()
        .all(|pos| stone.state_at(*pos).is_air()));
}

#[test]
fn explosions_never_reach_outside_the_build_height() {
    let mut world = TestWorld::default();
    world.set(0, -64, 0, "minecraft:stone");
    world.set(0, -65, 0, "minecraft:stone");
    let broken = calculate_exploded(
        &world,
        v(0.5, -63.5, 0.5),
        4.0,
        BlockCalculator::Default,
        &mut Fixed::new(0.5),
    );
    assert!(broken.contains(&BlockPos { x: 0, y: -64, z: 0 }));
    assert!(!broken.contains(&BlockPos { x: 0, y: -65, z: 0 }));
}

#[test]
fn shuffle_is_fisher_yates_from_the_back() {
    let mut list = vec![0, 1, 2, 3];
    // Fixed ints: i=4 -> 0, i=3 -> 1, i=2 -> 2 % 2 = 0.
    shuffle(&mut list, &mut Fixed::new(0.0));
    // swap(3,0) -> [3,1,2,0]; swap(2,1) -> [3,2,1,0]; swap(1,0) -> [2,3,1,0].
    assert_eq!(list, vec![2, 3, 1, 0]);
    let mut single = vec![9];
    shuffle(&mut single, &mut Fixed::new(0.0));
    assert_eq!(single, vec![9]);
}

fn entity_box(x: f64, y: f64, z: f64) -> Aabb {
    Aabb::new(x - 0.3, y, z - 0.3, x + 0.3, y + 1.8, z + 0.3)
}

#[test]
fn an_unobstructed_entity_is_fully_exposed() {
    let world = TestWorld::floor(63);
    let exposure = get_seen_percent(&world, v(0.5, 66.0, 0.5), &entity_box(3.5, 64.0, 0.5));
    assert_eq!(exposure, 1.0);
}

#[test]
fn a_full_wall_hides_the_entity() {
    let mut world = TestWorld::floor(63);
    for y in 64..70 {
        for z in -3..4 {
            world.set(2, y, z, "minecraft:stone");
        }
    }
    let exposure = get_seen_percent(&world, v(0.5, 65.0, 0.5), &entity_box(4.5, 64.0, 0.5));
    assert_eq!(exposure, 0.0);
}

#[test]
fn a_low_wall_partially_covers_a_tall_entity() {
    let mut world = TestWorld::floor(63);
    for z in -3..4 {
        world.set(2, 64, z, "minecraft:stone");
        world.set(2, 65, z, "minecraft:stone");
    }
    let exposure = get_seen_percent(&world, v(0.5, 68.0, 0.5), &entity_box(4.5, 64.0, 0.5));
    assert!(exposure > 0.0 && exposure < 1.0, "partly exposed: {exposure}");
}

#[test]
fn impact_matches_the_java_damage_and_knockback_formulas() {
    // Radius 4 (TNT): doubleRadius 8. An entity 4 blocks away has dist 0.5.
    let target = EntityTarget {
        position: v(4.0, 0.0, 0.0),
        origin: v(4.0, 0.0, 0.0),
        knockback_resistance: 0.0,
    };
    let impact = entity_impact(v(0.0, 0.0, 0.0), 4.0, &target, 1.0, 1.0).unwrap();
    // pow = 0.5 -> (0.25 + 0.5) / 2 * 7 * 8 + 1 = 22.
    assert!((impact.damage - 22.0).abs() < 1.0E-5, "{}", impact.damage);
    assert!((impact.knockback.x - 0.5).abs() < 1.0E-12);
    assert_eq!((impact.knockback.y, impact.knockback.z), (0.0, 0.0));
}

#[test]
fn knockback_resistance_and_exposure_scale_the_push_but_not_the_direction() {
    let target = EntityTarget {
        position: v(0.0, 0.0, 4.0),
        origin: v(0.0, 2.0, 4.0),
        knockback_resistance: 0.5,
    };
    let impact = entity_impact(v(0.0, 0.0, 0.0), 4.0, &target, 0.5, 1.0).unwrap();
    let direction_length = (2.0_f64 * 2.0 + 4.0 * 4.0).sqrt();
    // power = (1 - 0.5) * 0.5 * 1 * (1 - 0.5) = 0.125 along (0, 2, 4)/|.|.
    assert!((impact.knockback.y - 0.125 * 2.0 / direction_length).abs() < 1.0E-12);
    assert!((impact.knockback.z - 0.125 * 4.0 / direction_length).abs() < 1.0E-12);
    // Damage uses the feet distance (dist 0.5) and exposure 0.5: pow = 0.25.
    assert!((impact.damage - ((0.0625 + 0.25) / 2.0 * 56.0 + 1.0) as f32).abs() < 1.0E-5);
}

#[test]
fn entities_beyond_twice_the_radius_are_untouched() {
    let target = EntityTarget {
        position: v(8.5, 0.0, 0.0),
        origin: v(8.5, 0.0, 0.0),
        knockback_resistance: 0.0,
    };
    assert!(entity_impact(v(0.0, 0.0, 0.0), 4.0, &target, 1.0, 1.0).is_none());
    assert!((explosion_distance(v(0.0, 0.0, 0.0), 4.0, v(8.0, 0.0, 0.0)) - 1.0).abs() < 1.0E-12);
}

#[test]
fn drops_of_one_item_merge_into_stacks_of_at_most_sixteen() {
    let mut stacks = Vec::new();
    let first = BlockPos { x: 0, y: 0, z: 0 };
    let second = BlockPos { x: 5, y: 0, z: 0 };
    add_or_append_stack(&mut stacks, "minecraft:dirt", 10, first);
    add_or_append_stack(&mut stacks, "minecraft:dirt", 10, second);
    // 10 + 6 fills the first collector to 16; the remaining 4 start a new one
    // at the second block's position.
    assert_eq!(stacks.len(), 2);
    assert_eq!((stacks[0].count, stacks[0].pos), (16, first));
    assert_eq!((stacks[1].count, stacks[1].pos), (4, second));
    add_or_append_stack(&mut stacks, "minecraft:dirt", 3, first);
    assert_eq!(stacks[1].count, 7, "the next drop tops up the partial stack");
}

#[test]
fn different_items_and_unstackables_never_merge() {
    let mut stacks = Vec::new();
    let pos = BlockPos { x: 0, y: 0, z: 0 };
    add_or_append_stack(&mut stacks, "minecraft:dirt", 1, pos);
    add_or_append_stack(&mut stacks, "minecraft:cobblestone", 1, pos);
    add_or_append_stack(&mut stacks, "minecraft:diamond_pickaxe", 1, pos);
    add_or_append_stack(&mut stacks, "minecraft:diamond_pickaxe", 1, pos);
    assert_eq!(stacks.len(), 4);
}
