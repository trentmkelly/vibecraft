use super::*;

impl BlockEntity {
    pub fn new(
        ty: BlockEntityTypeId,
        pos: BlockPos,
        block_state: &str,
    ) -> Result<Self, BlockEntityError> {
        if !is_valid_block_state(ty, block_state) {
            return Err(BlockEntityError::InvalidBlockState {
                ty,
                block_state: block_state.to_string(),
            });
        }

        Ok(Self {
            ty,
            pos,
            block_state: block_state.to_string(),
            custom_data: BTreeMap::new(),
            components: BTreeMap::new(),
            has_level: false,
            removed: false,
            changed: false,
            tick_count: 0,
        })
    }

    pub fn set_level(&mut self) {
        self.has_level = true;
    }

    pub fn set_removed(&mut self) {
        self.removed = true;
    }

    pub fn clear_removed(&mut self) {
        self.removed = false;
    }

    pub fn set_changed(&mut self) -> Option<BlockEntityChangedEffect> {
        if !self.has_level {
            return None;
        }
        self.changed = true;
        Some(block_entity_changed_effect(self.pos, &self.block_state))
    }

    pub fn set_changed_in_chunk_manager(
        &mut self,
        chunks: &mut ChunkManager,
    ) -> Option<BlockEntityChangedEffect> {
        let effect = self.set_changed()?;
        chunks.mark_dirty(effect.chunk_pos);
        Some(effect)
    }

    pub fn save_custom_only(&self) -> Tag {
        compound_from_map(&self.custom_data)
    }

    pub fn save_without_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn save_with_id(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        compound_from_map(&values)
    }

    pub fn save_with_full_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        values.insert("x".to_string(), Tag::Int(self.pos.x));
        values.insert("y".to_string(), Tag::Int(self.pos.y));
        values.insert("z".to_string(), Tag::Int(self.pos.z));
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn data_get_block_nbt(&self) -> Tag {
        self.save_with_full_metadata()
    }

