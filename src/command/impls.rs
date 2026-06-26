use super::*;

impl PermissionLevel {
    pub fn by_id(id: i32) -> Self {
        match id {
            i32::MIN..=-1 => Self::All,
            0 => Self::All,
            1 => Self::Moderators,
            2 => Self::Gamemasters,
            3 => Self::Admins,
            4..=i32::MAX => Self::Owners,
        }
    }

    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Moderators => "moderators",
            Self::Gamemasters => "gamemasters",
            Self::Admins => "admins",
            Self::Owners => "owners",
        }
    }

    pub fn is_equal_or_higher_than(self, other: Self) -> bool {
        self.id() >= other.id()
    }
}

macro_rules! default_server_command_state {
    () => {
        Self {
            player_idle_timeout_minutes: 0,
            autosave_enabled: true,
            save_all_requests: Vec::new(),
            save_all_should_fail: false,
            side_feedback: Vec::new(),
            published_server: None,
            publish_should_fail: false,
            next_available_publish_port: 25565,
            random_sequences: crate::random_sequences::RandomSequences::default(),
            level_random: None,
            random_seed_defaults: RandomSeedDefaults::default(),
            random_broadcasts: Vec::new(),
            available_data_packs: vec!["vanilla".to_string()],
            selected_data_packs: vec!["vanilla".to_string()],
            disabled_data_packs: Vec::new(),
            feature_data_packs: Vec::new(),
            unavailable_feature_data_packs: Vec::new(),
            created_data_packs: Vec::new(),
            reload_requests: Vec::new(),
            transfer_requests: Vec::new(),
            chase_session: None,
            chase_events: Vec::new(),
            perf_recording: false,
            perf_reports: Vec::new(),
            perf_report_should_fail: false,
            debug_profiler_running: false,
            debug_profiler_results: Vec::new(),
            debug_trace_events: Vec::new(),
            config_players: Vec::new(),
            config_dialog_events: Vec::new(),
            dialog_events: Vec::new(),
            active_effects: Vec::new(),
            mob_spawning_events: Vec::new(),
            debug_path_events: Vec::new(),
            unreachable_debug_paths: Vec::new(),
            incomplete_debug_paths: Vec::new(),
            jfr_recording: false,
            jfr_recordings: Vec::new(),
            next_jfr_recording_path: "debug/vibecraft.jfr".to_string(),
            known_recipes: Vec::new(),
            player_recipes: Vec::new(),
            advancements: Vec::new(),
            player_advancements: Vec::new(),
            advancement_flush_events: Vec::new(),
            entity_attributes: Vec::new(),
            fetched_profiles: Vec::new(),
            avatar_profiles: Vec::new(),
            bossbars: Vec::new(),
            command_time_millis: 0,
            game_time_ticks: 0,
            world_clock_ticks: 0,
            world_clock_paused: false,
            world_clock_rate: 1.0,
            world_preset: "minecraft:normal".to_string(),
            stopwatches: Vec::new(),
            scheduled_functions: Vec::new(),
            available_functions: Vec::new(),
            function_tags: Vec::new(),
            queued_functions: Vec::new(),
            macro_functions: Vec::new(),
            macro_entity_nbt_sources: Vec::new(),
            macro_block_nbt_sources: Vec::new(),
            macro_storage_nbt_sources: Vec::new(),
            function_permission_level: PermissionLevel::Gamemasters,
            command_source_player: None,
            command_source_entity: None,
            command_source_position: Vec3::default(),
            command_source_dimension: "minecraft:overworld".to_string(),
            execute_events: Vec::new(),
            debug_world: false,
            blocks: Vec::new(),
            biomes: Vec::new(),
            clone_events: Vec::new(),
            fill_events: Vec::new(),
            fill_biome_events: Vec::new(),
            forced_chunks: Vec::new(),
            locatable_entries: Vec::new(),
            locate_results: Vec::new(),
            max_block_modifications: 32768,
            online_players: Vec::new(),
            last_list_includes_uuids: false,
            player_inventories: Vec::new(),
            entity_item_slots: Vec::new(),
            block_item_slots: Vec::new(),
            item_modifier_events: Vec::new(),
            item_enchantments: Vec::new(),
            command_loot_tables: Vec::new(),
            entity_loot_tables: Vec::new(),
            loot_events: Vec::new(),
            available_templates: Vec::new(),
            place_events: Vec::new(),
            raids: Vec::new(),
            raid_events: Vec::new(),
            player_game_modes: Vec::new(),
            player_experience: Vec::new(),
            default_game_mode: GameMode::Survival,
            force_game_mode: None,
            difficulty: Difficulty::Easy,
            game_rules: default_game_rules(),
            game_rule_syncs: Vec::new(),
            camera_targets: Vec::new(),
            untrackable_entities: Vec::new(),
            max_players: 20,
            singleplayer_owner: None,
            disconnected_players: Vec::new(),
            online_player_addresses: Vec::new(),
            banned_players: Vec::new(),
            banned_ips: Vec::new(),
            operator_players: Vec::new(),
            killed_entities: Vec::new(),
            ban_ip_feedback_events: Vec::new(),
            teams: Vec::new(),
            player_teams: Vec::new(),
            scoreboard_objectives: Vec::new(),
            scoreboard_scores: Vec::new(),
            scoreboard_display_slots: Vec::new(),
            chat_events: Vec::new(),
            title_events: Vec::new(),
            sound_events: Vec::new(),
            particle_events: Vec::new(),
            warden_spawn_trackers: Vec::new(),
            waypoints: Vec::new(),
            world_border: WorldBorder::default(),
            setblock_events: Vec::new(),
            server_pack_events: Vec::new(),
            summoned_entities: Vec::new(),
            armor_trim_spawns: Vec::new(),
            swing_events: Vec::new(),
            rotation_requests: Vec::new(),
            return_events: Vec::new(),
            ride_events: Vec::new(),
            damage_events: Vec::new(),
            invulnerable_entities: Vec::new(),
            entity_mounts: Vec::new(),
            ride_mount_failures: Vec::new(),
            entity_positions: Vec::new(),
            teleport_side_effects: Vec::new(),
            entity_states: Vec::new(),
            entity_tags: Vec::new(),
            world_spawn: RespawnData::default(),
            player_spawns: Vec::new(),
            weather: WeatherState::default(),
            whitelist_enabled: false,
            whitelisted_players: Vec::new(),
            whitelist_reload_requests: 0,
            kick_unlisted_requests: 0,
            tick_rate: TickRateController::default(),
            tick_feedback_events: Vec::new(),
            average_tick_time_nanos: 50_000_000,
            tick_time_samples_nanos: vec![50_000_000],
            halt_requested: false,
            world_seed: 0,
            version: VersionInfo::CURRENT_26_1_2,
        }
    };
}

