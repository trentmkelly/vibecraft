use super::*;

pub fn default_recipe_book_settings() -> ClientboundRecipeBookSettingsPacket {
    ClientboundRecipeBookSettingsPacket {
        crafting: RecipeBookTypeSettings::CLOSED_UNFILTERED,
        furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
        blast_furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
        smoker: RecipeBookTypeSettings::CLOSED_UNFILTERED,
    }
}

pub fn load_recipe_book_from_nbt(
    player_compound: &[(String, Tag)],
    recipes: &RecipeMap,
) -> (
    ClientboundRecipeBookSettingsPacket,
    Vec<&'static str>,
    Vec<&'static str>,
) {
    let Some(Tag::Compound(recipe_book)) = compound_tag(player_compound, "recipeBook") else {
        return (default_recipe_book_settings(), Vec::new(), Vec::new());
    };

    let settings = ClientboundRecipeBookSettingsPacket {
        crafting: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isGuiOpen", false),
            filtering: compound_bool_byte(recipe_book, "isFilteringCraftable", false),
        },
        furnace: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isFurnaceGuiOpen", false),
            filtering: compound_bool_byte(recipe_book, "isFurnaceFilteringCraftable", false),
        },
        blast_furnace: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isBlastingFurnaceGuiOpen", false),
            filtering: compound_bool_byte(
                recipe_book,
                "isBlastingFurnaceFilteringCraftable",
                false,
            ),
        },
        smoker: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isSmokerGuiOpen", false),
            filtering: compound_bool_byte(recipe_book, "isSmokerFilteringCraftable", false),
        },
    };

    let known = load_recipe_id_list(recipe_book, "recipes", recipes);
    let highlighted = load_recipe_id_list(recipe_book, "toBeDisplayed", recipes);
    (settings, known, highlighted)
}

pub fn load_recipe_id_list(
    compound: &[(String, Tag)],
    key: &str,
    recipes: &RecipeMap,
) -> Vec<&'static str> {
    match compound_tag(compound, key) {
        Some(Tag::List(values)) => values
            .iter()
            .filter_map(|tag| match tag {
                Tag::String(id) => recipes.by_key(id).map(|holder| holder.id),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub fn apply_recipe_book_settings_packet(
    state: &mut PlaySessionState,
    packet: ServerboundRecipeBookChangeSettingsPacket,
) {
    let settings = RecipeBookTypeSettings {
        open: packet.is_open,
        filtering: packet.is_filtering,
    };
    match packet.book_type {
        RecipeBookType::Crafting => state.recipe_book_settings.crafting = settings,
        RecipeBookType::Furnace => state.recipe_book_settings.furnace = settings,
        RecipeBookType::BlastFurnace => state.recipe_book_settings.blast_furnace = settings,
        RecipeBookType::Smoker => state.recipe_book_settings.smoker = settings,
    }
}

pub fn apply_recipe_book_seen_recipe_packet(
    state: &mut PlaySessionState,
    packet: ServerboundRecipeBookSeenRecipePacket,
    recipes: &RecipeMap,
) {
    if packet.recipe_index < 0 {
        return;
    }
    if let Some(holder) = recipes.values().get(packet.recipe_index as usize) {
        state.inventory_menu.mark_recipe_seen(holder.id);
    }
}

pub fn apply_place_recipe_packet(
    state: &mut PlaySessionState,
    packet: ServerboundPlaceRecipePacket,
    recipes: &RecipeMap,
) -> bool {
    if packet.container_id != 0 {
        return false;
    }
    if packet.recipe_index < 0 {
        return false;
    }
    let Some(holder) = recipes.values().get(packet.recipe_index as usize) else {
        return false;
    };
    if state
        .inventory_menu
        .place_recipe_from_inventory(holder.id, packet.use_max_items)
    {
        state.container_state_id = state.container_state_id.wrapping_add(1);
        true
    } else {
        false
    }
}

pub fn write_inventory_menu_full_sync(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    let slots = state.inventory_menu.all_slots();
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        |payload| {
            payload.write_all(&[0])?;
            write_var_i32(payload, state.container_state_id)?;
            write_var_i32(payload, slots.len() as i32)?;
            for stack in &slots {
                let raw =
                    raw_item_stack_from_item_stack(stack).unwrap_or_else(|_| RawItemStack::empty());
                raw.write_optional_trusted(payload)?;
            }
            let carried = &state.carried_item;
            let raw_carried =
                raw_item_stack_from_item_stack(carried).unwrap_or_else(|_| RawItemStack::empty());
            raw_carried.write_optional_trusted(payload)
        },
    )
}

pub fn compound_tag<'a>(compound: &'a [(String, Tag)], key: &str) -> Option<&'a Tag> {
    compound
        .iter()
        .find_map(|(name, value)| (name == key).then_some(value))
}

