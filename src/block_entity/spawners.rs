use super::*;

impl Default for SpawnerCustomSpawnRules {
    fn default() -> Self {
        Self {
            block_light_limit: (0, 15),
            sky_light_limit: (0, 15),
            requires_no_sky_access: false,
        }
    }
}

impl SpawnerCustomSpawnRules {
    pub fn is_valid_position(&self, block_light: i32, sky_light: i32, no_sky_access: bool) -> bool {
        (self.block_light_limit.0..=self.block_light_limit.1).contains(&block_light)
            && (self.sky_light_limit.0..=self.sky_light_limit.1).contains(&sky_light)
            && (!self.requires_no_sky_access || no_sky_access)
    }

    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "block_light_limit".to_string(),
                Tag::IntArray(vec![self.block_light_limit.0, self.block_light_limit.1]),
            ),
            (
                "sky_light_limit".to_string(),
                Tag::IntArray(vec![self.sky_light_limit.0, self.sky_light_limit.1]),
            ),
            (
                "requires_no_sky_access".to_string(),
                Tag::Byte(i8::from(self.requires_no_sky_access)),
            ),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            block_light_limit: int_range_field(entries, "block_light_limit").unwrap_or((0, 15)),
            sky_light_limit: int_range_field(entries, "sky_light_limit").unwrap_or((0, 15)),
            requires_no_sky_access: get_byte(entries, "requires_no_sky_access").unwrap_or(0) != 0,
        })
    }
}

impl Default for SpawnDataModel {
    fn default() -> Self {
        Self::new("minecraft:pig")
    }
}

impl SpawnDataModel {
    pub fn new(entity_id: impl Into<String>) -> Self {
        Self {
            entity: Tag::Compound(vec![("id".to_string(), Tag::String(entity_id.into()))]),
            custom_spawn_rules: None,
            equipment: None,
            weight: 1,
        }
    }

    pub fn entity_id(&self) -> Option<&str> {
        compound_entries(&self.entity).and_then(|entries| get_string(entries, "id"))
    }

    pub(super) fn to_tag(&self) -> Tag {
        let mut entries = vec![("entity".to_string(), self.entity.clone())];
        if let Some(rules) = &self.custom_spawn_rules {
            entries.push(("custom_spawn_rules".to_string(), rules.to_tag()));
        }
        if let Some(equipment) = &self.equipment {
            entries.push(("equipment".to_string(), equipment.clone()));
        }
        if self.weight != 1 {
            entries.push(("weight".to_string(), Tag::Int(self.weight)));
        }
        Tag::Compound(entries)
    }

    pub(super) fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let entity = entries
            .iter()
            .find(|(name, _)| name == "entity")
            .map(|(_, tag)| tag.clone())
            .unwrap_or_else(|| {
                Tag::Compound(vec![(
                    "id".to_string(),
                    Tag::String("minecraft:pig".to_string()),
                )])
            });
        let custom_spawn_rules = entries
            .iter()
            .find(|(name, _)| name == "custom_spawn_rules")
            .and_then(|(_, tag)| SpawnerCustomSpawnRules::from_tag(tag));
        let equipment = entries
            .iter()
            .find(|(name, _)| name == "equipment")
            .map(|(_, tag)| tag.clone());
        let weight = get_int(entries, "weight").unwrap_or(1).max(1);
        Some(Self {
            entity,
            custom_spawn_rules,
            equipment,
            weight,
        })
    }
}

impl Default for SpawnerBlockEntity {
    fn default() -> Self {
        let config = SpawnerConfig::default();
        Self {
            spawn_delay: config.spawn_delay,
            min_spawn_delay: config.min_spawn_delay,
            max_spawn_delay: config.max_spawn_delay,
            spawn_count: config.spawn_count,
            max_nearby_entities: config.max_nearby_entities,
            required_player_range: config.required_player_range,
            spawn_range: config.spawn_range,
            spawn_potentials: vec![SpawnDataModel::default()],
            next_spawn_data: None,
            spin: 0.0,
            old_spin: 0.0,
        }
    }
}

impl SpawnerBlockEntity {
    pub const EVENT_SPAWN: i32 = 1;
    pub const NEXT_SPAWN_DATA_UPDATE_FLAGS: i32 = 260;

    pub fn config(&self) -> SpawnerConfig {
        SpawnerConfig {
            spawn_delay: self.spawn_delay,
            min_spawn_delay: self.min_spawn_delay,
            max_spawn_delay: self.max_spawn_delay,
            spawn_count: self.spawn_count,
            max_nearby_entities: self.max_nearby_entities,
            required_player_range: self.required_player_range,
            spawn_range: self.spawn_range,
        }
    }

    pub fn get_or_create_next_spawn_data(&mut self, random_roll: usize) -> &SpawnDataModel {
        self.next_spawn_data.get_or_insert_with(|| {
            let selected = weighted_spawn_data(&self.spawn_potentials, random_roll)
                .cloned()
                .unwrap_or_default();
            selected
        })
    }

