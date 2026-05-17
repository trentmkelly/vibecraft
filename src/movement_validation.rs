#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovePacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub has_position: bool,
    pub has_rotation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerMovementState {
    pub current: Vec3,
    pub first_good: Vec3,
    pub last_good: Vec3,
    pub velocity: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub tick_count: i32,
    pub known_move_packet_count: i32,
    pub received_move_packet_count: i32,
    pub awaiting_teleport: Option<AwaitingTeleport>,
    pub client_loaded: bool,
    pub sleeping: bool,
    pub passenger: bool,
    pub changing_dimension: bool,
    pub creative: bool,
    pub spectator: bool,
    pub post_impulse_grace: bool,
    pub fall_flying: bool,
    pub auto_spin_attack: bool,
    pub singleplayer_owner: bool,
    pub movement_check_enabled: bool,
    pub elytra_movement_check_enabled: bool,
    pub allow_flight: bool,
    pub mayfly: bool,
    pub levitation: bool,
    pub no_physics: bool,
    pub vertical_collision_below: bool,
    pub no_blocks_around: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AwaitingTeleport {
    pub target: Vec3,
    pub sent_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidationEnvironment {
    pub collision_remainder: Vec3,
    pub old_box_still_clear: bool,
    pub collides_with_new_blocks: bool,
    pub tick_rate_runs_normally: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleMovementState {
    pub current: Vec3,
    pub first_good: Vec3,
    pub last_good: Vec3,
    pub velocity: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub controlling_player: bool,
    pub same_vehicle_as_last_tick: bool,
    pub singleplayer_owner: bool,
    pub allow_flight: bool,
    pub flying_vehicle: bool,
    pub no_gravity: bool,
    pub vertical_collision_below: bool,
    pub no_blocks_around: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingState {
    pub client_is_floating: bool,
    pub above_ground_ticks: i32,
    pub gravity: f64,
    pub sleeping: bool,
    pub passenger: bool,
    pub dead_or_dying: bool,
}

impl Default for ValidationEnvironment {
    fn default() -> Self {
        Self {
            collision_remainder: Vec3::ZERO,
            old_box_still_clear: true,
            collides_with_new_blocks: false,
            tick_rate_runs_normally: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementDecision {
    Accept(AcceptedMove),
    AcceptVehicle(AcceptedVehicleMove),
    RotateWhileAwaitingTeleport { y_rot: f32, x_rot: f32 },
    ResendAwaitingTeleport { target: Vec3 },
    TeleportBack(TeleportCorrection),
    Disconnect(&'static str),
    IgnoreUntilLoaded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceptedMove {
    pub target: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub client_delta: Vec3,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub client_is_floating: bool,
    pub reset_impulse_context: bool,
    pub reset_fall_distance: bool,
    pub updated_move_packet_count: i32,
    pub updated_last_good: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceptedVehicleMove {
    pub target: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub client_delta: Vec3,
    pub client_vehicle_is_floating: bool,
    pub updated_last_good: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeleportCorrection {
    pub target: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub reason: CorrectionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionReason {
    SleepingMovedTooFar,
    MovedTooQuickly,
    VehicleMovedTooQuickly,
    MovedWrongly,
    VehicleMovedWrongly,
    NewCollision,
}

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

    pub fn length_sqr(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

impl ServerMovementState {
    pub fn at(x: f64, y: f64, z: f64) -> Self {
        let position = Vec3 { x, y, z };
        Self {
            current: position,
            first_good: position,
            last_good: position,
            velocity: Vec3::ZERO,
            y_rot: 0.0,
            x_rot: 0.0,
            tick_count: 1,
            known_move_packet_count: 0,
            received_move_packet_count: 0,
            awaiting_teleport: None,
            client_loaded: true,
            sleeping: false,
            passenger: false,
            changing_dimension: false,
            creative: false,
            spectator: false,
            post_impulse_grace: false,
            fall_flying: false,
            auto_spin_attack: false,
            singleplayer_owner: false,
            movement_check_enabled: true,
            elytra_movement_check_enabled: true,
            allow_flight: false,
            mayfly: false,
            levitation: false,
            no_physics: false,
            vertical_collision_below: true,
            no_blocks_around: false,
        }
    }
}

impl VehicleMovementState {
    pub fn at(x: f64, y: f64, z: f64) -> Self {
        let position = Vec3 { x, y, z };
        Self {
            current: position,
            first_good: position,
            last_good: position,
            velocity: Vec3::ZERO,
            y_rot: 0.0,
            x_rot: 0.0,
            controlling_player: true,
            same_vehicle_as_last_tick: true,
            singleplayer_owner: false,
            allow_flight: false,
            flying_vehicle: false,
            no_gravity: false,
            vertical_collision_below: true,
            no_blocks_around: false,
        }
    }
}

pub fn validate_player_move(
    state: ServerMovementState,
    packet: MovePacket,
    env: ValidationEnvironment,
) -> MovementDecision {
    let raw_x = packet_value_f64(packet.x, state.current.x, packet.has_position);
    let raw_y = packet_value_f64(packet.y, state.current.y, packet.has_position);
    let raw_z = packet_value_f64(packet.z, state.current.z, packet.has_position);
    let raw_y_rot = packet_value_f32(packet.y_rot, state.y_rot, packet.has_rotation);
    let raw_x_rot = packet_value_f32(packet.x_rot, state.x_rot, packet.has_rotation);

    if contains_invalid_values(raw_x, raw_y, raw_z, raw_y_rot, raw_x_rot) {
        return MovementDecision::Disconnect("multiplayer.disconnect.invalid_player_movement");
    }
    if !state.client_loaded {
        return MovementDecision::IgnoreUntilLoaded;
    }

    let target_y_rot = wrap_degrees(raw_y_rot);
    let target_x_rot = wrap_degrees(raw_x_rot);
    if let Some(awaiting) = state.awaiting_teleport {
        if state.tick_count - awaiting.sent_tick > 20 {
            return MovementDecision::ResendAwaitingTeleport {
                target: awaiting.target,
            };
        }
        return MovementDecision::RotateWhileAwaitingTeleport {
            y_rot: target_y_rot,
            x_rot: target_x_rot,
        };
    }

    let target = Vec3 {
        x: clamp_horizontal(raw_x),
        y: clamp_vertical(raw_y),
        z: clamp_horizontal(raw_z),
    };

    if state.passenger {
        return MovementDecision::Accept(AcceptedMove {
            target: state.current,
            y_rot: target_y_rot,
            x_rot: target_x_rot,
            client_delta: Vec3::ZERO,
            on_ground: packet.on_ground,
            horizontal_collision: packet.horizontal_collision,
            client_is_floating: false,
            reset_impulse_context: packet.on_ground,
            reset_fall_distance: false,
            updated_move_packet_count: state.received_move_packet_count,
            updated_last_good: state.last_good,
        });
    }

    let first_delta = sub(target, state.first_good);
    let moved_from_first = first_delta.length_sqr();
    if state.sleeping {
        if moved_from_first > 1.0 {
            return MovementDecision::TeleportBack(TeleportCorrection {
                target: state.current,
                y_rot: target_y_rot,
                x_rot: target_x_rot,
                reason: CorrectionReason::SleepingMovedTooFar,
            });
        }
        return accept(
            state,
            packet,
            target,
            target_y_rot,
            target_x_rot,
            Vec3::ZERO,
            false,
        );
    }

    let mut received_count = state.received_move_packet_count;
    if env.tick_rate_runs_normally {
        received_count += 1;
        let raw_delta_packets = received_count - state.known_move_packet_count;
        let delta_packets = if raw_delta_packets > 5 {
            1
        } else {
            raw_delta_packets.max(1)
        };
        if should_check_player_movement(state) {
            let meters_per_tick = if state.fall_flying { 300.0 } else { 100.0 };
            if moved_from_first - state.velocity.length_sqr()
                > meters_per_tick * f64::from(delta_packets)
            {
                return MovementDecision::TeleportBack(TeleportCorrection {
                    target: state.current,
                    y_rot: state.y_rot,
                    x_rot: state.x_rot,
                    reason: CorrectionReason::MovedTooQuickly,
                });
            }
        }
    }

    let requested_delta = sub(target, state.last_good);
    let moved_upwards = requested_delta.y > 0.0;
    let remainder = env.collision_remainder;
    let adjusted_remainder = Vec3 {
        x: remainder.x,
        y: if remainder.y > -0.5 || remainder.y < 0.5 {
            0.0
        } else {
            remainder.y
        },
        z: remainder.z,
    };
    let remainder_sqr = adjusted_remainder.length_sqr();
    let moved_wrongly = !state.changing_dimension
        && remainder_sqr > 0.0625
        && !state.sleeping
        && !state.creative
        && !state.spectator
        && !state.post_impulse_grace;
    if !state.no_physics
        && ((moved_wrongly && env.old_box_still_clear) || env.collides_with_new_blocks)
    {
        return MovementDecision::TeleportBack(TeleportCorrection {
            target: state.current,
            y_rot: target_y_rot,
            x_rot: target_x_rot,
            reason: if moved_wrongly {
                CorrectionReason::MovedWrongly
            } else {
                CorrectionReason::NewCollision
            },
        });
    }

    let client_delta = sub(target, state.current);
    let client_is_floating = requested_delta.y >= -0.03125
        && !state.vertical_collision_below
        && !state.spectator
        && !state.allow_flight
        && !state.mayfly
        && !state.levitation
        && !state.fall_flying
        && !state.auto_spin_attack
        && state.no_blocks_around;

    MovementDecision::Accept(AcceptedMove {
        target,
        y_rot: target_y_rot,
        x_rot: target_x_rot,
        client_delta,
        on_ground: packet.on_ground,
        horizontal_collision: packet.horizontal_collision,
        client_is_floating,
        reset_impulse_context: packet.on_ground
            || state.spectator
            || state.fall_flying
            || state.auto_spin_attack,
        reset_fall_distance: moved_upwards,
        updated_move_packet_count: received_count,
        updated_last_good: target,
    })
}

pub fn maximum_flying_ticks(gravity: f64) -> i32 {
    if gravity < 1.0E-5 {
        i32::MAX
    } else {
        (80.0 * (0.08 / gravity).max(1.0)).ceil() as i32
    }
}

pub fn validate_vehicle_move(
    state: VehicleMovementState,
    packet: MovePacket,
    env: ValidationEnvironment,
) -> MovementDecision {
    if contains_invalid_values(packet.x, packet.y, packet.z, packet.y_rot, packet.x_rot) {
        return MovementDecision::Disconnect("multiplayer.disconnect.invalid_vehicle_movement");
    }
    if !state.controlling_player || !state.same_vehicle_as_last_tick {
        return MovementDecision::IgnoreUntilLoaded;
    }

    let target = Vec3 {
        x: clamp_horizontal(packet.x),
        y: clamp_vertical(packet.y),
        z: clamp_horizontal(packet.z),
    };
    let target_y_rot = wrap_degrees(packet.y_rot);
    let target_x_rot = wrap_degrees(packet.x_rot);
    let first_delta = sub(target, state.first_good);
    if first_delta.length_sqr() - state.velocity.length_sqr() > 100.0 && !state.singleplayer_owner {
        return MovementDecision::TeleportBack(TeleportCorrection {
            target: state.current,
            y_rot: state.y_rot,
            x_rot: state.x_rot,
            reason: CorrectionReason::VehicleMovedTooQuickly,
        });
    }

    let remainder = env.collision_remainder;
    let adjusted_remainder = Vec3 {
        x: remainder.x,
        y: if remainder.y > -0.5 || remainder.y < 0.5 {
            0.0
        } else {
            remainder.y
        },
        z: remainder.z,
    };
    let vehicle_moved_wrongly = adjusted_remainder.length_sqr() > 0.0625;
    if (vehicle_moved_wrongly && env.old_box_still_clear) || env.collides_with_new_blocks {
        return MovementDecision::TeleportBack(TeleportCorrection {
            target: state.current,
            y_rot: target_y_rot,
            x_rot: target_x_rot,
            reason: if vehicle_moved_wrongly {
                CorrectionReason::VehicleMovedWrongly
            } else {
                CorrectionReason::NewCollision
            },
        });
    }

    let requested_delta = sub(target, state.last_good);
    let client_vehicle_is_floating = requested_delta.y >= -0.03125
        && !state.vertical_collision_below
        && !state.allow_flight
        && !state.flying_vehicle
        && !state.no_gravity
        && state.no_blocks_around;
    MovementDecision::AcceptVehicle(AcceptedVehicleMove {
        target,
        y_rot: target_y_rot,
        x_rot: target_x_rot,
        client_delta: sub(target, state.current),
        client_vehicle_is_floating,
        updated_last_good: target,
    })
}

pub fn illegal_position_or_stance(packet: MovePacket) -> bool {
    contains_invalid_values(packet.x, packet.y, packet.z, packet.y_rot, packet.x_rot)
        || packet.y < -2.0E7
        || packet.y > 2.0E7
        || packet.x < -3.0E7
        || packet.x > 3.0E7
        || packet.z < -3.0E7
        || packet.z > 3.0E7
}

pub fn tick_floating_state(state: FloatingState) -> Result<i32, &'static str> {
    if state.client_is_floating && !state.sleeping && !state.passenger && !state.dead_or_dying {
        let ticks = state.above_ground_ticks + 1;
        if ticks > maximum_flying_ticks(state.gravity) {
            Err("multiplayer.disconnect.flying")
        } else {
            Ok(ticks)
        }
    } else {
        Ok(0)
    }
}

pub fn validate_teleport_ack(
    awaiting_id: i32,
    awaiting_position: Option<Vec3>,
    packet_id: i32,
) -> MovementDecision {
    if awaiting_id != packet_id {
        return MovementDecision::IgnoreUntilLoaded;
    }
    match awaiting_position {
        Some(target) => MovementDecision::Accept(AcceptedMove {
            target,
            y_rot: 0.0,
            x_rot: 0.0,
            client_delta: Vec3::ZERO,
            on_ground: false,
            horizontal_collision: false,
            client_is_floating: false,
            reset_impulse_context: false,
            reset_fall_distance: false,
            updated_move_packet_count: 0,
            updated_last_good: target,
        }),
        None => MovementDecision::Disconnect("multiplayer.disconnect.invalid_player_movement"),
    }
}

fn accept(
    state: ServerMovementState,
    packet: MovePacket,
    target: Vec3,
    y_rot: f32,
    x_rot: f32,
    client_delta: Vec3,
    reset_fall_distance: bool,
) -> MovementDecision {
    MovementDecision::Accept(AcceptedMove {
        target,
        y_rot,
        x_rot,
        client_delta,
        on_ground: packet.on_ground,
        horizontal_collision: packet.horizontal_collision,
        client_is_floating: false,
        reset_impulse_context: packet.on_ground || state.spectator || state.fall_flying,
        reset_fall_distance,
        updated_move_packet_count: state.received_move_packet_count,
        updated_last_good: target,
    })
}

fn should_check_player_movement(state: ServerMovementState) -> bool {
    !state.singleplayer_owner
        && !state.changing_dimension
        && state.movement_check_enabled
        && (!state.fall_flying || state.elytra_movement_check_enabled)
}

fn contains_invalid_values(x: f64, y: f64, z: f64, y_rot: f32, x_rot: f32) -> bool {
    x.is_nan() || y.is_nan() || z.is_nan() || !y_rot.is_finite() || !x_rot.is_finite()
}

fn packet_value_f64(value: f64, fallback: f64, present: bool) -> f64 {
    if present {
        value
    } else {
        fallback
    }
}

fn packet_value_f32(value: f32, fallback: f32, present: bool) -> f32 {
    if present {
        value
    } else {
        fallback
    }
}

fn clamp_horizontal(value: f64) -> f64 {
    value.clamp(-3.0E7, 3.0E7)
}

fn clamp_vertical(value: f64) -> f64 {
    value.clamp(-2.0E7, 2.0E7)
}

fn wrap_degrees(value: f32) -> f32 {
    let wrapped = (value + 180.0).rem_euclid(360.0) - 180.0;
    if wrapped == -180.0 {
        180.0
    } else {
        wrapped
    }
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet_at(x: f64, y: f64, z: f64) -> MovePacket {
        MovePacket {
            x,
            y,
            z,
            y_rot: 0.0,
            x_rot: 0.0,
            on_ground: true,
            horizontal_collision: false,
            has_position: true,
            has_rotation: true,
        }
    }

    #[test]
    fn invalid_values_disconnect_and_coordinates_are_clamped_like_vanilla() {
        let state = ServerMovementState::at(0.0, 64.0, 0.0);
        assert_eq!(
            validate_player_move(
                state,
                MovePacket {
                    x: f64::NAN,
                    ..packet_at(0.0, 64.0, 0.0)
                },
                ValidationEnvironment::default()
            ),
            MovementDecision::Disconnect("multiplayer.disconnect.invalid_player_movement")
        );

        let mut clamping_state = state;
        clamping_state.movement_check_enabled = false;
        let accepted = validate_player_move(
            clamping_state,
            packet_at(99_000_000.0, 99_000_000.0, -99_000_000.0),
            ValidationEnvironment::default(),
        );
        match accepted {
            MovementDecision::Accept(move_) => {
                assert_eq!(move_.target.x, 3.0E7);
                assert_eq!(move_.target.y, 2.0E7);
                assert_eq!(move_.target.z, -3.0E7);
            }
            other => panic!("expected accept, got {other:?}"),
        }
    }

    #[test]
    fn awaiting_teleport_rotates_or_resends_without_accepting_position() {
        let mut state = ServerMovementState::at(0.0, 64.0, 0.0);
        state.awaiting_teleport = Some(AwaitingTeleport {
            target: Vec3 {
                x: 8.0,
                y: 70.0,
                z: 8.0,
            },
            sent_tick: 10,
        });
        state.tick_count = 12;
        assert_eq!(
            validate_player_move(
                state,
                packet_at(4.0, 65.0, 4.0),
                ValidationEnvironment::default()
            ),
            MovementDecision::RotateWhileAwaitingTeleport {
                y_rot: 0.0,
                x_rot: 0.0
            }
        );

        state.tick_count = 31;
        assert_eq!(
            validate_player_move(
                state,
                packet_at(4.0, 65.0, 4.0),
                ValidationEnvironment::default()
            ),
            MovementDecision::ResendAwaitingTeleport {
                target: Vec3 {
                    x: 8.0,
                    y: 70.0,
                    z: 8.0
                }
            }
        );
    }

    #[test]
    fn sleeping_passenger_and_teleport_ack_paths_match_server_authority() {
        let mut state = ServerMovementState::at(0.0, 64.0, 0.0);
        state.sleeping = true;
        assert_eq!(
            validate_player_move(
                state,
                packet_at(2.0, 64.0, 0.0),
                ValidationEnvironment::default()
            ),
            MovementDecision::TeleportBack(TeleportCorrection {
                target: state.current,
                y_rot: 0.0,
                x_rot: 0.0,
                reason: CorrectionReason::SleepingMovedTooFar
            })
        );

        state.sleeping = false;
        state.passenger = true;
        match validate_player_move(
            state,
            packet_at(4.0, 70.0, 4.0),
            ValidationEnvironment::default(),
        ) {
            MovementDecision::Accept(move_) => assert_eq!(move_.target, state.current),
            other => panic!("expected passenger rotation accept, got {other:?}"),
        }

        assert_eq!(
            validate_teleport_ack(4, None, 4),
            MovementDecision::Disconnect("multiplayer.disconnect.invalid_player_movement")
        );
        match validate_teleport_ack(
            4,
            Some(Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }),
            4,
        ) {
            MovementDecision::Accept(move_) => {
                assert_eq!(
                    move_.target,
                    Vec3 {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0
                    }
                );
                assert_eq!(move_.updated_last_good, move_.target);
            }
            other => panic!("expected teleport ack accept, got {other:?}"),
        }
    }

    #[test]
    fn server_rejects_too_fast_or_wrong_collision_adjusted_moves() {
        let state = ServerMovementState::at(0.0, 64.0, 0.0);
        assert_eq!(
            validate_player_move(
                state,
                packet_at(11.0, 64.0, 0.0),
                ValidationEnvironment::default()
            ),
            MovementDecision::TeleportBack(TeleportCorrection {
                target: state.current,
                y_rot: state.y_rot,
                x_rot: state.x_rot,
                reason: CorrectionReason::MovedTooQuickly
            })
        );

        assert_eq!(
            validate_player_move(
                state,
                packet_at(0.2, 64.0, 0.2),
                ValidationEnvironment {
                    collision_remainder: Vec3 {
                        x: 0.3,
                        y: 0.0,
                        z: 0.0
                    },
                    old_box_still_clear: true,
                    collides_with_new_blocks: false,
                    tick_rate_runs_normally: true,
                }
            ),
            MovementDecision::TeleportBack(TeleportCorrection {
                target: state.current,
                y_rot: 0.0,
                x_rot: 0.0,
                reason: CorrectionReason::MovedWrongly
            })
        );
    }

    #[test]
    fn accepted_move_updates_ground_flags_floating_and_packet_count() {
        let mut state = ServerMovementState::at(0.0, 64.0, 0.0);
        state.vertical_collision_below = false;
        state.no_blocks_around = true;
        let mut packet = packet_at(0.1, 64.0, 0.0);
        packet.on_ground = false;
        packet.horizontal_collision = true;
        match validate_player_move(state, packet, ValidationEnvironment::default()) {
            MovementDecision::Accept(move_) => {
                assert_eq!(
                    move_.client_delta,
                    Vec3 {
                        x: 0.1,
                        y: 0.0,
                        z: 0.0
                    }
                );
                assert!(!move_.on_ground);
                assert!(move_.horizontal_collision);
                assert!(move_.client_is_floating);
                assert_eq!(move_.updated_move_packet_count, 1);
                assert_eq!(move_.updated_last_good, move_.target);
            }
            other => panic!("expected accept, got {other:?}"),
        }

        assert_eq!(maximum_flying_ticks(0.08), 80);
        assert_eq!(maximum_flying_ticks(0.04), 160);
        assert_eq!(maximum_flying_ticks(0.0), i32::MAX);
    }

    #[test]
    fn anti_cheat_vehicle_movement_corrects_too_fast_wrong_and_floating_moves() {
        let state = VehicleMovementState::at(0.0, 64.0, 0.0);
        assert_eq!(
            validate_vehicle_move(
                state,
                packet_at(10.1, 64.0, 0.0),
                ValidationEnvironment::default()
            ),
            MovementDecision::TeleportBack(TeleportCorrection {
                target: state.current,
                y_rot: state.y_rot,
                x_rot: state.x_rot,
                reason: CorrectionReason::VehicleMovedTooQuickly
            })
        );

        assert_eq!(
            validate_vehicle_move(
                state,
                packet_at(0.1, 64.0, 0.0),
                ValidationEnvironment {
                    collision_remainder: Vec3 {
                        x: 0.26,
                        y: 0.0,
                        z: 0.0
                    },
                    old_box_still_clear: true,
                    collides_with_new_blocks: false,
                    tick_rate_runs_normally: true,
                }
            ),
            MovementDecision::TeleportBack(TeleportCorrection {
                target: state.current,
                y_rot: 0.0,
                x_rot: 0.0,
                reason: CorrectionReason::VehicleMovedWrongly
            })
        );

        let mut floating = state;
        floating.vertical_collision_below = false;
        floating.no_blocks_around = true;
        match validate_vehicle_move(
            floating,
            packet_at(0.1, 64.0, 0.0),
            ValidationEnvironment::default(),
        ) {
            MovementDecision::AcceptVehicle(move_) => {
                assert_eq!(
                    move_.target,
                    Vec3 {
                        x: 0.1,
                        y: 64.0,
                        z: 0.0
                    }
                );
                assert!(move_.client_vehicle_is_floating);
            }
            other => panic!("expected vehicle accept, got {other:?}"),
        }
    }

    #[test]
    fn anti_cheat_illegal_stance_position_and_flying_ticks_match_vanilla_thresholds() {
        assert!(illegal_position_or_stance(MovePacket {
            x: 3.0E7 + 1.0,
            ..packet_at(0.0, 64.0, 0.0)
        }));
        assert!(illegal_position_or_stance(MovePacket {
            y: -2.0E7 - 1.0,
            ..packet_at(0.0, 64.0, 0.0)
        }));
        assert!(illegal_position_or_stance(MovePacket {
            x_rot: f32::INFINITY,
            ..packet_at(0.0, 64.0, 0.0)
        }));
        assert!(!illegal_position_or_stance(packet_at(3.0E7, 2.0E7, -3.0E7)));

        assert_eq!(
            tick_floating_state(FloatingState {
                client_is_floating: true,
                above_ground_ticks: 79,
                gravity: 0.08,
                sleeping: false,
                passenger: false,
                dead_or_dying: false,
            }),
            Ok(80)
        );
        assert_eq!(
            tick_floating_state(FloatingState {
                client_is_floating: true,
                above_ground_ticks: 80,
                gravity: 0.08,
                sleeping: false,
                passenger: false,
                dead_or_dying: false,
            }),
            Err("multiplayer.disconnect.flying")
        );
        assert_eq!(
            tick_floating_state(FloatingState {
                client_is_floating: true,
                above_ground_ticks: 80,
                gravity: 0.08,
                sleeping: true,
                passenger: false,
                dead_or_dying: false,
            }),
            Ok(0)
        );
    }
}
