use super::*;

// ============================================================================
// LecternMenu
// ============================================================================

/// Lectern menu — no quick-move (`quickMoveStack` always returns `EMPTY`),
/// only a one-slot book container and a page index.
#[derive(Debug, Clone, PartialEq)]
pub struct LecternMenu {
    book: ItemStack,
    /// Current page index.
    pub page: i32,
}

impl LecternMenu {
    pub const BUTTON_PREV_PAGE: i32 = 1;
    pub const BUTTON_NEXT_PAGE: i32 = 2;
    pub const BUTTON_TAKE_BOOK: i32 = 3;
    pub const BUTTON_PAGE_JUMP_RANGE_START: i32 = 100;
    pub const SLOT_COUNT: usize = 1;

    pub fn new() -> Self {
        Self {
            book: ItemStack::empty(),
            page: 0,
        }
    }

    pub fn with_book(book: ItemStack) -> Self {
        Self { book, page: 0 }
    }

    pub fn book(&self) -> &ItemStack {
        &self.book
    }

    pub fn get_slot(&self, slot: usize) -> Option<ItemStack> {
        if slot == 0 {
            Some(self.book.clone())
        } else {
            None
        }
    }

    pub fn set_slot(&mut self, slot: usize, stack: ItemStack) -> bool {
        if slot == 0 {
            self.book = stack;
            true
        } else {
            false
        }
    }

    pub fn may_place(&self, _slot: usize) -> bool {
        // Vanilla lectern menu uses a generic slot — but the only way to put a
        // book in is via the lectern block's `interact`. Quick-move always
        // returns empty, so we keep this true for direct API consistency.
        true
    }

    pub fn all_slots(&self) -> Vec<ItemStack> {
        vec![self.book.clone()]
    }

    pub fn quick_move(&mut self, _slot: usize) -> ItemStack {
        ItemStack::empty()
    }

    /// `clickMenuButton` — see Java for button IDs.
    pub fn click_button(&mut self, button_id: i32) -> bool {
        if button_id >= Self::BUTTON_PAGE_JUMP_RANGE_START {
            self.page = button_id - Self::BUTTON_PAGE_JUMP_RANGE_START;
            return true;
        }
        match button_id {
            Self::BUTTON_PREV_PAGE => {
                self.page = (self.page - 1).max(0);
                true
            }
            Self::BUTTON_NEXT_PAGE => {
                self.page += 1;
                true
            }
            Self::BUTTON_TAKE_BOOK => true,
            _ => false,
        }
    }
}

impl Default for LecternMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BeaconMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct BeaconMenu {
    payment: ItemStack,
    /// `[level, primary_effect, secondary_effect]`.
    pub data: [i32; 3],
}

impl BeaconMenu {
    pub const PAYMENT_SLOT: usize = 0;
    pub const MENU_SLOTS: usize = 1;
    pub const DATA_COUNT: usize = 3;
    pub const LEVELS_DATA: usize = 0;
    pub const PRIMARY_EFFECT_DATA: usize = 1;
    pub const SECONDARY_EFFECT_DATA: usize = 2;
    pub const INV_START: usize = 1;
    pub const HOTBAR_END: usize = 37;
    pub const SLOT_COUNT: usize = 37;

    pub fn new() -> Self {
        Self {
            payment: ItemStack::empty(),
            data: [0; 3],
        }
    }

    pub fn data(&self, index: usize) -> Option<i32> {
        self.data.get(index).copied()
    }

    pub fn set_data(&mut self, index: usize, value: i32) -> bool {
        let Some(slot) = self.data.get_mut(index) else {
            return false;
        };
        *slot = value;
        true
    }

    pub fn get_levels(&self) -> i32 {
        self.data[Self::LEVELS_DATA]
    }

    pub fn encode_effect(effect_id: Option<i32>) -> Option<i32> {
        match effect_id {
            None => Some(0),
            Some(id) if (0..crate::status_effect::STATUS_EFFECTS.len() as i32).contains(&id) => {
                Some(id + 1)
            }
            Some(_) => None,
        }
    }

