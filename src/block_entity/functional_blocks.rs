use super::*;

impl ComparatorBlockEntity {
    pub fn new(mode: ComparatorMode) -> Self {
        Self {
            mode,
            output_signal: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![(
            "OutputSignal".to_string(),
            Tag::Int(self.output_signal),
        )])
    }

    pub fn load_additional(mode: ComparatorMode, tag: &Tag) -> Self {
        let output_signal = compound_entries(tag)
            .and_then(|entries| get_int(entries, "OutputSignal"))
            .unwrap_or(0);
        Self {
            mode,
            output_signal,
        }
    }

    pub fn calculate_output(&self, rear_input: u8, side_input: u8) -> u8 {
        comparator_output(self.mode, rear_input, side_input)
    }

    pub fn update_output(&mut self, rear_input: u8, side_input: u8) -> bool {
        let next = self.calculate_output(rear_input, side_input);
        let next = i32::from(next);
        let changed = self.output_signal != next;
        self.output_signal = next;
        changed
    }
}

impl DaylightDetectorBlockEntity {
    pub const TICK_INTERVAL: u64 = 20;

    pub fn new(inverted: bool) -> Self {
        Self { inverted, power: 0 }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(Vec::new())
    }

    pub fn calculate_power(
        inverted: bool,
        effective_sky_brightness: i32,
        sun_angle_degrees: f32,
    ) -> u8 {
        let mut target = effective_sky_brightness;
        if inverted {
            target = i32::from(MAX_SIGNAL) - target;
        } else if target > 0 {
            let mut sun_angle = sun_angle_degrees.to_radians();
            let offset = if sun_angle < std::f32::consts::PI {
                0.0
            } else {
                std::f32::consts::TAU
            };
            sun_angle += (offset - sun_angle) * 0.2;
            target = ((target as f32) * sun_angle.cos()).round() as i32;
        }

        target.clamp(0, i32::from(MAX_SIGNAL)) as u8
    }

    pub fn update_signal(&mut self, effective_sky_brightness: i32, sun_angle_degrees: f32) -> bool {
        let next =
            Self::calculate_power(self.inverted, effective_sky_brightness, sun_angle_degrees);
        let changed = self.power != next;
        self.power = next;
        changed
    }

    pub fn tick(
        &mut self,
        game_time: u64,
        effective_sky_brightness: i32,
        sun_angle_degrees: f32,
    ) -> bool {
        if !game_time.is_multiple_of(Self::TICK_INTERVAL) {
            return false;
        }
        self.update_signal(effective_sky_brightness, sun_angle_degrees)
    }
}

impl CommandBlockEntity {
    pub const NO_LAST_EXECUTION: i64 = -1;
    pub const PERMISSION_LEVEL: &'static str = "gamemaster";
    pub const SEARGE_COMMAND: &'static str = "Searge";
    pub const SEARGE_OUTPUT: &'static str = "#itzlipofutzli";