pub fn compound_list<'a>(compound: &'a [(String, Tag)], key: &str) -> Option<&'a [Tag]> {
    match compound_tag(compound, key)? {
        Tag::List(values) => Some(values),
        _ => None,
    }
}

pub fn compound_bool_byte(compound: &[(String, Tag)], key: &str, default_value: bool) -> bool {
    match compound_tag(compound, key) {
        Some(Tag::Byte(value)) => *value != 0,
        _ => default_value,
    }
}

pub fn compound_float(compound: &[(String, Tag)], key: &str, default_value: f32) -> f32 {
    match compound_tag(compound, key) {
        Some(Tag::Float(value)) => *value,
        _ => default_value,
    }
}

pub fn login_access_disconnect_reason(
    properties: &ServerProperties,
    player_access: &Arc<Mutex<PlayerAccess>>,
    profile: &NameAndId,
    remote_ip: &str,
    login_host_ip: Option<&str>,
) -> io::Result<Option<&'static str>> {
    let access = player_access
        .lock()
        .map_err(|_| io::Error::other("player access lock poisoned"))?;
    if let Some(login_host_ip) = login_host_ip {
        if access.check_proxy_connection(
            properties.prevent_proxy_connections,
            login_host_ip,
            remote_ip,
        ) == ProxyConnectionDecision::RejectPreventProxyConnections
        {
            return Ok(Some("multiplayer.disconnect.unverified_username"));
        }
    }
    if access.is_player_banned(&profile.uuid) {
        return Ok(Some("multiplayer.disconnect.banned"));
    }
    if properties.enforce_whitelist
        && !access.is_op(&profile.uuid)
        && !access.is_whitelisted(&profile.uuid)
    {
        return Ok(Some("multiplayer.disconnect.not_whitelisted"));
    }
    if access.is_ip_banned(remote_ip) {
        return Ok(Some("multiplayer.disconnect.ip_banned"));
    }
    Ok(None)
}

pub fn login_host_ip(server_address: &str) -> Option<String> {
    let host = server_address
        .strip_prefix('[')
        .and_then(|address| address.split_once(']').map(|(host, _)| host))
        .or_else(|| server_address.split_once(':').map(|(host, _)| host))
        .unwrap_or(server_address);
    host.parse::<IpAddr>()
        .ok()
        .map(|address| address.to_string())
}

pub fn bug_report_server_links_packet(
    properties: &ServerProperties,
) -> Option<ClientboundServerLinksPacket> {
    let link = java_untrusted_http_uri(&properties.bug_report_link)?;
    Some(ClientboundServerLinksPacket {
        links: vec![ServerLinkEntry {
            label: ServerLinkLabel::Known(ServerLinkType::BugReport),
            link: link.to_string(),
        }],
    })
}

fn java_untrusted_http_uri(link: &str) -> Option<&str> {
    let (scheme, _) = link.split_once(':')?;
    match scheme.to_ascii_lowercase().as_str() {
        "http" | "https" => {}
        _ => return None,
    }
    if link
        .chars()
        .any(|ch| ch.is_ascii_control() || ch.is_ascii_whitespace())
    {
        return None;
    }
    Some(link)
}

pub fn load_code_of_conduct_for_language(
    properties: &ServerProperties,
    client_language: &str,
) -> io::Result<Option<String>> {
    if !properties.code_of_conduct {
        return Ok(None);
    }
    let texts = read_code_of_conducts(Path::new("codeofconduct"))?;
    if texts.is_empty() {
        return Ok(None);
    }
    let language = client_language.to_lowercase();
    Ok(texts
        .get(&language)
        .or_else(|| texts.get("en_us"))
        .or_else(|| texts.values().next())
        .cloned())
}

