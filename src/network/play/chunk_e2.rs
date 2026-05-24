use super::*;

impl ClientboundSetCursorItemPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.item_stack.write_optional_untrusted(writer)
    }
}

impl ClientboundMountScreenOpenPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.inventory_columns)?;
        write_i32(writer, self.entity_id)
    }
}

impl ClientboundCooldownPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.cooldown_group)?;
        write_var_i32(writer, self.duration)
    }
}

impl ClientboundPlayerAbilitiesPacket {
    pub fn flags(&self) -> u8 {
        (if self.invulnerable { 1 } else { 0 })
            | (if self.flying { 2 } else { 0 })
            | (if self.can_fly { 4 } else { 0 })
            | (if self.instant_build { 8 } else { 0 })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.flags()])?;
        write_f32(writer, self.flying_speed)?;
        write_f32(writer, self.walking_speed)
    }
}

impl ClientboundAwardStatsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.stats.len() as i32)?;
        for stat in &self.stats {
            write_var_i32(writer, stat.stat_type_id)?;
            write_var_i32(writer, stat.stat_value_id)?;
            write_var_i32(writer, stat.value)?;
        }
        Ok(())
    }
}

impl ClientboundUpdateAttributesPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.attributes.len() as i32)?;
        for attribute in &self.attributes {
            attribute.write(writer)?;
        }
        Ok(())
    }
}

impl AttributeSnapshot {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.attribute_id)?;
        write_f64(writer, self.base)?;
        write_var_i32(writer, self.modifiers.len() as i32)?;
        for modifier in &self.modifiers {
            modifier.write(writer)?;
        }
        Ok(())
    }
}

impl AttributeModifierSnapshot {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.id)?;
        write_f64(writer, self.amount)?;
        write_var_i32(writer, self.operation as i32)
    }
}

impl ClientboundPingPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(Self {
            id: i32::from_be_bytes(bytes),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
    }
}

impl ClientboundGameRuleValuesPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.values.len() as i32)?;
        for (key, value) in &self.values {
            write_identifier(writer, key)?;
            write_string(writer, value, 32767)?;
        }
        Ok(())
    }
}

impl ClientboundBossEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_uuid(writer, self.event_id)?;
        match &self.operation {
            BossEventOperation::Add {
                name,
                progress,
                color,
                overlay,
                flags,
            } => {
                write_var_i32(writer, 0)?;
                write_network_tag(writer, name)?;
                write_f32(writer, *progress)?;
                write_var_i32(writer, *color as i32)?;
                write_var_i32(writer, *overlay as i32)?;
                writer.write_all(&[flags.bits()])
            }
            BossEventOperation::Remove => write_var_i32(writer, 1),
            BossEventOperation::UpdateProgress { progress } => {
                write_var_i32(writer, 2)?;
                write_f32(writer, *progress)
            }
            BossEventOperation::UpdateName { name } => {
                write_var_i32(writer, 3)?;
                write_network_tag(writer, name)
            }
            BossEventOperation::UpdateStyle { color, overlay } => {
                write_var_i32(writer, 4)?;
                write_var_i32(writer, *color as i32)?;
                write_var_i32(writer, *overlay as i32)
            }
            BossEventOperation::UpdateProperties { flags } => {
                write_var_i32(writer, 5)?;
                writer.write_all(&[flags.bits()])
            }
        }
    }
}

impl ClientboundMapItemDataPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.map_id)?;
        writer.write_all(&[self.scale])?;
        write_bool(writer, self.locked)?;
        write_optional(writer, self.decorations.as_ref(), |writer, decorations| {
            write_var_i32(writer, decorations.len() as i32)?;
            for decoration in decorations {
                decoration.write(writer)?;
            }
            Ok(())
        })?;
        match &self.color_patch {
            Some(patch) => patch.write(writer),
            None => writer.write_all(&[0]),
        }
    }
}

impl MapDecorationData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.decoration_type_id)?;
        writer.write_all(&[self.x as u8, self.y as u8, self.rotation as u8])?;
        write_optional(writer, self.name.as_ref(), |writer, name| {
            write_network_tag(writer, name)
        })
    }
}

