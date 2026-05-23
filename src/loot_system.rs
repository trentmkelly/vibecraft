#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::block_entity::{PotItemStack, VaultBlockEntity, VaultInsertResult};
use crate::villager_system::{VillagerProfession, WanderingTraderOffers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootStack {
    pub item: String,
    pub count: i32,
    pub components: HashMap<String, String>,
}

impl LootStack {
    pub fn new(item: impl Into<String>, count: i32) -> Self {
        Self {
            item: item.into(),
            count,
            components: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item == "minecraft:air"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootTable {
    pub param_set: LootParamSet,
    pub random_sequence: Option<String>,
    pub pools: Vec<LootPool>,
    pub functions: Vec<LootFunction>,
}

impl LootTable {
    pub fn empty() -> Self {
        Self {
            param_set: LootParamSet::Empty,
            random_sequence: None,
            pools: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn evaluate(&self, context: &mut LootContext) -> Vec<LootStack> {
        let visit_key = self
            .random_sequence
            .as_ref()
            .map(|sequence| format!("table:{sequence}"))
            .unwrap_or_else(|| format!("table:{:p}", self));
        if !context.enter(visit_key) {
            context
                .warnings
                .push("Detected infinite loop in loot tables".to_string());
            return Vec::new();
        }

        if let Some(sequence) = &self.random_sequence {
            context.use_random_sequence(sequence);
        }

        let mut result = Vec::new();
        for pool in &self.pools {
            result.extend(pool.evaluate(context));
        }
        for function in &self.functions {
            result = result
                .into_iter()
                .filter_map(|stack| function.apply(stack, context))
                .collect();
        }

        context.exit();
        split_stacks(result, context.max_stack_size)
    }

    pub fn fill_container(
        &self,
        context: &mut LootContext,
        slots: usize,
    ) -> Vec<Option<LootStack>> {
        let mut items = self.evaluate(context);
        let mut available: Vec<usize> = (0..slots).collect();
        shuffle(&mut available, &mut context.random);
        shuffle(&mut items, &mut context.random);

        let mut filled = vec![None; slots];
        for item in items.into_iter().filter(|stack| !stack.is_empty()) {
            if let Some(slot) = available.pop() {
                filled[slot] = Some(item);
            } else {
                context
                    .warnings
                    .push("Tried to over-fill a container".to_string());
                break;
            }
        }
        filled
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (pool_index, pool) in self.pools.iter().enumerate() {
            if pool.entries.is_empty() {
                errors.push(format!("pools[{pool_index}] has no entries"));
            }
            pool.validate(format!("pools[{pool_index}]"), &mut errors);
        }
        errors
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootParamSet {
    Empty,
    AllParams,
    Block,
    Entity,
    Chest,
    Fishing,
    Archaeology,
    AdvancementReward,
    Gift,
    Barter,
    Vault,
    Command,
    Selector,
    AdvancementEntity,
    Equipment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootContextEntityType {
    Block,
    Entity,
    Chest,
    Fishing,
    Archaeology,
    AdvancementReward,
    Gift,
    Barter,
    Vault,
    Command,
    Selector,
    AdvancementEntity,
    Equipment,
}

impl LootContextEntityType {
    pub fn param_set(self) -> LootParamSet {
        match self {
            Self::Block => LootParamSet::Block,
            Self::Entity => LootParamSet::Entity,
            Self::Chest => LootParamSet::Chest,
            Self::Fishing => LootParamSet::Fishing,
            Self::Archaeology => LootParamSet::Archaeology,
            Self::AdvancementReward => LootParamSet::AdvancementReward,
            Self::Gift => LootParamSet::Gift,
            Self::Barter => LootParamSet::Barter,
            Self::Vault => LootParamSet::Vault,
            Self::Command => LootParamSet::Command,
            Self::Selector => LootParamSet::Selector,
            Self::AdvancementEntity => LootParamSet::AdvancementEntity,
            Self::Equipment => LootParamSet::Equipment,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootParamValue {
    BlockState(String),
    BlockEntity(String),
    Origin(f64, f64, f64),
    Tool(String),
    ThisEntity(String),
    LastDamagePlayer(String),
    KillerEntity(String),
    DirectKillerEntity(String),
    ExplosionRadius(f32),
    DamageSource(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootParamKey {
    BlockState,
    BlockEntity,
    Origin,
    Tool,
    ThisEntity,
    LastDamagePlayer,
    KillerEntity,
    DirectKillerEntity,
    ExplosionRadius,
    DamageSource,
}

impl LootParamValue {
    pub fn key(&self) -> LootParamKey {
        match self {
            Self::BlockState(_) => LootParamKey::BlockState,
            Self::BlockEntity(_) => LootParamKey::BlockEntity,
            Self::Origin(..) => LootParamKey::Origin,
            Self::Tool(_) => LootParamKey::Tool,
            Self::ThisEntity(_) => LootParamKey::ThisEntity,
            Self::LastDamagePlayer(_) => LootParamKey::LastDamagePlayer,
            Self::KillerEntity(_) => LootParamKey::KillerEntity,
            Self::DirectKillerEntity(_) => LootParamKey::DirectKillerEntity,
            Self::ExplosionRadius(_) => LootParamKey::ExplosionRadius,
            Self::DamageSource(_) => LootParamKey::DamageSource,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LootParams {
    values: HashMap<LootParamKey, LootParamValue>,
}

impl LootParams {
    pub fn insert(&mut self, value: LootParamValue) {
        self.values.insert(value.key(), value);
    }

    pub fn get(&self, key: LootParamKey) -> Option<&LootParamValue> {
        self.values.get(&key)
    }

    pub fn keys(&self) -> HashSet<LootParamKey> {
        self.values.keys().copied().collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootDynamicParamValue {
    EnchantmentLevel(i32),
    EnchantmentActive(bool),
    AttackingEntity(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootDynamicParamKey {
    EnchantmentLevel,
    EnchantmentActive,
    AttackingEntity,
}

impl LootDynamicParamValue {
    pub fn key(&self) -> LootDynamicParamKey {
        match self {
            Self::EnchantmentLevel(_) => LootDynamicParamKey::EnchantmentLevel,
            Self::EnchantmentActive(_) => LootDynamicParamKey::EnchantmentActive,
            Self::AttackingEntity(_) => LootDynamicParamKey::AttackingEntity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootSurface {
    BlockBreak,
    EntityDeath,
    ChestOpen,
    FishingRetrieve,
    ArchaeologyBrush,
    AdvancementReward,
    Gift,
    PiglinBarter,
    Vault,
    Command,
    Selector,
    AdvancementEntity,
    Equipment,
}

impl LootSurface {
    pub fn param_set(self) -> LootParamSet {
        match self {
            Self::BlockBreak => LootParamSet::Block,
            Self::EntityDeath => LootParamSet::Entity,
            Self::ChestOpen => LootParamSet::Chest,
            Self::FishingRetrieve => LootParamSet::Fishing,
            Self::ArchaeologyBrush => LootParamSet::Archaeology,
            Self::AdvancementReward => LootParamSet::AdvancementReward,
            Self::Gift => LootParamSet::Gift,
            Self::PiglinBarter => LootParamSet::Barter,
            Self::Vault => LootParamSet::Vault,
            Self::Command => LootParamSet::Command,
            Self::Selector => LootParamSet::Selector,
            Self::AdvancementEntity => LootParamSet::AdvancementEntity,
            Self::Equipment => LootParamSet::Equipment,
        }
    }

    pub fn entity_type(self) -> LootContextEntityType {
        match self {
            Self::BlockBreak => LootContextEntityType::Block,
            Self::EntityDeath => LootContextEntityType::Entity,
            Self::ChestOpen => LootContextEntityType::Chest,
            Self::FishingRetrieve => LootContextEntityType::Fishing,
            Self::ArchaeologyBrush => LootContextEntityType::Archaeology,
            Self::AdvancementReward => LootContextEntityType::AdvancementReward,
            Self::Gift => LootContextEntityType::Gift,
            Self::PiglinBarter => LootContextEntityType::Barter,
            Self::Vault => LootContextEntityType::Vault,
            Self::Command => LootContextEntityType::Command,
            Self::Selector => LootContextEntityType::Selector,
            Self::AdvancementEntity => LootContextEntityType::AdvancementEntity,
            Self::Equipment => LootContextEntityType::Equipment,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootRequest {
    pub surface: LootSurface,
    pub table: String,
    pub origin: (f64, f64, f64),
    pub actor: Option<String>,
    pub target_entity: Option<String>,
    pub killer_entity: Option<String>,
    pub direct_killer_entity: Option<String>,
    pub last_damage_player: Option<String>,
    pub block: Option<String>,
    pub block_entity: Option<String>,
    pub tool: Option<String>,
    pub damage_source: Option<String>,
    pub explosion_radius: Option<f32>,
    pub luck: f32,
    pub looting_level: i32,
    pub fortune_level: i32,
    pub killed_by_player: bool,
    pub correct_tool: bool,
    pub silk_touch: bool,
    pub do_tile_drops: bool,
    pub entity_properties: HashMap<String, String>,
}

impl LootRequest {
    pub fn new(surface: LootSurface, table: impl Into<String>) -> Self {
        Self {
            surface,
            table: table.into(),
            origin: (0.0, 0.0, 0.0),
            actor: None,
            target_entity: None,
            killer_entity: None,
            direct_killer_entity: None,
            last_damage_player: None,
            block: None,
            block_entity: None,
            tool: None,
            damage_source: None,
            explosion_radius: None,
            luck: 0.0,
            looting_level: 0,
            fortune_level: 0,
            killed_by_player: false,
            correct_tool: true,
            silk_touch: false,
            do_tile_drops: true,
            entity_properties: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootDelivery {
    DropAt((f64, f64, f64), Vec<LootStack>),
    GiveToEntity(String, Vec<LootStack>),
    FillContainer(Vec<Option<LootStack>>),
    ReplaceSlots {
        target: String,
        stacks: Vec<LootStack>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootResolution {
    pub surface: LootSurface,
    pub table: String,
    pub param_set: LootParamSet,
    pub delivery: LootDelivery,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementRewardLootInput {
    pub player: String,
    pub origin: (f64, f64, f64),
    pub experience: i32,
    pub loot_tables: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementRewardLootOutput {
    pub experience: i32,
    pub loot: Vec<LootResolution>,
}

pub fn resolve_advancement_reward_loot(
    engine: &LootBehaviorEngine,
    input: AdvancementRewardLootInput,
    seed: u64,
) -> AdvancementRewardLootOutput {
    let loot = input
        .loot_tables
        .iter()
        .enumerate()
        .map(|(index, table)| {
            let mut request = LootRequest::new(LootSurface::AdvancementReward, table.clone());
            request.origin = input.origin;
            request.actor = Some(input.player.clone());
            request.target_entity = Some(input.player.clone());
            engine.resolve(request, hash_seed(seed, &format!("{table}#{index}")))
        })
        .collect();

    AdvancementRewardLootOutput {
        experience: input.experience,
        loot,
    }
}

pub fn resolve_piglin_barter_loot(
    engine: &LootBehaviorEngine,
    piglin: impl Into<String>,
    offered_item: &str,
    origin: (f64, f64, f64),
    seed: u64,
) -> Option<LootResolution> {
    if offered_item != "minecraft:gold_ingot" {
        return None;
    }

    let piglin = piglin.into();
    let mut request = LootRequest::new(
        LootSurface::PiglinBarter,
        "minecraft:gameplay/piglin_bartering",
    );
    request.origin = origin;
    request.actor = Some(piglin.clone());
    request.target_entity = Some(piglin);
    Some(engine.resolve(request, seed))
}

pub fn resolve_fishing_loot(
    engine: &LootBehaviorEngine,
    tool: impl Into<String>,
    origin: (f64, f64, f64),
    hook_luck: f32,
    player_luck: f32,
    in_open_water: bool,
    seed: u64,
) -> LootResolution {
    let mut request = LootRequest::new(LootSurface::FishingRetrieve, "minecraft:gameplay/fishing");
    request.origin = origin;
    request.tool = Some(tool.into());
    request.target_entity = Some("FishingHook".to_string());
    request.luck = hook_luck.max(0.0) + player_luck;
    request
        .entity_properties
        .insert("in_open_water".to_string(), in_open_water.to_string());
    engine.resolve(request, seed)
}

pub fn resolve_block_break_loot(
    engine: &LootBehaviorEngine,
    mut request: LootRequest,
    seed: u64,
) -> LootResolution {
    request.surface = LootSurface::BlockBreak;
    if !request.do_tile_drops {
        return LootResolution {
            surface: request.surface,
            table: request.table,
            param_set: LootParamSet::Block,
            delivery: LootDelivery::DropAt(request.origin, Vec::new()),
            warnings: Vec::new(),
        };
    }
    engine.resolve(request, seed)
}

pub fn resolve_vault_unlock_loot(
    engine: &LootBehaviorEngine,
    vault: &mut VaultBlockEntity,
    player: impl Into<String>,
    inserted_key: impl Into<String>,
    origin: (f64, f64, f64),
    seed: u64,
    game_time: i64,
) -> VaultInsertResult {
    let player = player.into();
    let table = if vault.is_ominous {
        "minecraft:trial_chambers/reward_ominous"
    } else {
        "minecraft:trial_chambers/reward"
    };
    let mut request = LootRequest::new(LootSurface::Vault, table);
    request.origin = origin;
    request.actor = Some(player.clone());
    request.target_entity = request.actor.clone();
    let rewards = match engine.resolve(request, seed).delivery {
        LootDelivery::DropAt(_, stacks) => stacks
            .into_iter()
            .map(|stack| PotItemStack {
                item_id: stack.item,
                count: stack.count,
            })
            .collect(),
        _ => Vec::new(),
    };
    let inserted = PotItemStack {
        item_id: inserted_key.into(),
        count: 1,
    };
    vault.try_insert_key(player, &inserted, rewards, game_time)
}

pub fn hero_of_the_village_gift_table(
    profession: VillagerProfession,
    is_baby: bool,
) -> &'static str {
    if is_baby {
        return "minecraft:gameplay/hero_of_the_village/baby_gift";
    }
    match profession {
        VillagerProfession::Armorer => "minecraft:gameplay/hero_of_the_village/armorer_gift",
        VillagerProfession::Butcher => "minecraft:gameplay/hero_of_the_village/butcher_gift",
        VillagerProfession::Cartographer => {
            "minecraft:gameplay/hero_of_the_village/cartographer_gift"
        }
        VillagerProfession::Cleric => "minecraft:gameplay/hero_of_the_village/cleric_gift",
        VillagerProfession::Farmer => "minecraft:gameplay/hero_of_the_village/farmer_gift",
        VillagerProfession::Fisherman => "minecraft:gameplay/hero_of_the_village/fisherman_gift",
        VillagerProfession::Fletcher => "minecraft:gameplay/hero_of_the_village/fletcher_gift",
        VillagerProfession::Leatherworker => {
            "minecraft:gameplay/hero_of_the_village/leatherworker_gift"
        }
        VillagerProfession::Librarian => "minecraft:gameplay/hero_of_the_village/librarian_gift",
        VillagerProfession::Mason => "minecraft:gameplay/hero_of_the_village/mason_gift",
        VillagerProfession::Shepherd => "minecraft:gameplay/hero_of_the_village/shepherd_gift",
        VillagerProfession::Toolsmith => "minecraft:gameplay/hero_of_the_village/toolsmith_gift",
        VillagerProfession::Weaponsmith => {
            "minecraft:gameplay/hero_of_the_village/weaponsmith_gift"
        }
        VillagerProfession::None | VillagerProfession::Nitwit => {
            "minecraft:gameplay/hero_of_the_village/unemployed_gift"
        }
    }
}

pub fn resolve_mob_gift_loot(
    engine: &LootBehaviorEngine,
    actor: impl Into<String>,
    table: impl Into<String>,
    origin: (f64, f64, f64),
    seed: u64,
) -> LootResolution {
    let actor = actor.into();
    let mut request = LootRequest::new(LootSurface::Gift, table.into());
    request.origin = origin;
    request.actor = Some(actor.clone());
    request.target_entity = Some(actor);
    engine.resolve(request, seed)
}

pub fn wandering_trader_reward_offers() -> WanderingTraderOffers {
    WanderingTraderOffers::vanilla_sample()
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootTableResource {
    pub param_set: String,
    pub random_sequence: Option<String>,
    pub pools: Vec<LootPoolResource>,
    pub functions: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootPoolResource {
    pub entries: Vec<serde_json::Value>,
    pub conditions: Vec<serde_json::Value>,
    pub functions: Vec<serde_json::Value>,
    pub rolls: serde_json::Value,
    pub bonus_rolls: serde_json::Value,
}

pub fn parse_loot_table_resource(raw: &str) -> Result<LootTableResource, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid loot table JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "loot table must be a JSON object".to_string())?;
    let pools = object
        .get("pools")
        .map(parse_loot_pools)
        .transpose()?
        .unwrap_or_default();
    let functions = object
        .get("functions")
        .map(parse_json_list)
        .transpose()?
        .unwrap_or_default();

    Ok(LootTableResource {
        param_set: object
            .get("type")
            .map(json_string_value)
            .transpose()?
            .unwrap_or_else(|| "minecraft:all_params".to_string()),
        random_sequence: object
            .get("random_sequence")
            .map(json_string_value)
            .transpose()?,
        pools,
        functions,
    })
}

pub fn load_loot_table_resource(path: impl AsRef<Path>) -> Result<LootTableResource, String> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|err| format!("failed to read {}: {err}", path.as_ref().display()))?;
    parse_loot_table_resource(&raw)
}

fn parse_loot_pools(value: &serde_json::Value) -> Result<Vec<LootPoolResource>, String> {
    value
        .as_array()
        .ok_or_else(|| "pools must be a list".to_string())?
        .iter()
        .map(parse_loot_pool)
        .collect()
}

fn parse_loot_pool(value: &serde_json::Value) -> Result<LootPoolResource, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "loot pool must be a JSON object".to_string())?;
    let entries = object
        .get("entries")
        .map(parse_json_list)
        .transpose()?
        .ok_or_else(|| "loot pool entries are required".to_string())?;
    for entry in &entries {
        validate_loot_entry(entry)?;
    }
    let conditions = object
        .get("conditions")
        .map(parse_json_list)
        .transpose()?
        .unwrap_or_default();
    let functions = object
        .get("functions")
        .map(parse_json_list)
        .transpose()?
        .unwrap_or_default();

    Ok(LootPoolResource {
        entries,
        conditions,
        functions,
        rolls: required_value(object, "rolls")?.clone(),
        bonus_rolls: object
            .get("bonus_rolls")
            .cloned()
            .unwrap_or(serde_json::Value::from(0.0)),
    })
}

fn validate_loot_entry(value: &serde_json::Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "loot entry must be a JSON object".to_string())?;
    let entry_type = json_string(object, "type")?;
    match entry_type.as_str() {
        "minecraft:item" => {
            json_string(object, "name")?;
        }
        "minecraft:alternatives" | "minecraft:group" | "minecraft:sequence" => {
            let children = object
                .get("children")
                .map(parse_json_list)
                .transpose()?
                .unwrap_or_default();
            for child in &children {
                validate_loot_entry(child)?;
            }
        }
        "minecraft:loot_table" => {
            required_value(object, "value")?;
        }
        "minecraft:dynamic" => {
            json_string(object, "name")?;
        }
        "minecraft:tag" => {
            json_string(object, "name")?;
        }
        "minecraft:empty" => {}
        other => return Err(format!("unsupported loot entry type {other}")),
    }
    Ok(())
}

fn parse_json_list(value: &serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
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

mod runtime_engine;
pub use runtime_engine::*;

mod context;
pub use context::*;

#[cfg(test)]
mod tests;
