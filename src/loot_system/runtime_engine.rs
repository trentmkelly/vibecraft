#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use super::*;

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
        context.fortune_level = request.fortune_level;
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
        if let Some(block_entity) = &request.block_entity {
            context.insert_param(LootParamValue::BlockEntity(block_entity.clone()));
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
            .entity_properties
            .insert("correct_tool".to_string(), request.correct_tool.to_string());
        context
            .entity_properties
            .insert("silk_touch".to_string(), request.silk_touch.to_string());
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

    pub(super) fn validate(&self, path: String, errors: &mut Vec<String>) {
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
    RandomChanceWithEnchantedBonus {
        unenchanted_chance: f32,
        enchanted_chance: f32,
    },
    KilledByPlayer,
    SurvivesExplosion,
    EntityProperty {
        key: String,
        value: String,
    },
    EntityScore {
        name: String,
        min: i32,
        max: i32,
    },
    BlockState {
        block: String,
    },
    BlockStateProperty {
        property: String,
        value: String,
    },
    MatchTool {
        item: String,
    },
    DamageSourceProperty {
        key: String,
        value: String,
    },
    LocationCheck {
        key: String,
        value: String,
    },
    WeatherCheck {
        raining: Option<bool>,
        thundering: Option<bool>,
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
    EnchantmentActiveCheck,
    TableBonus {
        chances: Vec<f32>,
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
            Self::RandomChanceWithEnchantedBonus {
                unenchanted_chance,
                enchanted_chance,
            } => {
                let chance = if context.enchantment_active {
                    *enchanted_chance
                } else {
                    *unenchanted_chance
                };
                context.random.next_f32() < chance
            }
            Self::KilledByPlayer => context.killed_by_player,
            Self::SurvivesExplosion => match context.explosion_radius {
                Some(radius) if radius > 0.0 => context.random.next_f32() <= 1.0 / radius,
                _ => true,
            },
            Self::EntityProperty { key, value } => {
                context.entity_properties.get(key) == Some(value)
            }
            Self::EntityScore { name, min, max } => context
                .scores
                .get(name)
                .is_some_and(|score| score >= min && score <= max),
            Self::BlockState { block } => context.block.as_ref() == Some(block),
            Self::BlockStateProperty { property, value } => {
                context.block_state_properties.get(property) == Some(value)
            }
            Self::MatchTool { item } => context.tool.as_ref() == Some(item),
            Self::DamageSourceProperty { key, value } => {
                context
                    .entity_properties
                    .get(&format!("damage_source.{key}"))
                    .or_else(|| {
                        if key == "type" {
                            context.entity_properties.get("damage_source")
                        } else {
                            None
                        }
                    })
                    == Some(value)
            }
            Self::LocationCheck { key, value } => context.entity_properties.get(key) == Some(value),
            Self::WeatherCheck {
                raining,
                thundering,
            } => {
                raining.map_or(true, |expected| context.weather_raining == expected)
                    && thundering.map_or(true, |expected| context.weather_thundering == expected)
            }
            Self::TimeCheck { min, max } => context.game_time >= *min && context.game_time <= *max,
            Self::ValueCheck { provider, min, max } => {
                let value = provider.float(context);
                value >= *min && value <= *max
            }
            Self::EnchantmentActiveCheck => context.enchantment_active,
            Self::TableBonus { chances } => {
                let index = context.enchantment_level.max(0) as usize;
                let chance = chances
                    .get(index)
                    .or_else(|| chances.last())
                    .copied()
                    .unwrap_or(0.0);
                context.random.next_f32() < chance
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
    ApplyFortuneBonus {
        per_level: NumberProvider,
        limit: Option<i32>,
    },
    ApplyBonus(LootBonusFormula),
    SetItem(String),
    SetDamage(NumberProvider),
    SetNbt(HashMap<String, String>),
    EnchantWithLevels {
        levels: NumberProvider,
        options: Vec<String>,
    },
    EnchantRandomly(Vec<String>),
    SmeltItem,
    CopyName {
        source: String,
    },
    CopyNbt {
        provider: NbtProvider,
        target: String,
    },
    SetContents(Vec<LootStack>),
    ExplorationMap {
        destination: String,
        decoration: String,
    },
    FillPlayerHead,
    CopyState(Vec<String>),
    SetAttributes(Vec<String>),
    SetBannerPatterns(Vec<String>),
    SetBookContents {
        title: String,
        author: String,
        pages: Vec<String>,
    },
    SetComponents(HashMap<String, String>),
    SetInstrument(String),
    SetLore(Vec<String>),
    SetName(String),
    SetPotion(String),
    SetStewEffects(Vec<String>),
    SetWrittenBookPages(Vec<String>),
    ToggleTooltips(Vec<String>),
    SetFireworkExplosions(Vec<String>),
    SetFireworks {
        flight_duration: i32,
        explosions: Vec<String>,
    },
    Reference(String),
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
            Self::ApplyFortuneBonus { per_level, limit } => {
                stack.count += per_level.int(context) * context.fortune_level.max(0);
                if let Some(limit) = limit {
                    stack.count = stack.count.min(*limit);
                }
                Some(stack)
            }
            Self::ApplyBonus(formula) => {
                stack.count = formula.apply(stack.count, context);
                Some(stack)
            }
            Self::SetItem(item) => {
                stack.item.clone_from(item);
                Some(stack)
            }
            Self::SetDamage(provider) => {
                stack.components.insert(
                    "minecraft:damage_fraction".to_string(),
                    provider.float(context).to_string(),
                );
                Some(stack)
            }
            Self::SetNbt(values) | Self::SetComponents(values) => {
                stack.components.extend(values.clone());
                Some(stack)
            }
            Self::EnchantWithLevels { levels, options } => {
                let level = levels.int(context).max(1);
                if !options.is_empty() {
                    let index = context.random.next_i32(options.len() as i32) as usize;
                    stack.components.insert(
                        "minecraft:enchantments".to_string(),
                        format!("{}:{level}", options[index]),
                    );
                }
                Some(stack)
            }
            Self::EnchantRandomly(options) => {
                if !options.is_empty() {
                    let index = context.random.next_i32(options.len() as i32) as usize;
                    stack.components.insert(
                        "minecraft:enchantments".to_string(),
                        format!("{}:1", options[index]),
                    );
                }
                Some(stack)
            }
            Self::SmeltItem => {
                if context.block_on_fire {
                    if let Some(result) = context.smelting_results.get(&stack.item) {
                        stack.item.clone_from(result);
                    }
                }
                Some(stack)
            }
            Self::CopyName { source } => {
                if let Some(name) = context.entity_properties.get(source) {
                    stack
                        .components
                        .insert("minecraft:custom_name".to_string(), name.clone());
                }
                Some(stack)
            }
            Self::CopyNbt { provider, target } => {
                if let Some(value) = provider.get(context) {
                    stack.components.insert(target.clone(), value.to_string());
                }
                Some(stack)
            }
            Self::SetContents(contents) => {
                stack.components.insert(
                    "minecraft:container".to_string(),
                    contents
                        .iter()
                        .map(|item| format!("{}:{}", item.item, item.count))
                        .collect::<Vec<_>>()
                        .join(","),
                );
                Some(stack)
            }
            Self::ExplorationMap {
                destination,
                decoration,
            } => {
                stack.item = "minecraft:filled_map".to_string();
                stack
                    .components
                    .insert("minecraft:map_destination".to_string(), destination.clone());
                stack
                    .components
                    .insert("minecraft:map_decoration".to_string(), decoration.clone());
                Some(stack)
            }
            Self::FillPlayerHead => {
                if let Some(player) = context
                    .entity_properties
                    .get("last_damage_player")
                    .or_else(|| context.entity_properties.get("killer_entity"))
                {
                    stack
                        .components
                        .insert("minecraft:profile".to_string(), player.clone());
                }
                Some(stack)
            }
            Self::CopyState(properties) => {
                for property in properties {
                    if let Some(value) = context.block_state_properties.get(property) {
                        stack
                            .components
                            .insert(format!("minecraft:block_state.{property}"), value.clone());
                    }
                }
                Some(stack)
            }
            Self::SetAttributes(attributes) => {
                stack.components.insert(
                    "minecraft:attribute_modifiers".to_string(),
                    attributes.join(","),
                );
                Some(stack)
            }
            Self::SetBannerPatterns(patterns) => {
                stack
                    .components
                    .insert("minecraft:banner_patterns".to_string(), patterns.join(","));
                Some(stack)
            }
            Self::SetBookContents {
                title,
                author,
                pages,
            } => {
                stack
                    .components
                    .insert("minecraft:written_book_title".to_string(), title.clone());
                stack
                    .components
                    .insert("minecraft:written_book_author".to_string(), author.clone());
                stack
                    .components
                    .insert("minecraft:written_book_pages".to_string(), pages.join("\n"));
                Some(stack)
            }
            Self::SetInstrument(instrument) => {
                stack
                    .components
                    .insert("minecraft:instrument".to_string(), instrument.clone());
                Some(stack)
            }
            Self::SetLore(lines) => {
                stack
                    .components
                    .insert("minecraft:lore".to_string(), lines.join("\n"));
                Some(stack)
            }
            Self::SetName(name) => {
                stack
                    .components
                    .insert("minecraft:custom_name".to_string(), name.clone());
                Some(stack)
            }
            Self::SetPotion(potion) => {
                stack
                    .components
                    .insert("minecraft:potion_contents".to_string(), potion.clone());
                Some(stack)
            }
            Self::SetStewEffects(effects) => {
                stack.components.insert(
                    "minecraft:suspicious_stew_effects".to_string(),
                    effects.join(","),
                );
                Some(stack)
            }
            Self::SetWrittenBookPages(pages) => {
                stack.components.insert(
                    "minecraft:writable_book_pages".to_string(),
                    pages.join("\n"),
                );
                Some(stack)
            }
            Self::ToggleTooltips(components) => {
                stack
                    .components
                    .insert("minecraft:tooltip_hidden".to_string(), components.join(","));
                Some(stack)
            }
            Self::SetFireworkExplosions(explosions) => {
                stack.components.insert(
                    "minecraft:firework_explosion".to_string(),
                    explosions.join(","),
                );
                Some(stack)
            }
            Self::SetFireworks {
                flight_duration,
                explosions,
            } => {
                stack.components.insert(
                    "minecraft:fireworks".to_string(),
                    format!("{flight_duration}:{}", explosions.join(",")),
                );
                Some(stack)
            }
            Self::Reference(name) => {
                if let Some(functions) = context.function_references.get(name).cloned() {
                    let mut next = Some(stack);
                    for function in functions {
                        next = function.apply(next?, context);
                    }
                    next
                } else {
                    context
                        .warnings
                        .push(format!("Unknown loot function reference {name}"));
                    Some(stack)
                }
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

#[derive(Debug, Clone, PartialEq)]
pub enum LootBonusFormula {
    UniformBonusCount { bonus_multiplier: i32 },
    BinomialWithBonusCount { extra: i32, probability: f32 },
    OreDrops,
}

impl LootBonusFormula {
    fn apply(&self, base_count: i32, context: &mut LootContext) -> i32 {
        let fortune = context.fortune_level.max(0);
        match self {
            Self::UniformBonusCount { bonus_multiplier } => {
                let bound = fortune * (*bonus_multiplier).max(0) + 1;
                base_count + context.random.next_i32(bound)
            }
            Self::BinomialWithBonusCount { extra, probability } => {
                let rolls = fortune + (*extra).max(0);
                let bonus = (0..rolls)
                    .filter(|_| context.random.next_f32() < *probability)
                    .count() as i32;
                base_count + bonus
            }
            Self::OreDrops => {
                if fortune <= 0 {
                    return base_count;
                }
                let multiplier = (context.random.next_i32(fortune + 2) - 1).max(0) + 1;
                base_count * multiplier
            }
        }
    }
}

