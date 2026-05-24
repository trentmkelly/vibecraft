use super::*;

impl BeaconBeamSection {
    pub fn new(color: i32) -> Self {
        Self { color, height: 1 }
    }

    pub fn increase_height(&mut self) {
        self.height += 1;
    }
}

impl BeaconBlockEntity {
    pub const MAX_LEVELS: i32 = 4;
    pub const BLOCKS_CHECK_PER_TICK: i32 = 10;
    pub const DEFAULT_NAME: &'static str = "container.beacon";

    pub fn new() -> Self {
        Self {
            levels: 0,
            primary_power: None,
            secondary_power: None,
            custom_name: None,
            lock_key: None,
            payment_item: None,
            beam_sections: Vec::new(),
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(primary) = &self.primary_power {
            entries.push(("primary_effect".to_string(), Tag::String(primary.clone())));
        }
        if let Some(secondary) = &self.secondary_power {
            entries.push((
                "secondary_effect".to_string(),
                Tag::String(secondary.clone()),
            ));
        }
        entries.push(("Levels".to_string(), Tag::Int(self.levels)));
        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        if let Some(lock_key) = &self.lock_key {
            entries.push(("Lock".to_string(), Tag::String(lock_key.clone())));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut beacon = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return beacon;
        };
        beacon.primary_power = get_string(entries, "primary_effect").and_then(filter_beacon_effect);
        beacon.secondary_power =
            get_string(entries, "secondary_effect").and_then(filter_beacon_effect);
        beacon.levels = get_int(entries, "Levels")
            .unwrap_or(0)
            .clamp(0, Self::MAX_LEVELS);
        beacon.custom_name = get_string(entries, "CustomName").map(ToString::to_string);
        beacon.lock_key = get_string(entries, "Lock").map(ToString::to_string);
        beacon
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(Self::DEFAULT_NAME)
    }

    pub fn set_primary_power(&mut self, effect: Option<&str>) {
        self.primary_power = effect.and_then(filter_beacon_effect);
    }

    pub fn set_secondary_power(&mut self, effect: Option<&str>) {
        self.secondary_power = effect.and_then(filter_beacon_effect);
    }

    pub fn can_pay_with(item: &PotItemStack) -> bool {
        matches!(
            item.item_id.as_str(),
            "minecraft:netherite_ingot"
                | "minecraft:emerald"
                | "minecraft:diamond"
                | "minecraft:gold_ingot"
                | "minecraft:iron_ingot"
                | "minecraft:amethyst_shard"
        )
    }

    pub fn set_payment_item(&mut self, item: Option<PotItemStack>) -> bool {
        if item.as_ref().is_some_and(|item| !Self::can_pay_with(item)) {
            return false;
        }
        self.payment_item = item.filter(|item| !item.is_empty());
        true
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "beacon",
            initial_slots: vec![self.payment_item.clone()],
        }
    }

    pub fn comparator_output(&self) -> u8 {
        self.levels.clamp(0, Self::MAX_LEVELS) as u8
    }