    pub fn new(mode: CommandBlockMode, conditional: bool) -> Self {
        Self {
            command: String::new(),
            success_count: 0,
            custom_name: None,
            track_output: true,
            last_output: None,
            update_last_execution: true,
            last_execution: Self::NO_LAST_EXECUTION,
            powered: false,
            automatic: false,
            condition_met: false,
            mode,
            conditional,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![
            ("Command".to_string(), Tag::String(self.command.clone())),
            ("SuccessCount".to_string(), Tag::Int(self.success_count)),
            (
                "TrackOutput".to_string(),
                Tag::Byte(self.track_output as i8),
            ),
            (
                "UpdateLastExecution".to_string(),
                Tag::Byte(self.update_last_execution as i8),
            ),
            ("powered".to_string(), Tag::Byte(self.powered as i8)),
            (
                "conditionMet".to_string(),
                Tag::Byte(self.condition_met as i8),
            ),
            ("auto".to_string(), Tag::Byte(self.automatic as i8)),
        ];

        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        if self.track_output {
            if let Some(last_output) = &self.last_output {
                entries.push(("LastOutput".to_string(), Tag::String(last_output.clone())));
            }
        }
        if self.update_last_execution && self.last_execution != Self::NO_LAST_EXECUTION {
            entries.push(("LastExecution".to_string(), Tag::Long(self.last_execution)));
        }

        Tag::Compound(entries)
    }

    pub fn load_additional(mode: CommandBlockMode, conditional: bool, tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(mode, conditional);
        };
        let track_output = get_bool(entries, "TrackOutput").unwrap_or(true);
        let update_last_execution = get_bool(entries, "UpdateLastExecution").unwrap_or(true);
        Self {
            command: get_string(entries, "Command").unwrap_or("").to_string(),
            success_count: get_int(entries, "SuccessCount").unwrap_or(0),
            custom_name: get_string(entries, "CustomName").map(ToString::to_string),
            track_output,
            last_output: if track_output {
                get_string(entries, "LastOutput").map(ToString::to_string)
            } else {
                None
            },
            update_last_execution,
            last_execution: if update_last_execution {
                get_long(entries, "LastExecution").unwrap_or(Self::NO_LAST_EXECUTION)
            } else {
                Self::NO_LAST_EXECUTION
            },
            powered: get_bool(entries, "powered").unwrap_or(false),
            automatic: get_bool(entries, "auto").unwrap_or(false),
            condition_met: get_bool(entries, "conditionMet").unwrap_or(false),
            mode,
            conditional,
        }
    }

    pub fn set_command(&mut self, command: impl Into<String>) {
        self.command = command.into();
        self.success_count = 0;
    }

    pub fn set_automatic(&mut self, automatic: bool, has_level: bool) -> bool {
        let previous = self.automatic;
        self.automatic = automatic;
        !previous
            && automatic
            && !self.powered
            && has_level
            && self.mode != CommandBlockMode::Sequence
    }

    pub fn mark_condition_met(&mut self, previous_command_success: bool) -> bool {
        self.condition_met = !self.conditional || previous_command_success;
        self.condition_met
    }

    pub fn can_use(&self, player_can_use_gamemaster_blocks: bool) -> bool {
        player_can_use_gamemaster_blocks
    }

    pub fn open_editor_packet(
        &self,
        pos: BlockPos,
        player_can_use_gamemaster_blocks: bool,
    ) -> Option<ClientboundBlockEntityDataPacket> {
        self.can_use(player_can_use_gamemaster_blocks)
            .then(|| ClientboundBlockEntityDataPacket {
                pos,
                ty: BlockEntityTypeId::CommandBlock,
                tag: self.save_additional(),
            })
    }

    pub fn apply_client_update(
        &mut self,
        update: CommandBlockUpdate,
        player_can_use_gamemaster_blocks: bool,
        has_level: bool,
    ) -> bool {
        if !self.can_use(player_can_use_gamemaster_blocks) {
            return false;
        }

        self.mode = update.mode;
        self.conditional = update.conditional;
        self.track_output = update.track_output;
        if !self.track_output {
            self.last_output = None;
        }
        self.set_automatic(update.automatic, has_level);
        self.set_command(update.command);
        true
    }

    pub fn command_source_stack(
        pos: BlockPos,
        level: impl Into<String>,
    ) -> CommandSourceStackModel {
        CommandSourceStackModel::new("CommandBlockEntity", level, 2).with_position(Vec3 {
            x: f64::from(pos.x) + 0.5,
            y: f64::from(pos.y) + 0.5,
            z: f64::from(pos.z) + 0.5,
        })
    }

    pub fn execution_action(
        &self,
        has_permission: bool,
        previous_success: bool,
    ) -> SpecialBlockAction {
        command_block_tick(
            &CommandBlockState {
                command: self.command.clone(),
                mode: self.mode.clone(),
                powered: self.powered,
                previously_powered: false,
                conditional: self.conditional,
                previous_success,
            },
            has_permission,
        )
    }

