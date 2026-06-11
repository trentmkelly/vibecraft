use super::*;

impl VaultStateModel {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Unlocking => "unlocking",
            Self::Ejecting => "ejecting",
        }
    }

    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "inactive" => Some(Self::Inactive),
            "active" => Some(Self::Active),
            "unlocking" => Some(Self::Unlocking),
            "ejecting" => Some(Self::Ejecting),
            _ => None,
        }
    }

    pub fn light_level(self) -> i32 {
        match self {
            Self::Inactive => 6,
            Self::Active | Self::Unlocking | Self::Ejecting => 12,
        }
    }
}

impl Default for VaultConfigModel {
    fn default() -> Self {
        Self {
            loot_table: "minecraft:chests/trial_chambers/reward".to_string(),
            activation_range: 4.0,
            deactivation_range: 4.5,
            key_item: PotItemStack {
                item_id: "minecraft:trial_key".to_string(),
                count: 1,
            },
            override_loot_table_to_display: None,
        }
    }
}

impl VaultConfigModel {
    pub fn validate(&self) -> Result<(), String> {
        if self.activation_range > self.deactivation_range {
            Err(format!(
                "Activation range must ({:?}) be less or equal to deactivation range ({:?})",
                self.activation_range, self.deactivation_range
            ))
        } else {
            Ok(())
        }
    }

    pub(super) fn to_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "loot_table".to_string(),
                Tag::String(self.loot_table.clone()),
            ),
            (
                "activation_range".to_string(),
                Tag::Double(self.activation_range),
            ),
            (
                "deactivation_range".to_string(),
                Tag::Double(self.deactivation_range),
            ),
            ("key_item".to_string(), pot_item_to_tag(&self.key_item, 0)),
        ];
        if let Some(table) = &self.override_loot_table_to_display {
            fields.push((
                "override_loot_table_to_display".to_string(),
                Tag::String(table.clone()),
            ));
        }
        Tag::Compound(fields)
    }

    pub(super) fn from_tag(tag: &Tag) -> Self {
        let mut config = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return config;
        };
        config.loot_table = get_string(entries, "loot_table")
            .unwrap_or(&config.loot_table)
            .to_string();
        config.activation_range =
            get_double(entries, "activation_range").unwrap_or(config.activation_range);
        config.deactivation_range =
            get_double(entries, "deactivation_range").unwrap_or(config.deactivation_range);
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "key_item") {
            config.key_item = pot_item_from_tag(tag).unwrap_or(config.key_item);
        }
        config.override_loot_table_to_display =
            get_string(entries, "override_loot_table_to_display").map(ToString::to_string);
        config
    }
}

impl Default for VaultBlockEntity {
    fn default() -> Self {
        Self {
            state: VaultStateModel::Inactive,
            is_ominous: false,
            config: VaultConfigModel::default(),
            rewarded_players: BTreeSet::new(),
            connected_players: BTreeSet::new(),
            display_item: None,
            items_to_eject: Vec::new(),
            total_ejections_needed: 0,
            state_updating_resumes_at: 0,
            last_insert_fail_timestamp: 0,
            connected_particles_range: VaultConfigModel::default().deactivation_range,
            current_spin: 0.0,
            previous_spin: 0.0,
        }
    }
}

impl VaultBlockEntity {
    pub const UNLOCKING_DELAY_TICKS: i64 = 14;
    pub const STATE_UPDATE_RATE_TICKS: i64 = 20;
    pub const DISPLAY_CYCLE_TICK_RATE: i64 = 20;
    pub const INSERT_FAIL_SOUND_BUFFER_TICKS: i64 = 15;
    pub const CLIENT_ROTATION_SPEED: f32 = 10.0;
    pub const CLIENT_PARTICLE_TICK_RATE: i64 = 20;
    pub const CLIENT_IDLE_PARTICLE_CHANCE: f32 = 0.5;
    pub const CLIENT_AMBIENT_SOUND_CHANCE: f32 = 0.02;
    pub const ACTIVATION_PARTICLE_COUNT: i32 = 20;
    pub const DEACTIVATION_PARTICLE_COUNT: i32 = 20;
    pub const MAX_REWARDED_PLAYERS: usize = 128;