    pub fn update_base(
        beacon_pos: BlockPos,
        min_y: i32,
        is_base_block: impl Fn(BlockPos) -> bool,
    ) -> i32 {
        let mut levels = 0;
        for step in 1..=Self::MAX_LEVELS {
            let y = beacon_pos.y - step;
            if y < min_y {
                break;
            }
            let mut ok = true;
            'layer: for x in beacon_pos.x - step..=beacon_pos.x + step {
                for z in beacon_pos.z - step..=beacon_pos.z + step {
                    if !is_base_block(BlockPos { x, y, z }) {
                        ok = false;
                        break 'layer;
                    }
                }
            }
            if !ok {
                break;
            }
            levels = step;
        }
        levels
    }

    pub fn scan_beam(blocks: impl IntoIterator<Item = BeaconBeamBlock>) -> Vec<BeaconBeamSection> {
        let mut sections: Vec<BeaconBeamSection> = Vec::new();
        let mut last: Option<BeaconBeamSection> = None;
        for block in blocks {
            match block {
                BeaconBeamBlock::TintedGlass(color) => {
                    if sections.len() <= 1 {
                        let section = BeaconBeamSection::new(color);
                        sections.push(section.clone());
                        last = Some(section);
                    } else if let Some(current) = last.as_mut() {
                        if current.color == color {
                            current.increase_height();
                            if let Some(stored) = sections.last_mut() {
                                stored.increase_height();
                            }
                        } else {
                            let averaged = average_argb(current.color, color);
                            let section = BeaconBeamSection::new(averaged);
                            sections.push(section.clone());
                            last = Some(section);
                        }
                    }
                }
                BeaconBeamBlock::Transparent | BeaconBeamBlock::Bedrock => {
                    if let Some(stored) = sections.last_mut() {
                        stored.increase_height();
                    } else {
                        let mut section = BeaconBeamSection::new(0xFFFF_FFFFu32 as i32);
                        section.increase_height();
                        sections.push(section.clone());
                        last = Some(section);
                    }
                }
                BeaconBeamBlock::Blocking => return Vec::new(),
            }
        }
        sections
    }

    pub fn effect_applications(&self) -> Vec<BeaconEffectApplication> {
        if self.levels <= 0 {
            return Vec::new();
        }
        let Some(primary) = self.primary_power.as_ref() else {
            return Vec::new();
        };
        let range = self.levels * 10 + 10;
        let duration_ticks = (9 + self.levels * 2) * 20;
        let mut out = vec![BeaconEffectApplication {
            effect: primary.clone(),
            duration_ticks,
            amplifier: if self.levels >= 4 && self.secondary_power.as_ref() == Some(primary) {
                1
            } else {
                0
            },
            range,
        }];
        if self.levels >= 4 {
            if let Some(secondary) = self.secondary_power.as_ref() {
                if secondary != primary {
                    out.push(BeaconEffectApplication {
                        effect: secondary.clone(),
                        duration_ticks,
                        amplifier: 0,
                        range,
                    });
                }
            }
        }
        out
    }
}

pub(super) fn filter_beacon_effect(effect: &str) -> Option<String> {
    matches!(
        effect,
        "minecraft:speed"
            | "minecraft:haste"
            | "minecraft:resistance"
            | "minecraft:jump_boost"
            | "minecraft:strength"
            | "minecraft:regeneration"
    )
    .then(|| effect.to_string())
}

pub(super) fn average_argb(left: i32, right: i32) -> i32 {
    let left = left as u32;
    let right = right as u32;
    let avg = |shift| (((left >> shift) & 0xFFu32) + ((right >> shift) & 0xFFu32)) / 2u32;
    ((avg(24) << 24) | (avg(16) << 16) | (avg(8) << 8) | avg(0)) as i32
}

impl LecternBlockEntity {
    pub const DATA_PAGE: i32 = 0;
    pub const SLOT_BOOK: usize = 0;
    pub const NUM_SLOTS: usize = 1;
    pub const PAGE_CHANGE_IMPULSE_TICKS: i32 = 2;
    pub const DISPLAY_NAME: &'static str = "container.lectern";

    pub fn new() -> Self {
        Self {
            book: None,
            page: 0,
            page_count: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(book) = self.book.as_ref().filter(|book| !book.is_empty()) {
            entries.push(("Book".to_string(), book.to_tag()));
            entries.push(("Page".to_string(), Tag::Int(self.page)));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag, page_count: i32) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let book = entries
            .iter()
            .find(|(key, _)| key == "Book")
            .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        let mut lectern = Self {
            book,
            page: 0,
            page_count: page_count.max(0),
        };
        lectern.page = lectern.clamp_page(get_int(entries, "Page").unwrap_or(0));
        lectern
    }

    pub fn has_book(&self) -> bool {
        self.book.as_ref().is_some_and(|book| {
            !book.is_empty()
                && matches!(
                    book.item_id.as_str(),
                    "minecraft:written_book" | "minecraft:writable_book"
                )
        })
    }

    pub fn set_book(&mut self, book: Option<PotItemStack>, page_count: i32) {
        self.book = book.filter(|book| !book.is_empty());
        self.page = 0;
        self.page_count = if self.has_book() {
            page_count.max(0)
        } else {
            0
        };
    }

    pub fn clear_content(&mut self) {
        self.book = None;
        self.page = 0;
        self.page_count = 0;
    }

    pub fn set_page(&mut self, page: i32) -> bool {
        let new_page = self.clamp_page(page);
        let changed = self.page != new_page;
        self.page = new_page;
        changed
    }

    pub fn remove_book_no_update(&mut self) -> Option<PotItemStack> {
        let book = self.book.take();
        self.page = 0;
        self.page_count = 0;
        book
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "lectern",
            initial_slots: vec![self.book.clone()],
        }
    }

