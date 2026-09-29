use super::*;

impl LootFunction {
    pub fn apply(&self, stack: LootStack, context: &mut LootContext) -> Option<LootStack> {
        match self {
            Self::SetCount(_)
            | Self::LimitCount { .. }
            | Self::AddLootingBonus { .. }
            | Self::ApplyFortuneBonus { .. }
            | Self::ApplyBonus { .. }
            | Self::AddCount(_)
            | Self::EnchantedCountIncrease { .. }
            | Self::SetOminousBottleAmplifier(_) => self.apply_count_function(stack, context),
            Self::SetItem(_)
            | Self::SetEnchantments { .. }
            | Self::SetDamage { .. }
            | Self::SetNbt(_)
            | Self::SetComponents(_)
            | Self::EnchantWithLevels { .. }
            | Self::EnchantRandomly { .. }
            | Self::SmeltItem { .. }
            | Self::SetRandomPotion(_)
            | Self::SetRandomDyes(_) => self.apply_item_function(stack, context),
            Self::CopyName { .. }
            | Self::CopyNbt { .. }
            | Self::SetContents(_)
            | Self::ModifyContents(_)
            | Self::ExplorationMap { .. }
            | Self::FillPlayerHead
            | Self::CopyState { .. }
            | Self::CopyComponents { .. }
            | Self::SetAttributes(_) => self.apply_context_component_function(stack, context),
            Self::SetBannerPatterns(_)
            | Self::SetBookContents { .. }
            | Self::SetLore(_)
            | Self::SetName { .. }
            | Self::SetPotion(_)
            | Self::SetWrittenBookPages(_)
            | Self::SetWritableBookPages(_)
            | Self::SetBookCover { .. }
            | Self::ToggleTooltips(_)
            | Self::SetFireworkExplosions(_)
            | Self::SetFireworks { .. }
            | Self::SetCustomModelData(_)
            | Self::SetLootTable(_) => self.apply_text_component_function(stack),
            Self::SetInstrument(_) | Self::SetStewEffects(_) => {
                self.apply_random_component_function(stack, context)
            }
            Self::Reference(_)
            | Self::Discard
            | Self::ApplyExplosionDecay
            | Self::Filtered { .. }
            | Self::Unmodeled { .. }
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
            Self::ApplyBonus {
                enchantment,
                formula,
            } => {
                let level = context.enchantment_level_of(enchantment);
                stack.count = formula.apply(stack.count, level, context);
            }
            Self::AddCount(provider) => stack.count += provider.int(context),
            Self::EnchantedCountIncrease {
                enchantment,
                count,
                limit,
            } => {
                let level = context.enchantment_level_of(enchantment);
                if level != 0 {
                    let addition = level as f32 * count.float(context);
                    // `Math.round(float)`.
                    stack.count += (addition + 0.5).floor() as i32;
                    if *limit > 0 {
                        stack.count = stack.count.min(*limit);
                    }
                }
            }
            Self::SetOminousBottleAmplifier(provider) => {
                stack.components.insert(
                    "minecraft:ominous_bottle_amplifier".to_string(),
                    provider.int(context).clamp(0, 4).to_string(),
                );
            }
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
            Self::SetEnchantments {
                enchantments,
                add,
            } => return Some(item_functions::set_enchantments(stack, context, enchantments, *add)),
            Self::SetDamage { damage, add } => {
                return Some(item_functions::set_damage(stack, context, damage, *add));
            }
            Self::SetNbt(values) => {
                stack.components.extend(values.clone());
            }
            Self::SetComponents(edits) => {
                for edit in edits {
                    match &edit.value {
                        Some(value) => {
                            stack.components.insert(edit.component.clone(), value.clone());
                        }
                        None => {
                            stack.components.remove(&edit.component);
                        }
                    }
                }
            }
            Self::EnchantWithLevels {
                levels,
                options,
                include_additional_cost_component,
            } => {
                return Some(item_functions::enchant_with_levels(
                    stack,
                    context,
                    levels,
                    options.as_ref(),
                    *include_additional_cost_component,
                ));
            }
            Self::EnchantRandomly {
                options,
                only_compatible,
                include_additional_cost_component,
            } => {
                return Some(item_functions::enchant_randomly(
                    stack,
                    context,
                    options.as_ref(),
                    *only_compatible,
                    *include_additional_cost_component,
                ));
            }
            Self::SmeltItem { use_input_count } => {
                return Some(item_functions::smelt_item(stack, context, *use_input_count));
            }
            Self::SetRandomPotion(potions) => {
                if let Some(potion) = choose_random_text(potions, context) {
                    stack
                        .components
                        .insert("minecraft:potion_contents".to_string(), potion);
                }
            }
            Self::SetRandomDyes(dyes) => {
                if let Some(dye) = choose_random_text(dyes, context) {
                    stack
                        .components
                        .insert("minecraft:dyed_color".to_string(), dye);
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
            Self::ModifyContents(functions) => {
                if let Some(container) = stack.components.get("minecraft:container").cloned() {
                    let modified = container
                        .split(',')
                        .filter_map(parse_container_entry)
                        .filter_map(|item| apply_function_sequence(functions, item, context))
                        .map(|item| format!("{}:{}", item.item, item.count))
                        .collect::<Vec<_>>()
                        .join(",");
                    stack
                        .components
                        .insert("minecraft:container".to_string(), modified);
                }
            }
            Self::ExplorationMap {
                destination,
                decoration,
            } => set_exploration_map_components(&mut stack, destination, decoration),
            Self::FillPlayerHead => fill_player_head_component(&mut stack, context),
            Self::CopyState { properties, .. } => {
                return Some(item_functions::copy_block_state(stack, context, properties));
            }
            Self::CopyComponents {
                source,
                include,
                exclude,
            } => {
                return Some(item_functions::copy_components(
                    stack,
                    context,
                    *source,
                    include.as_deref(),
                    exclude.as_deref(),
                ));
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
            Self::SetLore(lines) => {
                insert_joined_component(&mut stack, "minecraft:lore", lines, "\n");
            }
            Self::SetName { name, target } => {
                // `SetNameFunction.run`: the resolver is the identity for the
                // component contents the codec accepts as modeled.
                if let Some(name) = name {
                    stack
                        .components
                        .insert(target.component().to_string(), name.clone());
                }
            }
            Self::SetPotion(potion) => {
                stack
                    .components
                    .insert("minecraft:potion_contents".to_string(), potion.clone());
            }
            Self::SetWrittenBookPages(pages) => {
                insert_joined_component(&mut stack, "minecraft:written_book_pages", pages, "\n");
            }
            Self::SetWritableBookPages(pages) => {
                insert_joined_component(&mut stack, "minecraft:writable_book_pages", pages, "\n");
            }
            Self::SetBookCover { title, author } => {
                stack
                    .components
                    .insert("minecraft:written_book_title".to_string(), title.clone());
                stack
                    .components
                    .insert("minecraft:written_book_author".to_string(), author.clone());
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
            Self::SetCustomModelData(model_data) => {
                stack
                    .components
                    .insert("minecraft:custom_model_data".to_string(), model_data.clone());
            }
            Self::SetLootTable(table) => {
                stack
                    .components
                    .insert("minecraft:container_loot_table".to_string(), table.clone());
            }
            _ => unreachable!("non-text loot function routed to text component handler"),
        }
        Some(stack)
    }

    /// Functions that draw from the context's random source to pick a component value.
    fn apply_random_component_function(
        &self,
        stack: LootStack,
        context: &mut LootContext,
    ) -> Option<LootStack> {
        Some(match self {
            Self::SetInstrument(options) => item_functions::set_instrument(stack, context, options),
            Self::SetStewEffects(effects) => item_functions::set_stew_effect(stack, context, effects),
            _ => unreachable!("non-random component function routed to random handler"),
        })
    }

    fn apply_control_function(
        &self,
        mut stack: LootStack,
        context: &mut LootContext,
    ) -> Option<LootStack> {
        match self {
            Self::Reference(name) => apply_referenced_functions(name, stack, context),
            Self::Discard => None,
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
            Self::Unmodeled { .. } => Some(stack),
            _ => unreachable!("non-control loot function routed to control handler"),
        }
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

fn parse_container_entry(value: &str) -> Option<LootStack> {
    let (item, count) = value.rsplit_once(':')?;
    count
        .parse::<i32>()
        .ok()
        .map(|count| LootStack::new(item.to_string(), count))
}

fn choose_random_text(values: &[String], context: &mut LootContext) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    let index = context.random.next_i32(values.len() as i32) as usize;
    values.get(index).cloned()
}
