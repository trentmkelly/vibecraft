#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

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
    Command,
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
    Command,
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
            Self::Command => LootParamSet::Command,
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
    pub block: Option<String>,
    pub tool: Option<String>,
    pub damage_source: Option<String>,
    pub explosion_radius: Option<f32>,
    pub luck: f32,
    pub looting_level: i32,
    pub killed_by_player: bool,
}

impl LootRequest {
    pub fn new(surface: LootSurface, table: impl Into<String>) -> Self {
        Self {
            surface,
            table: table.into(),
            origin: (0.0, 0.0, 0.0),
            actor: None,
            target_entity: None,
            block: None,
            tool: None,
            damage_source: None,
            explosion_radius: None,
            luck: 0.0,
            looting_level: 0,
            killed_by_player: false,
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
            | LootSurface::ArchaeologyBrush => {
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
        context.luck = request.luck;
        context.looting_level = request.looting_level;
        context.killed_by_player = request.killed_by_player;
        context.explosion_radius = request.explosion_radius;
        context.block = request.block.clone();
        context.tool = request.tool.clone();
        if let Some(actor) = &request.actor {
            context
                .entity_properties
                .insert("actor".to_string(), actor.clone());
        }
        if let Some(target) = &request.target_entity {
            context
                .entity_properties
                .insert("this_entity".to_string(), target.clone());
        }
        if let Some(damage_source) = &request.damage_source {
            context
                .entity_properties
                .insert("damage_source".to_string(), damage_source.clone());
        }
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
            Self::NestedTable(table) => {
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
            Self::NestedTable(table) | Self::Dynamic(table) if table.is_empty() => {
                errors.push(format!("{path} has an empty reference"));
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
        }
    }

    fn minimum(&self) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Uniform { min, .. } => *min,
            Self::Binomial { .. } => 0.0,
            Self::Score { .. } | Self::Storage { .. } => f32::MIN,
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
    pub random: DeterministicRandom,
    pub luck: f32,
    pub looting_level: i32,
    pub killed_by_player: bool,
    pub explosion_radius: Option<f32>,
    pub game_time: i64,
    pub block: Option<String>,
    pub tool: Option<String>,
    pub scores: HashMap<String, i32>,
    pub storage_numbers: HashMap<String, f32>,
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
            random: DeterministicRandom::new(seed),
            luck: 0.0,
            looting_level: 0,
            killed_by_player: false,
            explosion_radius: None,
            game_time: 0,
            block: None,
            tool: None,
            scores: HashMap::new(),
            storage_numbers: HashMap::new(),
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
            random: self.random,
            luck: self.luck,
            looting_level: self.looting_level,
            killed_by_player: self.killed_by_player,
            explosion_radius: self.explosion_radius,
            game_time: self.game_time,
            block: self.block.clone(),
            tool: self.tool.clone(),
            scores: self.scores.clone(),
            storage_numbers: self.storage_numbers.clone(),
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

    fn table_with_pool(pool: LootPool) -> LootTable {
        LootTable {
            param_set: LootParamSet::AllParams,
            random_sequence: None,
            pools: vec![pool],
            functions: Vec::new(),
        }
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
}