    // TODO(vault-live-level): wire Java's live ServerLevel/client operations for loot-table
    // resolution, player stat/key consumption, block-state mutation, level events, sounds,
    // spawned reward items, and particle emission once those runtime systems are unified.

    pub fn tick_client(&mut self) {
        self.previous_spin = self.current_spin;
        self.current_spin = (self.current_spin + Self::CLIENT_ROTATION_SPEED).rem_euclid(360.0);
    }

    pub fn should_cycle_display_item(game_time: i64, state: VaultStateModel) -> bool {
        game_time % Self::DISPLAY_CYCLE_TICK_RATE == 0 && state == VaultStateModel::Active
    }

    pub fn can_eject_reward(&self) -> bool {
        self.config.key_item.count > 0
            && !self.config.key_item.item_id.is_empty()
            && self.state != VaultStateModel::Inactive
    }

    pub fn ejection_progress(&self) -> f32 {
        if self.total_ejections_needed <= 0 {
            return 0.0;
        }
        if self.total_ejections_needed == 1 {
            return 1.0;
        }
        let remaining = self.items_to_eject.len() as f32;
        1.0 - ((remaining - 1.0) / (self.total_ejections_needed as f32 - 1.0))
    }

    pub fn display_active_effects(&self) -> bool {
        self.display_item.is_some()
    }

    pub fn keyhole_position(pos: BlockPos, facing: Direction) -> (f64, f64, f64) {
        let (step_x, step_z) = match facing {
            Direction::West => (-1.0, 0.0),
            Direction::East => (1.0, 0.0),
            Direction::North => (0.0, -1.0),
            Direction::South => (0.0, 1.0),
            Direction::Down | Direction::Up => (0.0, 0.0),
        };
        (
            pos.x as f64 + 0.5 + step_x * 0.5,
            pos.y as f64 + 1.75,
            pos.z as f64 + 0.5 + step_z * 0.5,
        )
    }

    pub fn random_pos_center_of_cage(pos: BlockPos, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        (
            pos.x as f64 + 0.4 + x * 0.2,
            pos.y as f64 + 0.4 + y * 0.2,
            pos.z as f64 + 0.4 + z * 0.2,
        )
    }

    pub fn random_pos_inside_cage(pos: BlockPos, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        (
            pos.x as f64 + 0.1 + x * 0.8,
            pos.y as f64 + 0.25 + y * 0.5,
            pos.z as f64 + 0.1 + z * 0.8,
        )
    }

    pub fn tick_server(
        &mut self,
        game_time: i64,
        detected_players: &[String],
        display_roll: Option<PotItemStack>,
    ) -> VaultTickResult {
        let cycled_display = Self::should_cycle_display_item(game_time, self.state);
        if cycled_display {
            self.display_item = display_roll;
        }
        if game_time < self.state_updating_resumes_at {
            return if cycled_display {
                VaultTickResult::DisplayItemCycled(self.display_item.clone())
            } else {
                VaultTickResult::Waiting
            };
        }
        let result = match self.state {
            VaultStateModel::Inactive => {
                self.update_connected_players(detected_players, self.config.activation_range);
                self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                if self.connected_players.is_empty() {
                    VaultTickResult::Waiting
                } else {
                    self.state = VaultStateModel::Active;
                    VaultTickResult::StateChanged(self.state)
                }
            }
            VaultStateModel::Active => {
                self.update_connected_players(detected_players, self.config.deactivation_range);
                self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                if self.connected_players.is_empty() {
                    self.state = VaultStateModel::Inactive;
                    self.display_item = None;
                    VaultTickResult::StateChanged(self.state)
                } else {
                    VaultTickResult::Waiting
                }
            }
            VaultStateModel::Unlocking => {
                self.state = VaultStateModel::Ejecting;
                self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                VaultTickResult::StateChanged(self.state)
            }
            VaultStateModel::Ejecting => {
                if let Some(item) = self.items_to_eject.pop() {
                    self.display_item = self.items_to_eject.last().cloned();
                    self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                    VaultTickResult::EjectedItem(item)
                } else {
                    self.total_ejections_needed = 0;
                    self.update_connected_players(detected_players, self.config.deactivation_range);
                    self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                    self.state = if self.connected_players.is_empty() {
                        VaultStateModel::Inactive
                    } else {
                        VaultStateModel::Active
                    };
                    VaultTickResult::EjectionFinished
                }
            }
        };
        if result == VaultTickResult::Waiting && cycled_display {
            VaultTickResult::DisplayItemCycled(self.display_item.clone())
        } else {
            result
        }
    }