    pub fn set_entity_id(&mut self, entity_id: impl Into<String>) {
        let entity_id = entity_id.into();
        let data = self
            .next_spawn_data
            .get_or_insert_with(SpawnDataModel::default);
        data.entity = Tag::Compound(vec![("id".to_string(), Tag::String(entity_id))]);
    }

    pub fn set_next_spawn_data(&mut self, data: SpawnDataModel, has_level: bool) -> Option<i32> {
        self.next_spawn_data = Some(data);
        has_level.then_some(Self::NEXT_SPAWN_DATA_UPDATE_FLAGS)
    }

    pub fn delay(&mut self, random_roll: i32) {
        self.spawn_delay = if self.max_spawn_delay <= self.min_spawn_delay {
            self.min_spawn_delay
        } else {
            self.min_spawn_delay
                + random_roll.rem_euclid(self.max_spawn_delay - self.min_spawn_delay)
        };
    }

    pub fn server_tick(
        &mut self,
        player_in_range: bool,
        spawner_blocks_work: bool,
        nearby_entities: i32,
        random_roll: i32,
    ) -> SpawnerTickResult {
        match spawner_tick_plan(
            self.config(),
            player_in_range,
            spawner_blocks_work,
            nearby_entities,
        ) {
            SpawnerTickPlan::Idle => SpawnerTickResult::Idle,
            SpawnerTickPlan::CountDown { next_delay } => {
                self.spawn_delay = next_delay;
                SpawnerTickResult::CountDown
            }
            SpawnerTickPlan::Delay { .. } => {
                self.delay(random_roll);
                SpawnerTickResult::Delay
            }
            SpawnerTickPlan::TrySpawn { attempts } => {
                let entity_id = self
                    .get_or_create_next_spawn_data(random_roll as usize)
                    .entity_id()
                    .unwrap_or("minecraft:pig")
                    .to_string();
                SpawnerTickResult::TrySpawn {
                    entity_id,
                    attempts,
                }
            }
        }
    }

    pub fn server_tick_with_context(&mut self, context: SpawnerSpawnContext) -> SpawnerTickResult {
        match self.server_tick(
            context.player_in_range,
            context.spawner_blocks_work,
            context.nearby_entities,
            context.delay_roll,
        ) {
            SpawnerTickResult::TrySpawn {
                entity_id,
                attempts,
            } => {
                if context.nearby_entities >= self.max_nearby_entities {
                    self.finish_spawn_cycle(context.delay_roll, context.potential_roll);
                    return SpawnerTickResult::MobCapReached;
                }
                let rules = self
                    .next_spawn_data
                    .as_ref()
                    .and_then(|data| data.custom_spawn_rules.as_ref());
                let custom_rules_ok = rules
                    .map(|rules| {
                        rules.is_valid_position(
                            context.block_light,
                            context.sky_light,
                            context.no_sky_access,
                        )
                    })
                    .unwrap_or(true);
                if !custom_rules_ok
                    || !context.collision_free
                    || !context.spawn_rules_ok
                    || !context.obstruction_free
                {
                    return SpawnerTickResult::SpawnRulesFailed;
                }
                self.finish_spawn_cycle(context.delay_roll, context.potential_roll);
                SpawnerTickResult::Spawned {
                    entity_id,
                    count: attempts,
                }
            }
            other => other,
        }
    }

    pub fn finish_spawn_cycle(&mut self, random_roll: i32, potential_roll: usize) {
        self.delay(random_roll);
        if let Some(data) = weighted_spawn_data(&self.spawn_potentials, potential_roll).cloned() {
            self.next_spawn_data = Some(data);
        }
    }

    pub fn client_tick(&mut self, player_in_range: bool) {
        self.client_tick_with_display(player_in_range, true);
    }

    pub fn client_tick_with_display(&mut self, player_in_range: bool, has_display_entity: bool) {
        if !player_in_range {
            self.old_spin = self.spin;
            return;
        }
        if !has_display_entity {
            return;
        }
        if self.spawn_delay > 0 {
            self.spawn_delay -= 1;
        }
        self.old_spin = self.spin;
        self.spin = (self.spin + 1000.0 / (self.spawn_delay as f64 + 200.0)) % 360.0;
    }

    pub fn on_event_triggered(&mut self, client_side: bool, event_id: i32) -> bool {
        if event_id != Self::EVENT_SPAWN {
            return false;
        }
        if client_side {
            self.spawn_delay = self.min_spawn_delay;
        }
        true
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![
            ("Delay".to_string(), Tag::Short(self.spawn_delay as i16)),
            (
                "MinSpawnDelay".to_string(),
                Tag::Short(self.min_spawn_delay as i16),
            ),
            (
                "MaxSpawnDelay".to_string(),
                Tag::Short(self.max_spawn_delay as i16),
            ),
            (
                "SpawnCount".to_string(),
                Tag::Short(self.spawn_count as i16),
            ),
            (
                "MaxNearbyEntities".to_string(),
                Tag::Short(self.max_nearby_entities as i16),
            ),
            (
                "RequiredPlayerRange".to_string(),
                Tag::Short(self.required_player_range as i16),
            ),
            (
                "SpawnRange".to_string(),
                Tag::Short(self.spawn_range as i16),
            ),
        ];
        if let Some(data) = &self.next_spawn_data {
            entries.push(("SpawnData".to_string(), data.to_tag()));
        }
        entries.push((
            "SpawnPotentials".to_string(),
            Tag::List(
                self.spawn_potentials
                    .iter()
                    .map(SpawnDataModel::to_tag)
                    .collect(),
            ),
        ));
        Tag::Compound(entries)
    }