    pub fn perform_command(
        &mut self,
        game_time: i64,
        command_blocks_enabled: bool,
        has_permission: bool,
        previous_success: bool,
    ) -> bool {
        if self.update_last_execution && self.last_execution == game_time {
            return false;
        }
        if self.command.eq_ignore_ascii_case(Self::SEARGE_COMMAND) {
            self.last_output = Some(Self::SEARGE_OUTPUT.to_string());
            self.success_count = 1;
            return true;
        }

        self.success_count = 0;
        let executed = match self
            .execution_action(has_permission && command_blocks_enabled, previous_success)
        {
            SpecialBlockAction::ExecuteCommand { success_count } => {
                self.success_count = success_count;
                true
            }
            SpecialBlockAction::Noop => false,
            _ => false,
        };

        if executed {
            if self.update_last_execution {
                self.last_execution = game_time;
            } else {
                self.last_execution = Self::NO_LAST_EXECUTION;
            }
        }
        executed
    }

    pub fn execute_from_context(
        &mut self,
        context: CommandBlockExecutionContext,
        output: Option<String>,
    ) -> Option<CommandBlockExecution> {
        let source = Self::command_source_stack(context.pos, context.level);
        if !self.perform_command(
            context.game_time,
            context.command_blocks_enabled,
            context.has_permission,
            context.previous_success,
        ) {
            return None;
        }

        if self.track_output {
            if let Some(output) = output.clone() {
                self.last_output = Some(output);
            }
        } else {
            self.last_output = None;
        }

        let output = output.or_else(|| self.last_output.clone());
        Some(CommandBlockExecution {
            command: self.command.clone(),
            source,
            success_count: self.success_count,
            output,
        })
    }
}

pub fn execute_command_block_chain(
    entries: &mut [CommandBlockChainEntry],
    start_pos: BlockPos,
    context: CommandBlockExecutionContext,
) -> Vec<CommandBlockChainStep> {
    let mut steps = Vec::new();
    let mut current_pos = start_pos;
    let mut previous_success = context.previous_success;

    for _ in 0..entries.len() {
        let Some(index) = entries.iter().position(|entry| entry.pos == current_pos) else {
            break;
        };
        let next_pos = entries[index].pos.relative(entries[index].facing);
        let entry = &mut entries[index];
        let mut step_context = context.clone();
        step_context.pos = entry.pos;
        step_context.previous_success = previous_success;

        let success_count = if entry
            .block
            .execute_from_context(step_context, None)
            .is_some()
        {
            entry.block.success_count
        } else {
            0
        };
        previous_success = success_count > 0;
        steps.push(CommandBlockChainStep {
            pos: entry.pos,
            command: entry.block.command.clone(),
            success_count,
        });

        current_pos = next_pos;
        if !entries
            .iter()
            .any(|entry| entry.pos == current_pos && entry.block.mode == CommandBlockMode::Sequence)
        {
            break;
        }
    }

    steps
}

impl JukeboxBlockEntity {
    pub const RECORD_ITEM_TAG_ID: &'static str = "RecordItem";
    pub const TICKS_SINCE_SONG_STARTED_TAG_ID: &'static str = "ticks_since_song_started";
    pub const STOP_LEVEL_EVENT: i32 = 1011;
    pub const MAX_STACK_SIZE: i32 = 1;
    pub const STOP_GAME_EVENT: &'static str = "minecraft:jukebox_stop_play";
    pub const PLAY_GAME_EVENT: &'static str = "minecraft:jukebox_play";

