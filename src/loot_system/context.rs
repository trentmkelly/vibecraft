#![allow(dead_code)]

use std::collections::HashMap;

use super::*;

pub struct LootContext {
    pub param_set: LootParamSet,
    pub entity_type: LootContextEntityType,
    pub params: LootParams,
    pub dynamic_params: HashMap<LootDynamicParamKey, LootDynamicParamValue>,
    pub random: DeterministicRandom,
    pub luck: f32,
    pub looting_level: i32,
    pub fortune_level: i32,
    pub enchantment_level: i32,
    pub enchantment_active: bool,
    pub killed_by_player: bool,
    pub explosion_radius: Option<f32>,
    pub game_time: i64,
    pub weather_raining: bool,
    pub weather_thundering: bool,
    pub block_on_fire: bool,
    pub block: Option<String>,
    pub tool: Option<String>,
    pub scores: HashMap<String, i32>,
    pub storage_numbers: HashMap<String, f32>,
    pub environment_attributes: HashMap<String, f32>,
    pub context_nbt: HashMap<String, HashMap<String, String>>,
    pub storage_nbt: HashMap<String, HashMap<String, String>>,
    pub entity_properties: HashMap<String, String>,
    pub block_state_properties: HashMap<String, String>,
    pub smelting_results: HashMap<String, String>,
    pub condition_references: HashSet<String>,
    pub function_references: HashMap<String, Vec<LootFunction>>,
    pub tables: HashMap<String, LootTable>,
    pub tags: HashMap<String, Vec<String>>,
    pub dynamic_drops: HashMap<String, Vec<LootStack>>,
    pub warnings: Vec<String>,
    pub(super) max_stack_size: i32,
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
            fortune_level: 0,
            enchantment_level: 0,
            enchantment_active: false,
            killed_by_player: false,
            explosion_radius: None,
            game_time: 0,
            weather_raining: false,
            weather_thundering: false,
            block_on_fire: false,
            block: None,
            tool: None,
            scores: HashMap::new(),
            storage_numbers: HashMap::new(),
            environment_attributes: HashMap::new(),
            context_nbt: HashMap::new(),
            storage_nbt: HashMap::new(),
            entity_properties: HashMap::new(),
            block_state_properties: HashMap::new(),
            smelting_results: HashMap::new(),
            condition_references: HashSet::new(),
            function_references: HashMap::new(),
            tables: HashMap::new(),
            tags: HashMap::new(),
            dynamic_drops: HashMap::new(),
            warnings: Vec::new(),
            max_stack_size: 64,
            visited: Vec::new(),
            base_seed: seed,
        }
    }

    pub(super) fn clone_for_condition(&self) -> Self {
        Self {
            param_set: self.param_set,
            entity_type: self.entity_type,
            params: self.params.clone(),
            dynamic_params: self.dynamic_params.clone(),
            random: self.random,
            luck: self.luck,
            looting_level: self.looting_level,
            fortune_level: self.fortune_level,
            enchantment_level: self.enchantment_level,
            enchantment_active: self.enchantment_active,
            killed_by_player: self.killed_by_player,
            explosion_radius: self.explosion_radius,
            game_time: self.game_time,
            weather_raining: self.weather_raining,
            weather_thundering: self.weather_thundering,
            block_on_fire: self.block_on_fire,
            block: self.block.clone(),
            tool: self.tool.clone(),
            scores: self.scores.clone(),
            storage_numbers: self.storage_numbers.clone(),
            environment_attributes: self.environment_attributes.clone(),
            context_nbt: self.context_nbt.clone(),
            storage_nbt: self.storage_nbt.clone(),
            entity_properties: self.entity_properties.clone(),
            block_state_properties: self.block_state_properties.clone(),
            smelting_results: self.smelting_results.clone(),
            condition_references: self.condition_references.clone(),
            function_references: self.function_references.clone(),
            tables: HashMap::new(),
            tags: self.tags.clone(),
            dynamic_drops: self.dynamic_drops.clone(),
            warnings: Vec::new(),
            max_stack_size: self.max_stack_size,
            visited: self.visited.clone(),
            base_seed: self.base_seed,
        }
    }

    pub(super) fn enter(&mut self, key: String) -> bool {
        if self.visited.contains(&key) {
            return false;
        }
        self.visited.push(key);
        true
    }

    pub(super) fn exit(&mut self) {
        self.visited.pop();
    }

    pub(super) fn use_random_sequence(&mut self, sequence: &str) {
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

pub(super) fn split_stacks(stacks: Vec<LootStack>, max_stack_size: i32) -> Vec<LootStack> {
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

pub(super) fn shuffle<T>(values: &mut [T], random: &mut DeterministicRandom) {
    for index in (1..values.len()).rev() {
        let other = random.next_i32(index as i32 + 1) as usize;
        values.swap(index, other);
    }
}

pub(super) fn hash_seed(base: u64, text: &str) -> u64 {
    let mut hash = base ^ 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