pub fn read_code_of_conducts(dir: &Path) -> io::Result<HashMap<String, String>> {
    let metadata = fs::metadata(dir)?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "codeofconduct is not a directory",
        ));
    }
    let canonical_dir = fs::canonicalize(dir)?;
    let mut texts = HashMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(language) = filename.strip_suffix(".txt") else {
            continue;
        };
        let parent = fs::canonicalize(&path)?
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid codeofconduct path")
            })?;
        if parent != canonical_dir {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "codeofconduct file links outside allowed directory",
            ));
        }
        let text = fs::read_to_string(&path)?;
        let text = strip_minecraft_formatting(&text.lines().collect::<Vec<_>>().join("\n"));
        texts.insert(language.to_lowercase(), text);
    }
    Ok(texts)
}

pub fn strip_minecraft_formatting(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '§' {
            match chars.peek().copied() {
                Some(code)
                    if code.is_ascii_hexdigit()
                        || matches!(code.to_ascii_lowercase(), 'k'..='o' | 'r') =>
                {
                    chars.next();
                    continue;
                }
                _ => {}
            }
        }
        out.push(ch);
    }
    out
}

pub fn read_packet_with_rate_limit<R: Read>(
    reader: &mut R,
    compression: CompressionState,
    rate_limiter: &mut PacketRateLimiter,
) -> io::Result<Vec<u8>> {
    let packet = read_packet_with_compression(reader, compression)?;
    match rate_limiter.record_packet(Instant::now()) {
        PacketRateDecision::Allow => Ok(packet),
        PacketRateDecision::Kick { reason } => Err(rate_limit_disconnect_error(&reason)),
    }
}

pub fn rate_limit_disconnect_error(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason.to_string())
}

#[cfg(test)]
pub fn wait_for_configuration_packet<R: Read>(
    reader: &mut R,
    compression: CompressionState,
    expected_packet_id: i32,
    expected_name: &'static str,
) -> io::Result<()> {
    let mut rate_limiter = PacketRateLimiter::new(0, Instant::now());
    wait_for_configuration_packet_with_rate_limit(
        reader,
        compression,
        expected_packet_id,
        expected_name,
        &mut rate_limiter,
        None,
    )
}

pub fn wait_for_configuration_packet_with_rate_limit<R: Read>(
    reader: &mut R,
    compression: CompressionState,
    expected_packet_id: i32,
    expected_name: &'static str,
    rate_limiter: &mut PacketRateLimiter,
    active_login: Option<&ActiveLoginGuard>,
) -> io::Result<()> {
    for _ in 0..32 {
        let packet = read_packet_with_rate_limit(reader, compression, rate_limiter)?;
        let mut input = Cursor::new(packet);
        let packet_id = read_var_i32(&mut input)?;
        if packet_id == expected_packet_id {
            validate_expected_configuration_packet(&mut input, expected_packet_id)?;
            return Ok(());
        }
        if packet_id == SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID {
            // The vanilla client interleaves its ClientInformation during
            // configuration; capture the listing preference so the status player
            // sample is correct when this player reaches the PLAY state (Java
            // `ServerConfigurationPacketListenerImpl.handleClientInformation`).
            if let Some(active_login) = active_login {
                let packet = ServerboundClientInformationPacket::read(&mut input)?;
                active_login.set_allows_listing(packet.information.allows_listing);
            }
            continue;
        }
        if is_tolerated_serverbound_configuration_packet(packet_id) {
            continue;
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected {expected_name}, got configuration packet {packet_id}"),
        ));
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("timed out waiting for {expected_name}"),
    ))
}

pub fn validate_expected_configuration_packet(
    input: &mut Cursor<Vec<u8>>,
    packet_id: i32,
) -> io::Result<()> {
    match packet_id {
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID => {
            let pack_count = read_var_i32(input)?;
            if !(0..=64).contains(&pack_count) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid selected known pack count",
                ));
            }
            for _ in 0..pack_count {
                let _namespace = read_string(input, 64)?;
                let _id = read_string(input, 128)?;
                let _version = read_string(input, 64)?;
            }
        }
        SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID => {}
        _ => {}
    }

    if input.position() != input.get_ref().len() as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("trailing bytes in configuration packet {packet_id}"),
        ));
    }

    Ok(())
}

