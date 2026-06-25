use super::super::*;
#[test]
fn vanilla_join_sequence_matches_player_list_packet_and_side_effect_order() {
    let mut session = PlaySession::new(42, 3);
    session.container_state_id = 42;
    let login = ClientboundLoginPacket {
        player_id: 42,
        hardcore: true,
        levels: BTreeSet::from([
            Identifier::parse("minecraft:overworld").unwrap(),
            Identifier::parse("minecraft:the_nether").unwrap(),
            Identifier::parse("minecraft:the_end").unwrap(),
        ]),
        max_players: 20,
        chunk_radius: 10,
        simulation_distance: 10,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo::default(),
        enforces_secure_chat: true,
    };
    let abilities = PlayerAbilities {
        invulnerable: false,
        flying: false,
        may_fly: false,
        instabuild: false,
        flying_speed: 0.05,
        walking_speed: 0.1,
    };

    let instructions = session.vanilla_join_sequence(JoinGameSettings {
        login: login.clone(),
        difficulty: GameDifficulty::Hard,
        difficulty_locked: true,
        abilities,
        permission_level: 2,
        initial_recipes: true,
        initial_recipe_book: true,
        scoreboard: true,
        server_status: true,
        player_info_existing_count: 2,
        active_effect_count: 1,
    });

    assert_eq!(session.state, PlayState::WaitingForPlayerLoaded);
    assert_eq!(session.container_state_id, 0);
    assert_eq!(
        instructions,
        vec![
            PlayInstruction::Login(login),
            PlayInstruction::ChangeDifficulty {
                difficulty: GameDifficulty::Hard,
                locked: true,
            },
            PlayInstruction::PlayerAbilities(abilities),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket { slot: 3 }),
            PlayInstruction::UpdateRecipes,
            PlayInstruction::UpdatePermissionLevel(2),
            PlayInstruction::SendInitialRecipeBook,
            PlayInstruction::UpdateScoreboard,
            PlayInstruction::TeleportToSpawn { teleport_id: 0 },
            PlayInstruction::ServerStatus,
            PlayInstruction::PlayerInfoUpdate {
                existing_players: 2,
            },
            PlayInstruction::BroadcastSelfPlayerInfo,
            PlayInstruction::SendLevelInfo,
            PlayInstruction::AddPlayerToLevel,
            PlayInstruction::BossEventsOnConnect,
            PlayInstruction::ActiveEffects { count: 1 },
            PlayInstruction::InitInventoryMenu,
        ]
    );
}
