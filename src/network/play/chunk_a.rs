use super::*;

impl PlaySession {
    pub fn new(entity_id: i32, selected_slot: i16) -> Self {
        Self {
            state: PlayState::Joining,
            entity_id,
            selected_slot,
            container_state_id: 0,
            pending_teleports: BTreeSet::new(),
            last_move: None,
            last_vehicle_move: None,
            last_chat_ack: None,
            last_chat: None,
            last_chat_command: None,
            last_signed_chat_command: None,
            last_chat_session_update: None,
            last_change_game_mode: None,
            last_attack: None,
            last_player_command: None,
            last_player_action: None,
            last_use_item: None,
            last_use_item_on: None,
            last_pong: None,
            last_jigsaw_generate: None,
            last_sign_update: None,
            last_set_beacon: None,
            last_set_command_block: None,
            last_set_command_minecart: None,
            last_set_structure_block: None,
            last_select_trade: None,
            last_rename_item: None,
            last_command_suggestion: None,
            last_edit_book: None,
            last_block_entity_tag_query: None,
            last_entity_tag_query: None,
            last_interact: None,
            last_resource_pack_response: None,
            last_container_close: None,
            last_container_button_click: None,
            last_container_click: None,
            last_container_slot_state_changed: None,
            last_set_creative_mode_slot: None,
            last_player_abilities: None,
            last_pick_item_from_block: None,
            last_pick_item_from_entity: None,
            last_recipe_book_change_settings: None,
            last_recipe_book_seen_recipe: None,
            loaded: false,
            disconnect_reason: None,
        }
    }

