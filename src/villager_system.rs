#![allow(dead_code)]

use std::path::Path;

use crate::ai_system::{GossipContainer, GossipType, PoiTicket};
use crate::item_stack::ItemStack;
use crate::player_inventory::{ItemCost, MerchantOffer};
use crate::spawning::WanderingTraderData;

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
            if !self
                .offers
                .iter()
                .any(|offer| offer.result.item_id() == template.result.item_id())
            {
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
        self.level = VillagerLevel::from_xp(self.xp);
        self.gossip.add(player, GossipType::Trading, 2);
        Some(result)
    }

    pub fn restock(&mut self, game_time: i64) -> bool {
        if self.restock_window_changed(game_time) {
            self.restocks_today = 0;
        }
        if self.restocks_today >= 2 {
            return false;
        }
        let mut changed = false;
        for offer in &mut self.offers {
            if offer.needs_restock() {
                offer.update_demand();
                offer.reset_uses();
                changed = true;
            }
        }
        if changed {
            self.restocks_today += 1;
            self.last_restock_game_time = game_time;
        }
        changed
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
    match (profession, level) {
        (VillagerProfession::Farmer, VillagerLevel::Novice) => vec![
            MerchantOffer::new(
                ItemCost::new("minecraft:wheat", 20),
                None,
                ItemStack::new("minecraft:emerald", 1),
                16,
                2,
                0.05,
            ),
            MerchantOffer::new(
                ItemCost::new("minecraft:emerald", 1),
                None,
                ItemStack::new("minecraft:bread", 6),
                16,
                1,
                0.05,
            ),
        ],
        (VillagerProfession::Librarian, VillagerLevel::Novice) => vec![MerchantOffer::new(
            ItemCost::new("minecraft:paper", 24),
            None,
            ItemStack::new("minecraft:emerald", 1),
            16,
            2,
            0.05,
        )],
        (VillagerProfession::Toolsmith, VillagerLevel::Apprentice) => vec![MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 4),
            None,
            ItemStack::new("minecraft:stone_pickaxe", 1),
            12,
            10,
            0.2,
        )],
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VillagerTradeResource {
    pub wants: TradeCostResource,
    pub additional_wants: Option<TradeCostResource>,
    pub gives: ItemStackTemplateResource,
    pub max_uses: NumberProviderResource,
    pub reputation_discount: NumberProviderResource,
    pub xp: NumberProviderResource,
    pub merchant_predicate: Option<serde_json::Value>,
    pub given_item_modifiers: Vec<serde_json::Value>,
    pub double_trade_price_enchantments: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeSetResource {
    pub trades: HolderSetResource,
    pub amount: NumberProviderResource,
    pub allow_duplicates: bool,
    pub random_sequence: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeCostResource {
    pub item_id: String,
    pub count: NumberProviderResource,
    pub components: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackTemplateResource {
    pub item_id: String,
    pub count: i32,
    pub components: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberProviderResource {
    Constant(f64),
    Provider(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetResource {
    Tag(String),
    List(Vec<String>),
}

pub fn parse_villager_trade_resource(raw: &str) -> Result<VillagerTradeResource, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid villager trade JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "villager trade must be a JSON object".to_string())?;

    Ok(VillagerTradeResource {
        wants: parse_trade_cost(required_value(object, "wants")?)?,
        additional_wants: object
            .get("additional_wants")
            .map(parse_trade_cost)
            .transpose()?,
        gives: parse_item_stack_template(required_value(object, "gives")?)?,
        max_uses: object
            .get("max_uses")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(4.0)),
        reputation_discount: object
            .get("reputation_discount")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(0.0)),
        xp: object
            .get("xp")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(1.0)),
        merchant_predicate: object.get("merchant_predicate").cloned(),
        given_item_modifiers: object
            .get("given_item_modifiers")
            .map(parse_value_list)
            .transpose()?
            .unwrap_or_default(),
        double_trade_price_enchantments: object
            .get("double_trade_price_enchantments")
            .map(parse_holder_set_id)
            .transpose()?,
    })
}

pub fn parse_trade_set_resource(raw: &str) -> Result<TradeSetResource, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid trade set JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "trade set must be a JSON object".to_string())?;

    Ok(TradeSetResource {
        trades: parse_holder_set(required_value(object, "trades")?)?,
        amount: parse_number_provider(required_value(object, "amount")?)?,
        allow_duplicates: object
            .get("allow_duplicates")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        random_sequence: object
            .get("random_sequence")
            .map(json_string_value)
            .transpose()?,
    })
}