    pub fn destruction_drops(
        &self,
        context: BlockEntityDestructionContext,
    ) -> BlockEntityDestructionDrops {
        if !context.do_tile_drops || !context.explosion_survives {
            return BlockEntityDestructionDrops {
                block_item: None,
                stored_items: Vec::new(),
            };
        }

        let block_item = (context.correct_tool || context.silk_touch)
            .then(|| block_item_from_state(&self.block_state).to_string());
        let stored_items = stored_item_drops_from_tag(&self.save_without_metadata());
        BlockEntityDestructionDrops {
            block_item,
            stored_items,
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        match self.ty {
            BlockEntityTypeId::Chest
            | BlockEntityTypeId::TrappedChest
            | BlockEntityTypeId::Barrel
            | BlockEntityTypeId::Hopper
            | BlockEntityTypeId::Dispenser
            | BlockEntityTypeId::Dropper => Tag::Compound(Vec::new()),
            BlockEntityTypeId::Sign | BlockEntityTypeId::HangingSign | BlockEntityTypeId::Skull => {
                self.save_custom_only()
            }
            BlockEntityTypeId::MobSpawner => {
                let Tag::Compound(mut values) = self.save_custom_only() else {
                    return Tag::Compound(Vec::new());
                };
                values.retain(|(key, _)| key != "SpawnPotentials");
                Tag::Compound(values)
            }
            _ => self.save_without_metadata(),
        }
    }

    pub fn get_update_packet(&self) -> ClientboundBlockEntityDataPacket {
        ClientboundBlockEntityDataPacket {
            pos: self.pos,
            ty: self.ty,
            tag: self.get_update_tag(),
        }
    }

    pub fn handle_update_tag(&mut self, tag: &Tag) {
        let Some(entries) = compound_entries(tag) else {
            return;
        };

        self.custom_data.clear();
        self.components.clear();
        for (key, value) in entries {
            match key.as_str() {
                "id" | "x" | "y" | "z" => {}
                "components" => {
                    self.components = map_from_compound(value);
                }
                _ => {
                    self.custom_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    pub fn tick(&mut self, client_side: bool) -> bool {
        let tick_kind = type_info(self.ty).tick_kind;
        let should_tick = matches!(
            (tick_kind, client_side),
            (BlockEntityTickKind::Both, _)
                | (BlockEntityTickKind::Server, false)
                | (BlockEntityTickKind::Client, true)
        );

        if should_tick && !self.removed && self.has_level {
            self.tick_count += 1;
            true
        } else {
            false
        }
    }
}

impl TickingBlockEntity {
    pub fn new(entity: BlockEntity, client_side: bool) -> Self {
        Self {
            entity,
            client_side,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.entity.tick(self.client_side)
    }

    pub fn is_removed(&self) -> bool {
        self.entity.removed
    }

    pub fn pos(&self) -> BlockPos {
        self.entity.pos
    }

    pub fn type_key(&self) -> &'static str {
        type_info(self.entity.ty).key
    }
}

fn block_entity_changed_effect(pos: BlockPos, block_state: &str) -> BlockEntityChangedEffect {
    BlockEntityChangedEffect {
        chunk_pos: ChunkPos {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        },
        update_output_signal: block_state != "minecraft:air",
    }
}

pub fn load_static(
    pos: BlockPos,
    block_state: &str,
    tag: &Tag,
) -> Result<BlockEntity, BlockEntityError> {
    let values = compound_entries(tag);
    let id = values
        .and_then(|entries| get_string(entries, "id"))
        .ok_or(BlockEntityError::MissingId)?;
    let ty = type_by_key(id).ok_or_else(|| BlockEntityError::UnknownType(id.to_string()))?;
    let mut entity = BlockEntity::new(ty, pos, block_state)?;

    if let Some(entries) = values {
        for (key, value) in entries {
            match key.as_str() {
                "id" | "x" | "y" | "z" => {}
                "components" => {
                    entity.components = map_from_compound(value);
                }
                _ => {
                    entity.custom_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    Ok(entity)
}

pub fn load_static_with_data_version(
    pos: BlockPos,
    block_state: &str,
    tag: &Tag,
    data_version: i32,
) -> Result<BlockEntity, BlockEntityError> {
    require_current_world_data_version(data_version)
        .map_err(BlockEntityError::UnsupportedDataVersion)?;
    load_static(pos, block_state, tag)
}

pub fn corrected_pos_from_chunk(base_chunk_x: i32, base_chunk_z: i32, tag: &Tag) -> BlockPos {
    let entries = compound_entries(tag);
    let x = entries
        .and_then(|entries| get_int(entries, "x"))
        .unwrap_or(0);
    let y = entries
        .and_then(|entries| get_int(entries, "y"))
        .unwrap_or(0);
    let z = entries
        .and_then(|entries| get_int(entries, "z"))
        .unwrap_or(0);
    let section_x = x.div_euclid(16);
    let section_z = z.div_euclid(16);

    if section_x == base_chunk_x && section_z == base_chunk_z {
        BlockPos { x, y, z }
    } else {
        BlockPos {
            x: base_chunk_x * 16 + x.rem_euclid(16),
            y,
            z: base_chunk_z * 16 + z.rem_euclid(16),
        }
    }
}

pub fn block_entity_packet_from_chunk(
    entity: &BlockEntity,
    chunk_min_y: i32,
) -> (u8, i16, BlockEntityTypeId, Tag) {
    let packed_xz = ((entity.pos.x & 15) << 4) | (entity.pos.z & 15);
    let section_y = (entity.pos.y - chunk_min_y) as i16;
    (
        packed_xz as u8,
        section_y,
        entity.ty,
        entity.get_update_tag(),
    )
}

pub(super) fn compound_from_map(values: &BTreeMap<String, Tag>) -> Tag {
    Tag::Compound(
        values
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

pub(super) fn map_from_compound(tag: &Tag) -> BTreeMap<String, Tag> {
    compound_entries(tag)
        .map(|entries| entries.iter().cloned().collect())
        .unwrap_or_default()
}

pub(super) fn inventory_comparator_output(items: &[Option<PotItemStack>]) -> u8 {
    let non_empty = items.iter().filter(|stack| stack.is_some()).count();
    if non_empty == 0 {
        return 0;
    }
    let fullness = items
        .iter()
        .filter_map(|stack| stack.as_ref())
        .map(|stack| (stack.count.max(0) as f32 / 64.0).min(1.0))
        .sum::<f32>()
        / items.len().max(1) as f32;
    (1 + (fullness * 14.0).floor() as u8).min(MAX_SIGNAL)
}

pub(super) fn block_item_from_state(block_state: &str) -> &str {
    block_state.split('[').next().unwrap_or(block_state)
}

pub(super) fn stored_item_drops_from_tag(tag: &Tag) -> Vec<PotItemStack> {
    let Some(entries) = compound_entries(tag) else {
        return Vec::new();
    };
    let mut drops = Vec::new();

    if let Some(Tag::List(items)) = entries
        .iter()
        .find(|(key, _)| key == "Items")
        .map(|(_, tag)| tag)
    {
        drops.extend(items.iter().filter_map(PotItemStack::from_tag));
    }

    for key in ["item", "Book", "RecordItem"] {
        if let Some(item) = entries
            .iter()
            .find(|(name, _)| name == key)
            .and_then(|(_, tag)| PotItemStack::from_tag(tag))
        {
            drops.push(item);
        }
    }
    drops
}

pub(super) fn container_items_tag(items: &[Option<PotItemStack>]) -> Tag {
    Tag::List(
        items
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

pub(super) fn load_container_items(entries: &[(String, Tag)], items: &mut [Option<PotItemStack>]) {
    if let Some(Tag::List(saved_items)) = entries
        .iter()
        .find(|(name, _)| name == "Items")
        .map(|(_, tag)| tag)
    {
        for item in saved_items {
            if let Some(item_entries) = compound_entries(item) {
                let slot = get_byte(item_entries, "Slot").unwrap_or(-1);
                if (0..items.len() as i8).contains(&slot) {
                    items[slot as usize] = PotItemStack::from_tag(item);
                }
            }
        }
    }
}

pub(super) fn shrink_stack(stack: &mut Option<PotItemStack>, amount: i32) {
    if let Some(item) = stack {
        item.count -= amount;
        if item.count <= 0 {
            *stack = None;
        }
    }
}

pub(super) fn is_brewing_fuel(stack: &PotItemStack) -> bool {
    stack.item_id == "minecraft:blaze_powder" && stack.count > 0
}

pub(super) fn is_brewing_container(item_id: &str) -> bool {
    let (item, potion) = brewing_stack_parts(item_id);
    matches!(
        item,
        "minecraft:potion" | "minecraft:splash_potion" | "minecraft:lingering_potion"
    ) && potion.is_some()
        || item == "minecraft:glass_bottle"
}

pub(super) fn brewing_stack_id(item: &str, potion: &str) -> String {
    format!("{item}#{potion}")
}

pub(super) fn brewing_stack_parts(item_id: &str) -> (&str, Option<&str>) {
    item_id
        .split_once('#')
        .map_or((item_id, None), |(item, potion)| (item, Some(potion)))
}

pub(super) fn sign_line_to_tag(line: &SignLine) -> Tag {
    if let Some(command) = &line.click_command {
        Tag::Compound(vec![
            ("text".to_string(), Tag::String(line.raw.clone())),
            ("run_command".to_string(), Tag::String(command.clone())),
        ])
    } else {
        Tag::String(line.raw.clone())
    }
}

pub(super) fn sign_line_from_tag(tag: &Tag) -> SignLine {
    match tag {
        Tag::String(raw) => SignLine::new(raw.clone(), raw.clone()),
        Tag::Compound(entries) => {
            let raw = get_string(entries, "text").unwrap_or("").to_string();
            let mut line = SignLine::new(raw.clone(), raw);
            line.click_command = get_string(entries, "run_command").map(str::to_string);
            line
        }
        _ => SignLine::default(),
    }
}

pub(super) fn compound_entries(tag: &Tag) -> Option<&Vec<(String, Tag)>> {
    match tag {
        Tag::Compound(entries) => Some(entries),
        _ => None,
    }
}

pub(super) fn get_string<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::String(value) if name == key => Some(value.as_str()),
        _ => None,
    })
}

pub(super) fn get_int(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Int(value) if name == key => Some(*value),
        _ => None,
    })
}

pub(super) fn get_short(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Short(value) if name == key => Some(i32::from(*value)),
        _ => None,
    })
}

pub(super) fn get_long(entries: &[(String, Tag)], key: &str) -> Option<i64> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Long(value) if name == key => Some(*value),
        _ => None,
    })
}

pub(super) fn get_int_array<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a [i32]> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::IntArray(value) if name == key => Some(value.as_slice()),
        _ => None,
    })
}