impl MapPatch {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.width == 0 {
            return writer.write_all(&[0]);
        }
        writer.write_all(&[self.width, self.height, self.start_x, self.start_y])?;
        write_var_i32(writer, self.colors.len() as i32)?;
        writer.write_all(&self.colors)
    }
}

impl BossEventFlags {
    pub fn bits(self) -> u8 {
        (if self.darken_screen { 1 } else { 0 })
            | (if self.play_music { 2 } else { 0 })
            | (if self.create_world_fog { 4 } else { 0 })
    }
}

impl ServerboundMovePlayerPacket {
    pub(super) fn read_shape<R: Read>(reader: &mut R, shape: MoveShape) -> io::Result<Self> {
        let mut packet = Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            y_rot: 0.0,
            x_rot: 0.0,
            on_ground: false,
            horizontal_collision: false,
            has_position: shape.has_position(),
            has_rotation: shape.has_rotation(),
        };
        if shape.has_position() {
            packet.x = read_f64(reader)?;
            packet.y = read_f64(reader)?;
            packet.z = read_f64(reader)?;
        }
        if shape.has_rotation() {
            packet.y_rot = read_f32(reader)?;
            packet.x_rot = read_f32(reader)?;
        }
        let flags = read_u8(reader)?;
        packet.on_ground = flags & 1 != 0;
        packet.horizontal_collision = flags & 2 != 0;
        Ok(packet)
    }

    pub(super) fn write_shape<W: Write>(&self, writer: &mut W, shape: MoveShape) -> io::Result<()> {
        if shape.has_position() {
            writer.write_all(&self.x.to_be_bytes())?;
            writer.write_all(&self.y.to_be_bytes())?;
            writer.write_all(&self.z.to_be_bytes())?;
        }
        if shape.has_rotation() {
            writer.write_all(&self.y_rot.to_be_bytes())?;
            writer.write_all(&self.x_rot.to_be_bytes())?;
        }
        writer.write_all(&[pack_move_flags(self.on_ground, self.horizontal_collision)])
    }

    pub fn write_pos<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::Pos)
    }

    pub fn write_pos_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::PosRot)
    }

    pub fn write_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::Rot)
    }

    pub fn write_status_only<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::StatusOnly)
    }
}

impl ServerboundMoveVehiclePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            position: Vec3 {
                x: read_f64(reader)?,
                y: read_f64(reader)?,
                z: read_f64(reader)?,
            },
            y_rot: read_f32(reader)?,
            x_rot: read_f32(reader)?,
            on_ground: read_bool(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.position.x.to_be_bytes())?;
        writer.write_all(&self.position.y.to_be_bytes())?;
        writer.write_all(&self.position.z.to_be_bytes())?;
        write_f32(writer, self.y_rot)?;
        write_f32(writer, self.x_rot)?;
        write_bool(writer, self.on_ground)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MoveShape {
    Pos,
    PosRot,
    Rot,
    StatusOnly,
}

impl MoveShape {
    pub(super) fn has_position(self) -> bool {
        matches!(self, Self::Pos | Self::PosRot)
    }

    pub(super) fn has_rotation(self) -> bool {
        matches!(self, Self::Rot | Self::PosRot)
    }
}

pub(super) fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(byte[0])
}

pub(super) fn read_i8<R: Read>(reader: &mut R) -> io::Result<i8> {
    Ok(read_u8(reader)? as i8)
}

pub(super) fn write_i8<W: Write>(writer: &mut W, value: i8) -> io::Result<()> {
    writer.write_all(&[value as u8])
}

pub(super) fn read_clamped_i8<R: Read>(reader: &mut R, min: i8, max: i8) -> io::Result<i8> {
    Ok((read_u8(reader)? as i8).clamp(min, max))
}

pub(super) fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(super) fn expect_empty_payload<R: Read>(reader: &mut R) -> io::Result<()> {
    let mut byte = [0u8; 1];
    match reader.read(&mut byte)? {
        0 => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected empty payload",
        )),
    }
}

pub(super) fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

pub(super) fn write_f32<W: Write>(writer: &mut W, value: f32) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

