#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootStack {
    pub item: String,
    pub count: i32,
}

impl LootStack {
    pub fn new(item: impl Into<String>, count: i32) -> Self {
        Self {
            item: item.into(),
            count,
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
    pub tool: Option<String>,
    pub damage_source: Option<String>,
    pub explosion_radius: Option<f32>,
    pub luck: f32,
    pub looting_level: i32,
    pub killed_by_player: bool,
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
            tool: None,
            damage_source: None,
            explosion_radius: None,
            luck: 0.0,
            looting_level: 0,
            killed_by_player: false,
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

#[derive(Debug, Clone, Default)]
pub struct LootBehaviorEngine {
    tables: HashMap<String, LootTable>,
}

impl LootBehaviorEngine {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn insert_table(&mut self, id: impl Into<String>, table: LootTable) {
        self.tables.insert(id.into(), table);
    }

    pub fn resolve(&self, request: LootRequest, seed: u64) -> LootResolution {
        let mut context = self.context_for(&request, seed);
        let table = self
            .tables
            .get(&request.table)
            .cloned()
            .unwrap_or_else(LootTable::empty);
        context.tables = self.tables.clone();
        let delivery = match request.surface {
            LootSurface::ChestOpen => {
                LootDelivery::FillContainer(table.fill_container(&mut context, 27))
            }
            LootSurface::AdvancementReward | LootSurface::Gift | LootSurface::PiglinBarter => {
                LootDelivery::GiveToEntity(
                    request
                        .actor
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    table.evaluate(&mut context),
                )
            }
            LootSurface::Vault => {
                LootDelivery::DropAt(request.origin, table.evaluate(&mut context))
            }
            LootSurface::Command if request.actor.is_some() => LootDelivery::GiveToEntity(
                request.actor.clone().unwrap(),
                table.evaluate(&mut context),
            ),
            LootSurface::Command => {
                LootDelivery::DropAt(request.origin, table.evaluate(&mut context))
            }
            LootSurface::BlockBreak
            | LootSurface::EntityDeath
            | LootSurface::FishingRetrieve
            | LootSurface::ArchaeologyBrush
            | LootSurface::Selector
            | LootSurface::AdvancementEntity
            | LootSurface::Equipment => {
                LootDelivery::DropAt(request.origin, table.evaluate(&mut context))
            }
        };

        LootResolution {
            surface: request.surface,
            table: request.table,
            param_set: request.surface.param_set(),
            delivery,
            warnings: context.warnings,
        }
    }

    fn context_for(&self, request: &LootRequest, seed: u64) -> LootContext {
        let mut context = LootContext::new(request.surface.param_set(), seed);
        context.entity_type = request.surface.entity_type();
        context.luck = request.luck;
        context.looting_level = request.looting_level;
        context.killed_by_player = request.killed_by_player;
        context.insert_param(LootParamValue::Origin(
            request.origin.0,
            request.origin.1,
            request.origin.2,
        ));
        if let Some(radius) = request.explosion_radius {
            context.insert_param(LootParamValue::ExplosionRadius(radius));
        }
        if let Some(block) = &request.block {
            context.insert_param(LootParamValue::BlockState(block.clone()));
        }
        if let Some(tool) = &request.tool {
            context.insert_param(LootParamValue::Tool(tool.clone()));
        }
        if let Some(actor) = &request.actor {
            context
                .entity_properties
                .insert("actor".to_string(), actor.clone());
        }
        if let Some(target) = &request.target_entity {
            context.insert_param(LootParamValue::ThisEntity(target.clone()));
        }
        if let Some(killer) = &request.killer_entity {
            context.insert_param(LootParamValue::KillerEntity(killer.clone()));
        }
        if let Some(direct_killer) = &request.direct_killer_entity {
            context.insert_param(LootParamValue::DirectKillerEntity(direct_killer.clone()));
        }
        if let Some(last_damage_player) = &request.last_damage_player {
            context.insert_param(LootParamValue::LastDamagePlayer(last_damage_player.clone()));
        }
        if let Some(damage_source) = &request.damage_source {
            context.insert_param(LootParamValue::DamageSource(damage_source.clone()));
        }
        context
            .entity_properties
            .extend(request.entity_properties.clone());
        context
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomizableContainerLoot {
    pub loot_table: Option<String>,
    pub loot_table_seed: u64,
    pub unpacked: bool,
}

impl RandomizableContainerLoot {
    pub fn unpack_once(
        &mut self,
        engine: &LootBehaviorEngine,
        origin: (f64, f64, f64),
    ) -> Option<Vec<Option<LootStack>>> {
        if self.unpacked {
            return None;
        }
        let table = self.loot_table.clone()?;
        self.unpacked = true;
        let mut request = LootRequest::new(LootSurface::ChestOpen, table);
        request.origin = origin;
        match engine.resolve(request, self.loot_table_seed).delivery {
            LootDelivery::FillContainer(slots) => Some(slots),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootPool {
    pub entries: Vec<LootEntry>,
    pub conditions: Vec<LootCondition>,
    pub functions: Vec<LootFunction>,
    pub rolls: NumberProvider,
    pub bonus_rolls: NumberProvider,
}

impl LootPool {
    pub fn single(entry: LootEntry) -> Self {
        Self {
            entries: vec![entry],
            conditions: Vec::new(),
            functions: Vec::new(),
            rolls: NumberProvider::Constant(1.0),
            bonus_rolls: NumberProvider::Constant(0.0),
        }
    }

    pub fn evaluate(&self, context: &mut LootContext) -> Vec<LootStack> {
        if !self
            .conditions
            .iter()
            .all(|condition| condition.matches(context))
        {
            return Vec::new();
        }

        let count = self.rolls.int(context)
            + (self.bonus_rolls.float(context) * context.luck).floor() as i32;
        let mut result = Vec::new();
        for _ in 0..count.max(0) {
            let mut candidates = Vec::new();
            for entry in &self.entries {
                entry.expand(context, &mut candidates);
            }
            let total_weight: i32 = candidates
                .iter()
                .map(|entry| entry.effective_weight(context.luck))
                .sum();
            if total_weight <= 0 {
                continue;
            }
            let mut pick = context.random.next_i32(total_weight);
            for candidate in candidates {
                pick -= candidate.effective_weight(context.luck);
                if pick < 0 {
                    result.extend(candidate.create(context));
                    break;
                }
            }
        }

        for function in &self.functions {
            result = result
                .into_iter()
                .filter_map(|stack| function.apply(stack, context))
                .collect();
        }
        result
    }

    fn validate(&self, path: String, errors: &mut Vec<String>) {
        if self.rolls.minimum() < 0.0 {
            errors.push(format!("{path}.rolls can produce a negative count"));
        }
        for (entry_index, entry) in self.entries.iter().enumerate() {
            entry.validate(format!("{path}.entries[{entry_index}]"), errors);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootEntry {
    Item {
        item: String,
        weight: i32,
        quality: i32,
        conditions: Vec<LootCondition>,
        functions: Vec<LootFunction>,
    },
    Empty {
        weight: i32,
        quality: i32,
        conditions: Vec<LootCondition>,
    },
    Tag {
        tag: String,
        expand: bool,
        weight: i32,
        quality: i32,
        conditions: Vec<LootCondition>,
    },
    Alternatives(Vec<LootEntry>),
    Sequence(Vec<LootEntry>),
    Group(Vec<LootEntry>),
    NestedTable(String),
    WeightedNestedTable {
        table: String,
        weight: i32,
        quality: i32,
        conditions: Vec<LootCondition>,
    },
    Dynamic(String),
}

impl LootEntry {
    pub fn item(item: impl Into<String>, weight: i32) -> Self {
        Self::Item {
            item: item.into(),
            weight,
            quality: 0,
            conditions: Vec::new(),
            functions: Vec::new(),
        }
    }

    fn expand(&self, context: &LootContext, output: &mut Vec<ExpandedEntry>) {
        match self {
            Self::Item {
                weight,
                quality,
                conditions,
                ..
            } => {
                if conditions
                    .iter()
                    .all(|condition| condition.matches(context))
                {
                    output.push(ExpandedEntry {
                        entry: self.clone(),
                        weight: *weight,
                        quality: *quality,
                    });
                }
            }
            Self::Empty {
                weight,
                quality,
                conditions,
            } => {
                if conditions
                    .iter()
                    .all(|condition| condition.matches(context))
                {
                    output.push(ExpandedEntry {
                        entry: self.clone(),
                        weight: *weight,
                        quality: *quality,
                    });
                }
            }
            Self::Tag {
                tag,
                expand,
                weight,
                quality,
                conditions,
            } => {
                if !conditions
                    .iter()
                    .all(|condition| condition.matches(context))
                {
                    return;
                }
                if *expand {
                    if let Some(items) = context.tags.get(tag) {
                        for item in items {
                            output.push(ExpandedEntry {
                                entry: LootEntry::item(item.clone(), *weight),
                                weight: *weight,
                                quality: *quality,
                            });
                        }
                    }
                } else {
                    output.push(ExpandedEntry {
                        entry: self.clone(),
                        weight: *weight,
                        quality: *quality,
                    });
                }
            }
            Self::Alternatives(children) | Self::Sequence(children) | Self::Group(children)
                if !children.is_empty() =>
            {
                output.push(ExpandedEntry {
                    entry: self.clone(),
                    weight: 1,
                    quality: 0,
                });
            }
            Self::Alternatives(_) | Self::Sequence(_) | Self::Group(_) => {}
            Self::NestedTable(_) | Self::Dynamic(_) => output.push(ExpandedEntry {
                entry: self.clone(),
                weight: 1,
                quality: 0,
            }),
            Self::WeightedNestedTable {
                weight,
                quality,
                conditions,
                ..
            } => {
                if conditions
                    .iter()
                    .all(|condition| condition.matches(context))
                {
                    output.push(ExpandedEntry {
                        entry: self.clone(),
                        weight: *weight,
                        quality: *quality,
                    });
                }
            }
        }
    }

    fn create(&self, context: &mut LootContext) -> Vec<LootStack> {
        match self {
            Self::Item {
                item, functions, ..
            } => {
                let mut stack = LootStack::new(item, 1);
                for function in functions {
                    match function.apply(stack, context) {
                        Some(next) => stack = next,
                        None => return Vec::new(),
                    }
                }
                vec![stack]
            }
            Self::Empty { .. } => Vec::new(),
            Self::Tag { tag, expand, .. } => context
                .tags
                .get(tag)
                .map(|items| {
                    if *expand {
                        items.iter().map(|item| LootStack::new(item, 1)).collect()
                    } else {
                        vec![LootStack::new(format!("#{tag}"), 1)]
                    }
                })
                .unwrap_or_default(),
            Self::NestedTable(table) | Self::WeightedNestedTable { table, .. } => {
                if let Some(nested) = context.tables.get(table).cloned() {
                    nested.evaluate(context)
                } else {
                    context
                        .warnings
                        .push(format!("Unknown nested loot table {table}"));
                    Vec::new()
                }
            }
            Self::Dynamic(name) => context.dynamic_drops.get(name).cloned().unwrap_or_default(),
            Self::Alternatives(children) => {
                for child in children {
                    let mut expanded = Vec::new();
                    child.expand(context, &mut expanded);
                    expanded.retain(|entry| entry.effective_weight(context.luck) > 0);
                    if !expanded.is_empty() {
                        return expanded
                            .into_iter()
                            .flat_map(|entry| entry.entry.create(context))
                            .collect();
                    }
                }
                Vec::new()
            }
            Self::Sequence(children) => {
                let mut result = Vec::new();
                for child in children {
                    let mut expanded = Vec::new();
                    child.expand(context, &mut expanded);
                    if expanded.is_empty() {
                        return Vec::new();
                    }
                    result.extend(
                        expanded
                            .into_iter()
                            .flat_map(|entry| entry.entry.create(context)),
                    );
                }
                result
            }
            Self::Group(children) => children
                .iter()
                .flat_map(|child| child.create(context))
                .collect(),
        }
    }

    fn validate(&self, path: String, errors: &mut Vec<String>) {
        match self {
            Self::Item { item, weight, .. } if item.is_empty() || *weight < 0 => {
                errors.push(format!("{path} has invalid item or weight"));
            }
            Self::Empty { weight, .. } if *weight < 0 => {
                errors.push(format!("{path} has invalid empty-entry weight"));
            }
            Self::Tag { tag, weight, .. } if tag.is_empty() || *weight < 0 => {
                errors.push(format!("{path} has invalid tag or weight"));
            }
            Self::Alternatives(children) | Self::Sequence(children) | Self::Group(children) => {
                if children.is_empty() {
                    errors.push(format!("{path} has no children"));
                }
                for (index, child) in children.iter().enumerate() {
                    child.validate(format!("{path}.children[{index}]"), errors);
                }
            }
            Self::NestedTable(table)
            | Self::WeightedNestedTable { table, .. }
            | Self::Dynamic(table)
                if table.is_empty() =>
            {
                errors.push(format!("{path} has an empty reference"));
            }
            Self::WeightedNestedTable { weight, .. } if *weight < 0 => {
                errors.push(format!("{path} has invalid nested-table weight"));
            }
            _ => {}
        }
    }
}

struct ExpandedEntry {
    entry: LootEntry,
    weight: i32,
    quality: i32,
}

impl ExpandedEntry {
    fn effective_weight(&self, luck: f32) -> i32 {
        (self.weight as f32 + self.quality as f32 * luck)
            .floor()
            .max(0.0) as i32
    }

    fn create(&self, context: &mut LootContext) -> Vec<LootStack> {
        self.entry.create(context)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberProvider {
    Constant(f32),
    Uniform { min: f32, max: f32 },
    Binomial { n: i32, p: f32 },
    Score { name: String, scale: f32 },
    Storage { key: String, scale: f32 },
    EnchantmentLevel { scale: f32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreProvider {
    Context { target: String },
    Fixed { name: String },
}

impl ScoreProvider {
    pub fn scoreboard_name(&self, context: &LootContext) -> Option<String> {
        match self {
            Self::Context { target } => context.entity_properties.get(target).cloned(),
            Self::Fixed { name } => Some(name.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtProvider {
    Context { target: String, path: String },
    Storage { key: String, path: String },
}

impl NbtProvider {
    pub fn get<'a>(&self, context: &'a LootContext) -> Option<&'a str> {
        match self {
            Self::Context { target, path } => context
                .context_nbt
                .get(target)
                .and_then(|values| values.get(path))
                .map(String::as_str),
            Self::Storage { key, path } => context
                .storage_nbt
                .get(key)
                .and_then(|values| values.get(path))
                .map(String::as_str),
        }
    }
}

impl NumberProvider {
    pub fn int(&self, context: &mut LootContext) -> i32 {
        self.float(context).floor() as i32
    }

    pub fn float(&self, context: &mut LootContext) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Uniform { min, max } => *min + (*max - *min) * context.random.next_f32(),
            Self::Binomial { n, p } => {
                let mut successes = 0;
                for _ in 0..(*n).max(0) {
                    if context.random.next_f32() < *p {
                        successes += 1;
                    }
                }
                successes as f32
            }
            Self::Score { name, scale } => {
                context.scores.get(name).copied().unwrap_or(0) as f32 * *scale
            }
            Self::Storage { key, scale } => {
                context.storage_numbers.get(key).copied().unwrap_or(0.0) * *scale
            }
            Self::EnchantmentLevel { scale } => context.enchantment_level as f32 * *scale,
        }
    }

    fn minimum(&self) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Uniform { min, .. } => *min,
            Self::Binomial { .. } => 0.0,
            Self::Score { .. } | Self::Storage { .. } => f32::MIN,
            Self::EnchantmentLevel { .. } => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootCondition {
    RandomChance(f32),
    RandomChanceWithLooting {
        chance: f32,
        looting_multiplier: f32,
    },
    KilledByPlayer,
    SurvivesExplosion,
    EntityProperty {
        key: String,
        value: String,
    },
    BlockState {
        block: String,
    },
    MatchTool {
        item: String,
    },
    TimeCheck {
        min: i64,
        max: i64,
    },
    ValueCheck {
        provider: NumberProvider,
        min: f32,
        max: f32,
    },
    Reference(String),
    Inverted(Box<LootCondition>),
    AllOf(Vec<LootCondition>),
    AnyOf(Vec<LootCondition>),
}

impl LootCondition {
    pub fn matches(&self, context: &LootContext) -> bool {
        let mut fork = context.clone_for_condition();
        self.matches_mut(&mut fork)
    }

    fn matches_mut(&self, context: &mut LootContext) -> bool {
        match self {
            Self::RandomChance(chance) => context.random.next_f32() < *chance,
            Self::RandomChanceWithLooting {
                chance,
                looting_multiplier,
            } => {
                context.random.next_f32()
                    < *chance + *looting_multiplier * context.looting_level as f32
            }
            Self::KilledByPlayer => context.killed_by_player,
            Self::SurvivesExplosion => match context.explosion_radius {
                Some(radius) if radius > 0.0 => context.random.next_f32() <= 1.0 / radius,
                _ => true,
            },
            Self::EntityProperty { key, value } => {
                context.entity_properties.get(key) == Some(value)
            }
            Self::BlockState { block } => context.block.as_ref() == Some(block),
            Self::MatchTool { item } => context.tool.as_ref() == Some(item),
            Self::TimeCheck { min, max } => context.game_time >= *min && context.game_time <= *max,
            Self::ValueCheck { provider, min, max } => {
                let value = provider.float(context);
                value >= *min && value <= *max
            }
            Self::Reference(name) => context.condition_references.contains(name),
            Self::Inverted(condition) => !condition.matches_mut(context),
            Self::AllOf(conditions) => conditions
                .iter()
                .all(|condition| condition.matches_mut(context)),
            Self::AnyOf(conditions) => conditions
                .iter()
                .any(|condition| condition.matches_mut(context)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootFunction {
    SetCount(NumberProvider),
    LimitCount {
        min: i32,
        max: i32,
    },
    AddLootingBonus {
        per_level: NumberProvider,
        limit: Option<i32>,
    },
    SetItem(String),
    ApplyExplosionDecay,
    Filtered {
        condition: LootCondition,
        function: Box<LootFunction>,
    },
    Sequence(Vec<LootFunction>),
}

impl LootFunction {
    pub fn apply(&self, mut stack: LootStack, context: &mut LootContext) -> Option<LootStack> {
        match self {
            Self::SetCount(provider) => {
                stack.count = provider.int(context);
                Some(stack)
            }
            Self::LimitCount { min, max } => {
                stack.count = stack.count.clamp(*min, *max);
                Some(stack)
            }
            Self::AddLootingBonus { per_level, limit } => {
                stack.count += per_level.int(context) * context.looting_level.max(0);
                if let Some(limit) = limit {
                    stack.count = stack.count.min(*limit);
                }
                Some(stack)
            }
            Self::SetItem(item) => {
                stack.item.clone_from(item);
                Some(stack)
            }
            Self::ApplyExplosionDecay => {
                if let Some(radius) = context.explosion_radius {
                    if radius > 0.0 {
                        let chance = 1.0 / radius;
                        let kept = (0..stack.count)
                            .filter(|_| context.random.next_f32() <= chance)
                            .count() as i32;
                        stack.count = kept;
                    }
                }
                (!stack.is_empty()).then_some(stack)
            }
            Self::Filtered {
                condition,
                function,
            } => {
                if condition.matches(context) {
                    function.apply(stack, context)
                } else {
                    Some(stack)
                }
            }
            Self::Sequence(functions) => {
                let mut next = Some(stack);
                for function in functions {
                    next = function.apply(next?, context);
                }
                next
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LootContext {
    pub param_set: LootParamSet,
    pub entity_type: LootContextEntityType,
    pub params: LootParams,
    pub dynamic_params: HashMap<LootDynamicParamKey, LootDynamicParamValue>,
    pub random: DeterministicRandom,
    pub luck: f32,
    pub looting_level: i32,
    pub enchantment_level: i32,
    pub enchantment_active: bool,
    pub killed_by_player: bool,
    pub explosion_radius: Option<f32>,
    pub game_time: i64,
    pub block: Option<String>,
    pub tool: Option<String>,
    pub scores: HashMap<String, i32>,
    pub storage_numbers: HashMap<String, f32>,
    pub context_nbt: HashMap<String, HashMap<String, String>>,
    pub storage_nbt: HashMap<String, HashMap<String, String>>,
    pub entity_properties: HashMap<String, String>,
    pub condition_references: HashSet<String>,
    pub tables: HashMap<String, LootTable>,
    pub tags: HashMap<String, Vec<String>>,
    pub dynamic_drops: HashMap<String, Vec<LootStack>>,
    pub warnings: Vec<String>,
    max_stack_size: i32,
    visited: Vec<String>,
    base_seed: u64,
}

impl LootContext {
    pub fn new(param_set: LootParamSet, seed: u64) -> Self {
        Self {
            param_set,
            entity_type: entity_type_for_param_set(param_set),
            params: LootParams::default(),
            dynamic_params: HashMap::new(),
            random: DeterministicRandom::new(seed),
            luck: 0.0,
            looting_level: 0,
            enchantment_level: 0,
            enchantment_active: false,
            killed_by_player: false,
            explosion_radius: None,
            game_time: 0,
            block: None,
            tool: None,
            scores: HashMap::new(),
            storage_numbers: HashMap::new(),
            context_nbt: HashMap::new(),
            storage_nbt: HashMap::new(),
            entity_properties: HashMap::new(),
            condition_references: HashSet::new(),
            tables: HashMap::new(),
            tags: HashMap::new(),
            dynamic_drops: HashMap::new(),
            warnings: Vec::new(),
            max_stack_size: 64,
            visited: Vec::new(),
            base_seed: seed,
        }
    }

    fn clone_for_condition(&self) -> Self {
        Self {
            param_set: self.param_set,
            entity_type: self.entity_type,
            params: self.params.clone(),
            dynamic_params: self.dynamic_params.clone(),
            random: self.random,
            luck: self.luck,
            looting_level: self.looting_level,
            enchantment_level: self.enchantment_level,
            enchantment_active: self.enchantment_active,
            killed_by_player: self.killed_by_player,
            explosion_radius: self.explosion_radius,
            game_time: self.game_time,
            block: self.block.clone(),
            tool: self.tool.clone(),
            scores: self.scores.clone(),
            storage_numbers: self.storage_numbers.clone(),
            context_nbt: self.context_nbt.clone(),
            storage_nbt: self.storage_nbt.clone(),
            entity_properties: self.entity_properties.clone(),
            condition_references: self.condition_references.clone(),
            tables: HashMap::new(),
            tags: self.tags.clone(),
            dynamic_drops: self.dynamic_drops.clone(),
            warnings: Vec::new(),
            max_stack_size: self.max_stack_size,
            visited: self.visited.clone(),
            base_seed: self.base_seed,
        }
    }

    fn enter(&mut self, key: String) -> bool {
        if self.visited.contains(&key) {
            return false;
        }
        self.visited.push(key);
        true
    }

    fn exit(&mut self) {
        self.visited.pop();
    }

    fn use_random_sequence(&mut self, sequence: &str) {
        self.random = DeterministicRandom::new(hash_seed(self.base_seed, sequence));
    }

    pub fn insert_param(&mut self, value: LootParamValue) {
        match &value {
            LootParamValue::BlockState(block) => self.block = Some(block.clone()),
            LootParamValue::Origin(x, y, z) => {
                self.entity_properties
                    .insert("origin".to_string(), format!("{x},{y},{z}"));
            }
            LootParamValue::Tool(tool) => self.tool = Some(tool.clone()),
            LootParamValue::ThisEntity(entity) => {
                self.entity_properties
                    .insert("this_entity".to_string(), entity.clone());
            }
            LootParamValue::LastDamagePlayer(player) => {
                self.killed_by_player = true;
                self.entity_properties
                    .insert("last_damage_player".to_string(), player.clone());
            }
            LootParamValue::KillerEntity(entity) => {
                self.entity_properties
                    .insert("killer_entity".to_string(), entity.clone());
            }
            LootParamValue::DirectKillerEntity(entity) => {
                self.entity_properties
                    .insert("direct_killer_entity".to_string(), entity.clone());
            }
            LootParamValue::ExplosionRadius(radius) => self.explosion_radius = Some(*radius),
            LootParamValue::DamageSource(source) => {
                self.entity_properties
                    .insert("damage_source".to_string(), source.clone());
            }
            LootParamValue::BlockEntity(entity) => {
                self.entity_properties
                    .insert("block_entity".to_string(), entity.clone());
            }
        }
        self.params.insert(value);
    }

    pub fn insert_dynamic_param(&mut self, value: LootDynamicParamValue) {
        match &value {
            LootDynamicParamValue::EnchantmentLevel(level) => self.enchantment_level = *level,
            LootDynamicParamValue::EnchantmentActive(active) => self.enchantment_active = *active,
            LootDynamicParamValue::AttackingEntity(entity) => {
                self.entity_properties
                    .insert("attacking_entity".to_string(), entity.clone());
            }
        }
        self.dynamic_params.insert(value.key(), value);
    }
}

fn entity_type_for_param_set(param_set: LootParamSet) -> LootContextEntityType {
    match param_set {
        LootParamSet::Block => LootContextEntityType::Block,
        LootParamSet::Entity => LootContextEntityType::Entity,
        LootParamSet::Chest => LootContextEntityType::Chest,
        LootParamSet::Fishing => LootContextEntityType::Fishing,
        LootParamSet::Archaeology => LootContextEntityType::Archaeology,
        LootParamSet::AdvancementReward => LootContextEntityType::AdvancementReward,
        LootParamSet::Gift => LootContextEntityType::Gift,
        LootParamSet::Barter => LootContextEntityType::Barter,
        LootParamSet::Vault => LootContextEntityType::Vault,
        LootParamSet::Command => LootContextEntityType::Command,
        LootParamSet::Selector => LootContextEntityType::Selector,
        LootParamSet::AdvancementEntity => LootContextEntityType::AdvancementEntity,
        LootParamSet::Equipment => LootContextEntityType::Equipment,
        LootParamSet::Empty | LootParamSet::AllParams => LootContextEntityType::Command,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeterministicRandom {
    state: u64,
}

impl DeterministicRandom {
    pub fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_i32(&mut self, bound: i32) -> i32 {
        if bound <= 1 {
            return 0;
        }
        (self.next_u64() % bound as u64) as i32
    }

    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u64 << 24) as f32)
    }
}

fn split_stacks(stacks: Vec<LootStack>, max_stack_size: i32) -> Vec<LootStack> {
    let mut result = Vec::new();
    for stack in stacks.into_iter().filter(|stack| !stack.is_empty()) {
        let mut remaining = stack.count;
        while remaining > 0 {
            let count = remaining.min(max_stack_size);
            result.push(LootStack::new(stack.item.clone(), count));
            remaining -= count;
        }
    }
    result
}

fn shuffle<T>(values: &mut [T], random: &mut DeterministicRandom) {
    for index in (1..values.len()).rev() {
        let other = random.next_i32(index as i32 + 1) as usize;
        values.swap(index, other);
    }
}

fn hash_seed(base: u64, text: &str) -> u64 {
    let mut hash = base ^ 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advancement_system::{
        AdvancementDefinition, AdvancementRewards, PlayerAdvancementSet,
    };
    use crate::registry::Identifier;

    fn table_with_pool(pool: LootPool) -> LootTable {
        LootTable {
            param_set: LootParamSet::AllParams,
            random_sequence: None,
            pools: vec![pool],
            functions: Vec::new(),
        }
    }

    #[test]
    fn context_entity_types_params_and_dynamic_params_cover_java_surface() {
        let entity_types = [
            LootContextEntityType::Block,
            LootContextEntityType::Entity,
            LootContextEntityType::Chest,
            LootContextEntityType::Fishing,
            LootContextEntityType::Archaeology,
            LootContextEntityType::AdvancementReward,
            LootContextEntityType::Gift,
            LootContextEntityType::Barter,
            LootContextEntityType::Vault,
            LootContextEntityType::Command,
            LootContextEntityType::Selector,
            LootContextEntityType::AdvancementEntity,
            LootContextEntityType::Equipment,
        ];
        assert_eq!(entity_types.len(), 13);
        assert_eq!(
            LootSurface::Vault.entity_type(),
            LootContextEntityType::Vault
        );
        assert_eq!(
            LootContextEntityType::Equipment.param_set(),
            LootParamSet::Equipment
        );

        let mut params = LootParams::default();
        for value in [
            LootParamValue::BlockState("minecraft:stone".to_string()),
            LootParamValue::BlockEntity("Chest".to_string()),
            LootParamValue::Origin(1.0, 64.0, 2.0),
            LootParamValue::Tool("minecraft:diamond_pickaxe".to_string()),
            LootParamValue::ThisEntity("Zombie".to_string()),
            LootParamValue::LastDamagePlayer("Steve".to_string()),
            LootParamValue::KillerEntity("Steve".to_string()),
            LootParamValue::DirectKillerEntity("Arrow".to_string()),
            LootParamValue::ExplosionRadius(2.0),
            LootParamValue::DamageSource("minecraft:player_attack".to_string()),
        ] {
            params.insert(value);
        }
        assert_eq!(params.keys().len(), 10);
        assert!(matches!(
            params.get(LootParamKey::DamageSource),
            Some(LootParamValue::DamageSource(id)) if id == "minecraft:player_attack"
        ));

        let mut context = LootContext::new(LootParamSet::Entity, 4);
        context.insert_param(LootParamValue::ThisEntity("Zombie".to_string()));
        context.insert_param(LootParamValue::LastDamagePlayer("Steve".to_string()));
        context.insert_param(LootParamValue::ExplosionRadius(3.0));
        context.insert_dynamic_param(LootDynamicParamValue::EnchantmentLevel(5));
        context.insert_dynamic_param(LootDynamicParamValue::EnchantmentActive(true));
        context.insert_dynamic_param(LootDynamicParamValue::AttackingEntity("Steve".to_string()));

        assert_eq!(context.params.keys().len(), 3);
        assert_eq!(context.enchantment_level, 5);
        assert!(context.enchantment_active);
        assert!(context.killed_by_player);
        assert_eq!(context.explosion_radius, Some(3.0));
        assert_eq!(
            NumberProvider::EnchantmentLevel { scale: 2.0 }.float(&mut context),
            10.0
        );
        assert!(matches!(
            context
                .dynamic_params
                .get(&LootDynamicParamKey::AttackingEntity),
            Some(LootDynamicParamValue::AttackingEntity(entity)) if entity == "Steve"
        ));
    }

    #[test]
    fn score_and_nbt_providers_resolve_context_and_storage_values() {
        let mut context = LootContext::new(LootParamSet::AllParams, 9);
        context
            .entity_properties
            .insert("this_entity".to_string(), "Zombie".to_string());
        context.context_nbt.insert(
            "this_entity".to_string(),
            HashMap::from([("CustomName".to_string(), "\"Dinnerbone\"".to_string())]),
        );
        context.storage_nbt.insert(
            "minecraft:loot_state".to_string(),
            HashMap::from([("bonus".to_string(), "enabled".to_string())]),
        );

        assert_eq!(
            ScoreProvider::Context {
                target: "this_entity".to_string()
            }
            .scoreboard_name(&context),
            Some("Zombie".to_string())
        );
        assert_eq!(
            ScoreProvider::Fixed {
                name: "global_counter".to_string()
            }
            .scoreboard_name(&context),
            Some("global_counter".to_string())
        );
        assert_eq!(
            NbtProvider::Context {
                target: "this_entity".to_string(),
                path: "CustomName".to_string(),
            }
            .get(&context),
            Some("\"Dinnerbone\"")
        );
        assert_eq!(
            NbtProvider::Storage {
                key: "minecraft:loot_state".to_string(),
                path: "bonus".to_string(),
            }
            .get(&context),
            Some("enabled")
        );
    }

    #[test]
    fn weighted_entries_rolls_bonus_rolls_and_luck_follow_pool_shape() {
        let mut pool = LootPool::single(LootEntry::item("minecraft:stick", 1));
        pool.entries.push(LootEntry::Item {
            item: "minecraft:diamond".to_string(),
            weight: 1,
            quality: 10,
            conditions: Vec::new(),
            functions: vec![LootFunction::SetCount(NumberProvider::Constant(2.0))],
        });
        pool.rolls = NumberProvider::Constant(2.0);
        pool.bonus_rolls = NumberProvider::Constant(1.0);

        let table = table_with_pool(pool);
        let mut context = LootContext::new(LootParamSet::Chest, 11);
        context.luck = 2.0;
        let drops = table.evaluate(&mut context);

        assert_eq!(drops.len(), 4);
        assert!(drops.iter().all(|stack| stack.item == "minecraft:diamond"));
        assert!(drops.iter().all(|stack| stack.count == 2));
    }

    #[test]
    fn conditions_cover_random_player_explosion_time_tool_score_and_combinators() {
        let condition = LootCondition::AllOf(vec![
            LootCondition::KilledByPlayer,
            LootCondition::SurvivesExplosion,
            LootCondition::TimeCheck { min: 10, max: 30 },
            LootCondition::MatchTool {
                item: "minecraft:diamond_pickaxe".to_string(),
            },
            LootCondition::ValueCheck {
                provider: NumberProvider::Score {
                    name: "kills".to_string(),
                    scale: 0.5,
                },
                min: 2.0,
                max: 4.0,
            },
            LootCondition::Inverted(Box::new(LootCondition::BlockState {
                block: "minecraft:dirt".to_string(),
            })),
            LootCondition::AnyOf(vec![
                LootCondition::Reference("bonus".to_string()),
                LootCondition::RandomChance(0.0),
            ]),
        ]);

        let mut context = LootContext::new(LootParamSet::Block, 7);
        context.killed_by_player = true;
        context.explosion_radius = None;
        context.game_time = 20;
        context.tool = Some("minecraft:diamond_pickaxe".to_string());
        context.block = Some("minecraft:stone".to_string());
        context.scores.insert("kills".to_string(), 6);
        context.condition_references.insert("bonus".to_string());

        assert!(condition.matches(&context));
    }

    #[test]
    fn entry_types_cover_tags_nested_tables_dynamic_alternatives_sequences_and_groups() {
        let nested = table_with_pool(LootPool::single(LootEntry::item("minecraft:apple", 1)));
        let pool = LootPool {
            entries: vec![LootEntry::Group(vec![
                LootEntry::Tag {
                    tag: "minecraft:logs".to_string(),
                    expand: true,
                    weight: 1,
                    quality: 0,
                    conditions: Vec::new(),
                },
                LootEntry::NestedTable("minecraft:bonus".to_string()),
                LootEntry::Dynamic("minecraft:sherds".to_string()),
                LootEntry::Alternatives(vec![
                    LootEntry::Empty {
                        weight: 0,
                        quality: 0,
                        conditions: Vec::new(),
                    },
                    LootEntry::item("minecraft:emerald", 1),
                ]),
                LootEntry::Sequence(vec![
                    LootEntry::item("minecraft:string", 1),
                    LootEntry::item("minecraft:feather", 1),
                ]),
            ])],
            conditions: Vec::new(),
            functions: Vec::new(),
            rolls: NumberProvider::Constant(1.0),
            bonus_rolls: NumberProvider::Constant(0.0),
        };
        let table = table_with_pool(pool);
        let mut context = LootContext::new(LootParamSet::Archaeology, 3);
        context.tables.insert("minecraft:bonus".to_string(), nested);
        context.tags.insert(
            "minecraft:logs".to_string(),
            vec![
                "minecraft:oak_log".to_string(),
                "minecraft:birch_log".to_string(),
            ],
        );
        context.dynamic_drops.insert(
            "minecraft:sherds".to_string(),
            vec![LootStack::new("minecraft:angler_pottery_sherd", 1)],
        );

        let names: Vec<_> = table
            .evaluate(&mut context)
            .into_iter()
            .map(|stack| stack.item)
            .collect();
        assert!(names.contains(&"minecraft:oak_log".to_string()));
        assert!(names.contains(&"minecraft:birch_log".to_string()));
        assert!(names.contains(&"minecraft:apple".to_string()));
        assert!(names.contains(&"minecraft:angler_pottery_sherd".to_string()));
        assert!(names.contains(&"minecraft:emerald".to_string()));
        assert!(names.contains(&"minecraft:string".to_string()));
        assert!(names.contains(&"minecraft:feather".to_string()));
    }

    #[test]
    fn functions_apply_in_entry_pool_and_table_order_with_stack_splitting() {
        let mut pool = LootPool::single(LootEntry::Item {
            item: "minecraft:stone".to_string(),
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: vec![LootFunction::SetCount(NumberProvider::Constant(130.0))],
        });
        pool.functions
            .push(LootFunction::LimitCount { min: 1, max: 70 });
        let mut table = table_with_pool(pool);
        table
            .functions
            .push(LootFunction::SetItem("minecraft:cobblestone".to_string()));

        let mut context = LootContext::new(LootParamSet::Block, 13);
        let drops = table.evaluate(&mut context);

        assert_eq!(
            drops,
            vec![
                LootStack::new("minecraft:cobblestone", 64),
                LootStack::new("minecraft:cobblestone", 6)
            ]
        );
    }

    #[test]
    fn random_sequences_are_repeatable_and_independent_from_context_seed_stream() {
        let mut table = table_with_pool(LootPool {
            entries: vec![
                LootEntry::item("minecraft:coal", 1),
                LootEntry::item("minecraft:iron_ingot", 1),
            ],
            conditions: Vec::new(),
            functions: Vec::new(),
            rolls: NumberProvider::Constant(8.0),
            bonus_rolls: NumberProvider::Constant(0.0),
        });
        table.random_sequence = Some("minecraft:chests/simple_dungeon".to_string());

        let mut first = LootContext::new(LootParamSet::Chest, 99);
        let mut second = LootContext::new(LootParamSet::Chest, 99);
        let mut different_sequence = table.clone();
        different_sequence.random_sequence =
            Some("minecraft:chests/abandoned_mineshaft".to_string());

        assert_eq!(table.evaluate(&mut first), table.evaluate(&mut second));
        assert_ne!(
            table.evaluate(&mut LootContext::new(LootParamSet::Chest, 99)),
            different_sequence.evaluate(&mut LootContext::new(LootParamSet::Chest, 99))
        );
    }

    #[test]
    fn container_fill_shuffles_once_and_reports_overfill() {
        let mut pool = LootPool::single(LootEntry::Item {
            item: "minecraft:bread".to_string(),
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: vec![LootFunction::SetCount(NumberProvider::Constant(1.0))],
        });
        pool.rolls = NumberProvider::Constant(4.0);
        let table = table_with_pool(pool);
        let mut context = LootContext::new(LootParamSet::Chest, 5);

        let slots = table.fill_container(&mut context, 2);

        assert_eq!(slots.iter().filter(|slot| slot.is_some()).count(), 2);
        assert_eq!(context.warnings, vec!["Tried to over-fill a container"]);
    }

    #[test]
    fn validation_reports_invalid_pool_and_entry_shapes() {
        let table = table_with_pool(LootPool {
            entries: vec![LootEntry::Tag {
                tag: String::new(),
                expand: false,
                weight: -1,
                quality: 0,
                conditions: Vec::new(),
            }],
            conditions: Vec::new(),
            functions: Vec::new(),
            rolls: NumberProvider::Constant(-1.0),
            bonus_rolls: NumberProvider::Constant(0.0),
        });

        let errors = table.validate();
        assert!(errors.iter().any(|error| error.contains("rolls")));
        assert!(errors.iter().any(|error| error.contains("invalid tag")));
    }

    #[test]
    fn behavior_engine_maps_named_surfaces_to_vanilla_param_sets_and_delivery() {
        let mut engine = LootBehaviorEngine::new();
        engine.insert_table(
            "minecraft:blocks/stone",
            table_with_pool(LootPool::single(LootEntry::Item {
                item: "minecraft:cobblestone".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![LootCondition::AllOf(vec![
                    LootCondition::BlockState {
                        block: "minecraft:stone".to_string(),
                    },
                    LootCondition::MatchTool {
                        item: "minecraft:iron_pickaxe".to_string(),
                    },
                ])],
                functions: Vec::new(),
            })),
        );

        let mut request = LootRequest::new(LootSurface::BlockBreak, "minecraft:blocks/stone");
        request.origin = (1.0, 64.0, 2.0);
        request.block = Some("minecraft:stone".to_string());
        request.tool = Some("minecraft:iron_pickaxe".to_string());

        let resolution = engine.resolve(request, 17);

        assert_eq!(resolution.param_set, LootParamSet::Block);
        assert_eq!(
            resolution.delivery,
            LootDelivery::DropAt(
                (1.0, 64.0, 2.0),
                vec![LootStack::new("minecraft:cobblestone", 1)]
            )
        );
    }

    #[test]
    fn behavior_engine_covers_entity_fishing_archaeology_reward_gift_barter_and_command_paths() {
        let mut engine = LootBehaviorEngine::new();
        for (id, item) in [
            ("minecraft:entities/zombie", "minecraft:rotten_flesh"),
            ("minecraft:gameplay/fishing", "minecraft:cod"),
            (
                "minecraft:archaeology/desert_pyramid",
                "minecraft:pottery_sherd",
            ),
            (
                "minecraft:advancements/story/mine_stone",
                "minecraft:emerald",
            ),
            ("minecraft:gameplay/cat_morning_gift", "minecraft:string"),
            ("minecraft:gameplay/piglin_bartering", "minecraft:quartz"),
            ("minecraft:commands/debug", "minecraft:stick"),
        ] {
            engine.insert_table(
                id,
                table_with_pool(LootPool::single(LootEntry::item(item, 1))),
            );
        }

        let mut entity = LootRequest::new(LootSurface::EntityDeath, "minecraft:entities/zombie");
        entity.target_entity = Some("Zombie".to_string());
        entity.damage_source = Some("minecraft:player_attack".to_string());
        entity.killed_by_player = true;
        assert_eq!(engine.resolve(entity, 1).param_set, LootParamSet::Entity);

        let mut fishing =
            LootRequest::new(LootSurface::FishingRetrieve, "minecraft:gameplay/fishing");
        fishing.tool = Some("minecraft:fishing_rod".to_string());
        assert_eq!(engine.resolve(fishing, 1).param_set, LootParamSet::Fishing);

        let mut archaeology = LootRequest::new(
            LootSurface::ArchaeologyBrush,
            "minecraft:archaeology/desert_pyramid",
        );
        archaeology.tool = Some("minecraft:brush".to_string());
        assert_eq!(
            engine.resolve(archaeology, 1).param_set,
            LootParamSet::Archaeology
        );

        let mut advancement = LootRequest::new(
            LootSurface::AdvancementReward,
            "minecraft:advancements/story/mine_stone",
        );
        advancement.actor = Some("Steve".to_string());
        assert_eq!(
            engine.resolve(advancement, 1).delivery,
            LootDelivery::GiveToEntity(
                "Steve".to_string(),
                vec![LootStack::new("minecraft:emerald", 1)]
            )
        );

        let mut gift = LootRequest::new(LootSurface::Gift, "minecraft:gameplay/cat_morning_gift");
        gift.actor = Some("Steve".to_string());
        assert_eq!(engine.resolve(gift, 1).param_set, LootParamSet::Gift);

        let mut barter = LootRequest::new(
            LootSurface::PiglinBarter,
            "minecraft:gameplay/piglin_bartering",
        );
        barter.actor = Some("Piglin".to_string());
        assert_eq!(engine.resolve(barter, 1).param_set, LootParamSet::Barter);

        let mut command = LootRequest::new(LootSurface::Command, "minecraft:commands/debug");
        command.actor = Some("Steve".to_string());
        assert_eq!(
            engine.resolve(command, 1).delivery,
            LootDelivery::GiveToEntity(
                "Steve".to_string(),
                vec![LootStack::new("minecraft:stick", 1)]
            )
        );
    }

    #[test]
    fn entity_death_context_carries_java_kill_params_and_looting_bonus() {
        let mut engine = LootBehaviorEngine::new();
        let mut pool = LootPool::single(LootEntry::Item {
            item: "minecraft:rotten_flesh".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![
                LootCondition::KilledByPlayer,
                LootCondition::EntityProperty {
                    key: "this_entity".to_string(),
                    value: "Zombie".to_string(),
                },
                LootCondition::EntityProperty {
                    key: "killer_entity".to_string(),
                    value: "Steve".to_string(),
                },
                LootCondition::EntityProperty {
                    key: "direct_killer_entity".to_string(),
                    value: "Arrow".to_string(),
                },
                LootCondition::EntityProperty {
                    key: "last_damage_player".to_string(),
                    value: "Steve".to_string(),
                },
            ],
            functions: vec![LootFunction::AddLootingBonus {
                per_level: NumberProvider::Constant(2.0),
                limit: Some(5),
            }],
        });
        pool.conditions
            .push(LootCondition::RandomChanceWithLooting {
                chance: 0.0,
                looting_multiplier: 1.0,
            });
        engine.insert_table("minecraft:entities/zombie", table_with_pool(pool));

        let mut request = LootRequest::new(LootSurface::EntityDeath, "minecraft:entities/zombie");
        request.origin = (2.0, 64.0, 3.0);
        request.target_entity = Some("Zombie".to_string());
        request.killer_entity = Some("Steve".to_string());
        request.direct_killer_entity = Some("Arrow".to_string());
        request.last_damage_player = Some("Steve".to_string());
        request.damage_source = Some("minecraft:arrow".to_string());
        request.looting_level = 2;

        let resolution = engine.resolve(request, 11);

        assert_eq!(resolution.param_set, LootParamSet::Entity);
        assert_eq!(
            resolution.delivery,
            LootDelivery::DropAt(
                (2.0, 64.0, 3.0),
                vec![LootStack::new("minecraft:rotten_flesh", 5)]
            )
        );
    }

    #[test]
    fn advancement_reward_loot_grants_xp_and_tables_with_player_context() {
        let mut advancements = PlayerAdvancementSet::default();
        let definition = AdvancementDefinition::all_of(
            "minecraft:story/mine_stone",
            None,
            &["stone"],
            AdvancementRewards {
                experience: 5,
                loot: vec![Identifier::parse("minecraft:advancements/story/mine_stone").unwrap()],
                recipes: Vec::new(),
                function: None,
            },
            None,
        )
        .unwrap();
        let reward = advancements.grant(&definition, "stone", 1).unwrap();
        let mut engine = LootBehaviorEngine::new();
        engine.insert_table(
            "minecraft:advancements/story/mine_stone",
            table_with_pool(LootPool::single(LootEntry::Item {
                item: "minecraft:emerald".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![LootCondition::EntityProperty {
                    key: "this_entity".to_string(),
                    value: "Steve".to_string(),
                }],
                functions: Vec::new(),
            })),
        );

        let output = resolve_advancement_reward_loot(
            &engine,
            AdvancementRewardLootInput {
                player: "Steve".to_string(),
                origin: (8.0, 65.0, 9.0),
                experience: reward.experience,
                loot_tables: reward.loot.iter().map(ToString::to_string).collect(),
            },
            44,
        );

        assert_eq!(output.experience, 5);
        assert_eq!(output.loot.len(), 1);
        assert_eq!(output.loot[0].param_set, LootParamSet::AdvancementReward);
        assert_eq!(
            output.loot[0].delivery,
            LootDelivery::GiveToEntity(
                "Steve".to_string(),
                vec![LootStack::new("minecraft:emerald", 1)]
            )
        );
    }

    #[test]
    fn piglin_barter_accepts_gold_and_uses_barter_context() {
        let mut engine = LootBehaviorEngine::new();
        engine.insert_table(
            "minecraft:gameplay/piglin_bartering",
            table_with_pool(LootPool::single(LootEntry::Item {
                item: "minecraft:quartz".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![LootCondition::EntityProperty {
                    key: "this_entity".to_string(),
                    value: "Piglin".to_string(),
                }],
                functions: Vec::new(),
            })),
        );

        assert!(resolve_piglin_barter_loot(
            &engine,
            "Piglin",
            "minecraft:iron_ingot",
            (0.0, 64.0, 0.0),
            7,
        )
        .is_none());

        let resolution = resolve_piglin_barter_loot(
            &engine,
            "Piglin",
            "minecraft:gold_ingot",
            (4.0, 65.0, 4.0),
            7,
        )
        .unwrap();

        assert_eq!(resolution.param_set, LootParamSet::Barter);
        assert_eq!(
            resolution.delivery,
            LootDelivery::GiveToEntity(
                "Piglin".to_string(),
                vec![LootStack::new("minecraft:quartz", 1)]
            )
        );
    }

    #[test]
    fn fishing_loot_uses_tool_origin_luck_and_category_tables() {
        let mut engine = LootBehaviorEngine::new();
        engine.insert_table(
            "minecraft:gameplay/fishing/junk",
            table_with_pool(LootPool::single(LootEntry::item("minecraft:bowl", 1))),
        );
        engine.insert_table(
            "minecraft:gameplay/fishing/fish",
            table_with_pool(LootPool::single(LootEntry::item("minecraft:cod", 1))),
        );
        engine.insert_table(
            "minecraft:gameplay/fishing/treasure",
            table_with_pool(LootPool::single(LootEntry::item("minecraft:name_tag", 1))),
        );
        let fishing_table = LootTable {
            param_set: LootParamSet::Fishing,
            random_sequence: Some("minecraft:gameplay/fishing".to_string()),
            pools: vec![LootPool {
                entries: vec![
                    LootEntry::WeightedNestedTable {
                        table: "minecraft:gameplay/fishing/junk".to_string(),
                        weight: 1,
                        quality: -1,
                        conditions: Vec::new(),
                    },
                    LootEntry::WeightedNestedTable {
                        table: "minecraft:gameplay/fishing/fish".to_string(),
                        weight: 0,
                        quality: 0,
                        conditions: Vec::new(),
                    },
                    LootEntry::WeightedNestedTable {
                        table: "minecraft:gameplay/fishing/treasure".to_string(),
                        weight: 1,
                        quality: 2,
                        conditions: vec![LootCondition::EntityProperty {
                            key: "in_open_water".to_string(),
                            value: "true".to_string(),
                        }],
                    },
                ],
                conditions: Vec::new(),
                functions: Vec::new(),
                rolls: NumberProvider::Constant(1.0),
                bonus_rolls: NumberProvider::Constant(0.0),
            }],
            functions: Vec::new(),
        };
        engine.insert_table("minecraft:gameplay/fishing", fishing_table);

        let no_open_water = resolve_fishing_loot(
            &engine,
            "minecraft:fishing_rod",
            (3.0, 62.0, 4.0),
            0.0,
            0.0,
            false,
            3,
        );
        assert_eq!(no_open_water.param_set, LootParamSet::Fishing);
        assert_eq!(
            no_open_water.delivery,
            LootDelivery::DropAt((3.0, 62.0, 4.0), vec![LootStack::new("minecraft:bowl", 1)])
        );

        let lucky_open_water = resolve_fishing_loot(
            &engine,
            "minecraft:fishing_rod",
            (3.0, 62.0, 4.0),
            2.0,
            8.0,
            true,
            3,
        );
        assert_eq!(
            lucky_open_water.delivery,
            LootDelivery::DropAt(
                (3.0, 62.0, 4.0),
                vec![LootStack::new("minecraft:name_tag", 1)]
            )
        );
    }

    #[test]
    fn randomizable_container_loot_realizes_table_once_and_preserves_seed() {
        let mut engine = LootBehaviorEngine::new();
        let mut pool = LootPool::single(LootEntry::Item {
            item: "minecraft:bread".to_string(),
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: vec![LootFunction::SetCount(NumberProvider::Uniform {
                min: 1.0,
                max: 4.0,
            })],
        });
        pool.rolls = NumberProvider::Constant(3.0);
        engine.insert_table("minecraft:chests/simple_dungeon", table_with_pool(pool));

        let mut first = RandomizableContainerLoot {
            loot_table: Some("minecraft:chests/simple_dungeon".to_string()),
            loot_table_seed: 42,
            unpacked: false,
        };
        let mut second = first.clone();

        let first_slots = first.unpack_once(&engine, (0.0, 64.0, 0.0)).unwrap();
        let second_slots = second.unpack_once(&engine, (0.0, 64.0, 0.0)).unwrap();

        assert_eq!(first_slots, second_slots);
        assert!(first.unpack_once(&engine, (0.0, 64.0, 0.0)).is_none());
    }

    #[test]
    fn loot_table_resources_decode_all_vanilla_tables() {
        let root = std::path::Path::new("../decompiled-server-26.1.2/data/minecraft/loot_table");
        let mut paths = Vec::new();
        collect_json_paths(root, &mut paths);
        paths.sort();

        assert_eq!(paths.len(), 1326);
        let mut param_sets = HashSet::new();
        let mut nested_table_count = 0;
        let mut item_entry_count = 0;
        for path in &paths {
            let table = load_loot_table_resource(path).unwrap_or_else(|err| {
                panic!("{} failed to decode: {err}", path.display());
            });
            param_sets.insert(table.param_set);
            for pool in table.pools {
                assert!(
                    !pool.entries.is_empty(),
                    "{} has empty pool",
                    path.display()
                );
                for entry in pool.entries {
                    let entry_type = entry
                        .get("type")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default();
                    if entry_type == "minecraft:item" {
                        item_entry_count += 1;
                    } else if entry_type == "minecraft:loot_table" {
                        nested_table_count += 1;
                    }
                }
            }
        }

        assert!(param_sets.contains("minecraft:block"));
        assert!(param_sets.contains("minecraft:chest"));
        assert!(param_sets.contains("minecraft:entity"));
        assert!(param_sets.contains("minecraft:fishing"));
        assert!(item_entry_count > 1_000);
        assert!(nested_table_count > 0);
    }

    #[test]
    fn loot_table_resource_defaults_match_java_direct_codec() {
        let table = parse_loot_table_resource("{}").unwrap();
        assert_eq!(table.param_set, "minecraft:all_params");
        assert_eq!(table.random_sequence, None);
        assert!(table.pools.is_empty());
        assert!(table.functions.is_empty());

        assert!(parse_loot_table_resource(r#"{"pools":[{"rolls":1.0}]}"#).is_err());
        assert!(parse_loot_table_resource(
            r#"{"pools":[{"rolls":1.0,"entries":[{"type":"minecraft:item"}]}]}"#
        )
        .is_err());
    }

    fn collect_json_paths(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(root).unwrap() {
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