pub fn is_tolerated_serverbound_configuration_packet(packet_id: i32) -> bool {
    matches!(
        packet_id,
        SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID
            | SERVERBOUND_CONFIGURATION_COOKIE_RESPONSE_PACKET_ID
            | SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID
            | SERVERBOUND_CONFIGURATION_KEEP_ALIVE_PACKET_ID
            | SERVERBOUND_CONFIGURATION_PONG_PACKET_ID
            | SERVERBOUND_CONFIGURATION_RESOURCE_PACK_PACKET_ID
            | SERVERBOUND_CONFIGURATION_CUSTOM_CLICK_ACTION_PACKET_ID
            | SERVERBOUND_CONFIGURATION_ACCEPT_CODE_OF_CONDUCT_PACKET_ID
    )
}

pub struct MinimalPlayJoinContext<'a> {
    pub properties: &'a ServerProperties,
    pub world_seed: i64,
    pub profile: &'a NameAndId,
    pub play_state: &'a PlaySessionState,
    pub recipe_manager: &'a RecipeManagerModel,
    pub world_root: &'a Path,
    pub clock_game_time: i64,
    pub clock_data: &'a [(i32, ClockNetworkState)],
    pub rain_level: f32,
    pub thunder_level: f32,
}

pub fn write_minimal_play_join(
    stream: &mut TcpStream,
    compression: CompressionState,
    context: MinimalPlayJoinContext<'_>,
) -> io::Result<()> {
    let center = ChunkPos {
        x: chunk_coordinate(context.play_state.x),
        z: chunk_coordinate(context.play_state.z),
    };
    write_join_login_and_profile_packets(stream, compression, &context)?;
    write_join_player_state_packets(stream, compression, &context)?;
    write_join_inventory_packets(stream, compression, context.play_state)?;
    write_join_world_state_packets(stream, compression, &context, center)?;
    // Note: the `/biome` debug command tree is intentionally NOT sent here. It is
    // emitted once by the play loop right after the first chunk batch (see
    // `tick_player_and_chunk_sender` / `write_join_commands_packet`), so it lands
    // after the join-ready prefix (login..chunk_batch_start) and does not disturb
    // the vanilla join capture in `harness/mineflayer/raw_26_1_2_join_probe.mjs`.
    // Optional disconnect-probe delay before chunks start flowing — kept
    // for parity with the legacy blocking-batch path. With the async
    // pipeline, the actual chunk batches start arriving from the play
    // loop's per-tick drain (see `drain_chunk_sender`) instead of being
    // synchronously generated here.
    delay_initial_chunk_batch_for_probe(stream, compression)?;
    Ok(())
}

/// Emits the `/biome` debug command tree. Sent late in the join sequence (after
/// the join-ready state/level prefix) so the live wire order matches the vanilla
/// join capture, where the command tree arrives after the inventory and
/// level-info packets. RustCraft exposes `/biome` as an in-game debugging helper
/// even though vanilla 26.1.2 has no root `/biome` command — an intentional,
/// documented Java-parity divergence in the packet *contents* (not its position).
pub(super) fn write_join_commands_packet(
    stream: &mut TcpStream,
    compression: CompressionState,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_COMMANDS_PACKET_ID,
        |payload| rustcraft_debug_commands_packet().write(payload),
    )
}

fn write_join_login_and_profile_packets(
    stream: &mut TcpStream,
    compression: CompressionState,
    context: &MinimalPlayJoinContext<'_>,
) -> io::Result<()> {
    let login = ClientboundLoginPacket {
        player_id: 1,
        hardcore: context.properties.hardcore,
        levels: vec![overworld_identifier()?],
        max_players: context.properties.max_players as i32,
        chunk_radius: context.properties.view_distance as i32,
        simulation_distance: context.properties.simulation_distance as i32,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo {
            seed: context.world_seed,
            game_mode: context.play_state.game_mode,
            previous_game_mode: context.play_state.previous_game_mode,
            is_flat: false,
            ..CommonPlayerSpawnInfo::default()
        },
        // Java mirror: ClientboundLoginPacket carries `server.enforceSecureProfile()`
        // (PlayerList.java:179) = enforce-secure-profile && online-mode &&
        // canValidateProfileKeys(). RustCraft has no loaded profile-key validation,
        // so this is false (matching vanilla offline). See the matching
        // TODO(secure-profile-enforcement) in `status_json` (chunk_e_2.rs).
        enforces_secure_chat: false,
    };
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_LOGIN_PACKET_ID,
        |payload| write_clientbound_login_packet(payload, &login),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID,
        |payload| {
            write_player_info_initializing_packet(
                payload,
                context.profile,
                context.play_state.game_mode,
            )
        },
    )?;
    Ok(())
}