pub(super) fn read_i16<R: Read>(reader: &mut R) -> io::Result<i16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(i16::from_be_bytes(bytes))
}

pub(super) fn write_i16<W: Write>(writer: &mut W, value: i16) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

pub(super) fn read_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_be_bytes(bytes))
}

pub(super) fn write_i32<W: Write>(writer: &mut W, value: i32) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

pub(super) fn read_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

pub(super) fn write_i64<W: Write>(writer: &mut W, value: i64) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

pub(super) fn write_network_compound_tag<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    match tag {
        Tag::Compound(fields) if fields.is_empty() => writer.write_all(&[0]),
        Tag::Compound(_) => write_network_tag(writer, tag),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "network compound tag payload must be a compound",
        )),
    }
}

pub(super) fn write_network_tag<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    if !matches!(tag, Tag::End) {
        tag.write_payload(writer)?;
    }
    Ok(())
}

pub(super) fn read_length_prefixed_bytes<R: Read>(reader: &mut R, max_size: usize) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "length-prefixed payload too large",
        ));
    }
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub(super) fn write_length_prefixed_bytes<W: Write>(
    writer: &mut W,
    payload: &[u8],
    max_size: usize,
) -> io::Result<()> {
    if payload.len() > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "length-prefixed payload too large",
        ));
    }
    write_var_i32(writer, payload.len() as i32)?;
    writer.write_all(payload)
}

pub(super) fn read_limited_collection<R, T, F>(
    reader: &mut R,
    max_len: usize,
    mut read: F,
) -> io::Result<Vec<T>>
where
    R: Read,
    F: FnMut(&mut R) -> io::Result<T>,
{
    let len = read_var_i32(reader)?;
    if len < 0 || len as usize > max_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "collection length exceeds packet limit",
        ));
    }
    let mut values = Vec::with_capacity(len as usize);
    for _ in 0..len {
        values.push(read(reader)?);
    }
    Ok(values)
}

pub(super) fn read_limited_len<R: Read>(
    reader: &mut R,
    max_len: usize,
    description: &'static str,
) -> io::Result<usize> {
    let len = read_var_i32(reader)?;
    if len < 0 || len as usize > max_len {
        return Err(io::Error::new(io::ErrorKind::InvalidData, description));
    }
    Ok(len as usize)
}

pub(super) fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

pub(super) fn write_f64<W: Write>(writer: &mut W, value: f64) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

pub(super) fn write_vec3<W: Write>(writer: &mut W, value: Vec3) -> io::Result<()> {
    write_f64(writer, value.x)?;
    write_f64(writer, value.y)?;
    write_f64(writer, value.z)
}

pub(super) fn pack_move_flags(on_ground: bool, horizontal_collision: bool) -> u8 {
    (if on_ground { 1 } else { 0 }) | (if horizontal_collision { 2 } else { 0 })
}

pub(super) static SERVERBOUND_PLAY_PACKET_NAMES: [&str; SERVERBOUND_PLAY_PACKET_COUNT_26_1_2] = [
    "accept_teleportation",
    "attack",
    "block_entity_tag_query",
    "bundle_item_selected",
    "change_difficulty",
    "change_game_mode",
    "chat_ack",
    "chat_command",
    "chat_command_signed",
    "chat",
    "chat_session_update",
    "chunk_batch_received",
    "client_command",
    "client_tick_end",
    "client_information",
    "command_suggestion",
    "configuration_acknowledged",
    "container_button_click",
    "container_click",
    "container_close",
    "container_slot_state_changed",
    "cookie_response",
    "custom_payload",
    "debug_subscription_request",
    "edit_book",
    "entity_tag_query",
    "interact",
    "jigsaw_generate",
    "keep_alive",
    "lock_difficulty",
    "move_player_pos",
    "move_player_pos_rot",
    "move_player_rot",
    "move_player_status_only",
    "move_vehicle",
    "paddle_boat",
    "pick_item_from_block",
    "pick_item_from_entity",
    "ping_request",
    "place_recipe",
    "player_abilities",
    "player_action",
    "player_command",
    "player_input",
    "player_loaded",
    "pong",
    "recipe_book_change_settings",
    "recipe_book_seen_recipe",
    "rename_item",
    "resource_pack",
    "seen_advancements",
    "select_trade",
    "set_beacon",
    "set_carried_item",
    "set_command_block",
    "set_command_minecart",
    "set_creative_mode_slot",
    "set_game_rule",
    "set_jigsaw_block",
    "set_structure_block",
    "set_test_block",
    "sign_update",
    "spectate_entity",
    "swing",
    "teleport_to_entity",
    "test_instance_block_action",
    "use_item_on",
    "use_item",
    "custom_click_action",
];