    pub fn decode_effect(encoded_id: i32) -> Option<i32> {
        if encoded_id == 0 {
            return None;
        }
        let effect_id = encoded_id - 1;
        (0..crate::status_effect::STATUS_EFFECTS.len() as i32)
            .contains(&effect_id)
            .then_some(effect_id)
    }

    pub fn primary_effect_id(&self) -> Option<i32> {
        Self::decode_effect(self.data[Self::PRIMARY_EFFECT_DATA])
    }

    pub fn secondary_effect_id(&self) -> Option<i32> {
        Self::decode_effect(self.data[Self::SECONDARY_EFFECT_DATA])
    }

    pub fn has_payment(&self) -> bool {
        !self.payment.is_empty()
    }

    /// Java `BeaconMenu.updateEffects`: only a present payment applies the
    /// selected effects, then one payment item is consumed.
    pub fn update_effects(&mut self, primary: Option<i32>, secondary: Option<i32>) -> bool {
        if !self.has_payment() {
            return false;
        }
        let Some(primary) = Self::encode_effect(primary) else {
            return false;
        };
        let Some(secondary) = Self::encode_effect(secondary) else {
            return false;
        };
        self.data[Self::PRIMARY_EFFECT_DATA] = primary;
        self.data[Self::SECONDARY_EFFECT_DATA] = secondary;
        self.payment.shrink(1);
        true
    }

    pub fn removed(&mut self) -> ItemStack {
        std::mem::replace(&mut self.payment, ItemStack::empty())
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(if slot == 0 {
            self.payment.clone()
        } else {
            read_player_slot(slot, Self::INV_START, player)
        })
    }

