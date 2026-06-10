use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerOpenersGameEvent {
    Open,
    Close,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ContainerOpenersCounterModel {
    pub open_count: i32,
    pub max_interaction_range: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContainerUserOpenState {
    pub has_container_open: bool,
    pub spectator: bool,
    pub interaction_range: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerOpenersCounterEffect {
    pub on_open: bool,
    pub on_close: bool,
    pub game_event: Option<ContainerOpenersGameEvent>,
    pub opener_count_changed: (i32, i32),
    pub schedule_recheck_delay: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopperGolemStatueSpawn {
    pub custom_name: Option<String>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub y_head_rot: f32,
    pub y_body_rot: f32,
    pub play_spawn_sound: bool,
}

impl ContainerOpenersCounterEffect {
    fn changed(previous: i32, current: i32) -> Self {
        Self {
            on_open: false,
            on_close: false,
            game_event: None,
            opener_count_changed: (previous, current),
            schedule_recheck_delay: None,
        }
    }
}

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

impl ContainerOpenersCounterModel {
    pub const CHECK_TICK_DELAY: u32 = 5;

    pub fn increment_openers(
        &mut self,
        max_interaction_range: f64,
    ) -> ContainerOpenersCounterEffect {
        let previous = self.open_count;
        self.open_count += 1;
        let mut effect = ContainerOpenersCounterEffect::changed(previous, self.open_count);
        if previous == 0 {
            effect.on_open = true;
            effect.game_event = Some(ContainerOpenersGameEvent::Open);
            effect.schedule_recheck_delay = Some(Self::CHECK_TICK_DELAY);
        }
        self.max_interaction_range = self.max_interaction_range.max(max_interaction_range);
        effect
    }

    pub fn decrement_openers(&mut self) -> ContainerOpenersCounterEffect {
        let previous = self.open_count;
        self.open_count -= 1;
        let mut effect = ContainerOpenersCounterEffect::changed(previous, self.open_count);
        if self.open_count == 0 {
            effect.on_close = true;
            effect.game_event = Some(ContainerOpenersGameEvent::Close);
            self.max_interaction_range = 0.0;
        }
        effect
    }

    pub fn search_box_inflate_range(&self) -> f64 {
        self.max_interaction_range + 4.0
    }

    pub fn entities_with_container_open<'a>(
        &self,
        entities: &'a [ContainerUserOpenState],
    ) -> Vec<&'a ContainerUserOpenState> {
        let _ = self.search_box_inflate_range();
        entities
            .iter()
            .filter(|entity| entity.has_container_open && !entity.spectator)
            .collect()
    }

    pub fn recheck_openers(
        &mut self,
        entities: &[ContainerUserOpenState],
    ) -> ContainerOpenersCounterEffect {
        let active_entities = self.entities_with_container_open(entities);
        self.max_interaction_range = active_entities
            .iter()
            .fold(0.0_f64, |max, entity| max.max(entity.interaction_range));

        let previous = self.open_count;
        let current = active_entities.len() as i32;
        let mut effect = ContainerOpenersCounterEffect::changed(previous, current);
        if previous != current {
            let is_open = current != 0;
            let was_open = previous != 0;
            if is_open && !was_open {
                effect.on_open = true;
                effect.game_event = Some(ContainerOpenersGameEvent::Open);
            } else if !is_open {
                effect.on_close = true;
                effect.game_event = Some(ContainerOpenersGameEvent::Close);
            }
            self.open_count = current;
        }
        if current > 0 {
            effect.schedule_recheck_delay = Some(Self::CHECK_TICK_DELAY);
        }
        effect
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
            world_position: BlockPos { x: 0, y: 0, z: 0 },
            items: vec![None; kind.size()],
            custom_name: None,
            lock_key: None,
            loot_table: None,
            loot_table_seed: 0,
            viewer_count: 0,
            content_changed: false,
            chest_lid: ChestLidController::new(),
            lid_progress: 0.0,
            shulker_progress_old: 0.0,
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
        self.set_item_no_update(slot, stack);
        self.content_changed = true;
        true
    }

    pub fn set_item_no_update(&mut self, slot: usize, stack: Option<PotItemStack>) -> bool {
        if slot >= self.items.len() {
            return false;
        }
        self.items[slot] = stack
            .filter(|stack| !stack.is_empty())
            .map(Self::limit_stack_size);
        true
    }

    pub fn count(&self) -> usize {
        self.items.iter().flatten().count()
    }

    pub fn container_size(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.iter().all(Option::is_none)
    }

    pub fn clear_content(&mut self) {
        self.items.fill(None);
    }

    pub fn get_item(&self, slot: usize) -> Option<&PotItemStack> {
        self.items.get(slot).and_then(Option::as_ref)
    }

    pub fn remove_item(&mut self, slot: usize, count: i32) -> Option<PotItemStack> {
        let removed = self.remove_item_no_update_with_count(slot, count)?;
        self.content_changed = true;
        Some(removed)
    }

    pub fn remove_item_no_update(&mut self, slot: usize) -> Option<PotItemStack> {
        self.remove_item_no_update_with_count(slot, Self::MAX_STACK_SIZE)
    }

    pub fn can_place_item(&self, slot: usize, item: &PotItemStack) -> bool {
        if slot >= self.items.len() || !self.accepts_item_type(item) {
            return false;
        }
        self.items[slot]
            .as_ref()
            .is_none_or(|current| current.count < Self::MAX_STACK_SIZE)
    }

    pub fn accepts_item_type(&self, item: &PotItemStack) -> bool {
        let _ = item;
        true
    }

    fn remove_item_no_update_with_count(
        &mut self,
        slot: usize,
        count: i32,
    ) -> Option<PotItemStack> {
        if count <= 0 {
            return None;
        }
        let stack = self.items.get_mut(slot)?.as_mut()?;
        let removed_count = stack.count.min(count);
        let removed = PotItemStack {
            item_id: stack.item_id.clone(),
            count: removed_count,
        };
        stack.count -= removed_count;
        if stack.is_empty() {
            self.items[slot] = None;
        }
        Some(removed)
    }

    fn limit_stack_size(mut stack: PotItemStack) -> PotItemStack {
        stack.count = stack.count.min(Self::MAX_STACK_SIZE);
        stack
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

    pub fn trapped_chest_signal_open_count(
        &self,
        previous: i32,
        current: i32,
    ) -> Option<TrappedChestOpenCountEffect> {
        if self.kind != ContainerBlockEntityKind::TrappedChest || previous == current {
            return None;
        }

        Some(TrappedChestOpenCountEffect {
            update_positions: vec![
                self.world_position,
                BlockPos {
                    x: self.world_position.x,
                    y: self.world_position.y - 1,
                    z: self.world_position.z,
                },
            ],
            source_block: "minecraft:trapped_chest",
            orientation_facing: self.facing.opposite(),
            orientation_up: Direction::Up,
        })
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

    pub fn insert_item(&mut self, mut item_stack: PotItemStack) -> Option<PotItemStack> {
        if item_stack.is_empty() {
            return None;
        }

        let max_stack_size = Self::MAX_STACK_SIZE;
        for slot in 0..self.items.len() {
            let target = self.items[slot].as_mut();
            let can_insert = target
                .as_ref()
                .is_none_or(|target| target.item_id == item_stack.item_id);
            if !can_insert {
                continue;
            }

            let target_count = target.as_ref().map_or(0, |target| target.count);
            let transfer_count = item_stack.count.min(max_stack_size - target_count);
            if transfer_count > 0 {
                if let Some(target) = target {
                    target.count += transfer_count;
                    item_stack.count -= transfer_count;
                } else {
                    let inserted = PotItemStack {
                        item_id: item_stack.item_id.clone(),
                        count: transfer_count,
                    };
                    self.set_item(slot, Some(inserted));
                    item_stack.count -= transfer_count;
                }
            }

            if item_stack.is_empty() {
                break;
            }
        }

        (!item_stack.is_empty()).then_some(item_stack)
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

    pub const fn hopper_suck_aabb() -> crate::collision_shape::Aabb {
        crate::collision_shape::Aabb::new(0.0, 11.0 / 16.0, 0.0, 1.0, 2.0, 1.0)
    }

    pub fn hopper_level_x(&self) -> f64 {
        f64::from(self.world_position.x) + 0.5
    }

    pub fn hopper_level_y(&self) -> f64 {
        f64::from(self.world_position.y) + 0.5
    }

    pub fn hopper_level_z(&self) -> f64 {
        f64::from(self.world_position.z) + 0.5
    }

    pub const fn hopper_is_grid_aligned(&self) -> bool {
        true
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
            if self.loot_table_seed != 0 {
                fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
            }
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

    pub fn collect_implicit_components(&self) -> Tag {
        let mut components = Vec::new();
        if let Some(name) = &self.custom_name {
            components.push((
                "minecraft:custom_name".to_string(),
                Tag::String(name.clone()),
            ));
        }
        if let Some(lock) = &self.lock_key {
            components.push(("minecraft:lock".to_string(), Tag::String(lock.clone())));
        }
        components.push((
            "minecraft:container".to_string(),
            container_items_tag(&self.items),
        ));
        if let Some(loot_table) = &self.loot_table {
            let mut loot = vec![("loot_table".to_string(), Tag::String(loot_table.clone()))];
            if self.loot_table_seed != 0 {
                loot.push(("seed".to_string(), Tag::Long(self.loot_table_seed)));
            }
            components.push(("minecraft:container_loot".to_string(), Tag::Compound(loot)));
        }
        Tag::Compound(components)
    }

    pub fn apply_implicit_components(&mut self, components: &Tag) {
        let Some(entries) = compound_entries(components) else {
            return;
        };
        self.custom_name = get_string(entries, "minecraft:custom_name").map(ToString::to_string);
        self.lock_key = get_string(entries, "minecraft:lock").map(ToString::to_string);
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(name, _)| name == "minecraft:container")
            .map(|(_, tag)| tag)
        {
            self.items.fill(None);
            load_container_component_items(items, &mut self.items);
        }
        if let Some(loot_entries) = entries
            .iter()
            .find(|(name, _)| name == "minecraft:container_loot")
            .and_then(|(_, tag)| compound_entries(tag))
        {
            self.loot_table = get_string(loot_entries, "loot_table").map(ToString::to_string);
            self.loot_table_seed = get_long(loot_entries, "seed").unwrap_or(0);
        }
    }

    pub fn remove_components_from_tag(tag: &Tag) -> Tag {
        let Some(entries) = compound_entries(tag) else {
            return tag.clone();
        };
        Tag::Compound(
            entries
                .iter()
                .filter(|(name, _)| {
                    !matches!(
                        name.as_str(),
                        "CustomName" | "lock" | "Items" | "LootTable" | "LootTableSeed"
                    )
                })
                .cloned()
                .collect(),
        )
    }

    pub(super) fn tick_shulker_animation(&mut self) {
        self.shulker_progress_old = self.lid_progress;
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

fn load_container_component_items(saved_items: &[Tag], items: &mut [Option<PotItemStack>]) {
    for item in saved_items {
        let Some(item_entries) = compound_entries(item) else {
            continue;
        };
        let slot = get_byte(item_entries, "Slot").unwrap_or(-1);
        if (0..items.len() as i8).contains(&slot) {
            items[slot as usize] = PotItemStack::from_tag(item);
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

    pub fn update_packet_type(&self) -> BlockEntityTypeId {
        BlockEntityTypeId::DecoratedPot
    }

    pub fn update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn create_decorated_pot_template(decorations: &PotDecorations) -> Tag {
        Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:decorated_pot".to_string()),
            ),
            (
                "components".to_string(),
                Tag::Compound(vec![(
                    "minecraft:pot_decorations".to_string(),
                    decorations.to_tag(),
                )]),
            ),
        ])
    }

    pub fn create_decorated_pot_instance(decorations: &PotDecorations) -> Tag {
        Self::create_decorated_pot_template(decorations)
    }

    pub fn collect_implicit_components(&self) -> Tag {
        Tag::Compound(vec![
            (
                "minecraft:pot_decorations".to_string(),
                self.decorations.to_tag(),
            ),
            (
                "minecraft:container".to_string(),
                Tag::List(
                    self.item
                        .iter()
                        .filter(|item| !item.is_empty())
                        .map(PotItemStack::to_tag)
                        .collect(),
                ),
            ),
        ])
    }

    pub fn apply_implicit_components(&mut self, components: &Tag) {
        let Some(entries) = compound_entries(components) else {
            return;
        };
        if let Some((_, decorations)) = entries
            .iter()
            .find(|(name, _)| name == "minecraft:pot_decorations")
        {
            self.decorations = PotDecorations::from_tag(decorations);
        }
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(name, _)| name == "minecraft:container")
            .map(|(_, tag)| tag)
        {
            self.item = items.first().and_then(PotItemStack::from_tag);
        }
    }

    pub fn remove_components_from_tag(tag: &Tag) -> Tag {
        let Some(entries) = compound_entries(tag) else {
            return tag.clone();
        };
        Tag::Compound(
            entries
                .iter()
                .filter(|(name, _)| name != "sherds" && name != "item")
                .cloned()
                .collect(),
        )
    }

    pub fn get_the_item(&mut self) -> Option<&PotItemStack> {
        self.unpack_loot_table();
        self.item.as_ref()
    }

    pub fn split_the_item(&mut self, count: i32) -> Option<PotItemStack> {
        self.unpack_loot_table();
        let item = self.item.as_mut()?;
        let split_count = item.count.min(count).max(0);
        if split_count == 0 {
            return None;
        }
        let result = PotItemStack {
            item_id: item.item_id.clone(),
            count: split_count,
        };
        item.count -= split_count;
        if item.is_empty() {
            self.item = None;
        }
        Some(result)
    }

    pub fn set_the_item(&mut self, item: Option<PotItemStack>) {
        self.unpack_loot_table();
        self.item = item.filter(|item| !item.is_empty());
    }

    pub fn unpack_loot_table(&mut self) -> bool {
        if self.loot_table.take().is_some() {
            self.loot_table_seed = 0;
            true
        } else {
            false
        }
    }

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

    pub fn create_statue(&mut self, copper_golem_custom_name: Option<String>) {
        self.custom_name = copper_golem_custom_name;
    }

    pub fn remove_statue(&self, pos: BlockPos, facing: Direction) -> CopperGolemStatueSpawn {
        let y_rot = copper_golem_statue_y_rot(facing);
        CopperGolemStatueSpawn {
            custom_name: self.custom_name.clone(),
            x: f64::from(pos.x) + 0.5,
            y: f64::from(pos.y),
            z: f64::from(pos.z) + 0.5,
            y_rot,
            y_head_rot: y_rot,
            y_body_rot: y_rot,
            play_spawn_sound: true,
        }
    }

    pub const fn update_packet_type(&self) -> BlockEntityTypeId {
        BlockEntityTypeId::CopperGolemStatue
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

    pub fn item_components_for_pose(&self, pose: CopperGolemStatuePose) -> Tag {
        let mut fields = vec![(
            "minecraft:block_state".to_string(),
            Tag::Compound(vec![(
                "copper_golem_pose".to_string(),
                Tag::String(pose.serialized_name().to_string()),
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

    pub fn clone_item_components(&self) -> Tag {
        self.item_components_for_pose(self.pose)
    }
}

const fn copper_golem_statue_y_rot(facing: Direction) -> f32 {
    match facing {
        Direction::South => 0.0,
        Direction::West => 90.0,
        Direction::North => 180.0,
        Direction::East => 270.0,
        Direction::Up | Direction::Down => 0.0,
    }
}
