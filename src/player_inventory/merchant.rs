use crate::item_stack::ItemStack;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemCost {
    pub item_id: &'static str,
    pub count: i32,
}

impl ItemCost {
    pub fn new(item_id: &'static str, count: i32) -> Self {
        Self { item_id, count }
    }

    pub fn matches(&self, stack: &ItemStack) -> bool {
        !stack.is_empty() && stack.item_id() == self.item_id
    }

    pub fn item_stack(&self) -> ItemStack {
        ItemStack::new(self.item_id, self.count)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantOffer {
    pub base_cost_a: ItemCost,
    pub cost_b: Option<ItemCost>,
    pub result: ItemStack,
    pub uses: i32,
    pub max_uses: i32,
    pub reward_exp: bool,
    pub special_price_diff: i32,
    pub demand: i32,
    pub price_multiplier: f32,
    pub xp: i32,
    pub ignore_discount: bool,
}

impl MerchantOffer {
    pub fn new(
        base_cost_a: ItemCost,
        cost_b: Option<ItemCost>,
        result: ItemStack,
        max_uses: i32,
        xp: i32,
        price_multiplier: f32,
    ) -> Self {
        Self {
            base_cost_a,
            cost_b,
            result,
            uses: 0,
            max_uses,
            reward_exp: true,
            special_price_diff: 0,
            demand: 0,
            price_multiplier,
            xp,
            ignore_discount: false,
        }
    }

    pub fn cost_a_count(&self) -> i32 {
        let base = self.base_cost_a.count;
        let demand_diff =
            ((base as f32 * self.demand as f32 * self.price_multiplier).floor() as i32).max(0);
        (base + demand_diff + self.special_price_diff)
            .clamp(1, self.base_cost_a.item_stack().max_stack_size() as i32)
    }

    pub fn get_cost_a(&self) -> ItemStack {
        ItemStack::new(self.base_cost_a.item_id, self.cost_a_count())
    }

    pub fn assemble(&self) -> ItemStack {
        self.result.clone()
    }

    pub fn update_demand(&mut self) {
        self.demand = self.demand + self.uses - (self.max_uses - self.uses);
    }

    pub fn increase_uses(&mut self) {
        self.uses += 1;
    }

    pub fn is_out_of_stock(&self) -> bool {
        self.uses >= self.max_uses
    }

    pub fn needs_restock(&self) -> bool {
        self.uses > 0
    }

    pub fn reset_uses(&mut self) {
        self.uses = 0;
    }

    pub fn satisfied_by(&self, buy_a: &ItemStack, buy_b: &ItemStack) -> bool {
        self.base_cost_a.matches(buy_a)
            && buy_a.count() >= self.cost_a_count()
            && match &self.cost_b {
                Some(cost_b) => cost_b.matches(buy_b) && buy_b.count() >= cost_b.count,
                None => buy_b.is_empty(),
            }
    }

    pub fn take(&self, buy_a: &mut ItemStack, buy_b: &mut ItemStack) -> bool {
        if !self.satisfied_by(buy_a, buy_b) {
            return false;
        }
        buy_a.shrink(self.cost_a_count());
        if let Some(cost_b) = &self.cost_b {
            buy_b.shrink(cost_b.count);
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantContainer {
    slots: [ItemStack; 3],
    offers: Vec<MerchantOffer>,
    active_offer: Option<usize>,
    selection_hint: usize,
    future_xp: i32,
    villager_xp: i32,
}

impl MerchantContainer {
    pub fn new(offers: Vec<MerchantOffer>) -> Self {
        Self {
            slots: [ItemStack::empty(), ItemStack::empty(), ItemStack::empty()],
            offers,
            active_offer: None,
            selection_hint: 0,
            future_xp: 0,
            villager_xp: 0,
        }
    }

    pub fn set_selection_hint(&mut self, selection_hint: usize) {
        self.selection_hint = selection_hint;
        self.update_sell_item();
    }

    pub fn set_payment(&mut self, slot: usize, stack: ItemStack) {
        if slot <= 1 {
            self.slots[slot] = stack;
            self.update_sell_item();
        }
    }

    pub fn can_trade(&self) -> bool {
        self.active_offer.is_some()
    }

    pub fn prepare_trade(&mut self) {
        self.update_sell_item();
    }

    pub fn result(&self) -> &ItemStack {
        &self.slots[2]
    }

    pub fn active_offer(&self) -> Option<&MerchantOffer> {
        self.active_offer.and_then(|index| self.offers.get(index))
    }

    pub fn offer(&self, index: usize) -> Option<&MerchantOffer> {
        self.offers.get(index)
    }

    pub fn future_xp(&self) -> i32 {
        self.future_xp
    }

    pub fn villager_xp(&self) -> i32 {
        self.villager_xp
    }

    pub fn take_result(&mut self) -> ItemStack {
        let result = std::mem::replace(&mut self.slots[2], ItemStack::empty());
        if !result.is_empty() {
            if let Some(index) = self.active_offer {
                let (payment_a, rest) = self.slots.split_at_mut(1);
                if self.offers[index].take(&mut payment_a[0], &mut rest[0])
                    || self.offers[index].take(&mut rest[0], &mut payment_a[0])
                {
                    self.offers[index].increase_uses();
                    self.villager_xp += self.offers[index].xp;
                }
            }
        }
        self.update_sell_item();
        result
    }

    pub fn update_sell_item(&mut self) {
        self.active_offer = None;
        let (buy_a, buy_b, swapped) = if self.slots[0].is_empty() {
            (&self.slots[1], &ItemStack::empty(), true)
        } else {
            (&self.slots[0], &self.slots[1], false)
        };
        if buy_a.is_empty() {
            self.slots[2] = ItemStack::empty();
            self.future_xp = 0;
            return;
        }

        let hinted = self.find_matching_offer(buy_a, buy_b, self.selection_hint);
        let fallback = if hinted.is_some() {
            hinted
        } else if swapped {
            None
        } else {
            self.find_matching_offer(&self.slots[1], &self.slots[0], self.selection_hint)
        };
        if let Some(index) = fallback.filter(|index| !self.offers[*index].is_out_of_stock()) {
            self.active_offer = Some(index);
            self.slots[2] = self.offers[index].assemble();
            self.future_xp = self.offers[index].xp;
        } else {
            self.slots[2] = ItemStack::empty();
            self.future_xp = 0;
        }
    }

    fn find_matching_offer(
        &self,
        buy_a: &ItemStack,
        buy_b: &ItemStack,
        hint: usize,
    ) -> Option<usize> {
        if let Some(offer) = self.offers.get(hint) {
            if offer.satisfied_by(buy_a, buy_b) {
                return Some(hint);
            }
        }
        self.offers
            .iter()
            .position(|offer| offer.satisfied_by(buy_a, buy_b))
    }

    /// Number of offers exposed by the merchant. Java: `MerchantOffers.size()`.
    pub fn offer_count(&self) -> usize {
        self.offers.len()
    }

    /// Read-only access to the offers. Used by `MerchantMenu::try_move_items`
    /// and by clients persisting trade state.
    pub fn offers(&self) -> &[MerchantOffer] {
        &self.offers
    }

    /// Run the full restock cycle for this merchant.
    ///
    /// Java: `Villager.restock()` calls `updateDemand()` over every offer
    /// *before* resetting uses so the next demand tick observes the trades
    /// that have happened during this restock window. After that, every
    /// offer's `uses` counter is cleared and the result slot is refreshed
    /// so the player immediately sees the restored stock.
    pub fn restock(&mut self) {
        self.update_demand();
        for offer in &mut self.offers {
            offer.reset_uses();
        }
        self.update_sell_item();
    }

    /// Update demand for every offer.
    ///
    /// Java: `Villager.updateDemand()` iterates the offers and delegates to
    /// `MerchantOffer.updateDemand()`, which uses the formula
    /// `demand = demand + uses - (maxUses - uses)`.
    pub fn update_demand(&mut self) {
        for offer in &mut self.offers {
            offer.update_demand();
        }
    }

    /// Apply the hero-of-the-village price reduction to every offer.
    ///
    /// Java: `Villager.updateSpecialPrices()` computes
    /// `modifier = 0.3 + 0.0625 * amplifier`, then for every offer subtracts
    /// `max(floor(modifier * baseCostA.count), 1)` from `specialPriceDiff`.
    /// `amplifier` is the 0-based effect amplifier from
    /// `MobEffectInstance.getAmplifier()`.
    pub fn apply_hero_discount(&mut self, amplifier: i32) {
        let modifier = 0.3 + 0.0625 * amplifier as f64;
        for offer in &mut self.offers {
            let cost_reduction =
                ((modifier * offer.base_cost_a.count as f64).floor() as i32).max(1);
            offer.special_price_diff -= cost_reduction;
        }
    }

    /// Clear the hero-of-the-village discount for every offer.
    ///
    /// Called when the effect wears off and after restock so prices return to
    /// their base values plus any demand modifier. Java: `Villager` re-runs
    /// `updateSpecialPrices()` on each interaction; this matches the side
    /// effect of an empty reputation/effect combination.
    pub fn reset_special_prices(&mut self) {
        for offer in &mut self.offers {
            offer.special_price_diff = 0;
        }
    }
}

// ============================================================================
// MerchantRestockTracker
// ============================================================================

/// Tracks how many times a villager has restocked today and at what game time.
///
/// Java: `Villager` holds two persistent fields: `numberOfRestocksToday` and
/// `lastRestockGameTime`. The merchant is allowed to restock if it has not
/// restocked yet today, or it has restocked once and at least 2400 ticks have
/// passed since that restock. `Villager.shouldRestock()` resets the counter
/// when a new game day starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantRestockTracker {
    pub restocks_today: i32,
    pub last_restock_game_time: i64,
}

impl MerchantRestockTracker {
    /// Build a fresh tracker for a villager that has never restocked.
    pub fn new() -> Self {
        Self {
            restocks_today: 0,
            last_restock_game_time: 0,
        }
    }

    /// Java: `Villager.allowedToRestock()`.
    ///
    /// A villager may restock either if it has not restocked at all today, or
    /// it has restocked exactly once and the configured cooldown (2400 ticks
    /// = 2 minutes of game time) has elapsed since the last restock.
    pub fn allowed_to_restock(&self, game_time: i64) -> bool {
        self.restocks_today == 0
            || (self.restocks_today < 2 && game_time > self.last_restock_game_time + 2400)
    }

    /// Record a restock at the given game time.
    ///
    /// Java: at the end of `Villager.restock()` the villager assigns
    /// `lastRestockGameTime = level.getGameTime()` and increments
    /// `numberOfRestocksToday`.
    pub fn record_restock(&mut self, game_time: i64) {
        self.last_restock_game_time = game_time;
        self.restocks_today += 1;
    }

    /// Java: `Villager.resetNumberOfRestocks()`. Called when a new day begins.
    pub fn reset_for_new_day(&mut self) {
        self.restocks_today = 0;
    }
}

impl Default for MerchantRestockTracker {
    fn default() -> Self {
        Self::new()
    }
}
