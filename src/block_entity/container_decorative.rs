use super::*;

impl ContainerBlockEntityKind {
    pub fn size(self) -> usize {
        match self {
            Self::Dispenser | Self::Dropper => 9,
            Self::Hopper => 5,
            Self::Chest | Self::TrappedChest | Self::Barrel | Self::ShulkerBox => 27,
        }
    }

    pub fn menu_type(self) -> &'static str {
        match self {
            Self::Chest | Self::TrappedChest | Self::Barrel => "generic_9x3",
            Self::ShulkerBox => "shulker_box",
            Self::Dispenser | Self::Dropper => "generic_3x3",
            Self::Hopper => "hopper",
        }
    }

    pub fn default_name(self) -> &'static str {
        match self {
            Self::Chest | Self::TrappedChest => "container.chest",
            Self::Barrel => "container.barrel",
            Self::ShulkerBox => "container.shulkerBox",
            Self::Dispenser => "container.dispenser",
            Self::Dropper => "container.dropper",
            Self::Hopper => "container.hopper",
        }
    }
}

impl ContainerBlockEntityModel {
    pub const CHEST_LID_STEP: f32 = 0.1;
    pub const SHULKER_OPENING_TICK_LENGTH: i32 = 10;
    pub const SHULKER_MAX_LID_HEIGHT: f32 = 0.5;
    pub const SHULKER_MAX_LID_ROTATION: f32 = 270.0;
    pub const HOPPER_MOVE_ITEM_SPEED: i32 = 8;
    pub const HOPPER_NO_COOLDOWN: i32 = -1;
    pub const MAX_STACK_SIZE: i32 = 64;

    pub fn new(kind: ContainerBlockEntityKind) -> Self {
        Self {
            kind,
            items: vec![None; kind.size()],
            custom_name: None,
            lock_key: None,
            loot_table: None,
            loot_table_seed: 0,
            viewer_count: 0,
            chest_lid: ChestLidController::new(),
            lid_progress: 0.0,
            shulker_status: ShulkerBoxAnimationStatus::Closed,
            shulker_color: None,
            transfer_cooldown: if kind == ContainerBlockEntityKind::Hopper {
                Self::HOPPER_NO_COOLDOWN
            } else {
                0
            },
            facing: Direction::Down,
        }
    }

    pub fn can_open(&self, player_lock_key: Option<&str>, spectator: bool) -> bool {
        if self.loot_table.is_some() && spectator {
            return false;
        }
        self.lock_key
            .as_deref()
            .is_none_or(|lock| player_lock_key == Some(lock))
    }

