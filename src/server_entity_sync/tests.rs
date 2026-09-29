//! Tests for the `ServerEntity.sendChanges` decision logic.

use super::*;

fn at(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}

#[test]
fn an_unmoved_entity_sends_nothing_between_forced_updates() {
    let mut sync = EntitySync::new(10, true, at(0.0, 64.0, 0.0), Vec3::ZERO, false);
    // Tick 0 matches `tickCount % updateInterval == 0` and `% 60 == 0`: the
    // forced relative update goes out (with zero deltas).
    assert_eq!(
        sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, false).position,
        Some(PositionUpdate::Relative([0, 0, 0]))
    );
    for _ in 1..10 {
        assert_eq!(
            sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, false),
            SyncOutput::default()
        );
    }
    // Tick 10 is an update tick but neither moved nor a 60-tick boundary.
    assert_eq!(
        sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, false),
        SyncOutput::default()
    );
}

#[test]
fn dirty_entity_data_runs_the_position_check_every_tick() {
    let mut sync = EntitySync::new(10, true, at(0.0, 64.0, 0.0), Vec3::ZERO, false);
    sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, true);
    // Tick 1 is not an update tick, but dirty data forces the check: the
    // entity moved, so a relative move is sent with the 1/4096 encoding.
    let output = sync.send_changes(at(0.5, 64.0, 0.0), Vec3::ZERO, false, false, true);
    assert_eq!(output.position, Some(PositionUpdate::Relative([2048, 0, 0])));
}

#[test]
fn leaving_or_touching_the_ground_forces_a_full_position_sync() {
    let mut sync = EntitySync::new(10, true, at(0.0, 64.0, 0.0), Vec3::ZERO, false);
    sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, true);
    let output = sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, true, false, true);
    assert_eq!(output.position, Some(PositionUpdate::Sync));
}

#[test]
fn a_move_beyond_the_short_range_falls_back_to_a_full_sync() {
    let mut sync = EntitySync::new(10, true, at(0.0, 64.0, 0.0), Vec3::ZERO, false);
    sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, true);
    // 9 blocks * 4096 = 36864 > i16::MAX.
    let output = sync.send_changes(at(9.0, 64.0, 0.0), Vec3::ZERO, false, false, true);
    assert_eq!(output.position, Some(PositionUpdate::Sync));
}

#[test]
fn motion_changes_are_sent_once_when_tracking_deltas() {
    let mut sync = EntitySync::new(10, true, at(0.0, 64.0, 0.0), Vec3::ZERO, false);
    let push = at(0.0, 0.2, 0.0);
    sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, true);
    let sent = sync.send_changes(at(0.0, 64.0, 0.0), push, false, false, true);
    assert_eq!(sent.motion, Some(push));
    assert_eq!(sync.last_sent_movement(), push);
    let repeat = sync.send_changes(at(0.0, 64.0, 0.0), push, false, false, true);
    assert_eq!(repeat.motion, None);
}

#[test]
fn a_stop_is_reported_even_when_the_squared_difference_is_tiny() {
    let mut sync = EntitySync::new(10, true, at(0.0, 64.0, 0.0), at(0.0, 0.0001, 0.0), false);
    let output = sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, true);
    assert_eq!(output.motion, Some(Vec3::ZERO));
}

#[test]
fn needs_sync_forces_an_update_off_the_interval() {
    let mut sync = EntitySync::new(10, false, at(0.0, 64.0, 0.0), Vec3::ZERO, false);
    sync.send_changes(at(0.0, 64.0, 0.0), Vec3::ZERO, false, false, false);
    let push = at(1.0, 0.0, 0.0);
    // Without trackDelta motion is only sent when `needsSync` is set.
    let quiet = sync.send_changes(at(0.0, 64.0, 0.0), push, false, false, false);
    assert_eq!(quiet.motion, None);
    let forced = sync.send_changes(at(0.0, 64.0, 0.0), push, false, true, false);
    assert_eq!(forced.motion, Some(push));
}

#[test]
fn spawn_uses_the_recorded_base_and_movement() {
    let sync = EntitySync::new(10, true, at(1.0, 2.0, 3.0), at(0.1, 0.2, 0.3), false);
    assert_eq!(sync.position_base(), at(1.0, 2.0, 3.0));
    assert_eq!(sync.last_sent_movement(), at(0.1, 0.2, 0.3));
}
