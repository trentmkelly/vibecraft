#![allow(dead_code)]

use crate::ai_system::{GossipContainer, GossipType, PoiTicket};
use crate::item_stack::ItemStack;
use crate::player_inventory::{ItemCost, MerchantOffer};
use crate::spawning::WanderingTraderData;
#[cfg(test)]
use crate::villager_trade_resources::{
    load_trade_set_resource, load_villager_trade_resource, parse_trade_set_resource,
    parse_villager_trade_resource, HolderSetResource, NumberProviderResource,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VillagerLevel {
    Novice = 1,
    Apprentice = 2,
    Journeyman = 3,
    Expert = 4,
    Master = 5,
}

impl VillagerLevel {
    pub fn from_xp(xp: i32) -> Self {
        match xp {
            i32::MIN..=9 => Self::Novice,
            10..=69 => Self::Apprentice,
            70..=149 => Self::Journeyman,
            150..=249 => Self::Expert,
            _ => Self::Master,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VillagerProfession {
    None,
    Armorer,
    Butcher,
    Cartographer,
    Cleric,
    Farmer,
    Fisherman,
    Fletcher,
    Leatherworker,
    Librarian,
    Mason,
    Nitwit,
    Shepherd,
    Toolsmith,
    Weaponsmith,
}

impl VillagerProfession {
    pub fn workstation(self) -> Option<&'static str> {
        match self {
            Self::None | Self::Nitwit => None,
            Self::Armorer => Some("minecraft:blast_furnace"),
            Self::Butcher => Some("minecraft:smoker"),
            Self::Cartographer => Some("minecraft:cartography_table"),
            Self::Cleric => Some("minecraft:brewing_stand"),
            Self::Farmer => Some("minecraft:composter"),
            Self::Fisherman => Some("minecraft:barrel"),
            Self::Fletcher => Some("minecraft:fletching_table"),
            Self::Leatherworker => Some("minecraft:cauldron"),
            Self::Librarian => Some("minecraft:lectern"),
            Self::Mason => Some("minecraft:stonecutter"),
            Self::Shepherd => Some("minecraft:loom"),
            Self::Toolsmith => Some("minecraft:smithing_table"),
            Self::Weaponsmith => Some("minecraft:grindstone"),
        }
    }

    pub fn from_workstation(block: &str) -> Option<Self> {
        match block {
            "minecraft:blast_furnace" => Some(Self::Armorer),
            "minecraft:smoker" => Some(Self::Butcher),
            "minecraft:cartography_table" => Some(Self::Cartographer),
            "minecraft:brewing_stand" => Some(Self::Cleric),
            "minecraft:composter" => Some(Self::Farmer),
            "minecraft:barrel" => Some(Self::Fisherman),
            "minecraft:fletching_table" => Some(Self::Fletcher),
            "minecraft:cauldron" => Some(Self::Leatherworker),
            "minecraft:lectern" => Some(Self::Librarian),
            "minecraft:stonecutter" => Some(Self::Mason),
            "minecraft:loom" => Some(Self::Shepherd),
            "minecraft:smithing_table" => Some(Self::Toolsmith),
            "minecraft:grindstone" => Some(Self::Weaponsmith),
            _ => None,
        }
    }

    pub fn held_job_site_matches(self, poi_type: &str) -> bool {
        self.workstation() == Some(poi_type)
    }

    pub fn acquirable_job_site_matches(self, poi_type: &str) -> bool {
        match self {
            Self::None => Self::from_workstation(poi_type).is_some(),
            _ => self.held_job_site_matches(poi_type),
        }
    }

    pub fn work_sound(self) -> Option<&'static str> {
        match self {
            Self::None | Self::Nitwit => None,
            Self::Armorer => Some("minecraft:entity.villager.work_armorer"),
            Self::Butcher => Some("minecraft:entity.villager.work_butcher"),
            Self::Cartographer => Some("minecraft:entity.villager.work_cartographer"),
            Self::Cleric => Some("minecraft:entity.villager.work_cleric"),
            Self::Farmer => Some("minecraft:entity.villager.work_farmer"),
            Self::Fisherman => Some("minecraft:entity.villager.work_fisherman"),
            Self::Fletcher => Some("minecraft:entity.villager.work_fletcher"),
            Self::Leatherworker => Some("minecraft:entity.villager.work_leatherworker"),
            Self::Librarian => Some("minecraft:entity.villager.work_librarian"),
            Self::Mason => Some("minecraft:entity.villager.work_mason"),
            Self::Shepherd => Some("minecraft:entity.villager.work_shepherd"),
            Self::Toolsmith => Some("minecraft:entity.villager.work_toolsmith"),
            Self::Weaponsmith => Some("minecraft:entity.villager.work_weaponsmith"),
        }
    }
}

pub const VILLAGER_MIN_LEVEL: i32 = 1;
pub const VILLAGER_MAX_LEVEL: i32 = 5;
pub const VILLAGER_LEVEL_XP_THRESHOLDS: [i32; 5] = [0, 10, 70, 150, 250];
pub const VILLAGER_ASSIGN_PROFESSION_CLEAR_TICK: bool = true;
pub const VILLAGER_UNHAPPY_COUNTER_TICKS: i32 = 40;
pub const VILLAGER_LEVEL_UP_DELAY_TICKS: i32 = 40;
pub const VILLAGER_LEVEL_UP_REGENERATION_TICKS: i32 = 200;
pub const VILLAGER_BREED_FOOD_THRESHOLD: i32 = 12;
pub const VILLAGER_EXCESS_FOOD_THRESHOLD: i32 = 24;
pub const VILLAGER_BREED_DIGEST_FOOD_POINTS: i32 = 12;
pub const VILLAGER_OFFSPRING_BIOME_TYPE_CHANCE: f64 = 0.5;
pub const VILLAGER_OFFSPRING_FIRST_PARENT_TYPE_CHANCE: f64 = 0.25;
pub const VILLAGER_RESTOCK_COOLDOWN_TICKS: i64 = 2400;
pub const VILLAGER_RESTOCK_HALF_DAY_TICKS: i64 = 12000;
pub const VILLAGER_MAX_RESTOCKS_PER_DAY: i32 = 2;
pub const HERO_OF_THE_VILLAGE_DISCOUNT_BASE: f64 = 0.3;
pub const HERO_OF_THE_VILLAGE_DISCOUNT_PER_AMPLIFIER: f64 = 0.0625;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VillagerDataModel {
    pub villager_type: &'static str,
    pub profession: VillagerProfession,
    pub level: i32,
}

impl VillagerDataModel {
    pub fn new(villager_type: &'static str, profession: VillagerProfession, level: i32) -> Self {
        Self {
            villager_type,
            profession,
            level: level.max(VILLAGER_MIN_LEVEL),
        }
    }

    pub fn default_plains() -> Self {
        Self::new(
            "minecraft:plains",
            VillagerProfession::None,
            VILLAGER_MIN_LEVEL,
        )
    }

    pub fn with_profession(self, profession: VillagerProfession) -> Self {
        Self { profession, ..self }
    }

    pub fn with_level(self, level: i32) -> Self {
        Self {
            level: level.max(VILLAGER_MIN_LEVEL),
            ..self
        }
    }
}

pub fn villager_min_xp_per_level(level: i32) -> i32 {
    if villager_can_level_up(level) {
        VILLAGER_LEVEL_XP_THRESHOLDS[(level - 1) as usize]
    } else {
        0
    }
}

pub fn villager_max_xp_per_level(level: i32) -> i32 {
    if villager_can_level_up(level) {
        VILLAGER_LEVEL_XP_THRESHOLDS[level as usize]
    } else {
        0
    }
}

pub fn villager_can_level_up(level: i32) -> bool {
    (VILLAGER_MIN_LEVEL..VILLAGER_MAX_LEVEL).contains(&level)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VillagerProfessionPlan {
    Keep {
        data: VillagerDataModel,
        clear_offers: bool,
        stop_trading: bool,
        release_job_site: bool,
    },
    Change {
        data: VillagerDataModel,
        clear_offers: bool,
        stop_trading: bool,
        acquire_job_site: bool,
        release_previous_job_site: bool,
    },
}

pub fn villager_set_profession_plan(
    data: VillagerDataModel,
    new_profession: VillagerProfession,
    is_trading: bool,
) -> VillagerProfessionPlan {
    if data.profession == new_profession {
        return VillagerProfessionPlan::Keep {
            data,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        };
    }

    let release_previous_job_site = data.profession.workstation().is_some();
    let next = data.with_profession(new_profession);
    VillagerProfessionPlan::Change {
        data: next,
        clear_offers: true,
        stop_trading: new_profession == VillagerProfession::None && is_trading,
        acquire_job_site: new_profession.workstation().is_some(),
        release_previous_job_site,
    }
}

pub fn villager_acquire_profession_from_poi(
    data: VillagerDataModel,
    poi_type: &str,
    poi_has_free_ticket: bool,
    is_baby: bool,
) -> VillagerProfessionPlan {
    if is_baby || data.profession != VillagerProfession::None || !poi_has_free_ticket {
        return VillagerProfessionPlan::Keep {
            data,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        };
    }

    if let Some(profession) = VillagerProfession::from_workstation(poi_type) {
        villager_set_profession_plan(data, profession, false)
    } else {
        VillagerProfessionPlan::Keep {
            data,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        }
    }
}

pub fn villager_lose_profession_after_job_site_removed(
    data: VillagerDataModel,
    job_site_poi_type: Option<&str>,
    is_trading: bool,
) -> VillagerProfessionPlan {
    if data.profession == VillagerProfession::None || data.profession == VillagerProfession::Nitwit
    {
        return VillagerProfessionPlan::Keep {
            data,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        };
    }

    if job_site_poi_type.is_some_and(|poi| data.profession.held_job_site_matches(poi)) {
        VillagerProfessionPlan::Keep {
            data,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        }
    } else {
        villager_set_profession_plan(data, VillagerProfession::None, is_trading)
    }
}

pub fn villager_finalize_spawn_profession(
    data: VillagerDataModel,
    spawn_reason: &str,
) -> (VillagerDataModel, bool) {
    match spawn_reason {
        "breeding" => (data.with_profession(VillagerProfession::None), false),
        "structure" => (data, true),
        _ => (data, false),
    }
}

pub fn villager_food_points(item: &str) -> Option<i32> {
    match item {
        "minecraft:bread" => Some(4),
        "minecraft:potato" | "minecraft:carrot" | "minecraft:beetroot" => Some(1),
        _ => None,
    }
}

pub fn villager_inventory_food_points(items: &[(&str, i32)]) -> i32 {
    items
        .iter()
        .map(|(item, count)| villager_food_points(item).unwrap_or(0) * (*count).max(0))
        .sum()
}

pub fn villager_can_breed(
    food_level: i32,
    inventory_food_points: i32,
    sleeping: bool,
    age: i32,
) -> bool {
    food_level + inventory_food_points >= VILLAGER_BREED_FOOD_THRESHOLD && !sleeping && age == 0
}

pub fn villager_hungry(food_level: i32) -> bool {
    food_level < VILLAGER_BREED_FOOD_THRESHOLD
}

pub fn villager_has_excess_food(inventory_food_points: i32) -> bool {
    inventory_food_points >= VILLAGER_EXCESS_FOOD_THRESHOLD
}

pub fn villager_wants_more_food(inventory_food_points: i32) -> bool {
    inventory_food_points < VILLAGER_BREED_FOOD_THRESHOLD
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VillagerEatPlan {
    pub food_level_after_eating: i32,
    pub consumed_items: i32,
    pub food_level_after_digest: i32,
}

pub fn villager_eat_and_digest_plan(food_level: i32, inventory: &[(&str, i32)]) -> VillagerEatPlan {
    let mut food_level_after_eating = food_level;
    let mut consumed_items = 0;
    if villager_hungry(food_level_after_eating) && villager_inventory_food_points(inventory) != 0 {
        for (item, count) in inventory {
            if let Some(points) = villager_food_points(item) {
                for _ in 0..(*count).max(0) {
                    food_level_after_eating += points;
                    consumed_items += 1;
                    if !villager_hungry(food_level_after_eating) {
                        return VillagerEatPlan {
                            food_level_after_eating,
                            consumed_items,
                            food_level_after_digest: food_level_after_eating
                                - VILLAGER_BREED_DIGEST_FOOD_POINTS,
                        };
                    }
                }
            }
        }
    }

    VillagerEatPlan {
        food_level_after_eating,
        consumed_items,
        food_level_after_digest: food_level_after_eating - VILLAGER_BREED_DIGEST_FOOD_POINTS,
    }
}

pub fn villager_wants_to_pick_up(
    item: &str,
    profession: VillagerProfession,
    inventory_can_add: bool,
) -> bool {
    let villager_pickup_tag = villager_food_points(item).is_some()
        || matches!(
            item,
            "minecraft:wheat_seeds" | "minecraft:beetroot_seeds" | "minecraft:bone_meal"
        );
    let profession_requested = profession == VillagerProfession::Farmer
        && matches!(
            item,
            "minecraft:wheat"
                | "minecraft:wheat_seeds"
                | "minecraft:beetroot_seeds"
                | "minecraft:bone_meal"
        );
    inventory_can_add && (villager_pickup_tag || profession_requested)
}

pub fn villager_offspring_type(
    biome_type: &'static str,
    first_parent_type: &'static str,
    second_parent_type: &'static str,
    random_double_0_to_1: f64,
) -> &'static str {
    if random_double_0_to_1 < VILLAGER_OFFSPRING_BIOME_TYPE_CHANCE {
        biome_type
    } else if random_double_0_to_1
        < VILLAGER_OFFSPRING_BIOME_TYPE_CHANCE + VILLAGER_OFFSPRING_FIRST_PARENT_TYPE_CHANCE
    {
        first_parent_type
    } else {
        second_parent_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VillagerRestockState {
    pub restocks_today: i32,
    pub last_restock_game_time: i64,
    pub last_restock_check_day: i64,
}

impl VillagerRestockState {
    pub fn new() -> Self {
        Self {
            restocks_today: 0,
            last_restock_game_time: 0,
            last_restock_check_day: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VillagerShouldRestockPlan {
    pub state: VillagerRestockState,
    pub should_restock: bool,
    pub reset_for_new_day: bool,
}

pub fn villager_allowed_to_restock(state: VillagerRestockState, game_time: i64) -> bool {
    state.restocks_today == 0
        || (state.restocks_today < VILLAGER_MAX_RESTOCKS_PER_DAY
            && game_time > state.last_restock_game_time + VILLAGER_RESTOCK_COOLDOWN_TICKS)
}

pub fn villager_should_restock_plan(
    mut state: VillagerRestockState,
    game_time: i64,
    current_day: i64,
    offers_need_restock: bool,
) -> VillagerShouldRestockPlan {
    let mut is_new_day = game_time > state.last_restock_game_time + VILLAGER_RESTOCK_HALF_DAY_TICKS;
    is_new_day |= state.last_restock_check_day > 0 && current_day > state.last_restock_check_day;
    state.last_restock_check_day = current_day;
    if is_new_day {
        state.last_restock_game_time = game_time;
        state.restocks_today = 0;
    }

    VillagerShouldRestockPlan {
        should_restock: villager_allowed_to_restock(state, game_time) && offers_need_restock,
        state,
        reset_for_new_day: is_new_day,
    }
}

pub fn villager_record_restock(
    mut state: VillagerRestockState,
    game_time: i64,
) -> VillagerRestockState {
    state.last_restock_game_time = game_time;
    state.restocks_today += 1;
    state
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VillagerCatchUpDemandPlan {
    pub missed_updates: i32,
    pub reset_uses: bool,
    pub demand_updates: i32,
    pub resend_offers: bool,
}

pub fn villager_catch_up_demand_plan(restocks_today: i32) -> VillagerCatchUpDemandPlan {
    let missed_updates = (VILLAGER_MAX_RESTOCKS_PER_DAY - restocks_today).max(0);
    VillagerCatchUpDemandPlan {
        missed_updates,
        reset_uses: missed_updates > 0,
        demand_updates: missed_updates,
        resend_offers: true,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VillagerTradeState {
    pub profession: VillagerProfession,
    pub level: VillagerLevel,
    pub xp: i32,
    pub offers: Vec<MerchantOffer>,
    pub gossip: GossipContainer,
    pub last_traded_player: Option<String>,
    pub update_merchant_timer: i32,
    pub increase_profession_level_on_update: bool,
    pub restocks_today: i32,
    pub last_restock_game_time: i64,
    pub workstation: Option<PoiTicket>,
}

impl VillagerTradeState {
    pub fn new(profession: VillagerProfession) -> Self {
        Self {
            profession,
            level: VillagerLevel::Novice,
            xp: 0,
            offers: Vec::new(),
            gossip: GossipContainer::new(),
            last_traded_player: None,
            update_merchant_timer: 0,
            increase_profession_level_on_update: false,
            restocks_today: 0,
            last_restock_game_time: 0,
            workstation: profession.workstation().map(|poi_type| PoiTicket {
                poi_type,
                max_tickets: 1,
                free_tickets: 1,
            }),
        }
    }

    pub fn assign_workstation(&mut self) -> bool {
        self.workstation.as_mut().is_some_and(PoiTicket::acquire)
    }

    pub fn release_workstation(&mut self) {
        if let Some(workstation) = &mut self.workstation {
            workstation.release();
        }
    }

    pub fn generate_level_offers(&mut self) {
        let templates = profession_offers(self.profession, self.level);
        for template in templates {
            if !self.offers.iter().any(|offer| offer == &template) {
                self.offers.push(template);
            }
        }
    }

    pub fn trade(&mut self, offer_index: usize, player: &str) -> Option<ItemStack> {
        let offer = self.offers.get_mut(offer_index)?;
        if offer.is_out_of_stock() {
            return None;
        }
        let result = offer.assemble();
        offer.increase_uses();
        self.xp += offer.xp;
        self.last_traded_player = Some(player.to_string());
        if self.should_increase_level() {
            self.update_merchant_timer = VILLAGER_LEVEL_UP_DELAY_TICKS;
            self.increase_profession_level_on_update = true;
        }
        self.gossip.add(player, GossipType::Trading, 2);
        Some(result)
    }

    pub fn should_increase_level(&self) -> bool {
        let current_level = self.level as i32;
        villager_can_level_up(current_level) && self.xp >= villager_max_xp_per_level(current_level)
    }

    /// Advance the delayed trade-level-up timer.
    ///
    /// Java `Villager.customServerAiStep` only decrements `updateMerchantTimer`
    /// while the villager is not trading. When the timer reaches zero,
    /// `increaseMerchantCareer` increments the profession level and adds the
    /// next level's trade set.
    pub fn tick_level_progression(&mut self, is_trading: bool) -> bool {
        if is_trading || self.update_merchant_timer <= 0 {
            return false;
        }
        self.update_merchant_timer -= 1;
        if self.update_merchant_timer > 0 {
            return false;
        }
        if self.increase_profession_level_on_update {
            self.increase_profession_level_on_update = false;
            self.increase_merchant_career();
            return true;
        }
        false
    }

    fn increase_merchant_career(&mut self) {
        self.level = match self.level {
            VillagerLevel::Novice => VillagerLevel::Apprentice,
            VillagerLevel::Apprentice => VillagerLevel::Journeyman,
            VillagerLevel::Journeyman => VillagerLevel::Expert,
            VillagerLevel::Expert => VillagerLevel::Master,
            VillagerLevel::Master => VillagerLevel::Master,
        };
        self.generate_level_offers();
    }

    pub fn restock(&mut self, game_time: i64) -> bool {
        if self.restock_window_changed(game_time) {
            self.restocks_today = 0;
        }
        if self.restocks_today >= 2 {
            return false;
        }
        if !self.offers.iter().any(MerchantOffer::needs_restock) {
            return false;
        }
        for offer in &mut self.offers {
            offer.update_demand();
            offer.reset_uses();
        }
        self.restocks_today += 1;
        self.last_restock_game_time = game_time;
        true
    }

    fn restock_window_changed(&self, game_time: i64) -> bool {
        game_time / 24_000 != self.last_restock_game_time / 24_000
    }

    pub fn apply_reputation_prices(&mut self, player: &str) {
        let reputation = self.gossip.reputation(player);
        if reputation != 0 {
            for offer in &mut self.offers {
                offer.special_price_diff -=
                    (reputation as f32 * offer.price_multiplier).floor() as i32;
            }
        }
    }

    pub fn apply_hero_of_the_village_prices(&mut self, amplifier: i32) {
        let modifier = HERO_OF_THE_VILLAGE_DISCOUNT_BASE
            + HERO_OF_THE_VILLAGE_DISCOUNT_PER_AMPLIFIER * amplifier as f64;
        for offer in &mut self.offers {
            let cost_reduction = (modifier * offer.base_cost_a.count as f64).floor() as i32;
            offer.special_price_diff -= cost_reduction.max(1);
        }
    }

    pub fn reset_special_prices(&mut self) {
        for offer in &mut self.offers {
            offer.special_price_diff = 0;
        }
    }
}

pub fn profession_offers(
    profession: VillagerProfession,
    level: VillagerLevel,
) -> Vec<MerchantOffer> {
    crate::villager_trade_resources::profession_offers_from_default_data(profession, level)
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq)]
pub struct WanderingTraderOffers {
    pub buying: Vec<MerchantOffer>,
    pub uncommon: Vec<MerchantOffer>,
    pub common: Vec<MerchantOffer>,
}

impl WanderingTraderOffers {
    pub fn vanilla_sample() -> Self {
        Self::from_default_data().unwrap_or_else(|_| Self::fallback_sample())
    }

    pub fn from_default_data() -> Result<Self, String> {
        let resources = crate::villager_trade_resources::load_villager_trade_data_root(
            crate::villager_trade_resources::configured_vanilla_data_root()?,
        )?;
        Self::from_resources(&resources)
    }

    pub fn from_resources(resources: &crate::resources::DataResourceIndex) -> Result<Self, String> {
        let (buying, uncommon, common) =
            crate::villager_trade_resources::wandering_trader_offers_from_resources(resources)?;
        Ok(Self {
            buying,
            uncommon,
            common,
        })
    }

    fn fallback_sample() -> Self {
        Self {
            buying: Vec::new(),
            common: vec![
                MerchantOffer::new(
                    ItemCost::new("minecraft:emerald", 1),
                    None,
                    ItemStack::new("minecraft:red_sand", 4),
                    8,
                    1,
                    0.05,
                ),
                MerchantOffer::new(
                    ItemCost::new("minecraft:emerald", 3),
                    None,
                    ItemStack::new("minecraft:packed_ice", 1),
                    6,
                    1,
                    0.05,
                ),
            ],
            uncommon: vec![MerchantOffer::new(
                ItemCost::new("minecraft:emerald", 5),
                None,
                ItemStack::new("minecraft:nautilus_shell", 1),
                5,
                1,
                0.05,
            )],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WanderingTraderSpawnPlan {
    pub should_spawn: bool,
    pub next_data: WanderingTraderData,
}

pub const WANDERING_TRADER_SPAWNER_TICK_DELAY: i32 = 1200;
pub const WANDERING_TRADER_MIN_SPAWN_CHANCE: i32 = 25;
pub const WANDERING_TRADER_MAX_SPAWN_CHANCE: i32 = 75;
pub const WANDERING_TRADER_SPAWN_CHANCE_INCREASE: i32 = 25;
pub const WANDERING_TRADER_ONE_IN_X_CHANCE: i32 = 10;
pub const WANDERING_TRADER_SPAWN_ATTEMPTS: i32 = 10;
pub const WANDERING_TRADER_SPAWN_RADIUS: i32 = 48;
pub const WANDERING_TRADER_MEETING_POI_RADIUS: i32 = 48;
pub const WANDERING_TRADER_LLAMA_COUNT: i32 = 2;
pub const WANDERING_TRADER_LLAMA_SPAWN_RADIUS: i32 = 4;
pub const WANDERING_TRADER_DESPAWN_DELAY: i32 = 48_000;
pub const WANDERING_TRADER_HOME_RADIUS: i32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WanderingTraderSpawnerState {
    pub tick_delay: i32,
    pub data: WanderingTraderData,
}

impl Default for WanderingTraderSpawnerState {
    fn default() -> Self {
        Self {
            tick_delay: WANDERING_TRADER_SPAWNER_TICK_DELAY,
            data: WanderingTraderData::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WanderingTraderSpawnContext {
    pub spawn_traders_rule: bool,
    pub random_player_present: bool,
    pub outer_chance_roll_0_to_99: i32,
    pub one_in_ten_roll: i32,
    pub spawn_position_found: bool,
    pub has_enough_space: bool,
    pub biome_allows_spawn: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WanderingTraderSpawnerTickPlan {
    pub state: WanderingTraderSpawnerState,
    pub spawn_trader: bool,
    pub spawn_llamas: i32,
    pub trader_despawn_delay: Option<i32>,
    pub trader_home_radius: Option<i32>,
    pub returns_spawn_success: bool,
}

pub fn wandering_trader_spawner_tick(
    state: WanderingTraderSpawnerState,
    ctx: WanderingTraderSpawnContext,
) -> WanderingTraderSpawnerTickPlan {
    let mut plan = WanderingTraderSpawnerTickPlan {
        state,
        spawn_trader: false,
        spawn_llamas: 0,
        trader_despawn_delay: None,
        trader_home_radius: None,
        returns_spawn_success: false,
    };

    if !ctx.spawn_traders_rule {
        return plan;
    }

    plan.state.tick_delay -= 1;
    if plan.state.tick_delay > 0 {
        return plan;
    }

    plan.state.tick_delay = WANDERING_TRADER_SPAWNER_TICK_DELAY;
    plan.state.data.spawn_delay -= WANDERING_TRADER_SPAWNER_TICK_DELAY;
    if plan.state.data.spawn_delay > 0 {
        return plan;
    }

    let chance_to_spawn = plan.state.data.spawn_chance;
    plan.state.data.spawn_delay = crate::spawning::DEFAULT_WANDERING_TRADER_SPAWN_DELAY;
    plan.state.data.spawn_chance = (chance_to_spawn + WANDERING_TRADER_SPAWN_CHANCE_INCREASE)
        .clamp(
            WANDERING_TRADER_MIN_SPAWN_CHANCE,
            WANDERING_TRADER_MAX_SPAWN_CHANCE,
        );

    if ctx.outer_chance_roll_0_to_99 > chance_to_spawn {
        return plan;
    }

    plan.returns_spawn_success = wandering_trader_spawn_attempt_success(ctx);
    if plan.returns_spawn_success {
        plan.state.data.spawn_chance = WANDERING_TRADER_MIN_SPAWN_CHANCE;
        if ctx.random_player_present
            && ctx.one_in_ten_roll == 0
            && ctx.spawn_position_found
            && ctx.has_enough_space
            && ctx.biome_allows_spawn
        {
            plan.spawn_trader = true;
            plan.spawn_llamas = WANDERING_TRADER_LLAMA_COUNT;
            plan.trader_despawn_delay = Some(WANDERING_TRADER_DESPAWN_DELAY);
            plan.trader_home_radius = Some(WANDERING_TRADER_HOME_RADIUS);
        }
    }

    plan
}

pub fn wandering_trader_spawn_attempt_success(ctx: WanderingTraderSpawnContext) -> bool {
    if !ctx.random_player_present {
        return true;
    }
    ctx.one_in_ten_roll == 0
        && ctx.spawn_position_found
        && ctx.has_enough_space
        && ctx.biome_allows_spawn
}

pub fn wandering_trader_candidate_offset(
    random_0_to_radius_times_two_minus_one: i32,
    radius: i32,
) -> i32 {
    random_0_to_radius_times_two_minus_one.clamp(0, radius * 2 - 1) - radius
}

pub fn wandering_trader_has_enough_space(collision_shapes_empty: &[bool]) -> bool {
    collision_shapes_empty.len() == 12 && collision_shapes_empty.iter().all(|empty| *empty)
}

pub fn wandering_trader_should_despawn(despawn_delay: i32, trading: bool) -> (i32, bool) {
    if despawn_delay > 0 && !trading {
        let next = despawn_delay - 1;
        (next, next == 0)
    } else {
        (despawn_delay, false)
    }
}

pub fn wandering_trader_remove_when_far_away() -> bool {
    false
}

pub fn wandering_trader_tick(
    mut data: WanderingTraderData,
    spawn_traders_rule: bool,
    has_players: bool,
    random_roll: i32,
) -> WanderingTraderSpawnPlan {
    if !spawn_traders_rule || !has_players || data.spawn_delay > 1 {
        if spawn_traders_rule && has_players {
            data.spawn_delay -= 1;
        }
        return WanderingTraderSpawnPlan {
            should_spawn: false,
            next_data: data,
        };
    }
    let should_spawn = random_roll <= data.spawn_chance;
    data.spawn_delay = crate::spawning::DEFAULT_WANDERING_TRADER_SPAWN_DELAY;
    data.spawn_chance = if should_spawn {
        WANDERING_TRADER_MIN_SPAWN_CHANCE
    } else {
        (data.spawn_chance + WANDERING_TRADER_SPAWN_CHANCE_INCREASE)
            .min(WANDERING_TRADER_MAX_SPAWN_CHANCE)
    };
    WanderingTraderSpawnPlan {
        should_spawn,
        next_data: data,
    }
}

#[cfg(test)]
mod tests;