pub(super) static CLIENTBOUND_PLAY_PACKET_NAMES: [&str; CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2] = [
    "bundle",
    "add_entity",
    "animate",
    "award_stats",
    "block_changed_ack",
    "block_destruction",
    "block_entity_data",
    "block_event",
    "block_update",
    "boss_event",
    "change_difficulty",
    "chunk_batch_finished",
    "chunk_batch_start",
    "chunks_biomes",
    "clear_titles",
    "command_suggestions",
    "commands",
    "container_close",
    "container_set_content",
    "container_set_data",
    "container_set_slot",
    "cookie_request",
    "cooldown",
    "custom_chat_completions",
    "custom_payload",
    "damage_event",
    "debug_block_value",
    "debug_chunk_value",
    "debug_entity_value",
    "debug_event",
    "debug_sample",
    "delete_chat",
    "disconnect",
    "disguised_chat",
    "entity_event",
    "entity_position_sync",
    "explode",
    "forget_level_chunk",
    "game_event",
    "game_rule_values",
    "game_test_highlight_pos",
    "mount_screen_open",
    "hurt_animation",
    "initialize_border",
    "keep_alive",
    "level_chunk_with_light",
    "level_event",
    "level_particles",
    "light_update",
    "login",
    "low_disk_space_warning",
    "map_item_data",
    "merchant_offers",
    "move_entity_pos",
    "move_entity_pos_rot",
    "move_minecart_along_track",
    "move_entity_rot",
    "move_vehicle",
    "open_book",
    "open_screen",
    "open_sign_editor",
    "ping",
    "pong_response",
    "place_ghost_recipe",
    "player_abilities",
    "player_chat",
    "player_combat_end",
    "player_combat_enter",
    "player_combat_kill",
    "player_info_remove",
    "player_info_update",
    "player_look_at",
    "player_position",
    "player_rotation",
    "recipe_book_add",
    "recipe_book_remove",
    "recipe_book_settings",
    "remove_entities",
    "remove_mob_effect",
    "reset_score",
    "resource_pack_pop",
    "resource_pack_push",
    "respawn",
    "rotate_head",
    "section_blocks_update",
    "select_advancements_tab",
    "server_data",
    "set_action_bar_text",
    "set_border_center",
    "set_border_lerp_size",
    "set_border_size",
    "set_border_warning_delay",
    "set_border_warning_distance",
    "set_camera",
    "set_chunk_cache_center",
    "set_chunk_cache_radius",
    "set_cursor_item",
    "set_default_spawn_position",
    "set_display_objective",
    "set_entity_data",
    "set_entity_link",
    "set_entity_motion",
    "set_equipment",
    "set_experience",
    "set_health",
    "set_held_slot",
    "set_objective",
    "set_passengers",
    "set_player_inventory",
    "set_player_team",
    "set_score",
    "set_simulation_distance",
    "set_subtitle_text",
    "set_time",
    "set_title_text",
    "set_titles_animation",
    "sound_entity",
    "sound",
    "start_configuration",
    "stop_sound",
    "store_cookie",
    "system_chat",
    "tab_list",
    "tag_query",
    "take_item_entity",
    "teleport_entity",
    "test_instance_block_status",
    "ticking_state",
    "ticking_step",
    "transfer",
    "update_advancements",
    "update_attributes",
    "update_mob_effect",
    "update_recipes",
    "update_tags",
    "projectile_power",
    "custom_report_details",
    "server_links",
    "waypoint",
    "clear_dialog",
    "show_dialog",
];
