#![allow(dead_code)]
use super::*;
use std::collections::HashMap;

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
            LootSurface::Command => match &request.actor {
                Some(actor) => {
                    LootDelivery::GiveToEntity(actor.clone(), table.evaluate(&mut context))
                }
                None => LootDelivery::DropAt(request.origin, table.evaluate(&mut context)),
            },
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
            context.insert_param(LootParamValue::AttackingEntity(killer.clone()));
        }
        if let Some(direct_killer) = &request.direct_killer_entity {
            context.insert_param(LootParamValue::DirectAttackingEntity(direct_killer.clone()));
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
            // `LootPool.addRandomItem`: only entries of positive weight are valid.
            candidates.retain(|entry| entry.effective_weight(context.luck) > 0);
            let total_weight: i32 = candidates
                .iter()
                .map(|entry| entry.effective_weight(context.luck))
                .sum();
            if total_weight <= 0 {
                continue;
            }
            if candidates.len() == 1 {
                // A lone valid entry is chosen without drawing a random number.
                result.extend(candidates[0].create(context));
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
        functions: Vec<LootFunction>,
    },
    Tag {
        tag: String,
        expand: bool,
        weight: i32,
        quality: i32,
        conditions: Vec<LootCondition>,
        functions: Vec<LootFunction>,
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
        functions: Vec<LootFunction>,
    },
    /// `NestedLootTable` with an inline (`DIRECT_CODEC`) table.
    InlineTable {
        table: Box<LootTable>,
        weight: i32,
        quality: i32,
        conditions: Vec<LootCondition>,
        functions: Vec<LootFunction>,
    },
    /// `DynamicLoot`: drops the context's dynamic drop named `name`.
    Dynamic {
        name: String,
        weight: i32,
        quality: i32,
        conditions: Vec<LootCondition>,
        functions: Vec<LootFunction>,
    },
    /// `CompositeEntryBase.conditions`: `entry` (an alternatives/sequence/group
    /// container) only expands when every condition holds.
    Conditional {
        conditions: Vec<LootCondition>,
        entry: Box<LootEntry>,
    },
    /// A `LootPoolEntries` type (`loot_table` with an inline table, `slots`) the
    /// runtime cannot evaluate yet. The JSON codec validates and keeps the document,
    /// but the entry never produces drops and [`LootTable::is_fully_modeled`] reports
    /// the table as unusable.
    /// TODO(loot-json-codec): model these entry types.
    Unmodeled {
        kind: String,
        data: serde_json::Value,
    },
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
            }
            | Self::Empty {
                weight,
                quality,
                conditions,
                ..
            }
            | Self::WeightedNestedTable {
                weight,
                quality,
                conditions,
                ..
            }
            | Self::InlineTable {
                weight,
                quality,
                conditions,
                ..
            }
            | Self::Dynamic {
                weight,
                quality,
                conditions,
                ..
            } => {
                if conditions_hold(conditions, context) {
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
                functions,
            } => {
                if !conditions_hold(conditions, context) {
                    return;
                }
                if !*expand {
                    output.push(ExpandedEntry {
                        entry: self.clone(),
                        weight: *weight,
                        quality: *quality,
                    });
                    return;
                }
                for item in context.tags.get(tag).into_iter().flatten() {
                    output.push(ExpandedEntry {
                        entry: LootEntry::Item {
                            item: item.clone(),
                            weight: *weight,
                            quality: *quality,
                            conditions: Vec::new(),
                            functions: functions.clone(),
                        },
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
            Self::NestedTable(_) => output.push(ExpandedEntry {
                entry: self.clone(),
                weight: 1,
                quality: 0,
            }),
            Self::Conditional { conditions, entry } => {
                if conditions_hold(conditions, context) {
                    entry.expand(context, output);
                }
            }
            Self::Unmodeled { .. } => {}
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
            Self::Empty { .. } | Self::Unmodeled { .. } => Vec::new(),
            Self::Tag {
                tag,
                expand,
                functions,
                ..
            } => {
                let stacks = context
                    .tags
                    .get(tag)
                    .map(|items| {
                        if *expand {
                            items.iter().map(|item| LootStack::new(item, 1)).collect()
                        } else {
                            vec![LootStack::new(format!("#{tag}"), 1)]
                        }
                    })
                    .unwrap_or_default();
                apply_entry_functions(functions, stacks, context)
            }
            Self::NestedTable(table) => create_nested_table(table, &[], context),
            Self::WeightedNestedTable {
                table, functions, ..
            } => create_nested_table(table, functions, context),
            Self::InlineTable {
                table, functions, ..
            } => {
                let stacks = table.evaluate(context);
                apply_entry_functions(functions, stacks, context)
            }
            Self::Dynamic {
                name, functions, ..
            } => {
                let stacks = context.dynamic_drops.get(name).cloned().unwrap_or_default();
                apply_entry_functions(functions, stacks, context)
            }
            Self::Conditional { entry, .. } => entry.create(context),
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
            Self::Group(children) => {
                let mut result = Vec::new();
                for child in children {
                    let mut expanded = Vec::new();
                    child.expand(context, &mut expanded);
                    result.extend(
                        expanded
                            .into_iter()
                            .flat_map(|entry| entry.entry.create(context)),
                    );
                }
                result
            }
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
            | Self::Dynamic { name: table, .. }
                if table.is_empty() =>
            {
                errors.push(format!("{path} has an empty reference"));
            }
            Self::Conditional { entry, .. } => entry.validate(path, errors),
            Self::WeightedNestedTable { weight, .. } if *weight < 0 => {
                errors.push(format!("{path} has invalid nested-table weight"));
            }
            _ => {}
        }
    }
}

/// True when every condition matches (`LootItemConditions.andConditions`).
fn conditions_hold(conditions: &[LootCondition], context: &LootContext) -> bool {
    conditions
        .iter()
        .all(|condition| condition.matches(context))
}

/// `LootPoolSingletonContainer.functions`: runs the entry's item modifiers over the
/// stacks it produced; a modifier returning nothing drops the stack.
fn apply_entry_functions(
    functions: &[LootFunction],
    stacks: Vec<LootStack>,
    context: &mut LootContext,
) -> Vec<LootStack> {
    stacks
        .into_iter()
        .filter_map(|stack| {
            functions
                .iter()
                .try_fold(stack, |stack, function| function.apply(stack, context))
        })
        .collect()
}

/// `NestedLootTable.createItemStack`: rolls the referenced table, then applies the
/// entry's own functions to its output.
fn create_nested_table(
    table: &str,
    functions: &[LootFunction],
    context: &mut LootContext,
) -> Vec<LootStack> {
    let Some(nested) = context.tables.get(table).cloned() else {
        context
            .warnings
            .push(format!("Unknown nested loot table {table}"));
        return Vec::new();
    };
    let stacks = nested.evaluate(context);
    apply_entry_functions(functions, stacks, context)
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
    Uniform {
        min: f32,
        max: f32,
    },
    Binomial {
        n: i32,
        p: f32,
    },
    UniformProvider {
        min: Box<NumberProvider>,
        max: Box<NumberProvider>,
    },
    BinomialProvider {
        n: Box<NumberProvider>,
        p: Box<NumberProvider>,
    },
    Score {
        name: String,
        scale: f32,
    },
    Storage {
        key: String,
        scale: f32,
    },
    EnchantmentLevel {
        scale: f32,
    },
    Sum(Vec<NumberProvider>),
    EnvironmentAttribute {
        attribute: String,
    },
    /// A `NumberProviders` type the runtime cannot evaluate yet (`score`, `storage`
    /// and `enchantment_level` with their full Java parameters). The codec validated
    /// the document; it evaluates to zero and marks the table as not fully modeled.
    /// TODO(loot-json-codec): model these providers.
    Unmodeled {
        kind: String,
        data: serde_json::Value,
    },
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
        match self {
            Self::Uniform { min, max } => {
                let min = (*min).round() as i32;
                let max = (*max).round() as i32;
                min + context
                    .random
                    .next_i32(max.saturating_sub(min).saturating_add(1))
            }
            Self::UniformProvider { min, max } => {
                let min = min.int(context);
                let max = max.int(context);
                item_functions::mth_next_int(context, min, max)
            }
            Self::Binomial { n, p } => binomial_roll(context, *n, *p),
            Self::BinomialProvider { n, p } => {
                let n = n.int(context);
                let p = p.float(context);
                binomial_roll(context, n, p)
            }
            Self::Storage { key, .. } => {
                context.storage_numbers.get(key).copied().unwrap_or(0.0) as i32
            }
            Self::Sum(summands) => summands
                .iter()
                .map(|provider| provider.float(context))
                .sum::<f32>()
                .floor() as i32,
            _ => self.float(context).round() as i32,
        }
    }

    pub fn float(&self, context: &mut LootContext) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Uniform { min, max } => *min + (*max - *min) * context.random.next_f32(),
            Self::Binomial { n, p } => binomial_roll(context, *n, *p) as f32,
            Self::UniformProvider { min, max } => {
                // `Mth.nextFloat`: no draw when the range is empty.
                let min = min.float(context);
                let max = max.float(context);
                if min >= max {
                    min
                } else {
                    context.random.next_f32() * (max - min) + min
                }
            }
            Self::BinomialProvider { n, p } => {
                let n = n.int(context);
                let p = p.float(context);
                binomial_roll(context, n, p) as f32
            }
            Self::Score { name, scale } => {
                context.scores.get(name).copied().unwrap_or(0) as f32 * *scale
            }
            Self::Storage { key, scale } => {
                context.storage_numbers.get(key).copied().unwrap_or(0.0) * *scale
            }
            Self::EnchantmentLevel { scale } => context.enchantment_level as f32 * *scale,
            Self::Sum(summands) => summands
                .iter()
                .map(|provider| provider.float(context))
                .sum(),
            Self::EnvironmentAttribute { attribute } => context
                .environment_attributes
                .get(attribute)
                .copied()
                .unwrap_or(0.0),
            Self::Unmodeled { .. } => 0.0,
        }
    }

    fn minimum(&self) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Uniform { min, .. } => *min,
            Self::UniformProvider { min, .. } => min.minimum(),
            Self::Binomial { .. } | Self::BinomialProvider { .. } => 0.0,
            Self::Score { .. } | Self::Storage { .. } => f32::MIN,
            Self::EnchantmentLevel { .. } => 0.0,
            Self::Sum(summands) => summands.iter().map(NumberProvider::minimum).sum(),
            Self::EnvironmentAttribute { .. } => f32::MIN,
            Self::Unmodeled { .. } => 0.0,
        }
    }
}

