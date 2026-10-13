use super::*;

#[test]
fn armadillo_spawn_blocks_match_the_vanilla_tag_expansion() {
    for block in [
        "minecraft:grass_block",
        "minecraft:red_sand",
        "minecraft:coarse_dirt",
        "minecraft:terracotta",
        "minecraft:white_terracotta",
        "minecraft:yellow_terracotta",
        "minecraft:orange_terracotta",
        "minecraft:red_terracotta",
        "minecraft:brown_terracotta",
        "minecraft:light_gray_terracotta",
    ] {
        assert!(
            armadillo_spawnable_on(block),
            "expected {block} to be valid"
        );
    }
    for block in [
        "minecraft:magenta_terracotta",
        "minecraft:light_blue_terracotta",
        "minecraft:lime_terracotta",
        "minecraft:pink_terracotta",
        "minecraft:gray_terracotta",
        "minecraft:cyan_terracotta",
        "minecraft:purple_terracotta",
        "minecraft:blue_terracotta",
        "minecraft:green_terracotta",
        "minecraft:black_terracotta",
    ] {
        assert!(
            !armadillo_spawnable_on(block),
            "expected {block} to be rejected"
        );
    }
}

#[test]
fn armadillo_peek_countdown_and_danger_edges_preserve_java_rng_order() {
    let mut memory = ArmadilloBallUpMemory::default();
    let mut draws = 0;
    let mut sample = || {
        draws += 1;
        100
    };
    assert_eq!(
        memory.tick(ArmadilloStateModel::Scared, 0, true, 80, &mut sample),
        ArmadilloBallUpTickPlan::None
    );
    assert_eq!(memory.next_peek_timer, 150);
    for _ in 0..149 {
        assert_eq!(
            memory.tick(ArmadilloStateModel::Scared, 0, true, 80, &mut sample),
            ArmadilloBallUpTickPlan::None
        );
    }
    assert_eq!(memory.next_peek_timer, 1);
    assert!(matches!(
        memory.tick(ArmadilloStateModel::Scared, 0, true, 80, &mut sample),
        ArmadilloBallUpTickPlan::Peek { event: 64, .. }
    ));
    assert_eq!(memory.next_peek_timer, 150);
    // 75 is the first non-danger value, not 74. Both edges resample.
    memory.tick(ArmadilloStateModel::Scared, 0, true, 75, &mut sample);
    assert!(!memory.danger_was_around);
    assert_eq!(memory.next_peek_timer, 150);
    memory.tick(ArmadilloStateModel::Scared, 0, true, 76, &mut sample);
    assert!(memory.danger_was_around);
    assert_eq!(memory.next_peek_timer, 150);
    assert_eq!(draws, 4);
}

#[test]
fn armadillo_rolling_completion_skips_danger_update_but_decrements_timer() {
    let mut memory = ArmadilloBallUpMemory {
        next_peek_timer: 2,
        danger_was_around: false,
    };
    let mut draws = 0;
    assert_eq!(
        memory.tick(ArmadilloStateModel::Rolling, 11, false, 80, || {
            draws += 1;
            400
        }),
        ArmadilloBallUpTickPlan::SwitchToScared { sound: None }
    );
    assert_eq!(draws, 0);
    assert_eq!(memory.next_peek_timer, 1);
    assert!(!memory.danger_was_around);
    assert_eq!(
        memory.tick(ArmadilloStateModel::Scared, 0, true, 80, || {
            draws += 1;
            400
        }),
        ArmadilloBallUpTickPlan::None
    );
    assert_eq!(draws, 1);
    assert_eq!(memory.next_peek_timer, 450);
}

#[test]
fn armadillo_airborne_peek_waits_and_unrolling_uses_strict_boundaries() {
    let mut memory = ArmadilloBallUpMemory {
        next_peek_timer: 1,
        danger_was_around: true,
    };
    let mut draws = 0;
    assert_eq!(
        memory.tick(ArmadilloStateModel::Scared, 0, false, 80, || {
            draws += 1;
            100
        }),
        ArmadilloBallUpTickPlan::None
    );
    assert_eq!(draws, 0);
    assert_eq!(memory.next_peek_timer, 0);
    assert!(matches!(
        memory.tick(ArmadilloStateModel::Scared, 0, true, 80, || 100),
        ArmadilloBallUpTickPlan::Peek { .. }
    ));
    assert_eq!(
        memory.tick(ArmadilloStateModel::Scared, 0, true, 30, || 100),
        ArmadilloBallUpTickPlan::None
    );
    assert!(matches!(
        memory.tick(ArmadilloStateModel::Scared, 0, true, 29, || 100),
        ArmadilloBallUpTickPlan::StartUnrolling { .. }
    ));
    assert_eq!(
        memory.tick(ArmadilloStateModel::Unrolling, 0, true, 30, || 100),
        ArmadilloBallUpTickPlan::None
    );
    assert_eq!(
        memory.tick(ArmadilloStateModel::Unrolling, 0, true, 31, || 100),
        ArmadilloBallUpTickPlan::ReturnToScared
    );
}

#[test]
fn armadillo_scute_timer_short_circuit_and_loot_failure_match_java() {
    for timer in [1, 0, -1, i32::MIN] {
        let dead = armadillo_scute_drop_tick(false, timer, true, true, 0);
        assert_eq!(dead.next_scute_time, timer);
        assert_eq!(dead.loot_table, None);
    }
    let disabled = armadillo_scute_drop_tick(true, 0, false, true, 0);
    assert_eq!(disabled.next_scute_time, -1);
    assert_eq!(disabled.loot_table, None);
    let empty_loot = armadillo_scute_drop_tick(true, -1, true, false, 5999);
    assert_eq!(empty_loot.next_scute_time, 11999);
    assert_eq!(
        empty_loot.loot_table,
        Some("minecraft:gameplay/armadillo_shed")
    );
    assert_eq!(empty_loot.sound, None);
    assert_eq!(empty_loot.game_event, None);
    let wrapped = armadillo_scute_drop_tick(true, i32::MIN, true, true, 0);
    assert_eq!(wrapped.next_scute_time, i32::MAX);
    assert_eq!(wrapped.loot_table, None);
}
