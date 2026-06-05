use super::*;

// ============================================================================
// AnvilMenu (combiner-style)
// ============================================================================

/// Threshold (in survival mode) above which the anvil result becomes
/// "Too Expensive!" and the player cannot complete the combine.
pub const ANVIL_MAX_COST: i32 = 40;

/// `AnvilMenu` cost contributions, matching Java constants.
pub mod anvil_cost {
    pub const BASE: i32 = 1;
    pub const ADDED_BASE: i32 = 1;
    pub const REPAIR_MATERIAL: i32 = 1;
    pub const REPAIR_SACRIFICE: i32 = 2;
    pub const RENAME: i32 = 1;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnvilMenu {
    input_left: ItemStack,
    input_right: ItemStack,
    result: ItemStack,
    item_name: Option<String>,
    pub cost: i32,
    pub repair_item_count_cost: i32,
    pub only_renaming: bool,
}

impl AnvilMenu {
    pub const INPUT_SLOT: usize = 0;
    pub const ADDITIONAL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const MENU_SLOTS: usize = 3;
    pub const INV_START: usize = 3;
    pub const HOTBAR_END: usize = 39;
    pub const SLOT_COUNT: usize = 39;
    pub const MAX_NAME_LENGTH: usize = 50;

    pub fn new() -> Self {
        Self {
            input_left: ItemStack::empty(),
            input_right: ItemStack::empty(),
            result: ItemStack::empty(),
            item_name: None,
            cost: 0,
            repair_item_count_cost: 0,
            only_renaming: false,
        }
    }

    /// `ItemCombinerMenu.removed`: route the carried cursor item, then
    /// `clearContainer(inputSlots)` — the two inputs return to the player on a
    /// normal close and drop in-world on disconnect. The (virtual) result is not
    /// returned.
    pub fn removed(
        &mut self,
        player: &mut PlayerInventory,
        carried: &mut ItemStack,
        disconnected: bool,
    ) {
        drop_or_place_in_inventory(
            player,
            std::mem::replace(carried, ItemStack::empty()),
            disconnected,
        );
        for slot in [&mut self.input_left, &mut self.input_right] {
            drop_or_place_in_inventory(
                player,
                std::mem::replace(slot, ItemStack::empty()),
                disconnected,
            );
        }
        self.result = ItemStack::empty();
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.input_left.clone(),
            1 => self.input_right.clone(),
            2 => self.result.clone(),
            _ => read_player_slot(slot, Self::INV_START, player),
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
            0 => self.input_left = stack,
            1 => self.input_right = stack,
            2 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        match slot {
            2 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.input_left.clone());
        out.push(self.input_right.clone());
        out.push(self.result.clone());
        append_player_slots(&mut out, player);
        out
    }

    pub fn set_item_name(&mut self, name: Option<String>) {
        self.item_name = name.map(|n| n.chars().take(Self::MAX_NAME_LENGTH).collect::<String>());
    }

    pub fn item_name(&self) -> Option<&str> {
        self.item_name.as_deref()
    }

