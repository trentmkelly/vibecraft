use super::*;

// ============================================================================
// ChestMenu — 1..6 rows
// ============================================================================

/// Generic chest. `rows` may be 1..=6.
#[derive(Debug, Clone, PartialEq)]
pub struct ChestMenu {
    rows: usize,
    slots: Vec<ItemStack>,
}

impl ChestMenu {
    pub const MAX_ROWS: usize = 6;

    pub fn new(rows: usize) -> Self {
        assert!(rows >= 1 && rows <= Self::MAX_ROWS);
        Self {
            rows,
            slots: vec![ItemStack::empty(); rows * 9],
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn chest_size(&self) -> usize {
        self.rows * 9
    }

    pub fn slot_count(&self) -> usize {
        self.chest_size() + PLAYER_SLOTS
    }

    pub fn inv_start(&self) -> usize {
        self.chest_size()
    }

    pub fn hotbar_start(&self) -> usize {
        self.chest_size() + PLAYER_MAIN_STORAGE
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= self.slot_count() {
            return None;
        }
        if slot < self.chest_size() {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, self.inv_start(), player))
        }
    }

    pub fn set_slot(
        &mut self,
        slot: usize,
        stack: ItemStack,
        player: &mut PlayerInventory,
    ) -> bool {
        if slot >= self.slot_count() {
            return false;
        }
        if slot < self.chest_size() {
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, self.inv_start(), player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < self.slot_count()
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(self.slot_count());
        for s in &self.slots {
            out.push(s.clone());
        }
        append_player_slots(&mut out, player);
        out
    }

    pub fn quick_move(&mut self, slot: usize, player: &mut PlayerInventory) -> ItemStack {
        if slot >= self.slot_count() {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let chest_size = self.chest_size();
        let total = self.slot_count();
        let mut moving = original.clone();
        self.set_slot(slot, ItemStack::empty(), player);

        let moved = if slot < chest_size {
            self.move_into_range(&mut moving, chest_size, total, true, player)
        } else {
            self.move_into_range(&mut moving, 0, chest_size, false, player)
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
        let inv_start = self.inv_start();
        let chest_size = self.chest_size();
        let snapshot = self.all_slots(player);
        let (leftover, writes, moved) =
            plan_move_item_stack_to(stack.clone(), start, end, reverse, &snapshot, |s, st| {
                self.may_place(s, st)
            });
        for w in writes {
            if w.slot < chest_size {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, inv_start, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

// ============================================================================
// HopperMenu
// ============================================================================

/// `HopperMenu` — 5 hopper slots + player inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct HopperMenu {
    slots: [ItemStack; 5],
}

impl HopperMenu {
    pub const HOPPER_SIZE: usize = 5;
    pub const SLOT_COUNT: usize = Self::HOPPER_SIZE + PLAYER_SLOTS;

    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::empty()),
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        if slot < Self::HOPPER_SIZE {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, Self::HOPPER_SIZE, player))
        }
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
        if slot < Self::HOPPER_SIZE {
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, Self::HOPPER_SIZE, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < Self::SLOT_COUNT
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for s in &self.slots {
            out.push(s.clone());
        }
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

        let moved = if slot < Self::HOPPER_SIZE {
            self.move_into_range(
                &mut moving,
                Self::HOPPER_SIZE,
                Self::SLOT_COUNT,
                true,
                player,
            )
        } else {
            self.move_into_range(&mut moving, 0, Self::HOPPER_SIZE, false, player)
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
            if w.slot < Self::HOPPER_SIZE {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, Self::HOPPER_SIZE, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for HopperMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DispenserMenu (3×3, used by Dispenser + Dropper)
// ============================================================================

/// `DispenserMenu` — 3×3 grid + player inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct DispenserMenu {
    slots: [ItemStack; 9],
}

impl DispenserMenu {
    pub const GRID_SIZE: usize = 9;
    pub const SLOT_COUNT: usize = Self::GRID_SIZE + PLAYER_SLOTS;

    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::empty()),
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        if slot < Self::GRID_SIZE {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, Self::GRID_SIZE, player))
        }
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
        if slot < Self::GRID_SIZE {
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, Self::GRID_SIZE, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < Self::SLOT_COUNT
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for s in &self.slots {
            out.push(s.clone());
        }
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
        let moved = if slot < Self::GRID_SIZE {
            self.move_into_range(&mut moving, Self::GRID_SIZE, Self::SLOT_COUNT, true, player)
        } else {
            self.move_into_range(&mut moving, 0, Self::GRID_SIZE, false, player)
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
            if w.slot < Self::GRID_SIZE {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, Self::GRID_SIZE, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for DispenserMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ShulkerBoxMenu
// ============================================================================

/// `ShulkerBoxMenu` — 27 slots (no nested shulker boxes allowed).
#[derive(Debug, Clone, PartialEq)]
pub struct ShulkerBoxMenu {
    slots: Vec<ItemStack>,
}

impl ShulkerBoxMenu {
    pub const CONTAINER_SIZE: usize = 27;
    pub const SLOT_COUNT: usize = Self::CONTAINER_SIZE + PLAYER_SLOTS;

    pub fn new() -> Self {
        Self {
            slots: vec![ItemStack::empty(); Self::CONTAINER_SIZE],
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        if slot < Self::CONTAINER_SIZE {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, Self::CONTAINER_SIZE, player))
        }
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
        if slot < Self::CONTAINER_SIZE {
            // ShulkerBoxSlot rejects nested shulker box items.
            if Self::is_shulker_box(&stack) {
                return false;
            }
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, Self::CONTAINER_SIZE, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        if slot < Self::CONTAINER_SIZE && Self::is_shulker_box(stack) {
            return false;
        }
        true
    }

    fn is_shulker_box(stack: &ItemStack) -> bool {
        let id = stack.item_id();
        id == "minecraft:shulker_box" || id.ends_with("_shulker_box")
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for s in &self.slots {
            out.push(s.clone());
        }
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
        let moved = if slot < Self::CONTAINER_SIZE {
            self.move_into_range(
                &mut moving,
                Self::CONTAINER_SIZE,
                Self::SLOT_COUNT,
                true,
                player,
            )
        } else {
            self.move_into_range(&mut moving, 0, Self::CONTAINER_SIZE, false, player)
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
            if w.slot < Self::CONTAINER_SIZE {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, Self::CONTAINER_SIZE, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for ShulkerBoxMenu {
    fn default() -> Self {
        Self::new()
    }
}

