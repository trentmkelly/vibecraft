#![allow(dead_code)]

use std::collections::BTreeSet;
use std::io::{self, Read, Write};

use crate::network::dispatch::{DecodedPacket, DispatchOutcome, PacketDirection, ProtocolState};
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;

pub const SERVERBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 69;
pub const CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 141;

pub const SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
pub const SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID: i32 = 30;
pub const SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID: i32 = 31;
pub const SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID: i32 = 32;
pub const SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID: i32 = 33;
pub const SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
pub const SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID: i32 = 53;
pub const SERVERBOUND_USE_ITEM_ON_PACKET_ID: i32 = 66;
pub const SERVERBOUND_USE_ITEM_PACKET_ID: i32 = 67;

pub const CLIENTBOUND_LOGIN_PACKET_ID: i32 = 49;
pub const CLIENTBOUND_PLAYER_POSITION_PACKET_ID: i32 = 72;
pub const CLIENTBOUND_SET_HELD_SLOT_PACKET_ID: i32 = 105;
pub const CLIENTBOUND_START_CONFIGURATION_PACKET_ID: i32 = 118;
pub const CLIENTBOUND_DISCONNECT_PACKET_ID: i32 = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayProtocolRegistry {
    serverbound: Vec<&'static str>,
    clientbound: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonPlayerSpawnInfo {
    pub dimension_type: Identifier,
    pub dimension: Identifier,
    pub seed: i64,
    pub game_mode: GameMode,
    pub previous_game_mode: Option<GameMode>,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<(Identifier, [i32; 3])>,
    pub portal_cooldown: i32,
    pub sea_level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginPacket {
    pub player_id: i32,
    pub hardcore: bool,
    pub levels: Vec<Identifier>,
    pub max_players: i32,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub spawn_info: CommonPlayerSpawnInfo,
    pub enforces_secure_chat: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundMovePlayerPacket {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundAcceptTeleportationPacket {
    pub teleport_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSetCarriedItemPacket {
    pub slot: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetHeldSlotPacket {
    pub slot: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayInstruction {
    Login(ClientboundLoginPacket),
    SetHeldSlot(ClientboundSetHeldSlotPacket),
    PlayerPosition { teleport_id: i32 },
    StartConfiguration,
    Disconnect(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Joining,
    WaitingForPlayerLoaded,
    Playing,
    Reconfiguring,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaySession {
    pub state: PlayState,
    pub entity_id: i32,
    pub selected_slot: i16,
    pub pending_teleports: BTreeSet<i32>,
    pub last_move: Option<ServerboundMovePlayerPacket>,
    pub loaded: bool,
    pub disconnect_reason: Option<String>,
}

impl Default for PlayProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayProtocolRegistry {
    pub fn new() -> Self {
        Self {
            serverbound: SERVERBOUND_PLAY_PACKET_NAMES.to_vec(),
            clientbound: CLIENTBOUND_PLAY_PACKET_NAMES.to_vec(),
        }
    }

    pub fn serverbound(&self) -> &[&'static str] {
        &self.serverbound
    }

    pub fn clientbound(&self) -> &[&'static str] {
        &self.clientbound
    }

    pub fn serverbound_name(&self, packet_id: i32) -> Option<&'static str> {
        self.serverbound.get(packet_id as usize).copied()
    }

    pub fn clientbound_name(&self, packet_id: i32) -> Option<&'static str> {
        self.clientbound.get(packet_id as usize).copied()
    }

    pub fn is_serverbound_play_packet(&self, packet_id: i32) -> bool {
        packet_id >= 0 && (packet_id as usize) < self.serverbound.len()
    }

    pub fn is_clientbound_play_packet(&self, packet_id: i32) -> bool {
        packet_id >= 0 && (packet_id as usize) < self.clientbound.len()
    }
}

impl Default for CommonPlayerSpawnInfo {
    fn default() -> Self {
        Self {
            dimension_type: Identifier::parse("minecraft:overworld").unwrap(),
            dimension: Identifier::parse("minecraft:overworld").unwrap(),
            seed: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
            is_debug: false,
            is_flat: false,
            last_death_location: None,
            portal_cooldown: 0,
            sea_level: 63,
        }
    }
}

impl PlaySession {
    pub fn new(entity_id: i32, selected_slot: i16) -> Self {
        Self {
            state: PlayState::Joining,
            entity_id,
            selected_slot,
            pending_teleports: BTreeSet::new(),
            last_move: None,
            loaded: false,
            disconnect_reason: None,
        }
    }

    pub fn join_sequence(&mut self, login: ClientboundLoginPacket) -> Vec<PlayInstruction> {
        self.state = PlayState::WaitingForPlayerLoaded;
        vec![
            PlayInstruction::Login(login),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket {
                slot: self.selected_slot as i32,
            }),
            PlayInstruction::PlayerPosition { teleport_id: 0 },
        ]
    }

    pub fn handle_decoded(&mut self, packet: DecodedPacket) -> DispatchOutcome {
        if packet.state != ProtocolState::Play || packet.direction != PacketDirection::Serverbound {
            return DispatchOutcome::Disconnect(format!(
                "unexpected {:?} {:?} packet {} during play",
                packet.state, packet.direction, packet.id
            ));
        }

        match packet.id {
            SERVERBOUND_PLAYER_LOADED_PACKET_ID => {
                self.loaded = true;
                self.state = PlayState::Playing;
                DispatchOutcome::Handled
            }
            SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundAcceptTeleportationPacket::read(&mut input) {
                    Ok(ack) => {
                        self.pending_teleports.remove(&ack.teleport_id);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad teleport ack: {err}")),
                }
            }
            SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::Pos)
            }
            SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::PosRot)
            }
            SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::Rot)
            }
            SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::StatusOnly)
            }
            SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCarriedItemPacket::read(&mut input) {
                    Ok(held) if (0..=8).contains(&held.slot) => {
                        self.selected_slot = held.slot;
                        DispatchOutcome::Handled
                    }
                    Ok(held) => DispatchOutcome::Disconnect(format!(
                        "invalid carried item slot {}",
                        held.slot
                    )),
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad carried item packet: {err}"))
                    }
                }
            }
            _ => {
                if PlayProtocolRegistry::new().is_serverbound_play_packet(packet.id) {
                    DispatchOutcome::Handled
                } else {
                    DispatchOutcome::Disconnect(format!("unknown play packet id {}", packet.id))
                }
            }
        }
    }

    pub fn request_reconfiguration(&mut self) -> PlayInstruction {
        self.state = PlayState::Reconfiguring;
        PlayInstruction::StartConfiguration
    }

    pub fn disconnect(&mut self, reason: impl Into<String>) -> PlayInstruction {
        let reason = reason.into();
        self.state = PlayState::Disconnected;
        self.disconnect_reason = Some(reason.clone());
        PlayInstruction::Disconnect(reason)
    }

    fn handle_move_payload(&mut self, payload: Vec<u8>, shape: MoveShape) -> DispatchOutcome {
        let mut input = &payload[..];
        match ServerboundMovePlayerPacket::read_shape(&mut input, shape) {
            Ok(packet) => {
                self.last_move = Some(packet);
                DispatchOutcome::Handled
            }
            Err(err) => DispatchOutcome::Disconnect(format!("bad movement packet: {err}")),
        }
    }
}

impl ServerboundAcceptTeleportationPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            teleport_id: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.teleport_id)
    }
}

impl ServerboundSetCarriedItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 2];
        reader.read_exact(&mut bytes)?;
        Ok(Self {
            slot: i16::from_be_bytes(bytes),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.slot.to_be_bytes())
    }
}

impl ClientboundSetHeldSlotPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            slot: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.slot)
    }
}

impl ServerboundMovePlayerPacket {
    fn read_shape<R: Read>(reader: &mut R, shape: MoveShape) -> io::Result<Self> {
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

    fn write_shape<W: Write>(&self, writer: &mut W, shape: MoveShape) -> io::Result<()> {
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

    pub fn write_pos_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::PosRot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveShape {
    Pos,
    PosRot,
    Rot,
    StatusOnly,
}

impl MoveShape {
    fn has_position(self) -> bool {
        matches!(self, Self::Pos | Self::PosRot)
    }

    fn has_rotation(self) -> bool {
        matches!(self, Self::Rot | Self::PosRot)
    }
}

fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(byte[0])
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

fn pack_move_flags(on_ground: bool, horizontal_collision: bool) -> u8 {
    (if on_ground { 1 } else { 0 }) | (if horizontal_collision { 2 } else { 0 })
}

static SERVERBOUND_PLAY_PACKET_NAMES: [&str; SERVERBOUND_PLAY_PACKET_COUNT_26_1_2] = [
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

static CLIENTBOUND_PLAY_PACKET_NAMES: [&str; CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2] = [
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::codec::cursor;

    fn decoded(id: i32, payload: Vec<u8>) -> DecodedPacket {
        DecodedPacket {
            state: ProtocolState::Play,
            direction: PacketDirection::Serverbound,
            id,
            payload,
        }
    }

    #[test]
    fn play_packet_registry_matches_game_protocol_order_and_counts() {
        let registry = PlayProtocolRegistry::new();
        assert_eq!(
            registry.serverbound().len(),
            SERVERBOUND_PLAY_PACKET_COUNT_26_1_2
        );
        assert_eq!(
            registry.clientbound().len(),
            CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2
        );
        assert_eq!(
            registry.serverbound_name(SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID),
            Some("accept_teleportation")
        );
        assert_eq!(
            registry.serverbound_name(SERVERBOUND_PLAYER_LOADED_PACKET_ID),
            Some("player_loaded")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_LOGIN_PACKET_ID),
            Some("login")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_START_CONFIGURATION_PACKET_ID),
            Some("start_configuration")
        );
        assert_eq!(registry.serverbound().last(), Some(&"custom_click_action"));
        assert_eq!(registry.clientbound().last(), Some(&"show_dialog"));
    }

    #[test]
    fn join_sequence_enters_play_with_login_held_slot_and_position_packets() {
        let mut session = PlaySession::new(42, 3);
        let login = ClientboundLoginPacket {
            player_id: 42,
            hardcore: false,
            levels: vec![Identifier::parse("minecraft:overworld").unwrap()],
            max_players: 20,
            chunk_radius: 10,
            simulation_distance: 10,
            reduced_debug_info: false,
            show_death_screen: true,
            do_limited_crafting: false,
            spawn_info: CommonPlayerSpawnInfo::default(),
            enforces_secure_chat: false,
        };

        let instructions = session.join_sequence(login.clone());
        assert_eq!(session.state, PlayState::WaitingForPlayerLoaded);
        assert_eq!(
            instructions,
            vec![
                PlayInstruction::Login(login),
                PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket { slot: 3 }),
                PlayInstruction::PlayerPosition { teleport_id: 0 }
            ]
        );
    }

    #[test]
    fn player_loaded_packet_moves_session_to_playing() {
        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, Vec::new())),
            DispatchOutcome::Handled
        );
        assert_eq!(session.state, PlayState::Playing);
        assert!(session.loaded);
    }

    #[test]
    fn movement_packets_decode_flags_position_and_rotation_by_shape() {
        let movement = ServerboundMovePlayerPacket {
            x: 1.25,
            y: 65.0,
            z: -2.5,
            y_rot: 90.0,
            x_rot: 30.0,
            on_ground: true,
            horizontal_collision: true,
            has_position: true,
            has_rotation: true,
        };
        let mut payload = Vec::new();
        movement.write_pos_rot(&mut payload).unwrap();

        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, payload)),
            DispatchOutcome::Handled
        );
        let decoded = session.last_move.unwrap();
        assert_eq!(decoded.x, 1.25);
        assert_eq!(decoded.z, -2.5);
        assert_eq!(decoded.y_rot, 90.0);
        assert!(decoded.on_ground);
        assert!(decoded.horizontal_collision);
        assert!(decoded.has_position);
        assert!(decoded.has_rotation);
    }

    #[test]
    fn teleport_ack_and_held_slot_follow_play_state_validation() {
        let mut session = PlaySession::new(1, 0);
        session.pending_teleports.insert(7);

        let mut ack = Vec::new();
        ServerboundAcceptTeleportationPacket { teleport_id: 7 }
            .write(&mut ack)
            .unwrap();
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, ack)),
            DispatchOutcome::Handled
        );
        assert!(session.pending_teleports.is_empty());

        let mut held = Vec::new();
        ServerboundSetCarriedItemPacket { slot: 8 }
            .write(&mut held)
            .unwrap();
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, held)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.selected_slot, 8);

        let mut invalid = Vec::new();
        ServerboundSetCarriedItemPacket { slot: 9 }
            .write(&mut invalid)
            .unwrap();
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, invalid)),
            DispatchOutcome::Disconnect(_)
        ));
    }

    #[test]
    fn play_session_rejects_wrong_state_or_unknown_packets_and_can_reconfigure() {
        let mut session = PlaySession::new(1, 0);
        let wrong_state = DecodedPacket {
            state: ProtocolState::Configuration,
            direction: PacketDirection::Serverbound,
            id: SERVERBOUND_PLAYER_LOADED_PACKET_ID,
            payload: Vec::new(),
        };
        assert!(matches!(
            session.handle_decoded(wrong_state),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(999, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert_eq!(
            session.request_reconfiguration(),
            PlayInstruction::StartConfiguration
        );
        assert_eq!(session.state, PlayState::Reconfiguring);
    }

    #[test]
    fn small_play_packets_round_trip_vanilla_codecs() {
        let mut bytes = Vec::new();
        ClientboundSetHeldSlotPacket { slot: 4 }
            .write(&mut bytes)
            .unwrap();
        assert_eq!(
            ClientboundSetHeldSlotPacket::read(&mut cursor(bytes)).unwrap(),
            ClientboundSetHeldSlotPacket { slot: 4 }
        );

        let mut carried = Vec::new();
        ServerboundSetCarriedItemPacket { slot: 5 }
            .write(&mut carried)
            .unwrap();
        assert_eq!(
            ServerboundSetCarriedItemPacket::read(&mut cursor(carried)).unwrap(),
            ServerboundSetCarriedItemPacket { slot: 5 }
        );
    }
}