fn write_join_player_state_packets(
    stream: &mut TcpStream,
    compression: CompressionState,
    context: &MinimalPlayJoinContext<'_>,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
        |payload| {
            payload.write_all(&[1])?;
            write_bool(payload, false)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
        |payload| write_player_abilities_packet(payload, context.play_state.game_mode),
    )?;
    // Note: the `/biome` debug commands packet (an intentional Java-parity
    // divergence — vanilla 26.1.2 has no root `/biome` command) is NOT sent here.
    // Vanilla `PlayerList.placeNewPlayer` sends `ClientboundSetHeldSlotPacket`
    // (105) before the command tree, and the empirical vanilla join capture in
    // `harness/mineflayer/raw_26_1_2_join_probe.mjs` shows the commands packet
    // arrives after the join-ready prefix, so it is emitted last in
    // `write_minimal_play_join` (see `write_join_commands_packet`). Emitting it
    // here previously produced the wire order `...,64,16,105,...` and failed the
    // raw join-parity probe at index 4 ("expected 105, got 16").
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
        |payload| write_var_i32(payload, context.play_state.selected_slot),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        |payload| {
            payload.write_all(&context.play_state.xp_progress.to_be_bytes())?;
            write_var_i32(payload, context.play_state.xp_level)?;
            write_var_i32(payload, context.play_state.xp_total)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_HEALTH_PACKET_ID,
        |payload| {
            payload.write_all(&context.play_state.health.to_be_bytes())?;
            write_var_i32(payload, context.play_state.food_level)?;
            payload.write_all(&context.play_state.food_saturation.to_be_bytes())
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID,
        |payload| context.play_state.recipe_book_settings.write(payload),
    )?;
    let known_recipes = context
        .play_state
        .inventory_menu
        .recipe_book_known_recipes();
    let highlighted_recipes = context
        .play_state
        .inventory_menu
        .recipe_book_highlighted_recipes();
    if let Some(packet) = build_recipe_book_add_with_flags(
        &known_recipes,
        context.recipe_manager.recipe_map(),
        false,
        false,
        true,
        Some(&highlighted_recipes),
    ) {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
            |payload| packet.write(payload),
        )?;
    }
    Ok(())
}

fn write_join_inventory_packets(
    stream: &mut TcpStream,
    compression: CompressionState,
    play_state: &PlaySessionState,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        |payload| {
            payload.write_all(&[0])?; // container ID = player inventory (InventoryMenu.CONTAINER_ID)
            write_var_i32(payload, play_state.container_state_id)?;
            // Emit all 46 InventoryMenu slots (result + crafting grid + armour + storage + hotbar + offhand).
            // Java: AbstractContainerMenu.sendAllDataToRemote() iterates containerSlots[0..size].
            let slots = play_state.inventory_menu.all_slots();
            write_var_i32(payload, slots.len() as i32)?;
            for stack in &slots {
                raw_item_stack_for_join_sync(stack).write_optional_trusted(payload)?;
            }
            // Carried (cursor) item.
            // Java: ServerPlayer.containerMenu.setRemoteCarried(carried)
            raw_item_stack_for_join_sync(&play_state.carried_item).write_optional_trusted(payload)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
        |payload| write_var_i32(payload, 0),
    )?;
    Ok(())
}