    pub fn get_redstone_signal(&self) -> u8 {
        if !self.has_book() {
            return 0;
        }
        let progress = if self.page_count > 1 {
            self.page as f32 / (self.page_count as f32 - 1.0)
        } else {
            1.0
        };
        (progress * 14.0).floor() as u8 + 1
    }

    pub(super) fn clamp_page(&self, page: i32) -> i32 {
        if self.page_count <= 0 {
            0
        } else {
            page.clamp(0, self.page_count - 1)
        }
    }
}

impl HangingSignAttachment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wall => "wall",
            Self::Ceiling => "ceiling",
            Self::CeilingMiddle => "ceiling_middle",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "wall" => Some(Self::Wall),
            "ceiling" => Some(Self::Ceiling),
            "ceiling_middle" => Some(Self::CeilingMiddle),
            _ => None,
        }
    }
}

impl Default for SignLine {
    fn default() -> Self {
        Self {
            raw: String::new(),
            filtered: String::new(),
            click_command: None,
        }
    }
}

impl SignLine {
    pub fn new(raw: impl Into<String>, filtered: impl Into<String>) -> Self {
        Self {
            raw: raw.into(),
            filtered: filtered.into(),
            click_command: None,
        }
    }

    pub fn with_click_command(mut self, command: impl Into<String>) -> Self {
        self.click_command = Some(command.into());
        self
    }

    pub fn visible_text(&self, should_filter: bool) -> &str {
        if should_filter {
            &self.filtered
        } else {
            &self.raw
        }
    }
}

impl Default for SignText {
    fn default() -> Self {
        Self {
            lines: std::array::from_fn(|_| SignLine::default()),
            color: DyeColor::Black,
            has_glowing_text: false,
        }
    }
}

impl SignText {
    pub const LINES: usize = 4;

    pub fn set_message(
        &mut self,
        index: usize,
        raw: impl Into<String>,
        filtered: impl Into<String>,
    ) -> bool {
        if index >= Self::LINES {
            return false;
        }
        self.lines[index] = SignLine::new(raw, filtered);
        true
    }

    pub fn has_message(&self, should_filter: bool) -> bool {
        self.lines
            .iter()
            .any(|line| !line.visible_text(should_filter).is_empty())
    }

    pub fn has_any_click_commands(&self, should_filter: bool) -> bool {
        self.lines.iter().any(|line| {
            !line.visible_text(should_filter).is_empty() && line.click_command.is_some()
        })
    }

