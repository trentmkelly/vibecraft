use super::*;

impl ConduitBlockEntity {
    pub const BLOCK_REFRESH_RATE: i64 = 40;
    pub const EFFECT_DURATION_TICKS: i32 = 260;
    pub const MIN_ACTIVE_SIZE: usize = 16;
    pub const MIN_KILL_SIZE: usize = 42;
    pub const KILL_RANGE: f64 = 8.0;
    pub const ROTATION_SPEED: f32 = -0.0375;

    pub fn new() -> Self {
        Self {
            tick_count: 0,
            active_rotation: 0,
            is_active: false,
            is_hunting: false,
            effect_blocks: Vec::new(),
            destroy_target: None,
            next_ambient_sound_activation: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(target) = &self.destroy_target {
            fields.push(("Target".to_string(), Tag::String(target.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut conduit = Self::new();
        if let Some(entries) = compound_entries(tag) {
            conduit.destroy_target = get_string(entries, "Target").map(ToString::to_string);
        }
        conduit
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn active_rotation(&self, partial_tick: f32) -> f32 {
        (self.active_rotation as f32 + partial_tick) * Self::ROTATION_SPEED
    }

    pub fn effect_range(effect_block_count: usize) -> i32 {
        (effect_block_count / 7) as i32 * 16
    }

    pub fn apply_effects(&self) -> Option<ConduitEffectApplication> {
        self.is_active.then(|| ConduitEffectApplication {
            range: Self::effect_range(self.effect_blocks.len()),
            duration_ticks: Self::EFFECT_DURATION_TICKS,
        })
    }

    pub fn server_tick(
        &mut self,
        game_time: i64,
        pos: BlockPos,
        is_water_at: impl Fn(BlockPos) -> bool,
        block_at: impl Fn(BlockPos) -> &'static str,
        targets: &[ConduitTarget],
    ) -> Option<String> {
        self.tick_count += 1;
        if game_time % Self::BLOCK_REFRESH_RATE != 0 {
            if self.is_active {
                self.active_rotation += 1;
            }
            return None;
        }

        self.is_active = self.update_shape(pos, is_water_at, block_at);
        self.is_hunting = self.effect_blocks.len() >= Self::MIN_KILL_SIZE;
        if !self.is_active {
            self.destroy_target = None;
            return None;
        }
        let target_changed = self.update_destroy_target(pos, targets);
        if self.is_active {
            self.active_rotation += 1;
        }
        if self.destroy_target.is_some() && self.is_hunting && target_changed {
            self.destroy_target.clone()
        } else {
            None
        }
    }

    pub fn update_shape(
        &mut self,
        pos: BlockPos,
        is_water_at: impl Fn(BlockPos) -> bool,
        block_at: impl Fn(BlockPos) -> &'static str,
    ) -> bool {
        self.effect_blocks.clear();
        for ox in -1..=1 {
            for oy in -1..=1 {
                for oz in -1..=1 {
                    if !is_water_at(offset_pos(pos, ox, oy, oz)) {
                        return false;
                    }
                }
            }
        }

        for ox in -2_i32..=2 {
            for oy in -2_i32..=2 {
                for oz in -2_i32..=2 {
                    let ax = ox.abs();
                    let ay = oy.abs();
                    let az = oz.abs();
                    let frame_position = (ax > 1 || ay > 1 || az > 1)
                        && ((ox == 0 && (ay == 2 || az == 2))
                            || (oy == 0 && (ax == 2 || az == 2))
                            || (oz == 0 && (ax == 2 || ay == 2)));
                    if frame_position {
                        let test_pos = offset_pos(pos, ox, oy, oz);
                        if Self::is_valid_frame_block(block_at(test_pos)) {
                            self.effect_blocks.push(test_pos);
                        }
                    }
                }
            }
        }
        self.effect_blocks.len() >= Self::MIN_ACTIVE_SIZE
    }

    pub fn update_destroy_target(&mut self, pos: BlockPos, targets: &[ConduitTarget]) -> bool {
        if !self.is_hunting {
            let changed = self.destroy_target.is_some();
            self.destroy_target = None;
            return changed;
        }
        if let Some(current) = self.destroy_target.as_ref() {
            if targets.iter().any(|target| {
                target.id == *current
                    && target.alive
                    && target.enemy
                    && target.in_water_or_rain
                    && closer_than(pos, target.pos, Self::KILL_RANGE)
            }) {
                return false;
            }
        }
        let next = targets
            .iter()
            .find(|target| {
                target.alive
                    && target.enemy
                    && target.in_water_or_rain
                    && closer_than(pos, target.pos, Self::KILL_RANGE)
            })
            .map(|target| target.id.clone());
        let changed = self.destroy_target != next;
        self.destroy_target = next;
        changed
    }

    pub fn is_valid_frame_block(block: &str) -> bool {
        matches!(
            block,
            "minecraft:prismarine"
                | "minecraft:prismarine_bricks"
                | "minecraft:sea_lantern"
                | "minecraft:dark_prismarine"
        )
    }
}

impl CampfireBlockEntity {
    pub const NUM_SLOTS: usize = 4;
    pub const DEFAULT_COOKING_TIME: i32 = 600;
    pub const BURN_COOL_SPEED: i32 = 2;

    pub fn new(signal_fire: bool) -> Self {
        Self {
            items: vec![None; Self::NUM_SLOTS],
            cooking_progress: [0; Self::NUM_SLOTS],
            cooking_time: [0; Self::NUM_SLOTS],
            signal_fire,
        }
    }

    pub fn place_food(&mut self, item: PotItemStack, cooking_time: Option<i32>) -> bool {
        if item.is_empty() {
            return false;
        }
        let Some(slot) = self.items.iter().position(Option::is_none) else {
            return false;
        };
        self.items[slot] = Some(PotItemStack {
            item_id: item.item_id,
            count: 1,
        });
        self.cooking_progress[slot] = 0;
        self.cooking_time[slot] = cooking_time.unwrap_or(Self::DEFAULT_COOKING_TIME).max(1);
        true
    }

    pub fn cook_tick(
        &mut self,
        lit: bool,
        recipe_result: impl Fn(&PotItemStack) -> PotItemStack,
    ) -> Vec<CampfireTickResult> {
        if !lit {
            return self.cooldown_tick();
        }

        let mut results = Vec::new();
        for slot in 0..Self::NUM_SLOTS {
            if let Some(item) = self.items[slot].as_ref() {
                self.cooking_progress[slot] += 1;
                if self.cooking_progress[slot] >= self.cooking_time[slot] {
                    let cooked = recipe_result(item);
                    self.items[slot] = None;
                    self.cooking_progress[slot] = 0;
                    self.cooking_time[slot] = 0;
                    results.push(CampfireTickResult::Cooked { slot, item: cooked });
                } else {
                    results.push(CampfireTickResult::Changed);
                }
            }
        }
        if results.is_empty() {
            results.push(CampfireTickResult::NoChange);
        }
        results
    }

    pub fn cooldown_tick(&mut self) -> Vec<CampfireTickResult> {
        let mut changed = false;
        for slot in 0..Self::NUM_SLOTS {
            if self.cooking_progress[slot] > 0 {
                changed = true;
                self.cooking_progress[slot] = (self.cooking_progress[slot] - Self::BURN_COOL_SPEED)
                    .clamp(0, self.cooking_time[slot]);
            }
        }
        vec![if changed {
            CampfireTickResult::Changed
        } else {
            CampfireTickResult::NoChange
        }]
    }

    pub fn clear_content(&mut self) {
        self.items.fill(None);
    }

    pub fn get_update_tag(&self) -> Tag {
        Tag::Compound(vec![("Items".to_string(), self.items_tag())])
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("Items".to_string(), self.items_tag()),
            (
                "CookingTimes".to_string(),
                Tag::IntArray(self.cooking_progress.to_vec()),
            ),
            (
                "CookingTotalTimes".to_string(),
                Tag::IntArray(self.cooking_time.to_vec()),
            ),
            (
                "SignalFire".to_string(),
                Tag::Byte(i8::from(self.signal_fire)),
            ),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(false);
        };
        let mut campfire = Self::new(get_bool(entries, "SignalFire").unwrap_or(false));
        if let Some(Tag::List(items)) = entries
            .iter()
            .find_map(|(name, tag)| (name == "Items").then_some(tag))
        {
            for item_tag in items {
                let Some(item_entries) = compound_entries(item_tag) else {
                    continue;
                };
                let Some(slot) =
                    get_byte(item_entries, "Slot").and_then(|slot| usize::try_from(slot).ok())
                else {
                    continue;
                };
                if slot < Self::NUM_SLOTS {
                    campfire.items[slot] = PotItemStack::from_tag(item_tag);
                }
            }
        }
        if let Some(values) = get_int_array(entries, "CookingTimes") {
            for (slot, value) in values.iter().copied().take(Self::NUM_SLOTS).enumerate() {
                campfire.cooking_progress[slot] = value;
            }
        }
        if let Some(values) = get_int_array(entries, "CookingTotalTimes") {
            for (slot, value) in values.iter().copied().take(Self::NUM_SLOTS).enumerate() {
                campfire.cooking_time[slot] = value;
            }
        }
        campfire
    }

    pub(super) fn items_tag(&self) -> Tag {
        Tag::List(
            self.items
                .iter()
                .enumerate()
                .filter_map(|(slot, item)| {
                    let mut tag = item.as_ref()?.to_tag();
                    if let Tag::Compound(entries) = &mut tag {
                        entries.insert(0, ("Slot".to_string(), Tag::Byte(slot as i8)));
                    }
                    Some(tag)
                })
                .collect(),
        )
    }
}

impl SculkSensorPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listening => "listening",
            Self::Ticking => "ticking",
            Self::VibrationDone => "vibration_done",
            Self::Cooldown => "cooldown",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "listening" => Some(Self::Listening),
            "ticking" => Some(Self::Ticking),
            "vibration_done" => Some(Self::VibrationDone),
            "cooldown" => Some(Self::Cooldown),
            _ => None,
        }
    }
}

impl SculkSensorBlockEntity {
    pub const DEFAULT_LAST_VIBRATION_FREQUENCY: u8 = 0;
    pub const LISTENER_RADIUS: i32 = 8;
    pub const ACTIVE_TICKS: i32 = 30;
    pub const COOLDOWN_TICKS: i32 = 10;

    pub fn new() -> Self {
        Self {
            vibration_data: VibrationData::new(),
            last_vibration_frequency: Self::DEFAULT_LAST_VIBRATION_FREQUENCY,
            phase: SculkSensorPhase::Listening,
            listener_radius: Self::LISTENER_RADIUS,
            power: 0,
            active_ticks: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "last_vibration_frequency".to_string(),
                Tag::Int(i32::from(self.last_vibration_frequency)),
            ),
            ("listener".to_string(), self.listener_tag()),
            (
                "phase".to_string(),
                Tag::String(self.phase.as_str().to_string()),
            ),
            ("power".to_string(), Tag::Byte(self.power as i8)),
            ("active_ticks".to_string(), Tag::Int(self.active_ticks)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let mut sensor = Self::new();
        sensor.last_vibration_frequency = get_int(entries, "last_vibration_frequency")
            .unwrap_or(0)
            .clamp(0, 15) as u8;
        sensor.phase = get_string(entries, "phase")
            .and_then(SculkSensorPhase::from_str)
            .unwrap_or(SculkSensorPhase::Listening);
        sensor.power = get_byte(entries, "power").unwrap_or(0).clamp(0, 15) as u8;
        sensor.active_ticks = get_int(entries, "active_ticks").unwrap_or(0).max(0);
        if let Some(listener_tag) = entries.iter().find(|(name, _)| name == "listener") {
            sensor.vibration_data = vibration_data_from_tag(&listener_tag.1);
        }
        sensor
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn can_receive_vibration(&self, event_id: &str, sensor_can_activate: bool) -> bool {
        sensor_can_activate
            && self.phase == SculkSensorPhase::Listening
            && vibration_frequency(event_id) != NO_VIBRATION_FREQUENCY
    }

    pub fn queue_vibration(&mut self, vibration: VibrationInfo, game_time: i64) -> bool {
        if !self.can_receive_vibration(vibration.event.id, true) {
            return false;
        }
        self.vibration_data
            .selector
            .add_candidate(vibration, game_time);
        true
    }

    pub fn receive_vibration(
        &mut self,
        event_id: &str,
        distance: f32,
    ) -> Option<SculkSensorTickResult> {
        if !self.can_receive_vibration(event_id, true) {
            return None;
        }
        let frequency = vibration_frequency(event_id);
        let redstone = redstone_strength_for_distance(distance, self.listener_radius);
        self.last_vibration_frequency = frequency;
        self.power = redstone;
        self.phase = SculkSensorPhase::VibrationDone;
        self.active_ticks = Self::ACTIVE_TICKS;
        Some(SculkSensorTickResult::Activate {
            frequency,
            redstone,
        })
    }

    pub fn tick(&mut self, game_time: i64) -> SculkSensorTickResult {
        match self.phase {
            SculkSensorPhase::Listening | SculkSensorPhase::Ticking => {
                match tick_vibration(&mut self.vibration_data, game_time) {
                    VibrationTickAction::Selected {
                        travel_time_in_ticks,
                        ..
                    } => {
                        self.phase = SculkSensorPhase::Ticking;
                        SculkSensorTickResult::Particle {
                            travel_time_in_ticks,
                        }
                    }
                    VibrationTickAction::ReloadParticle {
                        travel_time_in_ticks,
                    } => SculkSensorTickResult::Particle {
                        travel_time_in_ticks,
                    },
                    VibrationTickAction::Received {
                        event_id,
                        frequency,
                    } => {
                        let distance = 0.0;
                        let redstone =
                            redstone_strength_for_distance(distance, self.listener_radius);
                        self.last_vibration_frequency = frequency;
                        self.power = redstone;
                        self.phase = SculkSensorPhase::VibrationDone;
                        self.active_ticks = Self::ACTIVE_TICKS;
                        let _ = event_id;
                        SculkSensorTickResult::Activate {
                            frequency,
                            redstone,
                        }
                    }
                    VibrationTickAction::None => SculkSensorTickResult::None,
                }
            }
            SculkSensorPhase::VibrationDone => {
                self.active_ticks = self.active_ticks.saturating_sub(1);
                if self.active_ticks == 0 {
                    self.phase = SculkSensorPhase::Cooldown;
                    self.active_ticks = Self::COOLDOWN_TICKS;
                    SculkSensorTickResult::Cooldown
                } else {
                    SculkSensorTickResult::None
                }
            }
            SculkSensorPhase::Cooldown => {
                self.active_ticks = self.active_ticks.saturating_sub(1);
                if self.active_ticks == 0 {
                    self.phase = SculkSensorPhase::Listening;
                    self.power = 0;
                    SculkSensorTickResult::Deactivate
                } else {
                    SculkSensorTickResult::None
                }
            }
        }
    }

    pub(super) fn listener_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "travel_time_in_ticks".to_string(),
                Tag::Int(self.vibration_data.travel_time_in_ticks),
            ),
            (
                "reload_vibration_particle".to_string(),
                Tag::Byte(i8::from(self.vibration_data.reload_vibration_particle)),
            ),
        ];
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

impl CalibratedSculkSensorBlockEntity {
    pub const LISTENER_RADIUS: i32 = 16;

    pub fn new(back_signal: u8) -> Self {
        let mut sensor = SculkSensorBlockEntity::new();
        sensor.listener_radius = Self::LISTENER_RADIUS;
        Self {
            sensor,
            back_signal: back_signal.min(15),
        }
    }

    pub fn set_back_signal(&mut self, back_signal: u8) {
        self.back_signal = back_signal.min(15);
    }

    pub fn can_receive_vibration(&self, event_id: &str, sensor_can_activate: bool) -> bool {
        if !self
            .sensor
            .can_receive_vibration(event_id, sensor_can_activate)
        {
            return false;
        }
        let frequency = vibration_frequency(event_id);
        self.back_signal == 0 || self.back_signal == frequency
    }

    pub fn receive_vibration(
        &mut self,
        event_id: &str,
        distance: f32,
    ) -> Option<SculkSensorTickResult> {
        match calibrated_sculk_sensor_receive(
            self.back_signal,
            event_id,
            distance,
            Self::LISTENER_RADIUS,
        ) {
            SculkSensorAction::Activate {
                frequency,
                redstone,
            } if self.sensor.can_receive_vibration(event_id, true) => {
                self.sensor.last_vibration_frequency = frequency;
                self.sensor.power = redstone;
                self.sensor.phase = SculkSensorPhase::VibrationDone;
                self.sensor.active_ticks = SculkSensorBlockEntity::ACTIVE_TICKS;
                Some(SculkSensorTickResult::Activate {
                    frequency,
                    redstone,
                })
            }
            _ => None,
        }
    }

    pub fn tick(&mut self, game_time: i64) -> SculkSensorTickResult {
        self.sensor.tick(game_time)
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = match self.sensor.save_additional() {
            Tag::Compound(entries) => entries,
            _ => Vec::new(),
        };
        entries.push(("back_signal".to_string(), Tag::Byte(self.back_signal as i8)));
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let sensor = SculkSensorBlockEntity::load_additional(tag);
        let back_signal = compound_entries(tag)
            .and_then(|entries| get_byte(entries, "back_signal"))
            .unwrap_or(0)
            .clamp(0, 15) as u8;
        Self {
            sensor: SculkSensorBlockEntity {
                listener_radius: Self::LISTENER_RADIUS,
                ..sensor
            },
            back_signal,
        }
    }
}

impl SculkChargeCursor {
    pub const MAX_CHARGE: i32 = 1000;

    pub fn new(pos: BlockPos, charge: i32) -> Self {
        Self {
            pos,
            charge: charge.clamp(0, Self::MAX_CHARGE),
            decay_delay: 1,
            update_delay: 0,
            facings: Vec::new(),
        }
    }

    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("pos".to_string(), block_pos_to_tag(self.pos)),
            ("charge".to_string(), Tag::Int(self.charge)),
            ("decay_delay".to_string(), Tag::Int(self.decay_delay)),
            ("update_delay".to_string(), Tag::Int(self.update_delay)),
            (
                "facings".to_string(),
                Tag::List(
                    self.facings
                        .iter()
                        .map(|direction| Tag::String(direction_name(*direction).to_string()))
                        .collect(),
                ),
            ),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            pos: entries
                .iter()
                .find(|(name, _)| name == "pos")
                .and_then(|(_, tag)| block_pos_from_tag(tag))?,
            charge: get_int(entries, "charge")
                .unwrap_or(0)
                .clamp(0, Self::MAX_CHARGE),
            decay_delay: get_int(entries, "decay_delay").unwrap_or(1).clamp(0, 1),
            update_delay: get_int(entries, "update_delay").unwrap_or(0).max(0),
            facings: entries
                .iter()
                .find(|(name, _)| name == "facings")
                .and_then(|(_, tag)| match tag {
                    Tag::List(values) => Some(
                        values
                            .iter()
                            .filter_map(|tag| match tag {
                                Tag::String(name) => direction_from_name(name),
                                _ => None,
                            })
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_default(),
        })
    }
}

impl SculkCatalystBlockEntity {
    pub const LISTENER_RADIUS: i32 = 8;
    pub const DELIVERY_MODE: &'static str = "by_distance";
    pub const PULSE_TICKS: i32 = 8;
    pub const MAX_CURSORS: usize = 32;
    pub const MAX_CHARGE: i32 = 1000;
    pub const MAX_CURSOR_DISTANCE: i32 = 1024;

    pub fn new() -> Self {
        Self {
            cursors: Vec::new(),
            pulse_ticks: 0,
        }
    }

    pub fn add_cursors(&mut self, start_pos: BlockPos, mut charge: i32) {
        while charge > 0 && self.cursors.len() < Self::MAX_CURSORS {
            let current_charge = charge.min(Self::MAX_CHARGE);
            self.cursors
                .push(SculkChargeCursor::new(start_pos, current_charge));
            charge -= current_charge;
        }
    }

    pub fn handle_entity_die(
        &mut self,
        source_pos: BlockPos,
        experience_reward: i32,
        should_drop_experience: bool,
        experience_already_consumed: bool,
        last_hurt_by_player: bool,
    ) -> SculkCatalystEventResult {
        if experience_already_consumed {
            return SculkCatalystEventResult::Ignored;
        }
        let before = self.cursors.len();
        if should_drop_experience && experience_reward > 0 {
            self.add_cursors(offset_pos(source_pos, 0, 1, 0), experience_reward);
        }
        self.pulse_ticks = Self::PULSE_TICKS;
        SculkCatalystEventResult::Bloom {
            pulse_ticks: Self::PULSE_TICKS,
            consumed_experience: true,
            added_cursors: self.cursors.len() - before,
            award_it_spreads: last_hurt_by_player,
        }
    }

    pub fn tick(&mut self, origin: BlockPos) {
        self.pulse_ticks = self.pulse_ticks.saturating_sub(1);
        // TODO(Java parity): replace this cursor decay model with full
        // SculkSpreader::updateCursors once SculkBehaviour world mutation,
        // vein spreading, cursor movement, and level-event particles are ported.
        self.cursors.retain_mut(|cursor| {
            if chessboard_distance(cursor.pos, origin) > Self::MAX_CURSOR_DISTANCE {
                return false;
            }
            if cursor.update_delay > 0 {
                cursor.update_delay -= 1;
                return true;
            }
            if cursor.decay_delay > 0 {
                cursor.decay_delay -= 1;
            } else {
                cursor.charge = (cursor.charge - 1).max(0);
                cursor.decay_delay = 1;
            }
            cursor.charge > 0
        });
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![(
            "cursors".to_string(),
            Tag::List(self.cursors.iter().map(SculkChargeCursor::to_tag).collect()),
        )])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let cursors = entries
            .iter()
            .find(|(name, _)| name == "cursors")
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(SculkChargeCursor::from_tag)
                        .take(Self::MAX_CURSORS)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Self {
            cursors,
            pulse_ticks: 0,
        }
    }
}