pub(super) fn string_list_field(entries: &[(String, Tag)], key: &str) -> Option<Vec<String>> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::List(values) if name == key => Some(
            values
                .iter()
                .filter_map(|tag| match tag {
                    Tag::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect(),
        ),
        _ => None,
    })
}

pub(super) fn string_list_tag(values: impl IntoIterator<Item = String>) -> Tag {
    Tag::List(values.into_iter().map(Tag::String).collect())
}

pub(super) fn int_range_field(entries: &[(String, Tag)], key: &str) -> Option<(i32, i32)> {
    let values = get_int_array(entries, key)?;
    (values.len() == 2).then_some((values[0], values[1]))
}

pub(super) fn get_float(entries: &[(String, Tag)], key: &str) -> Option<f32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Float(value) if name == key => Some(*value),
        _ => None,
    })
}

pub(super) fn get_double(entries: &[(String, Tag)], key: &str) -> Option<f64> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Double(value) if name == key => Some(*value),
        _ => None,
    })
}

pub(super) fn get_byte(entries: &[(String, Tag)], key: &str) -> Option<i8> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Byte(value) if name == key => Some(*value),
        _ => None,
    })
}

pub(super) fn get_bool(entries: &[(String, Tag)], key: &str) -> Option<bool> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Byte(value) if name == key => Some(*value != 0),
        _ => None,
    })
}