    pub fn join_sequence(&mut self, login: ClientboundLoginPacket) -> Vec<PlayInstruction> {
        self.state = PlayState::WaitingForPlayerLoaded;
        self.container_state_id = 0;
        vec![
            PlayInstruction::Login(login),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket {
                slot: self.selected_slot as i32,
            }),
            PlayInstruction::PlayerPosition { teleport_id: 0 },
        ]
    }

    pub fn vanilla_join_sequence(&mut self, settings: JoinGameSettings) -> Vec<PlayInstruction> {
        self.state = PlayState::WaitingForPlayerLoaded;
        self.container_state_id = 0;
        let mut instructions = vec![
            PlayInstruction::Login(settings.login),
            PlayInstruction::ChangeDifficulty {
                difficulty: settings.difficulty,
                locked: settings.difficulty_locked,
            },
            PlayInstruction::PlayerAbilities(settings.abilities),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket {
                slot: self.selected_slot as i32,
            }),
        ];
        if settings.initial_recipes {
            instructions.push(PlayInstruction::UpdateRecipes);
        }
        instructions.push(PlayInstruction::UpdatePermissionLevel(
            settings.permission_level,
        ));
        if settings.initial_recipe_book {
            instructions.push(PlayInstruction::SendInitialRecipeBook);
        }
        if settings.scoreboard {
            instructions.push(PlayInstruction::UpdateScoreboard);
        }
        instructions.push(PlayInstruction::TeleportToSpawn { teleport_id: 0 });
        if settings.server_status {
            instructions.push(PlayInstruction::ServerStatus);
        }
        instructions.push(PlayInstruction::PlayerInfoUpdate {
            existing_players: settings.player_info_existing_count,
        });
        instructions.push(PlayInstruction::BroadcastSelfPlayerInfo);
        instructions.push(PlayInstruction::SendLevelInfo);
        instructions.push(PlayInstruction::AddPlayerToLevel);
        instructions.push(PlayInstruction::BossEventsOnConnect);
        if settings.active_effect_count > 0 {
            instructions.push(PlayInstruction::ActiveEffects {
                count: settings.active_effect_count,
            });
        }
        instructions.push(PlayInstruction::InitInventoryMenu);
        instructions
    }

    pub fn apply_scripted_container_click(
        &mut self,
        menu: &mut Menu,
        packet: &ScriptedContainerClickPacket,
    ) -> InventoryTransactionResult {
        let result = apply_scripted_packet(menu, self.container_state_id, packet);
        if result.accepted {
            self.container_state_id = result.next_state_id;
        }
        result
    }

    /// Process the pending `last_container_click` against the player's `InventoryMenu`,
    /// advancing the state ID on accepted actions and returning `PlayInstruction`s for
    /// every slot that changed plus any recipe-book unlocks.
    ///
    /// Matches the server-side click dispatch in Java's
    /// `ServerGamePacketListenerImpl.handleContainerClick` + `AbstractContainerMenu.clicked`.
    pub fn process_pending_container_click(
        &mut self,
        inventory_menu: &mut InventoryMenu,
        carried: &mut ItemStack,
    ) -> Vec<PlayInstruction> {
        let Some(packet) = self.last_container_click.take() else {
            return Vec::new();
        };
        if packet.container_id != 0 {
            return Vec::new();
        }
        handle_container_click(
            &packet,
            &mut self.container_state_id,
            inventory_menu,
            carried,
        )
    }

    /// Build a `ClientboundContainerPacket` (ContainerSetContent) from the current
    /// InventoryMenu state.  Called by the server runtime when it processes the
    /// `PlayInstruction::InitInventoryMenu` signal.
    pub fn build_container_set_content(
        inventory_menu: &InventoryMenu,
        carried: &ItemStack,
        state_id: i32,
    ) -> io::Result<ClientboundContainerPacket> {
        let slots = inventory_menu
            .all_slots()
            .iter()
            .map(raw_item_stack_from_item_stack)
            .collect::<io::Result<Vec<_>>>()?;
        Ok(ClientboundContainerPacket {
            container_id: 0,
            state_id,
            slots,
            carried_item: raw_item_stack_from_item_stack(carried)?,
        })
    }

    pub fn handle_decoded(&mut self, packet: DecodedPacket) -> DispatchOutcome {
        if packet.state != ProtocolState::Play || packet.direction != PacketDirection::Serverbound {
            return DispatchOutcome::Disconnect(format!(
                "unexpected {:?} {:?} packet {} during play",
                packet.state, packet.direction, packet.id
            ));
        }

        if matches!(self.state, PlayState::WaitingForPlayerLoaded)
            && is_command_like_play_packet(packet.id)
        {
            return DispatchOutcome::Disconnect(format!(
                "command packet {} before player_loaded",
                packet.id
            ));
        }

        self.handle_play_packet(packet.id, &packet.payload)
    }

    fn handle_play_packet(&mut self, packet_id: i32, payload: &[u8]) -> DispatchOutcome {
        match packet_id {
            SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID => {
                self.handle_move_payload(payload, MoveShape::Pos)
            }
            SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID => {
                self.handle_move_payload(payload, MoveShape::PosRot)
            }
            SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID => {
                self.handle_move_payload(payload, MoveShape::Rot)
            }
            SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID => {
                self.handle_move_payload(payload, MoveShape::StatusOnly)
            }
            _ => {
                if let Some(outcome) = self.handle_control_or_chat_packet(packet_id, payload) {
                    return outcome;
                }
                if let Some(outcome) = self.handle_container_or_book_packet(packet_id, payload) {
                    return outcome;
                }
                if let Some(outcome) = self.handle_world_interaction_packet(packet_id, payload) {
                    return outcome;
                }
                if let Some(outcome) = self.handle_misc_play_packet(packet_id, payload) {
                    return outcome;
                }
                self.handle_unknown_play_packet(packet_id)
            }
        }
    }

    fn handle_control_or_chat_packet(
        &mut self,
        packet_id: i32,
        payload: &[u8],
    ) -> Option<DispatchOutcome> {
        Some(match packet_id {
            SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID => {
                match read_serverbound_play_packet(payload, "teleport ack", |input| {
                    ServerboundAcceptTeleportationPacket::read(input)
                }) {
                    Ok(ack) => {
                        self.pending_teleports.remove(&ack.teleport_id);
                        DispatchOutcome::Handled
                    }
                    Err(outcome) => outcome,
                }
            }
            SERVERBOUND_ATTACK_PACKET_ID => self.decode_and_store(
                payload,
                "attack packet",
                |input| ServerboundAttackPacket::read(input),
                |session, attack| session.last_attack = Some(attack),
            ),
            SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID => {
                self.decode_ignored(payload, "change difficulty packet", |input| {
                    ServerboundChangeDifficultyPacket::read(input)
                })
            }
            SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID => self.decode_and_store(
                payload,
                "change game mode packet",
                |input| ServerboundChangeGameModePacket::read(input),
                |session, packet| session.last_change_game_mode = Some(packet),
            ),
            SERVERBOUND_CHAT_ACK_PACKET_ID => self.decode_and_store(
                payload,
                "chat ack packet",
                |input| ServerboundChatAckPacket::read(input),
                |session, ack| session.last_chat_ack = Some(ack),
            ),
            SERVERBOUND_CHAT_COMMAND_PACKET_ID => self.decode_and_store(
                payload,
                "chat command packet",
                |input| ServerboundChatCommandPacket::read(input),
                |session, command| session.last_chat_command = Some(command),
            ),
            SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID => self.decode_and_store(
                payload,
                "signed chat command packet",
                |input| ServerboundChatCommandSignedPacket::read(input),
                |session, command| session.last_signed_chat_command = Some(command),
            ),
            SERVERBOUND_CHAT_PACKET_ID => self.decode_and_store(
                payload,
                "chat packet",
                |input| ServerboundChatPacket::read(input),
                |session, chat| session.last_chat = Some(chat),
            ),
            SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID => self.decode_and_store(
                payload,
                "chat session update packet",
                |input| ServerboundChatSessionUpdatePacket::read(input),
                |session, update| session.last_chat_session_update = Some(update),
            ),
            SERVERBOUND_CLIENT_COMMAND_PACKET_ID => {
                self.decode_ignored(payload, "client command packet", |input| {
                    ServerboundClientCommandPacket::read(input)
                })
            }
            SERVERBOUND_CLIENT_TICK_END_PACKET_ID => {
                self.decode_ignored(payload, "client tick end packet", |input| {
                    ServerboundClientTickEndPacket::read(input)
                })
            }
            SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID => {
                self.decode_ignored(payload, "chunk batch received", |input| {
                    ServerboundChunkBatchReceivedPacket::read(input)
                })
            }
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID => self.decode_and_store(
                payload,
                "command suggestion packet",
                |input| ServerboundCommandSuggestionPacket::read(input),
                |session, suggestion| session.last_command_suggestion = Some(suggestion),
            ),
            SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID => {
                match read_serverbound_play_packet(
                    payload,
                    "configuration acknowledged packet",
                    |input| ServerboundConfigurationAcknowledgedPacket::read(input),
                ) {
                    Ok(_) => {
                        self.state = PlayState::Reconfiguring;
                        DispatchOutcome::Handled
                    }
                    Err(outcome) => outcome,
                }
            }
            _ => return None,
        })
    }

    fn handle_container_or_book_packet(
        &mut self,
        packet_id: i32,
        payload: &[u8],
    ) -> Option<DispatchOutcome> {
        Some(match packet_id {
            SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID => self.decode_and_store(
                payload,
                "block entity tag query packet",
                |input| ServerboundBlockEntityTagQueryPacket::read(input),
                |session, query| session.last_block_entity_tag_query = Some(query),
            ),
            SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID => self.decode_and_store(
                payload,
                "container button click packet",
                |input| ServerboundContainerButtonClickPacket::read(input),
                |session, click| session.last_container_button_click = Some(click),
            ),
            SERVERBOUND_CONTAINER_CLICK_PACKET_ID => self.decode_and_store(
                payload,
                "container click packet",
                |input| ServerboundContainerClickPacket::read(input),
                |session, click| session.last_container_click = Some(click),
            ),
            SERVERBOUND_CONTAINER_CLOSE_PACKET_ID => self.decode_and_store(
                payload,
                "container close packet",
                |input| ServerboundContainerClosePacket::read(input),
                |session, close| session.last_container_close = Some(close),
            ),
            SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED_PACKET_ID => self.decode_and_store(
                payload,
                "container slot state changed packet",
                |input| ServerboundContainerSlotStateChangedPacket::read(input),
                |session, pkt| session.last_container_slot_state_changed = Some(pkt),
            ),
            SERVERBOUND_EDIT_BOOK_PACKET_ID => self.decode_and_store(
                payload,
                "edit book packet",
                |input| ServerboundEditBookPacket::read(input),
                |session, book| session.last_edit_book = Some(book),
            ),
            SERVERBOUND_ENTITY_TAG_QUERY_PACKET_ID => self.decode_and_store(
                payload,
                "entity tag query packet",
                |input| ServerboundEntityTagQueryPacket::read(input),
                |session, query| session.last_entity_tag_query = Some(query),
            ),
            _ => return None,
        })
    }

    fn handle_world_interaction_packet(
        &mut self,
        packet_id: i32,
        payload: &[u8],
    ) -> Option<DispatchOutcome> {
        Some(match packet_id {
            SERVERBOUND_INTERACT_PACKET_ID => self.decode_and_store(
                payload,
                "interact packet",
                |input| ServerboundInteractPacket::read(input),
                |session, interact| session.last_interact = Some(interact),
            ),
            SERVERBOUND_JIGSAW_GENERATE_PACKET_ID => self.decode_and_store(
                payload,
                "jigsaw generate packet",
                |input| ServerboundJigsawGeneratePacket::read(input),
                |session, jigsaw| session.last_jigsaw_generate = Some(jigsaw),
            ),
            SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID => {
                self.decode_ignored(payload, "lock difficulty packet", |input| {
                    ServerboundLockDifficultyPacket::read(input)
                })
            }
            SERVERBOUND_MOVE_VEHICLE_PACKET_ID => self.decode_and_store(
                payload,
                "vehicle movement packet",
                |input| ServerboundMoveVehiclePacket::read(input),
                |session, packet| session.last_vehicle_move = Some(packet),
            ),
            SERVERBOUND_PADDLE_BOAT_PACKET_ID => {
                self.decode_ignored(payload, "paddle boat packet", |input| {
                    ServerboundPaddleBoatPacket::read(input)
                })
            }
            SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID => self.decode_and_store(
                payload,
                "pick item from block packet",
                |input| ServerboundPickItemFromBlockPacket::read(input),
                |session, pick| session.last_pick_item_from_block = Some(pick),
            ),
            SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID => self.decode_and_store(
                payload,
                "pick item from entity packet",
                |input| ServerboundPickItemFromEntityPacket::read(input),
                |session, pick| session.last_pick_item_from_entity = Some(pick),
            ),
            SERVERBOUND_PLAYER_INPUT_PACKET_ID => {
                self.decode_ignored(payload, "player input packet", |input| {
                    ServerboundPlayerInputPacket::read(input)
                })
            }
            SERVERBOUND_PLAYER_LOADED_PACKET_ID => {
                match read_serverbound_play_packet(payload, "player loaded packet", |input| {
                    ServerboundPlayerLoadedPacket::read(input)
                }) {
                    Ok(_) => {
                        self.loaded = true;
                        self.state = PlayState::Playing;
                        DispatchOutcome::Handled
                    }
                    Err(outcome) => outcome,
                }
            }
            SERVERBOUND_PLAYER_COMMAND_PACKET_ID => self.decode_and_store(
                payload,
                "player command packet",
                |input| ServerboundPlayerCommandPacket::read(input),
                |session, command| session.last_player_command = Some(command),
            ),
            SERVERBOUND_PLAYER_ACTION_PACKET_ID => self.decode_and_store(
                payload,
                "player action packet",
                |input| ServerboundPlayerActionPacket::read(input),
                |session, action| session.last_player_action = Some(action),
            ),
            SERVERBOUND_PLAYER_ABILITIES_PACKET_ID => self.decode_and_store(
                payload,
                "player abilities packet",
                |input| ServerboundPlayerAbilitiesPacket::read(input),
                |session, abilities| session.last_player_abilities = Some(abilities),
            ),
            SERVERBOUND_PONG_PACKET_ID => self.decode_and_store(
                payload,
                "pong packet",
                |input| ServerboundPongPacket::read(input),
                |session, pong| session.last_pong = Some(pong),
            ),
            _ => return None,
        })
    }

    fn handle_misc_play_packet(
        &mut self,
        packet_id: i32,
        payload: &[u8],
    ) -> Option<DispatchOutcome> {
        Some(match packet_id {
            SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID => self.decode_and_store(
                payload,
                "recipe book change settings packet",
                |input| ServerboundRecipeBookChangeSettingsPacket::read(input),
                |session, settings| session.last_recipe_book_change_settings = Some(settings),
            ),
            SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID => self.decode_and_store(
                payload,
                "recipe book seen recipe packet",
                |input| ServerboundRecipeBookSeenRecipePacket::read(input),
                |session, recipe| session.last_recipe_book_seen_recipe = Some(recipe),
            ),
            SERVERBOUND_RENAME_ITEM_PACKET_ID => self.decode_and_store(
                payload,
                "rename item packet",
                |input| ServerboundRenameItemPacket::read(input),
                |session, rename_item| session.last_rename_item = Some(rename_item),
            ),
            SERVERBOUND_RESOURCE_PACK_PACKET_ID => {
                let mut input = payload;
                match ServerboundResourcePackPacket::read(&mut input) {
                    Ok(response) if input.is_empty() => {
                        self.last_resource_pack_response = Some(response);
                        DispatchOutcome::Handled
                    }
                    Ok(_) => DispatchOutcome::Disconnect(
                        "bad resource pack packet: trailing payload".to_string(),
                    ),
                    Err(err) => bad_play_packet("resource pack packet", err),
                }
            }
            SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID => {
                match read_serverbound_play_packet(payload, "carried item packet", |input| {
                    ServerboundSetCarriedItemPacket::read(input)
                }) {
                    Ok(held) => {
                        if (0..=8).contains(&held.slot) {
                            self.selected_slot = held.slot;
                        }
                        DispatchOutcome::Handled
                    }
                    Err(outcome) => outcome,
                }
            }
            SERVERBOUND_SET_BEACON_PACKET_ID => {
                let mut input = payload;
                match ServerboundSetBeaconPacket::read(&mut input) {
                    Ok(beacon) if input.is_empty() => {
                        self.last_set_beacon = Some(beacon);
                        DispatchOutcome::Handled
                    }
                    Ok(_) => DispatchOutcome::Disconnect(
                        "bad set beacon packet: trailing payload".to_string(),
                    ),
                    Err(err) => bad_play_packet("set beacon packet", err),
                }
            }
            _ => return self.handle_command_or_item_play_packet(packet_id, payload),
        })
    }

    fn handle_command_or_item_play_packet(
        &mut self,
        packet_id: i32,
        payload: &[u8],
    ) -> Option<DispatchOutcome> {
        Some(match packet_id {
            SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID => self.decode_and_store(
                payload,
                "set command block packet",
                |input| ServerboundSetCommandBlockPacket::read(input),
                |session, command| session.last_set_command_block = Some(command),
            ),
            SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID => self.decode_and_store(
                payload,
                "set command minecart packet",
                |input| ServerboundSetCommandMinecartPacket::read(input),
                |session, command| session.last_set_command_minecart = Some(command),
            ),
            SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID => self.decode_and_store(
                payload,
                "set creative mode slot packet",
                |input| ServerboundSetCreativeModeSlotPacket::read(input),
                |session, slot| session.last_set_creative_mode_slot = Some(slot),
            ),
            SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID => self.decode_and_store(
                payload,
                "set structure block packet",
                |input| ServerboundSetStructureBlockPacket::read(input),
                |session, structure| session.last_set_structure_block = Some(structure),
            ),
            SERVERBOUND_SELECT_TRADE_PACKET_ID => self.decode_and_store(
                payload,
                "select trade packet",
                |input| ServerboundSelectTradePacket::read(input),
                |session, select_trade| session.last_select_trade = Some(select_trade),
            ),
            SERVERBOUND_SIGN_UPDATE_PACKET_ID => self.decode_and_store(
                payload,
                "sign update packet",
                |input| ServerboundSignUpdatePacket::read(input),
                |session, sign_update| session.last_sign_update = Some(sign_update),
            ),
            SERVERBOUND_SWING_PACKET_ID => self.decode_ignored(payload, "swing packet", |input| {
                ServerboundSwingPacket::read(input)
            }),
            SERVERBOUND_USE_ITEM_PACKET_ID => self.decode_and_store(
                payload,
                "use item packet",
                |input| ServerboundUseItemPacket::read(input),
                |session, use_item| session.last_use_item = Some(use_item),
            ),
            SERVERBOUND_USE_ITEM_ON_PACKET_ID => self.decode_and_store(
                payload,
                "use item on packet",
                |input| ServerboundUseItemOnPacket::read(input),
                |session, use_item_on| session.last_use_item_on = Some(use_item_on),
            ),
            _ => return None,
        })
    }

    fn handle_unknown_play_packet(&self, packet_id: i32) -> DispatchOutcome {
        if PlayProtocolRegistry::new().is_serverbound_play_packet(packet_id) {
            DispatchOutcome::Handled
        } else {
            DispatchOutcome::Disconnect(format!("unknown play packet id {packet_id}"))
        }
    }

    fn decode_and_store<T>(
        &mut self,
        payload: &[u8],
        packet_name: &'static str,
        read: impl FnOnce(&mut &[u8]) -> io::Result<T>,
        store: impl FnOnce(&mut Self, T),
    ) -> DispatchOutcome {
        match read_serverbound_play_packet(payload, packet_name, read) {
            Ok(packet) => {
                store(self, packet);
                DispatchOutcome::Handled
            }
            Err(outcome) => outcome,
        }
    }

    fn decode_ignored<T>(
        &mut self,
        payload: &[u8],
        packet_name: &'static str,
        read: impl FnOnce(&mut &[u8]) -> io::Result<T>,
    ) -> DispatchOutcome {
        match read_serverbound_play_packet(payload, packet_name, read) {
            Ok(_) => DispatchOutcome::Handled,
            Err(outcome) => outcome,
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

    pub fn death_screen(&self, message: impl Into<String>) -> PlayInstruction {
        PlayInstruction::CombatKill(ClientboundPlayerCombatKillPacket {
            player_id: self.entity_id,
            message: message.into(),
        })
    }

    pub fn respawn_flow(&mut self, request: RespawnRequest) -> Vec<PlayInstruction> {
        let mut instructions = Vec::new();
        if request.missing_respawn_block {
            instructions.push(PlayInstruction::NoRespawnBlockAvailable);
        }
        instructions.push(PlayInstruction::Respawn(ClientboundRespawnPacket {
            spawn_info: request.spawn_info,
            data_to_keep: if request.keep_all_player_data {
                RespawnDataToKeep::KEEP_ATTRIBUTE_MODIFIERS
            } else {
                RespawnDataToKeep::NONE
            },
        }));
        instructions.push(PlayInstruction::TeleportToSpawn { teleport_id: 0 });
        instructions.push(PlayInstruction::SetDefaultSpawnPosition);
        instructions.push(PlayInstruction::ChangeDifficulty {
            difficulty: GameDifficulty::Normal,
            locked: false,
        });
        instructions.push(PlayInstruction::SetExperience);
        if request.active_effect_count > 0 {
            instructions.push(PlayInstruction::ActiveEffects {
                count: request.active_effect_count,
            });
        }
        instructions.push(PlayInstruction::SendLevelInfo);
        instructions.push(PlayInstruction::UpdatePermissionLevel(0));
        instructions.push(PlayInstruction::AddPlayerToLevel);
        instructions.push(PlayInstruction::InitInventoryMenu);
        instructions.push(PlayInstruction::SetHealth);
        if matches!(request.reason, RespawnReason::Death) && request.hardcore {
            instructions.push(PlayInstruction::SetGameModeSpectator);
            instructions.push(PlayInstruction::DisableSpectatorsGenerateChunks);
        }
        if request.respawn_anchor_depleted {
            instructions.push(PlayInstruction::RespawnAnchorDepleteSound);
        }
        instructions
    }

    pub(super) fn handle_move_payload(
        &mut self,
        payload: &[u8],
        shape: MoveShape,
    ) -> DispatchOutcome {
        let mut input = payload;
        match ServerboundMovePlayerPacket::read_shape(&mut input, shape) {
            Ok(packet) => {
                self.last_move = Some(packet);
                DispatchOutcome::Handled
            }
            Err(err) => DispatchOutcome::Disconnect(format!("bad movement packet: {err}")),
        }
    }
}

fn read_serverbound_play_packet<T>(
    payload: &[u8],
    packet_name: &'static str,
    read: impl FnOnce(&mut &[u8]) -> io::Result<T>,
) -> Result<T, DispatchOutcome> {
    let mut input = payload;
    read(&mut input).map_err(|err| bad_play_packet(packet_name, err))
}

fn bad_play_packet(packet_name: &str, err: io::Error) -> DispatchOutcome {
    DispatchOutcome::Disconnect(format!("bad {packet_name}: {err}"))
}

pub(super) fn is_command_like_play_packet(packet_id: i32) -> bool {
    matches!(
        packet_id,
        SERVERBOUND_CHAT_COMMAND_PACKET_ID
            | SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID
            | SERVERBOUND_CHAT_PACKET_ID
            | SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID
    )
}

/// A batch of chunks ready to be flushed to the client this tick.
///
/// Returned by [`PlayerChunkSender::send_next_chunks`]. Each entry is a
/// position plus the generated chunk payload pulled from the shared chunk
/// pipeline at the moment the batch was collected. Java mirror:
/// `PlayerChunkSender.collectChunksToSend` returns `List<LevelChunk>`.
#[derive(Debug)]
pub struct ReadyChunkBatch {
    pub chunks: Vec<(ChunkPos, Arc<LevelChunk>)>,
}

impl PlayerChunkSender {
    pub const MIN_CHUNKS_PER_TICK: f32 = 0.01;
    pub const MAX_CHUNKS_PER_TICK: f32 = 64.0;
    pub const START_CHUNKS_PER_TICK: f32 = 9.0;
    pub const MAX_UNACKNOWLEDGED_BATCHES_AFTER_ACK: i32 = 10;

    pub fn new(memory_connection: bool) -> Self {
        Self {
            pending_chunks: BTreeSet::new(),
            memory_connection,
            desired_chunks_per_tick: Self::START_CHUNKS_PER_TICK,
            batch_quota: 0.0,
            unacknowledged_batches: 0,
            max_unacknowledged_batches: 1,
        }
    }

    pub fn mark_chunk_pending_to_send(&mut self, pos: ChunkPos) {
        self.pending_chunks.insert(pos);
    }

    pub fn drop_chunk(&mut self, pos: ChunkPos, player_alive: bool) -> Option<PlayInstruction> {
        if self.pending_chunks.remove(&pos) || !player_alive {
            None
        } else {
            Some(PlayInstruction::ForgetLevelChunk { pos })
        }
    }

    /// Drain the next paced batch of *ready* chunks for the client.
    ///
    /// `try_get_ready(pos)` reports whether the chunk at `pos` has finished
    /// generating in the shared pipeline; chunks for which it returns `None`
    /// stay pending across ticks. Mirrors Java
    /// `PlayerChunkSender.sendNextChunks`: this is the central pacing decision
    /// (unacknowledged-batch gate, quota accumulation, nearest-first selection)
    /// and it never blocks waiting for generation.
    pub fn send_next_chunks<F>(
        &mut self,
        player_pos: ChunkPos,
        mut try_get_ready: F,
    ) -> Option<ReadyChunkBatch>
    where
        F: FnMut(ChunkPos) -> Option<Arc<LevelChunk>>,
    {
        if self.unacknowledged_batches >= self.max_unacknowledged_batches {
            return None;
        }

        // Java: batchQuota = min(batchQuota + desiredChunksPerTick, max(1, desiredChunksPerTick)).
        // The quota accumulates whether or not we end up flushing chunks this
        // tick, so a low desiredChunksPerTick can still pay down over time.
        let max_batch_size = self.desired_chunks_per_tick.max(1.0);
        self.batch_quota = (self.batch_quota + self.desired_chunks_per_tick).min(max_batch_size);
        if self.batch_quota < 1.0 || self.pending_chunks.is_empty() {
            return None;
        }

        let chunks = self.collect_chunks_to_send(player_pos, &mut try_get_ready);
        if chunks.is_empty() {
            return None;
        }

        // Java only increments unacknowledgedBatches when chunks are actually
        // sent — see PlayerChunkSender.sendNextChunks where the increment is
        // inside `if (!chunksToSend.isEmpty())`.
        self.unacknowledged_batches += 1;
        self.batch_quota -= chunks.len() as f32;
        Some(ReadyChunkBatch { chunks })
    }

    pub fn on_chunk_batch_received_by_client(&mut self, desired_chunks_per_tick: f32) {
        self.unacknowledged_batches -= 1;
        self.desired_chunks_per_tick = if desired_chunks_per_tick.is_nan() {
            Self::MIN_CHUNKS_PER_TICK
        } else {
            desired_chunks_per_tick.clamp(Self::MIN_CHUNKS_PER_TICK, Self::MAX_CHUNKS_PER_TICK)
        };
        if self.unacknowledged_batches == 0 {
            self.batch_quota = 1.0;
        }
        self.max_unacknowledged_batches = Self::MAX_UNACKNOWLEDGED_BATCHES_AFTER_ACK;
    }

    pub fn is_pending(&self, pos: ChunkPos) -> bool {
        self.pending_chunks.contains(&pos)
    }

    pub fn pending_count(&self) -> usize {
        self.pending_chunks.len()
    }

    pub fn desired_chunks_per_tick(&self) -> f32 {
        self.desired_chunks_per_tick
    }

    pub fn unacknowledged_batches(&self) -> i32 {
        self.unacknowledged_batches
    }

    pub fn max_unacknowledged_batches(&self) -> i32 {
        self.max_unacknowledged_batches
    }

    pub fn batch_quota(&self) -> f32 {
        self.batch_quota
    }

    pub(super) fn collect_chunks_to_send<F>(
        &mut self,
        player_pos: ChunkPos,
        try_get_ready: &mut F,
    ) -> Vec<(ChunkPos, Arc<LevelChunk>)>
    where
        F: FnMut(ChunkPos) -> Option<Arc<LevelChunk>>,
    {
        let max_batch_size = self.batch_quota.floor() as usize;
        // Java PlayerChunkSender.collectChunksToSend (26.1.2):
        //   • When pending > maxBatchSize and not a memory connection,
        //     pick the nearest maxBatchSize *positions first*, then look up
        //     readiness — unready picks are silently dropped this tick, so
        //     fewer than maxBatchSize chunks may actually be sent.
        //   • Otherwise, look up every pending position, drop unready, and
        //     sort by distance.
        // The "pick nearest positions first" path intentionally lets close
        // chunks block farther ready chunks so the player doesn't see a halo
        // of distant terrain while the near ring is still loading.
        let chunks = if !self.memory_connection && self.pending_chunks.len() > max_batch_size {
            let mut nearest_positions: Vec<_> = self.pending_chunks.iter().copied().collect();
            nearest_positions.sort_by_key(|pos| (chunk_distance_squared(player_pos, *pos), *pos));
            nearest_positions.truncate(max_batch_size);
            nearest_positions
                .into_iter()
                .filter_map(|pos| try_get_ready(pos).map(|chunk| (pos, chunk)))
                .collect::<Vec<_>>()
        } else {
            let mut chunks: Vec<_> = self
                .pending_chunks
                .iter()
                .copied()
                .filter_map(|pos| try_get_ready(pos).map(|chunk| (pos, chunk)))
                .collect();
            chunks.sort_by_key(|(pos, _)| (chunk_distance_squared(player_pos, *pos), *pos));
            chunks
        };

        for (pos, _) in &chunks {
            self.pending_chunks.remove(pos);
        }
        chunks
    }
}

pub(super) fn chunk_distance_squared(from: ChunkPos, to: ChunkPos) -> i32 {
    let dx = from.x - to.x;
    let dz = from.z - to.z;
    dx * dx + dz * dz
}

pub(super) fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    Ok(read_u8(reader)? != 0)
}