    pub fn to_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "messages".to_string(),
                Tag::List(self.lines.iter().map(sign_line_to_tag).collect()),
            ),
            (
                "color".to_string(),
                Tag::String(self.color.vanilla_name().to_string()),
            ),
            (
                "has_glowing_text".to_string(),
                Tag::Byte(self.has_glowing_text as i8),
            ),
        ];
        if self.lines.iter().any(|line| line.filtered != line.raw) {
            fields.push((
                "filtered_messages".to_string(),
                Tag::List(
                    self.lines
                        .iter()
                        .map(|line| Tag::String(line.filtered.clone()))
                        .collect(),
                ),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn from_tag(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::default();
        };
        let mut text = Self::default();
        if let Some(Tag::List(messages)) = entries
            .iter()
            .find(|(name, _)| name == "messages")
            .map(|(_, tag)| tag)
        {
            for (index, message) in messages.iter().take(Self::LINES).enumerate() {
                text.lines[index] = sign_line_from_tag(message);
            }
        }
        if let Some(Tag::List(filtered_messages)) = entries
            .iter()
            .find(|(name, _)| name == "filtered_messages")
            .map(|(_, tag)| tag)
        {
            for (index, message) in filtered_messages.iter().take(Self::LINES).enumerate() {
                if let Tag::String(filtered) = message {
                    text.lines[index].filtered = filtered.clone();
                }
            }
        } else {
            for line in &mut text.lines {
                line.filtered = line.raw.clone();
            }
        }
        text.color = get_string(entries, "color")
            .and_then(DyeColor::from_vanilla_name)
            .unwrap_or(DyeColor::Black);
        text.has_glowing_text = get_bool(entries, "has_glowing_text").unwrap_or(false);
        text
    }
}

impl Default for SignBlockEntityModel {
    fn default() -> Self {
        Self {
            front_text: SignText::default(),
            back_text: SignText::default(),
            is_waxed: false,
            player_who_may_edit: None,
        }
    }
}

impl SignBlockEntityModel {
    pub const MAX_TEXT_LINE_WIDTH: i32 = 90;
    pub const TEXT_LINE_HEIGHT: i32 = 10;

    pub fn text(&self, front_text: bool) -> &SignText {
        if front_text {
            &self.front_text
        } else {
            &self.back_text
        }
    }

    pub fn text_mut(&mut self, front_text: bool) -> &mut SignText {
        if front_text {
            &mut self.front_text
        } else {
            &mut self.back_text
        }
    }

    pub fn set_allowed_player_editor(&mut self, player_uuid: Option<String>) {
        self.player_who_may_edit = player_uuid;
    }

    pub fn player_is_too_far_away_to_edit(&self, player_uuid: &str, distance: f64) -> bool {
        self.player_who_may_edit.as_deref() != Some(player_uuid) || distance > 4.0
    }

    pub fn tick_editing_player(&mut self, player_uuid: &str, distance: f64) -> bool {
        if self.player_is_too_far_away_to_edit(player_uuid, distance) {
            self.player_who_may_edit = None;
            true
        } else {
            false
        }
    }

    pub fn update_sign_text(
        &mut self,
        player_uuid: &str,
        front_text: bool,
        lines: [SignLine; 4],
        player_filters_text: bool,
    ) -> bool {
        if self.is_waxed || self.player_who_may_edit.as_deref() != Some(player_uuid) {
            return false;
        }
        let text = self.text_mut(front_text);
        if player_filters_text {
            for (slot, line) in lines.into_iter().enumerate() {
                text.lines[slot].raw = line.filtered.clone();
                text.lines[slot].filtered = line.filtered;
                text.lines[slot].click_command = line.click_command;
            }
        } else {
            text.lines = lines;
        }
        self.player_who_may_edit = None;
        true
    }

    pub fn set_waxed(&mut self, is_waxed: bool) -> bool {
        if self.is_waxed == is_waxed {
            false
        } else {
            self.is_waxed = is_waxed;
            true
        }
    }

    pub fn can_execute_click_commands(&self, front_text: bool, should_filter: bool) -> bool {
        self.is_waxed && self.text(front_text).has_any_click_commands(should_filter)
    }

    pub fn executable_click_commands(&self, front_text: bool, should_filter: bool) -> Vec<String> {
        if !self.is_waxed {
            return Vec::new();
        }
        self.text(front_text)
            .lines
            .iter()
            .filter(|line| !line.visible_text(should_filter).is_empty())
            .filter_map(|line| line.click_command.clone())
            .collect()
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("front_text".to_string(), self.front_text.to_tag()),
            ("back_text".to_string(), self.back_text.to_tag()),
            ("is_waxed".to_string(), Tag::Byte(self.is_waxed as i8)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::default();
        };
        Self {
            front_text: entries
                .iter()
                .find(|(name, _)| name == "front_text")
                .map(|(_, tag)| SignText::from_tag(tag))
                .unwrap_or_default(),
            back_text: entries
                .iter()
                .find(|(name, _)| name == "back_text")
                .map(|(_, tag)| SignText::from_tag(tag))
                .unwrap_or_default(),
            is_waxed: get_bool(entries, "is_waxed").unwrap_or(false),
            player_who_may_edit: None,
        }
    }
}

impl HangingSignBlockEntityModel {
    pub const MAX_TEXT_LINE_WIDTH: i32 = 60;
    pub const TEXT_LINE_HEIGHT: i32 = 9;

    pub fn new(attachment: HangingSignAttachment) -> Self {
        Self {
            sign: SignBlockEntityModel::default(),
            attachment,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = match self.sign.save_additional() {
            Tag::Compound(entries) => entries,
            _ => Vec::new(),
        };
        entries.push((
            "attachment".to_string(),
            Tag::String(self.attachment.as_str().to_string()),
        ));
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let sign = SignBlockEntityModel::load_additional(tag);
        let attachment = compound_entries(tag)
            .and_then(|entries| get_string(entries, "attachment"))
            .and_then(HangingSignAttachment::from_str)
            .unwrap_or(HangingSignAttachment::Ceiling);
        Self { sign, attachment }
    }
}

impl BrewingRecipe {
    pub const fn new(
        source_item: &'static str,
        source_potion: &'static str,
        ingredient: &'static str,
        result_item: &'static str,
        result_potion: &'static str,
    ) -> Self {
        Self {
            source_item,
            source_potion,
            ingredient,
            result_item,
            result_potion,
        }
    }

    pub(super) fn applies_to(&self, stack: &PotItemStack, ingredient: &PotItemStack) -> bool {
        let (item, potion) = brewing_stack_parts(&stack.item_id);
        item == self.source_item
            && potion == Some(self.source_potion)
            && ingredient.item_id == self.ingredient
    }

    pub(super) fn result_stack(&self, count: i32) -> PotItemStack {
        PotItemStack {
            item_id: brewing_stack_id(self.result_item, self.result_potion),
            count,
        }
    }
}

impl BrewingStandBlockEntity {
    pub const CONTAINER_SIZE: usize = 5;
    pub const INGREDIENT_SLOT: usize = 3;
    pub const FUEL_SLOT: usize = 4;
    pub const FUEL_USES: i32 = 20;
    pub const BREW_TIME: i32 = 400;
    pub const DATA_BREW_TIME: i32 = 0;
    pub const DATA_FUEL_USES: i32 = 1;
    pub const DISPLAY_NAME: &'static str = "container.brewing";

    pub fn new() -> Self {
        Self {
            items: vec![None; Self::CONTAINER_SIZE],
            brew_time: 0,
            fuel: 0,
            ingredient: None,
        }
    }

    pub fn set_item(&mut self, slot: usize, stack: Option<PotItemStack>) -> bool {
        if slot >= Self::CONTAINER_SIZE {
            return false;
        }
        self.items[slot] = stack.filter(|stack| !stack.is_empty());
        true
    }

    pub fn potion_bits(&self) -> [bool; 3] {
        [
            self.items[0].is_some(),
            self.items[1].is_some(),
            self.items[2].is_some(),
        ]
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "brewing_stand",
            initial_slots: self.items.clone(),
        }
    }

    pub fn is_brewable(&self, recipes: &[BrewingRecipe]) -> bool {
        let Some(ingredient) = self.items[Self::INGREDIENT_SLOT].as_ref() else {
            return false;
        };
        recipes.iter().any(|recipe| {
            self.items[..3]
                .iter()
                .flatten()
                .any(|stack| recipe.applies_to(stack, ingredient))
        })
    }

    pub fn server_tick(&mut self, recipes: &[BrewingRecipe]) -> BrewingStandTickResult {
        if self.fuel <= 0
            && self.items[Self::FUEL_SLOT]
                .as_ref()
                .is_some_and(is_brewing_fuel)
        {
            self.fuel = Self::FUEL_USES;
            shrink_stack(&mut self.items[Self::FUEL_SLOT], 1);
            return BrewingStandTickResult::FuelLoaded;
        }

        let brewable = self.is_brewable(recipes);
        let ingredient_id = self.items[Self::INGREDIENT_SLOT]
            .as_ref()
            .map(|stack| stack.item_id.clone());
        if self.brew_time > 0 {
            self.brew_time -= 1;
            if self.brew_time == 0 && brewable {
                self.do_brew(recipes);
                return BrewingStandTickResult::Brewed;
            }
            if !brewable || ingredient_id != self.ingredient {
                self.brew_time = 0;
                return BrewingStandTickResult::Cancelled;
            }
            return BrewingStandTickResult::Brewing;
        }

        if brewable && self.fuel > 0 {
            self.fuel -= 1;
            self.brew_time = Self::BREW_TIME;
            self.ingredient = ingredient_id;
            return BrewingStandTickResult::Started;
        }

        BrewingStandTickResult::Idle
    }

    pub(super) fn do_brew(&mut self, recipes: &[BrewingRecipe]) {
        let Some(ingredient) = self.items[Self::INGREDIENT_SLOT].as_ref().cloned() else {
            return;
        };
        for slot in 0..3 {
            let Some(stack) = self.items[slot].as_ref() else {
                continue;
            };
            if let Some(recipe) = recipes
                .iter()
                .find(|recipe| recipe.applies_to(stack, &ingredient))
            {
                self.items[slot] = Some(recipe.result_stack(stack.count));
            }
        }
        shrink_stack(&mut self.items[Self::INGREDIENT_SLOT], 1);
        self.ingredient = self.items[Self::INGREDIENT_SLOT]
            .as_ref()
            .map(|stack| stack.item_id.clone());
    }

    pub fn can_place_item(
        &self,
        slot: usize,
        stack: &PotItemStack,
        recipes: &[BrewingRecipe],
    ) -> bool {
        match slot {
            Self::INGREDIENT_SLOT => recipes
                .iter()
                .any(|recipe| recipe.ingredient == stack.item_id),
            Self::FUEL_SLOT => is_brewing_fuel(stack),
            0..=2 => {
                is_brewing_container(&stack.item_id)
                    && self.items.get(slot).is_some_and(Option::is_none)
            }
            _ => false,
        }
    }

    pub fn slots_for_face(direction: Direction) -> &'static [usize] {
        match direction {
            Direction::Up => &[Self::INGREDIENT_SLOT],
            Direction::Down => &[0, 1, 2, Self::INGREDIENT_SLOT],
            _ => &[0, 1, 2, Self::FUEL_SLOT],
        }
    }

    pub fn can_take_item_through_face(
        slot: usize,
        stack: &PotItemStack,
        _direction: Direction,
    ) -> bool {
        slot != Self::INGREDIENT_SLOT || stack.item_id == "minecraft:glass_bottle"
    }

    pub fn comparator_output(&self) -> u8 {
        inventory_comparator_output(&self.items)
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("BrewTime".to_string(), Tag::Short(self.brew_time as i16)),
            ("Items".to_string(), container_items_tag(&self.items)),
            ("Fuel".to_string(), Tag::Byte(self.fuel as i8)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut stand = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return stand;
        };
        load_container_items(entries, &mut stand.items);
        stand.brew_time = get_short(entries, "BrewTime").unwrap_or(0) as i32;
        stand.fuel = get_byte(entries, "Fuel").unwrap_or(0) as i32;
        if stand.brew_time > 0 {
            stand.ingredient = stand.items[Self::INGREDIENT_SLOT]
                .as_ref()
                .map(|stack| stack.item_id.clone());
        }
        stand
    }
}