pub(super) fn block_pos_to_tag(pos: BlockPos) -> Tag {
    Tag::List(vec![Tag::Int(pos.x), Tag::Int(pos.y), Tag::Int(pos.z)])
}

pub(super) fn offset_pos(pos: BlockPos, x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos {
        x: pos.x + x,
        y: pos.y + y,
        z: pos.z + z,
    }
}

pub(super) fn closer_than(left: BlockPos, right: BlockPos, range: f64) -> bool {
    let dx = f64::from(left.x - right.x);
    let dy = f64::from(left.y - right.y);
    let dz = f64::from(left.z - right.z);
    dx * dx + dy * dy + dz * dz < range * range
}

pub(super) fn chessboard_distance(left: BlockPos, right: BlockPos) -> i32 {
    (left.x - right.x)
        .abs()
        .max((left.y - right.y).abs())
        .max((left.z - right.z).abs())
}

pub(super) fn block_pos_from_tag(tag: &Tag) -> Option<BlockPos> {
    match tag {
        Tag::List(values) if values.len() == 3 => Some(BlockPos {
            x: tag_int_or_zero(&values[0]),
            y: tag_int_or_zero(&values[1]),
            z: tag_int_or_zero(&values[2]),
        }),
        _ => None,
    }
}

pub(super) fn vibration_data_from_tag(tag: &Tag) -> VibrationData {
    let Some(entries) = compound_entries(tag) else {
        return VibrationData::new();
    };
    let mut data = VibrationData::new();
    data.travel_time_in_ticks = get_int(entries, "travel_time_in_ticks").unwrap_or(0).max(0);
    data.reload_vibration_particle =
        get_bool(entries, "reload_vibration_particle").unwrap_or(false);
    if let Some(event) = get_string(entries, "event").and_then(crate::game_event::game_event_by_id)
    {
        data.current_vibration = Some(VibrationInfo {
            event,
            distance: get_float(entries, "distance").unwrap_or(0.0),
            pos: crate::entity_physics::Vec3::ZERO,
            source_entity: None,
            projectile_owner: None,
        });
    }
    data
}

pub(super) fn tag_int_or_zero(tag: &Tag) -> i32 {
    match tag {
        Tag::Byte(value) => *value as i32,
        Tag::Short(value) => *value as i32,
        Tag::Int(value) => *value,
        Tag::Long(value) => *value as i32,
        _ => 0,
    }
}

pub(super) fn weighted_spawn_data(values: &[SpawnDataModel], roll: usize) -> Option<&SpawnDataModel> {
    if values.is_empty() {
        return None;
    }
    let total_weight: i32 = values.iter().map(|value| value.weight.max(1)).sum();
    let mut remaining = (roll as i32).rem_euclid(total_weight.max(1));
    for value in values {
        remaining -= value.weight.max(1);
        if remaining < 0 {
            return Some(value);
        }
    }
    values.last()
}

pub(super) fn pot_item_to_tag(item: &PotItemStack, slot: i8) -> Tag {
    let mut tag = match item.to_tag() {
        Tag::Compound(entries) => entries,
        _ => Vec::new(),
    };
    tag.push(("Slot".to_string(), Tag::Byte(slot)));
    Tag::Compound(tag)
}

pub(super) fn pot_item_from_tag(tag: &Tag) -> Option<PotItemStack> {
    PotItemStack::from_tag(tag)
}

pub(super) fn tag_long_or_zero(tag: &Tag) -> i64 {
    match tag {
        Tag::Byte(value) => *value as i64,
        Tag::Short(value) => *value as i64,
        Tag::Int(value) => *value as i64,
        Tag::Long(value) => *value,
        _ => 0,
    }
}

pub(super) fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

pub(super) fn direction_from_name(value: &str) -> Option<Direction> {
    match value {
        "down" => Some(Direction::Down),
        "up" => Some(Direction::Up),
        "north" => Some(Direction::North),
        "south" => Some(Direction::South),
        "west" => Some(Direction::West),
        "east" => Some(Direction::East),
        _ => None,
    }
}