    pub fn update_tag(&self) -> Tag {
        let Tag::Compound(mut entries) = self.save_additional() else {
            return Tag::Compound(Vec::new());
        };
        entries.retain(|(name, _)| name != "SpawnPotentials");
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut spawner = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return spawner;
        };
        spawner.spawn_delay = get_short(entries, "Delay").unwrap_or(20);
        spawner.min_spawn_delay = get_short(entries, "MinSpawnDelay").unwrap_or(200);
        spawner.max_spawn_delay = get_short(entries, "MaxSpawnDelay").unwrap_or(800);
        spawner.spawn_count = get_short(entries, "SpawnCount").unwrap_or(4);
        spawner.max_nearby_entities = get_short(entries, "MaxNearbyEntities").unwrap_or(6);
        spawner.required_player_range = get_short(entries, "RequiredPlayerRange").unwrap_or(16);
        spawner.spawn_range = get_short(entries, "SpawnRange").unwrap_or(4);
        spawner.next_spawn_data = entries
            .iter()
            .find(|(name, _)| name == "SpawnData")
            .and_then(|(_, tag)| SpawnDataModel::from_tag(tag));
        if let Some(Tag::List(potentials)) = entries
            .iter()
            .find(|(name, _)| name == "SpawnPotentials")
            .map(|(_, tag)| tag)
        {
            spawner.spawn_potentials = potentials
                .iter()
                .filter_map(SpawnDataModel::from_tag)
                .collect();
        }
        if spawner.spawn_potentials.is_empty() {
            spawner.spawn_potentials = vec![spawner.next_spawn_data.clone().unwrap_or_default()];
        }
        spawner
    }
}