impl CrafterRecipe {
    pub fn matches(&self, crafter: &CrafterBlockEntity) -> bool {
        self.pattern.iter().enumerate().all(|(slot, expected)| {
            if crafter.disabled_slots[slot] {
                expected.is_none()
            } else {
                match (expected, crafter.items[slot].as_ref()) {
                    (Some(expected), Some(stack)) => stack.item_id == *expected && stack.count > 0,
                    (None, None) => true,
                    _ => false,
                }
            }
        })
    }
}

impl CrafterBlockEntity {
    pub const CONTAINER_WIDTH: usize = 3;
    pub const CONTAINER_HEIGHT: usize = 3;
    pub const CONTAINER_SIZE: usize = 9;
    pub const DATA_TRIGGERED: usize = 9;
    pub const NUM_DATA: usize = 10;
    pub const SLOT_DISABLED: i32 = 1;
    pub const SLOT_ENABLED: i32 = 0;
    pub const MAX_STACK_SIZE: i32 = 64;
    pub const MAX_CRAFTING_TICKS: i32 = 6;
    pub const CRAFTING_TICK_DELAY: i32 = 4;
    pub const DISPLAY_NAME: &'static str = "container.crafter";

    pub fn new() -> Self {
        Self {
            items: vec![None; Self::CONTAINER_SIZE],
            disabled_slots: [false; Self::CONTAINER_SIZE],
            triggered: false,
            crafting_ticks_remaining: 0,
        }
    }

