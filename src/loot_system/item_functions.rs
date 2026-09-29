//! `LootItemFunction`s that edit enchantments, durability and item components.
//!
//! Each `apply_*` mirrors the `run` method of the Java function of the same name in
//! `net.minecraft.world.level.storage.loot.functions`, including the exact order of
//! `RandomSource` draws (they are part of the observable output).

use std::collections::BTreeMap;

use super::reference_data::item_loot_properties;
use super::stack_components::{
    set_enchantment, BLOCK_STATE_COMPONENT, BOOK, ENCHANTED_BOOK,
};
use super::{
    HolderSet, LootContext, LootEntityTarget, LootParamKey, LootStack, NumberProvider,
};
use crate::enchantment_system::{select_enchantments, EnchantingRandom};

/// `DataComponents.ADDITIONAL_TRADE_COST`.
const ADDITIONAL_TRADE_COST_COMPONENT: &str = "minecraft:additional_trade_cost";
/// `DataComponents.SUSPICIOUS_STEW_EFFECTS`.
const SUSPICIOUS_STEW_EFFECTS_COMPONENT: &str = "minecraft:suspicious_stew_effects";
/// `DataComponents.INSTRUMENT`.
const INSTRUMENT_COMPONENT: &str = "minecraft:instrument";
/// `Items.SUSPICIOUS_STEW`.
const SUSPICIOUS_STEW: &str = "minecraft:suspicious_stew";

/// `SetNameFunction.Target`: which name component `set_name` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameTarget {
    CustomName,
    ItemName,
}

impl NameTarget {
    /// `Target.component`.
    pub fn component(self) -> &'static str {
        match self {
            Self::CustomName => "minecraft:custom_name",
            Self::ItemName => "minecraft:item_name",
        }
    }

    /// `Target.CODEC` (`custom_name` / `item_name`).
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "custom_name" => Some(Self::CustomName),
            "item_name" => Some(Self::ItemName),
            _ => None,
        }
    }
}

/// The `LootContextArg<DataComponentGetter>` `copy_components` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentSource {
    /// An `EntityTarget` (`DirectSource` on the entity).
    Entity(LootEntityTarget),
    /// `BlockEntityTarget.BLOCK_ENTITY` (`BlockEntity.collectComponents`).
    BlockEntity,
    /// `ItemStackTarget.TOOL` (`DirectSource` on the tool stack).
    Tool,
}

impl ComponentSource {
    /// The serialized name of the argument (`this`, `block_entity`, `tool`, ...).
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "block_entity" => Some(Self::BlockEntity),
            "tool" => Some(Self::Tool),
            other => LootEntityTarget::by_name(other).map(Self::Entity),
        }
    }
}

/// `SetStewEffectFunction.EffectEntry`.
#[derive(Debug, Clone, PartialEq)]
pub struct StewEffect {
    /// The `MobEffect` registry id.
    pub effect: String,
    /// The duration in seconds (ticks for instantaneous effects).
    pub duration: NumberProvider,
}

/// One entry of a `DataComponentPatch`: set the component to a value, or remove it
/// (`!minecraft:id` keys).
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentEdit {
    pub component: String,
    /// The canonical text of the new value; `None` removes the component.
    pub value: Option<String>,
}

/// The enchantments an `options` holder set offers: its members, or every registered
/// enchantment (in id order, the registry's iteration order) when absent.
fn enchantment_candidates(options: Option<&HolderSet>, context: &LootContext) -> Vec<String> {
    match options {
        Some(set) => set.members("enchantment", context),
        // `registry.listElements()`: every registered enchantment in registry order.
        None => context.enchantments.ids().map(str::to_string).collect(),
    }
}

/// A `LootRandom` view implementing the draws of `EnchantmentHelper.selectEnchantment`.
struct ContextRandom<'a>(&'a LootContext);

impl EnchantingRandom for ContextRandom<'_> {
    fn next_int(&mut self, bound: i32) -> i32 {
        self.0.random.next_i32(bound)
    }

    fn next_float(&mut self) -> f32 {
        self.0.random.next_f32()
    }
}

/// `Mth.nextInt(random, min, max)`.
pub(super) fn mth_next_int(context: &LootContext, min: i32, max: i32) -> i32 {
    if min >= max {
        min
    } else {
        context.random.next_i32(max - min + 1) + min
    }
}

/// `Util.getRandomSafe(list, random)`.
fn random_element<'a, T>(list: &'a [T], context: &LootContext) -> Option<&'a T> {
    if list.is_empty() {
        None
    } else {
        Some(&list[context.random.next_i32(list.len() as i32) as usize])
    }
}

/// True when the additional-cost context parameter is available
/// (`context.hasParameter(ADDITIONAL_COST_COMPONENT_ALLOWED)`).
fn additional_cost_allowed(context: &LootContext) -> bool {
    context
        .params
        .get(LootParamKey::AdditionalCostComponentAllowed)
        .is_some()
}