impl TrialSpawnerStateModel {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::WaitingForPlayers => "waiting_for_players",
            Self::Active => "active",
            Self::WaitingForRewardEjection => "waiting_for_reward_ejection",
            Self::EjectingReward => "ejecting_reward",
            Self::Cooldown => "cooldown",
        }
    }

    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "inactive" => Some(Self::Inactive),
            "waiting_for_players" => Some(Self::WaitingForPlayers),
            "active" => Some(Self::Active),
            "waiting_for_reward_ejection" => Some(Self::WaitingForRewardEjection),
            "ejecting_reward" => Some(Self::EjectingReward),
            "cooldown" => Some(Self::Cooldown),
            _ => None,
        }
    }

    pub fn light_level(self) -> i32 {
        match self {
            Self::Inactive | Self::Cooldown => 0,
            Self::WaitingForPlayers => 4,
            Self::Active | Self::WaitingForRewardEjection | Self::EjectingReward => 8,
        }
    }

    pub fn spinning_mob_speed(self) -> f64 {
        match self {
            Self::WaitingForPlayers => 200.0,
            Self::Active => 1000.0,
            Self::Inactive
            | Self::WaitingForRewardEjection
            | Self::EjectingReward
            | Self::Cooldown => -1.0,
        }
    }

    pub fn has_spinning_mob(self) -> bool {
        self.spinning_mob_speed() >= 0.0
    }

    pub fn is_capable_of_spawning(self) -> bool {
        matches!(self, Self::WaitingForPlayers | Self::Active)
    }

    pub fn particle_emission(self) -> TrialSpawnerParticleEmission {
        match self {
            Self::Inactive => TrialSpawnerParticleEmission::None,
            Self::WaitingForPlayers | Self::WaitingForRewardEjection | Self::EjectingReward => {
                TrialSpawnerParticleEmission::SmallFlames
            }
            Self::Active => TrialSpawnerParticleEmission::FlamesAndSmoke,
            Self::Cooldown => TrialSpawnerParticleEmission::SmokeInsideAndTopFace,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialSpawnerParticleEmission {
    None,
    SmallFlames,
    FlamesAndSmoke,
    SmokeInsideAndTopFace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialSpawnerFlameParticle {
    Normal,
    Ominous,
}

impl TrialSpawnerFlameParticle {
    pub fn encode(self) -> i32 {
        match self {
            Self::Normal => 0,
            Self::Ominous => 1,
        }
    }

    pub fn decode(data: i32) -> Self {
        match data {
            1 => Self::Ominous,
            _ => Self::Normal,
        }
    }

    pub fn particle_id(self) -> &'static str {
        match self {
            Self::Normal => "minecraft:flame",
            Self::Ominous => "minecraft:soul_fire_flame",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialSpawnerTrackedMob {
    pub exists: bool,
    pub alive: bool,
    pub same_dimension: bool,
    pub distance_squared: i32,
}

impl Default for TrialSpawnerConfigModel {
    fn default() -> Self {
        Self {
            spawn_range: 4,
            total_mobs: 6.0,
            simultaneous_mobs: 2.0,
            total_mobs_added_per_player: 2.0,
            simultaneous_mobs_added_per_player: 1.0,
            ticks_between_spawn: 40,
            spawn_potentials: Vec::new(),
            loot_tables_to_eject: vec![
                "minecraft:spawners/trial_chamber/consumables".to_string(),
                "minecraft:spawners/trial_chamber/key".to_string(),
            ],
            items_to_drop_when_ominous:
                "minecraft:spawners/trial_chamber/items_to_drop_when_ominous".to_string(),
        }
    }
}

impl TrialSpawnerConfigModel {
    pub const TICKS_BETWEEN_ITEM_SPAWNERS: i64 = 160;

    pub fn target_total_mobs(&self, additional_players: usize) -> i32 {
        (self.total_mobs + self.total_mobs_added_per_player * additional_players as f32).floor()
            as i32
    }

    pub fn target_simultaneous_mobs(&self, additional_players: usize) -> i32 {
        (self.simultaneous_mobs
            + self.simultaneous_mobs_added_per_player * additional_players as f32)
            .floor() as i32
    }

    pub fn ticks_between_item_spawners(&self) -> i64 {
        Self::TICKS_BETWEEN_ITEM_SPAWNERS
    }

    pub fn with_spawning(&self, entity_id: impl Into<String>) -> Self {
        let mut config = self.clone();
        config.spawn_potentials = vec![SpawnDataModel::new(entity_id)];
        config
    }

    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("spawn_range".to_string(), Tag::Int(self.spawn_range)),
            ("total_mobs".to_string(), Tag::Float(self.total_mobs)),
            (
                "simultaneous_mobs".to_string(),
                Tag::Float(self.simultaneous_mobs),
            ),
            (
                "total_mobs_added_per_player".to_string(),
                Tag::Float(self.total_mobs_added_per_player),
            ),
            (
                "simultaneous_mobs_added_per_player".to_string(),
                Tag::Float(self.simultaneous_mobs_added_per_player),
            ),
            (
                "ticks_between_spawn".to_string(),
                Tag::Int(self.ticks_between_spawn),
            ),
            (
                "spawn_potentials".to_string(),
                Tag::List(
                    self.spawn_potentials
                        .iter()
                        .map(SpawnDataModel::to_tag)
                        .collect(),
                ),
            ),
            (
                "loot_tables_to_eject".to_string(),
                Tag::List(
                    self.loot_tables_to_eject
                        .iter()
                        .map(|value| Tag::String(value.clone()))
                        .collect(),
                ),
            ),
            (
                "items_to_drop_when_ominous".to_string(),
                Tag::String(self.items_to_drop_when_ominous.clone()),
            ),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Self {
        let mut config = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return config;
        };
        config.spawn_range = get_int(entries, "spawn_range").unwrap_or(config.spawn_range);
        config.total_mobs = get_float(entries, "total_mobs").unwrap_or(config.total_mobs);
        config.simultaneous_mobs =
            get_float(entries, "simultaneous_mobs").unwrap_or(config.simultaneous_mobs);
        config.total_mobs_added_per_player = get_float(entries, "total_mobs_added_per_player")
            .unwrap_or(config.total_mobs_added_per_player);
        config.simultaneous_mobs_added_per_player =
            get_float(entries, "simultaneous_mobs_added_per_player")
                .unwrap_or(config.simultaneous_mobs_added_per_player);
        config.ticks_between_spawn =
            get_int(entries, "ticks_between_spawn").unwrap_or(config.ticks_between_spawn);
        if let Some(Tag::List(values)) = entries
            .iter()
            .find(|(name, _)| name == "spawn_potentials")
            .map(|(_, tag)| tag)
        {
            config.spawn_potentials = values.iter().filter_map(SpawnDataModel::from_tag).collect();
        }
        if let Some(values) = string_list_field(entries, "loot_tables_to_eject") {
            config.loot_tables_to_eject = values;
        }
        config.items_to_drop_when_ominous = get_string(entries, "items_to_drop_when_ominous")
            .unwrap_or(&config.items_to_drop_when_ominous)
            .to_string();
        config
    }
}

impl Default for TrialSpawnerFullConfigModel {
    fn default() -> Self {
        Self {
            normal_config: TrialSpawnerConfigModel::default(),
            ominous_config: TrialSpawnerConfigModel::default(),
            target_cooldown_length: 36_000,
            required_player_range: 14,
        }
    }
}

impl TrialSpawnerFullConfigModel {
    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("normal_config".to_string(), self.normal_config.to_tag()),
            ("ominous_config".to_string(), self.ominous_config.to_tag()),
            (
                "target_cooldown_length".to_string(),
                Tag::Int(self.target_cooldown_length),
            ),
            (
                "required_player_range".to_string(),
                Tag::Int(self.required_player_range),
            ),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Self {
        let mut config = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return config;
        };
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "normal_config") {
            config.normal_config = TrialSpawnerConfigModel::from_tag(tag);
        }
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "ominous_config") {
            config.ominous_config = TrialSpawnerConfigModel::from_tag(tag);
        }
        config.target_cooldown_length =
            get_int(entries, "target_cooldown_length").unwrap_or(config.target_cooldown_length);
        config.required_player_range =
            get_int(entries, "required_player_range").unwrap_or(config.required_player_range);
        config
    }
}

impl Default for TrialSpawnerBlockEntity {
    fn default() -> Self {
        Self {
            state: TrialSpawnerStateModel::Inactive,
            is_ominous: false,
            config: TrialSpawnerFullConfigModel::default(),
            detected_players: Vec::new(),
            current_mobs: Vec::new(),
            cooldown_ends_at: 0,
            next_mob_spawns_at: 0,
            total_mobs_spawned: 0,
            next_spawn_data: None,
            ejecting_loot_table: None,
            spin: 0.0,
            old_spin: 0.0,
        }
    }
}

impl TrialSpawnerBlockEntity {
    pub const DETECT_PLAYER_SPAWN_BUFFER: i64 = 40;
    pub const TIME_BETWEEN_REWARD_EJECTIONS: i64 = 30;
    pub const TICKS_BETWEEN_OMINOUS_ITEM_SPAWNERS: i64 = 160;
    pub const BLOCK_UPDATE_FLAGS: i32 = 3;
    pub const SPAWN_MOB_EVENT: i32 = 3011;
    pub const SPAWN_MOB_AT_EVENT: i32 = 3012;
    pub const DETECT_PLAYER_EVENT: i32 = 3013;
    pub const EJECT_REWARD_EVENT: i32 = 3014;
    pub const OMINOUS_DETECT_PLAYER_EVENT: i32 = 3019;
    pub const BECOME_OMINOUS_EVENT: i32 = 3020;
    pub const SPAWN_PARTICLE_COUNT: i32 = 20;
    pub const DETECT_PLAYER_BASE_PARTICLE_COUNT: i32 = 30;
    pub const DETECT_PLAYER_PARTICLES_PER_PLAYER: i32 = 5;
    pub const DETECT_PLAYER_MAX_PLAYER_BONUS: i32 = 10;
    pub const EJECT_ITEM_PARTICLE_COUNT: i32 = 20;
    pub const SPAWNING_AMBIENT_SOUND_CHANCE: f32 = 0.02;
    pub const DELAY_BEFORE_EJECT_AFTER_KILLING_LAST_MOB: i64 = 40;
    pub const DELAY_BETWEEN_PLAYER_SCANS: i64 = 20;
    pub const TRIAL_OMEN_PER_BAD_OMEN_LEVEL: i32 = 18_000;
    pub const MAX_MOB_TRACKING_DISTANCE: i32 = 47;
    pub const MAX_MOB_TRACKING_DISTANCE_SQR: i32 =
        Self::MAX_MOB_TRACKING_DISTANCE * Self::MAX_MOB_TRACKING_DISTANCE;

    pub fn active_config(&self) -> &TrialSpawnerConfigModel {
        if self.is_ominous {
            &self.config.ominous_config
        } else {
            &self.config.normal_config
        }
    }

    pub fn can_spawn_in_level(
        spawner_blocks_work: bool,
        override_peaceful_and_mob_spawn_rule: bool,
        peaceful: bool,
        spawn_mobs_rule: bool,
    ) -> bool {
        spawner_blocks_work
            && (override_peaceful_and_mob_spawn_rule || (!peaceful && spawn_mobs_rule))
    }

    pub fn get_state(&self, has_trial_spawner_state_property: bool) -> TrialSpawnerStateModel {
        if has_trial_spawner_state_property {
            self.state
        } else {
            TrialSpawnerStateModel::Inactive
        }
    }

    pub fn set_state(&mut self, state: TrialSpawnerStateModel) -> i32 {
        self.state = state;
        Self::BLOCK_UPDATE_FLAGS
    }

    pub fn mark_updated_flags(&self) -> i32 {
        Self::BLOCK_UPDATE_FLAGS
    }

    pub fn set_entity_id(&mut self, entity_id: impl Into<String>, has_level: bool) -> bool {
        if !has_level {
            return false;
        }
        self.override_entity_to_spawn(entity_id);
        true
    }

    pub fn tick_client(&mut self, game_time: i64) {
        if self.state.has_spinning_mob() {
            let spawn_delay = (self.next_mob_spawns_at - game_time).max(0) as f64;
            self.old_spin = self.spin;
            self.spin =
                (self.spin + self.state.spinning_mob_speed() / (spawn_delay + 200.0)) % 360.0;
        }
    }

    pub fn reward_ejection_position(pos: BlockPos) -> (f64, f64, f64) {
        (pos.x as f64 + 0.5, pos.y as f64 + 1.2, pos.z as f64 + 0.5)
    }

    pub fn should_mob_be_untracked(mob: TrialSpawnerTrackedMob) -> bool {
        !mob.exists
            || !mob.alive
            || !mob.same_dimension
            || mob.distance_squared > Self::MAX_MOB_TRACKING_DISTANCE_SQR
    }

    pub fn detect_player_particle_count(detected_player_count: i32) -> i32 {
        Self::DETECT_PLAYER_BASE_PARTICLE_COUNT
            + detected_player_count.clamp(0, Self::DETECT_PLAYER_MAX_PLAYER_BONUS)
                * Self::DETECT_PLAYER_PARTICLES_PER_PLAYER
    }

    pub fn block_pos_as_long(pos: BlockPos) -> i64 {
        const HORIZONTAL_BITS: u32 = 26;
        const Y_BITS: u32 = 12;
        const Z_OFFSET: u32 = Y_BITS;
        const X_OFFSET: u32 = Y_BITS + HORIZONTAL_BITS;
        const HORIZONTAL_MASK: i64 = (1_i64 << HORIZONTAL_BITS) - 1;
        const Y_MASK: i64 = (1_i64 << Y_BITS) - 1;

        ((pos.x as i64 & HORIZONTAL_MASK) << X_OFFSET)
            | (pos.y as i64 & Y_MASK)
            | ((pos.z as i64 & HORIZONTAL_MASK) << Z_OFFSET)
    }

    pub fn is_player_scan_throttled(pos: BlockPos, game_time: i64) -> bool {
        (Self::block_pos_as_long(pos) + game_time) % Self::DELAY_BETWEEN_PLAYER_SCANS != 0
    }

    pub fn count_additional_players(detected_player_count: usize) -> usize {
        detected_player_count.saturating_sub(1)
    }

    pub fn is_ready_to_spawn_next_mob(&self, game_time: i64, additional_players: usize) -> bool {
        game_time >= self.next_mob_spawns_at
            && self.current_mobs.len()
                < self
                    .active_config()
                    .target_simultaneous_mobs(additional_players) as usize
    }

    pub fn is_ready_to_open_shutter(
        game_time: i64,
        cooldown_ends_at: i64,
        delay_before_open: f32,
        target_cooldown_length: i32,
    ) -> bool {
        let cooldown_started_at = cooldown_ends_at - i64::from(target_cooldown_length);
        game_time as f32 >= cooldown_started_at as f32 + delay_before_open
    }

    pub fn is_ready_to_eject_items(
        game_time: i64,
        cooldown_ends_at: i64,
        time_between_ejections: f32,
        target_cooldown_length: i32,
    ) -> bool {
        let cooldown_started_at = cooldown_ends_at - i64::from(target_cooldown_length);
        ((game_time - cooldown_started_at) as f32 % time_between_ejections) == 0.0
    }

    pub fn is_cooldown_finished(game_time: i64, cooldown_ends_at: i64) -> bool {
        game_time >= cooldown_ends_at
    }

    pub fn low_resolution_position_seed(level_seed: i64, pos: BlockPos) -> i64 {
        let low_resolution = BlockPos {
            x: pos.x.div_euclid(30),
            y: pos.y.div_euclid(20),
            z: pos.z.div_euclid(30),
        };
        level_seed + Self::block_pos_as_long(low_resolution)
    }

    pub fn trial_omen_duration_from_bad_omen_amplifier(amplifier: i32) -> i32 {
        Self::TRIAL_OMEN_PER_BAD_OMEN_LEVEL * (amplifier + 1)
    }

    pub fn reset_statistics(&mut self) {
        self.detected_players.clear();
        self.total_mobs_spawned = 0;
        self.next_mob_spawns_at = 0;
        self.cooldown_ends_at = 0;
    }

    pub fn reset_state_data(&mut self) {
        self.current_mobs.clear();
        self.next_spawn_data = None;
        self.reset_statistics();
    }

    // TODO(trial-spawner-live-level): wire Java's live ServerLevel operations for spawnMob,
    // ejectReward, particle emission, ambient sounds, and ominous item spawner placement once the
    // entity loader, loot-table runtime, world collision/clip, and level-event systems are unified.
    pub fn apply_ominous(&mut self, game_time: i64) {
        self.is_ominous = true;
        self.current_mobs.clear();
        self.total_mobs_spawned = 0;
        self.next_spawn_data = None;
        self.next_mob_spawns_at =
            game_time + i64::from(self.config.ominous_config.ticks_between_spawn);
        self.cooldown_ends_at = game_time + Self::TICKS_BETWEEN_OMINOUS_ITEM_SPAWNERS;
    }

    pub fn override_entity_to_spawn(&mut self, entity_id: impl Into<String>) {
        let data = SpawnDataModel::new(entity_id);
        self.config.normal_config.spawn_potentials = vec![data.clone()];
        self.config.ominous_config.spawn_potentials = vec![data];
        self.next_spawn_data = None;
        self.state = TrialSpawnerStateModel::Inactive;
        self.detected_players.clear();
        self.current_mobs.clear();
        self.total_mobs_spawned = 0;
    }

    pub fn tick_server(&mut self, context: TrialSpawnerTickContext) -> TrialSpawnerTickResult {
        self.current_mobs
            .truncate(context.current_mobs_alive.min(self.current_mobs.len()));
        if context.apply_ominous && !self.is_ominous {
            self.apply_ominous(context.game_time);
            return TrialSpawnerTickResult::BecameOminous;
        }

        match self.state {
            TrialSpawnerStateModel::Inactive => self.tick_inactive(),
            TrialSpawnerStateModel::WaitingForPlayers => self.tick_waiting_for_players(context),
            TrialSpawnerStateModel::Active => self.tick_active(context),
            TrialSpawnerStateModel::WaitingForRewardEjection => {
                self.tick_waiting_for_reward_ejection(context.game_time)
            }
            TrialSpawnerStateModel::EjectingReward => self.tick_ejecting_reward(context),
            TrialSpawnerStateModel::Cooldown => self.tick_cooldown(context),
        }
    }

    fn tick_inactive(&mut self) -> TrialSpawnerTickResult {
        self.state = TrialSpawnerStateModel::WaitingForPlayers;
        TrialSpawnerTickResult::StateChanged(self.state)
    }

    fn tick_waiting_for_players(
        &mut self,
        context: TrialSpawnerTickContext,
    ) -> TrialSpawnerTickResult {
        if !context.can_spawn_in_level || self.active_config().spawn_potentials.is_empty() {
            return TrialSpawnerTickResult::Waiting;
        }
        self.detect_players(context.detected_player_count, context.game_time);
        if self.detected_players.is_empty() {
            TrialSpawnerTickResult::Waiting
        } else {
            self.state = TrialSpawnerStateModel::Active;
            TrialSpawnerTickResult::DetectedPlayers(self.detected_players.len())
        }
    }

    fn tick_active(&mut self, context: TrialSpawnerTickContext) -> TrialSpawnerTickResult {
        if !context.can_spawn_in_level {
            self.state = TrialSpawnerStateModel::WaitingForPlayers;
            return TrialSpawnerTickResult::StateChanged(self.state);
        }
        self.detect_players(context.detected_player_count, context.game_time);
        let additional_players = self.detected_players.len().saturating_sub(1);
        let target_total = self.active_config().target_total_mobs(additional_players);
        if self.total_mobs_spawned >= target_total {
            return self.finish_spawning_if_mobs_defeated(context.game_time);
        }

        let simultaneous = self
            .active_config()
            .target_simultaneous_mobs(additional_players);
        if context.game_time >= self.next_mob_spawns_at
            && (self.current_mobs.len() as i32) < simultaneous
            && context.spawn_success
        {
            self.spawn_trial_mob(context)
        } else {
            TrialSpawnerTickResult::Waiting
        }
    }

    fn finish_spawning_if_mobs_defeated(&mut self, game_time: i64) -> TrialSpawnerTickResult {
        if !self.current_mobs.is_empty() {
            return TrialSpawnerTickResult::Waiting;
        }
        self.cooldown_ends_at = game_time + i64::from(self.config.target_cooldown_length);
        self.total_mobs_spawned = 0;
        self.next_mob_spawns_at = 0;
        self.state = TrialSpawnerStateModel::WaitingForRewardEjection;
        TrialSpawnerTickResult::ReadyForRewards
    }

    fn spawn_trial_mob(&mut self, context: TrialSpawnerTickContext) -> TrialSpawnerTickResult {
        let spawn_data = self.select_next_spawn_data(context.roll);
        let entity_id = spawn_data
            .entity_id()
            .unwrap_or("minecraft:pig")
            .to_string();
        self.current_mobs
            .push(format!("mob-{}", self.total_mobs_spawned + 1));
        self.total_mobs_spawned += 1;
        self.next_mob_spawns_at =
            context.game_time + i64::from(self.active_config().ticks_between_spawn);
        self.next_spawn_data =
            weighted_spawn_data(&self.active_config().spawn_potentials, context.roll).cloned();
        TrialSpawnerTickResult::SpawnMob { entity_id }
    }

    fn tick_waiting_for_reward_ejection(&mut self, game_time: i64) -> TrialSpawnerTickResult {
        let cooldown_started_at =
            self.cooldown_ends_at - i64::from(self.config.target_cooldown_length);
        if game_time >= cooldown_started_at + Self::DETECT_PLAYER_SPAWN_BUFFER {
            self.state = TrialSpawnerStateModel::EjectingReward;
            TrialSpawnerTickResult::StateChanged(self.state)
        } else {
            TrialSpawnerTickResult::Waiting
        }
    }

    fn tick_ejecting_reward(&mut self, context: TrialSpawnerTickContext) -> TrialSpawnerTickResult {
        let cooldown_started_at =
            self.cooldown_ends_at - i64::from(self.config.target_cooldown_length);
        if (context.game_time - cooldown_started_at) % Self::TIME_BETWEEN_REWARD_EJECTIONS != 0 {
            return TrialSpawnerTickResult::Waiting;
        }
        if self.detected_players.is_empty() {
            self.ejecting_loot_table = None;
            self.state = TrialSpawnerStateModel::Cooldown;
            return TrialSpawnerTickResult::StateChanged(self.state);
        }
        let loot_table = self
            .ejecting_loot_table
            .clone()
            .or_else(|| {
                self.active_config()
                    .loot_tables_to_eject
                    .get(context.roll)
                    .cloned()
            })
            .unwrap_or_else(|| "minecraft:empty".to_string());
        self.ejecting_loot_table = Some(loot_table.clone());
        self.detected_players.remove(0);
        TrialSpawnerTickResult::EjectedReward {
            loot_table,
            remaining_players: self.detected_players.len(),
        }
    }

    fn tick_cooldown(&mut self, context: TrialSpawnerTickContext) -> TrialSpawnerTickResult {
        self.detect_players(context.detected_player_count, context.game_time);
        if !self.detected_players.is_empty() {
            self.total_mobs_spawned = 0;
            self.next_mob_spawns_at = 0;
            self.state = TrialSpawnerStateModel::Active;
            TrialSpawnerTickResult::StateChanged(self.state)
        } else if context.game_time >= self.cooldown_ends_at {
            self.is_ominous = false;
            self.current_mobs.clear();
            self.next_spawn_data = None;
            self.ejecting_loot_table = None;
            self.state = TrialSpawnerStateModel::WaitingForPlayers;
            TrialSpawnerTickResult::CooldownFinished
        } else {
            TrialSpawnerTickResult::Waiting
        }
    }

    pub(super) fn detect_players(&mut self, count: usize, game_time: i64) {
        let previous_count = self.detected_players.len();
        for index in self.detected_players.len()..count {
            self.detected_players.push(format!("player-{index}"));
        }
        if self.detected_players.len() > previous_count {
            self.next_mob_spawns_at = self
                .next_mob_spawns_at
                .max(game_time + Self::DETECT_PLAYER_SPAWN_BUFFER);
        }
    }

    pub(super) fn select_next_spawn_data(&mut self, roll: usize) -> SpawnDataModel {
        if let Some(data) = &self.next_spawn_data {
            return data.clone();
        }
        let data = weighted_spawn_data(&self.active_config().spawn_potentials, roll)
            .cloned()
            .unwrap_or_default();
        self.next_spawn_data = Some(data.clone());
        data
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![
            (
                "state".to_string(),
                Tag::String(self.state.as_str().to_string()),
            ),
            (
                "is_ominous".to_string(),
                Tag::Byte(i8::from(self.is_ominous)),
            ),
            ("config".to_string(), self.config.to_tag()),
            (
                "registered_players".to_string(),
                Tag::List(
                    self.detected_players
                        .iter()
                        .map(|value| Tag::String(value.clone()))
                        .collect(),
                ),
            ),
            (
                "current_mobs".to_string(),
                Tag::List(
                    self.current_mobs
                        .iter()
                        .map(|value| Tag::String(value.clone()))
                        .collect(),
                ),
            ),
            (
                "cooldown_ends_at".to_string(),
                Tag::Long(self.cooldown_ends_at),
            ),
            (
                "next_mob_spawns_at".to_string(),
                Tag::Long(self.next_mob_spawns_at),
            ),
            (
                "total_mobs_spawned".to_string(),
                Tag::Int(self.total_mobs_spawned),
            ),
        ];
        if let Some(data) = &self.next_spawn_data {
            fields.push(("spawn_data".to_string(), data.to_tag()));
        }
        if let Some(loot_table) = &self.ejecting_loot_table {
            fields.push((
                "ejecting_loot_table".to_string(),
                Tag::String(loot_table.clone()),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn update_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if self.state == TrialSpawnerStateModel::Active {
            fields.push((
                "next_mob_spawns_at".to_string(),
                Tag::Long(self.next_mob_spawns_at),
            ));
        }
        if let Some(data) = &self.next_spawn_data {
            fields.push(("spawn_data".to_string(), data.to_tag()));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut spawner = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return spawner;
        };
        spawner.state = get_string(entries, "state")
            .and_then(TrialSpawnerStateModel::from_str)
            .unwrap_or(TrialSpawnerStateModel::Inactive);
        spawner.is_ominous = get_byte(entries, "is_ominous").unwrap_or(0) != 0;
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "config") {
            spawner.config = TrialSpawnerFullConfigModel::from_tag(tag);
        }
        spawner.detected_players =
            string_list_field(entries, "registered_players").unwrap_or_default();
        spawner.current_mobs = string_list_field(entries, "current_mobs").unwrap_or_default();
        spawner.cooldown_ends_at = get_long(entries, "cooldown_ends_at").unwrap_or(0);
        spawner.next_mob_spawns_at = get_long(entries, "next_mob_spawns_at").unwrap_or(0);
        spawner.total_mobs_spawned = get_int(entries, "total_mobs_spawned").unwrap_or(0);
        spawner.next_spawn_data = entries
            .iter()
            .find(|(name, _)| name == "spawn_data")
            .and_then(|(_, tag)| SpawnDataModel::from_tag(tag));
        spawner.ejecting_loot_table =
            get_string(entries, "ejecting_loot_table").map(ToString::to_string);
        spawner
    }
}