pub fn load_villager_trade_resource(
    path: impl AsRef<Path>,
) -> Result<VillagerTradeResource, String> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|err| format!("failed to read {}: {err}", path.as_ref().display()))?;
    parse_villager_trade_resource(&raw)
}

pub fn load_trade_set_resource(path: impl AsRef<Path>) -> Result<TradeSetResource, String> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|err| format!("failed to read {}: {err}", path.as_ref().display()))?;
    parse_trade_set_resource(&raw)
}

fn parse_trade_cost(value: &serde_json::Value) -> Result<TradeCostResource, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "trade cost must be a JSON object".to_string())?;
    Ok(TradeCostResource {
        item_id: json_string(object, "id")?,
        count: object
            .get("count")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(1.0)),
        components: object
            .get("components")
            .map(json_object_value)
            .transpose()?,
    })
}

fn parse_item_stack_template(
    value: &serde_json::Value,
) -> Result<ItemStackTemplateResource, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "item stack template must be a JSON object".to_string())?;
    Ok(ItemStackTemplateResource {
        item_id: json_string(object, "id")?,
        count: object
            .get("count")
            .map(json_i32_value)
            .transpose()?
            .unwrap_or(1),
        components: object
            .get("components")
            .map(json_object_value)
            .transpose()?,
    })
}

fn parse_number_provider(value: &serde_json::Value) -> Result<NumberProviderResource, String> {
    if let Some(number) = value.as_f64() {
        return Ok(NumberProviderResource::Constant(number));
    }
    if value.as_object().is_some() {
        return Ok(NumberProviderResource::Provider(value.clone()));
    }
    Err("number provider must be a number or object".to_string())
}

fn parse_holder_set(value: &serde_json::Value) -> Result<HolderSetResource, String> {
    if let Some(id) = value.as_str() {
        return if let Some(tag) = id.strip_prefix('#') {
            Ok(HolderSetResource::Tag(tag.to_string()))
        } else {
            Ok(HolderSetResource::List(vec![id.to_string()]))
        };
    }
    let entries = value
        .as_array()
        .ok_or_else(|| "holder set must be a tag string, id string, or list".to_string())?;
    entries
        .iter()
        .map(json_string_value)
        .collect::<Result<Vec<_>, _>>()
        .map(HolderSetResource::List)
}

fn parse_holder_set_id(value: &serde_json::Value) -> Result<String, String> {
    match parse_holder_set(value)? {
        HolderSetResource::Tag(tag) => Ok(format!("#{tag}")),
        HolderSetResource::List(entries) => Ok(entries.join(",")),
    }
}

fn parse_value_list(value: &serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    value
        .as_array()
        .cloned()
        .ok_or_else(|| "value must be a list".to_string())
}

fn required_value<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a serde_json::Value, String> {
    object
        .get(field)
        .ok_or_else(|| format!("{field} is required"))
}

fn json_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, String> {
    object
        .get(field)
        .map(json_string_value)
        .transpose()?
        .ok_or_else(|| format!("{field} must be a string"))
}

fn json_string_value(value: &serde_json::Value) -> Result<String, String> {
    value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| "value must be a string".to_string())
}

fn json_i32_value(value: &serde_json::Value) -> Result<i32, String> {
    let integer = value
        .as_i64()
        .ok_or_else(|| "value must be an integer".to_string())?;
    i32::try_from(integer).map_err(|_| "value is out of i32 range".to_string())
}

fn json_object_value(
    value: &serde_json::Value,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    value
        .as_object()
        .cloned()
        .ok_or_else(|| "value must be an object".to_string())
}

#[derive(Debug, Clone, PartialEq)]
pub struct WanderingTraderOffers {
    pub generic: Vec<MerchantOffer>,
    pub rare: Vec<MerchantOffer>,
}