    pub fn create_menu(
        &mut self,
        player_lock_key: Option<&str>,
        spectator: bool,
    ) -> Option<&'static str> {
        if !self.can_open(player_lock_key, spectator) {
            return None;
        }
        self.unpack_loot_table();
        Some(self.kind.menu_type())
    }

    pub fn open_menu(
        &mut self,
        container_id: i32,
        player_lock_key: Option<&str>,
        spectator: bool,
    ) -> Option<BlockEntityMenuOpen> {
        let menu_type = self.create_menu(player_lock_key, spectator)?;
        self.start_open();
        Some(BlockEntityMenuOpen {
            container_id,
            menu_type,
            initial_slots: self.items.clone(),
        })
    }

    pub fn close_menu(&mut self, menu: BlockEntityMenuOpen) -> BlockEntityMenuClose {
        self.stop_open();
        menu.close()
    }

    pub fn unpack_loot_table(&mut self) -> bool {
        if self.loot_table.take().is_some() {
            self.loot_table_seed = 0;
            true
        } else {
            false
        }
    }

    pub fn set_item(&mut self, slot: usize, stack: Option<PotItemStack>) -> bool {
        if slot >= self.items.len() {
            return false;
        }
        self.unpack_loot_table();
        self.items[slot] = stack.filter(|stack| !stack.is_empty());
        true
    }

    pub fn start_open(&mut self) {
        self.viewer_count = (self.viewer_count + 1).max(0);
        if self.kind == ContainerBlockEntityKind::ShulkerBox && self.viewer_count == 1 {
            self.shulker_status = ShulkerBoxAnimationStatus::Opening;
        }
    }

    pub fn stop_open(&mut self) {
        self.viewer_count -= 1;
        if self.kind == ContainerBlockEntityKind::ShulkerBox && self.viewer_count <= 0 {
            self.shulker_status = ShulkerBoxAnimationStatus::Closing;
        }
        if self.kind != ContainerBlockEntityKind::ShulkerBox {
            self.viewer_count = self.viewer_count.max(0);
        }
    }

    pub fn tick_lid(&mut self) {
        match self.kind {
            ContainerBlockEntityKind::Chest | ContainerBlockEntityKind::TrappedChest => {
                self.chest_lid.should_be_open(self.viewer_count > 0);
                self.chest_lid.tick_lid();
                self.lid_progress = self.chest_lid.openness();
            }
            ContainerBlockEntityKind::ShulkerBox => self.tick_shulker_animation(),
            _ => {}
        }
    }

    pub fn chest_lid_openness(&self, partial_tick: f32) -> f32 {
        self.chest_lid.get_openness(partial_tick)
    }

    pub fn trapped_chest_signal(&self) -> u8 {
        if self.kind == ContainerBlockEntityKind::TrappedChest {
            self.viewer_count.clamp(0, i32::from(MAX_SIGNAL)) as u8
        } else {
            0
        }
    }

    pub fn barrel_is_open(&self) -> bool {
        self.kind == ContainerBlockEntityKind::Barrel && self.viewer_count > 0
    }

    pub fn merged_chest_access_size(&self, neighbour_is_same_chest_type: bool) -> usize {
        if matches!(
            self.kind,
            ContainerBlockEntityKind::Chest | ContainerBlockEntityKind::TrappedChest
        ) && neighbour_is_same_chest_type
        {
            self.kind.size() * 2
        } else {
            self.kind.size()
        }
    }

    pub fn shulker_is_closed(&self) -> bool {
        self.kind == ContainerBlockEntityKind::ShulkerBox
            && self.shulker_status == ShulkerBoxAnimationStatus::Closed
    }

    pub fn can_place_through_face(
        &self,
        _slot: usize,
        item_id: &str,
        _direction: Direction,
    ) -> bool {
        self.kind != ContainerBlockEntityKind::ShulkerBox || !item_id.ends_with("shulker_box")
    }

    pub fn random_non_empty_slot(&self, random_rolls: &[usize]) -> Option<usize> {
        let mut replace_slot = None;
        let mut replace_odds = 1;
        let mut roll_index = 0;
        for (slot, stack) in self.items.iter().enumerate() {
            if stack.is_some() {
                let roll = random_rolls.get(roll_index).copied().unwrap_or(0) % replace_odds;
                roll_index += 1;
                if roll == 0 {
                    replace_slot = Some(slot);
                }
                replace_odds += 1;
            }
        }
        replace_slot
    }

    pub fn activate_once(&self, random_rolls: &[usize]) -> ContainerActivation {
        let Some(slot) = self.random_non_empty_slot(random_rolls) else {
            return ContainerActivation::None;
        };
        match self.kind {
            ContainerBlockEntityKind::Dispenser => ContainerActivation::Dispense { slot },
            ContainerBlockEntityKind::Dropper => ContainerActivation::Drop { slot },
            _ => ContainerActivation::None,
        }
    }

    pub fn hopper_tick(
        &mut self,
        enabled: bool,
        attached_has_space: bool,
        source_has_item: bool,
    ) -> ContainerActivation {
        if self.kind != ContainerBlockEntityKind::Hopper {
            return ContainerActivation::None;
        }
        self.transfer_cooldown -= 1;
        if self.transfer_cooldown > 0 || !enabled {
            return ContainerActivation::None;
        }
        self.transfer_cooldown = 0;

        if attached_has_space {
            if let Some(slot) = self.items.iter().position(Option::is_some) {
                self.transfer_cooldown = Self::HOPPER_MOVE_ITEM_SPEED;
                return ContainerActivation::Push { from_slot: slot };
            }
        }
        if source_has_item && self.items.iter().any(Option::is_none) {
            let slot = self.items.iter().position(Option::is_none).unwrap_or(0);
            self.transfer_cooldown = Self::HOPPER_MOVE_ITEM_SPEED;
            return ContainerActivation::Pull { to_slot: slot };
        }
        ContainerActivation::None
    }

    pub fn hopper_slots_for_face(&self, _direction: Direction) -> Vec<usize> {
        if self.kind == ContainerBlockEntityKind::Hopper {
            (0..self.items.len()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn hopper_can_place_item(
        &self,
        slot: usize,
        item: &PotItemStack,
        direction: Direction,
    ) -> bool {
        self.hopper_slots_for_face(direction).contains(&slot) && !item.is_empty()
    }

    pub fn hopper_can_take_item(&self, slot: usize, direction: Direction) -> bool {
        self.hopper_slots_for_face(direction).contains(&slot) && self.items[slot].is_some()
    }

    pub fn comparator_output(&self) -> u8 {
        inventory_comparator_output(&self.items)
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(name.clone())));
        }
        if let Some(lock) = &self.lock_key {
            fields.push(("lock".to_string(), Tag::String(lock.clone())));
        }
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
        } else {
            fields.push(("Items".to_string(), container_items_tag(&self.items)));
        }
        if self.kind == ContainerBlockEntityKind::Hopper {
            fields.push((
                "TransferCooldown".to_string(),
                Tag::Int(self.transfer_cooldown),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(kind: ContainerBlockEntityKind, tag: &Tag) -> Self {
        let mut container = Self::new(kind);
        let Some(entries) = compound_entries(tag) else {
            return container;
        };
        container.custom_name = get_string(entries, "CustomName").map(str::to_string);
        container.lock_key = get_string(entries, "lock").map(str::to_string);
        container.loot_table = get_string(entries, "LootTable").map(str::to_string);
        container.loot_table_seed = get_long(entries, "LootTableSeed").unwrap_or(0);
        if container.loot_table.is_none() {
            load_container_items(entries, &mut container.items);
        }
        if kind == ContainerBlockEntityKind::Hopper {
            container.transfer_cooldown =
                get_int(entries, "TransferCooldown").unwrap_or(Self::HOPPER_NO_COOLDOWN);
        }
        container
    }

    pub(super) fn tick_shulker_animation(&mut self) {
        match self.shulker_status {
            ShulkerBoxAnimationStatus::Closed => self.lid_progress = 0.0,
            ShulkerBoxAnimationStatus::Opening => {
                self.lid_progress += 0.1;
                if self.lid_progress >= 1.0 {
                    self.lid_progress = 1.0;
                    self.shulker_status = ShulkerBoxAnimationStatus::Opened;
                }
            }
            ShulkerBoxAnimationStatus::Opened => self.lid_progress = 1.0,
            ShulkerBoxAnimationStatus::Closing => {
                self.lid_progress -= 0.1;
                if self.lid_progress <= 0.0 {
                    self.lid_progress = 0.0;
                    self.shulker_status = ShulkerBoxAnimationStatus::Closed;
                }
            }
        }
    }
}

impl ChestLidController {
    pub const STEP: f32 = 0.1;

    pub const fn new() -> Self {
        Self {
            should_be_open: false,
            openness: 0.0,
            previous_openness: 0.0,
        }
    }

    pub fn tick_lid(&mut self) {
        self.previous_openness = self.openness;
        if !self.should_be_open && self.openness > 0.0 {
            self.openness = (self.openness - Self::STEP).max(0.0);
        } else if self.should_be_open && self.openness < 1.0 {
            self.openness = (self.openness + Self::STEP).min(1.0);
        }
    }

    pub fn get_openness(&self, partial_tick: f32) -> f32 {
        self.previous_openness + partial_tick * (self.openness - self.previous_openness)
    }

    pub fn should_be_open(&mut self, should_be_open: bool) {
        self.should_be_open = should_be_open;
    }

    pub const fn openness(&self) -> f32 {
        self.openness
    }

    pub const fn previous_openness(&self) -> f32 {
        self.previous_openness
    }
}

impl DecoratedPotWobbleStyle {
    pub fn id(self) -> i32 {
        match self {
            Self::Positive => 0,
            Self::Negative => 1,
        }
    }

    pub fn duration(self) -> i32 {
        match self {
            Self::Positive => 7,
            Self::Negative => 10,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Positive),
            1 => Some(Self::Negative),
            _ => None,
        }
    }
}

impl DecoratedPotBlockEntity {
    pub const EVENT_POT_WOBBLES: i32 = 1;

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if !self.decorations.is_empty() {
            fields.push(("sherds".to_string(), self.decorations.to_tag()));
        }
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            if self.loot_table_seed != 0 {
                fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
            }
        } else if let Some(item) = &self.item {
            if !item.is_empty() {
                fields.push(("item".to_string(), item.to_tag()));
            }
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::default();
        };
        let decorations = entries
            .iter()
            .find(|(name, _)| name == "sherds")
            .map(|(_, tag)| PotDecorations::from_tag(tag))
            .unwrap_or_default();
        let loot_table = get_string(entries, "LootTable").map(ToString::to_string);
        let loot_table_seed = entries
            .iter()
            .find_map(|(name, tag)| match tag {
                Tag::Long(seed) if name == "LootTableSeed" => Some(*seed),
                _ => None,
            })
            .unwrap_or(0);
        let item = if loot_table.is_some() {
            None
        } else {
            entries
                .iter()
                .find(|(name, _)| name == "item")
                .and_then(|(_, tag)| PotItemStack::from_tag(tag))
        };
        Self {
            decorations,
            item,
            loot_table,
            loot_table_seed,
            wobble_started_at_tick: 0,
            last_wobble_style: None,
        }
    }

    pub fn trigger_event(&mut self, event: i32, data: i32, game_time: i64) -> bool {
        let Some(style) = DecoratedPotWobbleStyle::from_id(data) else {
            return false;
        };
        if event != Self::EVENT_POT_WOBBLES {
            return false;
        }
        self.wobble_started_at_tick = game_time;
        self.last_wobble_style = Some(style);
        true
    }

    pub fn destruction_drops(&self) -> DecoratedPotDrops {
        DecoratedPotDrops {
            decoration_items: self.decorations.ordered(),
            stored_item: self.item.clone().filter(|item| !item.is_empty()),
        }
    }

    pub fn comparator_output(&self) -> u8 {
        inventory_comparator_output(std::slice::from_ref(&self.item))
    }
}

