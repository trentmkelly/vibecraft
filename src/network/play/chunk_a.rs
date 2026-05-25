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
            last_interact: None,
            last_resource_pack_response: None,
            last_container_close: None,
            last_container_button_click: None,
            last_container_click: None,
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

        match packet.id {
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
            SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChangeDifficultyPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad change difficulty packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CHAT_ACK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatAckPacket::read(&mut input) {
                    Ok(ack) => {
                        self.last_chat_ack = Some(ack);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad chat ack packet: {err}")),
                }
            }
            SERVERBOUND_CHAT_COMMAND_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatCommandPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_chat_command = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad chat command packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatCommandSignedPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_signed_chat_command = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad signed chat command packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CHAT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatPacket::read(&mut input) {
                    Ok(chat) => {
                        self.last_chat = Some(chat);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad chat packet: {err}")),
                }
            }
            SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatSessionUpdatePacket::read(&mut input) {
                    Ok(update) => {
                        self.last_chat_session_update = Some(update);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad chat session update packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CLIENT_COMMAND_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundClientCommandPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad client command packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CLIENT_TICK_END_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundClientTickEndPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad client tick end packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChunkBatchReceivedPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad chunk batch received: {err}"))
                    }
                }
            }
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundCommandSuggestionPacket::read(&mut input) {
                    Ok(suggestion) => {
                        self.last_command_suggestion = Some(suggestion);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad command suggestion packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundConfigurationAcknowledgedPacket::read(&mut input) {
                    Ok(_) => {
                        self.state = PlayState::Reconfiguring;
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad configuration acknowledged packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundContainerButtonClickPacket::read(&mut input) {
                    Ok(click) => {
                        self.last_container_button_click = Some(click);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad container button click packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CONTAINER_CLICK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundContainerClickPacket::read(&mut input) {
                    Ok(click) => {
                        self.last_container_click = Some(click);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad container click packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CONTAINER_CLOSE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundContainerClosePacket::read(&mut input) {
                    Ok(close) => {
                        self.last_container_close = Some(close);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad container close packet: {err}"))
                    }
                }
            }
            SERVERBOUND_EDIT_BOOK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundEditBookPacket::read(&mut input) {
                    Ok(book) => {
                        self.last_edit_book = Some(book);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad edit book packet: {err}")),
                }
            }
            SERVERBOUND_INTERACT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundInteractPacket::read(&mut input) {
                    Ok(interact) => {
                        self.last_interact = Some(interact);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad interact packet: {err}")),
                }
            }
            SERVERBOUND_JIGSAW_GENERATE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundJigsawGeneratePacket::read(&mut input) {
                    Ok(jigsaw) => {
                        self.last_jigsaw_generate = Some(jigsaw);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad jigsaw generate packet: {err}"))
                    }
                }
            }
            SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundLockDifficultyPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad lock difficulty packet: {err}"))
                    }
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
            SERVERBOUND_MOVE_VEHICLE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundMoveVehiclePacket::read(&mut input) {
                    Ok(packet) => {
                        self.last_vehicle_move = Some(packet);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad vehicle movement packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PADDLE_BOAT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPaddleBoatPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad paddle boat packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPickItemFromBlockPacket::read(&mut input) {
                    Ok(pick) => {
                        self.last_pick_item_from_block = Some(pick);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad pick item from block packet: {err}"
                    )),
                }
            }
            SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPickItemFromEntityPacket::read(&mut input) {
                    Ok(pick) => {
                        self.last_pick_item_from_entity = Some(pick);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad pick item from entity packet: {err}"
                    )),
                }
            }
            SERVERBOUND_PLAYER_INPUT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerInputPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player input packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PLAYER_LOADED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerLoadedPacket::read(&mut input) {
                    Ok(_) => {
                        self.loaded = true;
                        self.state = PlayState::Playing;
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player loaded packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PLAYER_COMMAND_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerCommandPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_player_command = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player command packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PLAYER_ACTION_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerActionPacket::read(&mut input) {
                    Ok(action) => {
                        self.last_player_action = Some(action);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player action packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PLAYER_ABILITIES_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerAbilitiesPacket::read(&mut input) {
                    Ok(abilities) => {
                        self.last_player_abilities = Some(abilities);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player abilities packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PONG_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPongPacket::read(&mut input) {
                    Ok(pong) => {
                        self.last_pong = Some(pong);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad pong packet: {err}")),
                }
            }
            SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundRecipeBookChangeSettingsPacket::read(&mut input) {
                    Ok(settings) => {
                        self.last_recipe_book_change_settings = Some(settings);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad recipe book change settings packet: {err}"
                    )),
                }
            }
            SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundRecipeBookSeenRecipePacket::read(&mut input) {
                    Ok(recipe) => {
                        self.last_recipe_book_seen_recipe = Some(recipe);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad recipe book seen recipe packet: {err}"
                    )),
                }
            }
            SERVERBOUND_RENAME_ITEM_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundRenameItemPacket::read(&mut input) {
                    Ok(rename_item) => {
                        self.last_rename_item = Some(rename_item);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad rename item packet: {err}"))
                    }
                }
            }
            SERVERBOUND_RESOURCE_PACK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundResourcePackPacket::read(&mut input) {
                    Ok(response) => {
                        self.last_resource_pack_response = Some(response);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad resource pack packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCarriedItemPacket::read(&mut input) {
                    Ok(held) => {
                        if (0..=8).contains(&held.slot) {
                            self.selected_slot = held.slot;
                        }
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad carried item packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SET_BEACON_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetBeaconPacket::read(&mut input) {
                    Ok(beacon) => {
                        self.last_set_beacon = Some(beacon);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad set beacon packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCommandBlockPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_set_command_block = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad set command block packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCommandMinecartPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_set_command_minecart = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad set command minecart packet: {err}"
                    )),
                }
            }
            SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCreativeModeSlotPacket::read(&mut input) {
                    Ok(slot) => {
                        self.last_set_creative_mode_slot = Some(slot);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad set creative mode slot packet: {err}"
                    )),
                }
            }
            SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetStructureBlockPacket::read(&mut input) {
                    Ok(structure) => {
                        self.last_set_structure_block = Some(structure);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad set structure block packet: {err}"
                    )),
                }
            }
            SERVERBOUND_SELECT_TRADE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSelectTradePacket::read(&mut input) {
                    Ok(select_trade) => {
                        self.last_select_trade = Some(select_trade);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad select trade packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SIGN_UPDATE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSignUpdatePacket::read(&mut input) {
                    Ok(sign_update) => {
                        self.last_sign_update = Some(sign_update);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad sign update packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SWING_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSwingPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => DispatchOutcome::Disconnect(format!("bad swing packet: {err}")),
                }
            }
            SERVERBOUND_USE_ITEM_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundUseItemPacket::read(&mut input) {
                    Ok(use_item) => {
                        self.last_use_item = Some(use_item);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad use item packet: {err}")),
                }
            }
            SERVERBOUND_USE_ITEM_ON_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundUseItemOnPacket::read(&mut input) {
                    Ok(use_item_on) => {
                        self.last_use_item_on = Some(use_item_on);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad use item on packet: {err}"))
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

    pub(super) fn handle_move_payload(&mut self, payload: Vec<u8>, shape: MoveShape) -> DispatchOutcome {
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
            nearest_positions
                .sort_by_key(|pos| (chunk_distance_squared(player_pos, *pos), *pos));
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

pub(super) fn read_nullable_signature<R: Read>(reader: &mut R) -> io::Result<Option<MessageSignature>> {
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

pub(super) fn write_block_position<W: Write>(writer: &mut W, x: i32, y: i32, z: i32) -> io::Result<()> {
    writer.write_all(&pack_block_position(x, y, z).to_be_bytes())
}
