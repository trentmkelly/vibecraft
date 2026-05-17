#![allow(dead_code)]

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
            if !self.offers.iter().any(|offer| offer.result.item_id() == template.result.item_id()) {
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
        villager.gossip.add("player-a", GossipType::MajorPositive, 20);

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
}