fn write_join_world_state_packets(
    stream: &mut TcpStream,
    compression: CompressionState,
    context: &MinimalPlayJoinContext<'_>,
    center: ChunkPos,
) -> io::Result<()> {
    // Full clock sync so the client's Timeline system can start rendering the sky.
    // Java: ServerClockManager.createFullSyncPacket() — sent during ServerLevel.sendLevelInfo()
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_TIME_PACKET_ID,
        |payload| {
            ClientboundSetTimePacket {
                game_time: context.clock_game_time,
                clock_updates: context.clock_data.iter().cloned().collect(),
            }
            .write(payload)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        |payload| {
            write_var_i32(payload, center.x)?;
            write_var_i32(payload, center.z)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
        |payload| write_var_i32(payload, context.properties.view_distance as i32),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAYER_POSITION_PACKET_ID,
        |payload| {
            write_var_i32(payload, 0)?;
            write_vec3(
                payload,
                context.play_state.x,
                context.play_state.y,
                context.play_state.z,
            )?;
            write_vec3(payload, 0.0, 0.0, 0.0)?;
            payload.write_all(&context.play_state.yaw.to_be_bytes())?;
            payload.write_all(&context.play_state.pitch.to_be_bytes())?;
            payload.write_all(&0_i32.to_be_bytes())
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID,
        write_initialize_world_border_packet,
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID,
        |payload| {
            let default_spawn = world_spawn_suggestion(context.world_root, context.world_seed);
            write_default_spawn_position_packet(
                payload,
                default_spawn.0,
                default_spawn.1,
                default_spawn.2,
            )
        },
    )?;
    // Type 2 = StopRaining (used to initialise client weather state even when not raining).
    // Java: ServerLevel.sendLevelInfo() sends BeginRaining/StopRaining on join.
    write_game_event(stream, compression, 2, 0.0)?;
    // Types 7 and 8: current rain/thunder levels.
    // Java: ServerLevel.advanceWeatherCycle() — RainLevelChange/ThunderLevelChange
    write_game_event(stream, compression, 7, context.rain_level)?;
    write_game_event(stream, compression, 8, context.thunder_level)?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[LEVEL_CHUNKS_LOAD_START_GAME_EVENT_ID])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )?;
    Ok(())
}

fn overworld_identifier() -> io::Result<Identifier> {
    Identifier::parse("minecraft:overworld").map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("built-in overworld identifier failed to parse: {err}"),
        )
    })
}

fn raw_item_stack_for_join_sync(stack: &ItemStack) -> RawItemStack {
    if stack.is_empty() {
        return RawItemStack::empty();
    }
    let Some(pid) = item_protocol_id(stack.item_id()) else {
        return RawItemStack::empty();
    };
    RawItemStack {
        count: stack.count(),
        item_id: Some(pid),
        components: RawDataComponentPatch::empty(),
    }
}

/// Builds the RustCraft-only command tree additions required by the vanilla client.
///
/// Intentional Java parity divergence: vanilla 26.1.2 has no root `/biome`
/// command, but exposing it in the client dispatcher makes the server-side
/// debug command usable from the normal slash-command UI.
pub fn rustcraft_debug_commands_packet() -> ClientboundCommandsPacket {
    ClientboundCommandsPacket {
        root_index: 0,
        entries: vec![
            CommandNodeEntryData {
                stub: CommandNodeStubData::Root,
                executable: false,
                restricted: false,
                redirect: None,
                children: vec![1],
            },
            CommandNodeEntryData {
                stub: CommandNodeStubData::Literal {
                    name: "biome".to_string(),
                },
                executable: true,
                restricted: false,
                redirect: None,
                children: Vec::new(),
            },
        ],
    }
}

/// Per-session diagnostic counters for the new async chunk pipeline.
///
/// Reported periodically by the play loop (`maybe_log_chunk_pipeline_stats`)
/// — see CHECKLIST_CHUNKING_CHANGES.md "Baseline and diagnostics".
#[derive(Debug, Default)]
pub struct ChunkPipelineSessionStats {
    /// Chunks the play loop has flushed to the client since startup.
    pub sent_total: u64,
    /// Tick at which we last logged (0 = never).
    pub last_log_tick: u64,
}

pub const CHUNK_PIPELINE_LOG_INTERVAL_TICKS: u64 = 40; // ~2 s at 20 TPS

/// Seed a per-session sender + pipeline with every chunk in the player's
/// initial view-distance window centred at (center_x, center_z).
///
/// O(view_distance^2) but each call is constant-work per chunk: marking
/// pending and scheduling generation never touches disk or worldgen
/// directly, so the play loop is ready to tick immediately after this
/// returns. Mirrors Java `ChunkMap.applyChunkTrackingView` for the join
/// flow.
pub fn seed_chunk_window(
    chunk_sender: &mut PlayerChunkSender,
    chunk_pipeline: &ChunkPipeline,
    center_x: i32,
    center_z: i32,
    radius: i32,
) {
    for z in (center_z - radius)..=(center_z + radius) {
        for x in (center_x - radius)..=(center_x + radius) {
            let pos = ChunkPos { x, z };
            chunk_sender.mark_chunk_pending_to_send(pos);
            chunk_pipeline.request_chunk(pos);
        }
    }
}

