use super::*;

fn player_at(x: f64, y: f64, z: f64) -> XpOrbPlayer<'static> {
    XpOrbPlayer {
        id: "p1",
        x,
        y,
        z,
        eye_height: 1.62,
        is_spectator: false,
        is_dead: false,
    }
}

fn orb_at(id: i32, value: i32, x: f64, y: f64, z: f64) -> XpOrbEntity {
    let mut orb = XpOrbEntity::spawn(id, (x, y, z), (0.0, 0.0, 0.0), value, &mut XpOrbRandom::new(1));
    orb.vel_x = 0.0;
    orb.vel_y = 0.0;
    orb.vel_z = 0.0;
    orb
}

#[test]
fn award_splits_amount_into_vanilla_orb_values() {
    // ExperienceOrb.award: 100 -> 73, 17, 7, 3 (greedy getExperienceValue).
    let mut store = WorldItemEntities::new();
    let spawned = store.award_experience((0.0, 64.0, 0.0), 100, &mut XpOrbRandom::new(7));
    let mut values: Vec<i32> = spawned.iter().map(XpOrbEntity::value).collect();
    values.sort_unstable();
    assert_eq!(values, vec![3, 7, 17, 73]);
    assert_eq!(store.xp_orbs.len(), 4);
    let ids: std::collections::BTreeSet<i32> = spawned.iter().map(XpOrbEntity::entity_id).collect();
    assert_eq!(ids.len(), 4, "each orb gets a fresh entity id");
}

#[test]
fn award_of_nothing_spawns_nothing() {
    let mut store = WorldItemEntities::new();
    assert!(store.award_experience((0.0, 0.0, 0.0), 0, &mut XpOrbRandom::new(1)).is_empty());
    assert!(store.xp_orbs.is_empty());
}

#[test]
fn spawn_velocity_matches_java_ranges() {
    for seed in 0..50 {
        let orb = XpOrbEntity::spawn(2, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1, &mut XpOrbRandom::new(seed));
        assert!((-0.2..0.2).contains(&orb.vel_x));
        assert!((0.0..0.4).contains(&orb.vel_y));
        assert!((-0.2..0.2).contains(&orb.vel_z));
    }
}

#[test]
fn rough_direction_flips_opposing_velocity_and_offsets_position() {
    // roughly.dot(randomMovement) < 0 -> randomMovement is negated; position moves by
    // roughly.normalize() * size * 0.5.
    let orb = XpOrbEntity::spawn(2, (0.0, 0.0, 0.0), (10.0, 0.0, 0.0), 1, &mut XpOrbRandom::new(3));
    assert!(orb.vel_x >= 0.0);
    assert!((orb.x - 0.25).abs() < 1e-12);
    assert_eq!((orb.y, orb.z), (0.0, 0.0));
}

#[test]
fn award_folds_into_existing_orb_of_the_same_merge_group() {
    // Java tryMergeToExisting: same value + (id - roll) % 40 == 0 inside the 1-block cube.
    let mut store = WorldItemEntities::new();
    let first = store.award_experience((0.0, 64.0, 0.0), 1, &mut XpOrbRandom::new(1));
    assert_eq!(first.len(), 1);
    let group_id = first[0].entity_id();
    // Find a seed whose first nextInt(40) roll equals the orb's id modulo 40.
    let seed = (0..10_000)
        .find(|seed| XpOrbRandom::new(*seed).next_int(40) == group_id.rem_euclid(40))
        .expect("some seed rolls the merge group");
    let again = store.award_experience((0.2, 64.0, 0.2), 1, &mut XpOrbRandom::new(seed));
    assert!(again.is_empty(), "merged into the existing orb");
    assert_eq!(store.xp_orbs.len(), 1);
    assert_eq!(store.xp_orbs[0].orb.count, 2);
    assert_eq!(store.xp_orbs[0].orb.age, 0);
}