impl WanderingTraderOffers {
    pub fn vanilla_sample() -> Self {
        Self {
            generic: vec![
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
            rare: vec![MerchantOffer::new(
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
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;

    #[test]
    fn professions_expose_workstations_and_generate_level_offers() {
        let mut farmer = VillagerTradeState::new(VillagerProfession::Farmer);
        assert_eq!(farmer.profession.workstation(), Some("minecraft:composter"));
        assert!(farmer.assign_workstation());
        assert!(!farmer.assign_workstation());

        farmer.generate_level_offers();
        assert_eq!(farmer.offers.len(), 2);
        assert!(farmer
            .offers
            .iter()
            .any(|offer| offer.result.item_id() == "minecraft:bread"));

        farmer.release_workstation();
        assert!(farmer.assign_workstation());
    }

    #[test]
    fn villager_profession_assignment_and_job_site_loss_match_java() {
        assert_eq!(VILLAGER_MIN_LEVEL, 1);
        assert_eq!(VILLAGER_MAX_LEVEL, 5);
        assert_eq!(VILLAGER_LEVEL_XP_THRESHOLDS, [0, 10, 70, 150, 250]);
        assert!(VILLAGER_ASSIGN_PROFESSION_CLEAR_TICK);
        assert_eq!(VILLAGER_UNHAPPY_COUNTER_TICKS, 40);
        assert_eq!(VILLAGER_LEVEL_UP_DELAY_TICKS, 40);
        assert_eq!(VILLAGER_LEVEL_UP_REGENERATION_TICKS, 200);
        assert_eq!(VillagerLevel::from_xp(9), VillagerLevel::Novice);
        assert_eq!(VillagerLevel::from_xp(10), VillagerLevel::Apprentice);
        assert_eq!(villager_min_xp_per_level(1), 0);
        assert_eq!(villager_max_xp_per_level(1), 10);
        assert_eq!(villager_max_xp_per_level(5), 0);
        assert!(villager_can_level_up(4));
        assert!(!villager_can_level_up(5));

        assert_eq!(
            VillagerProfession::from_workstation("minecraft:lectern"),
            Some(VillagerProfession::Librarian)
        );
        assert_eq!(VillagerProfession::from_workstation("minecraft:bed"), None);
        assert!(VillagerProfession::None.acquirable_job_site_matches("minecraft:composter"));
        assert!(VillagerProfession::Farmer.held_job_site_matches("minecraft:composter"));
        assert!(!VillagerProfession::Farmer.held_job_site_matches("minecraft:lectern"));
        assert_eq!(
            VillagerProfession::Toolsmith.work_sound(),
            Some("minecraft:entity.villager.work_toolsmith")
        );

        let none = VillagerDataModel::default_plains();
        assert_eq!(none.level, 1);
        assert_eq!(
            VillagerDataModel::new("minecraft:desert", VillagerProfession::Farmer, -5).level,
            1
        );

        assert_eq!(
            villager_acquire_profession_from_poi(none, "minecraft:lectern", true, false),
            VillagerProfessionPlan::Change {
                data: none.with_profession(VillagerProfession::Librarian),
                clear_offers: true,
                stop_trading: false,
                acquire_job_site: true,
                release_previous_job_site: false,
            }
        );
        assert_eq!(
            villager_acquire_profession_from_poi(none, "minecraft:lectern", false, false),
            VillagerProfessionPlan::Keep {
                data: none,
                clear_offers: false,
                stop_trading: false,
                release_job_site: false,
            }
        );
        assert_eq!(
            villager_acquire_profession_from_poi(none, "minecraft:lectern", true, true),
            VillagerProfessionPlan::Keep {
                data: none,
                clear_offers: false,
                stop_trading: false,
                release_job_site: false,
            }
        );

        let librarian = none.with_profession(VillagerProfession::Librarian);
        assert_eq!(
            villager_set_profession_plan(librarian, VillagerProfession::Farmer, false),
            VillagerProfessionPlan::Change {
                data: librarian.with_profession(VillagerProfession::Farmer),
                clear_offers: true,
                stop_trading: false,
                acquire_job_site: true,
                release_previous_job_site: true,
            }
        );
        assert_eq!(
            villager_set_profession_plan(librarian, VillagerProfession::Librarian, false),
            VillagerProfessionPlan::Keep {
                data: librarian,
                clear_offers: false,
                stop_trading: false,
                release_job_site: false,
            }
        );
        assert_eq!(
            villager_lose_profession_after_job_site_removed(
                librarian,
                Some("minecraft:lectern"),
                true
            ),
            VillagerProfessionPlan::Keep {
                data: librarian,
                clear_offers: false,
                stop_trading: false,
                release_job_site: false,
            }
        );
        assert_eq!(
            villager_lose_profession_after_job_site_removed(librarian, None, true),
            VillagerProfessionPlan::Change {
                data: librarian.with_profession(VillagerProfession::None),
                clear_offers: true,
                stop_trading: true,
                acquire_job_site: false,
                release_previous_job_site: true,
            }
        );
        assert_eq!(
            villager_finalize_spawn_profession(librarian, "breeding"),
            (librarian.with_profession(VillagerProfession::None), false)
        );
        assert_eq!(
            villager_finalize_spawn_profession(librarian, "structure"),
            (librarian, true)
        );
    }

    #[test]
    fn villager_breeding_food_pickup_and_offspring_type_match_java() {
        assert_eq!(VILLAGER_BREED_FOOD_THRESHOLD, 12);
        assert_eq!(VILLAGER_EXCESS_FOOD_THRESHOLD, 24);
        assert_eq!(VILLAGER_BREED_DIGEST_FOOD_POINTS, 12);
        assert_eq!(VILLAGER_OFFSPRING_BIOME_TYPE_CHANCE, 0.5);
        assert_eq!(VILLAGER_OFFSPRING_FIRST_PARENT_TYPE_CHANCE, 0.25);

        assert_eq!(villager_food_points("minecraft:bread"), Some(4));
        assert_eq!(villager_food_points("minecraft:potato"), Some(1));
        assert_eq!(villager_food_points("minecraft:carrot"), Some(1));
        assert_eq!(villager_food_points("minecraft:beetroot"), Some(1));
        assert_eq!(villager_food_points("minecraft:apple"), None);
        assert_eq!(
            villager_inventory_food_points(&[
                ("minecraft:bread", 2),
                ("minecraft:carrot", 3),
                ("minecraft:stone", 99),
            ]),
            11
        );

        assert!(villager_can_breed(4, 8, false, 0));
        assert!(!villager_can_breed(4, 7, false, 0));
        assert!(!villager_can_breed(12, 0, true, 0));
        assert!(!villager_can_breed(12, 0, false, -24000));
        assert!(villager_hungry(11));
        assert!(!villager_hungry(12));
        assert!(villager_has_excess_food(24));
        assert!(!villager_has_excess_food(23));
        assert!(villager_wants_more_food(11));
        assert!(!villager_wants_more_food(12));

        assert_eq!(
            villager_eat_and_digest_plan(
                5,
                &[
                    ("minecraft:stone", 64),
                    ("minecraft:bread", 2),
                    ("minecraft:carrot", 10)
                ]
            ),
            VillagerEatPlan {
                food_level_after_eating: 13,
                consumed_items: 2,
                food_level_after_digest: 1,
            }
        );
        assert_eq!(
            villager_eat_and_digest_plan(12, &[("minecraft:bread", 1)]),
            VillagerEatPlan {
                food_level_after_eating: 12,
                consumed_items: 0,
                food_level_after_digest: 0,
            }
        );

        assert!(villager_wants_to_pick_up(
            "minecraft:bread",
            VillagerProfession::None,
            true
        ));
        assert!(villager_wants_to_pick_up(
            "minecraft:wheat",
            VillagerProfession::Farmer,
            true
        ));
        assert!(!villager_wants_to_pick_up(
            "minecraft:wheat",
            VillagerProfession::Librarian,
            true
        ));
        assert!(!villager_wants_to_pick_up(
            "minecraft:bread",
            VillagerProfession::None,
            false
        ));

        assert_eq!(
            villager_offspring_type(
                "minecraft:savanna",
                "minecraft:plains",
                "minecraft:desert",
                0.49
            ),
            "minecraft:savanna"
        );
        assert_eq!(
            villager_offspring_type(
                "minecraft:savanna",
                "minecraft:plains",
                "minecraft:desert",
                0.5
            ),
            "minecraft:plains"
        );
        assert_eq!(
            villager_offspring_type(
                "minecraft:savanna",
                "minecraft:plains",
                "minecraft:desert",
                0.75
            ),
            "minecraft:desert"
        );
    }

    #[test]
    fn villager_restock_timing_new_day_and_catchup_match_java() {
        assert_eq!(VILLAGER_RESTOCK_COOLDOWN_TICKS, 2400);
        assert_eq!(VILLAGER_RESTOCK_HALF_DAY_TICKS, 12000);
        assert_eq!(VILLAGER_MAX_RESTOCKS_PER_DAY, 2);

        let initial = VillagerRestockState::new();
        assert!(villager_allowed_to_restock(initial, 0));
        let first = villager_record_restock(initial, 100);
        assert_eq!(
            first,
            VillagerRestockState {
                restocks_today: 1,
                last_restock_game_time: 100,
                last_restock_check_day: 0,
            }
        );
        assert!(!villager_allowed_to_restock(first, 2500));
        assert!(villager_allowed_to_restock(first, 2501));
        let second = villager_record_restock(first, 2501);
        assert!(!villager_allowed_to_restock(second, 9999));

        assert_eq!(
            villager_should_restock_plan(first, 2501, 0, true),
            VillagerShouldRestockPlan {
                state: VillagerRestockState {
                    restocks_today: 1,
                    last_restock_game_time: 100,
                    last_restock_check_day: 0,
                },
                should_restock: true,
                reset_for_new_day: false,
            }
        );
        assert_eq!(
            villager_should_restock_plan(first, 2501, 0, false).should_restock,
            false
        );
        assert_eq!(
            villager_should_restock_plan(second, 14_502, 0, true),
            VillagerShouldRestockPlan {
                state: VillagerRestockState {
                    restocks_today: 0,
                    last_restock_game_time: 14_502,
                    last_restock_check_day: 0,
                },
                should_restock: true,
                reset_for_new_day: true,
            }
        );
        assert_eq!(
            villager_should_restock_plan(
                VillagerRestockState {
                    restocks_today: 2,
                    last_restock_game_time: 10_000,
                    last_restock_check_day: 1,
                },
                10_100,
                2,
                true,
            ),
            VillagerShouldRestockPlan {
                state: VillagerRestockState {
                    restocks_today: 0,
                    last_restock_game_time: 10_100,
                    last_restock_check_day: 2,
                },
                should_restock: true,
                reset_for_new_day: true,
            }
        );
        assert_eq!(
            villager_catch_up_demand_plan(0),
            VillagerCatchUpDemandPlan {
                missed_updates: 2,
                reset_uses: true,
                demand_updates: 2,
                resend_offers: true,
            }
        );
        assert_eq!(
            villager_catch_up_demand_plan(2),
            VillagerCatchUpDemandPlan {
                missed_updates: 0,
                reset_uses: false,
                demand_updates: 0,
                resend_offers: true,
            }
        );
    }

    #[test]
    fn trading_adds_xp_gossip_demand_and_restock_caps_twice_per_day() {
        let mut villager = VillagerTradeState::new(VillagerProfession::Librarian);
        villager.generate_level_offers();
        villager.offers[0].max_uses = 1;

        assert_eq!(
            villager.trade(0, "player-a").unwrap().item_id(),
            "minecraft:emerald"
        );
        assert!(villager.offers[0].is_out_of_stock());
        assert_eq!(villager.xp, 2);
        assert_eq!(villager.gossip.reputation("player-a"), 2);

        assert!(villager.restock(1_000));
        assert!(!villager.offers[0].is_out_of_stock());
        assert_eq!(villager.offers[0].demand, 1);
        assert_eq!(villager.restock(1_100), false);

        villager.trade(0, "player-a");
        assert!(villager.restock(1_200));
        villager.trade(0, "player-a");
        assert!(!villager.restock(1_300));
        assert!(villager.restock(25_000));
    }

    #[test]
    fn reputation_changes_special_prices_and_level_unlocks_follow_xp_thresholds() {
        let mut villager = VillagerTradeState::new(VillagerProfession::Toolsmith);
        villager.xp = 10;
        villager.level = VillagerLevel::from_xp(villager.xp);
        villager.generate_level_offers();
        villager
            .gossip
            .add("player-a", GossipType::MajorPositive, 20);

        villager.apply_reputation_prices("player-a");

        assert_eq!(villager.level, VillagerLevel::Apprentice);
        assert_eq!(villager.offers[0].special_price_diff, -20);
        assert_eq!(villager.offers[0].cost_a_count(), 1);
    }

    #[test]
    fn zombie_villager_cure_duration_and_cured_reputation_discount_values_match_java() {
        assert_eq!(
            crate::mob_interaction::zombie_villager_conversion_time_from_roll(0),
            3600
        );
        assert_eq!(
            crate::mob_interaction::zombie_villager_conversion_time_from_roll(2400),
            6000
        );
        assert!(
            crate::mob_interaction::zombie_villager_finish_conversion(true, true, false)
                .emit_reputation_event
        );

        let player = "curing-player";
        let mut villager = VillagerTradeState::new(VillagerProfession::Toolsmith);
        villager.xp = 10;
        villager.level = VillagerLevel::from_xp(villager.xp);
        villager.generate_level_offers();

        // Villager.java#onReputationEventFrom adds both cure gossips, and
        // updateSpecialPrices applies -floor(reputation * offer.priceMultiplier).
        villager.gossip.add(player, GossipType::MajorPositive, 20);
        villager.gossip.add(player, GossipType::MinorPositive, 25);
        assert_eq!(villager.gossip.reputation(player), 125);

        villager.apply_reputation_prices(player);

        assert_eq!(villager.offers[0].price_multiplier, 0.2);
        assert_eq!(villager.offers[0].special_price_diff, -25);
        assert_eq!(villager.offers[0].cost_a_count(), 1);
    }

    #[test]
    fn wandering_trader_spawn_data_updates_chance_and_sample_trade_groups() {
        let trades = WanderingTraderOffers::vanilla_sample();
        assert!(!trades.generic.is_empty());
        assert!(!trades.rare.is_empty());

        let data = WanderingTraderData {
            spawn_delay: 1,
            spawn_chance: 50,
        };
        let spawned = wandering_trader_tick(data.clone(), true, true, 10);
        assert!(spawned.should_spawn);
        assert_eq!(spawned.next_data.spawn_chance, 25);

        let missed = wandering_trader_tick(data, true, true, 99);
        assert!(!missed.should_spawn);
        assert_eq!(missed.next_data.spawn_chance, 75);
    }

    #[test]
    fn wandering_trader_spawner_timing_attempts_llamas_and_despawn_match_java() {
        assert_eq!(WANDERING_TRADER_SPAWNER_TICK_DELAY, 1200);
        assert_eq!(WANDERING_TRADER_MIN_SPAWN_CHANCE, 25);
        assert_eq!(WANDERING_TRADER_MAX_SPAWN_CHANCE, 75);
        assert_eq!(WANDERING_TRADER_SPAWN_CHANCE_INCREASE, 25);
        assert_eq!(WANDERING_TRADER_ONE_IN_X_CHANCE, 10);
        assert_eq!(WANDERING_TRADER_SPAWN_ATTEMPTS, 10);
        assert_eq!(WANDERING_TRADER_SPAWN_RADIUS, 48);
        assert_eq!(WANDERING_TRADER_MEETING_POI_RADIUS, 48);
        assert_eq!(WANDERING_TRADER_LLAMA_COUNT, 2);
        assert_eq!(WANDERING_TRADER_LLAMA_SPAWN_RADIUS, 4);
        assert_eq!(WANDERING_TRADER_DESPAWN_DELAY, 48_000);
        assert_eq!(WANDERING_TRADER_HOME_RADIUS, 16);

        let idle = wandering_trader_spawner_tick(
            WanderingTraderSpawnerState::default(),
            WanderingTraderSpawnContext {
                spawn_traders_rule: false,
                random_player_present: true,
                outer_chance_roll_0_to_99: 0,
                one_in_ten_roll: 0,
                spawn_position_found: true,
                has_enough_space: true,
                biome_allows_spawn: true,
            },
        );
        assert_eq!(idle.state.tick_delay, 1200);
        assert!(!idle.spawn_trader);

        let waiting = wandering_trader_spawner_tick(
            WanderingTraderSpawnerState {
                tick_delay: 2,
                data: WanderingTraderData {
                    spawn_delay: 24_000,
                    spawn_chance: 25,
                },
            },
            WanderingTraderSpawnContext {
                spawn_traders_rule: true,
                random_player_present: true,
                outer_chance_roll_0_to_99: 0,
                one_in_ten_roll: 0,
                spawn_position_found: true,
                has_enough_space: true,
                biome_allows_spawn: true,
            },
        );
        assert_eq!(waiting.state.tick_delay, 1);
        assert_eq!(waiting.state.data.spawn_delay, 24_000);

        let spawned = wandering_trader_spawner_tick(
            WanderingTraderSpawnerState {
                tick_delay: 1,
                data: WanderingTraderData {
                    spawn_delay: 1200,
                    spawn_chance: 25,
                },
            },
            WanderingTraderSpawnContext {
                spawn_traders_rule: true,
                random_player_present: true,
                outer_chance_roll_0_to_99: 25,
                one_in_ten_roll: 0,
                spawn_position_found: true,
                has_enough_space: true,
                biome_allows_spawn: true,
            },
        );
        assert!(spawned.spawn_trader);
        assert!(spawned.returns_spawn_success);
        assert_eq!(spawned.spawn_llamas, 2);
        assert_eq!(spawned.trader_despawn_delay, Some(48_000));
        assert_eq!(spawned.trader_home_radius, Some(16));
        assert_eq!(spawned.state.tick_delay, 1200);
        assert_eq!(spawned.state.data.spawn_delay, 24_000);
        assert_eq!(spawned.state.data.spawn_chance, 25);

        let missed_outer_roll = wandering_trader_spawner_tick(
            WanderingTraderSpawnerState {
                tick_delay: 1,
                data: WanderingTraderData {
                    spawn_delay: 1200,
                    spawn_chance: 25,
                },
            },
            WanderingTraderSpawnContext {
                spawn_traders_rule: true,
                random_player_present: true,
                outer_chance_roll_0_to_99: 26,
                one_in_ten_roll: 0,
                spawn_position_found: true,
                has_enough_space: true,
                biome_allows_spawn: true,
            },
        );
        assert!(!missed_outer_roll.spawn_trader);
        assert_eq!(missed_outer_roll.state.data.spawn_chance, 50);

        let no_player = wandering_trader_spawner_tick(
            WanderingTraderSpawnerState {
                tick_delay: 1,
                data: WanderingTraderData {
                    spawn_delay: 1200,
                    spawn_chance: 50,
                },
            },
            WanderingTraderSpawnContext {
                spawn_traders_rule: true,
                random_player_present: false,
                outer_chance_roll_0_to_99: 50,
                one_in_ten_roll: 9,
                spawn_position_found: false,
                has_enough_space: false,
                biome_allows_spawn: false,
            },
        );
        assert!(no_player.returns_spawn_success);
        assert!(!no_player.spawn_trader);
        assert_eq!(no_player.state.data.spawn_chance, 25);

        assert_eq!(wandering_trader_candidate_offset(0, 48), -48);
        assert_eq!(wandering_trader_candidate_offset(95, 48), 47);
        assert!(wandering_trader_has_enough_space(&[true; 12]));
        assert!(!wandering_trader_has_enough_space(&[true; 11]));
        let mut blocked = [true; 12];
        blocked[6] = false;
        assert!(!wandering_trader_has_enough_space(&blocked));
        assert_eq!(wandering_trader_should_despawn(2, false), (1, false));
        assert_eq!(wandering_trader_should_despawn(1, false), (0, true));
        assert_eq!(wandering_trader_should_despawn(1, true), (1, false));
        assert!(!wandering_trader_remove_when_far_away());
    }

    #[test]
    fn villager_trade_resources_decode_all_vanilla_entries() {
        let root =
            std::path::Path::new("../decompiled-server-26.1.2/data/minecraft/villager_trade");
        let mut paths = Vec::new();
        collect_json_paths(root, &mut paths);
        paths.sort();

        assert_eq!(paths.len(), 387);
        let mut result_items = BTreeSet::new();
        let mut modifier_count = 0;
        let mut exploration_map_count = 0;
        let mut additional_cost_count = 0;
        for path in &paths {
            let trade = load_villager_trade_resource(path).unwrap_or_else(|err| {
                panic!("{} failed to decode: {err}", path.display());
            });
            assert!(trade.wants.item_id.starts_with("minecraft:"));
            assert!(trade.gives.item_id.starts_with("minecraft:"));
            assert!(trade.gives.count >= 1);
            if trade.additional_wants.is_some() {
                additional_cost_count += 1;
            }
            if !trade.given_item_modifiers.is_empty() {
                modifier_count += 1;
            }
            if trade.given_item_modifiers.iter().any(|modifier| {
                modifier.get("function").and_then(serde_json::Value::as_str)
                    == Some("minecraft:exploration_map")
            }) {
                exploration_map_count += 1;
            }
            result_items.insert(trade.gives.item_id);
        }

        assert!(result_items.contains("minecraft:emerald"));
        assert!(result_items.contains("minecraft:enchanted_book"));
        assert!(result_items.contains("minecraft:map"));
        assert!(additional_cost_count > 0);
        assert!(modifier_count > 0);
        assert!(exploration_map_count > 0);
    }

    #[test]
    fn trade_set_resources_decode_all_vanilla_entries() {
        let root = std::path::Path::new("../decompiled-server-26.1.2/data/minecraft/trade_set");
        let mut paths = Vec::new();
        collect_json_paths(root, &mut paths);
        paths.sort();

        assert_eq!(paths.len(), 68);
        let mut tag_count = 0;
        let mut wandering_sets = 0;
        for path in &paths {
            let trade_set = load_trade_set_resource(path).unwrap_or_else(|err| {
                panic!("{} failed to decode: {err}", path.display());
            });
            if matches!(trade_set.trades, HolderSetResource::Tag(_)) {
                tag_count += 1;
            }
            assert!(
                trade_set
                    .random_sequence
                    .as_deref()
                    .is_some_and(|id| id.starts_with("minecraft:trade_set/")),
                "{} should define a namespaced random sequence",
                path.display()
            );
            if path.to_string_lossy().contains("wandering_trader") {
                wandering_sets += 1;
            }
        }

        assert_eq!(tag_count, 68);
        assert_eq!(wandering_sets, 3);
    }

    #[test]
    fn villager_trade_resource_defaults_match_java_codecs() {
        let trade = parse_villager_trade_resource(
            r#"{"wants":{"id":"minecraft:emerald"},"gives":{"id":"minecraft:bread"}}"#,
        )
        .unwrap();
        assert_eq!(trade.wants.count, NumberProviderResource::Constant(1.0));
        assert_eq!(trade.gives.count, 1);
        assert_eq!(trade.max_uses, NumberProviderResource::Constant(4.0));
        assert_eq!(
            trade.reputation_discount,
            NumberProviderResource::Constant(0.0)
        );
        assert_eq!(trade.xp, NumberProviderResource::Constant(1.0));

        let trade_set =
            parse_trade_set_resource(r##"{"trades":"#minecraft:farmer/level_1","amount":2.0}"##)
                .unwrap();
        assert_eq!(
            trade_set.trades,
            HolderSetResource::Tag("minecraft:farmer/level_1".to_string())
        );
        assert_eq!(trade_set.amount, NumberProviderResource::Constant(2.0));
        assert!(!trade_set.allow_duplicates);
        assert_eq!(trade_set.random_sequence, None);
    }

    #[test]
    fn hero_of_the_village_discount_and_special_price_reset_match_java() {
        let mut villager = VillagerTradeState::new(VillagerProfession::Farmer);
        villager.offers.push(MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 1),
            None,
            ItemStack::new("minecraft:bread", 6),
            16,
            1,
            0.05,
        ));
        villager.offers.push(MerchantOffer::new(
            ItemCost::new("minecraft:wheat", 20),
            None,
            ItemStack::new("minecraft:emerald", 1),
            16,
            2,
            0.05,
        ));

        villager.apply_hero_of_the_village_prices(0);
        assert_eq!(villager.offers[0].special_price_diff, -1);
        assert_eq!(villager.offers[1].special_price_diff, -6);

        villager.apply_hero_of_the_village_prices(2);
        assert_eq!(villager.offers[0].special_price_diff, -2);
        assert_eq!(villager.offers[1].special_price_diff, -14);

        villager.reset_special_prices();
        assert_eq!(villager.offers[0].special_price_diff, 0);
        assert_eq!(villager.offers[1].special_price_diff, 0);
    }

    fn collect_json_paths(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                collect_json_paths(&path, paths);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                paths.push(path);
            }
        }
    }
}