/// `SetItemDamageFunction.run`.
pub(super) fn set_damage(
    mut stack: LootStack,
    context: &mut LootContext,
    damage: &NumberProvider,
    add: bool,
) -> LootStack {
    if stack.is_damageable_item() {
        let max_damage = stack.max_damage();
        let base = if add {
            1.0 - stack.damage_value() as f32 / max_damage as f32
        } else {
            0.0
        };
        let pct = 1.0 - (damage.float(context) + base).clamp(0.0, 1.0);
        // `Mth.floor(pct * maxDamage)`.
        stack.set_damage_value((pct * max_damage as f32).floor() as i32);
    } else {
        context
            .warnings
            .push(format!("Couldn't set damage of loot item {}", stack.item));
    }
    stack
}

/// `SetEnchantmentsFunction.run`.
pub(super) fn set_enchantments(
    mut stack: LootStack,
    context: &mut LootContext,
    enchantments: &[(String, NumberProvider)],
    add: bool,
) -> LootStack {
    if stack.item == BOOK {
        stack.item = ENCHANTED_BOOK.to_string();
    }
    stack.update_enchantments(|held| {
        for (id, provider) in enchantments {
            let level = if add {
                held.get(id).copied().unwrap_or(0) + provider.int(context)
            } else {
                provider.int(context)
            };
            set_enchantment(held, id, level.clamp(0, 255));
        }
    });
    stack
}

/// `EnchantmentHelper.enchantItem(random, stack, cost, registryAccess, options)`.
fn enchant_item(
    context: &LootContext,
    stack: LootStack,
    cost: i32,
    options: Option<&HolderSet>,
) -> LootStack {
    let enchantability = item_loot_properties(&stack.item)
        .and_then(|properties| properties.enchantable)
        .unwrap_or(0);
    let candidates = enchantment_candidates(options, context);
    let registry = &context.enchantments;
    let selected = select_enchantments(
        &mut ContextRandom(context),
        cost,
        enchantability,
        |value| registry.available_results(value, &stack.item, &candidates, context),
        |first, second| registry.are_compatible(first, second, context),
    );
    let mut stack = if stack.item == BOOK {
        LootStack::new(ENCHANTED_BOOK, 1)
    } else {
        stack
    };
    for (id, level) in selected {
        stack.enchant(&id, level);
    }
    stack
}

/// `EnchantWithLevelsFunction.run`.
pub(super) fn enchant_with_levels(
    stack: LootStack,
    context: &mut LootContext,
    levels: &NumberProvider,
    options: Option<&HolderSet>,
    include_additional_cost_component: bool,
) -> LootStack {
    let cost = levels.int(context);
    let mut result = enchant_item(context, stack, cost, options);
    if include_additional_cost_component
        && additional_cost_allowed(context)
        && !result.is_empty()
        && cost > 0
    {
        result.components.insert(
            ADDITIONAL_TRADE_COST_COMPONENT.to_string(),
            cost.to_string(),
        );
    }
    result
}

/// `EnchantRandomlyFunction.run`.
pub(super) fn enchant_randomly(
    stack: LootStack,
    context: &mut LootContext,
    options: Option<&HolderSet>,
    only_compatible: bool,
    include_additional_cost_component: bool,
) -> LootStack {
    let target_is_book = stack.item == BOOK;
    let check_compatibility = !target_is_book && only_compatible;
    let compatible: Vec<String> = enchantment_candidates(options, context)
        .into_iter()
        .filter(|candidate| {
            // `Enchantment.canEnchant`; an unregistered id cannot be evaluated.
            context.enchantments.get(candidate).is_some_and(|definition| {
                !check_compatibility || definition.is_supported_item(&stack.item, context)
            })
        })
        .collect();
    let Some(chosen) = random_element(&compatible, context).cloned() else {
        context.warnings.push(format!(
            "Couldn't find a compatible enchantment for {}",
            stack.item
        ));
        return stack;
    };
    let max_level = context
        .enchantments
        .get(&chosen)
        .map_or(1, |definition| definition.max_level);
    let level = mth_next_int(context, 1, max_level);
    let mut stack = if target_is_book {
        LootStack::new(ENCHANTED_BOOK, 1)
    } else {
        stack
    };
    stack.enchant(&chosen, level);
    if include_additional_cost_component && additional_cost_allowed(context) {
        let cost = 2 + context.random.next_i32(5 + level * 10) + 3 * level;
        stack
            .components
            .insert(ADDITIONAL_TRADE_COST_COMPONENT.to_string(), cost.to_string());
    }
    stack
}