    pub fn try_insert_key(
        &mut self,
        player: impl Into<String>,
        inserted: &PotItemStack,
        rewards: Vec<PotItemStack>,
        game_time: i64,
    ) -> VaultInsertResult {
        if self.state == VaultStateModel::Inactive {
            return VaultInsertResult::IgnoredInactive;
        }
        if inserted.item_id != self.config.key_item.item_id
            || inserted.count < self.config.key_item.count
        {
            if game_time >= self.last_insert_fail_timestamp + Self::INSERT_FAIL_SOUND_BUFFER_TICKS {
                self.last_insert_fail_timestamp = game_time;
            }
            return VaultInsertResult::WrongKey {
                expected: self.config.key_item.item_id.clone(),
            };
        }
        let player = player.into();
        if self.rewarded_players.contains(&player) {
            if game_time >= self.last_insert_fail_timestamp + Self::INSERT_FAIL_SOUND_BUFFER_TICKS {
                self.last_insert_fail_timestamp = game_time;
            }
            return VaultInsertResult::AlreadyRewarded;
        }
        if rewards.is_empty() {
            return VaultInsertResult::EmptyReward;
        }
        self.items_to_eject = rewards;
        self.total_ejections_needed = self.items_to_eject.len() as i32;
        self.display_item = self.items_to_eject.last().cloned();
        self.state = VaultStateModel::Unlocking;
        self.state_updating_resumes_at = game_time + Self::UNLOCKING_DELAY_TICKS;
        self.add_rewarded_player(player);
        VaultInsertResult::Unlocking {
            items_to_eject: self.items_to_eject.len(),
        }
    }

    pub(super) fn add_rewarded_player(&mut self, player: String) {
        self.rewarded_players.insert(player);
        while self.rewarded_players.len() > Self::MAX_REWARDED_PLAYERS {
            if let Some(first) = self.rewarded_players.iter().next().cloned() {
                self.rewarded_players.remove(&first);
            }
        }
    }

    pub(super) fn update_connected_players(&mut self, detected_players: &[String], _range: f64) {
        self.connected_players = detected_players
            .iter()
            .filter(|player| !self.rewarded_players.contains(*player))
            .cloned()
            .collect();
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "state".to_string(),
                Tag::String(self.state.as_str().to_string()),
            ),
            (
                "is_ominous".to_string(),
                Tag::Byte(i8::from(self.is_ominous)),
            ),
            ("config".to_string(), self.config.to_tag()),
            ("shared_data".to_string(), self.shared_data_tag()),
            ("server_data".to_string(), self.server_data_tag()),
        ])
    }

    pub fn get_update_tag(&self) -> Tag {
        Tag::Compound(vec![("shared_data".to_string(), self.shared_data_tag())])
    }

    pub(super) fn shared_data_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "connected_players".to_string(),
                string_list_tag(self.connected_players.iter().cloned()),
            ),
            (
                "connected_particles_range".to_string(),
                Tag::Double(self.connected_particles_range),
            ),
        ];
        if let Some(item) = &self.display_item {
            fields.push(("display_item".to_string(), pot_item_to_tag(item, 0)));
        }
        Tag::Compound(fields)
    }

    pub(super) fn server_data_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "rewarded_players".to_string(),
                string_list_tag(self.rewarded_players.iter().cloned()),
            ),
            (
                "state_updating_resumes_at".to_string(),
                Tag::Long(self.state_updating_resumes_at),
            ),
            (
                "items_to_eject".to_string(),
                Tag::List(
                    self.items_to_eject
                        .iter()
                        .enumerate()
                        .map(|(slot, item)| pot_item_to_tag(item, slot as i8))
                        .collect(),
                ),
            ),
            (
                "total_ejections_needed".to_string(),
                Tag::Int(self.total_ejections_needed),
            ),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut vault = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return vault;
        };
        vault.state = get_string(entries, "state")
            .and_then(VaultStateModel::from_str)
            .unwrap_or(VaultStateModel::Inactive);
        vault.is_ominous = get_byte(entries, "is_ominous").unwrap_or(0) != 0;
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "config") {
            vault.config = VaultConfigModel::from_tag(tag);
        }
        if let Some(shared) = entries
            .iter()
            .find(|(name, _)| name == "shared_data")
            .and_then(|(_, tag)| compound_entries(tag))
        {
            vault.connected_players = string_list_field(shared, "connected_players")
                .unwrap_or_default()
                .into_iter()
                .collect();
            vault.connected_particles_range = get_double(shared, "connected_particles_range")
                .unwrap_or(vault.config.deactivation_range);
            vault.display_item = shared
                .iter()
                .find(|(name, _)| name == "display_item")
                .and_then(|(_, tag)| pot_item_from_tag(tag));
        }
        if let Some(server) = entries
            .iter()
            .find(|(name, _)| name == "server_data")
            .and_then(|(_, tag)| compound_entries(tag))
        {
            vault.rewarded_players = string_list_field(server, "rewarded_players")
                .unwrap_or_default()
                .into_iter()
                .collect();
            vault.state_updating_resumes_at =
                get_long(server, "state_updating_resumes_at").unwrap_or(0);
            vault.total_ejections_needed = get_int(server, "total_ejections_needed").unwrap_or(0);
            if let Some(Tag::List(items)) = server
                .iter()
                .find(|(name, _)| name == "items_to_eject")
                .map(|(_, tag)| tag)
            {
                vault.items_to_eject = items.iter().filter_map(pot_item_from_tag).collect();
            }
        }
        vault
    }
}