pub(super) fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

pub(super) fn read_nullable_signature<R: Read>(
    reader: &mut R,
) -> io::Result<Option<MessageSignature>> {
    if read_bool(reader)? {
        Ok(Some(MessageSignature::read(reader)?))
    } else {
        Ok(None)
    }
}

pub(super) fn write_nullable_signature<W: Write>(
    writer: &mut W,
    signature: Option<&MessageSignature>,
) -> io::Result<()> {
    match signature {
        Some(signature) => {
            write_bool(writer, true)?;
            signature.write(writer)
        }
        None => write_bool(writer, false),
    }
}

const BLOCK_POS_PACKED_HORIZONTAL_LENGTH: i64 = 26;
const BLOCK_POS_PACKED_Y_LENGTH: i64 = 12;
const BLOCK_POS_PACKED_X_MASK: i64 = (1_i64 << BLOCK_POS_PACKED_HORIZONTAL_LENGTH) - 1;
const BLOCK_POS_PACKED_Y_MASK: i64 = (1_i64 << BLOCK_POS_PACKED_Y_LENGTH) - 1;
const BLOCK_POS_PACKED_Z_MASK: i64 = (1_i64 << BLOCK_POS_PACKED_HORIZONTAL_LENGTH) - 1;
const BLOCK_POS_X_OFFSET: i64 = BLOCK_POS_PACKED_Y_LENGTH + BLOCK_POS_PACKED_HORIZONTAL_LENGTH;
const BLOCK_POS_Z_OFFSET: i64 = BLOCK_POS_PACKED_Y_LENGTH;

