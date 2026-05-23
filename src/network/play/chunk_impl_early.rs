use super::*;

impl GameDifficulty {
    pub(super) fn from_wire_index(index: i32) -> io::Result<Self> {
        match index {
            0 => Ok(Self::Peaceful),
            1 => Ok(Self::Easy),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Hard),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid difficulty index",
            )),
        }
    }

    pub(super) fn to_wire_index(self) -> i32 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub may_fly: bool,
    pub instabuild: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinGameSettings {
    pub login: ClientboundLoginPacket,
    pub difficulty: GameDifficulty,
    pub difficulty_locked: bool,
    pub abilities: PlayerAbilities,
    pub permission_level: u8,
    pub initial_recipes: bool,
    pub initial_recipe_book: bool,
    pub scoreboard: bool,
    pub server_status: bool,
    pub player_info_existing_count: usize,
    pub active_effect_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayInstruction {
    Login(ClientboundLoginPacket),
    ChangeDifficulty {
        difficulty: GameDifficulty,
        locked: bool,
    },
    PlayerAbilities(PlayerAbilities),
    SetHeldSlot(ClientboundSetHeldSlotPacket),
    UpdateRecipes,
    UpdatePermissionLevel(u8),
    SendInitialRecipeBook,
    UpdateScoreboard,
    TeleportToSpawn {
        teleport_id: i32,
    },
    ServerStatus,
    PlayerInfoUpdate {
        existing_players: usize,
    },
    BroadcastSelfPlayerInfo,
    SendLevelInfo,
    AddPlayerToLevel,
    BossEventsOnConnect,
    ActiveEffects {
        count: usize,
    },
    InitInventoryMenu,
    ChunkBatchStart,
    LevelChunkWithLight(ClientboundLevelChunkWithLightPacket),
    ChunkBatchFinished(ClientboundChunkBatchFinishedPacket),
    ForgetLevelChunk {
        pos: ChunkPos,
    },
    AddEntity(ClientboundAddEntityPacket),
    SetEntityData(ClientboundSetEntityDataPacket),
    SetEntityMotion(ClientboundSetEntityMotionPacket),
    SetEquipment(ClientboundSetEquipmentPacket),
    UpdateAttributes(ClientboundUpdateAttributesPacket),
    UpdateMobEffect(ClientboundUpdateMobEffectPacket),
    RemoveEntities(ClientboundRemoveEntitiesPacket),
    MoveEntity(ClientboundMoveEntityPacket),
    TeleportEntity(ClientboundTeleportEntityPacket),
    SetPassengers(ClientboundSetPassengersPacket),
    SetEntityLink(ClientboundSetEntityLinkPacket),
    RotateHead(ClientboundRotateHeadPacket),
    Animate(ClientboundAnimatePacket),
    Container(ClientboundContainerPacket),
    ContainerSetSlot(ClientboundContainerSetSlotPacket),
    /// Recipes that were first crafted in the last click; the caller is responsible for
    /// converting each ID to a full `ClientboundRecipeBookAddPacket` via the recipe registry
    /// (notification=true, highlight=true).
    RecipesUnlocked(Vec<&'static str>),
    SetCursorItem(ClientboundSetCursorItemPacket),
    RecipeBookAdd(ClientboundRecipeBookAddPacket),
    MerchantOffers(ClientboundMerchantOffersPacket),
    Recipes(ClientboundRecipePacket),
    Advancements(ClientboundAdvancementsPacket),
    AwardStats(ClientboundAwardStatsPacket),
    GameRuleValues(ClientboundGameRuleValuesPacket),
    Scoreboard(ClientboundScoreboardPacket),
    BossEvent(ClientboundBossEventPacket),
    Title(ClientboundTitlePacket),
    Sound(ClientboundSoundPacket),
    Particle(ClientboundParticlePacket),
    Explode(ClientboundExplodePacket),
    MapItemData(ClientboundMapItemDataPacket),
    WorldBorder(ClientboundWorldBorderPacket),
    Commands(ClientboundCommandsPacket),
    CommandSuggestions(ClientboundCommandSuggestionsPacket),
    Debug(ClientboundDebugPacket),
    CombatKill(ClientboundPlayerCombatKillPacket),
    NoRespawnBlockAvailable,
    Respawn(ClientboundRespawnPacket),
    SetDefaultSpawnPosition,
    SetExperience,
    SetHealth,
    SetGameModeSpectator,
    DisableSpectatorsGenerateChunks,
    RespawnAnchorDepleteSound,
    PlayerPosition {
        teleport_id: i32,
    },
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
    pub container_state_id: i32,
    pub pending_teleports: BTreeSet<i32>,
    pub last_move: Option<ServerboundMovePlayerPacket>,
    pub last_vehicle_move: Option<ServerboundMoveVehiclePacket>,
    pub last_chat_ack: Option<ServerboundChatAckPacket>,
    pub last_chat: Option<ServerboundChatPacket>,
    pub last_chat_command: Option<ServerboundChatCommandPacket>,
    pub last_signed_chat_command: Option<ServerboundChatCommandSignedPacket>,
    pub last_chat_session_update: Option<ServerboundChatSessionUpdatePacket>,
    pub last_player_command: Option<ServerboundPlayerCommandPacket>,
    pub last_player_action: Option<ServerboundPlayerActionPacket>,
    pub last_use_item: Option<ServerboundUseItemPacket>,
    pub last_use_item_on: Option<ServerboundUseItemOnPacket>,
    pub last_pong: Option<ServerboundPongPacket>,
    pub last_jigsaw_generate: Option<ServerboundJigsawGeneratePacket>,
    pub last_sign_update: Option<ServerboundSignUpdatePacket>,
    pub last_set_beacon: Option<ServerboundSetBeaconPacket>,
    pub last_set_command_block: Option<ServerboundSetCommandBlockPacket>,
    pub last_set_command_minecart: Option<ServerboundSetCommandMinecartPacket>,
    pub last_set_structure_block: Option<ServerboundSetStructureBlockPacket>,
    pub last_select_trade: Option<ServerboundSelectTradePacket>,
    pub last_rename_item: Option<ServerboundRenameItemPacket>,
    pub last_command_suggestion: Option<ServerboundCommandSuggestionPacket>,
    pub last_edit_book: Option<ServerboundEditBookPacket>,
    pub last_interact: Option<ServerboundInteractPacket>,
    pub last_resource_pack_response: Option<ServerboundResourcePackPacket>,
    pub last_container_close: Option<ServerboundContainerClosePacket>,
    pub last_container_button_click: Option<ServerboundContainerButtonClickPacket>,
    pub last_container_click: Option<ServerboundContainerClickPacket>,
    pub last_set_creative_mode_slot: Option<ServerboundSetCreativeModeSlotPacket>,
    pub last_pick_item_from_block: Option<ServerboundPickItemFromBlockPacket>,
    pub last_pick_item_from_entity: Option<ServerboundPickItemFromEntityPacket>,
    pub last_recipe_book_change_settings: Option<ServerboundRecipeBookChangeSettingsPacket>,
    pub last_recipe_book_seen_recipe: Option<ServerboundRecipeBookSeenRecipePacket>,
    pub loaded: bool,
    pub disconnect_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerChunkSender {
    pub(super) pending_chunks: BTreeSet<ChunkPos>,
    pub(super) memory_connection: bool,
    pub(super) desired_chunks_per_tick: f32,
    pub(super) batch_quota: f32,
    pub(super) unacknowledged_batches: i32,
    pub(super) max_unacknowledged_batches: i32,
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

impl CommonPlayerSpawnInfo {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, dimension_type_registry_id(&self.dimension_type)?)?;
        write_identifier(writer, &self.dimension)?;
        write_i64(writer, self.seed)?;
        writer.write_all(&[self.game_mode as u8])?;
        writer.write_all(&[match self.previous_game_mode {
            Some(GameMode::Survival) => 0,
            Some(GameMode::Creative) => 1,
            Some(GameMode::Adventure) => 2,
            Some(GameMode::Spectator) => 3,
            None => 255,
        }])?;
        write_bool(writer, self.is_debug)?;
        write_bool(writer, self.is_flat)?;
        write_optional(
            writer,
            self.last_death_location.as_ref(),
            |writer, (dimension, pos)| {
                write_identifier(writer, dimension)?;
                write_block_position(writer, pos[0], pos[1], pos[2])
            },
        )?;
        write_var_i32(writer, self.portal_cooldown)?;
        write_var_i32(writer, self.sea_level)
    }
}

pub(super) fn dimension_type_registry_id(dimension_type: &Identifier) -> io::Result<i32> {
    match (dimension_type.namespace(), dimension_type.path()) {
        ("minecraft", "overworld") => Ok(0),
        ("minecraft", "overworld_caves") => Ok(1),
        ("minecraft", "the_end") => Ok(2),
        ("minecraft", "the_nether") => Ok(3),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported dimension type {dimension_type} in play packet"),
        )),
    }
}

impl ClientboundLoginPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.player_id)?;
        write_bool(writer, self.hardcore)?;
        write_var_i32(writer, self.levels.len() as i32)?;
        for level in &self.levels {
            write_identifier(writer, level)?;
        }
        write_var_i32(writer, self.max_players)?;
        write_var_i32(writer, self.chunk_radius)?;
        write_var_i32(writer, self.simulation_distance)?;
        write_bool(writer, self.reduced_debug_info)?;
        write_bool(writer, self.show_death_screen)?;
        write_bool(writer, self.do_limited_crafting)?;
        self.spawn_info.write(writer)?;
        write_bool(writer, self.enforces_secure_chat)
    }
}

