use super::*;

impl LootFunction {
    pub fn apply(&self, stack: LootStack, context: &mut LootContext) -> Option<LootStack> {
        match self {
            Self::SetCount(_)
            | Self::LimitCount { .. }
            | Self::AddLootingBonus { .. }
            | Self::ApplyFortuneBonus { .. }
            | Self::ApplyBonus(_) => self.apply_count_function(stack, context),
            Self::SetItem(_)
            | Self::SetDamage(_)
            | Self::SetNbt(_)
            | Self::SetComponents(_)
            | Self::EnchantWithLevels { .. }
            | Self::EnchantRandomly(_)
            | Self::SmeltItem => self.apply_item_function(stack, context),
            Self::CopyName { .. }
            | Self::CopyNbt { .. }
            | Self::SetContents(_)
            | Self::ExplorationMap { .. }
            | Self::FillPlayerHead
            | Self::CopyState(_)
            | Self::SetAttributes(_) => self.apply_context_component_function(stack, context),
            Self::SetBannerPatterns(_)
            | Self::SetBookContents { .. }
            | Self::SetInstrument(_)
            | Self::SetLore(_)
            | Self::SetName(_)
            | Self::SetPotion(_)
            | Self::SetStewEffects(_)
            | Self::SetWrittenBookPages(_)
            | Self::ToggleTooltips(_)
            | Self::SetFireworkExplosions(_)
            | Self::SetFireworks { .. } => self.apply_text_component_function(stack),
            Self::Reference(_)
            | Self::ApplyExplosionDecay
            | Self::Filtered { .. }
            | Self::Sequence(_) => self.apply_control_function(stack, context),
        }
    }

    fn apply_count_function(
        &self,
        mut stack: LootStack,
        context: &mut LootContext,
    ) -> Option<LootStack> {
        match self {
            Self::SetCount(provider) => stack.count = provider.int(context),
            Self::LimitCount { min, max } => stack.count = stack.count.clamp(*min, *max),
            Self::AddLootingBonus { per_level, limit } => {
                stack.count += per_level.int(context) * context.looting_level.max(0);
                if let Some(limit) = limit {
                    stack.count = stack.count.min(*limit);
                }
            }
            Self::ApplyFortuneBonus { per_level, limit } => {
                stack.count += per_level.int(context) * context.fortune_level.max(0);
                if let Some(limit) = limit {
                    stack.count = stack.count.min(*limit);
                }
            }
            Self::ApplyBonus(formula) => stack.count = formula.apply(stack.count, context),
            _ => unreachable!("non-count loot function routed to count handler"),
        }
        Some(stack)
    }

    fn apply_item_function(
        &self,
        mut stack: LootStack,
        context: &mut LootContext,
    ) -> Option<LootStack> {
        match self {
            Self::SetItem(item) => stack.item.clone_from(item),
            Self::SetDamage(provider) => {
                stack.components.insert(
                    "minecraft:damage_fraction".to_string(),
                    provider.float(context).to_string(),
                );
            }
            Self::SetNbt(values) | Self::SetComponents(values) => {
                stack.components.extend(values.clone());
            }
            Self::EnchantWithLevels { levels, options } => {
                let level = levels.int(context).max(1);
                apply_enchantment_component(&mut stack, context, level, options);
            }
            Self::EnchantRandomly(options) => {
                apply_enchantment_component(&mut stack, context, 1, options);
            }
            Self::SmeltItem => {
                if context.block_on_fire {
                    if let Some(result) = context.smelting_results.get(&stack.item) {
                        stack.item.clone_from(result);
                    }
                }
            }
            _ => unreachable!("non-item loot function routed to item handler"),
        }
        Some(stack)
    }

    fn apply_context_component_function(
        &self,
        mut stack: LootStack,
        context: &mut LootContext,
    ) -> Option<LootStack> {
        match self {
            Self::CopyName { source } => copy_entity_name_component(&mut stack, context, source),
            Self::CopyNbt { provider, target } => {
                if let Some(value) = provider.get(context) {
                    stack.components.insert(target.clone(), value.to_string());
                }
            }
            Self::SetContents(contents) => set_container_component(&mut stack, contents),
            Self::ExplorationMap {
                destination,
                decoration,
            } => set_exploration_map_components(&mut stack, destination, decoration),
            Self::FillPlayerHead => fill_player_head_component(&mut stack, context),
            Self::CopyState(properties) => {
                copy_block_state_components(&mut stack, context, properties);
            }
            Self::SetAttributes(attributes) => {
                stack.components.insert(
                    "minecraft:attribute_modifiers".to_string(),
                    attributes.join(","),
                );
            }
            _ => unreachable!("non-context loot function routed to context component handler"),
        }
        Some(stack)
    }