    /// Re-implement the renaming-only path of `AnvilMenu.createResult`.
    /// When only `input_left` is present (no addition) and a new item-name is
    /// set, this produces a renamed copy with `cost = RENAME = 1`. For the
    /// merging/repair paths see vanilla — they require enchantment data we
    /// don't yet model here, so we leave the result empty and `cost = 0` in
    /// those cases (matching Java fall-through to clear result when no
    /// enchantment-storage is possible).
    pub fn set_result_from_inputs(&mut self) {
        self.cost = 1;
        self.only_renaming = false;
        self.repair_item_count_cost = 0;

        if self.input_left.is_empty() {
            self.result = ItemStack::empty();
            self.cost = 0;
            return;
        }

        let mut total_cost = 0;
        let mut naming_cost = 0;
        let mut result = self.input_left.clone();

        if let Some(name) = self.item_name.as_deref() {
            if !name.trim().is_empty() && name != self.input_left.item_id() {
                naming_cost = anvil_cost::RENAME;
                total_cost += naming_cost;
            }
        }

        if total_cost <= 0 {
            result = ItemStack::empty();
        }
        if naming_cost == total_cost && naming_cost > 0 {
            if self.cost >= ANVIL_MAX_COST {
                self.cost = ANVIL_MAX_COST - 1;
            }
            self.only_renaming = true;
        }
        self.cost = total_cost;
        if self.cost >= ANVIL_MAX_COST {
            // Survival cap; result is suppressed.
            result = ItemStack::empty();
        }
        self.result = result;
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
        if slot == 2 {
            self.result = ItemStack::empty();
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = match slot {
            2 => self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player),
            0 | 1 => self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            ),
            _ => self.move_into_range(&mut moving, 0, 2, false, player),
        };
        if !moving.is_empty() {
            if slot == 2 {
                // Result-slot leftover is dropped in vanilla; mirror by
                // simply discarding the remainder.
            } else {
                self.set_slot(slot, moving, player);
            }
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
                0 => self.input_left = w.new_stack,
                1 => self.input_right = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for AnvilMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SmithingMenu (combiner-style with template)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct SmithingMenu {
    template: ItemStack,
    base: ItemStack,
    addition: ItemStack,
    result: ItemStack,
    pub has_recipe_error: bool,
}

impl SmithingMenu {
    pub const TEMPLATE_SLOT: usize = 0;
    pub const BASE_SLOT: usize = 1;
    pub const ADDITIONAL_SLOT: usize = 2;
    pub const RESULT_SLOT: usize = 3;
    pub const MENU_SLOTS: usize = 4;
    pub const INV_START: usize = 4;
    pub const HOTBAR_END: usize = 40;
    pub const SLOT_COUNT: usize = 40;

    pub fn new() -> Self {
        Self {
            template: ItemStack::empty(),
            base: ItemStack::empty(),
            addition: ItemStack::empty(),
            result: ItemStack::empty(),
            has_recipe_error: false,
        }
    }

    /// `ItemCombinerMenu.removed`: route the carried cursor item, then
    /// `clearContainer(inputSlots)` — template/base/addition return to the player.
    pub fn removed(
        &mut self,
        player: &mut PlayerInventory,
        carried: &mut ItemStack,
        disconnected: bool,
    ) {
        drop_or_place_in_inventory(
            player,
            std::mem::replace(carried, ItemStack::empty()),
            disconnected,
        );
        for slot in [&mut self.template, &mut self.base, &mut self.addition] {
            drop_or_place_in_inventory(
                player,
                std::mem::replace(slot, ItemStack::empty()),
                disconnected,
            );
        }
        self.result = ItemStack::empty();
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.template.clone(),
            1 => self.base.clone(),
            2 => self.addition.clone(),
            3 => self.result.clone(),
            _ => read_player_slot(slot, Self::INV_START, player),
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
            0 => self.template = stack,
            1 => self.base = stack,
            2 => self.addition = stack,
            3 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        match slot {
            3 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.template.clone());
        out.push(self.base.clone());
        out.push(self.addition.clone());
        out.push(self.result.clone());
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
        if slot == 3 {
            self.result = ItemStack::empty();
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = match slot {
            3 => self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player),
            0..=2 => self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            ),
            _ => self.move_into_range(&mut moving, 0, 3, false, player),
        };
        if !moving.is_empty() && slot != 3 {
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
                0 => self.template = w.new_stack,
                1 => self.base = w.new_stack,
                2 => self.addition = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for SmithingMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// StonecutterMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct StonecutterMenu {
    input: ItemStack,
    result: ItemStack,
    recipes_for_input: Vec<StonecutterSelection>,
    pub selected_recipe_index: i32,
}

impl StonecutterMenu {
    pub const INPUT_SLOT: usize = 0;
    pub const RESULT_SLOT: usize = 1;
    pub const MENU_SLOTS: usize = 2;
    pub const INV_START: usize = 2;
    pub const INV_END: usize = 29;
    pub const USE_ROW_SLOT_START: usize = 29;
    pub const HOTBAR_END: usize = 38;
    pub const SLOT_COUNT: usize = 38;

    pub fn new() -> Self {
        Self {
            input: ItemStack::empty(),
            result: ItemStack::empty(),
            recipes_for_input: Vec::new(),
            selected_recipe_index: -1,
        }
    }

    /// `StonecutterMenu.removed`: discard the (virtual) result slot, then
    /// `clearContainer(container)` — the single input returns to the player.
    pub fn removed(
        &mut self,
        player: &mut PlayerInventory,
        carried: &mut ItemStack,
        disconnected: bool,
    ) {
        drop_or_place_in_inventory(
            player,
            std::mem::replace(carried, ItemStack::empty()),
            disconnected,
        );
        self.result = ItemStack::empty();
        drop_or_place_in_inventory(
            player,
            std::mem::replace(&mut self.input, ItemStack::empty()),
            disconnected,
        );
    }

    pub fn get_selected_recipe_index(&self) -> i32 {
        self.selected_recipe_index
    }

    pub fn get_visible_recipes(&self) -> &[StonecutterSelection] {
        &self.recipes_for_input
    }

    pub fn get_number_of_visible_recipes(&self) -> usize {
        self.recipes_for_input.len()
    }

    pub fn has_input_item(&self) -> bool {
        !self.input.is_empty() && !self.recipes_for_input.is_empty()
    }

    pub fn data(&self, index: usize) -> Option<i32> {
        (index == 0).then_some(self.selected_recipe_index)
    }

    pub fn set_data(&mut self, index: usize, value: i32) -> bool {
        if index != 0 {
            return false;
        }
        self.selected_recipe_index = value;
        true
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.input.clone(),
            1 => self.result.clone(),
            _ => read_player_slot(slot, Self::INV_START, player),
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
            0 => {
                let item_changed = self.input.item_id() != stack.item_id();
                self.input = stack;
                if item_changed {
                    self.clear_recipe_selection();
                }
            }
            1 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn set_result_internal(&mut self, stack: ItemStack) {
        self.result = stack;
    }

    pub fn slots_changed(&mut self, stonecutter_recipes: &[StonecutterSelection]) {
        self.setup_recipe_list(stonecutter_recipes);
    }

    pub fn click_button(&mut self, button_id: i32) -> bool {
        if self.selected_recipe_index == button_id {
            return false;
        }
        if self.is_valid_recipe_index(button_id) {
            self.selected_recipe_index = button_id;
            self.setup_result_slot(button_id);
        }
        true
    }

    pub fn take_result(&mut self) -> ItemStack {
        let taken = std::mem::replace(&mut self.result, ItemStack::empty());
        if taken.is_empty() {
            return taken;
        }
        self.input.shrink(1);
        if self.input.is_empty() {
            self.clear_recipe_selection();
        } else {
            self.setup_result_slot(self.selected_recipe_index);
        }
        taken
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        match slot {
            1 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.input.clone());
        out.push(self.result.clone());
        append_player_slots(&mut out, player);
        out
    }

    pub fn quick_move(&mut self, slot: usize, player: &mut PlayerInventory) -> ItemStack {
        self.quick_move_with_recipes(slot, player, &[])
    }

    pub fn quick_move_with_recipes(
        &mut self,
        slot: usize,
        player: &mut PlayerInventory,
        stonecutter_recipes: &[StonecutterSelection],
    ) -> ItemStack {
        if slot >= Self::SLOT_COUNT {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();
        if slot == 1 {
            self.result = ItemStack::empty();
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = match slot {
            1 => self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player),
            0 => self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            ),
            _ => {
                // Java only routes player stacks into the input slot when the
                // recipe manager has at least one stonecutting recipe for them.
                if Self::accepts_stonecutter_input(&moving, stonecutter_recipes)
                    && self.move_into_range(&mut moving, 0, 1, false, player)
                {
                    self.setup_recipe_list(stonecutter_recipes);
                    true
                } else if (Self::INV_START..Self::INV_END).contains(&slot) {
                    self.move_into_range(
                        &mut moving,
                        Self::USE_ROW_SLOT_START,
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
                }
            }
        };
        if slot == 1 && !moving.is_empty() {
            self.result = moving;
        } else if !moving.is_empty() {
            self.set_slot(slot, moving, player);
        }
        if !moved {
            return ItemStack::empty();
        }
        if slot == 1 {
            self.input.shrink(1);
            if self.input.is_empty() {
                self.clear_recipe_selection();
            } else {
                self.setup_result_slot(self.selected_recipe_index);
            }
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
                0 => self.input = w.new_stack,
                1 => self.result = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }

    fn clear_recipe_selection(&mut self) {
        self.selected_recipe_index = -1;
        self.result = ItemStack::empty();
        self.recipes_for_input.clear();
    }

    fn setup_recipe_list(&mut self, stonecutter_recipes: &[StonecutterSelection]) {
        self.selected_recipe_index = -1;
        self.result = ItemStack::empty();
        self.recipes_for_input = if self.input.is_empty() {
            Vec::new()
        } else {
            stonecutter_recipes_for_input(stonecutter_recipes, self.input.item_id())
        };
    }

    fn setup_result_slot(&mut self, index: i32) {
        self.result = self
            .recipes_for_input
            .get(index as usize)
            .map(|selection| item_amount_to_stack(&selection.result))
            .unwrap_or_else(ItemStack::empty);
    }

    fn is_valid_recipe_index(&self, button_id: i32) -> bool {
        button_id >= 0 && (button_id as usize) < self.recipes_for_input.len()
    }

    fn accepts_stonecutter_input(
        stack: &ItemStack,
        stonecutter_recipes: &[StonecutterSelection],
    ) -> bool {
        !stack.is_empty()
            && stonecutter_recipes
                .iter()
                .any(|recipe| recipe.matches_input(stack.item_id()))
    }
}

impl Default for StonecutterMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// GrindstoneMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct GrindstoneMenu {
    input_left: ItemStack,
    input_right: ItemStack,
    result: ItemStack,
}

impl GrindstoneMenu {
    pub const INPUT_SLOT: usize = 0;
    pub const ADDITIONAL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const MENU_SLOTS: usize = 3;
    pub const INV_START: usize = 3;
    pub const HOTBAR_END: usize = 39;
    pub const SLOT_COUNT: usize = 39;

    pub fn new() -> Self {
        Self {
            input_left: ItemStack::empty(),
            input_right: ItemStack::empty(),
            result: ItemStack::empty(),
        }
    }

    /// `GrindstoneMenu.removed`: `clearContainer(repairSlots)` — both inputs return
    /// to the player. The result is computed virtually and not returned.
    pub fn removed(
        &mut self,
        player: &mut PlayerInventory,
        carried: &mut ItemStack,
        disconnected: bool,
    ) {
        drop_or_place_in_inventory(
            player,
            std::mem::replace(carried, ItemStack::empty()),
            disconnected,
        );
        for slot in [&mut self.input_left, &mut self.input_right] {
            drop_or_place_in_inventory(
                player,
                std::mem::replace(slot, ItemStack::empty()),
                disconnected,
            );
        }
        self.result = ItemStack::empty();
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 | 1 => is_grindstone_input(stack),
            2 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.input_left.clone(),
            1 => self.input_right.clone(),
            2 => self.result.clone(),
            _ => read_player_slot(slot, Self::INV_START, player),
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
            0 => self.input_left = stack,
            1 => self.input_right = stack,
            2 => return false,
            _ => {
                write_player_slot(slot, Self::INV_START, player, stack);
                return true;
            }
        }
        // An input slot changed: recompute the disenchant/repair result (createResult).
        self.update_result();
        true
    }

    /// `GrindstoneMenu.createResult`: recompute the result slot from the two inputs.
    pub fn update_result(&mut self) {
        self.result = grindstone_compute_result(&self.input_left, &self.input_right);
    }

    /// `GrindstoneMenu.ResultSlot.onTake` experience: the XP dropped when the result is
    /// taken. `random_in_half` is the caller's `random.nextInt(ceil(total/2))` draw.
    pub fn experience_on_take(&self, random_in_half: i32) -> i32 {
        grindstone_experience_on_take(&self.input_left, &self.input_right, random_in_half)
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.input_left.clone());
        out.push(self.input_right.clone());
        out.push(self.result.clone());
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
        if slot == 2 {
            self.result = ItemStack::empty();
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = match slot {
            2 => self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player),
            0 | 1 => self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            ),
            _ => {
                if !self.input_left.is_empty() && !self.input_right.is_empty() {
                    if slot < Self::INV_START + PLAYER_MAIN_STORAGE {
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
                    }
                } else {
                    self.move_into_range(&mut moving, 0, 2, false, player)
                }
            }
        };
        if !moving.is_empty() && slot != 2 {
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
                0 => self.input_left = w.new_stack,
                1 => self.input_right = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for GrindstoneMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// EnchantmentMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct EnchantmentMenu {
    item: ItemStack,
    lapis: ItemStack,
    /// `[cost0, cost1, cost2, hint0, hint1, hint2]`. The hints map to
    /// enchantment IDs; -1 indicates "no enchantment available for this tier".
    pub data: [i32; 6],
}

impl EnchantmentMenu {
    pub const ITEM_SLOT: usize = 0;
    pub const LAPIS_SLOT: usize = 1;
    pub const MENU_SLOTS: usize = 2;
    pub const INV_START: usize = 2;
    pub const HOTBAR_END: usize = 38;
    pub const SLOT_COUNT: usize = 38;

    pub fn new() -> Self {
        Self {
            item: ItemStack::empty(),
            lapis: ItemStack::empty(),
            data: [0, 0, 0, -1, -1, -1],
        }
    }

    /// `EnchantmentMenu.removed`: `clearContainer(enchantSlots)` — the item and lapis
    /// return to the player.
    pub fn removed(
        &mut self,
        player: &mut PlayerInventory,
        carried: &mut ItemStack,
        disconnected: bool,
    ) {
        drop_or_place_in_inventory(
            player,
            std::mem::replace(carried, ItemStack::empty()),
            disconnected,
        );
        for slot in [&mut self.item, &mut self.lapis] {
            drop_or_place_in_inventory(
                player,
                std::mem::replace(slot, ItemStack::empty()),
                disconnected,
            );
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.item.clone(),
            1 => self.lapis.clone(),
            _ => read_player_slot(slot, Self::INV_START, player),
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
            0 => {
                let mut s = stack;
                s.limit_size(1);
                self.item = s;
            }
            1 => self.lapis = stack,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => slot < Self::SLOT_COUNT,
            1 => stack.is_empty() || stack.item_id() == "minecraft:lapis_lazuli",
            _ => slot < Self::SLOT_COUNT,
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
        out.push(self.item.clone());
        out.push(self.lapis.clone());
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
        let moved = match slot {
            0 | 1 => {
                self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
            }
            _ if moving.item_id() == "minecraft:lapis_lazuli" => {
                self.move_into_range(&mut moving, 1, 2, true, player)
            }
            _ => {
                if self.item.is_empty() && self.may_place(0, &moving) {
                    let one = moving.copy_with_count(1);
                    moving.shrink(1);
                    self.item = one;
                    true
                } else {
                    false
                }
            }
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
                0 => self.item = w.new_stack,
                1 => self.lapis = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for EnchantmentMenu {
    fn default() -> Self {
        Self::new()
    }
}