    pub fn new() -> Self {
        Self {
            item: None,
            is_playing: false,
            ticks_since_song_started: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(item) = self.item.as_ref().filter(|item| !item.is_empty()) {
            entries.push((Self::RECORD_ITEM_TAG_ID.to_string(), item.to_tag()));
        }
        if self.song_item().is_some() {
            entries.push((
                Self::TICKS_SINCE_SONG_STARTED_TAG_ID.to_string(),
                Tag::Long(self.ticks_since_song_started),
            ));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let item = entries
            .iter()
            .find(|(key, _)| key == Self::RECORD_ITEM_TAG_ID)
            .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        let mut jukebox = Self {
            item,
            is_playing: false,
            ticks_since_song_started: 0,
        };
        if jukebox.song_item().is_some() {
            jukebox.ticks_since_song_started =
                get_long(entries, Self::TICKS_SINCE_SONG_STARTED_TAG_ID).unwrap_or(0);
        }
        jukebox
    }

    pub fn set_the_item(&mut self, item: Option<PotItemStack>) -> JukeboxSongEvent {
        self.item = item.filter(|item| !item.is_empty());
        self.ticks_since_song_started = 0;
        if self.song_item().is_some() {
            self.is_playing = true;
            JukeboxSongEvent::Started
        } else {
            self.is_playing = false;
            JukeboxSongEvent::Stopped
        }
    }

    pub fn set_song_item_without_playing(&mut self, item: PotItemStack) -> JukeboxSongEvent {
        self.item = (!item.is_empty()).then_some(item);
        self.is_playing = false;
        self.ticks_since_song_started = 0;
        JukeboxSongEvent::ItemChanged
    }

    pub fn remove_the_item(&mut self) -> Option<PotItemStack> {
        self.is_playing = false;
        self.ticks_since_song_started = 0;
        self.item.take()
    }

    pub fn pop_out_the_item(&mut self) -> Option<PotItemStack> {
        self.remove_the_item()
    }

    pub fn tick(&mut self) -> bool {
        if self.is_playing {
            self.ticks_since_song_started += 1;
            true
        } else {
            false
        }
    }

    pub fn redstone_signal(&self) -> u8 {
        if self.is_playing {
            MAX_SIGNAL
        } else {
            0
        }
    }

    pub fn comparator_output(&self) -> u8 {
        self.song_item()
            .map(jukebox_song_comparator_output)
            .unwrap_or(0)
    }

    pub fn can_place_item(&self, item: &PotItemStack) -> bool {
        self.item.is_none() && jukebox_song_item_id(&item.item_id).is_some()
    }

    pub fn can_take_item(&self, destination_has_empty_slot: bool) -> bool {
        destination_has_empty_slot
    }

    pub const fn max_stack_size(&self) -> i32 {
        Self::MAX_STACK_SIZE
    }

    pub fn set_removed(&mut self) -> JukeboxRemovalEffect {
        let popped_item = self.pop_out_the_item();
        JukeboxRemovalEffect {
            popped_item,
            game_event: Self::STOP_GAME_EVENT,
            level_event: Self::STOP_LEVEL_EVENT,
        }
    }

    pub fn pre_remove_side_effects(&mut self) -> Option<PotItemStack> {
        self.pop_out_the_item()
    }

    pub(super) fn song_item(&self) -> Option<&str> {
        self.item
            .as_ref()
            .and_then(|item| jukebox_song_item_id(&item.item_id))
    }
}

pub(super) fn jukebox_song_item_id(item_id: &str) -> Option<&str> {
    let song_id = item_id.strip_prefix("minecraft:music_disc_")?;
    (jukebox_song_comparator_output(song_id) > 0).then_some(song_id)
}

pub(super) fn jukebox_song_comparator_output(song_id: &str) -> u8 {
    match song_id {
        "13" => 1,
        "cat" => 2,
        "blocks" => 3,
        "chirp" => 4,
        "far" => 5,
        "mall" => 6,
        "mellohi" => 7,
        "stal" => 8,
        "strad" | "lava_chicken" => 9,
        "ward" | "tears" => 10,
        "11" | "creator_music_box" => 11,
        "wait" | "creator" => 12,
        "pigstep" | "precipice" => 13,
        "otherside" | "relic" => 14,
        "5" => 15,
        _ => 0,
    }
}

impl EnchantingTableBlockEntity {
    pub const DEFAULT_NAME: &'static str = "container.enchant";

    pub fn new() -> Self {
        Self {
            custom_name: None,
            time: 0,
            flip: 0.0,
            o_flip: 0.0,
            flip_t: 0.0,
            flip_a: 0.0,
            open: 0.0,
            o_open: 0.0,
            rot: 0.0,
            o_rot: 0.0,
            t_rot: 0.0,
        }
    }

    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(Self::DEFAULT_NAME)
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "enchantment",
            initial_slots: vec![None, None],
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut table = Self::new();
        if let Some(entries) = compound_entries(tag) {
            table.custom_name = get_string(entries, "CustomName").map(ToString::to_string);
        }
        table
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn collect_implicit_components(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(custom_name) = &self.custom_name {
            entries.push((
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            ));
        }
        Tag::Compound(entries)
    }

    pub fn apply_implicit_components(&mut self, components: &Tag) {
        let Some(entries) = compound_entries(components) else {
            return;
        };
        self.custom_name = get_string(entries, "minecraft:custom_name").map(ToString::to_string);
    }

    pub fn remove_components_from_tag(tag: &Tag) -> Tag {
        let Some(entries) = compound_entries(tag) else {
            return tag.clone();
        };
        Tag::Compound(
            entries
                .iter()
                .filter(|(name, _)| name != "CustomName")
                .cloned()
                .collect(),
        )
    }

    pub fn bookshelf_offsets() -> Vec<BlockPos> {
        let mut offsets = Vec::new();
        for x in -2i32..=2 {
            for y in 0i32..=1 {
                for z in -2i32..=2 {
                    if x.abs() == 2 || z.abs() == 2 {
                        offsets.push(BlockPos { x, y, z });
                    }
                }
            }
        }
        offsets
    }

    pub fn count_valid_bookshelves(
        is_power_provider: impl Fn(BlockPos) -> bool,
        is_power_transmitter: impl Fn(BlockPos) -> bool,
    ) -> usize {
        Self::bookshelf_offsets()
            .into_iter()
            .filter(|offset| {
                is_power_provider(*offset)
                    && is_power_transmitter(BlockPos {
                        x: offset.x / 2,
                        y: offset.y,
                        z: offset.z / 2,
                    })
            })
            .take(15)
            .count()
    }

    pub fn book_animation_tick(
        &mut self,
        player_offset_xz: Option<(f64, f64)>,
        next_flip_delta: Option<f32>,
        force_page_turn: bool,
    ) {
        self.o_open = self.open;
        self.o_rot = self.rot;
        if let Some((xd, zd)) = player_offset_xz {
            self.t_rot = (zd.atan2(xd)) as f32;
            self.open += 0.1;
            if self.open < 0.5 || force_page_turn {
                if let Some(delta) = next_flip_delta {
                    let old = self.flip_t;
                    if delta != 0.0 {
                        self.flip_t += delta;
                    } else {
                        self.flip_t += 1.0;
                    }
                    if self.flip_t == old {
                        self.flip_t += 1.0;
                    }
                }
            }
        } else {
            self.t_rot += 0.02;
            self.open -= 0.1;
        }

        self.rot = wrap_radians(self.rot);
        self.t_rot = wrap_radians(self.t_rot);
        let rot_dir = wrap_radians(self.t_rot - self.rot);
        self.rot += rot_dir * 0.4;
        self.open = self.open.clamp(0.0, 1.0);
        self.time += 1;
        self.o_flip = self.flip;
        let diff = ((self.flip_t - self.flip) * 0.4).clamp(-0.2, 0.2);
        self.flip_a += (diff - self.flip_a) * 0.9;
        self.flip += self.flip_a;
    }
}

pub(super) fn wrap_radians(mut value: f32) -> f32 {
    while value >= std::f32::consts::PI {
        value -= std::f32::consts::TAU;
    }
    while value < -std::f32::consts::PI {
        value += std::f32::consts::TAU;
    }
    value
}

impl ShelfBlockEntity {
    pub const MAX_ITEMS: usize = 3;
    pub const ALIGN_ITEMS_TO_BOTTOM_TAG: &'static str = "align_items_to_bottom";

    pub fn new() -> Self {
        Self {
            items: vec![None; Self::MAX_ITEMS],
            align_items_to_bottom: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![(
            "Items".to_string(),
            Tag::List(
                self.items
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, item)| {
                        let item = item.as_ref().filter(|item| !item.is_empty())?;
                        let mut tag = match item.to_tag() {
                            Tag::Compound(entries) => entries,
                            _ => return None,
                        };
                        tag.push(("Slot".to_string(), Tag::Byte(slot as i8)));
                        Some(Tag::Compound(tag))
                    })
                    .collect(),
            ),
        )];
        entries.push((
            Self::ALIGN_ITEMS_TO_BOTTOM_TAG.to_string(),
            Tag::Byte(self.align_items_to_bottom as i8),
        ));
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut shelf = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return shelf;
        };
        shelf.align_items_to_bottom =
            get_bool(entries, Self::ALIGN_ITEMS_TO_BOTTOM_TAG).unwrap_or(false);
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(key, _)| key == "Items")
            .map(|(_, tag)| tag)
        {
            for item_tag in items {
                let Some(item_entries) = compound_entries(item_tag) else {
                    continue;
                };
                let Some(slot) = get_byte(item_entries, "Slot") else {
                    continue;
                };
                if let Some(slot) = usize::try_from(slot)
                    .ok()
                    .filter(|slot| *slot < Self::MAX_ITEMS)
                {
                    shelf.items[slot] = PotItemStack::from_tag(item_tag);
                }
            }
        }
        shelf
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn collect_implicit_components(&self) -> Tag {
        Tag::Compound(vec![(
            "minecraft:container".to_string(),
            container_items_tag(&self.items),
        )])
    }

