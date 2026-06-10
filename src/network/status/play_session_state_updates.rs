use super::*;

pub fn update_play_session_state<R: Read>(
    packet_id: i32,
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    match packet_id {
        SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID => update_position_packet(input, state),
        SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID => update_position_rotation_packet(input, state),
        SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID => update_rotation_packet(input, state),
        SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID => update_status_only_packet(input, state),
        SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID => update_carried_item_packet(input, state),
        SERVERBOUND_CLIENT_COMMAND_PACKET_ID => update_client_command_packet(input, state),
        SERVERBOUND_PLAYER_INPUT_PACKET_ID => update_player_input_packet(input, state),
        SERVERBOUND_PLAYER_ABILITIES_PACKET_ID => {
            let packet = ServerboundPlayerAbilitiesPacket::read(input)?;
            super::player_creative_packets::apply_serverbound_player_abilities_packet(
                state, packet,
            );
            Ok(PlaySessionUpdate::default())
        }
        SERVERBOUND_CONTAINER_CLOSE_PACKET_ID => {
            let packet = ServerboundContainerClosePacket::read(input)?;
            if packet.container_id == 0 {
                state.inventory_menu.removed(&mut state.carried_item);
            } else if state
                .active_block_menu
                .as_ref()
                .is_some_and(|menu| menu.container_id() == packet.container_id)
            {
                if let Some(menu) = state.active_block_menu.take() {
                    menu.close(state);
                }
            }
            Ok(PlaySessionUpdate::default())
        }
        _ => Ok(PlaySessionUpdate::default()),
    }
}

// TODO(live-move-validation): these handlers assign the client-sent position
// directly to state.x/y/z with NO server-authoritative validation. The
// comprehensive, Java-1:1, fully-tested validate_player_move/validate_vehicle_move
// in movement_validation.rs (moved-too-quickly 100²/300², coordinate clamping,
// teleport-ack authority, illegal-stance/flying anti-cheat, packet correction)
// exists but is dead code — never called from the live loop. Routing these
// handlers through it is what completes PLAYER checklist #130 (server
// authoritative movement validation) and #131 (anti-cheat checks), both left
// UNMARKED until then.
fn update_position_packet<R: Read>(
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    let old_x = state.x;
    let old_y = state.y;
    let old_z = state.z;
    state.x = read_f64(input)?;
    state.y = read_f64(input)?;
    state.z = read_f64(input)?;
    state.on_ground = read_bool(input)?;
    Ok(apply_player_movement(
        state,
        state.x - old_x,
        state.y - old_y,
        state.z - old_z,
        true,
    ))
}

fn update_position_rotation_packet<R: Read>(
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    let old_x = state.x;
    let old_y = state.y;
    let old_z = state.z;
    state.x = read_f64(input)?;
    state.y = read_f64(input)?;
    state.z = read_f64(input)?;
    state.yaw = read_f32(input)?;
    state.pitch = read_f32(input)?;
    state.on_ground = read_bool(input)?;
    Ok(apply_player_movement(
        state,
        state.x - old_x,
        state.y - old_y,
        state.z - old_z,
        true,
    ))
}

fn update_rotation_packet<R: Read>(
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    state.yaw = read_f32(input)?;
    state.pitch = read_f32(input)?;
    state.on_ground = read_bool(input)?;
    Ok(apply_player_fall_movement(state, 0.0, false))
}

fn update_status_only_packet<R: Read>(
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    state.on_ground = read_bool(input)?;
    Ok(apply_player_fall_movement(state, 0.0, false))
}

fn update_carried_item_packet<R: Read>(
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    let slot = i32::from(read_i16(input)?);
    if (0..9).contains(&slot) {
        state.selected_slot = slot;
    }
    Ok(PlaySessionUpdate::default())
}

fn update_client_command_packet<R: Read>(
    input: &mut R,
    state: &PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    let action = read_var_i32(input)?;
    // Java ServerboundClientCommandPacket.Action ordinal 0 = PERFORM_RESPAWN.
    // ServerGamePacketListenerImpl ignores it while the player is alive.
    Ok(PlaySessionUpdate {
        respawn_requested: action == 0 && state.health <= 0.0,
        ..PlaySessionUpdate::default()
    })
}

fn update_player_input_packet<R: Read>(
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    let flags = read_u8(input)?;
    let jumping = flags & 16 != 0;
    let sprinting = flags & 64 != 0;
    if jumping && !state.input_jumping && state.on_ground {
        add_player_food_exhaustion(
            state,
            if sprinting {
                SPRINT_JUMP_EXHAUSTION
            } else {
                JUMP_EXHAUSTION
            },
        );
    }
    state.input_forward = flags & 1 != 0;
    state.input_backward = flags & 2 != 0;
    state.input_left = flags & 4 != 0;
    state.input_right = flags & 8 != 0;
    state.input_shift = flags & 32 != 0;
    state.input_jumping = jumping;
    state.input_sprinting = sprinting;
    Ok(PlaySessionUpdate::default())
}