impl BannerPatternLayer {
    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("pattern".to_string(), Tag::String(self.pattern.clone())),
            (
                "color".to_string(),
                Tag::String(self.color.vanilla_name().to_string()),
            ),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            pattern: get_string(entries, "pattern")?.to_string(),
            color: DyeColor::from_vanilla_name(get_string(entries, "color")?)?,
        })
    }
}

impl BannerBlockEntity {
    pub const MAX_PATTERNS: usize = 6;

    pub fn from_block_state(block_state: &str) -> Option<Self> {
        let name = block_state.strip_prefix("minecraft:")?;
        let color_name = name
            .strip_suffix("_wall_banner")
            .or_else(|| name.strip_suffix("_banner"))?;
        Some(Self {
            base_color: DyeColor::from_vanilla_name(color_name)?,
            patterns: Vec::new(),
            custom_name: None,
        })
    }

    pub fn add_pattern(&mut self, pattern: impl Into<String>, color: DyeColor) -> bool {
        if self.patterns.len() >= Self::MAX_PATTERNS {
            return false;
        }
        self.patterns.push(BannerPatternLayer {
            pattern: pattern.into(),
            color,
        });
        true
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if !self.patterns.is_empty() {
            fields.push((
                "patterns".to_string(),
                Tag::List(
                    self.patterns
                        .iter()
                        .map(BannerPatternLayer::to_tag)
                        .collect(),
                ),
            ));
        }
        if let Some(custom_name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(block_state: &str, tag: &Tag) -> Option<Self> {
        let mut banner = Self::from_block_state(block_state)?;
        let entries = compound_entries(tag);
        banner.custom_name = entries
            .and_then(|entries| get_string(entries, "CustomName"))
            .map(ToString::to_string);
        banner.patterns = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "patterns"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(BannerPatternLayer::from_tag)
                        .take(Self::MAX_PATTERNS)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Some(banner)
    }
}

impl PotDecorations {
    const BRICK: &'static str = "minecraft:brick";

    pub fn new(
        back: Option<String>,
        left: Option<String>,
        right: Option<String>,
        front: Option<String>,
    ) -> Self {
        Self {
            back: Self::normalize_side(back),
            left: Self::normalize_side(left),
            right: Self::normalize_side(right),
            front: Self::normalize_side(front),
        }
    }

    pub(super) fn normalize_side(side: Option<String>) -> Option<String> {
        side.filter(|item| item != Self::BRICK)
    }

    pub fn ordered(&self) -> Vec<String> {
        [&self.back, &self.left, &self.right, &self.front]
            .into_iter()
            .map(|side| side.clone().unwrap_or_else(|| Self::BRICK.to_string()))
            .collect()
    }

    pub fn tooltip_items(&self) -> Vec<String> {
        if self.is_empty() {
            return Vec::new();
        }

        [&self.front, &self.left, &self.right, &self.back]
            .into_iter()
            .map(|side| side.clone().unwrap_or_else(|| Self::BRICK.to_string()))
            .collect()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.back.is_none() && self.left.is_none() && self.right.is_none() && self.front.is_none()
    }

    pub(super) fn to_tag(&self) -> Tag {
        Tag::List(self.ordered().into_iter().map(Tag::String).collect())
    }

    pub(super) fn from_tag(tag: &Tag) -> Self {
        let Tag::List(values) = tag else {
            return Self::default();
        };
        let item = |index: usize| -> Option<String> {
            values.get(index).and_then(|tag| match tag {
                Tag::String(item) if item != Self::BRICK => Some(item.clone()),
                _ => None,
            })
        };
        Self::new(item(0), item(1), item(2), item(3))
    }
}

impl PotItemStack {
    pub(super) fn is_empty(&self) -> bool {
        self.item_id == "minecraft:air" || self.count <= 0
    }

    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("id".to_string(), Tag::String(self.item_id.clone())),
            ("count".to_string(), Tag::Int(self.count)),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let stack = Self {
            item_id: get_string(entries, "id")?.to_string(),
            count: get_int(entries, "count").unwrap_or(1),
        };
        (!stack.is_empty()).then_some(stack)
    }
}

impl FurnaceBlockEntityKind {
    pub fn menu_type(self) -> &'static str {
        match self {
            Self::Furnace => "furnace",
            Self::BlastFurnace => "blast_furnace",
            Self::Smoker => "smoker",
        }
    }

