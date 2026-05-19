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
        for offer in &mut self.offers {
            offer.special_price_diff = (-reputation / 10).clamp(-30, 30);
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

pub fn wandering_trader_tick(
    mut data: WanderingTraderData,
    spawn_traders_rule: bool,
    has_players: bool,
    random_roll: i32,
) -> WanderingTraderSpawnPlan {
    if !spawn_traders_rule || !has_players {
        return WanderingTraderSpawnPlan {
            should_spawn: false,
            next_data: data,
        };
    }
    data.spawn_delay -= 1;
    if data.spawn_delay > 0 {
        return WanderingTraderSpawnPlan {
            should_spawn: false,
            next_data: data,
        };
    }
    data.spawn_delay = 24_000;
    let should_spawn = random_roll < data.spawn_chance;
    if should_spawn {
        data.spawn_chance = 25;
    } else {
        data.spawn_chance = (data.spawn_chance + 25).min(75);
    }
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
        assert_eq!(villager.offers[0].special_price_diff, -10);
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