    pub fn apply_implicit_components(&mut self, components: &Tag) {
        let Some(entries) = compound_entries(components) else {
            return;
        };
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(name, _)| name == "minecraft:container")
            .map(|(_, tag)| tag)
        {
            self.items.fill(None);
            for item_tag in items {
                let Some(item_entries) = compound_entries(item_tag) else {
                    continue;
                };
                let Some(slot) = get_byte(item_entries, "Slot") else {
                    continue;
                };
                if let Some(slot) = usize::try_from(slot)
                    .ok()
                    .filter(|slot| *slot < Self::MAX_ITEMS)
                {
                    self.items[slot] = PotItemStack::from_tag(item_tag);
                }
            }
        }
    }

    pub fn remove_components_from_tag(tag: &Tag) -> Tag {
        let Some(entries) = compound_entries(tag) else {
            return tag.clone();
        };
        Tag::Compound(
            entries
                .iter()
                .filter(|(name, _)| name != "Items")
                .cloned()
                .collect(),
        )
    }

    pub fn still_valid(&self, same_block_entity: bool, player_distance_sqr: f64) -> bool {
        same_block_entity && player_distance_sqr <= 64.0
    }

    pub fn set_changed_side_effects(
        &self,
        has_level: bool,
        event: Option<&'static str>,
    ) -> Option<(Option<&'static str>, i32)> {
        has_level.then_some((event, 3))
    }

    pub fn default_set_changed_side_effects(
        &self,
        has_level: bool,
    ) -> Option<(Option<&'static str>, i32)> {
        self.set_changed_side_effects(has_level, Some("minecraft:block_activate"))
    }

    pub fn item_owner_position(pos: BlockPos) -> (f64, f64, f64) {
        (pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5)
    }

    pub fn visual_rotation_y_degrees(facing: Direction) -> f32 {
        match facing.opposite() {
            Direction::South => 0.0,
            Direction::West => 90.0,
            Direction::North => 180.0,
            Direction::East => 270.0,
            Direction::Up | Direction::Down => 0.0,
        }
    }

    pub fn get_align_items_to_bottom(&self) -> bool {
        self.align_items_to_bottom
    }

    pub fn get_item(&self, slot: usize) -> Option<&PotItemStack> {
        self.items.get(slot).and_then(Option::as_ref)
    }

    pub fn set_item_no_update(&mut self, slot: usize, item: Option<PotItemStack>) -> bool {
        let Some(target) = self.items.get_mut(slot) else {
            return false;
        };
        *target = item.filter(|item| !item.is_empty());
        true
    }

    pub fn remove_item_no_update(&mut self, slot: usize) -> Option<PotItemStack> {
        self.items.get_mut(slot).and_then(Option::take)
    }

    pub fn swap_item_no_update(
        &mut self,
        slot: usize,
        held_item_stack: Option<PotItemStack>,
    ) -> Option<PotItemStack> {
        let retrieved = self.remove_item_no_update(slot);
        self.set_item_no_update(slot, held_item_stack);
        retrieved
    }

    pub fn filled_slot_count(&self) -> usize {
        self.items.iter().filter(|item| item.is_some()).count()
    }

    pub fn comparator_output(&self) -> u8 {
        self.filled_slot_count() as u8
    }
}