/// Per-tick chunk send drain. Mirrors Java
/// `MinecraftServer.tickChildren()` → `chunkSender.sendNextChunks(player)`
/// (one chunk batch per tick max, paced by the client's
/// `desiredChunksPerTick` feedback).
///
/// Returns the count of chunks actually flushed this tick.
pub fn drain_chunk_sender(
    stream: &mut TcpStream,
    compression: CompressionState,
    chunk_sender: &mut PlayerChunkSender,
    chunk_pipeline: &ChunkPipeline,
    player_chunk_pos: ChunkPos,
    mut live_fluid_unpack: Option<(&mut LiveFluidTicks, i64)>,
) -> io::Result<usize> {
    let Some(batch) =
        chunk_sender.send_next_chunks(player_chunk_pos, |pos| chunk_pipeline.try_get_ready(pos))
    else {
        return Ok(0);
    };
    write_chunk_batch_to_stream(stream, compression, &batch, live_fluid_unpack.as_mut())?;
    Ok(batch.chunks.len())
}

/// Flush a [`ReadyChunkBatch`] to the wire with the Java-mandated framing
/// (`ChunkBatchStart` → N × `LevelChunkWithLight` → `ChunkBatchFinished`).
pub fn write_chunk_batch_to_stream(
    stream: &mut TcpStream,
    compression: CompressionState,
    batch: &ReadyChunkBatch,
    mut live_fluid_unpack: Option<&mut (&mut LiveFluidTicks, i64)>,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
        |_| Ok(()),
    )?;

    for (_, chunk) in &batch.chunks {
        // Java mirror: ChunkAccess.unpackTicks(currentTick) +
        // LevelChunk.registerTickContainerInLevel — runs once per chunk
        // as it transitions to the loaded/sent state. Restores the
        // chunk's saved fluid_ticks (zero for freshly generated chunks).
        // No block scan and no neighbour reads — that's the Java
        // invariant, and it's what unblocked the play loop here.
        if let Some((ticks, game_time)) = live_fluid_unpack.as_deref_mut() {
            unpack_chunk_fluid_ticks(ticks, *game_time, chunk);
        }
        write_generated_spawn_chunk_packets_from_chunk(stream, compression, chunk)?;
    }

    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID,
        |payload| write_var_i32(payload, batch.chunks.len() as i32),
    )?;
    Ok(())
}

/// Apply a `ServerboundChunkBatchReceivedPacket` to the per-session sender.
/// Java mirror: `ServerGamePacketListenerImpl.handleChunkBatchReceived` →
/// `PlayerChunkSender.onChunkBatchReceivedByClient`.
pub fn handle_chunk_batch_received_packet<R: Read>(
    reader: &mut R,
    chunk_sender: &mut PlayerChunkSender,
) -> io::Result<()> {
    let packet = ServerboundChunkBatchReceivedPacket::read(reader)?;
    chunk_sender.on_chunk_batch_received_by_client(packet.desired_chunks_per_tick);
    Ok(())
}

pub struct ChunkMovementContext<'a> {
    pub chunk_sender: &'a mut PlayerChunkSender,
    pub chunk_pipeline: &'a ChunkPipeline,
    pub loaded_chunks: &'a mut BTreeSet<(i32, i32)>,
    pub new_center: ChunkPos,
    pub radius: i32,
    pub world_root: &'a Path,
    pub world_seed: i64,
}

/// Apply a player chunk movement to the per-session sender and pipeline.
///
/// `loaded_chunks` is the *old* tracked window; on return it is replaced
/// with `next_loaded_chunks`. Chunks leaving the window are forgotten
/// (or just unscheduled if they hadn't been sent yet); chunks entering
/// the window are scheduled for generation and queued for the next paced
/// batch.
pub fn apply_chunk_movement(
    stream: &mut TcpStream,
    compression: CompressionState,
    context: ChunkMovementContext<'_>,
) -> io::Result<()> {
    // Java: ChunkMap.updatePlayerStatus sends ClientboundSetChunkCacheCenter
    // before delta-loading the new visible window.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        |payload| {
            write_var_i32(payload, context.new_center.x)?;
            write_var_i32(payload, context.new_center.z)
        },
    )?;

    let next = chunk_window(context.new_center.x, context.new_center.z, context.radius);
    for stale in context
        .loaded_chunks
        .difference(&next)
        .copied()
        .collect::<Vec<_>>()
    {
        let pos = ChunkPos {
            x: stale.0,
            z: stale.1,
        };
        // If the chunk was already sent to the client, emit Forget +
        // entity-remove packets exactly like the legacy path; otherwise
        // just drop it from the pending queue (Java: dropChunk early-outs
        // when removeOk and player is alive).
        match context.chunk_sender.drop_chunk(pos, true) {
            Some(PlayInstruction::ForgetLevelChunk { pos: forget_pos }) => {
                debug_assert_eq!(forget_pos, pos);
                write_forget_generated_spawn_chunk_packets(
                    stream,
                    compression,
                    pos.x,
                    pos.z,
                    context.world_root,
                    context.world_seed,
                    context.chunk_pipeline.cache(),
                )?;
            }
            Some(_) | None => {
                // Pending-only: also tell the pipeline to drop it from the
                // queue so we don't waste a worker on chunks that are no
                // longer visible.
                context.chunk_pipeline.cancel_request(pos);
            }
        }
    }
    for fresh in next.difference(context.loaded_chunks).copied() {
        let pos = ChunkPos {
            x: fresh.0,
            z: fresh.1,
        };
        context.chunk_sender.mark_chunk_pending_to_send(pos);
        context.chunk_pipeline.request_chunk(pos);
    }
    *context.loaded_chunks = next;
    Ok(())
}