impl CopperWeatherState {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Unaffected => "unaffected",
            Self::Exposed => "exposed",
            Self::Weathered => "weathered",
            Self::Oxidized => "oxidized",
        }
    }
}

impl CopperGolemStatuePose {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Standing => "standing",
            Self::Sitting => "sitting",
            Self::Running => "running",
            Self::Star => "star",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Standing => Self::Sitting,
            Self::Sitting => Self::Running,
            Self::Running => Self::Star,
            Self::Star => Self::Standing,
        }
    }

    pub fn comparator_output(self) -> u8 {
        match self {
            Self::Standing => 1,
            Self::Sitting => 2,
            Self::Running => 3,
            Self::Star => 4,
        }
    }
}

impl CopperGolemStatueBlockEntity {
    pub fn from_block_state(block_state: &str, pose: CopperGolemStatuePose) -> Option<Self> {
        let id = block_state.strip_prefix("minecraft:")?;
        let (waxed, id) = id
            .strip_prefix("waxed_")
            .map(|id| (true, id))
            .unwrap_or((false, id));
        let weather_state = match id {
            "copper_golem_statue" => CopperWeatherState::Unaffected,
            "exposed_copper_golem_statue" => CopperWeatherState::Exposed,
            "weathered_copper_golem_statue" => CopperWeatherState::Weathered,
            "oxidized_copper_golem_statue" => CopperWeatherState::Oxidized,
            _ => return None,
        };
        Some(Self {
            weather_state,
            waxed,
            pose,
            custom_name: None,
        })
    }