    pub fn set_slot(
        &mut self,
        slot: usize,
        stack: ItemStack,
        player: &mut PlayerInventory,
    ) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        if slot == 0 {
            let mut s = stack;
            s.limit_size(1);
            self.payment = s;
        } else {
            write_player_slot(slot, Self::INV_START, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => !stack.is_empty() && BEACON_PAYMENT_ITEMS.contains(&stack.item_id()),
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn max_stack_size(&self, slot: usize) -> i32 {
        if slot == 0 {
            1
        } else {
            64
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.payment.clone());
        append_player_slots(&mut out, player);
        out
    }

    pub fn quick_move(&mut self, slot: usize, player: &mut PlayerInventory) -> ItemStack {
        if slot >= Self::SLOT_COUNT {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();
        self.set_slot(slot, ItemStack::empty(), player);

        let moved = if slot == 0 {
            self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
        } else if self.payment.is_empty() && self.may_place(0, &moving) && moving.count() == 1 {
            self.move_into_range(&mut moving, 0, 1, false, player)
        } else if slot < Self::INV_START + PLAYER_MAIN_STORAGE {
            self.move_into_range(
                &mut moving,
                Self::INV_START + PLAYER_MAIN_STORAGE,
                Self::HOTBAR_END,
                false,
                player,
            )
        } else {
            self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::INV_START + PLAYER_MAIN_STORAGE,
                false,
                player,
            )
        };

        if !moving.is_empty() {
            self.set_slot(slot, moving, player);
        }
        if !moved {
            return ItemStack::empty();
        }
        original
    }

    fn move_into_range(
        &mut self,
        stack: &mut ItemStack,
        start: usize,
        end: usize,
        reverse: bool,
        player: &mut PlayerInventory,
    ) -> bool {
        let snapshot = self.all_slots(player);
        let (leftover, writes, moved) =
            plan_move_item_stack_to(stack.clone(), start, end, reverse, &snapshot, |s, st| {
                self.may_place(s, st)
            });
        for w in writes {
            match w.slot {
                0 => self.payment = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for BeaconMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CrafterMenu — 3×3 + result, with per-slot disable toggle
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CrafterMenu {
    grid: [ItemStack; 9],
    /// Slot index 0..9 = disabled flags; index 9 = powered flag (matching
    /// `SimpleContainerData(10)` in vanilla).
    pub container_data: [i32; 10],
    result: ItemStack,
}

impl CrafterMenu {
    pub const GRID_SIZE: usize = 9;
    pub const GRID_START: usize = 0;
    pub const GRID_END: usize = 9;
    pub const INV_START: usize = 9;
    pub const INV_END: usize = 36;
    pub const HOTBAR_START: usize = 36;
    pub const HOTBAR_END: usize = 45;
    pub const RESULT_SLOT: usize = 45;
    pub const SLOT_COUNT: usize = 46;

    pub fn new() -> Self {
        Self {
            grid: std::array::from_fn(|_| ItemStack::empty()),
            container_data: [0; 10],
            result: ItemStack::empty(),
        }
    }

    pub fn set_slot_state(&mut self, slot_id: usize, enabled: bool) {
        if slot_id < Self::GRID_SIZE {
            self.container_data[slot_id] = if enabled { 0 } else { 1 };
        }
    }

    pub fn is_slot_disabled(&self, slot_id: usize) -> bool {
        slot_id < Self::GRID_SIZE && self.container_data[slot_id] == 1
    }

    pub fn is_powered(&self) -> bool {
        self.container_data[9] == 1
    }

    pub fn set_powered(&mut self, powered: bool) {
        self.container_data[9] = if powered { 1 } else { 0 };
    }

    pub fn refresh_result(&mut self, recipes: &RecipeMap) {
        let items: Vec<Option<&'static str>> = self
            .grid
            .iter()
            .map(|s| (!s.is_empty()).then(|| s.item_id()))
            .collect();
        if let Some(holder) = recipes.get_recipe_for("crafting", 3, 3, &items) {
            self.result = holder
                .recipe
                .assemble()
                .map(|r| ItemStack::new(r.item, r.count as i32))
                .unwrap_or_else(ItemStack::empty);
        } else {
            self.result = ItemStack::empty();
        }
    }

    pub fn result(&self) -> &ItemStack {
        &self.result
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0..=8 => self.grid[slot].clone(),
            9..=44 => read_player_slot(slot, Self::INV_START, player),
            45 => self.result.clone(),
            _ => unreachable!(),
        })
    }

    pub fn set_slot(
        &mut self,
        slot: usize,
        stack: ItemStack,
        player: &mut PlayerInventory,
    ) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        match slot {
            0..=8 => {
                self.grid[slot] = stack;
                true
            }
            9..=44 => {
                write_player_slot(slot, Self::INV_START, player, stack);
                true
            }
            45 => false,
            _ => false,
        }
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < Self::SLOT_COUNT && slot != 45
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for g in &self.grid {
            out.push(g.clone());
        }
        append_player_slots(&mut out, player);
        out.push(self.result.clone());
        out
    }

    pub fn quick_move(&mut self, slot: usize, player: &mut PlayerInventory) -> ItemStack {
        if slot >= Self::SLOT_COUNT {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();
        if slot != 45 {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = if slot < Self::GRID_SIZE {
            self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
        } else {
            self.move_into_range(&mut moving, 0, Self::GRID_SIZE, false, player)
        };
        if !moving.is_empty() && slot != 45 {
            self.set_slot(slot, moving, player);
        }
        if !moved {
            return ItemStack::empty();
        }
        original
    }

    fn move_into_range(
        &mut self,
        stack: &mut ItemStack,
        start: usize,
        end: usize,
        reverse: bool,
        player: &mut PlayerInventory,
    ) -> bool {
        let snapshot = self.all_slots(player);
        let (leftover, writes, moved) =
            plan_move_item_stack_to(stack.clone(), start, end, reverse, &snapshot, |s, st| {
                self.may_place(s, st)
            });
        for w in writes {
            match w.slot {
                0..=8 => self.grid[w.slot] = w.new_stack,
                9..=44 => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
                _ => {}
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for CrafterMenu {
    fn default() -> Self {
        Self::new()
    }
}