    pub fn set_slot_state(&mut self, slot: usize, enabled: bool) -> bool {
        if !self.slot_can_be_disabled(slot) {
            return false;
        }
        self.disabled_slots[slot] = !enabled;
        true
    }

    pub fn is_slot_disabled(&self, slot: usize) -> bool {
        self.disabled_slots.get(slot).copied().unwrap_or(false)
    }

    pub fn set_triggered(&mut self, triggered: bool) {
        self.triggered = triggered;
    }

    pub fn set_item(&mut self, slot: usize, stack: Option<PotItemStack>) -> bool {
        if slot >= Self::CONTAINER_SIZE {
            return false;
        }
        if self.is_slot_disabled(slot) {
            self.set_slot_state(slot, true);
        }
        self.items[slot] = stack.filter(|stack| !stack.is_empty());
        true
    }

    pub fn can_place_item(&self, slot: usize, stack: &PotItemStack) -> bool {
        if slot >= Self::CONTAINER_SIZE || self.is_slot_disabled(slot) || stack.is_empty() {
            return false;
        }
        let Some(slot_stack) = self.items[slot].as_ref() else {
            return true;
        };
        if slot_stack.count >= Self::MAX_STACK_SIZE {
            return false;
        }
        !self.smaller_stack_exists(slot_stack.count, slot_stack, slot)
    }