/// Periodic per-session pipeline diagnostic, gated on the
/// `RUSTCRAFT_LOG_CHUNK_PIPELINE` env var (default: on whenever there are
/// pending chunks, to make a stuck pipeline easy to spot).
pub fn maybe_log_chunk_pipeline_stats(
    stats: &mut ChunkPipelineSessionStats,
    chunk_sender: &PlayerChunkSender,
    chunk_pipeline: &ChunkPipeline,
    play_tick_count: u64,
) {
    if play_tick_count < stats.last_log_tick + CHUNK_PIPELINE_LOG_INTERVAL_TICKS {
        return;
    }
    let pending = chunk_sender.pending_count();
    let diag = chunk_pipeline.diagnostics();
    if pending == 0 && diag.queue_depth == 0 && diag.in_flight == 0 {
        // Nothing interesting to report; skip log spam.
        return;
    }
    stats.last_log_tick = play_tick_count;
    eprintln!(
        "[chunk-pipeline] tick={} pending_to_send={} queue={} in_flight={} oldest_age_ms={} generated_total={} sent_total={} desired_cpt={:.2} unacked_batches={}/{}",
        play_tick_count,
        pending,
        diag.queue_depth,
        diag.in_flight,
        diag.oldest_request_age_ms,
        diag.generated_total,
        stats.sent_total,
        chunk_sender.desired_chunks_per_tick(),
        chunk_sender.unacknowledged_batches(),
        chunk_sender.max_unacknowledged_batches(),
    );
}

/// Legacy blocking view-distance batch (deprecated by the async pipeline).
///
/// Kept temporarily as dead code so it stays available for diagnostics and
/// for the human playtester to fall back to via env var or direct call if a
/// regression turns up. Will be deleted once the new pipeline is confirmed
/// in-game — see CHECKLIST_CHUNKING_CHANGES.md "Migration steps".
#[allow(dead_code)]
pub struct PlayChunkBatchRequest<'a> {
    pub center: ChunkPos,
    pub radius: i32,
    pub update_cache_center: bool,
    pub world_root: &'a Path,
    pub world_seed: i64,
    pub chunk_cache: &'a GeneratedChunkCache,
}

#[allow(dead_code)]
pub fn write_play_chunk_batch(
    stream: &mut TcpStream,
    compression: CompressionState,
    request: PlayChunkBatchRequest<'_>,
) -> io::Result<()> {
    if request.update_cache_center {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
            |payload| {
                write_var_i32(payload, request.center.x)?;
                write_var_i32(payload, request.center.z)
            },
        )?;
    }
    let chunks: Vec<_> = ((request.center.z - request.radius)
        ..=(request.center.z + request.radius))
        .flat_map(|z| {
            ((request.center.x - request.radius)..=(request.center.x + request.radius))
                .map(move |x| (x, z))
        })
        .filter(|&(x, z)| x != request.center.x || z != request.center.z)
        .collect();
    write_play_chunk_delta(
        stream,
        compression,
        PlayChunkDeltaRequest {
            center: request.center,
            chunks: &chunks,
            update_cache_center: false,
            world_root: request.world_root,
            world_seed: request.world_seed,
            chunk_cache: request.chunk_cache,
            live_fluid_ticks: None,
        },
    )
}