#[test]
fn award_does_not_merge_across_different_values_or_far_away() {
    let mut store = WorldItemEntities::new();
    store.award_experience((0.0, 64.0, 0.0), 1, &mut XpOrbRandom::new(1));
    store.award_experience((0.0, 64.0, 0.0), 3, &mut XpOrbRandom::new(1));
    store.award_experience((5.0, 64.0, 0.0), 1, &mut XpOrbRandom::new(1));
    assert_eq!(store.xp_orbs.len(), 3);
}

#[test]
fn orb_expires_at_lifetime() {
    let mut orbs = vec![orb_at(2, 1, 0.0, 0.0, 0.0)];
    // ExperienceOrb.LIFETIME = 6000: the orb is discarded when age reaches it.
    orbs[0].orb.age = 5999;
    let result = tick_orbs(&mut orbs, &[]);
    assert_eq!(result.removed, vec![2]);
    assert!(orbs.is_empty());
}

#[test]
fn orb_follows_nearest_player_within_eight_blocks_only() {
    let mut orbs = vec![orb_at(2, 1, 0.0, 64.0, 0.0), orb_at(3, 1, 100.0, 64.0, 0.0)];
    tick_orbs(&mut orbs, &[player_at(4.0, 64.0, 0.0)]);
    assert!(orbs[0].vel_x > 0.0, "orb 4 blocks away is pulled toward the player");
    assert_eq!(orbs[0].following.as_deref(), Some("p1"));
    assert_eq!(orbs[1].vel_x, 0.0, "orb 100 blocks away is not");
    assert_eq!(orbs[1].following, None);
}

#[test]
fn orb_ignores_spectators_and_dead_players() {
    let mut orbs = vec![orb_at(2, 1, 0.0, 64.0, 0.0)];
    let mut spectator = player_at(2.0, 64.0, 0.0);
    spectator.is_spectator = true;
    tick_orbs(&mut orbs, &[spectator]);
    assert_eq!(orbs[0].following, None);
    let mut dead = player_at(2.0, 64.0, 0.0);
    dead.is_dead = true;
    tick_orbs(&mut orbs, &[dead]);
    assert_eq!(orbs[0].following, None);
}

#[test]
fn follow_pull_matches_java_power_formula() {
    // delta = (4, 1.62/2 - 0, 0) -> power = 1 - |delta| / 8; pull = normalize * power^2 * 0.1.
    let mut orbs = vec![orb_at(2, 1, 0.0, 64.0, 0.0)];
    tick_orbs(&mut orbs, &[player_at(4.0, 64.0, 0.0)]);
    let delta = (4.0_f64, 0.81_f64);
    let len = (delta.0 * delta.0 + delta.1 * delta.1).sqrt();
    let power = 1.0 - len / 8.0;
    let expected_vx = delta.0 / len * power * power * 0.1;
    // The tick applies the 0.98 drag after moving.
    assert!((orbs[0].vel_x - expected_vx * 0.98).abs() < 1e-12);
}

#[test]
fn nearby_same_value_orbs_merge_on_tick_count_one_mod_twenty() {
    // Both orbs must share a merge group ((id - id') % 40 == 0), so use ids 2 and 42.
    let mut orbs = vec![orb_at(2, 1, 0.0, 64.0, 0.0), orb_at(42, 1, 0.3, 64.0, 0.0)];
    orbs[0].orb.age = 10;
    orbs[1].orb.age = 3;
    let result = tick_orbs(&mut orbs, &[]);
    // First orb ticks first (tick_count 1 -> scan) and absorbs the second.
    assert_eq!(result.removed, vec![42]);
    assert_eq!(orbs.len(), 1);
    assert_eq!(orbs[0].orb.count, 2);
    // The absorbed orb had not aged yet this tick, so Math.min(age, orb.age) keeps its 3.
    assert_eq!(orbs[0].orb.age, 3);
}

#[test]
fn orbs_in_different_merge_groups_do_not_merge() {
    let mut orbs = vec![orb_at(2, 1, 0.0, 64.0, 0.0), orb_at(3, 1, 0.3, 64.0, 0.0)];
    let result = tick_orbs(&mut orbs, &[]);
    assert!(result.removed.is_empty());
    assert_eq!(orbs.len(), 2);
}