    pub(super) fn smaller_stack_exists(
        &self,
        base_size: i32,
        base_item: &PotItemStack,
        base_slot: usize,
    ) -> bool {
        for slot in (base_slot + 1)..Self::CONTAINER_SIZE {
            if self.is_slot_disabled(slot) {
                continue;
            }
            match self.items[slot].as_ref() {
                None => return true,
                Some(stack) if stack.count < base_size && stack.item_id == base_item.item_id => {
                    return true
                }
                _ => {}
            }
        }
        false
    }

    pub(super) fn slot_can_be_disabled(&self, slot: usize) -> bool {
        slot < Self::CONTAINER_SIZE && self.items[slot].is_none()
    }

    pub fn redstone_signal(&self) -> u8 {
        self.items
            .iter()
            .zip(self.disabled_slots)
            .filter(|(stack, disabled)| stack.is_some() || *disabled)
            .count() as u8
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        let mut initial_slots = self.items.clone();
        initial_slots.push(None);
        BlockEntityMenuOpen {
            container_id,
            menu_type: "crafter_3x3",
            initial_slots,
        }
    }

    pub fn server_tick(&mut self) -> bool {
        let next = self.crafting_ticks_remaining - 1;
        if next >= 0 {
            self.crafting_ticks_remaining = next;
            next == 0
        } else {
            false
        }
    }

    pub fn pulse_craft(&mut self, recipes: &[CrafterRecipe]) -> CrafterPulseResult {
        if !self.triggered {
            return CrafterPulseResult::NotTriggered;
        }
        let Some(recipe) = recipes.iter().find(|recipe| recipe.matches(self)) else {
            return CrafterPulseResult::NoRecipe;
        };
        self.crafting_ticks_remaining = Self::MAX_CRAFTING_TICKS;
        for stack in &mut self.items {
            if stack.is_some() {
                shrink_stack(stack, 1);
            }
        }
        CrafterPulseResult::Crafted {
            result: recipe.result.clone(),
            remaining_items: recipe.remaining_items.clone(),
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "crafting_ticks_remaining".to_string(),
                Tag::Int(self.crafting_ticks_remaining),
            ),
            ("Items".to_string(), container_items_tag(&self.items)),
            (
                "disabled_slots".to_string(),
                Tag::IntArray(
                    self.disabled_slots
                        .iter()
                        .enumerate()
                        .filter_map(|(slot, disabled)| disabled.then_some(slot as i32))
                        .collect(),
                ),
            ),
            ("triggered".to_string(), Tag::Int(self.triggered as i32)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut crafter = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return crafter;
        };
        load_container_items(entries, &mut crafter.items);
        crafter.crafting_ticks_remaining =
            get_int(entries, "crafting_ticks_remaining").unwrap_or(0);
        if let Some(Tag::IntArray(disabled_slots)) = entries
            .iter()
            .find(|(name, _)| name == "disabled_slots")
            .map(|(_, tag)| tag)
        {
            for slot in disabled_slots {
                if (0..Self::CONTAINER_SIZE as i32).contains(slot) {
                    crafter.disabled_slots[*slot as usize] = true;
                }
            }
        }
        crafter.triggered = get_int(entries, "triggered").unwrap_or(0) != 0;
        crafter
    }
}