    fn apply_text_component_function(&self, mut stack: LootStack) -> Option<LootStack> {
        match self {
            Self::SetBannerPatterns(patterns) => {
                insert_joined_component(&mut stack, "minecraft:banner_patterns", patterns, ",");
            }
            Self::SetBookContents {
                title,
                author,
                pages,
            } => set_written_book_components(&mut stack, title, author, pages),
            Self::SetInstrument(instrument) => {
                stack
                    .components
                    .insert("minecraft:instrument".to_string(), instrument.clone());
            }
            Self::SetLore(lines) => {
                insert_joined_component(&mut stack, "minecraft:lore", lines, "\n");
            }
            Self::SetName(name) => {
                stack
                    .components
                    .insert("minecraft:custom_name".to_string(), name.clone());
            }
            Self::SetPotion(potion) => {
                stack
                    .components
                    .insert("minecraft:potion_contents".to_string(), potion.clone());
            }
            Self::SetStewEffects(effects) => {
                insert_joined_component(
                    &mut stack,
                    "minecraft:suspicious_stew_effects",
                    effects,
                    ",",
                );
            }
            Self::SetWrittenBookPages(pages) => {
                insert_joined_component(&mut stack, "minecraft:writable_book_pages", pages, "\n");
            }
            Self::ToggleTooltips(components) => {
                insert_joined_component(&mut stack, "minecraft:tooltip_hidden", components, ",");
            }
            Self::SetFireworkExplosions(explosions) => {
                insert_joined_component(
                    &mut stack,
                    "minecraft:firework_explosion",
                    explosions,
                    ",",
                );
            }
            Self::SetFireworks {
                flight_duration,
                explosions,
            } => {
                stack.components.insert(
                    "minecraft:fireworks".to_string(),
                    format!("{flight_duration}:{}", explosions.join(",")),
                );
            }
            _ => unreachable!("non-text loot function routed to text component handler"),
        }
        Some(stack)
    }

    fn apply_control_function(
        &self,
        mut stack: LootStack,
        context: &mut LootContext,
    ) -> Option<LootStack> {
        match self {
            Self::Reference(name) => apply_referenced_functions(name, stack, context),
            Self::ApplyExplosionDecay => {
                apply_explosion_decay(&mut stack, context);
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
            Self::Sequence(functions) => apply_function_sequence(functions, stack, context),
            _ => unreachable!("non-control loot function routed to control handler"),
        }
    }
}

fn apply_enchantment_component(
    stack: &mut LootStack,
    context: &mut LootContext,
    level: i32,
    options: &[String],
) {
    if !options.is_empty() {
        let index = context.random.next_i32(options.len() as i32) as usize;
        stack.components.insert(
            "minecraft:enchantments".to_string(),
            format!("{}:{level}", options[index]),
        );
    }
}

fn copy_entity_name_component(stack: &mut LootStack, context: &LootContext, source: &str) {
    if let Some(name) = context.entity_properties.get(source) {
        stack
            .components
            .insert("minecraft:custom_name".to_string(), name.clone());
    }
}

fn set_container_component(stack: &mut LootStack, contents: &[LootStack]) {
    stack.components.insert(
        "minecraft:container".to_string(),
        contents
            .iter()
            .map(|item| format!("{}:{}", item.item, item.count))
            .collect::<Vec<_>>()
            .join(","),
    );
}

fn set_exploration_map_components(stack: &mut LootStack, destination: &str, decoration: &str) {
    stack.item = "minecraft:filled_map".to_string();
    stack.components.insert(
        "minecraft:map_destination".to_string(),
        destination.to_string(),
    );
    stack.components.insert(
        "minecraft:map_decoration".to_string(),
        decoration.to_string(),
    );
}

fn fill_player_head_component(stack: &mut LootStack, context: &LootContext) {
    if let Some(player) = context
        .entity_properties
        .get("last_damage_player")
        .or_else(|| context.entity_properties.get("attacking_entity"))
        .or_else(|| context.entity_properties.get("killer_entity"))
    {
        stack
            .components
            .insert("minecraft:profile".to_string(), player.clone());
    }
}

fn copy_block_state_components(
    stack: &mut LootStack,
    context: &LootContext,
    properties: &[String],
) {
    for property in properties {
        if let Some(value) = context.block_state_properties.get(property) {
            stack
                .components
                .insert(format!("minecraft:block_state.{property}"), value.clone());
        }
    }
}

fn set_written_book_components(stack: &mut LootStack, title: &str, author: &str, pages: &[String]) {
    stack.components.insert(
        "minecraft:written_book_title".to_string(),
        title.to_string(),
    );
    stack.components.insert(
        "minecraft:written_book_author".to_string(),
        author.to_string(),
    );
    stack
        .components
        .insert("minecraft:written_book_pages".to_string(), pages.join("\n"));
}

fn insert_joined_component(stack: &mut LootStack, key: &str, values: &[String], separator: &str) {
    stack
        .components
        .insert(key.to_string(), values.join(separator));
}

fn apply_referenced_functions(
    name: &str,
    stack: LootStack,
    context: &mut LootContext,
) -> Option<LootStack> {
    if let Some(functions) = context.function_references.get(name).cloned() {
        apply_function_sequence(&functions, stack, context)
    } else {
        context
            .warnings
            .push(format!("Unknown loot function reference {name}"));
        Some(stack)
    }
}

fn apply_explosion_decay(stack: &mut LootStack, context: &mut LootContext) {
    if let Some(radius) = context.explosion_radius {
        if radius > 0.0 {
            let chance = 1.0 / radius;
            let kept = (0..stack.count)
                .filter(|_| context.random.next_f32() <= chance)
                .count() as i32;
            stack.count = kept;
        }
    }
}

fn apply_function_sequence(
    functions: &[LootFunction],
    stack: LootStack,
    context: &mut LootContext,
) -> Option<LootStack> {
    let mut next = Some(stack);
    for function in functions {
        next = function.apply(next?, context);
    }
    next
}