    pub fn update_pose(&mut self) {
        self.pose = self.pose.next();
    }

    pub fn comparator_output(&self) -> u8 {
        self.pose.comparator_output()
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(custom_name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn clone_item_components(&self) -> Tag {
        let mut fields = vec![(
            "minecraft:block_state".to_string(),
            Tag::Compound(vec![(
                "copper_golem_pose".to_string(),
                Tag::String(self.pose.serialized_name().to_string()),
            )]),
        )];
        if let Some(custom_name) = &self.custom_name {
            fields.push((
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            ));
        }
        Tag::Compound(fields)
    }
}

impl SkullBlockEntity {
    pub fn new() -> Self {
        Self {
            profile: None,
            note_block_sound: None,
            custom_name: None,
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(profile) = &self.profile {
            fields.push(("profile".to_string(), profile.clone()));
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            fields.push((
                "note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            ));
        }
        if let Some(custom_name) = &self.custom_name {
            fields.push(("custom_name".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Tag::Compound(entries) = tag else {
            return Self::new();
        };
        Self {
            profile: entries
                .iter()
                .find(|(name, _)| name == "profile")
                .map(|(_, tag)| tag.clone()),
            note_block_sound: get_string(entries, "note_block_sound").map(ToString::to_string),
            custom_name: get_string(entries, "custom_name").map(ToString::to_string),
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn apply_implicit_components(&mut self, components: &BTreeMap<String, Tag>) {
        self.profile = components.get("minecraft:profile").cloned();
        self.note_block_sound = components
            .get("minecraft:note_block_sound")
            .and_then(|tag| match tag {
                Tag::String(id) => Some(id.clone()),
                _ => None,
            });
        self.custom_name = components
            .get("minecraft:custom_name")
            .and_then(|tag| match tag {
                Tag::String(name) => Some(name.clone()),
                _ => None,
            });
    }

    pub fn collect_implicit_components(&self) -> BTreeMap<String, Tag> {
        let mut components = BTreeMap::new();
        if let Some(profile) = &self.profile {
            components.insert("minecraft:profile".to_string(), profile.clone());
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            components.insert(
                "minecraft:note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            );
        }
        if let Some(custom_name) = &self.custom_name {
            components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            );
        }
        components
    }

    pub fn remove_components_from_tag(tag: &mut Tag) {
        if let Tag::Compound(entries) = tag {
            entries.retain(|(name, _)| {
                name != "profile" && name != "note_block_sound" && name != "custom_name"
            });
        }
    }

    pub fn animation_tick(&mut self, powered: bool) {
        if powered {
            self.is_animating = true;
            self.animation_tick_count += 1;
        } else {
            self.is_animating = false;
        }
    }

    pub fn animation(&self, partial_tick: f32) -> f32 {
        if self.is_animating {
            self.animation_tick_count as f32 + partial_tick
        } else {
            self.animation_tick_count as f32
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}