fn binomial_roll(context: &mut LootContext, n: i32, p: f32) -> i32 {
    let mut successes = 0;
    for _ in 0..n.max(0) {
        if context.random.next_f32() < p {
            successes += 1;
        }
    }
    successes
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootCondition {
    RandomChance(f32),
    RandomChanceWithLooting {
        chance: f32,
        looting_multiplier: f32,
    },
    /// `LootItemRandomChanceWithEnchantedBonusCondition`: the attacker's level of
    /// `enchantment` picks `enchanted_chance` (or `unenchanted_chance` at level 0).
    RandomChanceWithEnchantedBonus {
        unenchanted_chance: f32,
        enchanted_chance: LevelBasedValue,
        enchantment: String,
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
        period: Option<i64>,
    },
    ValueCheck {
        provider: NumberProvider,
        min: f32,
        max: f32,
    },
    EnchantmentActiveCheck {
        active: bool,
    },
    EnvironmentAttributeCheck {
        attribute: String,
        value: f32,
    },
    /// `BonusLevelTableCondition`: `chances[min(level, len - 1)]` for the tool's level
    /// of `enchantment`.
    TableBonus {
        enchantment: String,
        chances: Vec<f32>,
    },
    /// `MatchTool` with an item/enchantment `ItemPredicate`.
    ToolMatches(ToolPredicate),
    /// `LootItemEntityPropertyCondition`.
    EntityHasProperties {
        target: LootEntityTarget,
        predicate: Option<EntityPredicate>,
    },
    /// `DamageSourceCondition`.
    DamageSourceMatches(Option<DamageSourcePredicate>),
    /// `LocationCheck`: the predicate at the origin plus `offset`.
    LocationMatches {
        predicate: Option<LocationPredicate>,
        offset: (i32, i32, i32),
    },
    Reference(String),
    Inverted(Box<LootCondition>),
    AllOf(Vec<LootCondition>),
    AnyOf(Vec<LootCondition>),
    /// A `LootItemConditions` type (or parameterisation) the runtime cannot
    /// evaluate yet: entity/location/item/damage-source predicates, enchantment
    /// based chances, scores and clocks. The codec validated the document; the
    /// condition never matches and marks the table as not fully modeled.
    /// TODO(loot-json-codec): model these conditions.
    Unmodeled {
        kind: String,
        data: serde_json::Value,
    },
}

impl LootCondition {
    pub fn matches(&self, context: &LootContext) -> bool {
        let mut fork = context.clone_for_condition();
        self.matches_mut(&mut fork)
    }

    /// The conditions that embed advancement predicates (`entity_properties`,
    /// `damage_source_properties`, `location_check`).
    fn predicate_matches(&self, context: &LootContext) -> bool {
        match self {
            Self::EntityHasProperties { target, predicate } => predicate
                .as_ref()
                .is_none_or(|predicate| predicate.matches(context.entities.get(target), context)),
            Self::DamageSourceMatches(predicate) => {
                match (&context.damage_source, context.origin()) {
                    (Some(source), Some(_)) => predicate
                        .as_ref()
                        .is_none_or(|predicate| predicate.matches(source, context)),
                    _ => false,
                }
            }
            Self::LocationMatches { predicate, offset } => match context.origin() {
                Some((x, y, z)) => predicate.as_ref().is_none_or(|predicate| {
                    predicate.matches(
                        context,
                        x + f64::from(offset.0),
                        y + f64::from(offset.1),
                        z + f64::from(offset.2),
                    )
                }),
                None => false,
            },
            _ => unreachable!("non-predicate condition routed to predicate handler"),
        }
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
                enchantment,
            } => {
                let level = context.enchantment_level_of(enchantment);
                let chance = if level > 0 {
                    enchanted_chance.calculate(level)
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
            Self::EntityProperty { key, value } => context.entity_properties.get(key) == Some(value),
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
                raining.is_none_or(|expected| context.weather_raining == expected)
                    && thundering.is_none_or(|expected| context.weather_thundering == expected)
            }
            Self::TimeCheck { min, max, period } => {
                let time = period
                    .filter(|period| *period > 0)
                    .map_or(context.game_time, |period| context.game_time.rem_euclid(period));
                time >= *min && time <= *max
            }
            Self::ValueCheck { provider, min, max } => {
                let value = provider.float(context);
                value >= *min && value <= *max
            }
            Self::EnchantmentActiveCheck { active } => context.enchantment_active == *active,
            Self::EnvironmentAttributeCheck { attribute, value } => context
                .environment_attributes
                .get(attribute)
                .is_some_and(|actual| (*actual - *value).abs() <= f32::EPSILON),
            Self::ToolMatches(predicate) => predicate.matches(context),
            Self::TableBonus {
                enchantment,
                chances,
            } => {
                let index = context.enchantment_level_of(enchantment).max(0) as usize;
                let chance = chances
                    .get(index)
                    .or_else(|| chances.last())
                    .copied()
                    .unwrap_or(0.0);
                context.random.next_f32() < chance
            }
            Self::EntityHasProperties { .. }
            | Self::DamageSourceMatches(_)
            | Self::LocationMatches { .. } => self.predicate_matches(context),
            Self::Reference(name) => context.condition_references.contains(name),
            Self::Inverted(condition) => !condition.matches_mut(context),
            Self::AllOf(conditions) => conditions
                .iter()
                .all(|condition| condition.matches_mut(context)),
            Self::AnyOf(conditions) => conditions
                .iter()
                .any(|condition| condition.matches_mut(context)),
            Self::Unmodeled { .. } => false,
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
    /// `ApplyBonusCount`: `formula` over the tool's level of `enchantment`.
    ApplyBonus {
        enchantment: String,
        formula: LootBonusFormula,
    },
    /// `SetItemCountFunction` with `add: true`.
    AddCount(NumberProvider),
    /// `EnchantedCountIncreaseFunction`: grows the count by
    /// `round(level * count)` for the attacker's level of `enchantment`, capped at
    /// `limit` when it is positive.
    EnchantedCountIncrease {
        enchantment: String,
        count: NumberProvider,
        limit: i32,
    },
    SetItem(String),
    /// `SetEnchantmentsFunction`: level providers in declaration order.
    SetEnchantments {
        enchantments: Vec<(String, NumberProvider)>,
        add: bool,
    },
    /// `SetItemDamageFunction`.
    SetDamage {
        damage: NumberProvider,
        add: bool,
    },
    SetNbt(HashMap<String, String>),
    /// `EnchantWithLevelsFunction`; `options` absent means every enchantment.
    EnchantWithLevels {
        levels: NumberProvider,
        options: Option<HolderSet>,
        include_additional_cost_component: bool,
    },
    /// `EnchantRandomlyFunction`; `options` absent means every enchantment.
    EnchantRandomly {
        options: Option<HolderSet>,
        only_compatible: bool,
        include_additional_cost_component: bool,
    },
    /// `SmeltItemFunction`.
    SmeltItem {
        use_input_count: bool,
    },
    /// `CopyComponentsFunction`.
    CopyComponents {
        source: ComponentSource,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    },
    CopyName {
        source: String,
    },
    CopyNbt {
        provider: NbtProvider,
        target: String,
    },
    SetContents(Vec<LootStack>),
    ModifyContents(Vec<LootFunction>),
    ExplorationMap {
        destination: String,
        decoration: String,
    },
    FillPlayerHead,
    /// `CopyBlockState`: `properties` are those of `block` that exist on it.
    CopyState {
        block: String,
        properties: Vec<String>,
    },
    SetAttributes(Vec<String>),
    SetBannerPatterns(Vec<String>),
    SetBookContents {
        title: String,
        author: String,
        pages: Vec<String>,
    },
    /// `SetComponentsFunction`: the `DataComponentPatch` edits.
    SetComponents(Vec<ComponentEdit>),
    /// `SetInstrumentFunction`.
    SetInstrument(HolderSet),
    SetLore(Vec<String>),
    /// `SetNameFunction`: `name` is the compact JSON of the text component.
    SetName {
        name: Option<String>,
        target: NameTarget,
    },
    SetPotion(String),
    SetRandomPotion(Vec<String>),
    /// `SetStewEffectFunction`.
    SetStewEffects(Vec<StewEffect>),
    SetRandomDyes(Vec<String>),
    SetWrittenBookPages(Vec<String>),
    SetWritableBookPages(Vec<String>),
    SetBookCover {
        title: String,
        author: String,
    },
    ToggleTooltips(Vec<String>),
    SetFireworkExplosions(Vec<String>),
    SetFireworks {
        flight_duration: i32,
        explosions: Vec<String>,
    },
    SetOminousBottleAmplifier(NumberProvider),
    SetCustomModelData(String),
    SetLootTable(String),
    Reference(String),
    Discard,
    ApplyExplosionDecay,
    Filtered {
        condition: LootCondition,
        function: Box<LootFunction>,
    },
    Sequence(Vec<LootFunction>),
    /// A `LootItemFunctions` type (or parameterisation) the runtime cannot apply
    /// yet. The codec validated the document; the function leaves the stack
    /// untouched and marks the table as not fully modeled.
    /// TODO(loot-json-codec): model these functions.
    Unmodeled {
        kind: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootBonusFormula {
    UniformBonusCount { bonus_multiplier: i32 },
    BinomialWithBonusCount { extra: i32, probability: f32 },
    OreDrops,
}

impl LootBonusFormula {
    pub(super) fn apply(&self, base_count: i32, level: i32, context: &mut LootContext) -> i32 {
        let fortune = level.max(0);
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