    pub fn default_name(self) -> &'static str {
        match self {
            Self::Furnace => "container.furnace",
            Self::BlastFurnace => "container.blast_furnace",
            Self::Smoker => "container.smoker",
        }
    }

    pub fn recipe_type(self) -> &'static str {
        match self {
            Self::Furnace => "smelting",
            Self::BlastFurnace => "blasting",
            Self::Smoker => "smoking",
        }
    }

    pub fn default_cooking_time(self) -> i32 {
        match self {
            Self::Furnace => 200,
            Self::BlastFurnace | Self::Smoker => 100,
        }
    }

    pub(super) fn burn_duration(
        self,
        fuel_values: &FuelValues,
        fuel: Option<&PotItemStack>,
    ) -> i32 {
        let Some(fuel) = fuel else {
            return 0;
        };
        let burn = fuel_values.burn_duration(Some(fuel.item_id.as_str()));
        match self {
            Self::Furnace => burn,
            Self::BlastFurnace | Self::Smoker => burn / 2,
        }
    }
}

impl FurnaceCookingRecipe {
    pub fn new(
        recipe_id: &str,
        recipe_type: &str,
        input_item: &str,
        result_item: &str,
        cooking_time: i32,
        experience_millis: i32,
    ) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_type: recipe_type.to_string(),
            input_item: input_item.to_string(),
            result: PotItemStack {
                item_id: result_item.to_string(),
                count: 1,
            },
            cooking_time,
            experience_millis,
        }
    }

    /// Build the furnace's recipe view for `input_item` from the loaded cooking
    /// recipes (`RecipeKind::Cooking`), so the furnace is driven by the same recipe
    /// data as the rest of the game rather than a parallel table. `recipe_type` is
    /// the furnace kind's type id (`"smelting"`/`"blasting"`/`"smoking"`). Mirrors
    /// `AbstractFurnaceBlockEntity`'s `RecipeManager.getRecipeFor` lookup; the cook
    /// time falls back to the `AbstractCookingRecipe` default when unspecified.
    ///
    /// TODO(cooking-server-wiring): this unifies the cooking *model* (the furnace's
    /// recipe view now derives from `RecipeKind::Cooking` instead of a hand-authored
    /// table), but no production code yet calls `lookup` / `server_tick` — the
    /// furnace block entity is only ticked from tests, and `FuelValues::vanilla()`
    /// is `#[cfg(test)]`. Wiring the live cook loop (and production fuel values)
    /// belongs to the server block-entity-ticking subsystem; the cooking recipe
    /// items (CHECKLIST_RECIPES #75-81) cannot be marked until that exists.
    pub fn lookup(
        recipes: &crate::recipe_system::RecipeMap,
        recipe_type: &str,
        input_item: &str,
    ) -> Option<Self> {
        use crate::recipe_system::RecipeKind;
        for holder in recipes.values() {
            let RecipeKind::Cooking {
                kind,
                ingredient,
                result,
                experience_millis,
                cooking_time,
                ..
            } = &holder.recipe
            else {
                continue;
            };
            if holder.recipe.recipe_type() == recipe_type && ingredient.matches(input_item) {
                let time = cooking_time.unwrap_or_else(|| kind.default_cooking_time());
                return Some(Self::new(
                    holder.id,
                    recipe_type,
                    input_item,
                    result.item,
                    time,
                    *experience_millis,
                ));
            }
        }
        None
    }
}

