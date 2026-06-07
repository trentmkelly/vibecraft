use super::*;

impl BeehiveOccupant {
    pub const DEFAULT_ENTITY_TYPE: &'static str = "minecraft:bee";

    pub fn bee(ticks_in_hive: i32, has_nectar: bool) -> Self {
        let mut entity_fields = Vec::new();
        if has_nectar {
            entity_fields.push(("HasNectar".to_string(), Tag::Byte(1)));
        }
        Self {
            entity_type: Self::DEFAULT_ENTITY_TYPE.to_string(),
            entity_data: Tag::Compound(entity_fields),
            ticks_in_hive: ticks_in_hive.max(0),
            min_ticks_in_hive: if has_nectar {
                BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTAR
            } else {
                BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTARLESS
            },
        }
    }

    pub fn has_nectar(&self) -> bool {
        compound_entries(&self.entity_data)
            .and_then(|entries| get_bool(entries, "HasNectar"))
            .unwrap_or(false)
    }

    pub(super) fn tick_ready(&mut self) -> bool {
        let was_ready = self.ticks_in_hive > self.min_ticks_in_hive;
        self.ticks_in_hive += 1;
        was_ready
    }

    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "entity_type".to_string(),
                Tag::String(self.entity_type.clone()),
            ),
            ("entity_data".to_string(), self.entity_data.clone()),
            ("ticks_in_hive".to_string(), Tag::Int(self.ticks_in_hive)),
            (
                "min_ticks_in_hive".to_string(),
                Tag::Int(self.min_ticks_in_hive),
            ),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            entity_type: get_string(entries, "entity_type")
                .unwrap_or(Self::DEFAULT_ENTITY_TYPE)
                .to_string(),
            entity_data: entries
                .iter()
                .find(|(name, _)| name == "entity_data")
                .map(|(_, tag)| tag.clone())
                .unwrap_or_else(|| Tag::Compound(Vec::new())),
            ticks_in_hive: get_int(entries, "ticks_in_hive").unwrap_or(0).max(0),
            min_ticks_in_hive: get_int(entries, "min_ticks_in_hive")
                .unwrap_or(BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTARLESS)
                .max(0),
        })
    }
}

impl BeehiveBlockEntity {
    pub const MAX_OCCUPANTS: usize = 3;
    pub const MIN_TICKS_BEFORE_REENTERING_HIVE: i32 = 400;
    pub const MIN_OCCUPATION_TICKS_NECTAR: i32 = 2400;
    pub const MIN_OCCUPATION_TICKS_NECTARLESS: i32 = 600;
    pub const MAX_HONEY_LEVEL: i32 = 5;
    pub const PLAYER_ANGER_RADIUS_SQUARED: f64 = 16.0;
    pub const WORK_SOUND_CHANCE: f64 = 0.005;