pub fn pack_block_position(x: i32, y: i32, z: i32) -> i64 {
    ((x as i64 & BLOCK_POS_PACKED_X_MASK) << BLOCK_POS_X_OFFSET)
        | ((z as i64 & BLOCK_POS_PACKED_Z_MASK) << BLOCK_POS_Z_OFFSET)
        | (y as i64 & BLOCK_POS_PACKED_Y_MASK)
}

pub fn unpack_block_position(packed: i64) -> (i32, i32, i32) {
    let x = (packed << (64 - (BLOCK_POS_X_OFFSET + BLOCK_POS_PACKED_HORIZONTAL_LENGTH))
        >> (64 - BLOCK_POS_PACKED_HORIZONTAL_LENGTH)) as i32;
    let y = (packed << (64 - BLOCK_POS_PACKED_Y_LENGTH) >> (64 - BLOCK_POS_PACKED_Y_LENGTH)) as i32;
    let z = (packed << (64 - (BLOCK_POS_Z_OFFSET + BLOCK_POS_PACKED_HORIZONTAL_LENGTH))
        >> (64 - BLOCK_POS_PACKED_HORIZONTAL_LENGTH)) as i32;
    (x, y, z)
}

pub(super) fn read_block_position<R: Read>(reader: &mut R) -> io::Result<(i32, i32, i32)> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(unpack_block_position(i64::from_be_bytes(bytes)))
}

pub(super) fn write_block_position<W: Write>(
    writer: &mut W,
    x: i32,
    y: i32,
    z: i32,
) -> io::Result<()> {
    writer.write_all(&pack_block_position(x, y, z).to_be_bytes())
}