impl Default for ServerCommandState {
    fn default() -> Self {
        default_server_command_state!()
    }
}

impl VersionInfo {
    pub const CURRENT_26_1_2: Self = Self {
        id: "26.1.2",
        name: "26.1.2",
        data_version: crate::world_version::CURRENT_DATA_VERSION,
        series: crate::world_version::CURRENT_DATA_SERIES,
        protocol_version: crate::world_version::CURRENT_PROTOCOL_VERSION,
        build_time: crate::world_version::CURRENT_BUILD_TIME,
        resource_pack_version: crate::resources::PackFormat::current_client_resources(),
        data_pack_version: crate::resources::PackFormat::current_server_data(),
        stable: crate::world_version::CURRENT_STABLE,
    };

    pub fn command_lines(&self) -> Vec<String> {
        vec![
            "commands.version.header".to_string(),
            format!("commands.version.id {}", self.id),
            format!("commands.version.name {}", self.name),
            format!("commands.version.data {}", self.data_version),
            format!("commands.version.series {}", self.series),
            format!(
                "commands.version.protocol {} 0x{:x}",
                self.protocol_version, self.protocol_version
            ),
            format!("commands.version.build_time {}", self.build_time),
            format!(
                "commands.version.pack.resource {}",
                self.resource_pack_version
            ),
            format!("commands.version.pack.data {}", self.data_pack_version),
            if self.stable {
                "commands.version.stable.yes".to_string()
            } else {
                "commands.version.stable.no".to_string()
            },
        ]
    }
}

impl LevelBasedPermissionSet {
    pub const ALL: Self = Self::new(PermissionLevel::All);
    pub const MODERATOR: Self = Self::new(PermissionLevel::Moderators);
    pub const GAMEMASTER: Self = Self::new(PermissionLevel::Gamemasters);
    pub const ADMIN: Self = Self::new(PermissionLevel::Admins);
    pub const OWNER: Self = Self::new(PermissionLevel::Owners);

    pub const fn new(level: PermissionLevel) -> Self {
        Self { level }
    }

    pub fn level(self) -> PermissionLevel {
        self.level
    }

    pub fn has_permission(self, permission: Permission) -> bool {
        match permission {
            Permission::CommandLevel(level) => self.level.is_equal_or_higher_than(level),
            Permission::CommandsEntitySelectors => self
                .level
                .is_equal_or_higher_than(PermissionLevel::Gamemasters),
        }
    }

    pub fn can_run(self, command: &str) -> CommandAvailability {
        let command = command
            .trim_start()
            .strip_prefix('/')
            .unwrap_or(command.trim_start())
            .split_whitespace()
            .next()
            .unwrap_or_default();
        let required = command_required_permission(command);
        if self.has_permission(Permission::CommandLevel(required)) {
            CommandAvailability::Available
        } else {
            CommandAvailability::Hidden
        }
    }
}