    pub fn new() -> Self {
        Self {
            occupants: Vec::new(),
            saved_flower_pos: None,
            honey_level: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.occupants.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.occupants.len() == Self::MAX_OCCUPANTS
    }

    pub fn occupant_count(&self) -> usize {
        self.occupants.len()
    }

    pub fn add_occupant(
        &mut self,
        occupant: BeehiveOccupant,
        saved_flower_pos: Option<BlockPos>,
    ) -> bool {
        if self.is_full() {
            return false;
        }
        if self.saved_flower_pos.is_none() {
            self.saved_flower_pos = saved_flower_pos;
        }
        self.occupants.push(occupant);
        true
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![
            (
                "bees".to_string(),
                Tag::List(self.occupants.iter().map(BeehiveOccupant::to_tag).collect()),
            ),
            ("honey_level".to_string(), Tag::Int(self.honey_level)),
        ];
        if let Some(pos) = self.saved_flower_pos {
            fields.push(("flower_pos".to_string(), block_pos_to_tag(pos)));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let occupants = entries
            .iter()
            .find(|(name, _)| name == "bees")
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(BeehiveOccupant::from_tag)
                        .take(Self::MAX_OCCUPANTS)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Self {
            occupants,
            saved_flower_pos: entries
                .iter()
                .find(|(name, _)| name == "flower_pos")
                .and_then(|(_, tag)| block_pos_from_tag(tag)),
            honey_level: get_int(entries, "honey_level")
                .unwrap_or(0)
                .clamp(0, Self::MAX_HONEY_LEVEL),
        }
    }

    pub fn tick(
        &mut self,
        bees_stay_in_hive: bool,
        front_blocked: bool,
        honey_bonus_roll: bool,
    ) -> Vec<BeeReleaseEvent> {
        let mut released = Vec::new();
        let mut index = 0;
        while index < self.occupants.len() {
            if self.occupants[index].tick_ready() {
                let status = if self.occupants[index].has_nectar() {
                    BeeReleaseStatus::HoneyDelivered
                } else {
                    BeeReleaseStatus::BeeReleased
                };
                if let Some(event) = self.release_at(
                    index,
                    status,
                    bees_stay_in_hive,
                    front_blocked,
                    honey_bonus_roll,
                ) {
                    released.push(event);
                    continue;
                }
            }
            index += 1;
        }
        released
    }

    pub fn empty_all_living_from_hive(
        &mut self,
        status: BeeReleaseStatus,
        is_sedated: bool,
    ) -> Vec<BeeReleaseEvent> {
        let mut released = Vec::new();
        while !self.occupants.is_empty() {
            if let Some(mut event) = self.release_at(0, status, false, false, false) {
                event.stay_out_of_hive_ticks = if is_sedated {
                    Self::MIN_TICKS_BEFORE_REENTERING_HIVE
                } else {
                    0
                };
                released.push(event);
            } else {
                break;
            }
        }
        released
    }

    pub fn on_fire_nearby(&mut self) -> Vec<BeeReleaseEvent> {
        self.empty_all_living_from_hive(BeeReleaseStatus::Emergency, false)
    }

    pub(super) fn release_at(
        &mut self,
        index: usize,
        status: BeeReleaseStatus,
        bees_stay_in_hive: bool,
        front_blocked: bool,
        honey_bonus_roll: bool,
    ) -> Option<BeeReleaseEvent> {
        if status != BeeReleaseStatus::Emergency && (bees_stay_in_hive || front_blocked) {
            return None;
        }
        let occupant = self.occupants.remove(index);
        if status == BeeReleaseStatus::HoneyDelivered && self.honey_level < Self::MAX_HONEY_LEVEL {
            let level_increase = if honey_bonus_roll { 2 } else { 1 };
            self.honey_level = (self.honey_level + level_increase).min(Self::MAX_HONEY_LEVEL);
        }
        Some(BeeReleaseEvent {
            entity_type: occupant.entity_type,
            status,
            honey_level: self.honey_level,
            stay_out_of_hive_ticks: 0,
        })
    }
}

impl CreakingHeartBlockEntity {
    pub const PLAYER_DETECTION_RANGE: i32 = 32;
    pub const CREAKING_ROAMING_RADIUS: i32 = 32;
    pub const DISTANCE_CREAKING_TOO_FAR: f64 = 34.0;
    pub const SPAWN_RANGE_XZ: i32 = 16;
    pub const SPAWN_RANGE_Y: i32 = 8;
    pub const ATTEMPTS_PER_SPAWN: i32 = 5;
    pub const UPDATE_TICKS: i32 = 20;
    pub const UPDATE_TICKS_VARIANCE: i32 = 5;
    pub const HURT_CALL_TOTAL_TICKS: i32 = 100;
    pub const NUMBER_OF_HURT_CALLS: i32 = 10;
    pub const HURT_CALL_INTERVAL: i32 = 10;
    pub const HURT_CALL_PARTICLE_TICKS: i32 = 50;
    pub const MAX_RESIN_DEPTH: i32 = 2;
    pub const MAX_RESIN_COUNT: i32 = 64;
    pub const TICKS_GRACE_PERIOD: i64 = 30;

    pub fn new() -> Self {
        Self {
            creaking_uuid: None,
            ticks_existed: 0,
            ticker: 0,
            emitter_ticks: 0,
            output_signal: 0,
            state: CreakingHeartStateModel::Uprooted,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(uuid) = &self.creaking_uuid {
            fields.push(("creaking".to_string(), Tag::String(uuid.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut heart = Self::new();
        if let Some(entries) = compound_entries(tag) {
            if let Some(uuid) = get_string(entries, "creaking") {
                heart.set_creaking_uuid(uuid.to_string());
            }
        }
        heart
    }

    pub fn set_creaking_uuid(&mut self, uuid: String) {
        self.creaking_uuid = Some(uuid);
        self.ticks_existed = 0;
    }

    pub fn clear_creaking(&mut self) {
        self.creaking_uuid = None;
    }

    pub fn server_tick(&mut self, context: CreakingHeartTickContext) -> Vec<CreakingHeartAction> {
        self.ticks_existed += 1;
        let mut actions = Vec::new();
        let computed_signal = self.compute_analog_output_signal(context.protector_distance);
        if self.output_signal != computed_signal {
            let previous = self.output_signal;
            self.output_signal = computed_signal;
            actions.push(CreakingHeartAction::OutputSignalChanged {
                previous,
                current: computed_signal,
            });
        }

        if self.emitter_ticks > 0 {
            self.emitter_ticks -= 1;
        }

        self.ticker -= 1;
        if self.ticker >= 0 {
            return actions;
        }
        self.ticker = Self::UPDATE_TICKS
            + context
                .next_ticker_offset
                .clamp(0, Self::UPDATE_TICKS_VARIANCE.saturating_sub(1));

        let updated_state = self.updated_state(context.has_required_logs, context.creaking_active);
        if updated_state != self.state {
            self.state = updated_state;
            actions.push(CreakingHeartAction::StateChanged(updated_state));
            if updated_state == CreakingHeartStateModel::Uprooted {
                return actions;
            }
        }

        if self.creaking_uuid.is_none() {
            if self.state == CreakingHeartStateModel::Awake
                && context.spawning_monsters
                && context.player_nearby
            {
                actions.push(CreakingHeartAction::SpawnProtector {
                    attempts: Self::ATTEMPTS_PER_SPAWN,
                    range_xz: Self::SPAWN_RANGE_XZ,
                    range_y: Self::SPAWN_RANGE_Y,
                });
            }
        } else if context.protector_resolved {
            let too_far = context
                .protector_distance
                .map(|distance| distance > Self::DISTANCE_CREAKING_TOO_FAR)
                .unwrap_or(false);
            if (!context.creaking_active && !context.protector_persistent)
                || too_far
                || context.player_stuck_in_protector
            {
                self.clear_creaking();
                actions.push(CreakingHeartAction::RemoveProtector);
            }
        } else if self.ticks_existed >= Self::TICKS_GRACE_PERIOD {
            self.clear_creaking();
            actions.push(CreakingHeartAction::RemoveProtector);
        }
        actions
    }

    pub fn on_protector_spawned(&mut self, uuid: String) {
        self.creaking_uuid = Some(uuid);
    }

    pub fn creaking_hurt(&mut self, state_awake: bool, resin_clumps: i32) -> CreakingHeartAction {
        if self.creaking_uuid.is_none() || self.emitter_ticks > 0 {
            return CreakingHeartAction::None;
        }
        self.emitter_ticks = Self::HURT_CALL_TOTAL_TICKS;
        CreakingHeartAction::HurtPulse {
            total_ticks: Self::HURT_CALL_TOTAL_TICKS,
            particle_ticks: Self::HURT_CALL_PARTICLE_TICKS,
            resin_clumps: if state_awake {
                resin_clumps.clamp(2, 3)
            } else {
                0
            },
        }
    }

    pub fn remove_protector(&mut self) -> CreakingHeartAction {
        if self.creaking_uuid.take().is_some() {
            CreakingHeartAction::RemoveProtector
        } else {
            CreakingHeartAction::None
        }
    }

    pub fn compute_analog_output_signal(&self, protector_distance: Option<f64>) -> i32 {
        if self.creaking_uuid.is_none() {
            return 0;
        }
        let Some(distance) = protector_distance else {
            return 0;
        };
        let scaled_distance = distance.clamp(0.0, f64::from(Self::CREAKING_ROAMING_RADIUS))
            / f64::from(Self::CREAKING_ROAMING_RADIUS);
        15 - (scaled_distance * 15.0).floor() as i32
    }

    pub(super) fn updated_state(
        &self,
        has_required_logs: bool,
        creaking_active: bool,
    ) -> CreakingHeartStateModel {
        if !has_required_logs && self.creaking_uuid.is_none() {
            CreakingHeartStateModel::Uprooted
        } else if creaking_active {
            CreakingHeartStateModel::Awake
        } else {
            CreakingHeartStateModel::Dormant
        }
    }
}

impl SculkShriekerBlockEntity {
    pub const LISTENER_RADIUS: i32 = 8;
    pub const WARNING_SOUND_RADIUS: i32 = 10;
    pub const WARDEN_SPAWN_ATTEMPTS: i32 = 20;
    pub const WARDEN_SPAWN_RANGE_XZ: i32 = 5;
    pub const WARDEN_SPAWN_RANGE_Y: i32 = 6;
    pub const DARKNESS_RADIUS: i32 = 40;
    pub const SHRIEKING_TICKS: i32 = 90;
    pub const WARDEN_SUMMON_WARNING_LEVEL: i32 = 4;

    pub fn new(can_summon: bool) -> Self {
        Self {
            warning_level: 0,
            vibration_data: VibrationData::new(),
            shrieking_ticks: 0,
            can_summon,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("warning_level".to_string(), Tag::Int(self.warning_level)),
            ("listener".to_string(), self.listener_tag()),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(false);
        };
        Self {
            warning_level: get_int(entries, "warning_level").unwrap_or(0).clamp(0, 4),
            vibration_data: entries
                .iter()
                .find(|(name, _)| name == "listener")
                .map(|(_, tag)| vibration_data_from_tag(tag))
                .unwrap_or_default(),
            shrieking_ticks: 0,
            can_summon: false,
        }
    }

    pub fn can_receive_vibration(
        &self,
        shrieking_block_state: bool,
        has_player_source: bool,
    ) -> bool {
        !shrieking_block_state && has_player_source
    }

    pub fn try_shriek(
        &mut self,
        has_player: bool,
        can_respond: bool,
        tracker_warning_level: Option<i32>,
        warden_spawn_available: bool,
    ) -> SculkShriekResult {
        if !has_player || self.shrieking_ticks > 0 {
            return SculkShriekResult::Ignored;
        }

        self.warning_level = 0;
        if can_respond && tracker_warning_level.is_none() {
            return SculkShriekResult::Ignored;
        }
        if let Some(warning_level) = tracker_warning_level {
            self.warning_level = warning_level.clamp(0, Self::WARDEN_SUMMON_WARNING_LEVEL);
        }
        self.start_shrieking();
        self.try_respond(can_respond, warden_spawn_available)
    }

    pub fn try_respond(
        &self,
        can_respond: bool,
        warden_spawn_available: bool,
    ) -> SculkShriekResult {
        if !can_respond || !self.can_summon || self.warning_level <= 0 {
            return SculkShriekResult::Shriek {
                warning_level: self.warning_level,
            };
        }
        if self.warning_level >= Self::WARDEN_SUMMON_WARNING_LEVEL && warden_spawn_available {
            SculkShriekResult::SummonWarden {
                warning_level: self.warning_level,
                attempts: Self::WARDEN_SPAWN_ATTEMPTS,
                range_xz: Self::WARDEN_SPAWN_RANGE_XZ,
                range_y: Self::WARDEN_SPAWN_RANGE_Y,
                darkness_radius: Self::DARKNESS_RADIUS,
            }
        } else {
            SculkShriekResult::ReplySound {
                warning_level: self.warning_level,
                darkness_radius: Self::DARKNESS_RADIUS,
            }
        }
    }

    pub fn tick(&mut self) -> SculkShriekResult {
        if self.shrieking_ticks > 0 {
            self.shrieking_ticks -= 1;
        }
        SculkShriekResult::Ignored
    }

    pub(super) fn start_shrieking(&mut self) {
        self.shrieking_ticks = Self::SHRIEKING_TICKS;
    }

    pub(super) fn listener_tag(&self) -> Tag {
        let mut fields = vec![(
            "travel_time_in_ticks".to_string(),
            Tag::Int(self.vibration_data.travel_time_in_ticks),
        )];
        if let Some(vibration) = &self.vibration_data.current_vibration {
            fields.push((
                "event".to_string(),
                Tag::String(vibration.event.id.to_string()),
            ));
            fields.push(("distance".to_string(), Tag::Float(vibration.distance)));
        }
        Tag::Compound(fields)
    }
}

impl BellBlockEntity {
    pub const EVENT_RING: i32 = 1;
    pub const DURATION: i32 = 50;
    pub const GLOW_DURATION: i32 = 60;
    pub const MIN_TICKS_BETWEEN_SEARCHES: u64 = 60;
    pub const MAX_RESONATION_TICKS: i32 = 40;
    pub const TICKS_BEFORE_RESONATION: i32 = 5;
    pub const SEARCH_RADIUS: f64 = 48.0;
    pub const HEAR_BELL_RADIUS: f64 = 32.0;
    pub const HIGHLIGHT_RAIDERS_RADIUS: f64 = 48.0;

    pub fn new() -> Self {
        Self {
            last_ring_timestamp: 0,
            ticks: 0,
            shaking: false,
            click_direction: None,
            heard_bell_entities: 0,
            nearby_raiders_within_hear_radius: 0,
            nearby_raiders_within_highlight_radius: 0,
            resonating: false,
            resonation_ticks: 0,
        }
    }

    pub fn direction_3d_data_value(direction: Direction) -> i32 {
        match direction {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
        }
    }

    pub fn direction_from_3d_data_value(value: i32) -> Direction {
        match value {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            _ => Direction::Down,
        }
    }

    pub fn on_hit(&mut self, click_direction: Direction) -> BellBlockEvent {
        self.click_direction = Some(click_direction);
        if self.shaking {
            self.ticks = 0;
        } else {
            self.shaking = true;
        }
        BellBlockEvent {
            event_id: Self::EVENT_RING,
            event_param: Self::direction_3d_data_value(click_direction),
        }
    }

    pub fn trigger_event(
        &mut self,
        event_id: i32,
        event_param: i32,
        game_time: u64,
        nearby_living_within_hear_radius: usize,
        nearby_raiders_within_hear_radius: usize,
        nearby_raiders_within_highlight_radius: usize,
    ) -> bool {
        if event_id != Self::EVENT_RING {
            return false;
        }
        self.update_entities(
            game_time,
            nearby_living_within_hear_radius,
            nearby_raiders_within_hear_radius,
            nearby_raiders_within_highlight_radius,
        );
        self.resonation_ticks = 0;
        self.click_direction = Some(Self::direction_from_3d_data_value(event_param));
        self.ticks = 0;
        self.shaking = true;
        true
    }

    pub fn update_entities(
        &mut self,
        game_time: u64,
        nearby_living_within_hear_radius: usize,
        nearby_raiders_within_hear_radius: usize,
        nearby_raiders_within_highlight_radius: usize,
    ) {
        if game_time > self.last_ring_timestamp + Self::MIN_TICKS_BETWEEN_SEARCHES
            || self.last_ring_timestamp == 0
        {
            self.last_ring_timestamp = game_time;
            self.heard_bell_entities = nearby_living_within_hear_radius;
            self.nearby_raiders_within_hear_radius = nearby_raiders_within_hear_radius;
            self.nearby_raiders_within_highlight_radius = nearby_raiders_within_highlight_radius;
        }
    }

    pub fn tick(&mut self) -> BellTickEffects {
        let mut effects = BellTickEffects {
            play_resonate_sound: false,
            glowing_raiders: 0,
        };

        if self.shaking {
            self.ticks += 1;
        }

        if self.ticks >= Self::DURATION {
            self.shaking = false;
            self.ticks = 0;
        }

        if self.ticks >= Self::TICKS_BEFORE_RESONATION
            && self.resonation_ticks == 0
            && self.nearby_raiders_within_hear_radius > 0
        {
            self.resonating = true;
            effects.play_resonate_sound = true;
        }

        if self.resonating {
            if self.resonation_ticks < Self::MAX_RESONATION_TICKS {
                self.resonation_ticks += 1;
            } else {
                effects.glowing_raiders = self.nearby_raiders_within_highlight_radius;
                self.resonating = false;
            }
        }

        effects
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![])
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}

impl BrushableBlockEntity {
    pub const BRUSH_COOLDOWN_TICKS: u64 = 10;
    pub const BRUSH_RESET_TICKS: u64 = 40;
    pub const REQUIRED_BRUSHES_TO_BREAK: i32 = 10;
    pub const RETRACTION_SPEED: i32 = 2;
    pub const RETRACTION_TICKS: u64 = 4;

    pub fn new() -> Self {
        Self {
            brush_count: 0,
            brush_count_resets_at_tick: 0,
            cooldown_ends_at_tick: 0,
            item: None,
            hit_direction: None,
            loot_table: None,
            loot_table_seed: 0,
        }
    }

    pub fn set_loot_table(&mut self, loot_table: impl Into<String>, seed: i64) {
        self.loot_table = Some(loot_table.into());
        self.loot_table_seed = seed;
        self.item = None;
    }

    pub fn unpack_loot_table(&mut self, generated_item: Option<PotItemStack>) -> bool {
        if self.loot_table.take().is_some() {
            self.loot_table_seed = 0;
            self.item = generated_item;
            true
        } else {
            false
        }
    }

    pub fn brush(
        &mut self,
        game_time: u64,
        direction: Direction,
        generated_loot_item: Option<PotItemStack>,
    ) -> BrushResult {
        if self.hit_direction.is_none() {
            self.hit_direction = Some(direction);
        }
        self.brush_count_resets_at_tick = game_time + Self::BRUSH_RESET_TICKS;
        if game_time < self.cooldown_ends_at_tick {
            return BrushResult::CoolingDown;
        }

        self.cooldown_ends_at_tick = game_time + Self::BRUSH_COOLDOWN_TICKS;
        self.unpack_loot_table(generated_loot_item);
        self.brush_count += 1;
        if self.brush_count >= Self::REQUIRED_BRUSHES_TO_BREAK {
            self.brush_count = Self::REQUIRED_BRUSHES_TO_BREAK;
            return BrushResult::Completed;
        }

        BrushResult::InProgress {
            dusted: self.completion_state(),
        }
    }

    pub fn check_reset(&mut self, game_time: u64) -> Option<i32> {
        if self.brush_count != 0 && game_time >= self.brush_count_resets_at_tick {
            let previous = self.completion_state();
            self.brush_count = (self.brush_count - Self::RETRACTION_SPEED).max(0);
            let current = self.completion_state();
            if self.brush_count == 0 {
                self.hit_direction = None;
                self.brush_count_resets_at_tick = 0;
                self.cooldown_ends_at_tick = 0;
            } else {
                self.brush_count_resets_at_tick = game_time + Self::RETRACTION_TICKS;
            }
            return (previous != current).then_some(current);
        }

        None
    }

    pub fn completion_state(&self) -> i32 {
        if self.brush_count == 0 {
            0
        } else if self.brush_count < 3 {
            1
        } else if self.brush_count < 6 {
            2
        } else {
            3
        }
    }

    pub fn drop_content(&mut self) -> Option<(PotItemStack, Direction)> {
        let item = self.item.take()?;
        Some((item, self.hit_direction.unwrap_or(Direction::Up)))
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            if self.loot_table_seed != 0 {
                fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
            }
        } else if let Some(item) = &self.item {
            fields.push(("item".to_string(), item.to_tag()));
        }
        if let Some(direction) = self.hit_direction {
            fields.push((
                "hit_direction".to_string(),
                Tag::String(direction_name(direction).to_string()),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut brushable = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return brushable;
        };
        brushable.loot_table = get_string(entries, "LootTable").map(ToString::to_string);
        brushable.loot_table_seed = entries
            .iter()
            .find(|(name, _)| name == "LootTableSeed")
            .map(|(_, tag)| tag_long_or_zero(tag))
            .unwrap_or(0);
        if brushable.loot_table.is_none() {
            brushable.item = entries
                .iter()
                .find(|(name, _)| name == "item")
                .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        }
        brushable.hit_direction =
            get_string(entries, "hit_direction").and_then(direction_from_name);
        brushable
    }

    pub fn get_update_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(direction) = self.hit_direction {
            fields.push((
                "hit_direction".to_string(),
                Tag::String(direction_name(direction).to_string()),
            ));
        }
        if let Some(item) = &self.item {
            fields.push(("item".to_string(), item.to_tag()));
        }
        Tag::Compound(fields)
    }
}

pub fn type_info(ty: BlockEntityTypeId) -> &'static BlockEntityTypeInfo {
    if let Some(info) = BLOCK_ENTITY_TYPES.iter().find(|info| info.id == ty) {
        return info;
    }

    // BLOCK_ENTITY_TYPES is the canonical table for every BlockEntityTypeId variant.
    unreachable!("block entity type table does not cover {ty:?}");
}

pub fn type_by_key(key: &str) -> Option<BlockEntityTypeId> {
    BLOCK_ENTITY_TYPES
        .iter()
        .find(|info| info.key == key || format!("minecraft:{}", info.key) == key)
        .map(|info| info.id)
}

pub fn is_valid_block_state(ty: BlockEntityTypeId, block_state: &str) -> bool {
    type_info(ty).valid_blocks.contains(&block_state)
}

pub fn only_op_can_set_nbt(ty: BlockEntityTypeId) -> bool {
    type_info(ty).op_only_custom_data
}

pub fn has_block_entity_for_block(registry_id: &str) -> bool {
    BLOCK_ENTITY_TYPES
        .iter()
        .any(|entry| entry.valid_blocks.contains(&registry_id))
}