impl AbstractFurnaceBlockEntity {
    pub const INGREDIENT_SLOT: usize = 0;
    pub const FUEL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const SLOT_COUNT: usize = 3;
    pub const MAX_STACK_SIZE: i32 = 64;

    pub fn furnace() -> Self {
        Self::new(FurnaceBlockEntityKind::Furnace)
    }

    pub fn blast_furnace() -> Self {
        Self::new(FurnaceBlockEntityKind::BlastFurnace)
    }

    pub fn smoker() -> Self {
        Self::new(FurnaceBlockEntityKind::Smoker)
    }

    pub fn new(kind: FurnaceBlockEntityKind) -> Self {
        Self {
            kind,
            items: [None, None, None],
            lit_time_remaining: 0,
            lit_total_time: 0,
            cooking_time_spent: 0,
            cooking_total_time: kind.default_cooking_time(),
            recipes_used: BTreeMap::new(),
        }
    }

    pub fn set_item(
        &mut self,
        slot: usize,
        stack: Option<PotItemStack>,
        recipe: Option<&FurnaceCookingRecipe>,
    ) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        let same_input = slot == Self::INGREDIENT_SLOT && self.items[slot] == stack;
        self.items[slot] = stack.filter(|item| !item.is_empty());
        if slot == Self::INGREDIENT_SLOT && !same_input {
            self.cooking_total_time = recipe
                .filter(|recipe| self.recipe_matches(recipe))
                .map(|recipe| recipe.cooking_time)
                .unwrap_or_else(|| self.kind.default_cooking_time());
            self.cooking_time_spent = 0;
        }
        true
    }

    pub fn server_tick(
        &mut self,
        fuel_values: &FuelValues,
        recipe: Option<&FurnaceCookingRecipe>,
    ) -> FurnaceTickResult {
        let was_lit = self.is_lit();
        if self.lit_time_remaining > 0 {
            self.lit_time_remaining -= 1;
        }
        let is_lit_after_decrement = self.is_lit();
        let has_ingredient = self.items[Self::INGREDIENT_SLOT].is_some();
        let has_fuel = self.items[Self::FUEL_SLOT].is_some();

        if is_lit_after_decrement || has_fuel && has_ingredient {
            if let Some(recipe) = recipe.filter(|recipe| self.recipe_matches(recipe)) {
                self.cooking_total_time = recipe.cooking_time;
                if self.can_burn(recipe) {
                    if !self.is_lit() {
                        let new_lit_time = self
                            .kind
                            .burn_duration(fuel_values, self.items[Self::FUEL_SLOT].as_ref());
                        self.lit_time_remaining = new_lit_time;
                        self.lit_total_time = new_lit_time;
                        if new_lit_time > 0 {
                            self.consume_fuel();
                        }
                    }

                    if self.is_lit() {
                        self.cooking_time_spent += 1;
                        if self.cooking_time_spent == self.cooking_total_time {
                            self.cooking_time_spent = 0;
                            self.burn(recipe);
                            self.record_recipe(recipe);
                            return FurnaceTickResult::Burned {
                                output_count: self.items[Self::RESULT_SLOT]
                                    .as_ref()
                                    .map(|stack| stack.count)
                                    .unwrap_or(0),
                            };
                        }
                        return if was_lit != self.is_lit() {
                            FurnaceTickResult::LitChanged { lit: self.is_lit() }
                        } else {
                            FurnaceTickResult::Cooking
                        };
                    }
                }
                self.cooking_time_spent = 0;
            } else if has_ingredient {
                self.cooking_time_spent = 0;
            }
        } else if self.cooking_time_spent > 0 {
            self.cooking_time_spent =
                (self.cooking_time_spent - 2).clamp(0, self.cooking_total_time);
            return FurnaceTickResult::Cooling;
        }

        if was_lit != self.is_lit() {
            FurnaceTickResult::LitChanged { lit: self.is_lit() }
        } else {
            FurnaceTickResult::Idle
        }
    }

    pub fn is_lit(&self) -> bool {
        self.lit_time_remaining > 0
    }

    pub fn can_place_item(
        &self,
        slot: usize,
        stack: &PotItemStack,
        fuel_values: &FuelValues,
    ) -> bool {
        match slot {
            Self::RESULT_SLOT => false,
            Self::FUEL_SLOT => {
                fuel_values.is_fuel(stack.item_id.as_str())
                    || stack.item_id == "minecraft:bucket"
                        && self.items[Self::FUEL_SLOT]
                            .as_ref()
                            .is_none_or(|fuel| fuel.item_id != "minecraft:bucket")
            }
            _ => slot < Self::SLOT_COUNT,
        }
    }

    pub fn can_take_item_through_face(
        &self,
        slot: usize,
        item_id: &str,
        direction: Direction,
    ) -> bool {
        direction != Direction::Down
            || slot != Self::FUEL_SLOT
            || item_id == "minecraft:water_bucket"
            || item_id == "minecraft:bucket"
    }

    pub fn max_stack_size(&self, slot: usize, item: &PotItemStack) -> i32 {
        if slot == Self::FUEL_SLOT && item.item_id == "minecraft:bucket" {
            1
        } else {
            Self::MAX_STACK_SIZE
        }
    }

    pub fn get_slots_for_face(direction: Direction) -> &'static [usize] {
        match direction {
            Direction::Down => &[Self::RESULT_SLOT, Self::FUEL_SLOT],
            Direction::Up => &[Self::INGREDIENT_SLOT],
            _ => &[Self::FUEL_SLOT],
        }
    }

    pub fn comparator_output(&self) -> u8 {
        let non_empty = self.items.iter().filter(|stack| stack.is_some()).count();
        if non_empty == 0 {
            return 0;
        }
        let fullness: f32 = self
            .items
            .iter()
            .filter_map(|stack| stack.as_ref())
            .map(|stack| (stack.count.max(0) as f32 / Self::MAX_STACK_SIZE as f32).min(1.0))
            .sum::<f32>()
            / Self::SLOT_COUNT as f32;
        (1 + (fullness * 14.0).floor() as u8).min(MAX_SIGNAL)
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: self.kind.menu_type(),
            initial_slots: self.items.to_vec(),
        }
    }

    pub fn xp_to_award_and_clear(&mut self, fraction_roll: f32) -> i32 {
        let total = self
            .recipes_used
            .iter()
            .map(|(_, (times_used, experience_millis))| {
                if *times_used <= 0 || *experience_millis <= 0 {
                    return 0;
                }
                let total_millis = *times_used * *experience_millis;
                let whole = total_millis / 1000;
                let fraction = (total_millis % 1000) as f32 / 1000.0;
                if fraction != 0.0 && fraction_roll < fraction {
                    whole + 1
                } else {
                    whole
                }
            })
            .sum();
        self.recipes_used.clear();
        total
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "cooking_time_spent".to_string(),
                Tag::Short(self.cooking_time_spent as i16),
            ),
            (
                "cooking_total_time".to_string(),
                Tag::Short(self.cooking_total_time as i16),
            ),
            (
                "lit_time_remaining".to_string(),
                Tag::Short(self.lit_time_remaining as i16),
            ),
            (
                "lit_total_time".to_string(),
                Tag::Short(self.lit_total_time as i16),
            ),
            ("Items".to_string(), self.items_tag()),
            ("RecipesUsed".to_string(), self.recipes_used_tag()),
        ])
    }

    pub fn load_additional(kind: FurnaceBlockEntityKind, tag: &Tag) -> Self {
        let mut furnace = Self::new(kind);
        let Some(entries) = compound_entries(tag) else {
            return furnace;
        };
        furnace.cooking_time_spent = get_short(entries, "cooking_time_spent").unwrap_or(0).max(0);
        furnace.cooking_total_time = get_short(entries, "cooking_total_time")
            .unwrap_or(kind.default_cooking_time())
            .max(0);
        furnace.lit_time_remaining = get_short(entries, "lit_time_remaining").unwrap_or(0).max(0);
        furnace.lit_total_time = get_short(entries, "lit_total_time").unwrap_or(0).max(0);
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(name, _)| name == "Items")
            .map(|(_, tag)| tag)
        {
            for item in items {
                if let Some(item_entries) = compound_entries(item) {
                    let slot = get_byte(item_entries, "Slot").unwrap_or(-1);
                    if (0..Self::SLOT_COUNT as i8).contains(&slot) {
                        furnace.items[slot as usize] = PotItemStack::from_tag(item);
                    }
                }
            }
        }
        if let Some(Tag::Compound(recipes)) = entries
            .iter()
            .find(|(name, _)| name == "RecipesUsed")
            .map(|(_, tag)| tag)
        {
            for (recipe_id, tag) in recipes {
                if let Tag::Compound(values) = tag {
                    let count = get_int(values, "count").unwrap_or(0).max(0);
                    let experience = get_int(values, "experience_millis").unwrap_or(0).max(0);
                    if count > 0 {
                        furnace
                            .recipes_used
                            .insert(recipe_id.clone(), (count, experience));
                    }
                }
            }
        }
        furnace
    }

    pub(super) fn recipe_matches(&self, recipe: &FurnaceCookingRecipe) -> bool {
        recipe.recipe_type == self.kind.recipe_type()
            && self.items[Self::INGREDIENT_SLOT]
                .as_ref()
                .is_some_and(|input| input.item_id == recipe.input_item && input.count > 0)
    }

    pub(super) fn can_burn(&self, recipe: &FurnaceCookingRecipe) -> bool {
        match &self.items[Self::RESULT_SLOT] {
            None => true,
            Some(result) if result.item_id == recipe.result.item_id => {
                result.count + recipe.result.count <= Self::MAX_STACK_SIZE
            }
            Some(_) => false,
        }
    }

    pub(super) fn burn(&mut self, recipe: &FurnaceCookingRecipe) {
        match &mut self.items[Self::RESULT_SLOT] {
            Some(result) => result.count += recipe.result.count,
            slot @ None => *slot = Some(recipe.result.clone()),
        }
        if let Some(input) = &mut self.items[Self::INGREDIENT_SLOT] {
            input.count -= 1;
            if input.count <= 0 {
                self.items[Self::INGREDIENT_SLOT] = None;
            }
        }
        if self.items[Self::INGREDIENT_SLOT]
            .as_ref()
            .is_some_and(|input| input.item_id == "minecraft:wet_sponge")
            && self.items[Self::FUEL_SLOT]
                .as_ref()
                .is_some_and(|fuel| fuel.item_id == "minecraft:bucket")
        {
            self.items[Self::FUEL_SLOT] = Some(PotItemStack {
                item_id: "minecraft:water_bucket".to_string(),
                count: 1,
            });
        }
    }

    pub(super) fn consume_fuel(&mut self) {
        if let Some(fuel) = &mut self.items[Self::FUEL_SLOT] {
            fuel.count -= 1;
            if fuel.count <= 0 {
                self.items[Self::FUEL_SLOT] = if fuel.item_id == "minecraft:lava_bucket" {
                    Some(PotItemStack {
                        item_id: "minecraft:bucket".to_string(),
                        count: 1,
                    })
                } else {
                    None
                };
            }
        }
    }

    pub(super) fn record_recipe(&mut self, recipe: &FurnaceCookingRecipe) {
        let entry = self
            .recipes_used
            .entry(recipe.recipe_id.clone())
            .or_insert((0, recipe.experience_millis));
        entry.0 += 1;
        entry.1 = recipe.experience_millis;
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

    pub(super) fn recipes_used_tag(&self) -> Tag {
        Tag::Compound(
            self.recipes_used
                .iter()
                .map(|(recipe_id, (count, experience_millis))| {
                    (
                        recipe_id.clone(),
                        Tag::Compound(vec![
                            ("count".to_string(), Tag::Int(*count)),
                            (
                                "experience_millis".to_string(),
                                Tag::Int(*experience_millis),
                            ),
                        ]),
                    )
                })
                .collect(),
        )
    }
}