impl ServerCommandState {
    pub fn whitelist_names(&self) -> Vec<&str> {
        self.whitelisted_players
            .iter()
            .map(|profile| profile.name.as_str())
            .collect()
    }

    pub(super) fn is_whitelisted(&self, profile: &NameAndId) -> bool {
        self.whitelisted_players
            .iter()
            .any(|entry| entry.uuid == profile.uuid)
    }

    pub(super) fn add_whitelisted(&mut self, profile: NameAndId) -> bool {
        if self.is_whitelisted(&profile) {
            false
        } else {
            self.whitelisted_players.push(profile);
            true
        }
    }

    pub(super) fn remove_whitelisted(&mut self, profile: &NameAndId) -> bool {
        let old_len = self.whitelisted_players.len();
        self.whitelisted_players
            .retain(|entry| entry.uuid != profile.uuid);
        self.whitelisted_players.len() != old_len
    }

    pub fn operator_names(&self) -> Vec<&str> {
        self.operator_players
            .iter()
            .map(|profile| profile.name.as_str())
            .collect()
    }

    pub(super) fn is_operator(&self, profile: &NameAndId) -> bool {
        self.operator_players
            .iter()
            .any(|entry| entry.uuid == profile.uuid)
    }

    pub(super) fn add_operator(&mut self, profile: NameAndId) -> bool {
        if self.is_operator(&profile) {
            false
        } else {
            self.operator_players.push(profile);
            true
        }
    }

    pub(super) fn remove_operator(&mut self, profile: &NameAndId) -> bool {
        let old_len = self.operator_players.len();
        self.operator_players
            .retain(|entry| entry.uuid != profile.uuid);
        self.operator_players.len() != old_len
    }

    pub fn banned_player_names(&self) -> Vec<&str> {
        self.banned_players
            .iter()
            .map(|entry| entry.user.name.as_str())
            .collect()
    }

    pub fn banned_ip_names(&self) -> Vec<&str> {
        self.banned_ips
            .iter()
            .map(|entry| entry.user.as_str())
            .collect()
    }

    pub(super) fn is_player_banned(&self, profile: &NameAndId) -> bool {
        self.banned_players
            .iter()
            .any(|entry| entry.user.uuid == profile.uuid)
    }

    pub(super) fn is_ip_banned(&self, ip: &str) -> bool {
        self.banned_ips.iter().any(|entry| entry.user == ip)
    }

    pub(super) fn add_player_ban(&mut self, profile: NameAndId, reason: Option<String>) -> bool {
        if self.is_player_banned(&profile) {
            return false;
        }
        self.banned_players.push(BanEntry {
            user: profile,
            created: "now".to_string(),
            source: self.command_source_name(),
            expires: None,
            reason,
        });
        true
    }

    pub(super) fn add_ip_ban(&mut self, ip: String, reason: Option<String>) -> bool {
        if self.is_ip_banned(&ip) {
            return false;
        }
        self.banned_ips.push(BanEntry {
            user: ip,
            created: "now".to_string(),
            source: self.command_source_name(),
            expires: None,
            reason,
        });
        true
    }

    pub(super) fn remove_player_ban(&mut self, profile: &NameAndId) -> bool {
        let old_len = self.banned_players.len();
        self.banned_players
            .retain(|entry| entry.user.uuid != profile.uuid);
        self.banned_players.len() != old_len
    }

    pub(super) fn remove_ip_ban(&mut self, ip: &str) -> bool {
        let old_len = self.banned_ips.len();
        self.banned_ips.retain(|entry| entry.user != ip);
        self.banned_ips.len() != old_len
    }

    pub(super) fn command_source_name(&self) -> String {
        self.command_source_player
            .as_ref()
            .map(|player| player.name.clone())
            .unwrap_or_else(|| "Server".to_string())
    }

    pub(super) fn online_ip_for_name(&self, name: &str) -> Option<String> {
        self.online_player_addresses
            .iter()
            .find(|entry| entry.player.name.eq_ignore_ascii_case(name))
            .map(|entry| entry.ip.clone())
    }

    pub(super) fn players_with_ip(&self, ip: &str) -> Vec<NameAndId> {
        self.online_player_addresses
            .iter()
            .filter(|entry| entry.ip == ip)
            .map(|entry| entry.player.clone())
            .collect()
    }

    pub(super) fn team_for_player(&self, player: &NameAndId) -> Option<&str> {
        self.player_teams
            .iter()
            .find(|membership| membership.player.uuid == player.uuid)
            .map(|membership| membership.team.as_str())
    }

    pub(super) fn players_on_team(&self, team: &str) -> Vec<NameAndId> {
        self.player_teams
            .iter()
            .filter(|membership| membership.team == team)
            .map(|membership| membership.player.clone())
            .collect()
    }
}