/// `SetStewEffectFunction.run`.
pub(super) fn set_stew_effect(
    mut stack: LootStack,
    context: &mut LootContext,
    effects: &[StewEffect],
) -> LootStack {
    if stack.item != SUSPICIOUS_STEW || effects.is_empty() {
        return stack;
    }
    let Some(entry) = random_element(effects, context) else {
        return stack;
    };
    let mut duration = entry.duration.int(context);
    let instantaneous = crate::status_effect::status_effect(&entry.effect)
        .is_some_and(|definition| definition.instantaneous);
    if !instantaneous {
        duration *= 20;
    }
    // `SuspiciousStewEffects.withEffectAdded`: append the entry.
    let mut effects = stack
        .components
        .get(SUSPICIOUS_STEW_EFFECTS_COMPONENT)
        .cloned()
        .filter(|text| !text.is_empty())
        .into_iter()
        .collect::<Vec<_>>();
    effects.push(format!("{}:{duration}", entry.effect));
    stack.components.insert(
        SUSPICIOUS_STEW_EFFECTS_COMPONENT.to_string(),
        effects.join(","),
    );
    stack
}

/// `SetInstrumentFunction.run`.
pub(super) fn set_instrument(
    mut stack: LootStack,
    context: &LootContext,
    options: &HolderSet,
) -> LootStack {
    let members = options.members("instrument", context);
    if let Some(instrument) = random_element(&members, context) {
        stack
            .components
            .insert(INSTRUMENT_COMPONENT.to_string(), instrument.clone());
    }
    stack
}

/// `SmeltItemFunction.run`: replaces the stack with the result of the first smelting
/// recipe its item satisfies.
pub(super) fn smelt_item(
    stack: LootStack,
    context: &mut LootContext,
    use_input_count: bool,
) -> LootStack {
    if stack.is_empty() {
        return stack;
    }
    let smelted = context
        .recipes
        .as_ref()
        .zip(super::reference_data::vanilla_item_id(&stack.item))
        .and_then(|(recipes, input)| {
            recipes
                .recipe_map()
                .get_recipe_for("smelting", 1, 1, &[Some(input)])
        })
        .and_then(|holder| match &holder.recipe {
            crate::recipe_system::RecipeKind::Cooking { result, .. } => Some(result.clone()),
            _ => None,
        });
    match smelted {
        Some(result) if result.count > 0 => {
            let result_stack = LootStack::new(result.item, result.count as i32);
            let new_count =
                (if use_input_count { stack.count } else { 1 }) * result_stack.count;
            let mut smelted = result_stack;
            smelted.count = new_count.min(smelted.max_stack_size());
            smelted
        }
        _ => {
            context.warnings.push(format!(
                "Couldn't smelt {} because there is no smelting recipe",
                stack.item
            ));
            stack
        }
    }
}

/// `CopyBlockState.run`: copies the listed block-state properties into the
/// `block_state` component when the context has a block state.
pub(super) fn copy_block_state(
    mut stack: LootStack,
    context: &LootContext,
    properties: &[String],
) -> LootStack {
    if context.params.get(LootParamKey::BlockState).is_none() {
        return stack;
    }
    let mut copied: BTreeMap<String, String> = stack
        .components
        .get(BLOCK_STATE_COMPONENT)
        .map(|text| {
            text.split(',')
                .filter_map(|entry| entry.split_once('='))
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect()
        })
        .unwrap_or_default();
    for property in properties {
        if let Some(value) = context.block_state_properties.get(property) {
            copied.insert(property.clone(), value.clone());
        }
    }
    stack.components.insert(
        BLOCK_STATE_COMPONENT.to_string(),
        copied
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join(","),
    );
    stack
}

/// `CopyComponentsFunction.run`.
pub(super) fn copy_components(
    mut stack: LootStack,
    context: &LootContext,
    source: ComponentSource,
    include: Option<&[String]>,
    exclude: Option<&[String]>,
) -> LootStack {
    let components: Option<Vec<(String, String)>> = match source {
        ComponentSource::Entity(target) => context.entities.get(&target).map(|entity| {
            entity
                .components
                .iter()
                .map(|(component, value)| (component.clone(), value.clone()))
                .collect()
        }),
        ComponentSource::BlockEntity => context
            .params
            .get(LootParamKey::BlockEntity)
            .map(|_| clone_pairs(&context.block_entity_components)),
        ComponentSource::Tool => context
            .params
            .get(LootParamKey::Tool)
            .map(|_| clone_pairs(&context.tool_components)),
    };
    for (component, value) in components.into_iter().flatten() {
        let included = include.is_none_or(|list| list.contains(&component));
        let excluded = exclude.is_some_and(|list| list.contains(&component));
        if included && !excluded {
            stack.components.insert(component, value);
        }
    }
    stack
}

fn clone_pairs(map: &BTreeMap<String, String>) -> Vec<(String, String)> {
    map.iter()
        .map(|(component, value)| (component.clone(), value.clone()))
        .collect()
}
